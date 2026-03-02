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
            "aarch64-unknown-linux-gnu"
          ];
        };
        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };
        magi = naersk'.buildPackage {
          src = ./.;
          propagatedBuildInputs = with pkgs; [
            openssl
            pkg-config
            zlib
          ];
        };

        mkCrossPackage =
          {
            crossSystem,
            cargoEnvVar,
          }:
          let
            crossPkgs = import nixpkgs {
              localSystem = system;
              crossSystem = lib.systems.elaborate crossSystem;
              overlays = overlays;
            };
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
              libgit2
              zlib
            ];
            "${cargoEnvVar}" = "${crossPkgs.stdenv.cc}/bin/${crossPkgs.stdenv.cc.targetPrefix}cc";
          };
      in
      {
        packages = {
          default = magi;
          magi = magi;
          clippy = naersk'.buildPackage {
            src = ./.;
            mode = "clippy";
          };
        }
        # Cross-compile aarch64 Linux binary from x86_64-linux runner
        // lib.optionalAttrs (system == "x86_64-linux") {
          magi-aarch64-linux = mkCrossPackage {
            crossSystem = "aarch64-linux";
            cargoEnvVar = "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER";
          };
        }
        # Cross-compile x86_64 macOS binary from aarch64-darwin (Apple Silicon) runner
        // lib.optionalAttrs (system == "aarch64-darwin") {
          magi-x86_64-darwin = mkCrossPackage {
            crossSystem = "x86_64-darwin";
            cargoEnvVar = "CARGO_TARGET_X86_64_APPLE_DARWIN_LINKER";
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
