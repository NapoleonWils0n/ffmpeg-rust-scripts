{
  description = "rust flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, naersk, rust-overlay, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
    let
      # system is already provided by eachDefaultSystem, so we don't define it here
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
        targets = [ "x86_64-unknown-linux-musl" ]; 
      };

      naerskLib = (naersk.lib.${system}.override {
        cargo = rustToolchain;
        rustc = rustToolchain;
      });
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = [ rustToolchain ];

        shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
        '';
      };

      packages.default = naerskLib.buildPackage {
        src = ./.;
      };
    } 
  ); # This closes eachDefaultSystem
}    # This closes outputs
