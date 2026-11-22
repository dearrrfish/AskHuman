{ pkgs }:
let
  inherit (pkgs) lib;
  nodejs = pkgs.nodejs_24;
  pnpm = pkgs.pnpm_10.override { nodejs-slim = nodejs; };
  schemaPath = lib.concatStringsSep ":" (
    map (p: "${p}/share/gsettings-schemas/${p.name}") [
      pkgs.gsettings-desktop-schemas
      pkgs.gtk3
    ]
  );
in
{
  inherit nodejs pnpm;
  buildInputs = with pkgs; [
    webkitgtk_4_1
    gtk3
    glib
    cairo
    pango
    gdk-pixbuf
    librsvg
    openssl
    sqlite
    libayatana-appindicator
    gsettings-desktop-schemas
  ];
  nativeBuildInputs = [ pkgs.pkg-config ];
  checkInputs = with pkgs; [
    git
    python3
    procps
  ];
  variables = {
    GSETTINGS_SCHEMAS_PATH = schemaPath;
    LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.libayatana-appindicator ];
  };
}
