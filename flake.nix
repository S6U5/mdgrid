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

      # The tools for working on mdgrid: `nix develop`, or `use flake` with direnv. Their versions come from
      # flake.lock, so every machine and CI get the same ones, and a machine that does not work on mdgrid
      # gets none of them. ./ci.sh runs the same checks with these tools.
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          # `pkgs.` on each: `cargo` in this file is Cargo.toml's [package], and a `with pkgs;` would not win over it.
          packages = [
            # Rust: build, lint, format, tests, dependency checks.
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.cargo-nextest
            pkgs.cargo-deny
            # Python: scripts/ and the trace-gate checker (tools/trace_gate.py).
            pkgs.python3
            pkgs.ruff
            # git for the tests of saving with git (write-back.git-*): on macOS, /usr/bin/git asks Xcode for the
            # real git, which a Nix shell's SDK settings hide ("tool 'git' not found").
            pkgs.git
            # Shell scripts and GitHub Actions workflows.
            pkgs.shellcheck
            pkgs.shfmt
            pkgs.actionlint
          ];
        };
      });

      # `nix flake check` builds the package.
      checks = forAllSystems (pkgs: {
        inherit (self.packages.${pkgs.stdenv.hostPlatform.system}) mdgrid;
      });
    };
}
