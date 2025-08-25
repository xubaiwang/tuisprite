{
  description = "Rust dev shell template";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              rust-analyzer
              clippy
              rustfmt
            ];
            buildInputs = [
            ];
            nativeBuildInputs = [
              pkg-config
              rustc
              cargo
            ];

            # https://github.com/NixOS/nixpkgs/issues/177952#issuecomment-3172381779
            NIX_NO_SELF_RPATH = true;
          };
      }
    );
}
