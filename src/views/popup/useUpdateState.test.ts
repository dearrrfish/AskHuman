import { computed } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PushedUpdateState } from "../../lib/types";
import { useUpdateState } from "./useUpdateState";

const ipc = vi.hoisted(() => ({
  openSettings: vi.fn(), popupUpdateState: vi.fn(), updateApply: vi.fn(), updateGetNotes: vi.fn(),
}));
const events = vi.hoisted(() => ({ callback: null as null | ((event: { payload: PushedUpdateState }) => void) }));
vi.mock("../../lib/ipc", () => ipc);
vi.mock("vue-i18n", () => ({ useI18n: () => ({ t: (key: string) => key }) }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_name, callback) => { events.callback = callback; return vi.fn(); }),
}));
vi.mock("../../lib/markdown", () => ({
  markdownReady: Promise.resolve(), renderMarkdown: (text: string) => text,
}));

function setup() {
  return useUpdateState({ codeCopyLabels: computed(() => ({ copyLabel: "Copy", copiedLabel: "Copied" })) });
}

describe("popup update capabilities", () => {
  beforeEach(() => { vi.clearAllMocks(); events.callback = null; });

  it("routes Nix update actions to settings and still loads release notes", async () => {
    ipc.popupUpdateState.mockResolvedValue({ available: true, pending: false, latestVersion: "2.0.0", applyMode: "manualNix" });
    ipc.updateGetNotes.mockResolvedValue("Release notes");
    const state = setup();
    await state.initUpdateState();
    await state.toggleUpdatePopover();
    expect(state.updateNotesHtml.value).toBe("Release notes");
    await state.applyUpdateFromPopup();
    expect(ipc.openSettings).toHaveBeenCalledWith("general#manual-update");
    expect(ipc.updateApply).not.toHaveBeenCalled();
    expect(state.updateStarted.value).toBe(false);
    state.disposeUpdateState();
  });

  it("applies automatic updates and honors a later Nix snapshot", async () => {
    ipc.popupUpdateState.mockResolvedValue({ available: true, pending: false, latestVersion: "2.0.0", applyMode: "automatic" });
    const state = setup();
    await state.initUpdateState();
    await state.applyUpdateFromPopup();
    expect(ipc.updateApply).toHaveBeenCalledOnce();
    events.callback?.({ payload: { available: true, pending: false, latestVersion: "2.0.0", applyMode: "manualNix" } });
    await state.applyUpdateFromPopup();
    expect(ipc.updateApply).toHaveBeenCalledOnce();
    expect(ipc.openSettings).toHaveBeenCalledOnce();
    state.disposeUpdateState();
  });
});
