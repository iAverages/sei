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
      inherit (pkgs) lib;

      pnpm = pkgs.pnpm.override {nodejs = pkgs.nodejs_24;};
      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      };

      source = lib.cleanSourceWith {
        src = ./.;
        filter = path: type: let
          name = baseNameOf path;
        in
          !(
            lib.elem name [
              ".direnv"
              ".env"
              ".git"
              ".output"
              "dist"
              "node_modules"
              "target"
            ]
            || lib.hasPrefix "result" name
          );
      };

      api = pkgs.rustPlatform.buildRustPackage {
        pname = "sei-api";
        version = "0.1.0";
        src = source;

        cargoLock.lockFile = ./Cargo.lock;
        cargoBuildFlags = ["--bin" "sei"];
        SQLX_OFFLINE = "true";

        nativeBuildInputs = [pkgs.pkg-config];
        buildInputs = [pkgs.openssl];
      };

      webPnpmDeps = pnpm.fetchDeps {
        pname = "sei-web-pnpm-deps";
        version = "0.1.0";
        src = source;
        fetcherVersion = 2;
        hash = "sha256-y3cWCCzTttH/V5/E4Ry7mPSZLDE9lKG5AHtbbMtVdbw=";
      };

      web = pkgs.stdenvNoCC.mkDerivation {
        pname = "sei-web";
        version = "0.1.0";
        src = source;
        pnpmDeps = webPnpmDeps;
        NODE_ENV = "production";

        nativeBuildInputs = [
          pkgs.nodejs_24
          pnpm
          pnpm.configHook
        ];

        buildPhase = ''
          runHook preBuild
          pnpm --filter @sei/web build
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          cp -r apps/web/.output "$out"
          runHook postInstall
        '';
      };

      apiImage = pkgs.dockerTools.buildLayeredImage {
        name = "sei-api";
        tag = "latest";
        contents = [api pkgs.cacert];
        extraCommands = ''
          mkdir -m 1777 tmp
        '';
        config = {
          Cmd = ["${api}/bin/sei"];
          Env = [
            "BIND_ADDR=0.0.0.0:3001"
            "RUST_LOG=info"
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
          ExposedPorts."3001/tcp" = {};
          User = "10001:10001";
          WorkingDir = "/";
        };
      };

      webImage = pkgs.dockerTools.buildLayeredImage {
        name = "sei-web";
        tag = "latest";
        contents = [pkgs.nodejs_24 pkgs.cacert];
        extraCommands = ''
          mkdir -m 1777 tmp
        '';
        config = {
          Cmd = ["${pkgs.nodejs_24}/bin/node" "${web}/server/index.mjs"];
          Env = [
            "HOST=0.0.0.0"
            "NODE_ENV=production"
            "PORT=3000"
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
          ExposedPorts."3000/tcp" = {};
          User = "10001:10001";
          WorkingDir = web;
        };
      };
    in {
      packages = {
        inherit api web apiImage webImage;
        default = webImage;
      };

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
