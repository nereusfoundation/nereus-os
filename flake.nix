{
  inputs = {
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      flake-utils,
      naersk,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        lib = pkgs.lib;

        rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" ];
            targets = [ "x86_64-unknown-linux-gnu" ];

          }
        );

        naersk' = pkgs.callPackage naersk {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        commonArgs = pname: {
          inherit pname;
          cargoBuildOptions =
            x:
            x
            ++ [
              "-p"
              "${pname}"
              "-Z"
              "build-std=core,alloc,compiler_builtins,panic_abort"
              "-Z"
              "build-std-features=compiler-builtins-mem"
            ];

          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./kernel
              ./uefi-loader
              ./bootinfo
              ./framebuffer
              ./fonts
              ./hal
              ./mem
              ./sync
              ./scheduler
            ];
          };

          strictDeps = true;
          doCheck = false; # can't find crate for `test`
          buildInputs = [ ];
          additionalCargoLock = "${rustToolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/library/Cargo.lock";
        };

        kernel = naersk'.buildPackage (
          (commonArgs "kernel")
          // {
            CARGO_BUILD_TARGET = "${./kernel/x86_64-unknown-nereus.json}";
          }
        );
        loader = naersk'.buildPackage (
          (commonArgs "uefi-loader")
          // {
            CARGO_BUILD_TARGET = "x86_64-unknown-uefi";
          }
        );

      in
      {
        packages = { inherit kernel loader; };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
          ];
        };
      }
    );
}
