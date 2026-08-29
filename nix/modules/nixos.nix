{ self }:
{ config, lib, pkgs, ... }:

let
  cfg = config.services.vortexdl;
in
{
  options.services.vortexdl = {
    enable = lib.mkEnableOption "VortexDL service";

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
      default = "0.0.0.0";
      description = "Host IP to bind to. Use 0.0.0.0 to make it accessible outside localhost.";
    };

    openFirewall = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Whether to open the firewall for the specified port.";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/vortexdl";
      description = "Directory to store VortexDL state and data.";
    };

    downloadDir = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Directory where downloaded music will be saved. Defaults to dataDir if null.";
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

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "Extra command line arguments for vortexdl.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];

    environment.systemPackages = [
      cfg.package
      pkgs.android-tools
    ];

    services.udev.packages = [ pkgs.android-tools ];

    users.users = lib.optionalAttrs (cfg.user == "vortexdl") {
      vortexdl = {
        isSystemUser = true;
        group = cfg.group;
        extraGroups = [ "adbusers" ];
        home = cfg.dataDir;
        createHome = false;
      };
    };

    users.groups = lib.optionalAttrs (cfg.group == "vortexdl") {
      vortexdl = {};
    };

    systemd.tmpfiles.rules = [
      "d '${cfg.dataDir}' 0750 ${cfg.user} ${cfg.group} - -"
      "d '${cfg.dataDir}/.android' 0700 ${cfg.user} ${cfg.group} - -"
    ] ++ (lib.optional (cfg.downloadDir != null) "d '${cfg.downloadDir}' 0755 ${cfg.user} ${cfg.group} - -");

    systemd.services.vortexdl = {
      description = "VortexDL Web Service";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      
      path = [ pkgs.ffmpeg pkgs.yt-dlp pkgs.android-tools ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        RestartSec = "5s";
        WorkingDirectory = cfg.dataDir;
        Environment = [
          "HOME=${cfg.dataDir}"
          "ANDROID_USER_HOME=${cfg.dataDir}/.android"
        ];
        
        ExecStart = "${cfg.package}/bin/vortex-dl --serve --port ${toString cfg.port} --host ${cfg.host}"
          + (lib.optionalString (cfg.downloadDir != null) " --output ${cfg.downloadDir}")
          + (lib.optionalString (cfg.extraArgs != []) " ${lib.escapeShellArgs cfg.extraArgs}");
      };
    };
  };
}
