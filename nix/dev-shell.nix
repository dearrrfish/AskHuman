{ pkgs }:
let
  env = import ./environment.nix { inherit pkgs; };
in
pkgs.mkShell (
  {
    nativeBuildInputs =
      env.nativeBuildInputs
      ++ env.checkInputs
      ++ (with pkgs; [
        rustc
        cargo
        clippy
        rustfmt
        cargo-audit
        env.nodejs
        env.pnpm
      ]);
    buildInputs = env.buildInputs;
  }
  // env.variables
)
