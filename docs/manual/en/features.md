# What mdgrid can do

[日本語](../ja/features.md) · [Contents](index.md)

One screen and a few lines per feature. The how-to is behind each link. [Ways to see your notes](views.md) lists the screens; this page lists what you can do with them.

## By use case

Four starters are built in, and each one is a sample of a way to use mdgrid. The screens below open the sample ledgers made from them (`examples/tasks` and so on).

![The tasks starter](images/starter-tasks.svg)

Tasks. One note per task, with status tabs (Open, Doing, Waiting, Done), due date, priority and tags. A new note asks only for the due date and priority; the status and the created date are filled in for you. With its `start` and `due` columns, `C` shows it as a Gantt too.

![The tree screen](images/tree.svg)

Bookmarks. One note per bookmark, with its place written as `folder: bar > Dev > Web`; `T` shows them as a folder tree, and `o` on the tree screen opens the URL in your browser.

![The reading starter](images/starter-reading.svg)

Reading. Books, articles, films and talks, one note each, with the tabs Queue, Now and Done, and columns for kind, author and rating.

![The people starter: relation map](images/starter-people.svg)

People and meetings. Two linked tables, people (role, organization, contact) and meetings (date, attendees, next actions), with `R` for the relation map.

[Create a ledger from a starter](tasks.md#create-a-ledger-from-a-starter)

## Edit values in place

![Pick a value](images/edit-list.svg)

`Enter` on a cell. Text columns offer the values other notes use, and typing narrows them. Dates get a calendar, tags toggle on and off, checkboxes flip. A value that does not fit the column's type is shown with a `!` and left alone.

[Edit values and save](tasks.md#edit-values-and-save)

## Review the diff and save

![Review the diff, then save](images/save.svg)

Edits are held as pending changes until `Ctrl+S`, which shows a diff per file before writing. Only the lines with changed values are written; other keys, their order, comments and the body stay byte for byte.

[Edit values and save](tasks.md#edit-values-and-save), [Write-back safety](../../safety.md)

## Edit many rows at once

![Set one value on many rows](images/bulk-set.svg)

Mark rows with `Space` and set one value on all of them. Tags can be added or removed in bulk the same way, and marked rows can be handed to the next command.

[Edit many rows at once](tasks.md#edit-many-rows-at-once)

## Find, filter and sort

![Quick filter](images/quick-filter.svg)

`\` narrows the rows as you type, `/` searches, `s` sorts by the column. Keep only the rows with the same value, count a column's values and keep one (`%`), hide the columns you do not need.

[Find, filter and sort](tasks.md#find-filter-and-sort)

## Create a note

![Create a note](images/new-note.svg)

`a` (or "+ New" at the right end of the view bar) opens a form with the name and one field per column and writes one note. The folder, the file name and the body template come from the workspace marker.

[Create a note](tasks.md#create-a-note)

## Shape a view and save it

![View settings](images/view-settings.svg)

`o` sets the columns, filters, sorting, grouping and display; save it under a name and it becomes a tab. Choose "Set as default view" and the folder opens on that view next time.

[Shape a view and save it](tasks.md#shape-a-view-and-save-it)

## See the schedule as a gantt

![The gantt screen](images/gantt.svg)

`C` turns the rows of the current view into bars from the start date to the due date. With parent/child rows and WBS, the numbers and the progress (the fill of the bar) stay the same, and a parent without dates gets the span of its children. `+` and `-` switch between day, week and month; `t` goes to today. The start and end columns come from the marker, or are inferred from date columns named like `start` and `due`.

[Gantt in the list of views](views.md#gantt), [Key bindings](../../keys.md)

## Open a .base file

![Open a .base file](images/base-tabs.svg)

Pass a `.base` file and its table views line up as tabs. Filters, sorting, grouping, summary rows and part of the formulas are read; the `.base` is never written. You can also export the current view to a `.base`, or import a `.base` view.

[Open a .base file](tasks.md#open-a-base-file), [Obsidian Bases support](../../obsidian-bases.md)

## Link tables

![Relation map](images/demo-relmap.svg)

A link such as `project: "[[mdgrid]]"` turns folders into linked tables: pick a target by name, open it, list the notes that point here. Tables that belong together form a workspace, and `R` draws the relation map.

[Link tables with relations](tasks.md#link-tables-with-relations), [Group tables into workspaces](tasks.md#group-tables-into-workspaces)

## Create a ledger from a starter

![Create a ledger from a starter](images/starter-pick.svg)

Pick "Create a ledger from a starter" from the palette (`:`), or run `--init-workspace --starter <name>`. It writes the marker, templates and samples into an empty folder, so you start with a table already shaped. Your own starters go in the same form.

[Create a ledger from a starter](tasks.md#create-a-ledger-from-a-starter)

## Use with other commands

![Pick rows for another command](images/pick.svg)

`--print` writes the table as CSV, TSV, JSON or Markdown; edit the CSV in a spreadsheet and bring it back with `--apply`; `--pick` hands the paths of the chosen notes to the next command. A `.csv` or `.tsv` file opens directly as a ledger. For AI agents there is a skill that teaches these commands.

[Use with other commands](tasks.md#use-with-other-commands), [Open a CSV or TSV as a ledger](tasks.md#open-a-csv-or-tsv-as-a-ledger)

## Open read-only

![Read-only mode](images/readonly-mode.svg)

With `--readonly`, editing, saving and creating notes are off and nothing is written. Notes that cannot be written safely are read-only even in normal use: their cells show a `#`, and selecting one tells you why.

[When you cannot edit](tasks.md#when-you-cannot-edit), [Write-back safety](../../safety.md)

## Choose a look

![The sample configuration](images/showcase-table.svg)

Color themes, the shape of chips and checkboxes, row numbers and column lines, the date format: all set in the config, and overridable per workspace, table or view. Compare them in the look catalog and paste the settings.

[Make it look the way you like](appearance.md), [Themes](themes.md), [Look catalog](../../catalog/index.html)

## Keys and the palette

![The command palette](images/palette.svg)

Vim keys and arrow keys both work. `x` lists what you can do on the cell, `:` finds a command by name, `?` lists every key. All of them can be rebound.

[Key bindings](../../keys.md)
