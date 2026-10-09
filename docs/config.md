# Configuration

[日本語](config.ja.md)

mdgrid reads one TOML file:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`, or `~/.config/mdgrid/config.toml` when `XDG_CONFIG_HOME` is unset (also on macOS).
- `mdgrid --config <path>` reads that file instead.

The same folder also holds `views.toml` (mdgrid views) and `places.toml` (registered tables; see [Register the tables you use](manual/en/tasks.md#register-the-tables-you-use-and-switch-between-them)). mdgrid writes those two itself.

Every item is optional. Without a file mdgrid runs with the defaults. Unknown items and values of the wrong type produce a warning on the message line and are ignored; mdgrid still starts. A broken TOML file stops mdgrid with a one-line reason.

To start from the defaults, write them out with comments:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

The output reads back without warnings and behaves exactly like having no file. `editor` and `keys` have no default value, so they appear as commented examples.

The items below are the full list. Tests check that this list (names, the `Type` and `Default` lines, and the examples), the items mdgrid reads, and the `--print-config` output agree.

## Items

### `color`

- Type: `boolean`
- Default: `true`

Use colors. `false` draws without colors, the same as the `NO_COLOR` environment variable or `--no-color`.

```toml
color = false
```

### `candidates`

- Type: `integer (0 or more)`
- Default: `20`

Maximum number of value candidates offered when editing a text cell. A column with more distinct values than this offers no candidates.

```toml
candidates = 30
```

### `poll_ms`

- Type: `integer (1 or more)`
- Default: `1000`

Interval in milliseconds for re-reading notes that changed outside mdgrid.

```toml
poll_ms = 2000
```

### `ambiguous_wide`

- Type: `boolean`
- Default: `false`

Treat East Asian Ambiguous characters (such as `○` and `※`) as two columns wide. Set `true` if your terminal draws them wide, so that the columns line up.

```toml
ambiguous_wide = true
```

### `workspace_detect`

- Type: `array of strings`
- Default: `["vault"]`

When the table you open is in no written workspace (`-w`, a `.mdgrid/workspace.toml` marker, or `workspaces.toml`), treat the root of the Obsidian vault (`"vault"`, a folder with `.obsidian/`) or the git repository (`"git"`, a folder with `.git`) above it as a workspace: its folders with notes and `.base` files become the tables of the relation map. `[]` turns detection off.

```toml
workspace_detect = ["vault", "git"]
```

### `look`

- Type: `string`
- Default: `"modern"`

The look when colors are on: `"modern"` (lazygit-like: the selection is tinted instead of inverted, window borders and keys use an accent color, and descriptions are dimmed) or `"classic"` (inverse video, as before). The text on screen is the same either way, and without colors (`NO_COLOR`, `--no-color`) both look like classic. The colors come from the `theme`.

```toml
look = "classic"
```

### `cells`

- Type: `string or table`
- Default: `"rich"`

How table cells look when colors are on. `"rich"` shows them as parts, like a GUI table: booleans as `☑` and `☐`, each list item as a colored chip, the values of short text columns that repeat a few values (such as `status`) as colored chips, links in the accent color, and a type mark in each column heading (`#` number, `◷` date, `☑` boolean, `⋮` list, `◉` chips). A value always keeps the same chip color, and its text is always shown. `"plain"` shows the text as it is. Without colors (`--no-color`, `NO_COLOR`) cells are always plain. The values written to notes, edited, searched and printed (`--print`) do not change.

```toml
cells = "plain"
```

As a table, choose each part and each column. A column setting wins over the part settings; `"chip"` makes chips of a column that is not detected as one.

```
[cells]
style = "rich"     # or "plain"
checkbox = true    # booleans as ☑ ☐
chips = true       # list items as chips
select = true      # short repeated text values as chips
links = true       # links in the accent color
icons = true       # type marks in the column headings

[cells.columns]
status = "plain"   # this column as text
owner = "chip"     # this column as chips
```

### `view_tabs`

- Type: `string`
- Default: `"always"`

When to show the row of view tabs below the header: `"always"`, or `"auto"` to show it only when there are two or more views. A folder you open without a `.base` has one view, **All notes** (every note and every key); with `"auto"` its tab row is left out and the table uses that row, until you save a view of your own. `[display] tabs = false` hides the tabs either way.

