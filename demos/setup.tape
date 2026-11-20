# 共通の準備(各台本が Source で読む。録画には映さない)。
# 主役の見本(examples/studio。架空の制作会社)で動かす。今日は 2026-10-06 に固定する。examples/ を一時フォルダに写し、ホーム・設定・状態もその中にして動かす。本物のノートと設定には触らない。
# Herdr の窓から撮っても、録画の中の mdgrid が Herdr に開き直す命令を伝えないように、Herdr の環境変数を外す(resume.herdr-report)。
# リポの根で `vhs demos/<名前>.tape` と動かす前提(cargo build --release を先に)。

Hide
Type `export DEMO="$(cd "$(mktemp -d)" && pwd -P)" ROOT="$PWD" && cp -R examples "$DEMO/" && mkdir -p "$DEMO/config/mdgrid" "$DEMO/state"`
Enter
Type `export HOME="$DEMO" XDG_CONFIG_HOME="$DEMO/config" XDG_STATE_HOME="$DEMO/state" PATH="$ROOT/target/release:$PATH" MDGRID_TODAY=2026-10-06 PS1='$ ' && unset HERDR_ENV HERDR_PANE_ID HERDR_BIN_PATH`
Enter
Type `cd "$DEMO/examples/studio" && clear`
Enter
Sleep 300ms
Show
