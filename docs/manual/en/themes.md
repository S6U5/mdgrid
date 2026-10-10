# Themes

[日本語](../ja/themes.md) · [Contents](index.md)

You can pick one of twelve color themes for the screen (or `auto`, which follows the terminal background). Only the colors change; text and marks (`*` for a pending change, `>` for the selected row, and so on) stay the same.

## How to choose

Add two lines to the config file (`~/.config/mdgrid/config.toml`):

```toml
[look]
theme = "nord"
```

A workspace or a single table can have its own theme: see [A look per workspace, table or view](appearance.md#a-look-per-workspace-table-or-view).

To try one before changing your config, pass a sample config with `--config` (from the repository root):

```sh
mdgrid --config examples/themes/nord.toml /tmp/mdgrid-sample
```

| Name | Look |
|---|---|
| [`default` (default)](#default) | The current look: terminal colors, reverse video for the selection and bands |
| [`nord`](#nord) | Calm blues |
| [`solarized-light`](#solarized-light) | A light background |
| [`dracula`](#dracula) | Purple and pink accents |
| [`gruvbox`](#gruvbox) | Warm browns and orange |
| [`pink-monster`](#pink-monster) | Neon pink and lime on dark magenta |
| [`dozy-pink`](#dozy-pink) | Soft pink and cream |
| [`sumi`](#sumi) | Dark ink, soft grays and a teal accent |
| [`slate`](#slate) | Slate with a violet accent |
| [`saas`](#saas) | Light gray with an indigo accent |
| [`saas-dark`](#saas-dark) | The dark version of saas |
| [`paper`](#paper) | A light background with a blue accent |
| `auto` | `saas` on a light terminal background, `sumi` on a dark one. Pick your own pair with `theme = { light = "paper", dark = "saas-dark" }` |

## Gallery

<a id="default"></a>

### `default` — The default look

The current look: terminal colors, reverse video for the selection and bands. This is also what you get with no setting.

![The default look](images/table.svg)

<a id="nord"></a>

### `nord` — Nord

Calm blues. Config: `theme = "nord"`.

![Nord](images/theme-nord.svg)

<a id="solarized-light"></a>

### `solarized-light` — Solarized Light

A light background. Config: `theme = "solarized-light"`.

![Solarized Light](images/theme-solarized-light.svg)

<a id="dracula"></a>

### `dracula` — Dracula

Purple and pink accents. Config: `theme = "dracula"`.

![Dracula](images/theme-dracula.svg)

<a id="gruvbox"></a>

### `gruvbox` — Gruvbox

Warm browns and orange. Config: `theme = "gruvbox"`.

![Gruvbox](images/theme-gruvbox.svg)

<a id="pink-monster"></a>

### `pink-monster` — Pink Monster

Neon pink and lime on dark magenta. Config: `theme = "pink-monster"`.

![Pink Monster](images/theme-pink-monster.svg)

<a id="dozy-pink"></a>

### `dozy-pink` — Dozy Pink

Soft pink and cream. Config: `theme = "dozy-pink"`.

![Dozy Pink](images/theme-dozy-pink.svg)

<a id="sumi"></a>

### `sumi` — Sumi

Dark ink, soft grays and a teal accent. Setting: `theme = "sumi"`.

![Sumi](images/theme-sumi.svg)

<a id="slate"></a>

### `slate` — Slate

Slate with a violet accent. Setting: `theme = "slate"`.

![Slate](images/theme-slate.svg)

<a id="saas"></a>

### `saas` — SaaS

Light gray with an indigo accent. Setting: `theme = "saas"`.

![SaaS](images/theme-saas.svg)

<a id="saas-dark"></a>

### `saas-dark` — SaaS Dark

The dark version of saas. Setting: `theme = "saas-dark"`.

![SaaS Dark](images/theme-saas-dark.svg)

<a id="paper"></a>

### `paper` — Paper

A light background with a blue accent. Setting: `theme = "paper"`.

![Paper](images/theme-paper.svg)

## Notes

- With `NO_COLOR`, `--no-color`, `color = false` or `TERM=dumb`, no colors are used even if a theme is set.
- On terminals without true color (`COLORTERM` is not `truecolor`), the nearest 256 colors are used.
- An unknown name gives a warning at startup and opens with `default`.
- The bottom-right cell is never painted, to avoid the terminal's automatic wrap. On light themes the terminal's own background shows there.
- Zebra stripes (`[display] zebra = true`) use the theme's colors too; the samples other than `default` are taken with them on.
- Every setting is listed in [Configuration](../../config.md#theme).
