#!/bin/sh
# Build docs/assets/hero.png (or, with `ja`, docs/assets/ja/hero.png): the same table in the dark default look
# (sumi) and the light SaaS look, side by side, drawn with a Nerd Font so the round pill ends show.
# The PNG is drawn here, so readers do not need a Nerd Font (an SVG would use the reader's fonts).
# Needs: cargo, Python 3.11+, rsvg-convert (librsvg), ffmpeg, a "JetBrainsMono Nerd Font Mono" font, and the
# app-manual skill's tui_shot.py (path in TUI_SHOT). Run from anywhere: the script moves to the repository root.
set -eu
cd "$(dirname "$0")/.."
: "${TUI_SHOT:?set TUI_SHOT to the path of scripts/tui_shot.py in the app-manual skill}"
PY="${PYTHON:-python3}"
lang="${1:-en}"
case "$lang" in
  en) locale=en_US.UTF-8; out=docs/assets/hero.png ;;
  ja) locale=ja_JP.UTF-8; out=docs/assets/ja/hero.png ;;
  *) echo "usage: $0 [en|ja]" >&2; exit 2 ;;
esac
cargo build --release
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
font="'JetBrainsMono Nerd Font Mono', "
for variant in dark light; do
  mkdir -p "$work/$variant/mdgrid"
  case $variant in
    dark) printf 'theme = "sumi"\nnerd_font = true\n' > "$work/$variant/mdgrid/config.toml"; bg="#16171b" ;;
    light) printf 'theme = "saas"\nnerd_font = true\n\n[style]\npreset = "saas"\n' > "$work/$variant/mdgrid/config.toml"; bg="#f6f7f9" ;;
  esac
  env -u LC_ALL -u LC_MESSAGES "$PY" "$TUI_SHOT" docs/manual-scenarios.toml --out "$work/shots-$variant" \
    --lang "$lang" --env LANG="$locale" --env COLORTERM=truecolor --env XDG_CONFIG_HOME="$work/$variant" \
    --only table >/dev/null
  svg="$work/shots-$variant/$lang/images/table.svg"
  # The Nerd Font first (round pill ends), and the frame around the screen in the theme's background.
  # tui_shot draws unpainted cells (and the frame) in its default #1e1e1e; use the theme's background there.
  sed -e "s/font-family=\"/font-family=\"$font/" -e "s/#1e1e1e/$bg/g" "$svg" > "$work/$variant.svg"
  rsvg-convert -w 1000 "$work/$variant.svg" -o "$work/$variant.png"
done
# Side by side with a transparent gap (fits both the light and the dark GitHub page).
ffmpeg -loglevel error -y -i "$work/dark.png" -i "$work/light.png" \
  -filter_complex "[0]format=rgba,pad=iw+24:ih:0:0:color=0x00000000[a];[1]format=rgba[b];[a][b]hstack=inputs=2" "$out"
ls -l "$out"
