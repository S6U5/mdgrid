# Configuration

[日本語](config.ja.md)

mdgrid reads one TOML file:

- `$XDG_CONFIG_HOME/mdgrid/config.toml`, or `~/.config/mdgrid/config.toml` when `XDG_CONFIG_HOME` is unset (also on macOS).
- `mdgrid --config <path>` reads that file instead.

The same folder also holds files that mdgrid writes itself: `ui.toml` (what you pick on the screen, such as the theme in the "Look" section of the view settings), `views.toml` (mdgrid views and per-table settings), `workspaces.toml` (workspaces) and `places.toml` (registered tables; see [Register the tables you use](manual/en/tasks.md#register-the-tables-you-use-and-switch-between-them)). mdgrid never rewrites `config.toml`.

Every item is optional. Without a file mdgrid runs with the defaults. Unknown items, values of the wrong type and items written in a place that cannot hold them produce a warning on the message line and are ignored; mdgrid still starts. Every warning has the form `file: item path: reason`. A broken TOML file stops mdgrid with a one-line reason.

To start from the defaults, write them out with comments:

```sh
mdgrid --print-config > ~/.config/mdgrid/config.toml
```

The output reads back without warnings and behaves exactly like having no file. Items without a default appear as commented examples.

## Two kinds of items

- **Whole-app items** (scope "global only"): the top-level `language`, `editor` and `poll_ms`, and the sections `[terminal]`, `[workspace]`, `[keys]` and `[templates]`. They are written only in `config.toml` (`ui.toml` may hold `[terminal] nerd_font`).
- **The table profile** (scope "global, workspace, table, view"): `use`, `[look]`, `[display]`, `[dates]`, `[edit]` and `[new_note]`. The same shape can be written in every place below, and narrower places override wider ones.

## Overrides per workspace, table and view

| Place | Written by hand | Written by mdgrid |
|---|---|---|
| Global | `config.toml` | `ui.toml` |
| Workspace | the marker `.mdgrid/workspace.toml` (top level), or `[[workspace]]` in `workspaces.toml` (`[workspace.look]` …) | `workspaces.toml` (never the marker) |
| Table | the table entry of the workspace (`[[table]]` + `[table.look]` in the marker, `[[workspace.table]]` + `[workspace.table.look]`) | `[[table]]` in `views.toml` |
| View | — | the view settings (with a mdgrid view, saved in its definition too) |

The value of each item is taken from the narrowest place that writes it (view → table → workspace → global → default). In the same place, what mdgrid wrote wins over what you wrote by hand (`ui.toml` over `config.toml`, `views.toml` over the workspace's table entry). Two exceptions keep looks whole: a `preset` drops the part shapes (`[look.style]`) written in wider places, and a `theme` drops the role colors (`[look.colors]`, not the value colors) written in wider places. `use = "<template>"` lays the template under the values of that place.

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

- Type: `string ("auto", "en" or "ja")`
- Default: `"auto"`
- Scope: global only

Language of the screen and the startup messages (`--help`, reasons for not starting, warnings). `"auto"` looks at `LC_ALL`, `LC_MESSAGES` and `LANG` in this order and uses the first one that is not empty: Japanese if it starts with `ja`, otherwise English (also when none is set). `"en"` and `"ja"` choose the language regardless of the environment. Note values, column names, the contents of `.base` files and file names are never translated. Errors in the command line arguments are reported before the configuration is read, so they follow the environment.

```toml
language = "en"
```

### `editor`

- Type: `string`
- Default: none
- Scope: global only

Editor that opens the selected note (`e`, or a click on the selected row's file name). Arguments are allowed, such as `"code -w"`; the value is split into words without a shell (quotes and backslashes work as in POSIX), and the note path is added as one argument. When this is unset, empty, or only spaces, mdgrid uses `$VISUAL`, then `$EDITOR`, then `vi` — the first one that is not empty. After the editor exits, mdgrid re-reads the note.

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

## `[workspace]` — workspaces

### `workspace.detect`

- Type: `array of strings`
- Default: `["vault"]`
- Scope: global only

When the table you open is in no written workspace (`-w`, a `.mdgrid/workspace.toml` marker, or `workspaces.toml`), treat the root of the Obsidian vault (`"vault"`, a folder with `.obsidian/`) or the git repository (`"git"`, a folder with `.git`) above it as a workspace: its folders with notes become the tables of the relation map. `[]` turns detection off. A detected workspace has no workspace place for overrides.

```toml
[workspace]
detect = ["vault", "git"]
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

Override the theme's colors by role. Roles: `background`, `text`, `header` (first line), `selection` (selected row), `selection_text`, `band` (key band), `band_text`, `accent` (headings, keys, frames), `strong`, `zebra`, `zebra_text`, `mark`, `added`, `removed`, `pending` (unsaved changes) and `highlight` (search matches). Roles you leave out keep the theme's color. `[look.colors.values]` sets a color per value (matched ignoring case and surrounding spaces); it is used by every part shape and in the value list, and works for values in any language. Colors are `"#rrggbb"`, `"#rgb"` or a name: `black`, `white`, `gray`, `red`, `orange`, `yellow`, `green`, `teal`, `cyan`, `blue`, `purple`, `magenta`, `pink`, `brown`. With `theme = "default"`, the terminal's own background and text stay, and only `accent`, `selection` and value colors are used. Unknown roles and unreadable colors give a warning and are skipped; so does a value written twice. A theme written in a narrower place drops the role colors of wider places (value colors stay).

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

Show a search bar above the table (`\` moves into it). `false` hides the bar; the quick filter is then typed on the bottom line. Each view can override this in its view settings (`o`, the "Display" section).

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

## `[dates]` — dates

### `dates.format`

- Type: `string`
- Default: `"YYYY-MM-DD"`
- Scope: global, workspace, table, view

How dates are shown and typed in the table. Parts: `YYYY`, `YY`, `MM`, `M`, `DD`, `D`, `ddd` (weekday), and separators that are not letters or digits (`-`, `/`, `.`, space, `年`, …). Month and day are required once each; year and weekday at most once. `M` and `D` (not zero-padded) cannot touch another number part without a separator. Notes are always written as `YYYY-MM-DD`. An unreadable format falls back to the default with a warning.

```toml
[dates]
format = "YYYY/MM/DD (ddd)"
```

### `dates.week_start`

- Type: `string ("sun" or "mon")`
- Default: `"sun"`
- Scope: global, workspace, table, view

First day of the week in the date calendar.

```toml
[dates]
week_start = "mon"
```

## `[edit]` — editing

### `edit.candidates`

- Type: `integer (0 or more)`
- Default: `20`
- Scope: global, workspace, table, view

Maximum number of value candidates offered when editing a text cell. A column with more distinct values than this offers no candidates.

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

## `[new_note]` — new notes

### `new_note`

- Type: `table (mode, folder, name, ask, required, hidden, body and [new_note.set])`
- Default: none
- Scope: global, workspace, table, view

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
| `{date:YYYY/MM/DD}` | today in a date format (same tokens as `dates.format`) |
| `{time}` | the current time (`HH:MM`) |
| `{now}` | the current date and time (`YYYY-MM-DDTHH:MM`) |
| `{weekday}` | the day of the week |
| `{name}` | the note's name |
| `{folder}` | the folder the note is made in |

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

## `[keys]` — key bindings

### `keys`

- Type: `table of tables ([keys.<mode>] with key = action)`
- Default: none
- Scope: global only

Rebind keys per mode; without this item the built-in key bindings are used. Under `[keys.<mode>]`, write `key = "action name"`. The action name `"none"` removes the key. Keys are written like `"j"`, `"ctrl+s"`, `"shift+tab"`; a space separates a prefix key sequence such as `"g g"`.

Modes: `table`, `edit`, `review`, `quit`, `help`, `palette`, `search`, `filter`, `detail`, `settings`, `settings_input`, `chips`, `list_select`, `menu`, `freq`, `sorts`, `relations`. Action names are shown in the command palette (`:`); [keys.md](keys.md) lists every default key and action name per mode. Unknown modes, keys, or actions produce a warning and are skipped.

```toml
[keys.table]
"ctrl+f" = "search"
"x" = "none"
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
