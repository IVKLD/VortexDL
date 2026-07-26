{ self }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.vortexdl;
in
{
  options.services.vortexdl = {
    enable = lib.mkEnableOption "VortexDL Home Manager user service";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      description = "VortexDL package to use.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "Port to listen on.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Host IP to bind to.";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "${config.home.homeDirectory}/Music";
      description = "Directory where VortexDL runs and saves downloaded music.";
    };

    downloadDir = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Directory where downloaded music will be saved. Defaults to dataDir if null.";
    };

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "Extra command line arguments for vortexdl.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package pkgs.android-tools ];

    systemd.user.services.vortexdl = {
      Unit = {
        Description = "VortexDL Web Service";
        After = [ "network.target" ];
      };

      Install = {
        WantedBy = [ "default.target" ];
      };

      Service = {
        ExecStart = "${cfg.package}/bin/vortexdl --serve --port ${toString cfg.port} --host ${cfg.host}"
          + (if cfg.downloadDir != null then " --output ${cfg.downloadDir}" else " --output ${cfg.dataDir}")
          + (lib.optionalString (cfg.extraArgs != []) " ${lib.escapeShellArgs cfg.extraArgs}");
        WorkingDirectory = cfg.dataDir;
        Environment = [
          "ANDROID_USER_HOME=${config.home.homeDirectory}/.android"
        ];
        Restart = "always";
        RestartSec = "5s";
      };
    };
  };
}
