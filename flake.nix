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
        targets = [ "x86_64-unknown-linux-musl" "x86_64-pc-windows-gnu" ]; 
      };

      naerskLib = (naersk.lib.${system}.override {
        cargo = rustToolchain;
        rustc = rustToolchain;
      });
    in {
      devShells.default = pkgs.mkShell {
        # ADD MINGW TO THE SHELL FOR LINKING
        buildInputs = [ 
          rustToolchain 
          pkgs.pkgsCross.mingwW64.stdenv.cc 
        ];


        # Tell Cargo which linker to use for Windows
        # Add these lines to help the linker find pthreads
        shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
          export NIX_CROSS_LDFLAGS="-L${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib"
          export NIX_CROSS_CFLAGS_COMPILE="-I${pkgs.pkgsCross.mingwW64.windows.pthreads}/include"
          export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"
          export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L ${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib"
        '';
      };

      packages.default = naerskLib.buildPackage {
        src = ./.;
      };
    } 
  ); # This closes eachDefaultSystem
}    # This closes outputs
