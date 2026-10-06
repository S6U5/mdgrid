# mdgrid

**A spreadsheet for your Markdown notes, in the terminal.** mdgrid shows a folder of Markdown files as a table — one row per note, one column per frontmatter key — and lets you edit the values in place. It writes back only the bytes of the values you changed, after showing you the diff.

[日本語](README.ja.md)

![mdgrid: pick a status, pick a date on a calendar, review the diff and save, switch to a grouped view, open the action menu](docs/assets/demo.gif)

- **Edit frontmatter like a table.** Pick from values other notes already use (type to narrow the list), choose dates on a calendar, toggle tags and checkboxes, set one value on many rows at once, add a column for a new key, or rename and delete a key across all notes.
- **Never touches the rest of the file.** Other keys, key order, comments, line endings and the body stay byte-for-byte the same. Every save shows a per-file diff first, writes atomically, and stops if the file changed on disk in the meantime.
- **Speaks Obsidian Bases.** Open a `.base` file and get its table views as tabs, with filters, sorting, grouping, a summary row (`Sum`, `Average`, `Earliest` …) and a subset of formulas. You don't need Obsidian installed, and mdgrid never rewrites your `.base` files.
- **Fits the shell.** `--print` writes the view as CSV, JSON or a Markdown table, with `--filter` and `--sort` for one-off queries. Edit that CSV in a spreadsheet and bring it back with `--apply`: it shows the diff and writes only the cells that changed. `--pick` lets you choose rows on screen and prints their paths for the next command.
- **Keyboard first, and discoverable.** Vim keys and arrow keys both work; every key can be rebound. Press `x` (or right-click a cell) to see the actions available right there, each with its key. The command palette (`:`) and the help screen (`?`) list everything else.
- **Six color themes besides your terminal's own colors.** Set `theme = "nord"`, `"solarized-light"`, `"dracula"`, `"gruvbox"`, `"pink-monster"` or `"dozy-pink"`; the default keeps your terminal's colors.
- **A single binary.** Written in Rust. English and Japanese UI. Respects `NO_COLOR`.

## Install

**Prebuilt binaries.** Each release on the GitHub Releases page has archives for Linux (x86_64, aarch64), macOS (Intel, Apple silicon) and Windows (x86_64). Unpack one and put `mdgrid` on your `PATH`. The archive also holds shell completions (`completions/`) and a man page (`mdgrid.1`).

**From crates.io.** With Rust 1.90 or newer:

```sh
cargo install mdgrid --locked
```

**From source.**

```sh
git clone https://github.com/S6U5/mdgrid.git
cd mdgrid
cargo install --path . --locked
```

mdgrid is not on Homebrew yet.

## Quick start

Try it on the demo vault, a small team's task list (copy it first, so saving does not change the repository):

```sh
cp -R examples/demo /tmp/mdgrid-demo
mdgrid /tmp/mdgrid-demo/Tasks.base     # four table views: Open, By status, By owner, Due
mdgrid /tmp/mdgrid-demo                # every note in the folder
mdgrid "/tmp/mdgrid-demo/Tasks/Fix login redirect bug.md"   # a .md file opens its folder with that note selected
```

The demo's due dates are in early October 2026, so after that most tasks show as overdue. To see it as in the screenshots, set the date mdgrid treats as today: `MDGRID_TODAY=2026-10-03 mdgrid /tmp/mdgrid-demo/Tasks.base`.

Or open your own notes. Add `--readonly` to look without any risk of writing:

```sh
mdgrid ~/notes --readonly
```

| Key | Action |
|---|---|
| `Enter` | Edit the selected cell |
| `x` | Actions for the selected cell |
| `%` | Count the values of the column; pick one to keep only those rows |
| `A` | Add a column for a new key (only the notes you fill in get it) |
| `:rename_key` · `:delete_key` | Rename or delete the column's key in every note (shown in the diff before saving) |
| `Ctrl+S` | Review the diff and save |
| `/` · `\` | Search · filter rows as you type |
| `o` | View settings: columns, filters, sorting, grouping |
| `[` `]` | Switch views |
| `:` | Command palette |
| `?` | Help: every key, and what the marks in cells mean (`∅`, `!`, `*`, `#` …) |
| `q` | Quit |

