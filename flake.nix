{
  description = "VortexDL - High-performance SoundCloud downloader";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    let
      systemOutputs = flake-utils.lib.eachDefaultSystem (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          packages.default = pkgs.stdenv.mkDerivation rec {
            pname = "vortex-dl";
            version = "0.3.1";

            src = pkgs.fetchurl {
              url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
              hash = "sha256-05H+hnyrADJHkipkEZN9z6f1RR6rABaMAtVMzmsL6Fw=";
            };

            dontUnpack = true;

            nativeBuildInputs = [
              pkgs.autoPatchelfHook
            ];

            buildInputs = [
              pkgs.openssl
              pkgs.zlib
              pkgs.xz
              pkgs.stdenv.cc.cc.lib
            ];

            installPhase = ''
              install -m755 -D $src $out/bin/vortexdl
            '';
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustup
              pkg-config
              openssl
              udev
              systemd
              just
              nodejs
              yarn
              ffmpeg
            ];

            shellHook = ''
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.systemd.dev}/lib/pkgconfig:${pkgs.udev.dev}/lib/pkgconfig"
            '';
          };
        }
      );
    in
    systemOutputs // {
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.vortexdl;
        in
        {
          options.services.vortexdl = {
            enable = lib.mkEnableOption "VortexDL";
            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [
              self.packages.${pkgs.system}.default
            ];

            systemd.services.vortexdl = {
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              path = [ pkgs.ffmpeg ];
              serviceConfig = {
                DynamicUser = true;
                Restart = "always";
                StateDirectory = "vortexdl";
                WorkingDirectory = "/var/lib/vortexdl";
                Environment = [ "HOME=/var/lib/vortexdl" ];
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/vortexdl --serve --port ${toString config.services.vortexdl.port}";
              };
            };
          };
        };
    };
}
