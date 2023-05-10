{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.11";
    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    let derivation = pkgs:
      let naersk' = pkgs.callPackage naersk { };
      in
      naersk'.buildPackage {
        src = ./.;
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs; [ openssl ];
      };

    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };

          naersk' = pkgs.callPackage naersk { };

        in
        rec {
          # For `nix build` & `nix run`:
          defaultPackage = derivation pkgs;

          # For `nix develop`:
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs;
              [
                cargo
                openssl
                pkg-config
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
