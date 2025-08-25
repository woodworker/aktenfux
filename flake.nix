{
  description = "Aktenfux - Obsidian vault frontmatter indexing CLI tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "clippy" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            cargo-outdated
            rust-analyzer
            pkg-config
            openssl
          ];

          shellHook = ''
            echo "🦀 Rust development environment for Aktenfux"
            echo "Available commands:"
            echo "  cargo build    - Build the project"
            echo "  cargo run      - Run the CLI tool"
            echo "  cargo test     - Run tests"
            echo "  cargo watch -x check - Watch for changes and check"
            echo ""
            echo "Example usage:"
            echo "  cargo run -- fields"
            echo "  cargo run -- filter --filter=tag=work"
            echo "  cargo run -- values --field=status"
          '';

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "aktenfux";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          
          buildInputs = with pkgs; [
            openssl
          ];
          
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };
      });
}