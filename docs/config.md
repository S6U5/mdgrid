# Configuration

[日本語](config.ja.md)

mdgrid reads one TOML file:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`, or `~/.config/mdgrid/config.toml` when `XDG_CONFIG_HOME` is unset (also on macOS). On Windows, when `XDG_CONFIG_HOME` is unset: `%APPDATA%\mdgrid\config.toml`.
- `mdgrid --config <path>` reads that file instead.

The same folder also holds files that mdgrid writes itself: `ui.toml` (what you pick on the screen, such as the theme in the "Look" section of the view settings), `views.toml` (mdgrid views and per-table settings), `workspaces.toml` (workspaces and favorites; see [Switch between favorite workspaces](manual/en/tasks.md#switch-between-favorite-workspaces)), `places.toml` (registered tables; see [Register the tables you use](manual/en/tasks.md#register-the-tables-you-use-and-switch-between-them)) and `sessions.toml` (named sessions; see [Save what you see as a session](manual/en/tasks.md#save-what-you-see-as-a-session)). mdgrid never rewrites `config.toml`.

Every item is optional. Without a file mdgrid runs with the defaults. Unknown items, values of the wrong type and items written in a place that cannot hold them produce a warning on the message line and are ignored; mdgrid still starts. Every warning has the form `file: item path: reason`. A broken TOML file stops mdgrid with a one-line reason.

To start from the defaults, write them out with comments:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

The output reads back without warnings and behaves exactly like having no file. Items without a default appear as commented examples.

## Two kinds of items

- **Whole-app items** (scope "global only"): the top-level `language`, `editor` and `poll_ms`, and the sections `[terminal]`, `[workspace]`, `[git]`, `[keys]`, `[search]` and `[templates]`. They are written only in `config.toml` (`ui.toml` may hold `[terminal] nerd_font`).
- **The table profile** (scope "global, workspace, table, view"): `use`, `[look]`, `[display]`, `[dates]`, `[edit]` and `[new_note]`. The same shape can be written in every place below, and narrower places override wider ones.
- **Conditions and notifications per ledger** (scope "global, workspace marker"): `[conditions]`, `[[notices]]` and `[[notify]]`. Write them in `config.toml` and at the top of the marker `.mdgrid/workspace.toml`. What a marker holds works only in the tables of that ledger, next to the global ones ([named conditions](#conditions--named-conditions)).

## Overrides per workspace, table and view

| Place | Written by hand | Written by mdgrid |
|---|---|---|
| Global | `config.toml` | `ui.toml` |
| Workspace | the marker `.mdgrid/workspace.toml` (top level), or `[[workspace]]` in `workspaces.toml` (`[workspace.look]` …) | `workspaces.toml` (never the marker) |
| Table | the table entry of the workspace (`[[table]]` + `[table.look]` in the marker, `[[workspace.table]]` + `[workspace.table.look]`) | `[[table]]` in `views.toml` |
| View | — | the view settings (with a mdgrid view, saved in its definition too) |

The value of each item is taken from the narrowest place that writes it (view → table → workspace → global → default). In the same place, what mdgrid wrote wins over what you wrote by hand (`ui.toml` over `config.toml`, `views.toml` over the workspace's table entry). The one exception is the `[look]` written in a workspace marker (top level and `[table.look]`): it is what the ledger's author suggests, so it sits in a place wider than global (just above the defaults). A theme, parts or colors you chose yourself, in `config.toml` or on screen, win over the marker whatever place they are in (the marker's other sections stay at the workspace level). Two exceptions keep looks whole: a `preset` drops the part shapes (`[look.style]`) written in wider places, and a `theme` drops the role colors (`[look.colors]`, not the value colors) written in wider places. `use = "<template>"` lays the template under the values of that place.

`mdgrid [<table>] --print-config --resolved` prints what applies to that table, with a comment above every item naming where the value came from.

```toml
# notes/.mdgrid/workspace.toml — every table of the notes workspace is nord, tasks is dracula
name = "notes"

[look]
theme = "nord"

[[table]]
path = "tasks"

[table.look]
theme = "dracula"
```

The marker can also name the table to open for its root folder: `open = "<table name>"` at the top level (the `name` of a `[[table]]`, or the folder name of an automatic table). Opening the root folder (`mdgrid` there, `mdgrid .`, or its path from elsewhere) then opens that table (in its `view`, unless `--view` is given) instead of every note in the folder; `--print`, `--check` and `--print-config --resolved` use it too. `--raw` opens the folder itself. A subfolder, two or more folders, or a `.md` file do not use it. An `open` that names no table is a warning, and the folder opens as it is.

Open any other table by its name with `--table <table name>` (`mdgrid ~/notes/research --table News`). The marker is looked up from the opened folder (or the current folder) upwards, so the same name works from any folder inside the ledger. The screen, `--print`, `--count`, `--check`, `--new-note`, `--pick`, `--apply`, `--notices`, `--notify`, `--runs`, `--print-config --resolved` and `--page` all treat that table as the opened table, and `--view` looks among its views (without `--view`, its `view`, else its first view). `--new-note` creates the note with that table's `[new_note]` (folder, name head, preset values, body template, required columns). An unknown name, or no marker, stops with one line that lists the marker's table names. With `-w <workspace>`, the name is looked up among that workspace's tables in `workspaces.toml`. It cannot be combined with `--raw`, `-s` or `--continue`.

Opening the folder that holds a table's notes directly (the table's `[new_note] folder`, or the folder of the `file.inFolder` filter of its `.base`; `mdgrid ~/notes/research/News`) also opens that table instead of the bare folder, with its new-note rules, views and value lists. Add `--raw` to see the bare folder.

```toml
# tasks/.mdgrid/workspace.toml — `mdgrid` in tasks/ opens tasks.base, not README.md or AGENTS.md
open = "tasks"

