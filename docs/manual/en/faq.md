# When something looks wrong

[日本語](../ja/faq.md) · [Contents](index.md)

Answers to the questions people run into most. Press `?` in mdgrid for every key and the meaning of every mark.

- [What do the marks in cells mean?](#what-do-the-marks-in-cells-mean)
- [I cannot edit a cell or a row](#i-cannot-edit-a-cell-or-a-row)
- [My change did not reach the file](#my-change-did-not-reach-the-file)
- [A link shows `?` in front of its name](#a-link-shows--in-front-of-its-name)
- [The relation map shows the wrong tables, or none](#the-relation-map-shows-the-wrong-tables-or-none)
- [A `.base` column shows `?`](#a-base-column-shows-)
- [Columns do not line up, or characters look too wide](#columns-do-not-line-up-or-characters-look-too-wide)
- [The colors are wrong, or there are none](#the-colors-are-wrong-or-there-are-none)
- [A folder opens slowly](#a-folder-opens-slowly)
- [`mdgrid workspace` opens a folder named "workspace"](#mdgrid-workspace-opens-a-folder-named-workspace)
- [How do I start over?](#how-do-i-start-over)

## What do the marks in cells mean?

| Mark | Meaning |
|---|---|
| (blank) | The note has no such key |
| `""` | An empty string |
| `∅` | null: the key is there, with no value |
| `!` before a value | The value does not fit the column's type (shown as it is, never changed) |
| `*` before a value | Changed, not saved yet |
| `#` before a value | Read-only; select the cell to see why |
| `?` (dim) | A `.base` formula mdgrid cannot evaluate |
| `…⏎` | The value has line breaks; only the first line is shown |

On the left of a row: `>` is the row you are on, `+` a marked row (`Space`, `v`), `!` a note changed outside mdgrid (or a sync conflict file), and `~` a row you edited that stays in place until you save or move.

## I cannot edit a cell or a row

Select the cell: the reason appears on the bottom line. The usual ones:

- **The whole row is read-only**: the note has no frontmatter (or an empty one, unless `add_frontmatter` is on), a byte order mark, mixed line endings, invalid YAML, a key written twice, and so on. mdgrid refuses rather than guessing. The full list with examples is in [Write-back safety](../../safety.md#read-only-notes).
- **One cell is read-only**: nested maps, block scalars (`|`, `>`), anchors and tags, `file.*` and `formula.*` columns, and lists mdgrid cannot rewrite without changing their layout. See [Read-only values](../../safety.md#read-only-values).
- **Everything is read-only**: mdgrid was started with `--readonly` or `--pick`.

`e` opens the note in your editor (`editor` in the config, else `$VISUAL`, `$EDITOR`, `vi`); mdgrid reads it again when the editor exits.

## My change did not reach the file

Edits stay pending (marked `*`) until you save. Press `Ctrl+S`, check the diff, and press `Enter`. If you quit with pending changes, mdgrid asks whether to save or discard them. If the note changed outside mdgrid in the meantime, the save review says so for that file and does not overwrite it unless you choose to (`o`).

## A link shows `?` in front of its name

The note it points at was not found. mdgrid looks for the target by name in the tables of the current workspace (or the registered tables, when there is no workspace), and by path for Markdown links and plain paths. Check the spelling, or add the target's folder to the workspace: open it and choose **add this table to a workspace** in the palette (`:`), or run `mdgrid <folder> --add-to <workspace>`.

## The relation map shows the wrong tables, or none

The map shows the tables of the current workspace; the header shows `· workspace <name>` when one is in use. mdgrid picks it in this order: `-w <name>`, a `.mdgrid/workspace.toml` marker in the folder or above it, `workspaces.toml`, the detected Obsidian vault (or git repository with `workspace_detect = ["vault", "git"]`), and finally the registered tables. If the map is empty, register the tables or put them in a workspace. See [Workspaces](concepts.md#workspaces-which-tables-belong-together).

Automatic tables (from a marker without `[[table]]`, or from detection) are the folders of notes right under the root; `.base` files are not added automatically, and folders such as `node_modules` and `target` are skipped. List a `.base` explicitly with `[[table]]` to include it.

## A `.base` column shows `?`

The column is a formula, filter or summary that mdgrid cannot evaluate yet. Select the cell to see which part; [Obsidian Bases support](../../obsidian-bases.md) lists what mdgrid reads. The rest of the view still works.

## Columns do not line up, or characters look too wide

Some characters (`○`, `※`, box-drawing lines) are "ambiguous width": some terminals draw them one column wide, others two. If yours draws them wide, set `ambiguous_wide = true`; the frames switch to ASCII so that everything lines up. If the checkbox or type marks (`☑ ◷ ◉ ⋮`) look odd in your font, set `[cells] checkbox = false` and `icons = false`.

## The colors are wrong, or there are none

- No colors at all: `NO_COLOR` is set, `TERM=dumb`, `--no-color` was given, or `color = false` is in the config.
- Colors look off: your terminal may not support true color; mdgrid then uses the nearest of the 256 colors. Set `COLORTERM=truecolor` if your terminal does support it.
- You prefer the earlier look: `look = "classic"` (and `cells = "plain"`). See [Make it look the way you like](appearance.md).

## A folder opens slowly

The first screen appears at once, and the rest loads in the background with progress in the header; `Ctrl+G` stops loading and keeps what was read. Tens of thousands of notes take a couple of seconds. Opening the relation map reads every note of every table in the workspace, which takes about a second for twenty thousand notes.

## `mdgrid workspace` opens a folder named "workspace"

That is intended: workspace operations are options (`--workspaces`, `--add-to`, `--remove-from`, `--remove-workspace`, `--init-workspace`, `-w`), so a folder named `workspace` opens like any other folder.

## How do I start over?

- One table's look (widths, hidden columns, view): in **View settings** (`o`) choose **Reset**, or delete the table's state in `~/.local/state/mdgrid/`.
- Settings: move `~/.config/mdgrid/config.toml` away; `mdgrid --print-config` prints a fresh one with every default.
- Registered tables and workspaces: edit or delete `places.toml` and `workspaces.toml` in `~/.config/mdgrid/`.

Your notes are never touched by any of these.
