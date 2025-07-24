# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in {
        devShells.default = with pkgs;
          mkShell {
            packages = [
              toolchain
              pkgs.rust-analyzer-unwrapped
              pkgs.rustup
              pkgs.rustfmt
              pkgs.cargo-component
              pkgs.wasm-tools
              pkgs.fermyon-spin
              pkgs.gh
              pkgs.zsh
            ];
            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
            OPENSSL_LIB_DIR = "${openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
            OPENSSL_NO_VENDOR = "1";
            shellHook = ''
              exec zsh
            '';
          };
      }
    );
}
