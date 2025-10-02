{
  description = "Rust dev shell template";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        toolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            packages = [
              toolchain
              cargo-bloat
            ];
            buildInputs = [
            ];
            nativeBuildInputs = [
              pkg-config
            ];

            NIX_NO_SELF_RPATH = true;
          };
      }
    );
}
