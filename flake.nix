{
  description = "Tactical Domain-Driven Design building blocks";

  nixConfig = {
    extra-substituters = ["https://clemenscodes.cachix.org"];
    extra-trusted-public-keys = [
      "clemenscodes.cachix.org-1:yEwW1YgttL2xdsyfFDz/vv8zZRhRGMeDQsKKmtV1N18="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
    crane,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};

        # Rust toolchain — version, targets, and components declared in
        # rust-toolchain.toml; fenix reads from there.
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-h+t2xTBz5yt2YIO+1VMIIGlCU7gyp2LYOFvaV1nwOXU=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          pname = "ddd";
          version = "0.1.0";
          strictDeps = true;
          cargoExtraArgs = "--workspace";
        };

        # Cache cargo dependencies separately so a code-only change doesn't
        # rebuild the world.
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        ddd = craneLib.buildPackage (commonArgs // {inherit cargoArtifacts;});

        cargoFmt = craneLib.cargoFmt {inherit src;};

        cargoClippy = craneLib.cargoClippy (commonArgs
          // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });

        cargoTest = craneLib.cargoTest (commonArgs // {inherit cargoArtifacts;});
      in {
        formatter = pkgs.alejandra;

        packages = {
          default = ddd;
          inherit ddd cargoArtifacts;
        };

        checks = {
          inherit ddd cargoFmt cargoClippy cargoTest;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ddd];
          packages = with pkgs; [
            rustToolchain
            cargo-watch
            cargo-edit
            cargo-nextest
            taplo
            alejandra
            nil
          ];

          shellHook = ''
            echo ""
            echo "  ddd — Tactical DDD building blocks dev shell"
            echo ""
          '';
        };
      }
    );
}
