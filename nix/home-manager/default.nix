{
  config,
  lib,
  pkgs,
  options,
  ...
}:
let
  inherit (lib) mkOption types;
  cfg = config.programs.askhuman;
  runtimePath = types.addCheck types.str (
    value: lib.hasPrefix "/" value && value != "/nix/store" && !(lib.hasPrefix "/nix/store/" value)
  );
  secret =
    name:
    mkOption {
      type = types.nullOr runtimePath;
      default = null;
      description = "Absolute runtime path to ${name}, outside the Nix store. Contents are read only during real activation, trimmed by upstream, and may fall back to private plaintext config if the keychain is unavailable. Null preserves existing credentials. Do not use a Nix path literal.";
      example = "/run/secrets/askhuman-token";
    };
  hyprlandSupported = lib.hasAttrByPath [ "wayland" "windowManager" "hyprland" "settings" ] options;
  guiHostServiceEnabled =
    if cfg.services.guiHost.enable != null then
      cfg.services.guiHost.enable
    else
      (cfg.settings.general.menuBarIcon == "always");
  daemonServiceEnabled =
    if cfg.services.daemon.enable != null then
      cfg.services.daemon.enable
    else
      (cfg.settings.general.daemonLifecycle == "keepalive");
in
{
  imports = [
    ./settings.nix
    ./integrations.nix
    ./activation.nix
  ];
  options.programs.askhuman = {
    enable = lib.mkEnableOption "AskHuman with sparse writable configuration management";
    package = mkOption {
      type = types.package;
      default = pkgs.callPackage ../package.nix { source = ../..; };
      defaultText = lib.literalExpression "pkgs.callPackage <askhuman>/nix/package.nix { source = <askhuman>; }";
      description = "AskHuman package providing bin/AskHuman. The default currently supports x86_64-linux; use a compatible override on other platforms. No overlay is required.";
    };
    secretFiles = mkOption {
      default = { };
      description = "Runtime secret source paths. Neither secret values nor source files are copied to the Nix store. Omission preserves existing credentials; no secret rollback or deletion is performed.";
      type = types.submodule {
        options.channels = {
          telegram.botToken = secret "Telegram bot token";
          dingding.clientSecret = secret "DingTalk client secret";
          feishu.appSecret = secret "Feishu app secret";
          slack.botToken = secret "Slack bot token";
          slack.appToken = secret "Slack app token";
        };
      };
    };
    services = {
      guiHost.enable = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Whether to manage a declarative systemd user service (askhuman-guihost) on Linux. When null, enables automatically if programs.askhuman.settings.general.menuBarIcon is set to \"always\".";
      };
      daemon.enable = mkOption {
        type = types.nullOr types.bool;
        default = null;
        description = "Whether to manage a declarative systemd user service (askhuman-daemon) on Linux. When null, enables automatically if programs.askhuman.settings.general.daemonLifecycle is set to \"keepalive\".";
      };
    };
    compatibility = {
      webKitSessionEnvironment = lib.mkEnableOption "the session-wide WEBKIT_DISABLE_DMABUF_RENDERER=1 workaround (the package wrapper already supplies it for wrapped launches)";
      hyprland.enable = lib.mkEnableOption "an AskHuman floating-window rule in the existing Home Manager Hyprland configuration";
    };
  };
  config = lib.mkMerge [
    {
      assertions = [
        {
          assertion = !cfg.enable || lib.meta.availableOn pkgs.stdenv.hostPlatform cfg.package;
          message = "programs.askhuman.package is unavailable on ${pkgs.stdenv.hostPlatform.system}; provide a compatible package override or disable AskHuman.";
        }
        {
          assertion = !cfg.enable || !cfg.compatibility.hyprland.enable || hyprlandSupported;
          message = "programs.askhuman.compatibility.hyprland.enable requires Home Manager Hyprland settings support.";
        }
        {
          assertion =
            !cfg.enable
            || !cfg.compatibility.hyprland.enable
            || (config.wayland.windowManager.hyprland.enable or false);
          message = "programs.askhuman.compatibility.hyprland.enable requires wayland.windowManager.hyprland.enable.";
        }
      ];
    }
    (lib.mkIf cfg.enable {
      home.packages = [ cfg.package ];
      home.sessionVariables =
        lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          ASKHUMAN_MANAGED_AUTOSTART = "1";
        }
        // lib.optionalAttrs cfg.compatibility.webKitSessionEnvironment {
          WEBKIT_DISABLE_DMABUF_RENDERER = "1";
        };
    })
    (lib.mkIf (cfg.enable && pkgs.stdenv.hostPlatform.isLinux && guiHostServiceEnabled) {
      systemd.user.services.askhuman-guihost = {
        Unit = {
          Description = "AskHuman Menu Bar and GUI Host";
          After = [ "graphical-session.target" ];
          PartOf = [ "graphical-session.target" ];
          # Daemon live-update / handoff relies on AskHuman's internal draining and replacement
          # flow rather than restarting mid-session on store switch.
          X-RestartIfChanged = false;
        };
        Service = {
          ExecStart = "${cfg.package}/bin/AskHuman --gui-host";
          Environment = [ "ASKHUMAN_MANAGED_AUTOSTART=1" ];
          Restart = "always";
          RestartSec = "3s";
        };
        Install = {
          WantedBy = [ "graphical-session.target" ];
        };
      };
    })
    (lib.mkIf (cfg.enable && pkgs.stdenv.hostPlatform.isLinux && daemonServiceEnabled) {
      systemd.user.services.askhuman-daemon = {
        Unit = {
          Description = "AskHuman Background Daemon";
          After = [ "graphical-session.target" ];
          PartOf = [ "graphical-session.target" ];
          X-RestartIfChanged = false;
        };
        Service = {
          ExecStart = "${cfg.package}/bin/AskHuman daemon run";
          Environment = [ "ASKHUMAN_MANAGED_AUTOSTART=1" ];
          Restart = "on-failure";
          RestartSec = "3s";
        };
        Install = {
          WantedBy = [ "graphical-session.target" ];
        };
      };
    })
    (lib.mkIf (cfg.enable && cfg.compatibility.hyprland.enable) {
      wayland.windowManager.hyprland.settings.window_rule = lib.mkAfter [
        {
          match.class = "^(\\.)?AskHuman(-wrapped)?$";
          float = true;
        }
      ];
    })
  ];
}
