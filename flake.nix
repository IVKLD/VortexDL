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
              url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl-web";
              hash = "sha256-FGykQMH1vGUjaphSz9SOHrRUOKEGFzt+emnl388PELQ=";
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
            dataDir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/vortexdl";
            };
            user = lib.mkOption {
              type = lib.types.str;
              default = "vortexdl";
            };
            group = lib.mkOption {
              type = lib.types.str;
              default = "vortexdl";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [
              self.packages.${pkgs.system}.default
            ];

            users.users = lib.optionalAttrs (cfg.user == "vortexdl") {
              vortexdl = {
                isSystemUser = true;
                group = cfg.group;
                home = cfg.dataDir;
                createHome = true;
              };
            };

            users.groups = lib.optionalAttrs (cfg.group == "vortexdl") {
              vortexdl = {};
            };

            systemd.tmpfiles.rules = [
              "d '${cfg.dataDir}' 0750 '${cfg.user}' '${cfg.group}' - -"
            ];

            systemd.services.vortexdl = {
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              path = [ pkgs.ffmpeg ];
              serviceConfig = {
                User = cfg.user;
                Group = cfg.group;
                Restart = "always";
                ExecStartPre = [
                  "+${pkgs.bash}/bin/bash -c 'if [ -L \"${cfg.dataDir}\" ]; then rm \"${cfg.dataDir}\"; fi'"
                  "+${pkgs.coreutils}/bin/mkdir -p ${cfg.dataDir}"
                  "+${pkgs.coreutils}/bin/chown ${cfg.user}:${cfg.group} ${cfg.dataDir}"
                ];
                WorkingDirectory = cfg.dataDir;
                Environment = [ "HOME=${cfg.dataDir}" ];
                ExecStart = "${self.packages.${pkgs.system}.default}/bin/vortexdl --serve --port ${toString cfg.port}";
              };
            };
          };
        };
    };
}
