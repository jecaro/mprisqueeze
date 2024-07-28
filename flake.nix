{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let
      derivation = pkgs:
        let naersk' = pkgs.callPackage naersk { };
        in
        naersk'.buildPackage {
          src = ./.;
          meta.mainProgram = "mprisqueeze";
        };

    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };

        in
        {
          # For `nix build` & `nix run`:
          packages.default = derivation pkgs;

          # For `nix develop`:
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [
                cargo
                cargo-edit
                rust-analyzer
                rustc
                rustfmt
                squeezelite-pulse
              ];
          };
        }
      ) // {
      overlay = final: prev:
        {
          mprisqueeze = derivation final;
        };
    };
}