## A closer look

Pick a value that other notes use, or type a new one. Typing narrows the list to the values that contain what you typed:

![Choosing a status value from a list](docs/assets/demo-edit-list.svg)

Date columns open a calendar:

![Choosing a date on a calendar](docs/assets/demo-calendar.svg)

Nothing is written until you save, and you see exactly what will change:

![The save review showing a one-line diff](docs/assets/demo-save.svg)

Press `%` on a column to count its values, and pick one to keep only those rows:

![Value counts for the owner column](docs/assets/demo-freq.svg)

Press `x` to see what you can do with the selected cell:

![The action menu for a cell](docs/assets/demo-menu.svg)

`.base` views can group rows under headings that you can fold with `Enter`:

![A .base view grouped by status](docs/assets/demo-group.svg)

A view with `summaries` adds a summary row under the table, computed from the rows shown now (in the demo's `Open` view: `Earliest` due date and `Sum` of the estimates).

Pick a color theme in the config (here `theme = "nord"`):

![The table in the Nord theme](docs/assets/demo-theme.svg)

[examples/themes/](examples/themes) has a sample config for each theme: `mdgrid --config examples/themes/nord.toml examples/demo`.

## Use it from scripts

```sh
mdgrid ~/notes/Tasks.base --print                         # CSV with a header row
mdgrid ~/notes/Tasks.base --print --format json | jq .
mdgrid ~/notes --print --format md > table.md
mdgrid ~/notes/todo.md --print --format json             # just this note's row
mdgrid ~/notes --print --with-path                        # first column: each row's note path
mdgrid ~/notes --print --filter 'status != "done"' --sort due   # filter and sort without writing a .base
mdgrid ~/notes --print --with-path > notes.csv             # edit notes.csv in a spreadsheet, then:
mdgrid ~/notes --apply notes.csv                          # show the diff (nothing is written)
mdgrid ~/notes --apply notes.csv --yes                    # write only the cells that changed
mdgrid ~/notes --pick path | tr '\n' '\0' | xargs -0 -o vi   # choose notes, then open them (paths may contain spaces)
```

`--print` prints only the view's columns, and a folder's table has no column naming the note: add `--with-path` to tell which row is which note.

`--print` never writes to your notes. `--apply` checks every row first and writes nothing if any path, value or read-only cell is a problem; without `--yes` it only shows the diff. A value that is the same as what `--print` wrote is never rewritten, so an unchanged file writes nothing. If a note changes on disk while `--apply --yes` is writing, the notes already written stay written and mdgrid exits with code 1. `--pick` opens the table read-only and prints one line per marked row.

## Documentation

- [Configuration](docs/config.md) — every setting, with defaults. `mdgrid --print-config` prints a commented starting file.
- [Keys](docs/keys.md) — the default keys of each mode, and how to rebind them.
- [Obsidian Bases support](docs/obsidian-bases.md) — which parts of `.base` files mdgrid reads, evaluates or ignores.
- [Write-back safety](docs/safety.md) — what mdgrid writes, what it never touches, and which notes stay read-only and why.
- [Showcase](examples/showcase/README.md) — a sample config and vault (in Japanese) that turn most features on. `examples/vault` covers the edge cases: null, empty and mismatched values, read-only notes, sync conflicts.
- Shell completions: `mdgrid --completions <bash|zsh|fish|elvish|powershell>`. Man page: `mdgrid --man`.

## Contributing

Bug reports and ideas are welcome as issues; see [CONTRIBUTING.md](CONTRIBUTING.md) (pull requests are limited to collaborators). Please report security problems privately as described in [SECURITY.md](SECURITY.md).

## How it is built

The specification lives in [specs/](specs/README.md) and every change goes through a recorded proposal and decision (in Japanese). Tests that guard requirements decided by a person are locked, so they cannot be weakened silently.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

mdgrid is an independent project. It is not affiliated with or endorsed by Obsidian.
