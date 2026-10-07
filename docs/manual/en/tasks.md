# Task guides

[日本語](../ja/tasks.md) · [Contents](index.md)

- [Task guides](#task-guides)
  - [Edit values and save](#edit-values-and-save)
  - [Find, filter and sort](#find-filter-and-sort)
  - [Edit many rows at once](#edit-many-rows-at-once)
  - [Create a note](#create-a-note)
  - [Open a .base file](#open-a-base-file)
  - [Shape a view and save it](#shape-a-view-and-save-it)
  - [Give a folder its own command](#give-a-folder-its-own-command)
  - [Use with other commands](#use-with-other-commands)
  - [Choose a theme](#choose-a-theme)
  - [When you cannot edit](#when-you-cannot-edit)

`mdgrid` in the examples is the `target/release/mdgrid` you built, and `/tmp/mdgrid-sample` is the sample vault copied in [Getting started](getting-started.md#open-the-sample-vault).

## Edit values and save

1. Select a cell and press `Enter`. The input depends on the column's type:
   - Text: pick from the values other notes have in this column (when there are at most `candidates` distinct values, 20 by default; otherwise just type), or type a new one.
   - Date: a calendar opens. `←` `→` move by day, `↑` `↓` by week, `PageUp` and `PageDown` by month. You can also type `2026-10-15`, or `+3` for three days from today. `Ctrl+T` is today.
   - Date and time: the same calendar, with a time field under it. `Ctrl+O` switches to the time field, where `↑` `↓` move 15 minutes and `Shift+↑` `Shift+↓` one hour. Changing only the day keeps each note's time; you can also type `2026-10-15T09:30`.
   - List (such as `tags`): toggle values from the list of values found in the vault.
   - Checkbox: each `Enter` cycles empty → true → false → empty.これ
   - Number: only values that read as numbers are accepted.
2. Press `Enter` to confirm, `Esc` to cancel, or `Tab` to confirm and move right.
3. The edited cell is marked and `Unsaved` in the bottom bar goes up. Nothing is written yet. `u` undoes, `Ctrl+R` redoes.
4. `Ctrl+S` (or `:w`) opens the save review: a before/after diff for each file.
5. `Enter` writes everything. `Esc` goes back to the table without writing; the pending changes stay.

![Enter a date with the calendar](images/edit-date.svg)

![Review the diff, then save](images/save.svg)

- To empty a cell press `Backspace` or `Delete`. Saving writes `key:` and keeps the key.
- `q` with unsaved changes asks: `s` save and quit, `d` discard and quit, `Esc` back.
- `K` shows every property of one note (details), long values in full; `Enter` edits one.
- Edit the body or values with line breaks in your editor with `e`; mdgrid re-reads the note when you return. The editor is the `editor` setting, else `$VISUAL`, `$EDITOR`, then `vi`.

mdgrid writes only the values you changed. A missing key gets one added line, and a note without frontmatter gets frontmatter added at the top (the body is untouched). If a note changed on disk before you save, the review shows it and lets you choose to write over it (`o`). The details are in [Write-back safety](../../safety.md).

## Find, filter and sort

| To | Keys |
|---|---|
| highlight a word and move between matches | `/`, type, `Enter`; then `n`, `N` |
| keep only rows containing a word | `\` and type (filters as you type and shows the count); `Esc` clears |
| keep only rows with the selected cell's value | `,` (`*` only highlights them) |
| sort by this column for now | `s` (ascending → descending → off) |
| go to a row number | `:`, the number, `Enter` |
| hide a column / show it again | `-` / `+` |
| move a column / change its width | `H` `L` / `<` `>` |
| freeze the columns on the left while scrolling | `F` |

Search ignores case when the word is all lowercase. `Esc` clears the selection first, then the filter, then the search highlight.

![Quick filter](images/quick-filter.svg)

These sorts and filters are temporary. To keep them, see [Shape a view and save it](#shape-a-view-and-save-it).

## Edit many rows at once

1. Mark rows with `Space` (it marks and moves down). `v` marks a range, `Ctrl+A` all rows.
2. On one of the marked rows, open the cell of the column to change with `Enter` and choose the value.
3. Every marked row gets the same value. Rows that cannot take it (read-only notes, for example) are skipped, with the count and the reason.

In a list column, a value every row has shows as `[x]` and a value only some rows have as `[-]`. Each row keeps its other items.

![Add or remove list items on several rows](images/bulk-tags.svg)

Nothing is written until you save, so the `Ctrl+S` diff lets you check all of it at once.

## Create a note

1. Press `a`, or click `+ New` at the top right.
2. Type a name and press `Enter`. `sub/name` creates it in a subfolder (made if missing).
3. The file is created at once and its row is selected, ready for values.

![The new note](images/new-note-done.svg)

- With two or more folders open, you choose the folder first.
- In a view filtered to a single value, such as `status == "doing"`, the new note gets that value and stays in the view.
- mdgrid does not create notes outside the opened folders or over an existing name, and not at all with `--readonly`.
- The default folder, a name template, preset values and columns to ask for next are set in [`new_note`](../../config.md#new_note).

## Open a .base file

```sh
mdgrid /tmp/mdgrid-sample/タスク.base
mdgrid /tmp/mdgrid-sample/タスク.base --view 期限
```

The table views of the `.base` appear as tabs, and the first one (or the one named by `--view`) opens. Switch with `[` and `]`. mdgrid reads filters, order, sort, groupBy, limit, displayName and formulas; `Enter` on a group heading folds the group.

![Switch views; grouped rows](images/base-group.svg)

- A formula mdgrid cannot evaluate shows a dim `?` instead of a guess; select it to see which formula.
- Views other than table (cards, for example) do not open; mdgrid says why.
- mdgrid never rewrites a `.base` file.

What mdgrid reads — keys, operators, functions — is listed in [Obsidian Bases support](../../obsidian-bases.md).

## Shape a view and save it

`o` opens the view settings. `Tab` moves between sections:

- Columns: which columns to show, and their order
- Filters: add conditions. Choosing a column lists its values with counts, and you tick the values to keep or hide
- Sort and group
- Display: row numbers, zebra stripes, column lines, tabs, the search bar, the settings band

![View settings](images/view-settings.svg)

`Apply` applies it to the table; `Save as` keeps it as an mdgrid view. On the same screen, `Overwrite`, `Rename` and `Delete` manage saved views, and `Reset` returns the settings to the defaults. A saved view becomes a tab next to `Default` and is written to `views.toml` in the config folder. The conditions in effect are shown in the settings band above the table; `f` moves into the band and `Backspace` removes one.

![Save as an mdgrid view](images/native-view-save.svg)

To go back and forth with Obsidian, use the palette (`:`):

- `export_base`: write the current view to a new `.base` you name (an existing `.base` is never rewritten)
- `import_base`: take a `.base` view in as an mdgrid view (the parts it cannot hold are listed as dropped)

With `--readonly`, view settings are not remembered.

## Give a folder its own command

mdgrid remembers each folder (or `.base`) you open: the columns you show and their order and widths, the filters, sorting and grouping set with `o`, the folded groups, and the views you saved as tabs. The next time you open the same folder, all of it comes back. So a shell alias is enough to turn a folder into a command of your own.

Add one line per folder to `~/.zshrc` or `~/.bashrc`:

```sh
alias tasks='mdgrid ~/notes/Tasks'
alias books='mdgrid ~/notes/Books'
alias meetings='mdgrid ~/notes/Meetings --readonly'   # only for looking
alias standup='mdgrid ~/notes/Tasks.base --view "By owner"'   # a .base, opened on one view
```

- Typing `tasks` opens the task table the way you left it.
- What is not remembered: the quick sort from a column header, the quick filter `\`, and anything you change with `--readonly`.
- In fish: `alias --save tasks 'mdgrid ~/notes/Tasks'`.

The output side works the same way with a shell function:

```sh
todo() { mdgrid ~/notes/Tasks --print --format md --filter 'status != "done"' --sort due; }
pick-task() { mdgrid ~/notes/Tasks --pick path; }
```

## Use with other commands

Print the table of a view to standard output without opening the screen (csv by default; notes are not written):

```sh
mdgrid /tmp/mdgrid-sample/タスク.base --print
mdgrid /tmp/mdgrid-sample/タスク.base --print --format json | jq .
mdgrid /tmp/mdgrid-sample --print --format md > table.md
```

When you `--print` a folder, there is no note-name column (the columns are the frontmatter keys only).

Pick rows on screen and print their paths or a column's values. The screen is read-only; `Enter` prints the marked rows (or the selected row) one per line and exits. `q` cancels, prints nothing and exits with status 1.

```sh
mdgrid /tmp/mdgrid-sample --pick path | xargs -o vi    # open the picked notes
mdgrid /tmp/mdgrid-sample --pick status
```

![Pick rows for another command](images/pick.svg)

`y` (or `Ctrl+C`) copies the cell and `Y` the whole row to the clipboard.

## Choose a theme

Add a line such as `theme = "nord"` to the config to change the screen colors. All seven themes and how to choose them are on the [Themes](themes.md) page.

## When you cannot edit

| What you see | Why | What to do |
|---|---|---|
| a row of `#` cells | the note's frontmatter cannot be written safely (a duplicate key, a BOM, mixed line endings, …) | select it to see the reason; fix it in your editor (`e`). All reasons: [Read-only notes](../../safety.md#read-only-notes) |
| `Enter` says read-only | a nested value, a block scalar, a `file.*` or `formula.*` column | [Read-only values](../../safety.md#read-only-values) |
| a value with line breaks cannot be edited | the one-line input cannot hold it | open the note with `e` |
| a date or number is refused | the date does not exist, or the value is not a number | the input stays open; type it again |
| `!` at the start of a row | a conflict file made by a sync tool, or a note that changed on disk while you had pending changes | clean up conflict files by hand; for a changed note, write over it (`o`) or drop your changes (`d`) in the save review |
| nothing can be written | opened with `--readonly` | open again without it |

If a note changes on disk while mdgrid is open, mdgrid re-reads it automatically (not while you are editing a cell). If it changed before you save, the save review shows it.
