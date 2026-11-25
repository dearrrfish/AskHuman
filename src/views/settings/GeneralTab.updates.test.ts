import { mount } from "@vue/test-utils";
import { ref } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { i18n } from "../../i18n";
import type { UpdateApplyMode } from "../../lib/types";
import GeneralTab from "./GeneralTab.vue";

const useSettingsContext = vi.hoisted(() => vi.fn());
vi.mock("./context", () => ({ useSettingsContext }));
vi.mock("../../lib/ipc", () => ({
  openSetup: vi.fn(),
  openTestPopup: vi.fn(),
  quickReplyDefaults: vi.fn().mockResolvedValue({ en: [], zh: [] }),
}));

function setup(mode: UpdateApplyMode) {
  const ctx = {
    config: ref({
      general: {
        away: { capsule: true, welcomeBack: true, pushAfterSecs: 900 },
        quickReplies: null,
      },
      channels: { popup: {} },
      experimental: {},
    }),
    setupState: ref(null),
    persist: vi.fn(),
    isMac: false, soundSupport: ref({ kind: "none" }),
    appVersion: ref("1.0.0"), updateInfo: ref({ available: true, latestVersion: "2.0.0", applyMode: mode, manualCommand: "npm i -g askhuman@latest" }),
    updateChecking: ref(false), updateApplying: ref(false), updatePreparing: ref(false),
    updateDone: ref(false), currentNotesOpen: ref(false), notesHtml: ref("Release notes"),
    applyUpdate: vi.fn(), prepareManualUpdate: vi.fn(), checkUpdate: vi.fn(), openReleases: vi.fn(),
  };
  useSettingsContext.mockReturnValue(ctx);
  const wrapper = mount(GeneralTab, { global: { plugins: [i18n] } });
  return { ctx, wrapper };
}

describe("GeneralTab update controls", () => {
  beforeEach(() => { i18n.global.locale.value = "en"; });

  it("shows Nix guidance and notes without automatic or Windows prepare controls", async () => {
    const { ctx, wrapper } = setup("manualNix");
    expect(wrapper.text()).toContain("managed by Nix");
    expect(wrapper.text()).toContain("declarative configuration or Nix profile");
    expect(wrapper.text()).toContain("Release notes");
    const updateRow = wrapper.get("#manual-update");
    expect(updateRow.find("button").exists()).toBe(false);
    expect(wrapper.text()).not.toContain("Windows automatic update");
    expect(wrapper.text()).not.toContain("Preparation waits");
    await wrapper.get(".link:last-child").trigger("click");
    expect(ctx.applyUpdate).not.toHaveBeenCalled();
    expect(ctx.prepareManualUpdate).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("keeps the automatic apply button", async () => {
    const { ctx, wrapper } = setup("automatic");
    await wrapper.get("#manual-update button").trigger("click");
    expect(ctx.applyUpdate).toHaveBeenCalledOnce();
    expect(ctx.prepareManualUpdate).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it.each(["manualDirect", "manualNpm"] as const)("keeps Windows preparation for %s", async (mode) => {
    const { ctx, wrapper } = setup(mode);
    await wrapper.get("#manual-update button").trigger("click");
    expect(ctx.prepareManualUpdate).toHaveBeenCalledOnce();
    expect(ctx.applyUpdate).not.toHaveBeenCalled();
    wrapper.unmount();
  });
});
