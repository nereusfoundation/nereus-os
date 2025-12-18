{
  inputs = {
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      flake-utils,
      crane,
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
        # target = "x86_64-unknown-uefi";
        rustToolchainFor =
          p:
          p.rust-bin.selectLatestNightlyWith (
            toolchain:
            toolchain.default.override {
              extensions = [ "rust-src" ];
              targets = [ "x86_64-unknown-linux-gnu" ];
            }
          );
        rustToolchain = rustToolchainFor pkgs;

        # NB: we don't need to overlay our custom toolchain for the *entire*
        # pkgs (which would require rebuidling anything else which uses rust).
        # Instead, we just want to update the scope that crane will use by appending
        # our specific toolchain there.
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;
        src = ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          doCheck = false; # prevents --all-targets in check: error[E0463]: can't find crate for `test`
          buildInputs = [ ];
          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./uefi-loader
              ./kernel
              ./bootinfo
              ./framebuffer
              ./fonts
              ./hal
              ./mem
              ./sync
              ./scheduler
              crate
            ];
          };

        # Build the top-level crates of the workspace as individual derivations.
        # This allows consumers to only depend on (and build) only what they need.
        # Though it is possible to build the entire workspace as a single derivation,
        # so this is left up to you on how to organize things
        #
        # Note that the cargo workspace must define `workspace.members` using wildcards,
        # otherwise, omitting a crate (like we do below) will result in errors since
        # cargo won't be able to find the sources for all members.
        kernel = craneLib.buildPackage (
          commonArgs
          // {
            pname = "kernel";
            cargoExtraArgs = "-p kernel -Z build-std=core,alloc,compiler_builtins,panic_abort -Z build-std-features=compiler-builtins-mem --target ${./kernel/x86_64-unknown-nereus.json}";
            src = fileSetForCrate ./kernel;
            inherit
              (craneLib.crateNameFromCargoToml {
                inherit src;
                cargoToml = ./kernel/Cargo.toml;
              })
              version
              ;
            cargoVendorDir = craneLib.vendorMultipleCargoDeps {
              inherit (craneLib.findCargoFiles src) cargoConfigs;
              cargoLockList = [
                ./Cargo.lock

                # Unfortunately this approach requires IFD (import-from-derivation)
                # otherwise Nix will refuse to read the Cargo.lock from our toolchain
                # (unless we build with `--impure`).
                #
                # Another way around this is to manually copy the rustlib `Cargo.lock`
                # to the repo and import it with `./path/to/rustlib/Cargo.lock` which
                # will avoid IFD entirely but will require manually keeping the file
                # up to date!
                "${rustToolchain.passthru.availableComponents.rust-src}/lib/rustlib/src/rust/library/Cargo.lock"
              ];
            };

          }
        );

      in
      {
        packages = { inherit kernel; };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.rust-bin.nightly.latest.default
          ];
        };
      }
    );
}
