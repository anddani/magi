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
    { self
    , nixpkgs
    , naersk
    , rust-overlay
    , ...
    } @ inputs:
    let
      inherit (nixpkgs) lib;
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" "aarch64-linux" ];
      eachSystem =
        with lib;
        f: foldAttrs mergeAttrs { } (map (s: mapAttrs (_: v: { ${s} = v; }) (f s)) systems);
    in
    eachSystem
      (
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
        in
        {
          packages.default = naersk'.buildPackage { src = ./.; };
          packages.clippy = naersk'.buildPackage {
            src = ./.;
            mode = "clippy";
          };
          checks.default = naersk'.buildPackage {
            src = ./.;
            mode = "test";
          };
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = [ pkgs.perl toolchain pkgs.rust-analyzer ];
          };
        }
      );
}
