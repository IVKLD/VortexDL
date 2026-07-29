{ pkgs }:

pkgs.stdenv.mkDerivation rec {
  pname = "vortex-dl";
  version = "0.4.9";

  src = pkgs.fetchurl {
    url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
    hash = "sha256-3zPAcsAohszcR29tEo7r8FtgAC8b2xd4ENP9Pcv0yWI=";
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
}
