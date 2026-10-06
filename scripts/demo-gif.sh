#!/bin/sh
# Build docs/assets/demo.gif from the gif-* scenes in docs/manual-scenarios.toml.
# Needs: cargo, Python 3.11+, rsvg-convert (librsvg), ffmpeg, and the app-manual skill's tui_shot.py
# (path in TUI_SHOT). Run from anywhere: the script moves to the repository root.
set -eu
cd "$(dirname "$0")/.."
: "${TUI_SHOT:?set TUI_SHOT to the path of scripts/tui_shot.py in the app-manual skill}"
PY="${PYTHON:-python3}"
cargo build --release
ids=$(grep -o 'id = "gif-[0-9]*"' docs/manual-scenarios.toml | sed 's/id = "\(.*\)"/\1/')
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
for id in $ids; do
  env -u LC_ALL -u LC_MESSAGES "$PY" "$TUI_SHOT" docs/manual-scenarios.toml --out "$work/shots" \
    --lang en --env LANG=en_US.UTF-8 --only "$id" >/dev/null
done
n=0
for id in $ids; do
  n=$((n + 1))
  rsvg-convert -w 1000 "$work/shots/en/images/$id.svg" -o "$work/$(printf 'f%02d' "$n").png"
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
  docs/assets/demo.gif
ls -l docs/assets/demo.gif
