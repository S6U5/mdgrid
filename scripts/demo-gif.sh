#!/bin/sh
# Build docs/assets/demo.gif (or, with `ja` as the first argument, docs/assets/ja/demo.gif with the
# Japanese screens) from the gif-* scenes in docs/manual-scenarios.toml, in true color (the modern look and
# the cell parts of SR-33 and SR-35 need colors).
# Needs: cargo, Python 3.11+, rsvg-convert (librsvg), ffmpeg, a "JetBrainsMono Nerd Font Mono" font, and the app-manual skill's tui_shot.py
# (path in TUI_SHOT). Run from anywhere: the script moves to the repository root.
set -eu
cd "$(dirname "$0")/.."
: "${TUI_SHOT:?set TUI_SHOT to the path of scripts/tui_shot.py in the app-manual skill}"
PY="${PYTHON:-python3}"
lang="${1:-en}"
case "$lang" in
  en) locale=en_US.UTF-8; out=docs/assets/demo.gif ;;
  ja) locale=ja_JP.UTF-8; out=docs/assets/ja/demo.gif ;;
  *) echo "usage: $0 [en|ja]" >&2; exit 2 ;;
esac
cargo build --release
# Draw with a Nerd Font (round pill ends) and nerd_font = true; the GIF is drawn here, so readers need no font.
work_cfg=$(mktemp -d)
mkdir -p "$work_cfg/mdgrid"
printf 'nerd_font = true\n' > "$work_cfg/mdgrid/config.toml"
ids=$(grep -o 'id = "gif-[0-9]*"' docs/manual-scenarios.toml | sed 's/id = "\(.*\)"/\1/')
work=$(mktemp -d)
trap 'rm -rf "$work" "$work_cfg"' EXIT
for id in $ids; do
  env -u LC_ALL -u LC_MESSAGES "$PY" "$TUI_SHOT" docs/manual-scenarios.toml --out "$work/shots" \
    --lang "$lang" --env LANG="$locale" --env COLORTERM=truecolor --env XDG_CONFIG_HOME="$work_cfg" \
    --only "$id" >/dev/null
done
n=0
for id in $ids; do
  n=$((n + 1))
  sed "s/font-family=\"/font-family=\"'JetBrainsMono Nerd Font Mono', /" "$work/shots/$lang/images/$id.svg" > "$work/$id.svg"
  rsvg-convert -w 1000 "$work/$id.svg" -o "$work/$(printf 'f%02d' "$n").png"
done
# Each frame stays 1.4 s; the last one 3 s. One palette for all frames keeps the file small.
: > "$work/list.txt"
i=0
for f in "$work"/f*.png; do
  i=$((i + 1))
  d=1.4
  [ "$i" -eq "$n" ] && d=3
  printf "file '%s'\nduration %s\n" "$f" "$d" >> "$work/list.txt"
done
printf "file '%s'\n" "$f" >> "$work/list.txt"
# Variable frame rate keeps one GIF frame per screen; the GIF encoder stores only the changed rectangle.
ffmpeg -loglevel error -y -f concat -safe 0 -i "$work/list.txt" -fps_mode vfr \
  -vf "split[a][b];[a]palettegen=max_colors=64:stats_mode=full[p];[b][p]paletteuse=dither=none:diff_mode=rectangle" \
  "$out"
ls -l "$out"
