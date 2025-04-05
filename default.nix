{
  pkgs ? import <nixpkgs> { },
}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "powertray";
  version = "0.1";

  nativeBuildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    glib
    glibc
    lld
    openssl
    sqlite
    gobject-introspection
    dbus-glib
    gtk4
    gtk3
    makeWrapper
  ];

  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    glib
    glibc
    lld
    openssl
    sqlite
    gobject-introspection
    dbus-glib
    gtk4
    gtk3
  ];

  postInstall = ''
    wrapProgram $out/bin/powertray \
  '';

  cargoLock.lockFile = ./Cargo.lock;

  src = pkgs.lib.cleanSource ./.;
}
