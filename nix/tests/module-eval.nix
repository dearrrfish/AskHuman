{
  inputs,
  pkgs,
  system,
}:
let
  inherit (pkgs) lib;
  fixturePackage =
    pkgs.runCommand "askhuman-module-fixture"
      {
        meta = {
          platforms = lib.platforms.all;
          mainProgram = "AskHuman";
        };
      }
      ''
        mkdir -p "$out/bin"
        printf '#!/bin/sh\nexit 0\n' > "$out/bin/AskHuman"
        chmod +x "$out/bin/AskHuman"
      '';
  standalonePkgs = import inputs.nixpkgs { inherit system; };
  homeUsing =
    module: extraModules:
    inputs.home-manager.lib.homeManagerConfiguration {
      pkgs = standalonePkgs;
      modules = [
        module
        {
          home.username = "module-test";
          home.homeDirectory = "/home/module-test";
          home.stateVersion = "26.05";
        }
      ]
      ++ extraModules;
    };
  home = homeUsing inputs.self.homeManagerModules.default;
  configured =
    extra:
    home (
      [
        {
          programs.askhuman = {
            enable = true;
            package = fixturePackage;
          };
        }
      ]
      ++ extra
    );
  evaluates = configuration: (builtins.tryEval configuration.activationPackage.drvPath).success;
  rejects = module: !(evaluates (configured [ module ]));
  disabled = home [ { programs.askhuman.package = throw "Disabled import forced the package"; } ];
  enabled = configured [ { programs.askhuman.settings.general.theme = "dark"; } ];
  merged = configured [
    { programs.askhuman.settings.general.theme = lib.mkDefault "light"; }
    { programs.askhuman.settings.general.theme = "dark"; }
    { programs.askhuman.settings.general.theme = lib.mkForce "system"; }
    { programs.askhuman.settings.general.popupPrewarm = false; }
    { programs.askhuman.settings.channels.popup.width = 600; }
  ];
  inventory = builtins.fromJSON (builtins.readFile ./askhuman-schema.json);
  optionPaths =
    type:
    builtins.sort builtins.lessThan (
      map (option: lib.concatStringsSep "." option.loc) (
        builtins.filter (option: builtins.head option.loc != "_module") (
          lib.collect lib.isOption (type.getSubOptions [ ])
        )
      )
    );
  withDefaultPackage = home [ { programs.askhuman.enable = true; } ];
  compatibility = configured [
    {
      programs.askhuman.compatibility.hyprland.enable = true;
      programs.askhuman.compatibility.webKitSessionEnvironment = true;
      wayland.windowManager.hyprland.enable = true;
    }
  ];
  servicesGuiOnly = configured [ { programs.askhuman.settings.general.menuBarIcon = "always"; } ];
  servicesDaemonOnly = configured [ { programs.askhuman.settings.general.daemonLifecycle = "keepalive"; } ];
  servicesBothInactive = configured [
    {
      programs.askhuman.settings.general.menuBarIcon = "active";
      programs.askhuman.settings.general.daemonLifecycle = "activity";
    }
  ];
  servicesOverride = configured [
    {
      programs.askhuman.settings.general.menuBarIcon = "always";
      programs.askhuman.services.guiHost.enable = false;
      programs.askhuman.services.daemon.enable = true;
    }
  ];
  unsupportedPackage = fixturePackage.overrideAttrs (_: {
    meta.platforms = [ ];
  });
  overlayPkgs = import inputs.nixpkgs {
    inherit system;
    overlays = [ inputs.self.overlays.default ];
  };
  results = {
    disabledDoesNotForcePackage = evaluates disabled;
    standalonePackageOverride =
      evaluates enabled && builtins.elem fixturePackage enabled.config.home.packages;
    mergedOptions =
      evaluates merged
      && merged.config.programs.askhuman.settings.general.theme == "system"
      && merged.config.programs.askhuman.settings.general.popupPrewarm == false
      && merged.config.programs.askhuman.settings.channels.popup.width == 600;
    namedModuleExport = evaluates (homeUsing inputs.self.homeManagerModules.askhuman [ ]);
    overlayExportsAskHuman =
      overlayPkgs ? askhuman && builtins.isString overlayPkgs.askhuman.drvPath;
    defaultNativeIntegrationsDisabled = lib.all (entry: !entry.enable) (
      builtins.attrValues enabled.config.programs.askhuman.integrations
    );
    integerBounds = rejects { programs.askhuman.settings.general.historyLimit = -1; };
    completeSettingsInventory =
      optionPaths enabled.options.programs.askhuman.settings.type
      == builtins.sort builtins.lessThan inventory.settings;
    completeSecretsInventory =
      optionPaths enabled.options.programs.askhuman.secretFiles.type
      == builtins.sort builtins.lessThan inventory.secretFiles;
    defaultPackagePlatform =
      if system == "x86_64-linux" then evaluates withDefaultPackage else !(evaluates withDefaultPackage);
    managedAutostartSessionVar =
      if pkgs.stdenv.hostPlatform.isLinux then
        evaluates enabled && enabled.config.home.sessionVariables.ASKHUMAN_MANAGED_AUTOSTART == "1"
      else
        true;
    servicesDefaultFromSettings =
      if pkgs.stdenv.hostPlatform.isLinux then
        evaluates enabled
        && !(enabled.config.systemd.user.services ? askhuman-guihost)
        && !(enabled.config.systemd.user.services ? askhuman-daemon)
        && evaluates servicesGuiOnly
        && (servicesGuiOnly.config.systemd.user.services ? askhuman-guihost)
        && !(servicesGuiOnly.config.systemd.user.services ? askhuman-daemon)
        && servicesGuiOnly.config.systemd.user.services.askhuman-guihost.Service.ExecStart
        == [ "${fixturePackage}/bin/AskHuman --gui-host" ]
        && servicesGuiOnly.config.systemd.user.services.askhuman-guihost.Unit.X-RestartIfChanged
        == false
        && evaluates servicesDaemonOnly
        && !(servicesDaemonOnly.config.systemd.user.services ? askhuman-guihost)
        && (servicesDaemonOnly.config.systemd.user.services ? askhuman-daemon)
        && servicesDaemonOnly.config.systemd.user.services.askhuman-daemon.Service.ExecStart
        == [ "${fixturePackage}/bin/AskHuman daemon run" ]
        && servicesDaemonOnly.config.systemd.user.services.askhuman-daemon.Unit.X-RestartIfChanged
        == false
        && evaluates servicesBothInactive
        && !(servicesBothInactive.config.systemd.user.services ? askhuman-guihost)
        && !(servicesBothInactive.config.systemd.user.services ? askhuman-daemon)
      else
        true;
    servicesExplicitOverride =
      if pkgs.stdenv.hostPlatform.isLinux then
        evaluates servicesOverride
        && !(servicesOverride.config.systemd.user.services ? askhuman-guihost)
        && (servicesOverride.config.systemd.user.services ? askhuman-daemon)
        && builtins.elem "ASKHUMAN_MANAGED_AUTOSTART=1"
          servicesOverride.config.systemd.user.services.askhuman-daemon.Service.Environment
      else
        true;
    compatibilityOptIn =
      if pkgs.stdenv.hostPlatform.isLinux then
        evaluates compatibility
        && compatibility.config.home.sessionVariables.WEBKIT_DISABLE_DMABUF_RENDERER == "1"
        && builtins.length compatibility.config.wayland.windowManager.hyprland.settings.window_rule > 0
      else
        true;
    compatibilityRequiresHyprland = rejects { programs.askhuman.compatibility.hyprland.enable = true; };
    defaultDisabled = !(home [ ]).config.programs.askhuman.enable;
    wrongType = rejects { programs.askhuman.settings.general.popupPrewarm = "false"; };
    invalidEnum = rejects { programs.askhuman.settings.general.theme = "invalid-theme"; };
    unknownSetting = rejects { programs.askhuman.settings.general.nonexistent = true; };
    secretInSettings = rejects { programs.askhuman.settings.channels.telegram.botToken = "secret"; };
    relativeSecret = rejects {
      programs.askhuman.secretFiles.channels.telegram.botToken = "relative/token";
    };
    storeSecret = rejects {
      programs.askhuman.secretFiles.channels.telegram.botToken = "/nix/store/token";
    };
    invalidGrokMode = rejects {
      programs.askhuman.integrations.grok = {
        enable = true;
        mode = "cli";
      };
    };
    invalidPiMode = rejects {
      programs.askhuman.integrations.pi = {
        enable = true;
        mode = "mcp";
      };
    };
    invalidPermission = rejects { programs.askhuman.integrations.cursor.permission = true; };
    invalidAntigravityPermission = rejects {
      programs.askhuman.integrations.antigravity.permission = true;
    };
    codexDelegatedEvaluates = evaluates (configured [
      {
        programs.askhuman.integrations.codex = {
          enable = true;
          delegated = true;
        };
      }
    ]);
    antigravityModesEvaluate =
      evaluates (configured [
        {
          programs.askhuman.integrations.antigravity = {
            enable = true;
            mode = "cli";
            lifecycle = true;
            stop = true;
          };
        }
      ])
      && evaluates (configured [
        {
          programs.askhuman.integrations.antigravity = {
            enable = true;
            mode = "mcp";
            lifecycle = true;
            stop = true;
          };
        }
      ]);
    unavailablePackage = rejects { programs.askhuman.package = lib.mkForce unsupportedPackage; };
    mcpEnvironmentOnlyInMcp = rejects {
      programs.askhuman.integrations.codex = {
        enable = true;
        mode = "cli";
        mcpEnvironment.TEST = "value";
      };
    };
  };
  failures = builtins.attrNames (lib.filterAttrs (_: passed: !passed) results);
in
assert lib.assertMsg (
  failures == [ ]
) "Module evaluation failed: ${lib.concatStringsSep ", " failures}";
pkgs.runCommand "module-eval"
  {
    evidence = builtins.toJSON {
      inherit system results;
      homeManagerActivation = builtins.unsafeDiscardStringContext enabled.activationPackage.drvPath;
    };
  }
  ''
    mkdir -p "$out"
    printf '%s\n' "$evidence" > "$out/results.json"
  ''
