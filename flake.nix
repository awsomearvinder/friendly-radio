{
  description =
    "Friendly Radio is a way to send an audio stream over the network to another source.";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-21.11";
    naersk.url = "github:nmattia/naersk";
    fenix.url = "github:nix-community/fenix";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, utils, fenix }:
    utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages."${system}";
      in let
        compileDeps = [
          (pkgs.lib.getDev pkgs.alsaLib)
          (pkgs.alsaPlugins)
          (pkgs.lib.getDev pkgs.flac)
        ];
        naerskOveride = naersk.lib.${system}.override {
          inherit (fenix.packages.${system}.stable) cargo rustc;
        };
        nativeBuildInputs = [ pkgs.pkg-config ];
      in {
        defaultPackage = naerskOveride.buildPackage {
          pname = "rust template";
          version = "0.0";
          root = ./.;
          buildInputs = compileDeps;
          nativeBuildInputs = nativeBuildInputs;
          ALSA_PLUGIN_DIR = "${pkgs.alsaPlugins}/lib";
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = compileDeps ++ [
            fenix.packages.${system}.rust-analyzer
            fenix.packages.${system}.stable.rust-src
            fenix.packages.${system}.stable.rustfmt
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
          ] ++ nativeBuildInputs;
          RUST_SRC_PATH = "${
              fenix.packages.${system}.stable.rust-src
            }/lib/rustlib/src/rust/library";
          ALSA_PLUGIN_DIR = "${pkgs.alsaPlugins}/lib";
        };
      });
}
