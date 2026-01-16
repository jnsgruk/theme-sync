{
  description = "theme-sync";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-parts,
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { config, system, ... }:
        let
          overlays = [ (import rust-overlay) ];

          pkgs = import nixpkgs { inherit system overlays; };

          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };

          buildThemeSync = (
            pkgs:
            let
              inherit (pkgs) lib rustPlatform;
            in
            rustPlatform.buildRustPackage {
              pname = "theme-sync";
              version = (lib.trivial.importTOML ./Cargo.toml).package.version;
              src = lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;
              meta = {
                description = "Reconfigure applications to match system theme";
                homepage = "https://github.com/jnsgruk/theme-sync";
                license = lib.licenses.asl20;
                mainProgram = "theme-sync";
                platforms = lib.platforms.unix;
                maintainers = with lib.maintainers; [ jnsgruk ];
              };
            }
          );
        in
        {
          packages = {
            default = self.packages.${system}.theme-sync;
            theme-sync = buildThemeSync pkgs;
            theme-sync-cross-aarch64 = buildThemeSync pkgs.pkgsCross.aarch64-multiplatform;
          };

          devShells = {
            default = pkgs.mkShell {
              name = "theme-sync";

              NIX_CONFIG = "experimental-features = nix-command flakes";
              RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

              buildInputs = [
                rust
              ]
              ++ (with pkgs; [
                nil
                nixfmt
                rustup
              ]);
            };

          };
        };
    };
}
