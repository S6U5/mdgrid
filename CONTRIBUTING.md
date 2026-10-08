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

Open an issue with the feature template and describe the problem first, then the change you have in mind. mdgrid tries to stay a table of frontmatter (see [specs/scope](specs/scope/spec.md)), so ideas that turn it into a different kind of screen may be declined, with the reason.

## How changes are made

The specification is the source of truth: [specs/](specs/README.md), one `spec.md` per capability, with requirement IDs such as `SR-24`. The specs and the change records are written in Japanese; you do not need to read them to send a fix, and a maintainer will help map your change to the requirements.

- A change in behavior starts as a proposal in `specs/_proposals/`, which is accepted or rejected in `specs/_decisions/`.
- Each code change has a change record in `specs/_changes/` that lists its tasks, the files it touches, and how it was verified.
- Tests name the requirement they check, as `test_sr_24_…` or `[SR-24]` in the test body.
- Tests that guard requirements decided by a person are locked (`specs/test-locks.json`). Do not edit a locked test file; add a new test file instead.

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
