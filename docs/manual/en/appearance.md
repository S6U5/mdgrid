# Make it look the way you like

[日本語](../ja/appearance.md) · [Contents](index.md)

The look of mdgrid is set by a handful of items in the config file (`~/.config/mdgrid/config.toml`). This page shows what each one changes and how they combine. Every item is described exactly in [Configuration](../../config.md).

- [The default look](#the-default-look)
- [Colors: themes](#colors-themes)
- [Cells as parts or as text](#cells-as-parts-or-as-text)
- [Frames and selection](#frames-and-selection)
- [What is shown around the table](#what-is-shown-around-the-table)
- [Terminals and fonts](#terminals-and-fonts)
- [Recipes](#recipes)

## The default look

With colors on, mdgrid starts in a lazygit-like look: rounded window frames, the selection as a tinted background instead of reverse video, keys and headings in an accent color, and cells shown as parts — checkboxes, colored chips for lists and short repeated values, and a type mark in each column heading.

![The default look](../../assets/demo-edit-list.svg)

Without colors (`--no-color`, `NO_COLOR` set, `TERM=dumb`, or `color = false`), the screen is plain text with reverse video for the selection; nothing on screen depends on color alone.

## Colors: themes

`theme` picks one of seven color sets: `"default"` keeps your terminal's colors, and `"nord"`, `"solarized-light"`, `"dracula"`, `"gruvbox"`, `"pink-monster"` and `"dozy-pink"` paint the whole screen. Only colors change. See [Themes](themes.md) for a screenshot of each.

```toml
theme = "nord"
```

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
- `borders = "ascii"` draws window frames with `+ - |` instead of rounded lines (always ASCII with `ambiguous_wide = true`, so columns stay aligned).

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
