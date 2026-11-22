{
  description = "AskHuman — human-in-the-loop interaction for AI agents";

  # Nix 2.27+ honors this for Git sources; explicit git+ URLs with ?lfs=1
  # also work when fetching with older Nix versions that support Git LFS.
  inputs.self.lfs = true;
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.home-manager = {
    url = "github:nix-community/home-manager";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
    }@inputs:
    let
      systems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
      mkPackage = pkgs: pkgs.callPackage ./nix/package.nix { source = self; };
      mkModuleDocs =
        pkgs:
        let
          inherit (pkgs) lib;
          evaluated = home-manager.lib.homeManagerConfiguration {
            inherit pkgs;
            modules = [
              self.homeManagerModules.default
              {
                home.username = "module-docs";
                home.homeDirectory = "/home/module-docs";
                home.stateVersion = "26.05";
              }
            ];
          };
          optionsDoc = pkgs.nixosOptionsDoc {
            options.programs.askhuman = evaluated.options.programs.askhuman;
            documentType = "none";
            transformOptions =
              option:
              option
              // {
                declarations = map (
                  declaration:
                  let
                    relative = lib.removePrefix "${toString ./.}/" (toString declaration);
                  in
                  {
                    name = relative;
                    url = "https://github.com/Naituw/AskHuman/blob/main/${relative}";
                  }
                ) option.declarations;
              };
          };
        in
        pkgs.runCommand "askhuman-options.md"
          {
            nativeBuildInputs = [ pkgs.mdformat ];
          }
          ''
            {
              printf '# AskHuman Home Manager options\n\n'
              printf '<!-- Generated from evaluated module options. Run nix run .#generate-module-docs to update. -->\n\n'
              cat ${optionsDoc.optionsCommonMark}
            } > "$out"
            mdformat "$out"
          '';
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          askhuman = mkPackage pkgs;
        in
        {
          inherit askhuman;
          default = askhuman;
        }
      );

      apps = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          markdown = mkModuleDocs pkgs;
          generator = pkgs.writeShellApplication {
            name = "generate-module-docs";
            runtimeInputs = [ pkgs.coreutils ];
            text = ''
              if [[ ! -f flake.nix || ! -d nix/home-manager ]]; then
                echo 'Run this command from the AskHuman repository root.' >&2
                exit 1
              fi
              install -Dm644 ${markdown} docs/nix/askhuman-options.md
            '';
          };
        in
        {
          default = {
            type = "app";
            program = "${self.packages.${system}.askhuman}/bin/AskHuman";
            meta.description = "AskHuman CLI and desktop application";
          };
          generate-module-docs = {
            type = "app";
            program = pkgs.lib.getExe generator;
            meta.description = "Regenerate the evaluated AskHuman Home Manager option reference";
          };
        }
      );

      devShells = forAllSystems (system: {
        default = (pkgsFor system).callPackage ./nix/dev-shell.nix { };
      });

      overlays.default = final: _prev: {
        askhuman =
          assert final.lib.assertMsg (builtins.elem final.stdenv.hostPlatform.system systems)
            "AskHuman's Flake supports x86_64-linux";
          mkPackage final;
      };

      homeManagerModules = {
        default = ./nix/home-manager;
        askhuman = ./nix/home-manager;
      };

      checks = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          inherit (pkgs) lib;
          askhuman = self.packages.${system}.askhuman;
          markdown = mkModuleDocs pkgs;
          python = pkgs.python3.withPackages (packages: [ packages.tomlkit ]);
          testSource = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./nix/home-manager/reconcile.py
              ./nix/home-manager/runtime.py
              ./nix/tests/test_askhuman_reconcile.py
              ./nix/tests/test_askhuman_runtime.py
              ./nix/tests/test_askhuman_activation.py
              ./nix/tests/askhuman-schema.json
            ];
          };
          activationHome = home-manager.lib.homeManagerConfiguration {
            inherit pkgs;
            modules = [
              self.homeManagerModules.default
              {
                home.username = "activation-test";
                home.homeDirectory = "/build/askhuman-activation-home";
                home.stateVersion = "26.05";
                programs.askhuman = {
                  enable = true;
                  package = pkgs.writeShellScriptBin "AskHuman" "exit 0";
                  settings.general.theme = "dark";
                };
              }
            ];
          };
          activationScript = pkgs.writeText "askhuman-activation.sh" activationHome.config.home.activation.askhuman.data;
        in
        {
          package = askhuman;
          frontend = pkgs.stdenv.mkDerivation {
            pname = "askhuman-frontend-tests";
            inherit (askhuman) version src pnpmDeps;
            nativeBuildInputs = with pkgs; [
              nodejs_24
              askhuman.pnpm
              pnpmConfigHook
              git
              python3
              rustc
            ];
            postPatch = ''
              substituteInPlace scripts/build-frontend-if-needed.node-test.mjs scripts/dev-project.node-test.mjs \
                --replace-fail '#!/usr/bin/env node' '#!${pkgs.nodejs_24}/bin/node'
            '';
            dontBuild = true;
            doCheck = true;
            checkPhase = ''
              runHook preCheck
              pnpm test
              runHook postCheck
            '';
            installPhase = ''
              touch "$out"
            '';
          };
          module-eval = import ./nix/tests/module-eval.nix { inherit inputs pkgs system; };
          module-docs = pkgs.runCommand "module-docs" { } ''
            diff -u ${./docs/nix/askhuman-options.md} ${markdown}
            mkdir -p "$out"
            cp ${markdown} "$out/askhuman-options.md"
          '';
          askhuman-reconcile =
            pkgs.runCommand "askhuman-reconcile"
              {
                nativeBuildInputs = [ pkgs.python3 ];
              }
              ''
                cp -r ${testSource} source
                chmod -R u+w source
                cd source
                python -m unittest discover -s nix/tests -p test_askhuman_reconcile.py -v
                mkdir -p "$out"
                printf 'Ordinary reconciliation tests passed.\n' > "$out/results.txt"
              '';
          askhuman-activation =
            pkgs.runCommand "askhuman-activation"
              {
                nativeBuildInputs = [ python ];
                ASKHUMAN_EXECUTABLE = "${askhuman}/bin/AskHuman";
                ASKHUMAN_ACTIVATION_SCRIPT = activationScript;
                ASKHUMAN_ACTIVATION_HOME = "/build/askhuman-activation-home";
                ASKHUMAN_ACTIVATION_STATE = "/build/askhuman-activation-home/.local/state/askhuman/home-manager";
              }
              ''
                cp -r ${testSource} source
                chmod -R u+w source
                cd source
                test -f nix/tests/test_askhuman_runtime.py
                python -m unittest discover -s nix/tests -p test_askhuman_runtime.py -v
                test -f nix/tests/test_askhuman_activation.py
                python -m unittest discover -s nix/tests -p test_askhuman_activation.py -v
                mkdir -p "$out"
                printf 'Runtime adapter and activation checks passed.\n' > "$out/results.txt"
              '';
        }
      );
    };
}