[[table]]
name = "tasks"
path = "tasks.base"
```

A `[[table]]` in the marker can also set up the gantt screen (`C`; `gantt` in [Keys](keys.md)): `gantt = { start = "<column>", end = "<column>" }`, or `days = "<column>"` (the length in days) instead of `end`. Without the setting, mdgrid infers the columns from the names of the date columns (start: start, begin, from, 開始; end: end, due, finish, until, to, 期限, 終了) and the screen title says so. A `gantt` that is not a table, has no `start`, or has neither `end` nor `days` is a warning and the setting is dropped; a column the table does not have is reported once the notes are read. The screen types all take the same shape (`<type> = { <column names> }`), and the screen type menu lists only the types that have a setting or an inference.

A `[[table]]` in the marker can also set up the kanban screen (`B`; `kanban` in [Keys](keys.md)): `kanban = { column = "<column>" }`. The rows become cards in lanes, one lane per value of that column, in the order of the column's allowed values (`[rules.<column>] values`), then any other value as the rows show it, and last **(none)** for rows without a value. Without the setting, mdgrid infers the column from its name (status, state, stage, 状態, ステータス, 段階; any case) and the screen title says so. A list column is never inferred, and naming one in the setting is a warning, because a row would sit in several lanes. A `kanban` that is not a table or has no `column` is a warning and the setting is dropped; a column the table does not have is reported once the notes are read.

Under each card's name, `card = ["<column>", …]` shows the values of those columns, one line each, in the order you write them. The values look as they do in the table cells (glyphs and names from the rules, your date format, the names of linked notes, computed columns), and a column does not have to be in the view. A column with no value on that row gets no line. With `card`, the cards are spaced one line apart, and the rest of the name and the value lines sit two characters in from the name. A name too long for its lane wraps onto a second line and is cut with `…` after that (with or without `card`). A `card` that is not a list of column names is a warning and is dropped (the board still opens, with names only); a column the table does not have is reported once the notes are read.

```toml
[[table]]
name = "Reading"
kanban = { column = "status", card = ["author", "formula.progress", "formula.stars"] }
```

A `[[table]]` in the marker can also set up parents and children: `children = { key = "<column>" }`. Every view of that table then lists each row under the note its `key` column links to (the first link of a list), indented, the same way as turning on the "Tree" section of the view settings. A view where you turn it off in the view settings stays off when you open it again. Besides `key`, every item is optional:

- `status`: the status column. Without it, the `column` of the marker's `kanban`, or else `status`.
- `done`: the done value. `"done"` when left out.
- `closed`: the values that count as finished. `["done"]` when left out; the done value always counts as finished.
- `on_done`: what happens to unfinished descendants when you set a parent's status to the done value. `"complete"` (the default) sets them to the done value too, with the column's `then` values, in the same step as the parent, so one undo takes it all back. `"refuse"` leaves the parent as it is while descendants are unfinished and says how many. `"keep"` sets only the parent and shows how many are left on the bottom line.
- `closed_by`: the column where descendants finished together with their parent get a link to that parent (`[[Design]]`), in the same step as the parent, so one undo takes it back. Children that were already finished get nothing. Left out, nothing is written (there is no default column name, so no column appears in your notes unasked).
- `on_reopen`: what happens to the descendants whose `closed_by` points to a finished parent when you set that parent back to an unfinished value. `"keep"` (the default) leaves them as they are. `"reopen"` sets them to the parent's new value and empties `closed_by`, in the same step as the parent. `"ask"` opens a small window to choose "Bring the children back too" or "Only the parent" (`Esc` changes nothing). `"reopen"` or `"ask"` without `closed_by` is a warning and works as `"keep"`, since the children to bring back are unknown.
- `gantt`: what the gantt screen does when a descendant's bar runs outside the dates of a parent that has its own dates. `"mark"` (the default) puts a `!` at the right end of the name of the child's and the parent's rows, and the bottom line of a selected row says by how many days. `"extend"` moves the parent's start or end to cover the child when you move or stretch the child's bar, in the same step (moving a parent does not move its children). `"none"` does nothing.

Descendants are followed through the links of every row of the table, whatever the view filters or groups. When you edit a cell of the `key` column, or that field in the new-note window, the notes of the same table are offered as parents even before any note has one. The row itself and its descendants are not offered (choosing one would make a loop). Finishing a parent only touches its descendants when you edit on the screen (a cell, a bulk edit, a kanban card, `.` in vim mode), not with `--apply`, and not in a table where only the view settings turned parent/child on. A `key` column the table does not have (no note has a value in it, the `.base` `properties` give it no name, and no view lists it in `order`) is reported once the notes are read (in the form `.mdgrid/workspace.toml: table[1].children.key: …`), and the rows are not arranged as parents and children. A `children` that is not a table or has no `key` is a warning and the setting is dropped; another item with a wrong value is a warning and that item keeps its default.

```toml
# tasks/.mdgrid/workspace.toml — keep a parent open while children are left, and stretch it with its children
[[table]]
name = "tasks"
path = "tasks"
children = { key = "parent", on_done = "refuse", gantt = "extend" }
```

When a table has an order it should always open in, such as news newest first or expenses by date, write a default sort in the marker's `[[table]]`: `sort = [{ property = "<column>", direction = "DESC" }, …]`. It has the same shape as the `sort` of a `.base` view; `direction` is `ASC` or `DESC` (any case; ascending when left out). The rules apply in the order written, and each later rule orders the rows the earlier ones leave tied.

It applies to the views of that table that have no `sort` of their own (views in a `.base` file, `[[table.base.views]]` in the marker, mdgrid views without a sort) and to a table without views. The `--print` output, the cards inside a kanban lane and the gantt rows follow the same order. A view with its own `sort` keeps it. When you sort with `s` on a heading or in the sort window (`S`), your rules come first and rows they leave tied stay in the view's `sort` order (or the table default): in a table that defaults to due dates, sorting by status keeps each status group in due-date order. The sort window and the sort section of the view settings label rules that come from a default "default · marker" (the table default) or "default · view" (the view's `sort`), and once you change the sort on the screen, "Back to default" in the window undoes it.

Only the marker's `[[table]]` takes it (like the screen settings); `workspaces.toml` and `views.toml` do not, and the global config has no such setting because every table has different columns. A `sort` that is not an array is a warning and is not used; a rule without `property`, or with a `direction` other than `ASC` or `DESC`, is a warning and only that rule is dropped. A column the table does not have is reported once the notes are read (in the form `.mdgrid/workspace.toml: table[1].sort[1].property: …`).

```toml
# research/.mdgrid/workspace.toml — news newest first, same day by file name
[[table]]
name = "News"
sort = [{ property = "date", direction = "DESC" }, { property = "file.name", direction = "ASC" }]
```

A `[[table]]` in the marker can also set up the chart screen (`P`; `chart` in [Keys](keys.md)): `chart = { kind = "bar", x = "status" }`. Every key is optional. `kind` is the kind (`bar`, `stacked`, `line`, `scatter`, `heatmap`; bar when left out), `x` the column to split by, `y` a number column for the value (a count when left out; for a scatter, the vertical column, a number or a date), `agg` the aggregate (`count`, `sum`, `avg`, `min`, `max`; sum when `y` is given), `by` the breakdown column (needed by stacked bars and heatmaps and guessed when left out; optional for lines and scatters), `unit` the date range (`day`, `week`, `month`; the finest that fits when left out), `cumulative = true` a cumulative line, and `name` a name for the title. For several charts, write an array (`chart = [ { … }, { … } ]`) and move between them with `[` and `]` on the screen:

```toml
[[table]]
name = "tasks"
path = "tasks"
chart = [
  { name = "By status", x = "status" },
  { name = "Due by month", kind = "line", x = "due", unit = "month" },
  { name = "Owner and status", kind = "heatmap", x = "assignee", by = "status" },
]
```

Left-out columns are guessed (the split column is the first of: a status column, a column with allowed values, a checkbox, a list, a date, the first column). A chart with an unknown key or value, or a column name that is not a string, is a warning and is left out; a chart naming a column the table does not have is reported and left out once the notes are read. Kinds and columns changed on the screen are not written back to the marker.

A `[[table]]` in the marker can also set up the tree view (`T`; `tree` in [Keys](keys.md)): `tree = { path = "<column>", separator = "<separator>", order = "<column>" }`. The value of the `path` column, split at `separator` (default ` > `), is where the row sits in the tree, and the `order` column (optional) sorts the rows in a folder. When the column holds a list, its first item is used. A `path` column the table does not have, an empty separator, or a `tree` that is not a table is a warning (in the form `.mdgrid/workspace.toml: table[1].tree.separator: …`), and the tree is not offered.

```toml
# bookmarks/.mdgrid/workspace.toml — see notes whose folder is like "bar > dev > Web" as a folder tree
[[table]]
name = "Bookmarks"
path = "Bookmarks"
tree = { path = "folder", order = "order" }
```

A `[[table]]` in the marker with `runs` becomes a routine table: a table of recurring work (a weekly review, a monthly invoice), where each row's rule makes one note per run in another folder. `runs = { folder = "<folder>", rule = "<column>", name = "<name template>", set = { <column> = <value> }, copy = ["<column>"], body = "<template file>", where = { <column> = ["<value>"] } }`. Only `folder` (where the run notes go, relative to the marker's root) and `rule` (the column with the rule) are needed. `name` is the name template of a run note (default `{routine} {date:YYYY-MM-DD}`; `{routine}` is the routine note's name, `{date}` the date of the run, and the other variables are those of [`new_note`](#new_note)), `set` the values to put in it, `copy` the columns whose routine values it copies, `body` the body template (relative to the marker's root; default one heading with the run's name), and `where` the routines that get runs (the same form as a view's `where`; with `where = { status = ["active"] }` a paused routine keeps its rule and gets no runs). The frontmatter of a run note is the link `routine` (`"[[name]]"`), the run's date `date`, then `set` and `copy`. A rule is written `daily`, `weekly mon thu`, `monthly 1 15`, `monthly last` or `every 3d` (counted from the date in the `start` column), or the Japanese forms `毎日`, `毎週 月 木`, `毎月 1日 15日`, `毎月 末日` and `3日ごと`. Only the latest run up to today is made, the runs made are remembered in `routine.json` in the state folder, and a note with the same name is never overwritten. The open screen shows the names and asks before it creates them; `mdgrid <ledger> --runs` creates them without asking ([guide](manual/en/tasks.md#create-routine-runs)). A `runs` that is not a table, or has no `folder` or `rule`, is a warning (in the form `.mdgrid/workspace.toml: table[1].runs.folder: …`), and that table makes no runs.

```toml
# notes/.mdgrid/workspace.toml — make run notes in Runs/ from the rules written in the routine table
[[table]]
name = "Routines"
path = "Routines"
runs = { folder = "Runs", rule = "recurrence", set = { status = "todo" }, copy = ["actor"] }
```

A workspace can also hold **pages**: named arrangements of tables stacked top to bottom (`[[page]]` in a marker, `[[workspace.page]]` under the workspace in `workspaces.toml`). Write `name` and the list `tables`, top first; each entry names a table of the scope in `table`, the view to open in `view` (default: the table's usual view) and a height ratio in `height` (an integer of 1 or more; default 1). **Save the arrangement as a page** in the command palette writes the same shape (replacing a page of the same name); in a marker it only appends at the end of the file and leaves every other line and comment as it was. Pages are listed above the tables in the table list drawer, and `mdgrid -w <workspace> --page <name>` (for a marker workspace, `mdgrid <folder> --page <name>`) opens one. A row that cannot be read (a page with no name, a table that is not in the scope, …) is a warning in the form `.mdgrid/workspace.toml: page[1].tables[2].table: …` and is skipped.

```toml
# notes/.mdgrid/workspace.toml — in the morning, compare today's tasks with the active projects
[[page]]
name = "morning"
tables = [
  { table = "tasks", view = "Today", height = 2 },
  { table = "projects", view = "Active", height = 1 },
]
```

The top of a marker can also hold [named conditions](#conditions--named-conditions), [notice items](#notices--the-notice-box) and [notifications](#notify--notifications) (`[conditions.<name>]`, `[[notices]]` and `[[notify]]`, in the same shape as in `config.toml`). They work only while a table of that ledger is open, next to the global ones. A notice `note` in a marker must be a file inside the ledger (`~`, absolute paths and `..` that leave it are warned about and ignored).

```toml
# people/.mdgrid/workspace.toml — a condition only for the people ledger; here 1 is "time to get in touch"
name = "People"

