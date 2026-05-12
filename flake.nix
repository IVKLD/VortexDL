{
  description = "VortexDL - High-performance SoundCloud downloader";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (pkgs.lib.hasSuffix ".rs" path) ||
            (pkgs.lib.hasSuffix ".toml" path) ||
            (pkgs.lib.hasSuffix ".lock" path) ||
            (pkgs.lib.hasInfix "/frontend/dist/" path) ||
            (pkgs.lib.hasSuffix "build.rs" path) ||
            (craneLib.filterCargoSources path type);
        };

        frontendDist = pkgs.buildNpmPackage {
          pname = "vortex-dl-frontend";
          version = "0.2.0";
          src = ./frontend;
          
          npmDepsHash = "sha256-vS1y5pDqE1K7YI8Z5pE1K7YI8Z5pE1K7YI8Z5pE1K7Y="; 
          
          makeCacheWritable = true;
          npmBuildScript = "build";
          
          installPhase = ''
            mkdir -p $out
            cp -r dist/voltexdl/browser/* $out/
          '';
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = with pkgs; [
            openssl
            pkg-config
            ffmpeg 
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        vortex-dl = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "vortex-dl";
          version = "0.2.0";

          preBuild = ''
            mkdir -p frontend/dist/voltexdl/browser
            cp -r ${frontendDist}/* frontend/dist/voltexdl/browser/
          '';

          postInstall = ''
            wrapProgram $out/bin/vortex-dl \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.ffmpeg ]}
          '';
        });

      in
      {
        packages.default = vortex-dl;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ vortex-dl ];
          buildInputs = with pkgs; [
            rustToolchain
            just
            yarn
            nodejs_24
            mold
            clang
            cargo-edit
            cargo-watch
          ];
          shellHook = ''
            export RUST_BACKTRACE=1
            echo "🚀 VortexLD Dev Shell Loaded"
          '';
        };
      }
    );
}
