{
  description = "A basic rust devshell flake";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            rustfmt
          ];
        };

        packages.synxit-server = pkgs.rustPlatform.buildRustPackage rec {
          pname = "synxit-server";
          version = "0.0.1";
          src = ./.;

          useFetchCargoVendor = true;

          cargoHash = "sha256-LOmLivdtV+wKxHYoDfdg6Q2k/8Am7uxk0hw4th6ynhU=";

          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };
        defaultPackage = self.packages.${system}.synxit-server;
      }
    );
}
