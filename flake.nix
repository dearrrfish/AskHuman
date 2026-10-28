{
  description = "AskHuman — Human-in-the-loop interaction tool for AI agents (my-nixpkgs branch)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      baseVersion = (builtins.fromJSON (builtins.readFile ./package.json)).version;
      shortRev =
        if self ? shortRev then
          self.shortRev
        else if self ? dirtyShortRev then
          builtins.substring 0 7 self.dirtyShortRev
        else if self ? rev then
          builtins.substring 0 7 self.rev
        else if self ? dirtyRev then
          builtins.substring 0 7 self.dirtyRev
        else
          "dirty";

      mkPackage =
        pkgs:
        let
          inherit (pkgs) lib stdenv;
          nodejs = pkgs.nodejs_24;
          pnpm = pkgs.pnpm_10.override { nodejs-slim = nodejs; };
        in
        pkgs.rustPlatform.buildRustPackage (finalAttrs: {
          pname = "askhuman-my";
          version = "${baseVersion}+my-${shortRev}";

          src = self;

          pnpmDeps = pkgs.fetchPnpmDeps {
            inherit (finalAttrs) pname version src;
            inherit pnpm;
            fetcherVersion = 3;
            hash = "sha256-IBH1jniSjOC8/sBxuuuPEaDdgUQbkgjYhxFKhlsAR0Q=";
          };

          cargoRoot = "src-tauri";
          buildAndTestSubdir = finalAttrs.cargoRoot;
          cargoHash = "sha256-/kaKFKT0Wu9U2UY+kwNbVtzUjfONKI5BMjolLQjDZbo=";

          buildFeatures = [ "custom-protocol" ];

          # Stamp the <baseVersion>+my-<short_rev> version into manifests and lockfiles so the
          # built binary, daemon handshake, and tray report the branch commit distinctly without
          # mutating upstream's base version in git, while keeping compare_versions normalized.
          # Also patch Python fixture shebangs in the Codex Desktop runtime and model tests,
          # and verify that Git LFS icon assets were smudged before invoking tauri-build.
          postPatch = ''
            if head -c 7 src-tauri/icons/icon.png | grep -q '^version'; then
              echo "error: Git LFS files in src-tauri/icons/ are unsmudged pointers; pass '?lfs=1' in the flake URL (e.g. 'nix build .?lfs=1')" >&2
              exit 1
            fi
            substituteInPlace package.json src-tauri/tauri.conf.json \
              --replace-fail '"version": "${baseVersion}"' '"version": "${finalAttrs.version}"'
            substituteInPlace src-tauri/Cargo.toml \
              --replace-fail 'version = "${baseVersion}"' 'version = "${finalAttrs.version}"'
            substituteInPlace src-tauri/Cargo.lock "$cargoDepsCopy/Cargo.lock" \
              --replace-fail $'name = "humaninloop"\nversion = "${baseVersion}"' \
              $'name = "humaninloop"\nversion = "${finalAttrs.version}"'
            substituteInPlace src-tauri/src/codex_desktop/runtime.rs src-tauri/src/codex_desktop/models.rs \
              --replace-fail "#!/usr/bin/env python3" "#!${pkgs.python3}/bin/python3"
          '';

          # codex_desktop::runtime::tests writes and immediately execs a temporary script;
          # running it concurrently with other fork/exec tests triggers Linux ETXTBSY (os error 26).
          # Run the main suite in parallel and execute codex_desktop::runtime::tests single-threaded in postCheck.
          checkFlags = [
            "--skip=codex_desktop::runtime::tests::"
          ];

          postCheck = ''
            cargo test --release --manifest-path src-tauri/Cargo.toml \
              --target ${stdenv.hostPlatform.rust.rustcTargetSpec} \
              --features custom-protocol \
              -- codex_desktop::runtime::tests:: --test-threads=1
          '';

          nativeBuildInputs = [
            nodejs
            pnpm
            pkgs.pnpmConfigHook
            pkgs.pkg-config
            pkgs.wrapGAppsHook4
          ];

          nativeCheckInputs = [
            pkgs.git
            pkgs.python3
            pkgs.procps
          ];

          buildInputs = [
            pkgs.webkitgtk_4_1
            pkgs.gtk3
            pkgs.glib
            pkgs.cairo
            pkgs.pango
            pkgs.gdk-pixbuf
            pkgs.librsvg
            pkgs.openssl
          ];

          # The tray backend loads AppIndicator via dlopen at runtime, so expose it
          # through the wrapper's LD_LIBRARY_PATH alongside the WebKit DMA-BUF workaround.
          preFixup = ''
            gappsWrapperArgs+=(
              --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ pkgs.libayatana-appindicator ]}"
              --set WEBKIT_DISABLE_DMABUF_RENDERER 1
            )
          '';

          preBuild = ''
            pnpm run build
          '';

          meta = {
            description = "Human-in-the-loop interaction tool for AI agents (dearrrfish/AskHuman my-nixpkgs branch)";
            homepage = "https://github.com/dearrrfish/AskHuman";
            license = lib.licenses.mit;
            platforms = supportedSystems;
            mainProgram = "AskHuman";
          };
        });
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          askhuman-my = mkPackage pkgs;
        in
        {
          default = askhuman-my;
          inherit askhuman-my;
          askhuman = askhuman-my;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/AskHuman";
          meta.description = "Human-in-the-loop interaction tool for AI agents (dearrrfish/AskHuman my-nixpkgs branch)";
        };
      });

      checks = forAllSystems (system: {
        default = self.packages.${system}.default;
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          nodejs = pkgs.nodejs_24;
          pnpm = pkgs.pnpm_10.override { nodejs-slim = nodejs; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.rustc
              pkgs.cargo
              pkgs.clippy
              pkgs.rustfmt
              nodejs
              pnpm
              pkgs.pkg-config
              pkgs.git
              pkgs.python3
              pkgs.procps
              pkgs.webkitgtk_4_1
              pkgs.gtk3
              pkgs.glib
              pkgs.cairo
              pkgs.pango
              pkgs.gdk-pixbuf
              pkgs.librsvg
              pkgs.openssl
              pkgs.libayatana-appindicator
              pkgs.gsettings-desktop-schemas
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.libayatana-appindicator ];
            GSETTINGS_SCHEMAS_PATH = "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}";
          };
        }
      );

      overlays.default = final: _prev: {
        askhuman-my = mkPackage final;
      };
    };
}
