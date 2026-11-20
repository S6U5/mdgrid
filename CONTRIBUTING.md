# Contributing to mdgrid

Thanks for your interest. Bug reports and ideas are welcome as issues, in English or Japanese. mdgrid is maintained by its author together with an AI coding agent: issues are read, reproduced and fixed from the report, so the more exact the steps and the notes involved, the faster the fix. Pull requests are limited to collaborators: if you have a fix or a change in mind, open an issue that describes it (a patch or a link to a branch in your fork is fine), and a collaborator will bring it in.

## Reporting a bug

Open an issue with the bug template. The most useful reports include:

- the mdgrid version (`mdgrid --version`), your OS and terminal;
- the smallest set of notes (frontmatter is usually enough) and the keys you pressed;
- what you expected and what happened. For a write-back problem, the file before and after saving.

If a note was written incorrectly, that is the most serious kind of bug for mdgrid. Please say so in the title.

Security problems: see [SECURITY.md](SECURITY.md) instead of opening a public issue.

## Proposing a feature

Open an issue with the feature template and describe the problem first, then the change you have in mind. mdgrid tries to stay a table of frontmatter (see [specs/requirements/scope.md](specs/requirements/scope.md)), so ideas that turn it into a different kind of screen may be declined, with the reason.

## How changes are made

The requirements are the source of truth: [specs/requirements/](specs/requirements/README.md), one file per capability and one line per requirement, named like `req~screen.action-menu~1`. They are written in Japanese; you do not need to read them to send a fix, and a maintainer will help map your change to the requirements.

- A change in behavior changes the requirement line in the same pull request, and a person reviews it.
- Tests name the requirement they check in a comment, as `// [utest -> req~screen.action-menu~1]`.
- Run `python3 tools/trace_gate.py check` before you finish; it fails when a mark points to an unknown requirement or a critical (P0) requirement has no real test.
- The specs, change records and decisions written before 2026-11-17 are kept unchanged in `specs/_history/decidespec/`. Old IDs such as `SR-24` map to the new names in [specs/requirements/renames.md](specs/requirements/renames.md).

Design notes (how, not what) are in [docs/design.md](docs/design.md).

## Building and testing

You need Rust 1.90 or newer.

```sh
cargo build
cargo test
./ci.sh              # fmt, clippy, tests, and cargo-deny if it is installed
```

The tests run in Japanese by default (`.cargo/config.toml`). Screen tests drive the real binary in a pseudo-terminal (`tests/pty/`).

## Feature tours

`demos/` holds [VHS](https://github.com/charmbracelet/vhs) tapes that record mdgrid's features as GIFs (`sh demos/record.sh`). They run on a copy of `examples/` in a temporary folder and never touch your notes or settings. The GIFs go to `demos/out/` and are not committed; each release records them in CI and attaches them to the release. When you add a feature, add a tape for it (see `demos/README.md`).

## Pull requests (collaborators)

- Keep a pull request to one change. Small is easier to review.
- Run `./ci.sh` before pushing; CI runs the same checks on Linux, macOS and Windows.
- Add or update tests for the behavior you change, and update the docs in `docs/` (English and Japanese) when a key, setting or message changes.
- Describe what changed and why in the pull request. If you know the requirement IDs, list them.

## License

By contributing, you agree that your contributions are licensed under the same terms as the project: MIT or Apache-2.0, at the user's option.
