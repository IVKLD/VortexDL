{ pkgs }:

pkgs.stdenv.mkDerivation rec {
  pname = "vortex-dl";
  version = "0.4.11";

  src = pkgs.fetchurl {
    url = "https://github.com/IVKLD/VortexDL/releases/download/v${version}/vortex-dl";
    hash = "sha256-6vboAaywdJ/elfJOe+Cdnpu6lyBL4YoiIOkWBsgRsQw=";
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
    mkdir -p $out/bin
    install -m755 $src $out/bin/vortex-dl
    ln -s $out/bin/vortex-dl $out/bin/vortexdl
  '';
}
