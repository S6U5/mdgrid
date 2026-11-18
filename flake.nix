{
  description = "Browse and edit Markdown frontmatter as a table in the terminal";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;
      # The targets the release builds for, minus Windows (Nix does not run there).
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
      cargo = (lib.importTOML ./Cargo.toml).package;
    in
    {
      packages = forAllSystems (pkgs: rec {
        mdgrid = pkgs.rustPlatform.buildRustPackage {
          pname = cargo.name;
          inherit (cargo) version;

          # The same files as the crate (`include` in Cargo.toml), so editing docs or specs does not rebuild.
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./src
              ./build.rs
              ./locales
              ./starters
              ./Cargo.toml
              ./Cargo.lock
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;

          # The tests are not in the source above; CI runs them with `cargo test`.
          doCheck = false;

          nativeBuildInputs = [ pkgs.installShellFiles ];
          # Completions and the man page come from the binary itself, as in the release archives.
          postInstall = lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
            installShellCompletion --cmd mdgrid \
              --bash <($out/bin/mdgrid --completions bash) \
              --zsh <($out/bin/mdgrid --completions zsh) \
              --fish <($out/bin/mdgrid --completions fish)
            $out/bin/mdgrid --man > mdgrid.1
            installManPage mdgrid.1
          '';

          meta = {
            inherit (cargo) description homepage;
            license = with lib.licenses; [
              mit
              asl20
            ];
            mainProgram = "mdgrid";
          };
        };
        default = mdgrid;
      });

      # `nix flake check` builds the package.
      checks = forAllSystems (pkgs: {
        inherit (self.packages.${pkgs.stdenv.hostPlatform.system}) mdgrid;
      });
    };
}
