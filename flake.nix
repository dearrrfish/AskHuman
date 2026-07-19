{
  description = "AskHuman development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          pkgs.cargo
          pkgs.rustc
          pkgs.pkg-config
        ];
        buildInputs = [
          pkgs.dbus
          pkgs.glib
          pkgs.gtk3
          pkgs.webkitgtk_4_1
        ];
      };
    };
}
