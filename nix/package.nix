{ pkgs }:

pkgs.stdenv.mkDerivation rec {
  pname = "vortex-dl";
  version = "0.4.24";

  src = pkgs.fetchurl {
    url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
    hash = "sha256-2WWC3byo/gpEKftpMigdoG8hq++8Kwv4ZUdaO+yccHk=";
  };

  dontUnpack = true;

  nativeBuildInputs = [
    pkgs.autoPatchelfHook
    pkgs.makeWrapper
  ];

  buildInputs = [
    pkgs.openssl
    pkgs.zlib
    pkgs.xz
    pkgs.stdenv.cc.cc.lib
  ];

  installPhase = ''
    install -m755 -D $src $out/bin/vortex-dl
    wrapProgram $out/bin/vortex-dl \
      --prefix PATH : ${pkgs.lib.makeBinPath [
        pkgs.yt-dlp
        pkgs.ffmpeg
        pkgs.android-tools
        pkgs.deno
        pkgs.python3
      ]}
  '';
}
