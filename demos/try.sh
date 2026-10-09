#!/bin/sh
# 手で試す: 見本(examples/)を一時フォルダに写し、その中の設定・状態で mdgrid を開く。終わったら写しを消す。
# 本物の設定とノートには触らない。リポの根で動かす。
#   sh demos/try.sh              関係マップの見本(examples/relations の3つを登録して tasks を開く。R で関係マップ)
#   sh demos/try.sh workspace    ワークスペースの見本(Work に tasks と projects、Team に members。-w Work で開く)
#   sh demos/try.sh demo         英語の見本の表(examples/demo の Tasks。期限の印は 2026-10-03 を今日として)
#   sh demos/try.sh ai-human     人と AI のタスクに分ける .base(examples/ai-human の vault-en と設定)
#   sh demos/try.sh showcase     多くの機能を入れた設定と保管庫(examples/showcase。日本語)
#   sh demos/try.sh vault        日本語の見本の保管庫(examples/vault。空・食い違い・読むだけのノートなど)
#   sh demos/try.sh relations -- --readonly   -- のあとは mdgrid に渡す
set -eu
cd "$(dirname "$0")/.."
ROOT="$PWD"
what="${1:-relations}"
[ $# -gt 0 ] && shift
[ "${1:-}" = "--" ] && shift
cargo build --release -q
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM
cp -R examples "$work/"
mkdir -p "$work/config/mdgrid" "$work/state"
case "$what" in
  relations)
    r="$work/examples/relations"
    printf '[[place]]\nname = "Tasks"\npath = "%s/tasks"\n\n[[place]]\nname = "Projects"\npath = "%s/projects"\n\n[[place]]\nname = "Members"\npath = "%s/members"\n' \
      "$r" "$r" "$r" > "$work/config/mdgrid/places.toml"
    dir="$r"
    target="tasks"
    ;;
  workspace)
    r="$work/examples/relations"
    printf '[[workspace]]\nname = "Work"\n\n[[workspace.table]]\nname = "Tasks"\npath = "%s/tasks"\n\n[[workspace.table]]\nname = "Projects"\npath = "%s/projects"\n\n[[workspace]]\nname = "Team"\n\n[[workspace.table]]\nname = "Members"\npath = "%s/members"\n' \
      "$r" "$r" "$r" > "$work/config/mdgrid/workspaces.toml"
    dir="$r"
    set -- -w Work "$@"
    target="tasks"
    ;;
  demo) dir="$work/examples/demo"; target="Tasks"; export MDGRID_TODAY="${MDGRID_TODAY:-2026-10-03}" ;;
  ai-human)
    cp "$work/examples/ai-human/config.toml" "$work/config/mdgrid/config.toml"
    dir="$work/examples/ai-human/vault-en"; target="Tasks.base"
    ;;
  showcase)
    cp "$work/examples/showcase/config.toml" "$work/config/mdgrid/config.toml"
    dir="$work/examples/showcase/vault"; target="プロジェクト.base"; export MDGRID_TODAY="${MDGRID_TODAY:-2026-10-02}"
    ;;
  vault) dir="$work/examples/vault"; target="." ;;
  *) echo "unknown sample: $what (relations, workspace, demo, ai-human, showcase, vault)" >&2; exit 2 ;;
esac
cd "$dir"
XDG_CONFIG_HOME="$work/config" XDG_STATE_HOME="$work/state" "$ROOT/target/release/mdgrid" "$target" "$@"
