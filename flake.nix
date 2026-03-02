{
  description = "A Git terminal UI inspired by Magit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      rust-overlay,
      ...
    }@inputs:
    let
      inherit (nixpkgs) lib;
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      eachSystem =
        with lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) systems);
    in
    eachSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = pkgs.rust-bin.stable."1.92.0".default.override {
          targets = [
            "x86_64-apple-darwin"
            "aarch64-apple-darwin"
            "x86_64-unknown-linux-gnu"
            "x86_64-unknown-linux-musl"
            "aarch64-unknown-linux-gnu"
          ];
        };
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
        magi = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [
            openssl
            zlib
            libgit2
            libssh2
          ];
        };

        mkCrossPackage =
          {
            crossPkgs,
            linkerEnvVar,
          }:
          let
            naersk-cross = crossPkgs.callPackage naersk {
              cargo = toolchain;
              rustc = toolchain;
            };
          in
          naersk-cross.buildPackage {
            src = ./.;
            strictDeps = true;
            nativeBuildInputs = [ crossPkgs.buildPackages.pkg-config ];
            buildInputs = with crossPkgs; [
              openssl
              zlib.dev
              libgit2
              libssh2
            ];
            "${linkerEnvVar}" = "${crossPkgs.stdenv.cc}/bin/${crossPkgs.stdenv.cc.targetPrefix}cc";
          };
      in
      {
        packages =
          {
            default = magi;
            magi = magi;
            clippy = naersk'.buildPackage {
              src = ./.;
              mode = "clippy";
            };
          }
          // lib.optionalAttrs (system == "x86_64-linux") {
            magi-aarch64-linux = mkCrossPackage {
              crossPkgs = pkgs.pkgsCross.aarch64-multiplatform;
              linkerEnvVar = "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER";
            };
            magi-x86_64-linux-musl = mkCrossPackage {
              crossPkgs = pkgs.pkgsCross.musl64;
              linkerEnvVar = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER";
            };
          };
        checks.default = naersk'.buildPackage {
          src = ./.;
          mode = "test";
        };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.openssl
            pkgs.pkg-config
            pkgs.perl
            toolchain
            pkgs.rust-analyzer
          ];
        };
      }
    );
}
