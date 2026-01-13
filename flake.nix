{
  description = "theme-sync";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsForSystem =
        system:
        (import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        });
    in
    {
      packages = forAllSystems (
        system:
        let
          inherit (pkgsForSystem system)
            lib
            rustPlatform
            ;

          cargoToml = lib.trivial.importTOML ./Cargo.toml;
          version = cargoToml.package.version;
        in
        rec {
          default = theme-sync;

          theme-sync = rustPlatform.buildRustPackage {
            pname = "theme-sync";
            version = version;
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
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsForSystem system;
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          };
        in
        {
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
        }
      );
    };
}