```toml
view_tabs = "auto"
```

### `borders`

- Type: `string`
- Default: `"rounded"`

How window frames are drawn: `"rounded"` (connected lines with round corners, `╭─╮`) or `"ascii"` (`+ - |`). With `ambiguous_wide = true` the frames are always ASCII, so the columns stay aligned.

```toml
borders = "ascii"
```

### `search_bar`

- Type: `boolean`
- Default: `true`

Show a search bar above the table (`\` moves into it). `false` hides the bar; the quick filter is then typed on the bottom line.

```toml
search_bar = false
```

### `date_format`

- Type: `string`
- Default: `"YYYY-MM-DD"`

How dates are shown and typed in the table. Parts: `YYYY`, `YY`, `MM`, `M`, `DD`, `D`, `ddd` (weekday), and separators that are not letters or digits (`-`, `/`, `.`, space, `年`, …). Month and day are required once each; year and weekday at most once. `M` and `D` (not zero-padded) cannot touch another number part without a separator. Notes are always written as `YYYY-MM-DD`. An unreadable format falls back to the default with a warning.

```toml
date_format = "YYYY/MM/DD (ddd)"
```

### `week_start`

- Type: `string ("sun" or "mon")`
- Default: `"sun"`

First day of the week in the date calendar.

```toml
week_start = "mon"
```

### `add_frontmatter`

- Type: `boolean`
- Default: `true`

Allow writing to notes that have no front matter or an empty one (only the two `---` lines). mdgrid adds the front matter or the key line. `false` makes these two kinds of notes read-only and shows the reason. A front matter with blank or comment lines between the delimiters is not empty and is always writable.

```toml
add_frontmatter = false
```

### `editor`

- Type: `string`
- Default: none

Editor that opens the selected note (`e`, or a click on the selected row's file name). Arguments are allowed, such as `"code -w"`; the value is split into words without a shell (quotes and backslashes work as in POSIX), and the note path is added as one argument. When this is unset, empty, or only spaces, mdgrid uses `$VISUAL`, then `$EDITOR`, then `vi` — the first one that is not empty. After the editor exits, mdgrid re-reads the note.

```toml
editor = "nvim"
```

### `language`

- Type: `string ("auto", "en" or "ja")`
- Default: `"auto"`

Language of the screen and the startup messages (`--help`, reasons for not starting, warnings). `"auto"` looks at `LC_ALL`, `LC_MESSAGES` and `LANG` in this order and uses the first one that is not empty: Japanese if it starts with `ja`, otherwise English (also when none is set). `"en"` and `"ja"` choose the language regardless of the environment. Note values, column names, the contents of `.base` files and file names are never translated. Errors in the command line arguments are reported before the configuration is read, so they follow the environment.

```toml
language = "en"
```

### `theme`

- Type: `string ("default", "nord", "solarized-light", "dracula", "gruvbox", "pink-monster" or "dozy-pink")`
- Default: `"default"`

Color theme of the screen: `"default"`, `"nord"`, `"solarized-light"`, `"dracula"`, `"gruvbox"`, `"pink-monster"` or `"dozy-pink"`. `"default"` keeps the terminal's own foreground and background (the look before themes existed). The other themes paint the background, the text, the header line, the column headings, the selected cell, the bottom band, the shaded rows of `zebra` and the `>` marks with their own colors; colors that carry meaning (pending values, search matches, added and removed lines in the save review) also get theme colors chosen to stay readable on the theme's background. Without colors (`--no-color`, `NO_COLOR`, `TERM=dumb`, `color = false`) the theme is ignored. On terminals without true color (`COLORTERM` is not `truecolor` or `24bit`) the nearest of the 256 colors is used. An unknown name gives a warning and the default look.

```toml
theme = "nord"
```

### `display`

- Type: `table ([display] with row_numbers, zebra, column_lines, group_gap, tabs and chips)`
- Default: `{ row_numbers = false, zebra = false, column_lines = false, group_gap = false, tabs = true, chips = true }`

How the table is shown. Every part is optional and is `true` or `false`.

- `row_numbers` (default `false`): numbers 1, 2, 3… on the left of each row, in the order shown (after filters and sorting). Group heading rows get no number; numbering continues across groups.
- `zebra` (default `false`): every other row gets a shaded background. Not drawn without colors (`--no-color`, `NO_COLOR`, `color = false`).
- `column_lines` (default `false`): draw `│` between columns.
- `group_gap` (default `false`): in a grouped view, leave an empty line above every group heading but the first. The empty line is not a row: it is not counted or numbered, the cursor skips it, and clicking it does nothing.
- `tabs` (default `true`): show the view tabs. When hidden, `[` and `]` still switch views.
- `chips` (default `true`): show the band of active view settings. When hidden, `f` still selects its items.

The search bar is the top-level `search_bar` item; it is not part of `[display]`. Each view can override these seven (including the search bar) in its view settings (`o`, the "表示" section); a view keeps only the items that differ from this configuration, and a mdgrid view saves them with the view.

```toml
[display]
row_numbers = true
zebra = true
column_lines = true
```

### `new_note`

- Type: `table ([new_note] with mode, folder, name, ask, required, hidden, body and [new_note.set])`
- Default: `{}`

How a new note is made from the table ("+ New" in the header, or `a`). Every part is optional.

- `folder`: default folder for new notes, relative to the opened folder (empty: the opened folder). Values that leave the opened folder (`..` or an absolute path) are refused. In a `.base` view whose filters include `file.inFolder("X")`, the note is made in that folder (`X` from the vault root) so the new row stays in the view; `folder` is used only when it is inside that folder (such as `X/inbox`).
- `name`: template put into the name field. `{date}` is today's date (`YYYY-MM-DD`).
- `ask`: the columns shown as fields in the new note form, in order. Without `ask`, the form shows the columns visible in the table. Each field is edited with the input for its type; a field left empty is not written. `Ctrl+S` creates the note from any field.
- `required`: columns that must be filled; the note is not created while one is empty, and the field shows why.
- `hidden`: columns written into every new note without a field in the form (with `[new_note.set]`, for values such as the creation date).
- `mode`: `"form"` (default: the form with every field) or `"editor"` (ask only the name, create the note with the preset values and the body template, then open it in the editor right away). From the form, `Ctrl+E` also creates the note and opens it in the editor.
- `body`: a template file (relative to the opened folder) whose contents become the body of the new note.
- `[new_note.set]`: values written into every new note (column = value): a string, number, boolean, date, or an array of them (written as a vertical list).

Templates (`name`, the text values of `[new_note.set]`, and the `body` file) may use these variables; unknown ones are left as they are:

| Variable | Value |
|---|---|
| `{date}` | today (`YYYY-MM-DD`) |
| `{date+7}` · `{date-1}` | days from today |
| `{date:YYYY/MM/DD}` | today in a date format (same tokens as `date_format`) |
| `{time}` | the current time (`HH:MM`) |
| `{now}` | the current date and time (`YYYY-MM-DDTHH:MM`) |
| `{weekday}` | the day of the week |
| `{name}` | the note's name |
| `{folder}` | the folder the note is made in |

Filter conditions of the view that fix a single value (such as `status == "todo"` or a tag) are written first and win over `[new_note.set]`. A mdgrid view (`views.toml`) can use the same form under `[target.view.new_note]`; that view then uses it instead of this item.

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

### `keys`

- Type: `table of tables ([keys.<mode>] with key = action)`
- Default: none

Rebind keys per mode; without this item the built-in key bindings are used. Under `[keys.<mode>]`, write `key = "action name"`. The action name `"none"` removes the key. Keys are written like `"j"`, `"ctrl+s"`, `"shift+tab"`; a space separates a prefix key sequence such as `"g g"`.

Modes: `table`, `edit`, `review`, `quit`, `help`, `palette`, `search`, `filter`, `detail`, `settings`, `settings_input`, `chips`, `list_select`, `menu`, `freq`. Action names are shown in the command palette (`:`); [keys.md](keys.md) lists every default key and action name per mode. Unknown modes, keys, or actions produce a warning and are skipped.

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
```
