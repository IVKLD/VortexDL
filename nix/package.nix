{ pkgs }:

pkgs.stdenv.mkDerivation rec {
  pname = "vortex-dl";
  version = "0.4.21";

  src = pkgs.fetchurl {
    url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
    hash = "sha256-pjgPs8jUqn2KW6C9x/7VkE+vVaSgp/VgnDiazQu5CDA=";
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
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.yt-dlp pkgs.ffmpeg pkgs.android-tools ]}
  '';
}
