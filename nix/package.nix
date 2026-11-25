{
  lib,
  pkgs,
  stdenv,
  rustPlatform,
  source,
}:
let
  sourcePath =
    if builtins.isPath source then
      source
    else
      /. + builtins.unsafeDiscardStringContext source.outPath;
  env = import ./environment.nix { inherit pkgs; };
  version = (builtins.fromJSON (builtins.readFile (sourcePath + "/package.json"))).version;
  # Exclude nix/, unrelated docs/, and local developer outputs so editing them does not invalidate builds.
  src = lib.fileset.toSource {
    root = sourcePath;
    fileset = lib.fileset.unions (
      map (path: sourcePath + "/${path}") [
        "package.json"
        "pnpm-lock.yaml"
        "tsconfig.json"
        "vite.config.ts"
        "vite.attachment.config.ts"
        "vite.annotations.config.ts"
        "vitest.config.ts"
        "src"
        "scripts"
        ".agents/skills/ios-testflight/scripts"
        "docs/public/ios"
        "docs/wiki/ios-broker.md"
        "docs/wiki/ios-broker.en.md"
        "docs/wiki/ios-setup.md"
        "docs/wiki/ios-setup.en.md"
        "ios/AskHuman/Resources/Localizable.xcstrings"
        "ios/AskHuman/Resources/WebMarkdown"
        "ios/Fixtures"
        "ios/Tools/agent-logos"
        "ios/project.yml"
        "todo-sync-helper/Fixtures"
        "todo-sync-helper/scripts"
        "src-tauri/Cargo.toml"
        "src-tauri/Cargo.lock"
        "src-tauri/build.rs"
        "src-tauri/build"
        "src-tauri/tauri.conf.json"
        "src-tauri/Info.plist"
        "src-tauri/capabilities"
        "src-tauri/crates"
        "src-tauri/icons"
        "src-tauri/resources"
        "src-tauri/src"
        "src-tauri/swift"
        "src-tauri/tests"
        "src-tauri/vendor"
        "src-tauri/examples"
      ]
    );
  };
in
rustPlatform.buildRustPackage {
  pname = "askhuman";
  inherit version src;

  pnpmDeps = pkgs.fetchPnpmDeps {
    pname = "askhuman";
    inherit version;
    src = lib.fileset.toSource {
      root = sourcePath;
      fileset = lib.fileset.unions [
        (sourcePath + "/package.json")
        (sourcePath + "/pnpm-lock.yaml")
      ];
    };
    inherit (env) pnpm;
    fetcherVersion = 3;
    hash = "sha256-rLb4NQPilzeMyaxLenxIdj760a6IJlpBlrsFy6sb3q8=";
  };

  cargoRoot = "src-tauri";
  buildAndTestSubdir = "src-tauri";
  # Derive vendored crate downloads directly from Cargo.lock checksums so upstream
  # lockfile updates and release version bumps do not trigger cargoHash drift.
  cargoLock.lockFile = sourcePath + "/src-tauri/Cargo.lock";
  buildFeatures = [ "custom-protocol" ];

  # Only immutable package builds carry this marker; local devShell installs do not.
  env = env.variables // {
    ASKHUMAN_NIX_STORE_DIR = builtins.storeDir;
  };

  nativeBuildInputs =
    env.nativeBuildInputs
    ++ [
      env.nodejs
      env.pnpm
      pkgs.pnpmConfigHook
      pkgs.wrapGAppsHook4
    ];
  buildInputs = env.buildInputs;
  nativeCheckInputs = env.checkInputs;

  postPatch = ''
    ${pkgs.python3}/bin/python3 ${./check-lfs.py}
    substituteInPlace src-tauri/src/codex_desktop/runtime.rs src-tauri/src/codex_desktop/models.rs src-tauri/src/agents/codex_cli.rs \
      --replace-fail '#!/usr/bin/env python3' '#!${pkgs.python3}/bin/python3'
    substituteInPlace src-tauri/src/daemon/runtime/inbox.rs src-tauri/src/daemon/runtime/inbox/tests.rs \
      --replace-fail '/bin/sleep' '${pkgs.coreutils}/bin/sleep'
  '';

  preBuild = ''
    pnpm run build
  '';

  doCheck = true;

  # codex_desktop::runtime::tests and antigravity_desktop::sidecar_run::tests write and
  # immediately exec temporary scripts; running them concurrently with other fork/exec
  # tests triggers Linux ETXTBSY (os error 26). Run the main suite in parallel and
  # execute both modules single-threaded in postCheck.
  checkFlags = [
    "--skip=codex_desktop::runtime::tests::"
    "--skip=antigravity_desktop::sidecar_run::tests::"
  ];

  postCheck = ''
    cargo test --release --manifest-path src-tauri/Cargo.toml \
      --target ${stdenv.hostPlatform.rust.rustcTargetSpec} \
      --features custom-protocol \
      -- codex_desktop::runtime::tests:: --test-threads=1
    cargo test --release --manifest-path src-tauri/Cargo.toml \
      --target ${stdenv.hostPlatform.rust.rustcTargetSpec} \
      --features custom-protocol \
      -- antigravity_desktop::sidecar_run::tests:: --test-threads=1
  '';

  # The tray backend loads AppIndicator via dlopen at runtime, so expose it
  # through the wrapper's LD_LIBRARY_PATH alongside the WebKit DMA-BUF workaround.
  preFixup = ''
    gappsWrapperArgs+=(
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ pkgs.libayatana-appindicator ]}"
      --set WEBKIT_DISABLE_DMABUF_RENDERER 1
    )
  '';

  postFixup = ''
    mkdir -p "$out/share/askhuman"
    test -x "$out/bin/AskHuman"
    test -x "$out/bin/.AskHuman-wrapped"
    printf '%s\n' '{"version":1,"family":"askhuman","wrapper":"bin/AskHuman","image":"bin/.AskHuman-wrapped"}' > "$out/share/askhuman/entry-v1.json"
  '';

  passthru = { inherit (env) pnpm; };

  meta = {
    description = "Human-in-the-loop interaction tool for AI agents";
    homepage = "https://github.com/Naituw/AskHuman";
    license = lib.licenses.mit;
    mainProgram = "AskHuman";
    platforms = [ "x86_64-linux" ];
  };
}
