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
            version = "0.3.9";

            src = pkgs.fetchurl {
              url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
              hash = "sha256-ZoUEL1CFDrHd00F2Ra6iu3fIKZj0n6qu+GaCZCsKyOQ=";
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
              clang
              mold
              cargo-watch
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
            enable = lib.mkEnableOption "VortexDL service";

            port = lib.mkOption {
              type = lib.types.port;
              default = 3000;
              description = "Port to listen on.";
            };

            host = lib.mkOption {
              type = lib.types.str;
              default = "0.0.0.0";
              description = "Host IP to bind to. Use 0.0.0.0 to make it accessible outside localhost.";
            };

            openFirewall = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Whether to open the firewall for the specified port.";
            };

            dataDir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/vortexdl";
              description = "Directory to store VortexDL data.";
            };

            user = lib.mkOption {
              type = lib.types.str;
              default = "vortexdl";
              description = "User account under which VortexDL runs.";
            };

            group = lib.mkOption {
              type = lib.types.str;
              default = "vortexdl";
              description = "Group under which VortexDL runs.";
            };
          };

          config = lib.mkIf cfg.enable {
            networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];

            environment.systemPackages = [
              self.packages.${pkgs.system}.default
            ];

            users.users = lib.optionalAttrs (cfg.user == "vortexdl") {
              vortexdl = {
                isSystemUser = true;
                group = cfg.group;
                home = cfg.dataDir;
                createHome = false; # tmpfiles создаст директорию с нужными правами
              };
            };

            users.groups = lib.optionalAttrs (cfg.group == "vortexdl") {
              vortexdl = {};
            };

            systemd.tmpfiles.rules = [
              "d '${cfg.dataDir}' 0750 ${cfg.user} ${cfg.group} - -"
            ];

            systemd.services.vortexdl = {
              description = "VortexDL Web Service";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              
              path = [ pkgs.ffmpeg pkgs.android-tools ];

              serviceConfig = {
                Type = "simple";
                User = cfg.user;
                Group = cfg.group;
                Restart = "always";
                RestartSec = "5s";
                WorkingDirectory = cfg.dataDir;
                Environment = [ "HOME=${cfg.dataDir}" ];
                
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/vortexdl --serve --port ${toString cfg.port} --host ${cfg.host}";
              };
            };
          };
        };
    };
}