{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      pnpm = pkgs.pnpm;

      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      };
    in {
      devShells.default = with pkgs;
        mkShell {
          packages = [
            nodejs_24
            pnpm
            pkg-config
            openssl
            just
            mprocs
            rust
          ];

          shellHook = ''
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig";
            export PRISMA_QUERY_ENGINE_BINARY="${prisma-engines}/bin/query-engine";
            export PRISMA_SCHEMA_ENGINE_BINARY="${prisma-engines}/bin/schema-engine";
            export PRISMA_FMT_BINARY="${prisma-engines}/bin/prisma-fmt"
            export PRISMA_QUERY_ENGINE_LIBRARY="${prisma-engines}/lib/libquery_engine.node"
            export PATH="$PWD/node_modules/.bin/:$PATH"
          '';
        };
    });
}
