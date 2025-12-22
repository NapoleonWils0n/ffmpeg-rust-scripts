{
  description = "rust flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, naersk, rust-overlay }: 
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };

      # Define the toolchain with the IDE components (rust-src and rust-analyzer)
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
        targets = [ "x86_64-unknown-linux-musl" ]; # static Linux builds
      };

      # Override naersk to use our toolchain
      naerskLib = (naersk.lib.${system}.override {
        cargo = rustToolchain;
        rustc = rustToolchain;
      });
    in {
      # For 'nix develop'
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [ rustToolchain ];

        # Point tools (like Emacs Eglot) to the Rust source code in the Nix store
        shellHook = ''
          export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
        '';
      };

      # For 'nix build'
      packages.${system}.default = naerskLib.buildPackage {
        src = ./.;
        # Add these if your Hello World needs them later
        # buildInputs = [ pkgs.glib ]; 
        # nativeBuildInputs = [ pkgs.pkg-config ];
      };
    };
}
