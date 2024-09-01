{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";
    rust-overlay.url = "https://flakehub.com/f/oxalica/rust-overlay/*.tar.gz";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    allSystems = [
      "x86_64-linux"
    ];

    forEachSystem = f:
      nixpkgs.lib.genAttrs allSystems (system:
        f {
          inherit system;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
          };
        });
  in {
    devShells = forEachSystem ({
      pkgs,
      system,
    }: {
      default = with pkgs; mkShell {
        shellHook = ''
          export PKG_CONFIG_PATH="${openssl.dev}/lib/pkgconfig";
          export PRISMA_QUERY_ENGINE_BINARY="${prisma-engines}/bin/query-engine";
          export PRISMA_SCHEMA_ENGINE_BINARY="${prisma-engines}/bin/schema-engine";
          export PRISMA_FMT_BINARY="${prisma-engines}/bin/prisma-fmt"
          export PRISMA_QUERY_ENGINE_LIBRARY="${prisma-engines}/lib/libquery_engine.node"
        '';
        packages = with pkgs; [
          pkg-config
          openssl
          (rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          })
          cargo-watch
        ];
      };
    });
  };
}
