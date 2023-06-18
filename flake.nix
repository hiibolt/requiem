{
  # Tremendous thanks to @oati for her help
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        rust_overlay = import (builtins.fetchTarball {
            url = "https://github.com/oxalica/rust-overlay/archive/master.tar.gz";
            sha256 = "sha256:1cad6b8qra5945g7ilbyqniwna4x1gm7v7lc058w3a478znn6f8s";
        });
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust_overlay ];
        };
        rustVersion = "latest";
        rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
          extensions = [
            "rust-src" # for rust-analyzer
          ];
        };
      in
      {
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = [ rust ] ++ (with pkgs; [ 
            pkg-config
            cargo
            gcc
            rustfmt
            clippy
          ]);
          buildInputs = (with pkgs; [ 
            udev
            alsa-lib
            vulkan-loader
            libxkbcommon
            wayland
          ]) ++ (with pkgs.xorg; [
            libX11
            libXcursor 
            libXrandr
            libXi
          ]);

          # Certain Rust tools won't work without this
          # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
          # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
    });
}