[conditions.contact]
label = "Time to get in touch"
expr = 'formula.contact <= today()'
key = 1
```

A marker does not have to be written by hand. A **starter** is a marker plus the note templates (and a few sample notes) for a common kind of ledger, and `mdgrid <folder> --init-workspace --starter <name>` writes it into a new folder: the folder is created if missing, `--with-samples` adds the sample notes, and nothing is ever overwritten (if one target file exists, nothing is written). `mdgrid --starters` lists them, one per line: the built-in `tasks`, `tasks-ai`, `bookmarks`, `reading` and `people`, and your own. The command palette has the same as **Create a ledger from a starter** (pick one, type the folder, answer whether to add the samples). Your own starters live in the config folder under `starters/<name>/`, in the same shape as the built-in ones:

```
starters/kanban/
  starter.toml                  # [name] and [description], one key per language code (en, ja)
  ledger/                       # copied as is, relative to the ledger's root
    .mdgrid/workspace.toml
    .mdgrid/templates/card.md   # the body template named by [new_note] (its {date} is filled when a note is made)
  samples/                      # copied only with --with-samples; {date}, {date+N}, {now} … are filled when copied
    Board/First card.md
  ledger.ja/  samples.ja/       # optional: used instead of ledger/ and samples/ when the language is ja
```

A starter with the same name as a built-in one replaces it. The quickest way to make one is to copy the `.mdgrid/` of a ledger you like into `starters/<name>/ledger/`; [examples/places/mdgrid/starters/journal](../examples/places/mdgrid/starters/journal) is one. The starter's folder itself may be a symbolic link (as dotfiles tools make it), but links inside it are not followed, so link the whole folder rather than each file.

## Old names (0.3.0)

Configurations written for 0.3.0 still work: each old name is read as its new path with a warning. `mdgrid --migrate-config` prints your configuration in the current layout (comments are not carried over). The old `look.toml` is read when there is no `ui.toml`, and is moved to `ui.toml` the next time mdgrid writes it; `[[target]]` in `views.toml` becomes `[[table]]` the same way.

| Old name | New path |
|---|---|
| `color` | `terminal.color` |
| `ambiguous_wide` | `terminal.ambiguous_wide` |
| `nerd_font` | `terminal.nerd_font` |
| `workspace_detect` | `workspace.detect` |
| `theme` | `look.theme` |
| `theme_light` | `look.theme` |
| `theme_dark` | `look.theme` |
| `look` | `look.mode` |
| `borders` | `look.style.frames` |
| `style` | `look.style` |
| `style.preset` | `look.preset` |
| `cells` | `look.cells` |
| `cells.style` | `look.cells` |
| `cells.checkbox` | `look.style.check` |
| `cells.chips` | `look.style.tags` |
| `cells.select` | `look.style.status` |
| `cells.links` | `look.style.links` |
| `cells.icons` | `look.style.icons` |
| `cells.columns` | `look.columns` |
| `colors` | `look.colors` |
| `view_tabs` | `display.tabs` |
| `search_bar` | `display.search_bar` |
| `date_format` | `dates.format` |
| `week_start` | `dates.week_start` |
| `candidates` | `edit.candidates` |
| `add_frontmatter` | `edit.add_frontmatter` |

## Items

The items below are the full list. Tests check that this list (paths, the `Type`, `Default` and `Scope` lines, and the examples), the items mdgrid reads, and the `--print-config` output agree.

## Top level (the whole app)

### `language`

- Type: `string ("auto" or a language code such as "en" or "ja")`
- Default: `"auto"`
- Scope: global only

Language of the screen and the startup messages (`--help`, reasons for not starting, warnings). The languages are English (`en`), Japanese (`ja`), German (`de`), Spanish (`es`), French (`fr`), Korean (`ko`), Brazilian Portuguese (`pt-BR`), Simplified Chinese (`zh-CN`) and Traditional Chinese (`zh-TW`). `"auto"` looks at `LC_ALL`, `LC_MESSAGES` and `LANG` in this order and uses the first one that is not empty: the language whose language and region match (`zh_TW.UTF-8` gives `zh-TW`), else the one whose language matches (`de_AT.UTF-8` gives `de`; Chinese without Taiwan, Hong Kong or Macau gives `zh-CN`, and any Portuguese gives `pt-BR`), otherwise English (also when none is set). A language code chooses that language regardless of the environment; an unknown code is warned about and read as `"auto"`. Note values, column names, the contents of `.base` files and file names are never translated. Errors in the command line arguments are reported before the configuration is read, so they follow the environment.

```toml
language = "en"
```

### `editor`

- Type: `string`
- Default: none
- Scope: global only

Editor that opens the selected note (`e`, or a click on the selected row's file name). Arguments are allowed, such as `"code -w"`; the value is split into words without a shell (quotes and backslashes work as in POSIX), and the note path is added as one argument. When this is unset, empty, or only spaces, mdgrid uses `$VISUAL`, then `$EDITOR`, then `vi` (`notepad` on Windows) — the first one that is not empty. After the editor exits, mdgrid re-reads the note, so the editor must stay open until you close the note: an editor that returns at once (such as Windows Notepad with tabs, when it is already open) needs a waiting form in `editor`, for example `"code -w"`. On Windows, write a path with forward slashes (`editor = "C:/tools/nvim.exe"`) or inside quotes: a backslash outside quotes escapes the next character, so `C:\tools\nvim.exe` would not start.

```toml
editor = "nvim"
```

### `poll_ms`

- Type: `integer (1 or more)`
- Default: `1000`
- Scope: global only

Interval in milliseconds for re-reading notes that changed outside mdgrid.

```toml
poll_ms = 2000
```

### `use`

- Type: `string (a template name)`
- Default: none
- Scope: global, workspace, table, view

Lay the template `[templates.<name>]` under the values written in the same place. In config.toml it is the base of the whole app; in a workspace, a table or a view it is the base of that place. A name that no template has gives a warning and is ignored.

```toml
use = "night"
```

## `[terminal]` — the terminal

### `terminal.color`

- Type: `boolean`
- Default: `true`
- Scope: global only

Use colors. `false` draws without colors, the same as the `NO_COLOR` environment variable or `--no-color`.

```toml
[terminal]
color = false
```

### `terminal.ambiguous_wide`

- Type: `boolean`
- Default: `false`
- Scope: global only

Treat East Asian Ambiguous characters (such as `○` and `※`) as two columns wide. Set `true` if your terminal draws them wide, so that the columns line up. Window frames are then always ASCII.

```toml
[terminal]
ambiguous_wide = true
```

### `terminal.nerd_font`

- Type: `boolean or "auto"`
- Default: `"auto"`
- Scope: global only

Whether round pill ends (Nerd Font glyphs U+E0B6 and U+E0B4) can be drawn. `true`: your terminal font is a Nerd Font (any Nerd Font works). `"auto"`: only in terminals that draw these glyphs themselves without a font, Ghostty and WezTerm (by `TERM_PROGRAM`), so nothing breaks in other terminals. Without round ends, `"pill"` is drawn as `"soft"`, and soft and solid pills keep square ends. A terminal cannot tell programs which font it uses, so for other terminals with a Nerd Font, write `true`. The "Look" section of the view settings writes this item to ui.toml.

```toml
[terminal]
nerd_font = true
```

### `terminal.nerd_icons`

- Type: `boolean`
- Default: `false`
- Scope: global only

Use icon glyphs from a Nerd Font for the settings button (gear), the screen type button and menu (table, relation map), the type marks of column headings and relation map boxes (number, date, date and time, checkbox, list, chip columns, and link columns in the boxes) and the calendar title. Set `true` only if your terminal font is a Nerd Font: other fonts show empty boxes for these glyphs. By default plain characters are used (`⚙`, `▦`, `◇`, `#`, `◷`, `⋮`, `◉`, `↪`, and the check mark of the part shape, `✓` by default). Emoji are never used. The glyphs are counted with the same character widths as the rest of the screen.

```toml
[terminal]
nerd_icons = true
```

### `terminal.herdr`

- Type: `boolean`
- Default: `true`
- Scope: global only

