{
  description = "The synxit-server flake provides a development environment and a package";
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
        mkScript =
          name: text:
          let
            script = pkgs.writeShellScriptBin name text;
          in
          script;
        scripts = [
          (mkScript "run" "cargo run -- data/config.toml")
        ];
      in
      {
        formatter = pkgs.nixfmt-tree;
        devShells.default = pkgs.mkShell {
          buildInputs =
            with pkgs;
            [
              cargo
              rustc
              rust-analyzer
              rustfmt
            ]
            ++ scripts;

          nativeBuildInputs = with pkgs; [
            pkg-config
            openssl
          ];
        };

        packages.synxit-server = pkgs.rustPlatform.buildRustPackage rec {
          pname = "synxit-server";
          version = "0.0.1";
          src = ./.;

          useFetchCargoVendor = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          # cargoHash = "sha256-LOmLivdtV+wKxHYoDfdg6Q2k/8Am7uxk0hw4th6ynhU=";
          cargoLock.lockFile = ./Cargo.lock;
        };
        defaultPackage = self.packages.${system}.synxit-server;
      }
    );
}
