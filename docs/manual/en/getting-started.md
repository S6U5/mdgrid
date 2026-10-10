# Getting started

[日本語](../ja/getting-started.md) · [Contents](index.md)

## Install

mdgrid is a single executable written in Rust. Download an archive for your OS from the [GitHub Releases](https://github.com/S6U5/mdgrid/releases) page, or install it with Rust 1.90 or newer:

```sh
cargo install mdgrid --locked
```

To build from a clone of the repository instead, run `cargo build --release` to get `target/release/mdgrid`. The examples below use `target/release/mdgrid`; with an installed `mdgrid`, just type `mdgrid`.

It can also print shell completions (bash, zsh, fish, elvish, powershell) and a man page:

```sh
mdgrid --completions zsh > ~/.zfunc/_mdgrid
mdgrid --man > ~/.local/share/man/man1/mdgrid.1
```

## Open the sample vault

The repository has a sample vault, `examples/vault`. Saving rewrites files, so copy it to a temporary folder first:

```sh
cp -R examples/vault /tmp/mdgrid-sample
target/release/mdgrid /tmp/mdgrid-sample
```

To just look without copying, add `--readonly`. Editing, saving and creating notes are then off, and nothing is written:

```sh
target/release/mdgrid examples/vault --readonly
```

### All the samples

Every sample at once, from a clone of the repository. `demos/try.sh` builds mdgrid, copies the sample to a temporary folder, opens it with its own config, and deletes the copy when you quit, so your notes and settings are never touched:

```sh
git clone https://github.com/S6U5/mdgrid && cd mdgrid
sh demos/try.sh demo        # a small team's task list (the README screenshots)
sh demos/try.sh relations   # tasks → projects → members linked by [[links]]; press R for the relation map
sh demos/try.sh workspace   # the same tables grouped into a workspace (-w Work)
sh demos/try.sh ai-human    # a .base that splits tasks into human and AI work
sh demos/try.sh showcase    # a config that turns most features on (Japanese)
sh demos/try.sh vault       # edge cases: empty and mismatched values, read-only notes (Japanese)
```

Anything after `--` goes to mdgrid, for example `sh demos/try.sh demo -- --readonly`.

With no argument mdgrid opens the current folder. You can also pass several folders, or a `.base` file to open its views ([Task guides](tasks.md#open-a-base-file)).

## Reading the screen

![A folder opened in mdgrid](images/table.svg)

From the top:

| Area | What it shows |
|---|---|
| Header | what is open (`vault` here), the number of rows, and `· workspace <name>` when a workspace is in use. At the right, the **Table** / **Relations** tabs (when there are linked tables) and `+ New`, which creates a note |
| Tabs | view names: `Default table` (every note of the folder), plus the views of a `.base` and views you saved |
| Search bar | press `\` to type; rows are filtered as you type |
| Table | one row per note. The first column is the note's path; the others are frontmatter keys |
| Bottom bar | the number of unsaved changes, the current mode, the selected row and column, and the keys you can press now |
| Message line | the result of the last action, or why something is read-only |

Cell marks:

| Mark | Meaning |
|---|---|
| `∅` (dim) | the value is null (just `key:`) |
| `""` | an empty string |
| blank | the note has no such key |
| leading `!` | a value that does not fit the column's type (`someday` in a date column); sorts last |
| `*` | an unsaved change |
| `#` | a read-only cell (a note that cannot be written safely, or a read-only value such as a nested one); select it to see why |
| `!` at the start of the row | a conflict file made by a sync tool, or a note that changed on disk while you had pending changes |
| `~` at the start of the row | an edited row that would now sort or filter elsewhere, kept in place until you save or move |

![Cell marks](images/cell-marks.svg)

Column types come from `.obsidian/types.json`, or else from the first non-empty value. The type decides how you edit: pick from candidates for text, a calendar for dates, toggles for lists, cycling for checkboxes. With colors on, the type also shows: a mark in the column heading (`#` number, `◷` date, `☑` checkbox, `⋮` list, `◉` chips), checkboxes as `☑`/`☐`, and lists and short repeated values as colored chips ([Make it look the way you like](appearance.md#cells-as-parts-or-as-text)).

## The first keys

| Key | Does |
|---|---|
| `↑` `↓` `←` `→` (or `k` `j` `h` `l`) | move the selected cell |
| `Enter` | edit the cell |
| `Ctrl+S` | review the diff and save |
| `u` / `Ctrl+R` | undo / redo |
| `/` | search |
| `\` | keep only rows containing a word |
| `[` `]` | switch views |
| `?` | show every key |
| `:` or `Ctrl+P` | find a command by name |
| `q` | quit (asks first if there are unsaved changes) |

The mouse works too: click a row to select it, and click a cell of the selected row to edit it. Clicking the selected row's file name is the same as `e` (open in the editor). Click a column heading to sort (the view remembers it), drag a column border to resize, and scroll with the wheel.

Help (`?`) lists the keys you can press now first.

![Help](images/help.svg)

If you forget a key, type its name in the palette; each candidate shows its key.

![Find a command by name](images/palette-search.svg)

Every key, and how to rebind them, is in [Key bindings](../../keys.md).

## Configuration

mdgrid works without a config file. To change something, write out every item with its default and a description, and edit from there:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

The file is `$XDG_CONFIG_HOME/mdgrid/config.toml` (or `~/.config/mdgrid/config.toml`), or the one given with `--config <path>`. The UI language is `language` (`auto`, `en`, `ja`); the default `auto` looks at the first non-empty one of `LC_ALL`, `LC_MESSAGES` and `LANG`, in that order, and picks Japanese when it starts with `ja`, English otherwise. Every item is in [Configuration](../../config.md).

Next: [How mdgrid sees your notes](concepts.md), then the [Task guides](tasks.md).
