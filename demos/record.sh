#!/bin/sh
# 機能紹介の録画を撮る(VHS が要る: brew install vhs)。出力は demos/out/(git には入れない)。
#   sh demos/record.sh            全部
#   sh demos/record.sh places     選んだものだけ(名前は demos/<名前>.tape)
set -eu
cd "$(dirname "$0")/.."
command -v vhs >/dev/null 2>&1 || { echo "vhs が要る(brew install vhs)" >&2; exit 1; }
cargo build --release -q
if [ $# -eq 0 ]; then
  set -- $(ls demos/*.tape | sed 's|demos/||; s|\.tape$||' | grep -v '^setup$')
fi
for name in "$@"; do
  echo "撮る: $name"
  vhs "demos/$name.tape" >/dev/null
done
echo "できた: demos/out/"