Inside a [Herdr](https://herdr.dev) pane, tell Herdr the command that reopens the table and view you are looking at, and tell it again whenever you move to another table or view. When the Herdr server stops and starts again (after a restart of the computer, for example), Herdr runs that command in the same pane, so mdgrid comes back on the same table and view. When you quit mdgrid yourself, the pane is handed back and nothing is reopened. The Herdr sidebar shows the table and view as the pane title (such as `Tasks · In progress`, using the name you gave a workspace table), and the title goes away when you quit. `false` never calls Herdr, so no title either. Outside Herdr this item does nothing. See also `mdgrid --continue` in the [manual](manual/en/features.md).

```toml
[terminal]
herdr = false
```

## `[workspace]` — workspaces

### `workspace.detect`

- Type: `array of strings`
- Default: `[".obsidian"]`
- Scope: global only

When the table you open is in no written workspace (`-w`, a `.mdgrid/workspace.toml` marker, or `workspaces.toml`), treat the nearest folder above it that has one of these marker folders as a workspace (the root folder): `".obsidian"` (a folder with `.obsidian/`) or `".git"` (a git repository). Its folders with notes become the tables of the relation map. `[]` turns detection off. The old values `"vault"` and `"git"` are read the same way, with a warning that they moved. A detected workspace has no workspace place for overrides.

```toml
[workspace]
detect = [".obsidian", ".git"]
```

## `[git]` — git after a save

When your notes live in a git repository, mdgrid can hand the notes it wrote to git after a save. By default it does nothing. Only the notes mdgrid wrote (or created) are handed over; other changed or staged files never enter the commit. This section goes in `config.toml` only (not in a ledger's marker, so that a copied ledger cannot make mdgrid push).

### `git.auto`

- Type: `"off", "add", "commit" or "push"`
- Default: `"off"`
- Scope: global only

How far to go. `"off"` does nothing. `"add"` stages the notes it wrote. `"commit"` also commits them (only those notes). `"push"` also pushes. Folders that are not git repositories are left alone. If git fails (no push target, for example), mdgrid says so in one line and the save itself stays done.

```toml
[git]
auto = "commit"
```

### `git.when`

- Type: `"save" or "quit"`
- Default: `"save"`
- Scope: global only

When it runs. `"save"` runs after every save (one commit per save). `"quit"` runs once when mdgrid exits (one commit for the session). Notes created from the screen count too, and notes still waiting at exit are handled then with either setting.

```toml
[git]
when = "quit"
```

### `git.message`

- Type: `"auto" or "ask"`
- Default: `"auto"`
- Scope: global only

The commit message. `"auto"` writes "mdgrid: update N notes" on the first line, then a blank line and the paths from the repository root, one per line. `"ask"` opens a one-line field after each save and uses what you type as the first line (Enter on an empty field uses the automatic message; Esc skips the commit and leaves the notes for the commit at exit).

```toml
[git]
message = "ask"
```

## `[look]` — theme, parts and colors

### `look.theme`

- Type: `string ("auto", "default", "nord", "solarized-light", "dracula", "gruvbox", "pink-monster", "dozy-pink", "sumi", "slate", "saas", "saas-dark" or "paper") or table ({ light, dark })`
- Default: `"default"`
- Scope: global, workspace, table, view

Color theme of the screen: `"default"`, `"nord"`, `"solarized-light"`, `"dracula"`, `"gruvbox"`, `"pink-monster"`, `"dozy-pink"`, or one of the quieter sets `"sumi"`, `"slate"`, `"saas"`, `"saas-dark"` and `"paper"`. A table `{ light = "...", dark = "..." }` picks by the terminal's background (read from `COLORFGBG` or by asking the terminal; when it cannot tell, the dark one); a side you leave out uses its default. `"auto"` is `{ light = "saas", dark = "sumi" }`. `"default"` keeps the terminal's own foreground and background. The other themes paint the background, the text, the header line, the column headings, the selected cell, the bottom band, the shaded rows and the `>` marks with their own colors; colors that carry meaning (pending values, search matches, added and removed lines) also get readable theme colors. Without colors (`--no-color`, `NO_COLOR`, `TERM=dumb`, `terminal.color = false`) the theme is ignored. On terminals without true color the nearest of the 256 colors is used. An unknown name gives a warning and the default look. A theme written in a narrower place drops the role colors (`[look.colors]`) written in wider places.

```toml
[look]
theme = { light = "paper", dark = "sumi" }
```

### `look.preset`

- Type: `string ("sumi", "slate", "saas", "paper", "grid", "classic" or "dozy-pink")`
- Default: `"sumi"`
- Scope: global, workspace, table, view

A whole set of part shapes: `"sumi"` (the default: a colored dot before a status, tags separated by `·`, `✓`, the selected row's `>` mark in the accent color, a line under the headings), `"slate"`, `"saas"`, `"paper"`, `"grid"`, `"classic"` (the look of version 0.2.0) or `"dozy-pink"` (soft round pills to go with the dozy-pink theme). `[look.style]` overrides single parts. A preset written in a narrower place drops the part shapes written in wider places.

```toml
[look]
preset = "saas"
```

### `look.mode`

- Type: `string ("modern" or "classic")`
- Default: `"modern"`
- Scope: global, workspace, table, view

The look when colors are on: `"modern"` (lazygit-like: the selection is tinted instead of inverted, window borders and keys use an accent color, and descriptions are dimmed) or `"classic"` (inverse video, as before). The text on screen is the same either way, and without colors both look like classic.

```toml
[look]
mode = "classic"
```

### `look.cells`

- Type: `string ("rich" or "plain")`
- Default: `"rich"`
- Scope: global, workspace, table, view

How table cells look when colors are on. `"rich"` shows them as parts in the shapes of `[look.style]`: booleans (`check`), each list item (`tags`), the values of short text columns that repeat a few values such as `status` (`status`), links in the accent color (`links`), and a type mark in each column heading (`icons`). A value always keeps the same color, and its text is always shown. `"plain"` shows the text as it is. Without colors cells are always plain. The values written to notes, edited, searched and printed (`--print`) do not change.

```toml
[look]
cells = "plain"
```

### `look.style`

- Type: `table (status, tags, check, select, rules, tabs, frames, band, links and icons)`
- Default: none
- Scope: global, workspace, table, view

Override single part shapes of the preset:

- `status` (short repeated values such as status or owner): `"dot"`, `"shape"` (○ ◐ ●), `"text"`, `"pill"`, `"tint"`, `"solid"` (the value color as a solid fill), `"soft"`, `"chip"`, `"plain"`
- `tags` (lists): `"dots"`, `"hash"`, `"brackets"`, `"pill"`, `"tint"`, `"solid"`, `"soft"`, `"chip"`, `"plain"`
- `check` (booleans): `"box"`, `"tick"`, `"bracket"`, `"text"`
- `select` (the selected row and cell): `"bar"`, `"cross"`, `"tint"`, `"outline"`, `"fill"`, `"reverse"`
- `rules` (lines in the table): `"none"`, `"header"`, `"columns"`, `"grid"`
- `tabs`: `"underline"`, `"pill"`, `"segment"`, `"brackets"`, `"dim"`
- `frames` (windows): `"rounded"`, `"square"`, `"heavy"`, `"ascii"`, `"none"`
- `band` (the keys at the bottom): `"keys"`, `"boxed"`, `"quiet"`
- `links`: `"accent"` or `"plain"`
- `icons`: `true` or `false` (type marks in the column headings)

`"plain"` (`"text"` for `check`, `false` for `icons`) keeps that part as text. `"pill"` draws round ends with Nerd Font glyphs, so it needs round ends (`terminal.nerd_font`); otherwise it is drawn as `"soft"`. The catalog at `docs/catalog/index.html` shows every choice side by side and writes this table for you.

```toml
[look.style]
status = "chip"
select = "cross"
```

### `look.columns`

- Type: `table (column = "rich", "plain" or "chip")`
- Default: none
- Scope: global, workspace, table, view

How single columns look: `"rich"` (parts even when the part shape is plain), `"plain"` (text) or `"chip"` (chips even if the column is not detected as one). A column setting wins over the part shapes.

```toml
[look.columns]
status = "plain"
owner = "chip"
```

### `look.colors`

- Type: `table (color roles, and [look.colors.values] for value colors)`
- Default: none
- Scope: global, workspace, table, view

Override the theme's colors by role. Roles: `background`, `text`, `header` (first line), `selection` (selected row), `selection_text`, `band` (key band), `band_text`, `accent` (headings, keys, frames), `strong`, `zebra`, `zebra_text`, `mark`, `added`, `removed`, `pending` (unsaved changes) and `highlight` (search matches). Roles you leave out keep the theme's color. `[look.colors.values]` sets a color per value (matched ignoring case and surrounding spaces); it is used by every part shape, in the value list and for the value in group headings and kanban lane titles (drawn as the same chip as in the cells), and works for values in any language. Colors are `"#rrggbb"`, `"#rgb"` or a name: `black`, `white`, `gray`, `red`, `orange`, `yellow`, `green`, `teal`, `cyan`, `blue`, `purple`, `magenta`, `pink`, `brown`. With `theme = "default"`, the terminal's own background and text stay, and only `accent`, `selection` and value colors are used. Unknown roles and unreadable colors give a warning and are skipped; so does a value written twice. A theme written in a narrower place drops the role colors of wider places (value colors stay).

```toml
[look.colors]
accent = "#e0a458"

[look.colors.values]
done = "green"
```

## `[display]` — how the table is shown

### `display.row_numbers`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

Numbers 1, 2, 3… on the left of each row, in the order shown (after filters and sorting). Group heading rows get no number; numbering continues across groups. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
row_numbers = true
```

### `display.zebra`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

Every other row gets a shaded background. Not drawn without colors. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
zebra = true
```

### `display.column_lines`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

Draw `│` between columns. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
column_lines = true
```

### `display.group_gap`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

In a grouped view, leave an empty line above every group heading but the first. The empty line is not a row: it is not counted or numbered, the cursor skips it, and clicking it does nothing. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
group_gap = true
```

### `display.tabs`

- Type: `string ("always", "auto" or "never")`
- Default: `"always"`
- Scope: global, workspace, table, view

The row of view tabs below the header: `"always"`, `"auto"` (only with two or more views; a folder opened without a `.base` has one view, **Default table**, so its tab row is left out and the table uses that row until you save a view of your own) or `"never"`. `[` and `]` switch views either way. In the "Display" section of the view settings this item cycles through the three values. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
tabs = "auto"
```

### `display.search_bar`

- Type: `boolean`
- Default: `true`
- Scope: global, workspace, table, view

Show a search bar above the table (`\` moves into it). `false` hides the bar; the search term is then typed on the bottom line. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
search_bar = false
```

### `display.chips`

- Type: `boolean`
- Default: `true`
- Scope: global, workspace, table, view

Show the band of active view settings. When hidden, `f` still selects its items. Each view can override this in its view settings (`o`, the "Display" section).

```toml
[display]
chips = false
```

### `display.buttons`

- Type: `string ("plain" or "box")`
- Default: `"plain"`
- Scope: global, workspace, table, view

How the search bar and the **Filter** and **Sort** buttons in the view bar are drawn. `"plain"` shows them by their background. `"box"` draws each of them in its own box of lines, without filling it; the lines go on the padding lines next to the middle line of the view bar (see `display.bar_padding`), and clicking anywhere in a box works. With no padding, no lines are drawn. In the "Display" section of the view settings this item switches between the two values, and each view can override it there (`o`).

```toml
[display]
buttons = "box"
```

### `display.bar_padding`

- Type: `number (0 to 2 in steps of 0.5)`
- Default: `1`
- Scope: global, workspace, table, view

Padding above and below the view bar (the view tabs, the search bar and the buttons sit on its middle line), in lines: `0`, `0.5`, `1`, `1.5` or `2`. Both `1` and `1.0` work. The default is `1`: a half line looks different depending on the terminal and the theme (see below), so it is never the default. `0` draws the view bar on one line. Terminals can only leave whole lines, so a half line is drawn as a line next to the view bar whose cells get the view bar's background color, with its outer half covered by half blocks in the screen's background color (`▀` above, `▄` below); the view bar line itself gets the same background. The screen's background color is the theme's when the theme paints the screen; otherwise mdgrid asks the terminal for its background color once at startup. When that color is unknown, without colors, and with `terminal.ambiguous_wide = true` (half blocks may take two columns there), the half line is rounded up to an empty line. Other values produce a warning and the default is used. Terminals shorter than 16 lines, and a hidden search bar, always use `0`. In the "Display" section of the view settings this item cycles through 0, 0.5, 1, 1.5 and 2, and each view can override it there (`o`).

```toml
[display]
bar_padding = 0.5
```

### `display.top_padding`

- Type: `number (0 to 2 in steps of 0.5)`
- Default: `1`
- Scope: global, workspace, table, view

Padding above the header, in lines, with the same values as `display.bar_padding`. A half line is drawn the same way: a line right above the header in the view bar's background color, with its upper half covered by `▀` in the screen's background color; the header then gets the view bar's background color. Terminals shorter than 16 lines always use `0`. In the "Display" section of the view settings this item cycles through the values, and each view can override it there (`o`).

```toml
[display]
top_padding = 0
```

### `display.row_extra_lines`

- Type: `integer (0 to 9)`
- Default: `0`
- Scope: global, workspace, table, view

How many extra lines a table row may grow by. A row holding a long value, or a value with line breaks, wraps it at the column width (by character, counting widths like the rest of the table) and grows up to 1 + this many lines. Each row is only as tall as its tallest cell needs, so rows with short values stay on one line. A line break starts a new line; what does not fit ends with `…` on the last line. List values drawn as chips, group headings and the gaps between groups stay on one line. Lines below the first carry no row marker (`>`) and no row number; column lines and row colors cover the whole row. Scrolling, paging and mouse clicks count the grown lines, and the selected row scrolls into view with all its lines. The edit box and its pop-ups open on the row's first line. `0` keeps every row on one line, with line breaks shown as `…⏎`. Other values produce a warning and the default is used. In the "Display" section of the view settings this item cycles through 0, 1, 2 and 3, and each view can override it there (`o`).

```toml
[display]
row_extra_lines = 2
```

### `display.tab_wrap`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

When the view tabs do not fit on the row of the view bar (with the search box and the buttons), move them all off that row onto the rows right below it, filled from the left and as many rows as they need, so every tab stays visible and can be clicked; the bar's row keeps only the search box and the buttons, and the table moves down by the extra rows. When the tabs fit, nothing changes. `false` keeps one row: on a narrow screen the tabs other than the current one are left out (the `[` `]` keys still switch). When a view has `tab_break`, the tabs go on rows below whatever this setting says (see `tab_break` in [Obsidian Bases support](obsidian-bases.md)).

```toml
[display]
tab_wrap = true
```

### `display.tab_wrap_gap`

- Type: `boolean`
- Default: `false`
- Scope: global, workspace, table, view

With `tab_wrap`, put one empty row between the bar's row (the search box and the buttons) and the rows of tabs below it. Nothing changes when the tabs fit. When a view's `tab_break` puts the tabs below, the same one empty row goes in.

```toml
[display]
tab_wrap = true
tab_wrap_gap = true
```

### `display.note_name`

- Type: `string (a regular expression)`
- Default: `""`
- Scope: global, workspace, table, view

A regular expression applied to the name in the note column, after `note_path` (with folders, to the last part of the path only). When it matches, the column shows the group named `name`, or else the first group, or else the name with the matched part removed. A name that does not match, or that would become empty, is shown as it is. `""` sets nothing. Every other place that shows a note name uses it too: the target name in link cells (the alias text of `[[…|alias]]` is shown as written), the choices when you edit a link column, the linked rows and the "most linked" rows of the relation map, the right pane of the tree screen (which also follows `note_path`, like the note column) and the names in cross-workspace search results. What search matches (a hidden prefix still matches), the note names in messages and the detail view, the values written to notes (links keep the full name) and the `--print` values are not changed. An expression that cannot be read produces a warning and is not used. It is not switched in the "Display" section of the view settings (it is free text); a view can still override it in the view's `display` in `views.toml`.

```toml
[display]
note_name = '^\d{8}_'
```

With `note_path = "name"`, this shows `20261015_report.md` as `report`; `note_name = '^(?P<name>.+)_v\d+$'` shows `plan_v3.md` as `plan`.

### `display.note_path`

- Type: `string ("common", "name", "full" or "off")`
- Default: `"common"`
- Scope: global, workspace, table, view

How the note column on the left shows each note (without `.md`). `"common"` drops the folder shared by every row, `"name"` shows only the name, `"full"` shows the path from the root, and `"off"` shows no name and no heading, only the row marks (`!`, `~`) and the tree indent and ▾/▸, as when the table has a `file.name` column. Search and the note names in messages and the detail view keep the path from the root. Other values produce a warning and the default is used. In the "Display" section of the view settings this item cycles through common, name, full and off, and each view can override it there (`o`).

```toml
[display]
note_path = "name"
```

## `[dates]` — dates

### `dates.format`

- Type: `string`
- Default: `"YYYY-MM-DD"`
- Scope: global, workspace, table, view

How dates are shown and typed in the table. A date-time column shows the date in this format and the time after a space (`2026-11-25 16:00`; the note keeps `2026-11-25T16:00`). Parts: `YYYY`, `YY`, `MM`, `M`, `DD`, `D`, `ddd` (weekday), and separators that are not letters or digits (`-`, `/`, `.`, space, `年`, …). Month and day are required once each; year and weekday at most once. `M` and `D` (not zero-padded) cannot touch another number part without a separator. Notes are always written as `YYYY-MM-DD`. An unreadable format falls back to the default with a warning.

```toml
[dates]
format = "YYYY/MM/DD (ddd)"
```

### `dates.week_start`

- Type: `string ("sun" or "mon")`
- Default: `"sun"`
- Scope: global, workspace, table, view

First day of the week, used by the date calendar, the date words for this week and next week, the weekly buckets of charts, and the weeks and "this week" of the Gantt chart.

```toml
[dates]
week_start = "mon"
```

## `[edit]` — editing

### `edit.candidates`

- Type: `integer (0 or more)`
- Default: `20`
- Scope: global, workspace, table, view

Maximum number of value candidates offered when editing a text cell. Candidates come from the notes of the current table (not from other tables under the same root). A column with more distinct values than this offers none, and so does a column where five or more rows each have a value of their own (like a URL or a memo).

```toml
[edit]
candidates = 30
```

### `edit.add_frontmatter`

- Type: `boolean`
- Default: `true`
- Scope: global, workspace, table, view

Allow writing to notes that have no front matter or an empty one (only the two `---` lines). mdgrid adds the front matter or the key line. `false` makes these two kinds of notes read-only and shows the reason. A front matter with blank or comment lines between the delimiters is not empty and is always writable.

```toml
[edit]
add_frontmatter = false
```

### `edit.link_format`

- Type: `string ("auto", "wiki" or "markdown")`
- Default: `"auto"`
- Scope: global, workspace, table, view

The form of the links mdgrid writes in a link column: when you pick a candidate, when a name you type matches a note, and in the link fields of the new note form. `"wiki"` writes `[[name]]` and `"markdown"` writes `[name](path.md)`, whatever form the column already holds. `"auto"` follows the form the column already uses, and writes `[[name]]` while it has no values. Set it in a ledger you want to keep in the same form as your other table tools. The `routine` column of routine run notes stays `[[name]]` whatever this says. Other values give a warning and mean `"auto"`.

```toml
[edit]
link_format = "markdown"
```

### `edit.link_path`

- Type: `string ("shortest", "relative" or "absolute")`
- Default: `"shortest"`
- Scope: global, workspace, table, view

The path in the links mdgrid writes in a link column. `"shortest"` writes only the name in `[[…]]` unless another note has the same name (then the path from the note's folder), and the path from the note's folder in `[…](…)` and plain paths. `"relative"` writes the path from the note's folder in every form (`[[../Tasks/Send the quote]]` from `Projects/Tidy the house.md`), and `"absolute"` the path from the opened root (`[[Tasks/Send the quote]]`). `[[…]]` never gets `.md`. Other values give a warning and mean `"shortest"`.

```toml
[edit]
link_path = "absolute"
```

### `edit.rename_links`

- Type: `string ("always", "ask" or "never")`
- Default: `"always"`
- Scope: global, workspace, table, view

Whether renaming a note with "Rename the note" also rewrites the links to it. Only links that resolve, from the note holding them, to the renamed note are rewritten, in any form (`[[…]]`, `[…](…)`, plain paths): the folder part, the heading and the shown text stay, and only the name changes. A link to a note with the same name in another folder is left alone. The rewrites are pending changes, so you can review them in the diff before saving and undo them. `"always"` always stages them, `"ask"` shows how many links would change and stages them only when you answer `y`, and `"never"` does not. The file is renamed in every case. Other values give a warning and mean `"always"`.

```toml
[edit]
rename_links = "ask"
```

## `[ai]` — AI writes

### `ai.write`

- Type: `list of strings (column names)`
- Default: `[]`
- Scope: global, workspace, table, view

Columns that AI writes (`--apply <file> --propose <reason>` and `update` in the MCP server, `--mcp`) may change directly. Changes to columns not listed here are not written: they are left as proposals in `.mdgrid/proposals/` at the ledger root, and a person reviews the diff and takes them with "Show AI proposals" on screen. Empty by default, so no column is written directly. List only the columns you are happy to let AI change, such as a status or a progress column.

```toml
[ai]
write = ["status", "tags"]
```

## `[new_note]` — new notes

### `new_note`

- Type: `table (mode, folder, name, name_head, ask, required, hidden, body and [new_note.set])`
- Default: none
- Scope: global, workspace, table, view

How a new note is made from the table ("+ New" at the right end of the view bar, or `a`). Every part is optional.

- `folder`: default folder for new notes, relative to the opened folder (empty: the opened folder). Values that leave the opened folder (`..` or an absolute path) are refused. In a `.base` view whose filters include `file.inFolder("X")`, the note is made in that folder (`X` from the root folder) so the new row stays in the view; `folder` is used only when it is inside that folder (such as `X/inbox`).
- `name`: template put into the name field. `{date}` is today's date (`YYYY-MM-DD`); `{date:YYYYMMDD}_` starts the name with `20261118_`.
- `name_head`: how the filled `name` template is shown. `"show"` (default) puts it in the name field. `"hide"` keeps it out of the field (also after you leave the field) and adds it before the name you type (with `sub/name` it goes on the last part; a name that already starts with a prefix matched by `display.note_name` is left as typed). `mdgrid --new-note <name>` adds it to the given name the same way. Together with a `display.note_name` that hides the prefix in the table, you type only the title and never see the date. With `"hide"`, the variable `{name}` is the name you typed, without the prefix (a body template `# {name}` becomes `# Shopping`).
- `ask`: the columns shown as fields in the new note form, in order. Without `ask`, the form shows the columns visible in the table. Each field is edited with the input for its type; a field left empty is not written. `Ctrl+S` creates the note from any field.
- `required`: columns that must be filled; the note is not created while one is empty, and the field shows why.
- `hidden`: columns written into every new note without a field in the form (with `[new_note.set]`, for values such as the creation date).
- `mode`: `"form"` (default: the form with every field) or `"editor"` (ask only the name, create the note with the preset values and the body template, then open it in the editor right away). From the form, `Ctrl+E` also creates the note and opens it in the editor.
- `body`: a template file (relative to the opened folder) whose contents become the body of the new note. If it names a folder, the form gets a "template" field after the name, and you pick one of the folder's `.md` files (the first by name if you do not pick; "(empty)" for no body). `--new-note` takes the first one, as when you do not pick; a folder with no `.md` file gives no body.
- `[new_note.set]`: values written into every new note (column = value): a string, number, boolean, date, or an array of them (written as a vertical list).

Templates (`name`, the text values of `[new_note.set]`, and the `body` file) may use these variables; unknown ones are left as they are:

| Variable | Value |
|---|---|
| `{date}` | today (`YYYY-MM-DD`) |
| `{date+7}` · `{date-1}` | days from today |
| `{date:YYYY/MM/DD}` | today in a date format (same tokens as `dates.format`) |
| `{time}` | the current time (`HH:MM`) |
| `{now}` | the current date and time (`YYYY-MM-DDTHH:MM`) |
| `{weekday}` | the day of the week |
| `{name}` | the note's name (with `name_head = "hide"`, the name you typed, without the hidden prefix) |
| `{folder}` | the folder the note is made in |
| `{uuid}` | a new UUID for this note (version 4, like `3f2b8c1e-9a4d-4e7b-8c21-5d6f7a8b9c0d`) |
| `{uid}` | a new short id for this note (8 characters: a lowercase letter, then digits and lowercase letters, like `k3f9x2ab`) |

`{uuid}` and `{uid}` keep one value within a note, so `name = "{uid}_"` together with `id = "{uid}"` in `[new_note.set]` gives the file name and the frontmatter the same id. Add the column to `hidden` to fill it without showing a field.

In `name`, only the date variables (`{date}`, `{date:…}`, `{date+N}`, `{date-N}`, `{weekday}`) and the id variables (`{uuid}`, `{uid}`) are filled; the others stay as written, because the name is not decided yet and a time has a `:` that a file name cannot hold.

Filter conditions of the view that fix a single value (such as `status == "todo"` or a tag) are written first and win over `[new_note.set]`. `[new_note]` is one item: a narrower place that writes it (a workspace, a table, or a mdgrid view under `[table.view.new_note]` in `views.toml`) replaces it whole.

```toml
[new_note]
folder = "inbox"
name = "{date} "
ask = ["priority", "due"]
required = ["due"]
hidden = ["created"]
body = "templates/note.md"

[new_note.set]
tags = ["inbox"]
due = "{date+7}"
created = "{now}"
```

## `[types]` — column types

### `types`

- Type: `table (column = "text", "number", "checkbox", "date", "datetime", "list" or "auto")`
- Default: `{}`
- Scope: global, workspace, table, view

The type of single columns. A type written here wins over the root folder's type settings (`.obsidian/types.json`, which mdgrid only reads) and over guessing from the values. `"auto"` goes back to guessing, so a narrower scope can undo a type set in a wider one. The type decides the cell input, the `!` mark on values that do not fit, sorting, and the type mark in the column heading. Notes are never changed. Click the type mark of a column heading, or run `column_type` from the palette or the action menu, to pick a type (or "Auto") for the selected column; the choice is written to the table in `views.toml`. Columns of `file.*` and `formula.*` keep their own types. An unknown type name produces a warning and the column is guessed.

```toml
[types]
due = "date"
estimate = "number"
```

## `[rules]` — column rules

### `rules`

- Type: `table of tables (column = { required = boolean, values = [string, …], then = { value = { column = string or { value = string, if_empty = boolean } } }, icons = { value = string }, labels = { value = string }, candidates = { sort = "name" | "count" | "recent", where = { column = [value, …] } } })`
- Default: `{}`
- Scope: global, workspace, table, view

Rules for single columns, checked together with the column type. `required = true` makes a missing key, `null`, an empty string or an empty list a problem. `values` lists the allowed values: the value (each item, for a list) must be one of them, case-sensitive. A narrower scope wins item by item (a table can narrow `values` while keeping `required` from the global config). Values that break a rule or do not fit the column type are called problems: they get the `!` mark, the bottom band shows how many there are, `!` and `@` jump to the next and previous one, `problems_only` shows only the rows with problems, and `mdgrid --check` lists them for CI. Checking a rule never changes a note and never stops you writing a value. A column with `values` offers them in the cell input. Grouping by a column with `values` puts the groups in that order (reversed when descending), then any other value in text order, and the group without a value last: the same order as the kanban lanes. `--print` writes the rows in that order too. Computed columns (`file.*`, `formula.*`) cannot have rules; unknown items and values of the wrong type produce a warning and are ignored.

`then` fills other columns of the same row when you set the column to a value. Under `[rules.<column>.then]`, write `value = { column = "text" }`. The text may use the [template variables](#new_note) (`{date}`, `{now}`, `{uuid}` …), is read in the type of the column it goes to, and an empty text clears that column. The extra value is staged together with your edit: it shows in the diff before saving, one undo takes both back, and changing the cell to another value drops it again. It works for cell edits, bulk edits, checkboxes and moving a card on the kanban screen. To fill a column only while it is empty, write a table instead of the text: `{ value = "{date}", if_empty = true }`. The column is then filled only when it has no key, `null`, an empty string or an empty list (a value staged before counts), so a start date set the first time stays when the status goes back and forth. Changing the cell to another value drops the filled value only while it is still the one `then` put there, so a date you typed yourself stays. A table without `value`, with an `if_empty` that is not true or false, or with an unknown item is a warning, and that column is not filled. A narrower scope that writes `then` replaces it whole. In the example below, setting `status` to `done` fills `done_at` with today, and setting it to `doing` fills `started` with today only while `started` is empty.

`icons` and `labels` decide how the values read on screen. With `icons = { todo = "○" }` and `labels = { todo = "To do" }`, `todo` reads `○ To do` in group headings, kanban lane titles, table cells (chips or text), the detail view and the right pane of the tree; a value with only a glyph reads `○ todo`, and a value with neither stays as it is. Chip colors follow the value, so naming a value does not change its color. Any glyph works, Nerd Font ones too, and mdgrid prints it as you wrote it. Its width is counted like the rest of the screen, so if your font draws a glyph such as `○` two cells wide and it runs into the next letter, turn on [`terminal.ambiguous_wide`](#terminalambiguous_wide). Windows where you pick a value (the choices when you edit a cell or fill in the new note window, the list toggle window, the value lists of the filter window and view settings, and the frequency window) show a named value as its name followed by the value (`To do todo`), so you can tell which value you pick. The `\` quick filter and `/` search match the value or the name. The notes and `--print`, `--count` and CSV output keep the plain value. An entry for a value missing from `values` in the same table produces a warning naming it (`rules.status.icons.blocked`) and is ignored; when `values` comes from a wider scope, entries for values outside it are simply not shown. A narrower scope that writes `icons` or `labels` replaces it whole.

`candidates` decides the choices offered when you edit the column (cell edits, bulk edits and the new note window). `sort` picks the order: `"name"` by name, `"count"` most used first, `"recent"` most recently changed first. For a link column, `"count"` is the number of notes in the current table that point to the note, so a column pointing to the parent lists the parents with the most children first; for a value column it is the number of notes with that value. `"recent"` uses the newest change time of the target note for a link column, or of the notes with that value for a value column. Ties go by name. `where` has the same shape as a view's `where` and narrows the notes taken as choices: the target notes for a link column, the notes holding the value for a value column. That keeps finished tasks out of the parent choices, or tags only found on finished notes out of the tag choices. The current value stays among the choices even when it does not match. The choices of a column with `candidates` come from the notes of the current table, not from other tables under the same root. Without `sort`, a link column goes by name and a value column by count; a column without `candidates` works as before. A `sort` that is none of the three, a `where` that is not a table, or a `where` value that is not an array produces a warning and that item is ignored. A column with `values` keeps the order of the rule. A narrower scope that writes `candidates` replaces it whole. The last part of the example below, `[rules.parent.candidates]`, lists the unfinished parents with the most children first.

```toml
[rules.status]
required = true
values = ["todo", "doing", "done"]
icons = { todo = "○", doing = "◐", done = "●" }
labels = { todo = "To do", doing = "In progress" }

[rules.status.then]
done = { done_at = "{date}" }
doing = { started = { value = "{date}", if_empty = true } }

[rules.parent.candidates]
where = { status = ["todo", "doing", "waiting"] }
sort = "count"
```

## `[format]` — number formats

### `format`

- Type: `table of tables (column = { thousands = boolean, decimals = integer from 0 to 10, prefix = string, suffix = string })`
- Default: `{}`
- Scope: global, workspace, table, view

How the numbers of single columns read on screen. `thousands = true` groups the whole part by three digits. `decimals` sets the number of decimal places: missing places are filled with `0`, extra ones are rounded half away from zero (`1234.56` with `decimals = 1` reads `1,234.6`, `3` reads `3.0`); without it the value keeps its own digits. `prefix` and `suffix` go before and after the number, and a negative number puts `-` before the prefix (`-¥1,200`). The grouping and decimal separators follow the screen language: `1,234.5` in English, Japanese, Chinese and Korean, `1.234,5` in German, Spanish and Portuguese, and `1 234,5` (a narrow no-break space) in French. Columns of `formula.*` can have a format too (quote the name: `[format."formula.rate"]`); text values and values with a name from [`labels`](#rules) keep their own look.

The format applies to table cells, the detail view, pending values, the summary row, the subtotals in group headings and the values of a chart whose value column it is (sum, average, min, max). Counting summaries (`Checked`, `Unchecked`, `Empty`, `Filled`, `Unique`) and count charts stay plain numbers. It only changes what you see: the notes, sorting, filters, formulas, the cell input, `--print`, `--count` and copies keep the plain number. A narrower scope wins item by item. Unknown items, values of the wrong type, `decimals` outside 0 to 10, and a `prefix` or `suffix` with a control character or a text direction mark (which could garble the terminal, since a marker can come with a ledger someone sent you) produce a warning and are ignored; the other items still apply.

```toml
[format.amount]
thousands = true
prefix = "¥"

[format."formula.rate"]
decimals = 1
suffix = "%"
```

## `[keys]` — key bindings

### `keys`

- Type: `table of tables ([keys.<mode>] with key = action)`
- Default: none
- Scope: global only

Rebind keys per mode; without this item the built-in key bindings are used. Under `[keys.<mode>]`, write `key = "action name"`. The action name `"none"` removes the key. Keys are written like `"j"`, `"ctrl+s"`, `"shift+tab"`; a space separates a prefix key sequence such as `"g g"`.

Modes: `table`, `edit`, `review`, `quit`, `help`, `palette`, `search`, `filter`, `detail`, `settings`, `settings_input`, `chips`, `list_select`, `menu`, `freq`, `sorts`, `relations`, `drawer`, `filter_window`, `filter_input`, `screen_menu`, `column_type`, `tree`, `gantt`, `kanban`, `chart`, `notice`, `switch`, and `input_normal` in vim mode. Action names are shown in the command palette (`:`); [keys.md](keys.md) lists every default key and action name per mode. Unknown modes, keys, or actions produce a warning and are skipped.

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
```

### `keys.vim`

- Type: `boolean`
- Default: `false`
- Scope: global only

Turns on vim mode. Every input box (a cell, search, the palette, a settings value, the name of a new note) starts in insert mode, where you type as usual. Esc switches it to normal mode, where `h` `l` `0` `$` `w` `b` `e` move, `i` `a` `I` `A` go back to insert mode, `x` `X` `dw` `db` `d$` `D` `dd` delete, `cw` `c$` `C` `cc` change and `u` undoes. Esc in normal mode cancels the input, and Enter confirms in both modes. The bottom band shows `-- INSERT --` or `-- NORMAL --`, and the cursor is a bar or a block.

The table gets three keys: `dd` moves the notes of the marked rows (or the selected row) to the trash after confirming their names, `p` puts the value you copied with `y` (a cell) or `Y` (a row), and `.` puts the value you last set in a cell into the same column of the selected row. Digits stay free for other keys and are never counts. The palette's `q!` (quit without saving) works with or without vim mode.

With `false` (the default) every key works as before. The added keys are in [keys.md](keys.md#input_normal), and you can rebind or remove them under `[keys.table]` and `[keys.input_normal]`.

```toml
[keys]
vim = true
```

## `[search]` — searching across tables

### `search.across`

- Type: `boolean`
- Default: `false`
- Scope: global only

Adds "Search across the workspace" to the command palette (`:`). Use it when you remember a row but not which table you wrote it in. Type a word and press Enter: mdgrid reads every table in the current workspace (or your registered tables when no workspace is open) in the background and lists the matching rows per table, each with the note name and the column and value that matched. A row matches the same way as the search box (`\`): the note name or a value contains the word, ignoring case unless the word has a capital letter.

Results appear as each table is read, with "read 3 / 5" until all are done; Esc stops and leaves the table as it was. Tables that cannot be read are skipped and named at the end. Pick a row and press Enter to open its table with the word in the search box and that row selected; if you have unsaved changes, mdgrid asks first, as when switching tables from the drawer.

With `false` (the default) the palette does not show the command, so people who don't need it see nothing new.

```toml
[search]
across = true
```

## `[ask]` — asking AI

### `ask.command`

- Type: `list of strings (a command and its arguments)`
- Default: `[]`
- Scope: global only

The command and arguments that "Ask AI" in the palette runs. It runs without a shell and receives on standard input the selected rows (or the rows of the current view, up to 200) with their paths from the ledger root and front matter values, the columns with their types and allowed values, and the request you typed. Note bodies are not sent. The JSON changes the command prints (`[{"path": …, "set": {column: value}}]`) are narrowed to the rows and columns that were sent and become an AI proposal, which you review and take with "Show AI proposals". mdgrid holds no API keys and never goes online; the command (your local AI tool) holds the keys and decides where the data goes. So that a shared ledger's marker cannot make your machine run a command, it can only be set in the global config (config.toml). Empty by default, and then "Ask AI" does not appear in the palette.

Row values may have been written by other people, and someone could slip instructions for an AI into them. mdgrid puts the values inside marked delimiters, tells the model not to follow instructions found there, and narrows the reply to a proposal for the rows and columns it sent; still, choose a command that cannot use tools such as reading or writing files or running commands.

```toml
[ask]
command = ["claude", "-p", "--tools", ""]
```

## `[templates]` — templates

### `templates`

- Type: `table of tables ([templates.<name>] with profile items)`
- Default: none
- Scope: global only

Named pieces of a profile (`[look]`, `[display]`, `[dates]`, `[edit]`, `[new_note]`). `use = "<name>"` in any place lays one under that place's values. A template cannot `use` another. Templates saved from the "Look" section of the view settings go to ui.toml; a ui.toml template wins over a config.toml template of the same name.

```toml
[templates.night.look]
theme = "dracula"
preset = "dozy-pink"
```

## `[conditions]` — named conditions

### `conditions`

- Type: `table of tables ([conditions.<name>] with expr, label, key, hide)`
- Default: none
- Scope: global, workspace marker

Give a name to a condition you use often, such as "due within a week", and write it once. `expr` is an expression in the same language as the filters of a `.base` file. A name starts with a letter and uses only letters, digits, `_` and `-`. Then use the condition by name:

- in any expression as `cond("<name>")` (true when the row matches): `--print --filter`, `--count --filter`, the expression filter of the screen, a `.base` file, or another condition;
- with `key = 1` … `9`: that digit key shows only the matching rows, and pressing it again shows all rows (more than one digit narrows to the rows that match all of them); when nothing is selected and the search box does not filter, `Esc` in the table clears every condition turned on with a digit key or from the notice band (conditions with `hide = true` stay);
- in the filter section of the view settings (`o`), to keep only the matching rows or to hide them in that view;
- with `hide = true`: the matching rows are hidden in every table and screen and in `--print`, `--pick` and `--count`; the settings band shows the condition and how many rows it hides, the digit key of the condition shows them again, and `--show-hidden` prints them.

In the example, `far` hides the plans that start more than 30 days from now, unless they are due within a week. `label` is the name shown on the settings band and in the help (`label = "Due soon"`). Inside `cond()`, a column the table does not have reads as empty, so one condition works for tables with different columns. A name that is not allowed, an expression that cannot be read, conditions that call each other in a loop, and a digit that another condition in the same file already uses produce a warning and are skipped.

Conditions can also be written, in the same shape, at the top of a ledger's marker `.mdgrid/workspace.toml`. A people ledger can carry "time to get in touch" and a money ledger "payment due soon", and they travel with the ledger. A condition in a marker is available only while a table of that ledger is open, next to the global conditions. With the same name the marker wins, and `cond()` in a global condition points to the marker's one too. Put them in the marker of a starter and every ledger made from it has them. `mdgrid <table> --print-config --resolved` shows where each condition came from.

Digit keys go to the conditions with a `key`: first the marker's, then the global ones, numbered from 1 with no gaps. The `key` number sets the order within one file. A condition whose expression uses a column (a note key or `formula.*`) that no row of the current table has gets no digit there, and it is left out of the help and of the filter section of the view settings. So with task conditions (due date, owner) in `config.toml`, a people ledger still starts at 1 with its own conditions, and no digit leads to an empty table. Conditions with `hide = true` apply whatever the columns.

```toml
[conditions.soon]
expr = 'due <= today() + "7d"'
key = 1

[conditions.far]
expr = 'start > today() + "30d" && !(due <= today() + "7d")'
hide = true
key = 9
```

## `[[notices]]` — the notice box

### `notices`

- Type: `array of tables ([[notices]] with cond, text, scope, show_zero, or note, lines)`
- Default: none
- Scope: global, workspace marker

A small box above the header that shows what you want to notice, such as how many tasks are overdue. It appears only when you write items here; nothing changes for anyone else. Each `[[notices]]` is one item, shown in the order written. Items can also sit at the top of a ledger's marker: they show only in the tables of that ledger, before the global items (a relative `note` starts at the marker's folder).

- A count item counts the rows that match a [named condition](#conditions--named-conditions) (`cond`). `text` is the sentence with `{count}` replaced by the number (default: the condition's label and the number). `scope` picks what to count: `"table"` (the open table, the default), `"workspace"` (every table of the current workspace) or a list of table names. Rows are counted after the table's own filter and the conditions with `hide = true`, not after the view, search or digit keys. An item with 0 rows is left out unless `show_zero = true`. Tables without the columns the condition uses are not counted, and a `"table"` item is left out, `show_zero` or not, when the open table lacks them.
- A note item shows the first `lines` non-empty lines (default 1) of a note's body (`note`; `~` is the home folder, a relative path starts at the folder of the config file).

Count items share the first line, separated by ` · `; each note line gets its own line; the box is at most 3 lines. Move into the box with its key, pick an item and press `Enter`: a count item narrows the table to that condition (or lists the tables that have matching rows), a note item opens the note in your editor. `mdgrid --notices` prints the same items, one per line, for a tmux status line, a shell prompt or the Herdr sidebar.

```toml
[[notices]]
cond = "overdue"
text = "{count} overdue"
scope = "workspace"

[[notices]]
note = "~/notes/inbox.md"
lines = 2

[conditions.overdue]
expr = 'due < today() && status != "done"'
```

## `[[notify]]` — notifications

### `notify`

- Type: `array of tables ([[notify]] with cond, scope, title, text, to, group_over, group_text, command)`
- Default: none
- Scope: global, workspace marker

Tells you when a row starts to match a [named condition](#conditions--named-conditions): a reminder time has come, or a task has just become overdue. Nothing happens until you write an item here. Each `[[notify]]` watches one condition, and one condition can be watched by only one item. Items can also sit at the top of a ledger's marker: they are checked only by a screen showing a table of that ledger and by `mdgrid <ledger> --notify`. When the marker and `config.toml` watch the same condition, the ledger uses the marker's item. Rows of tables without the columns the condition uses are never notified. A marker item cannot send to a command (`command` in `to`, and `command`): markers also arrive inside folders other people give you, and opening one must never run their commands. mdgrid warns and drops that outlet; write command notifications in `config.toml`.

- `cond` is the condition to watch.
- `scope` picks the tables to watch: `"workspace"` (every table of the current workspace, the default), `"table"` (the open table) or a list of table names.
- `title` and `text` are the title and the text of the notification. `{column}` becomes the row's value, and `{file.name}` and `{formula.x}` work too; a column the row doesn't have becomes empty. Without them, the title is the condition's label and the text is the note's name.
- `to` lists where it goes: `"desktop"` (the OS notification, the default), `"terminal"` (your terminal's own notification) and `"command"` (your own program).
- `command` is the program and its arguments for `"command"`. `{title}`, `{text}` and `{path}` in the arguments are filled in. No shell runs it, so a value in a note never turns into a command.
- When more rows than `group_over` (default 3) start to match in one check, they arrive as one notification. `group_text` is its text, with `{count}` replaced by the number of rows (default: "4 new").

A row is notified once, when it starts to match. It stays quiet while it keeps matching, and is notified again if it stops matching and later matches again. When you add an item, the rows that already match are remembered without a notification, so nothing floods in. The record is kept in mdgrid's state folder, never in your notes.

mdgrid checks when you open a table, after you save and every minute while it is open. While mdgrid is closed, `mdgrid --notify` checks once without the screen, for your OS scheduler to run ([how to register it](manual/en/tasks.md#get-notified-when-a-row-matches)); it shares the record with the screen, so a row is never notified twice. If a notification can't be delivered, the reason shows at the bottom of the screen once, and the row is tried again at the next check.

macOS, Windows and WSL show the desktop notification with what the OS already has; inside WSL it goes to Windows. On Linux it needs `notify-send`. For `"terminal"`, mdgrid sends the one signal your terminal understands (iTerm2, kitty, Ghostty, WezTerm, foot, Windows Terminal) and a bell to other terminals. Inside tmux, turn on `set -g allow-passthrough on`.

```toml
[[notify]]
cond = "remind"
text = "{title} ({remind})"
to = ["desktop", "terminal"]

[conditions.remind]
expr = 'remind <= now() && status != "done"'
```
