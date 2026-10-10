# Make it look the way you like

[日本語](../ja/appearance.md) · [Contents](index.md)

The look of mdgrid is set by a handful of items in the config file (`~/.config/mdgrid/config.toml`). This page shows what each one changes and how they combine. Every item is described exactly in [Configuration](../../config.md).

- [The default look](#the-default-look)
- [Colors: themes](#colors-themes)
- [Shapes of parts: style](#shapes-of-parts-style)
- [Cells as parts or as text](#cells-as-parts-or-as-text)
- [Frames and selection](#frames-and-selection)
- [What is shown around the table](#what-is-shown-around-the-table)
- [Terminals and fonts](#terminals-and-fonts)
- [Recipes](#recipes)

## The default look

With colors on, mdgrid starts in the quiet "sumi" set (`[style] preset = "sumi"`): rounded window frames, the selection as a tinted background instead of reverse video, headings in an accent color with a line under them, and cells shown as parts — a colored dot before values like status (`● doing`), lists separated by `·` (`ui · web`), and `✓` for booleans. Only the dot is colored; the value text stays in the plain, readable color.

![The default look](images/demo-edit-list.svg)

Without colors (`--no-color`, `NO_COLOR` set, `TERM=dumb`, or `color = false`), the screen is plain text with reverse video for the selection; nothing on screen depends on color alone.

## Colors: themes

`theme` picks a color set: `"default"` keeps your terminal's colors; `"nord"`, `"solarized-light"`, `"dracula"`, `"gruvbox"`, `"pink-monster"`, `"dozy-pink"` and the quieter `"sumi"`, `"slate"`, `"saas"`, `"saas-dark"` and `"paper"` paint the whole screen. Only colors change. See [Themes](themes.md) for screenshots.

```toml
theme = "nord"
```

`theme = "auto"` uses `theme_light` (default `"saas"`) on a light terminal background and `theme_dark` (default `"sumi"`) on a dark one. mdgrid reads the background from `COLORFGBG` or asks the terminal; when it cannot tell, it assumes dark.

```toml
theme = "auto"
theme_light = "paper"
theme_dark = "saas-dark"
```

## Shapes of parts: style

`[style]` picks the shape of each part of the table and windows. `preset` picks a whole set; the other items override one part each.

| Preset | Feel |
|---|---|
| `sumi` (default) | Dot and text, tags separated by `·`, `✓`, a line under headings, underlined tabs |
| `slate` | Shapes (○ ◐ ● ⊘), `#tags`, `☑`, filled tabs, heading marks |
| `saas` | Soft tinted pills, segmented tabs |
| `paper` | Colored text only, `[x]`, column lines, square frames |
| `grid` | Plain text, column and heading lines, keys on a tint |
| `classic` | The 0.2.0 look (square chips, filled selection) |
| `dozy-pink` | Soft round pills (pair it with the `dozy-pink` theme) |

```toml
[style]
preset = "saas"
select = "cross"      # tint the selected row and column as a crosshair
```

The parts are `status`, `tags`, `check`, `select`, `rules`, `tabs`, `frames`, `band` and `icons` (all values in [Configuration](../../config.md#style)). Round pills (`"pill"`) are drawn with Nerd Font glyphs. With the default `nerd_font = "auto"` they are round on terminals that draw the rounded ends themselves (Ghostty, WezTerm) and clipped elsewhere; `true` always draws them round, `false` always clipped.

**Catalog**: open [`docs/catalog/index.html`](../../catalog/index.html) in a browser to compare themes and part shapes on a live sample. Select cells, pick values, search, filter, open details, the relation map and help — with clicks and keys. It also writes the settings to paste into `config.toml`.

## Cells as parts or as text

`cells` decides how values look in the table when colors are on:

| Value | `cells = "rich"` (default) | `cells = "plain"` |
|---|---|---|
| `done: true` | `☑` | `true` |
| `tags: [ui, web]` | chips `ui` `web`, each in its own color | `[ui, web]` |
| `status: doing` (a column of short values that repeat) | a chip `doing` | `doing` |
| `project: "[[mdgrid]]"` | `mdgrid` in the accent color | `mdgrid` |
| column headings | `◉ status`, `◷ due`, `# priority`, `☑ done`, `⋮ tags` | `status`, `due`, … |

The same value always gets the same chip color, in the table, in the value counts (`%`) and in the list of choices. The text of a value is always shown; color never stands in for it. Values written to notes, edited, searched and printed (`--print`) are the plain values.

Choose part by part, and column by column:

```toml
[cells]
checkbox = false      # keep true / false as text
select = true         # chips for short repeated values

[cells.columns]
memo = "plain"        # this column as text
owner = "chip"        # chips even if not detected (the setting for a column wins)
```

When a column is too narrow for all its chips, the chips that fit are shown whole and the rest are counted as `+N`.

## Frames and selection

- `look = "classic"` brings back the earlier look: reverse video for the selection, no accent colors. The text on screen is the same in both looks.
- `[style] frames` picks window frames: `"rounded"`, `"square"`, `"heavy"`, `"ascii"` or `"none"`. `borders = "ascii"` still works and means `frames = "ascii"` (always ASCII with `ambiguous_wide = true`, so columns stay aligned).

## What is shown around the table

| Item | What it changes |
|---|---|
| `view_tabs = "auto"` | Hide the row of view tabs while there is only one view (a folder with no saved views). The row comes back when you save a view. |
| `search_bar = false` | Hide the search bar above the table; `\` still filters, typing on the bottom line. |
| `[display] tabs = false` | Hide the view tabs always; `[` and `]` still switch views. |
| `[display] chips = false` | Hide the band of active view settings. |
| `[display] row_numbers = true` | Number the rows 1, 2, 3… in the order shown. |
| `[display] zebra = true` | Shade every other row (only with colors). |
| `[display] column_lines = true` | Draw `│` between columns. |
| `[display] group_gap = true` | Leave an empty line above every group heading but the first. |

Each view can override the `[display]` items and the search bar in its view settings (`o`), and a saved mdgrid view keeps them.

## Terminals and fonts

- If characters such as `○`, `※` or the box lines look twice as wide as they should and columns drift, set `ambiguous_wide = true`. The frames then use ASCII so that everything lines up.
- Without true color (`COLORTERM` is not `truecolor` or `24bit`), mdgrid uses the nearest of the 256 colors.
- The checkbox and type marks (`☑ ☐ ◉ ◷ ⋮`) are ordinary text characters; if your font draws them badly, use `[cells] checkbox = false` and `icons = false`.

## Recipes

Quiet and minimal:

```toml
view_tabs = "auto"
search_bar = false
cells = "plain"

[display]
chips = false
```

A dense table with guides:

```toml
[display]
row_numbers = true
zebra = true
column_lines = true
```

The earlier look:

```toml
look = "classic"
borders = "ascii"
cells = "plain"
```
