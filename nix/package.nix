{ pkgs }:

pkgs.stdenv.mkDerivation rec {
  pname = "vortex-dl";
  version = "0.4.25";

  src = pkgs.fetchurl {
    url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
    hash = "sha256-FBlROGH20R75TfIJ+vdGLP97xDbk3xAYkCeKDeA/fy4=";
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
