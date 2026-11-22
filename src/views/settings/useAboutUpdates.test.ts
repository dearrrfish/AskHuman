import { mount } from "@vue/test-utils";
import { defineComponent } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { i18n } from "../../i18n";
import type { UpdateApplyMode, UpdateInfo } from "../../lib/types";
import { useAboutUpdates } from "./useAboutUpdates";

const ipc = vi.hoisted(() => ({
  getAppVersion: vi.fn(), openPath: vi.fn(), restartSettings: vi.fn(),
  updateApply: vi.fn(), updateCheck: vi.fn(), updateGetNotes: vi.fn(),
  updateGetVersionNotes: vi.fn(), updatePrepare: vi.fn(),
}));
vi.mock("../../lib/ipc", () => ipc);
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => vi.fn()) }));
vi.mock("../../lib/markdown", () => ({
  markdownReady: Promise.resolve(), renderMarkdown: (text: string) => text,
  handleCodeCopyClick: vi.fn(() => false),
}));

export function updateInfo(applyMode: UpdateApplyMode): UpdateInfo {
  return {
    available: true, currentVersion: "1.0.0", latestVersion: "2.0.0",
    releaseNotes: "Notes", sourceUrl: "https://example.invalid/release",
    isNpm: applyMode === "manualNpm", applyMode,
    manualCommand: applyMode === "manualNpm" ? "npm i -g askhuman@latest" : "",
  };
}

function setup() {
  let updates!: ReturnType<typeof useAboutUpdates>;
  const wrapper = mount(defineComponent({
    setup() { updates = useAboutUpdates(); return () => null; },
  }), { global: { plugins: [i18n] } });
  return { updates, wrapper };
}

describe("settings update capabilities", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    ipc.updateGetNotes.mockResolvedValue("Release notes");
    i18n.global.locale.value = "en";
  });

  it("checks Nix releases and notes but never applies or prepares them", async () => {
    ipc.updateCheck.mockResolvedValue(updateInfo("manualNix"));
    const confirm = vi.spyOn(window, "confirm");
    const { updates, wrapper } = setup();
    await updates.checkUpdate(true);
    expect(updates.updateInfo.value?.applyMode).toBe("manualNix");
    expect(updates.notesHtml.value).toBe("Release notes");
    await updates.applyUpdate();
    await updates.prepareManualUpdate();
    expect(ipc.updateApply).not.toHaveBeenCalled();
    expect(ipc.updatePrepare).not.toHaveBeenCalled();
    expect(ipc.openPath).not.toHaveBeenCalled();
    expect(confirm).not.toHaveBeenCalled();
    expect(updates.updateDone.value).toBe(false);
    wrapper.unmount();
    confirm.mockRestore();
  });

  it("keeps Direct/npm automatic updates", async () => {
    const { updates, wrapper } = setup();
    for (const isNpm of [false, true]) {
      updates.updateInfo.value = { ...updateInfo("automatic"), isNpm };
      await updates.applyUpdate();
      expect(updates.updateDone.value).toBe(true);
    }
    expect(ipc.updateApply).toHaveBeenCalledTimes(2);
    wrapper.unmount();
  });

  it("keeps Windows manual Direct preparation", async () => {
    const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
    const { updates, wrapper } = setup();
    updates.updateInfo.value = updateInfo("manualDirect");
    await updates.prepareManualUpdate();
    expect(ipc.openPath).toHaveBeenCalledWith("https://example.invalid/release");
    expect(ipc.updatePrepare).toHaveBeenCalledOnce();
    expect(ipc.updateApply).not.toHaveBeenCalled();
    wrapper.unmount();
    confirm.mockRestore();
  });
});
