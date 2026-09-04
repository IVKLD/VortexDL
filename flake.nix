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
          packages.default = import ./nix/package.nix { inherit pkgs; };
          devShells.default = import ./nix/devshell.nix { inherit pkgs; };
          devShells.frontend = import ./nix/devshell-frontend.nix { inherit pkgs; };
        }
      );
    in
    systemOutputs // {
      nixosModules.default = import ./nix/modules/nixos.nix { inherit self; };
      nixosModules.vortexdl = self.nixosModules.default;

      homeManagerModules.default = import ./nix/modules/home-manager.nix { inherit self; };
      homeManagerModules.vortexdl = self.homeManagerModules.default;
    };
}