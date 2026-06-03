{
  inputs = {
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, naersk, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];

      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);

      nixpkgsFor = forAllSystems (system: import nixpkgs {
        inherit system;
      });

      derivation = pkgs:
        let naersk' = pkgs.callPackage naersk { };
        in
        naersk'.buildPackage {
          src = ./.;
          meta.mainProgram = "mprisqueeze";
        };

    in
    {
      devShell = forAllSystems (system:
        let pkgs = nixpkgsFor.${system};
        in pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.cargo-edit
            pkgs.rust-analyzer
            pkgs.rustc
            pkgs.rustfmt
            pkgs.squeezelite-pulse
          ];
        });

      packages = forAllSystems (system:
        let pkgs = nixpkgsFor.${system};
        in {
          default = derivation pkgs;
        }
      );

      overlay = final: prev: {
        mprisqueeze = derivation final;
      };
    };
}
