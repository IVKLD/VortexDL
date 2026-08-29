{ pkgs }:

pkgs.mkShell {
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
    yt-dlp
    clang
    mold
    cargo-watch
    android-tools
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.systemd.dev}/lib/pkgconfig:${pkgs.udev.dev}/lib/pkgconfig"
  '';
}
