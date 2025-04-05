{
  description = "Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        supportedSystems = [ "x86_64-linux" ];
        forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
        pkgsFor = nixpkgs.legacyPackages;
        pkgs = nixpkgs.legacyPackages.${system};

        # Read the file relative to the flake's root
        overrides = (builtins.fromTOML (builtins.readFile (self + "/rust-toolchain.toml")));
        libPath =
          with pkgs;
          lib.makeLibraryPath [
            # load external libraries that you need in your rust project here
          ];
      in
      {
        packages = forAllSystems (system: {
          default = pkgsFor.${system}.callPackage ./. { };
        });

        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            lld
            pkg-config
            rust-analyzer
          ];
          # LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          packages = with pkgs; [
            clang
            pango
            cairo
            atkmm
            gdk-pixbuf
            xdotool
            libayatana-appindicator
            libappindicator-gtk3
            libappindicator
            glib
            gtk3
            llvmPackages.bintools
            rustup
          ];

          RUSTC_VERSION = overrides.toolchain.channel;

          # https://github.com/rust-lang/rust-bindgen#environment-variables
          LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

          shellHook = ''
            export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
            export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
          '';

          # Add precompiled library to rustc search path
          RUSTFLAGS = (
            builtins.map (a: ''-L ${a}/lib'') [
              # add libraries here (e.g. pkgs.libvmi)
            ]
          );

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (packages ++ nativeBuildInputs);

          # Add glibc, clang, glib, and other headers to bindgen search path
          BINDGEN_EXTRA_CLANG_ARGS =
            # Includes normal include path
            (builtins.map (a: ''-I"${a}/include"'') [
              # add dev libraries here (e.g. pkgs.libvmi.dev)
              pkgs.glibc.dev
            ])
            # Includes with special directory paths
            ++ [
              ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
              ''-I"${pkgs.glib.dev}/include/glib-2.0"''
              ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
            ];
        };
      }
    );
}
