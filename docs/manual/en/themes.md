# Themes

[日本語](../ja/themes.md) · [Contents](index.md)

You can pick one of seven color themes for the screen. Only the colors change; text and marks (`*` for a pending change, `>` for the selected row, and so on) stay the same.

## How to choose

Add one line to the config file (`~/.config/mdgrid/config.toml`):

```toml
theme = "nord"
```

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

## Notes

- With `NO_COLOR`, `--no-color`, `color = false` or `TERM=dumb`, no colors are used even if a theme is set.
- On terminals without true color (`COLORTERM` is not `truecolor`), the nearest 256 colors are used.
- An unknown name gives a warning at startup and opens with `default`.
- The bottom-right cell is never painted, to avoid the terminal's automatic wrap. On light themes the terminal's own background shows there.
- Zebra stripes (`[display] zebra = true`) use the theme's colors too; the samples other than `default` are taken with them on.
- Every setting is listed in [Configuration](../../config.md#theme).
