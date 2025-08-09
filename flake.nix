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
          config = {
            allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) ["cursor"];
          };
        };
        wit-bindgen-go = pkgs.stdenv.mkDerivation {
          pname = "wit-bindgen-go";
          version = "0.16.0";
          src = pkgs.fetchurl {
            url = "https://github.com/bytecodealliance/go-modules/releases/download/v0.7.0/wit-bindgen-go-linux-amd64.tgz";
            sha256 = "sha256-X5VbidEXgEiynKfNih3XbMeSzXYCNGjfCOkENiI2hR8=";
          };
          dontUnpack = true;

          installPhase = ''
            mkdir -p $out/bin
            tar -xf $src -C $out/bin
            chmod +x $out/bin/wit-bindgen-go
          '';
        };
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        commonPackages = [
          toolchain
          pkgs.rust-analyzer-unwrapped
          pkgs.rustup
          pkgs.rustfmt
          pkgs.cargo-component
          pkgs.wasm-tools
          pkgs.wkg
          pkgs.wasmtime
          pkgs.fermyon-spin
          pkgs.gh
          pkgs.zsh
          pkgs.nodejs_20
          pkgs.nodePackages.npm
          pkgs.direnv
          pkgs.code-cursor
          pkgs.tinygo
          pkgs.go
          wit-bindgen-go
        ];
      in {
        devShells.default = pkgs.mkShell {
          packages = commonPackages;
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          shellHook = ''
            export AVANTE_ANTHROPIC_API_KEY=$(op item get "Claude.ai API Key" --vault Private --fields label=key --format json | jq -r '.value')
            export AVANTE_AZURE_OPENAI_API_KEY=$(op item get "Azure OpenAI API Key" --vault Private --fields label=key --format json | jq -r '.value')
            export SHELL="${pkgs.zsh}/bin/zsh"
            exec $SHELL
          '';
        };

        devShells.js-deps = pkgs.mkShell {
          packages = commonPackages;
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          shellHook = ''
            pushd api-js
            npm install
            export PATH="$PWD/node_modules/.bin:$PATH"
            popd
            export SHELL="${pkgs.zsh}/bin/zsh"
            exec $SHELL
          '';
        };
      }
    );
}
