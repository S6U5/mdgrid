# Ways to see your notes

[日本語](../ja/views.md) · [Contents](index.md)

The same notes can be shown in several screens. Each one is a key away, or a few choices in the view settings. None of them writes to your notes, and `Esc` brings the table back. This page shows one screen per way of seeing; the links tell you how.

## Table

![Open a folder](images/table.svg)

The basic screen. One row per note, one column per frontmatter key, the file name first. `Enter` on a cell edits it in place. The bar at the bottom shows how many changes are unsaved and which keys work right now.

[Getting started](getting-started.md#reading-the-screen) explains the parts of the screen.

## View tabs

![Open a .base file](images/base-tabs.svg)

A view is a named set of columns, filters and sorting. Views sit in tabs above the table; `[` and `]` switch between them. Open a `.base` file and its table views become tabs as they are. Views you make yourself are kept in the config folder, never next to the notes.

[Shape a view and save it](tasks.md#shape-a-view-and-save-it)

## Groups

![Grouped by status](images/demo-group.svg)

Rows gather under heading rows, one per value of a column. `Enter` on a heading folds it. Groups from a `.base` look the same. Handy for going through tasks one status at a time.

[Shape a view and save it](tasks.md#shape-a-view-and-save-it) (the sort and group section of the view settings)

## Tree

![The tree screen](images/tree.svg)

Write `tree = { path = "folder" }` on the table in the workspace file (`.mdgrid/workspace.toml`), and `T` turns the paths in that column (`bar > Dev > Web`) into a folder tree: the tree with counts on the left, the notes of the selected folder on the right. Good for bookmarks and reference material; the built-in bookmarks starter is set up this way. The tree screen is there from 0.4.0.

[The `tree` setting](../../config.md), [Create a ledger from a starter](tasks.md#create-a-ledger-from-a-starter)

## Gantt

![The gantt screen](images/gantt.svg)

`C` turns the rows of the current view into bars on a time axis. The start and end columns come from `gantt = { start = "start", end = "due" }` in the workspace marker, or, without a setting, are inferred from date columns named like `start` and `due`. With parent/child rows and WBS in the view settings, the order, numbers and progress (the fill of the bar) stay the same, and a parent without dates gets the span of its children. `+` and `-` switch between day, week and month, `t` goes to today. The gantt screen is there from 0.4.0.

The `gantt` section of [Key bindings](../../keys.md), [Configuration](../../config.md)

## Parent/child indent and WBS

If notes carry `parent: "[[Parent]]"`, the parent/child section of the view settings indents children under their parent. `Z` folds and opens. Turn on WBS in the same section and rows get `1.2.1` numbers and a progress figure from the status column (a parent shows the average of its descendants). The sample notes have no parents, so there is no screen here; the [release recording](https://github.com/S6U5/mdgrid/releases/latest/download/demo-tree.gif) shows it moving.

[Shape a view and save it](tasks.md#shape-a-view-and-save-it)

## Relation map

![Relation map](images/demo-relmap.svg)

`R` draws the tables of the workspace as boxes and the link columns as arrows. You can see which table points at which, and `Enter` opens the one under the cursor.

[The relation map](concepts.md#the-relation-map)

## Value counts

![Count the values of a column](images/demo-freq.svg)

`%` lists every value of the current column with its count and share. `Enter` keeps only the rows with that value. Use it to see how many tasks are in each status, or whether one person has too many.

The `freq` section of [Key bindings](../../keys.md)

## One note in detail

![Details of one note](images/detail.svg)

`K` lists every key and value of the selected row top to bottom. For the moment when there are more columns than the screen is wide.

The `detail` section of [Key bindings](../../keys.md)

## Filter band

![The settings band](images/chips.svg)

The filters, sorting and grouping in effect sit above the table as chips. `f` moves into the band and `Backspace` removes one. You never lose track of what the table is filtered by.

[Find, filter and sort](tasks.md#find-filter-and-sort)

## Row numbers, stripes and column lines

![Row numbers and column lines](images/display-row-numbers.svg)

The Display section of the view settings adds row numbers, alternating row colors and column lines. Each view remembers its own.

[Shape a view and save it](tasks.md#shape-a-view-and-save-it)

## Frozen columns

![Freeze columns](images/freeze-columns.svg)

`F` pins the columns up to the current one on the left. Scroll sideways and the file name and status stay in view.

The `table` section of [Key bindings](../../keys.md)

## Themes and part shapes

![Theme: Nord](images/theme-nord.svg)

Twelve color themes, and a choice of shapes for chips and checkboxes. Only the look changes; the text and marks stay the same. Compare them in the [look catalog](../../catalog/index.html) and paste the settings it gives you into `config.toml`.

[Make it look the way you like](appearance.md), [Themes](themes.md)

## Easy to step back

`Esc` brings the table back from any screen (`q` as well in the tree and the relation map, `K` from the detail, `f` from the band). On the table itself `q` quits mdgrid. `?` lists the keys, `:` finds a command by name. Switching screens never touches a file; only saving does.
