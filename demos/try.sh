#!/bin/sh
# 手で試す: 見本(examples/)を一時フォルダに写し、その中の設定・状態で mdgrid を開く。終わったら写しを消す。
# 本物の設定とノートには触らない。リポの根で動かす。見本の今日は 2026-10-06(MDGRID_TODAY で変えられる)。
#   sh demos/try.sh              主役の見本(examples/studio の Tasks.base。リンクのセルで x、R で関係マップ)
#   sh demos/try.sh workspace    ワークスペースの見本(Work に Tasks と Projects、Team に People。-w Work で開く)
#   sh demos/try.sh configs      多くの機能を入れた設定(examples/configs/studio.toml)で studio を開く
#   sh demos/try.sh quirks       扱いにくいノートの見本(examples/quirks。空・食い違い・読むだけのノートなど)
#   sh demos/try.sh ai-human     人と AI のタスクに分ける .base(examples/ai-human と設定)
#   sh demos/try.sh places       登録した表の見本(examples/places。studio の表とほかの見本の一覧)
#   sh demos/try.sh studio -- --readonly   -- のあとは mdgrid に渡す
set -eu
cd "$(dirname "$0")/.."
ROOT="$PWD"
what="${1:-studio}"
[ $# -gt 0 ] && shift
[ "${1:-}" = "--" ] && shift
cargo build --release -q
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM
cp -R examples "$work/"
mkdir -p "$work/config/mdgrid" "$work/state"
export MDGRID_TODAY="${MDGRID_TODAY:-2026-10-06}"
case "$what" in
  studio) dir="$work/examples/studio"; target="Tasks.base" ;;
  workspace)
    s="$work/examples/studio"
    printf '[[workspace]]\nname = "Work"\n\n[[workspace.table]]\nname = "Tasks"\npath = "%s/Tasks"\n\n[[workspace.table]]\nname = "Projects"\npath = "%s/Projects"\n\n[[workspace]]\nname = "Team"\n\n[[workspace.table]]\nname = "People"\npath = "%s/People"\n' \
      "$s" "$s" "$s" > "$work/config/mdgrid/workspaces.toml"
    dir="$s"
    set -- -w Work "$@"
    target="Tasks"
    ;;
  configs)
    cp "$work/examples/configs/studio.toml" "$work/config/mdgrid/config.toml"
    dir="$work/examples/studio"; target="Tasks.base"
    ;;
  quirks) dir="$work/examples/quirks"; target="." ;;
  ai-human)
    cp "$work/examples/ai-human/config.toml" "$work/config/mdgrid/config.toml"
    dir="$work/examples/ai-human"; target="Tasks.base"
    ;;
  places)
    sed "s|examples/|$work/examples/|" "$work/examples/places/mdgrid/places.toml" > "$work/config/mdgrid/places.toml"
    dir="$work/examples"; target=""
    ;;
  starter)
    XDG_CONFIG_HOME="$work/config" XDG_STATE_HOME="$work/state" "$ROOT/target/release/mdgrid" "$work/books" --init-workspace --starter reading --with-samples >/dev/null
    dir="$work/books"; target="."
    ;;
  *) echo "unknown sample: $what (studio, workspace, configs, quirks, ai-human, places, starter)" >&2; exit 2 ;;
esac
cd "$dir"
if [ -n "$target" ]; then
  XDG_CONFIG_HOME="$work/config" XDG_STATE_HOME="$work/state" "$ROOT/target/release/mdgrid" "$target" "$@"
else
  XDG_CONFIG_HOME="$work/config" XDG_STATE_HOME="$work/state" "$ROOT/target/release/mdgrid" "$@"
fi
