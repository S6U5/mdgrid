# 共通の準備(各台本が Source で読む。録画には映さない)。
# 英語の見本(examples/demo)で動かす。examples/ を一時フォルダに写し、ホーム・設定・状態もその中にして動かす。本物のノートと設定には触らない。
# リポの根で `vhs demos/<名前>.tape` と動かす前提(cargo build --release を先に)。

Hide
Type `export DEMO="$(cd "$(mktemp -d)" && pwd -P)" ROOT="$PWD" && cp -R examples "$DEMO/" && mkdir -p "$DEMO/config/mdgrid" "$DEMO/state"`
Enter
Type `export HOME="$DEMO" XDG_CONFIG_HOME="$DEMO/config" XDG_STATE_HOME="$DEMO/state" PATH="$ROOT/target/release:$PATH" PS1='$ '`
Enter
Type `cd "$DEMO/examples/demo" && clear`
Enter
Sleep 300ms
Show
