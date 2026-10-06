# mdgrid

[English](README.md)

フロントマター付き Markdown の集まりを、ターミナルの表で見て、その場で値を直す道具(Rust 製・単体の実行ファイル)。Obsidian を開かずに、Obsidian Bases の `.base` の table ビューの一部と互換の表を扱う。直したキーの値だけを書き戻し、ファイルの他の部分は1バイトも変えない。

最初の版(版1)の画面と核が揃い、フォルダか `.base` を開いて、表で見る・直す・差分を見て保存する、まで使える。仕様の正本は [specs/](specs/README.md)。

## 試す

ビルド済みのバイナリ(Linux の x86_64・aarch64、macOS の Intel・Apple シリコン、Windows の x86_64。補完と man ページつき)は、GitHub の Releases のページの各リリースにある。crates.io からは Rust 1.90 以上で `cargo install mdgrid --locked`。ソースから作るときは Rust 1.90 以上で:

ビルドする:

```sh
cargo build --release
```

サンプルの保管庫([examples/vault](examples/vault))を一時フォルダに写して開く:

```sh
cp -R examples/vault /tmp/mdgrid-sample
target/release/mdgrid /tmp/mdgrid-sample                 # フォルダを既定の表で開く
target/release/mdgrid /tmp/mdgrid-sample/タスク.base     # .base のビュー(進行中・状態ごと・期限・未対応の式・カード)で開く
target/release/mdgrid /tmp/mdgrid-sample/タスク/本を返す.md  # .md のファイルを渡すと、そのフォルダを開いてその行を選ぶ
```

写さずに見るだけなら `--readonly` を付ける:

```sh
target/release/mdgrid examples/vault/タスク.base --readonly
```

注意: `examples/vault` を `--readonly` なしで直接開いて保存すると、リポの中のサンプルが書き換わる。

英語の見本([examples/demo](examples/demo)。小さなチームの仕事の一覧と、4つの table ビューの `Tasks.base`)もある。英語の README の画面はこれで撮っている。見本の期日は 2026年10月の初めなので、それより後は多くが期限切れに見える。画面の写しと同じに見るには、今日とみなす日を `MDGRID_TODAY=2026-10-03` で渡す。

設定と `.base` のビューを一通り有効にした見本([examples/showcase](examples/showcase))は、設定のファイルを `--config` で渡して開く:

```sh
cp -R examples/showcase /tmp/mdgrid-showcase
target/release/mdgrid --config /tmp/mdgrid-showcase/config.toml /tmp/mdgrid-showcase/vault/プロジェクト.base
```

写さずに見るだけなら:

```sh
target/release/mdgrid --config examples/showcase/config.toml examples/showcase/vault/プロジェクト.base --readonly
```

設定の全項目(型・既定値・説明・例)は [docs/config.md](docs/config.md)(英語)と [docs/config.ja.md](docs/config.ja.md)(日本語)にある。`mdgrid --print-config` は全項目を既定値と説明のコメント付きの TOML で出すので、設定のファイルの出発点にできる。ノートを開くエディタは設定の `editor`(無ければ `$VISUAL`、`$EDITOR`、`vi` の順)。画面と起動の文言は英語と日本語で、設定の `language`(`auto`・`en`・`ja`。既定の `auto` は `LC_ALL`・`LC_MESSAGES`・`LANG` が `ja` で始まれば日本語、ほかは英語)で選ぶ。

シェルの補完の定義は `mdgrid --completions <シェル>`(bash・zsh・fish・elvish・powershell)、man ページは `mdgrid --man` で出せる:

```sh
mdgrid --completions zsh > ~/.zfunc/_mdgrid
mdgrid --man > ~/.local/share/man/man1/mdgrid.1
```

画面を出さずにビューの表を標準出力に出すには `--print` を付ける(形は `--format csv|json|md`、既定は csv)。行はビューの絞り込みと並べ替えのあと、列はビューの列の並びで、ノートは書き換えない。`--view` は `.base` のビューを先に、無ければ mdgrid のビューを名前で探す:

```sh
target/release/mdgrid examples/vault/タスク.base --print                     # csv(見出しの行つき)
target/release/mdgrid examples/vault/タスク.base --print --format json | jq .
target/release/mdgrid examples/vault --print --format md > 表.md
target/release/mdgrid examples/vault/タスク/本を返す.md --print --format json   # そのノートの1行だけ
target/release/mdgrid examples/vault --print --with-path                     # 先頭の列に各行のノートのパス
target/release/mdgrid examples/vault --print --filter 'status != "done"' --sort due   # .base を書かずに絞って並べる
target/release/mdgrid /tmp/vault --print --with-path > notes.csv   # 表計算で直して、
target/release/mdgrid /tmp/vault --apply notes.csv                 # 差分を見る(書かない)
target/release/mdgrid /tmp/vault --apply notes.csv --yes           # 違うセルだけを書く
```

`--print` が出すのはビューの列だけで、フォルダの表にはノートの名前の列が無い。どの行がどのノートかを知るには `--with-path` を付ける。`--with-path` を付けて出した CSV・JSON は、直して `--apply` で戻せる(既定は差分だけ。`--yes` で書く。path・値・読むだけのセルに1つでも理由があれば何も書かない。`--print` が出したのと同じ値は書き直さないので、直さずに戻しても何も書かない。書いている途中でノートが外で変わると、書き終えたノートはそのままで、終了コード 1)。`--filter <式>`(`.base` の式。何度でも渡せ、全部を満たす行)と `--sort <列>[:desc]`(何度でも)で、`.base` を書かずに絞って並べられる。`.base` のビューに添えると、絞り込みはビューの絞り込みと両方、並べ替えはビューの並べ替えの代わり。

画面で選んだ行を標準出力に出すには `--pick path|<列の名前>` を付ける。画面は端末(`/dev/tty`)に読むだけで出し、Enter で、印を付けた行(印が無ければ選んでいる行)ごとに、`path` ならノートのパス(起動の引数のフォルダにノートの相対のパスをつないだもの)、列の名前ならその列の値(印なしの素の文字。改行は空白)を1行ずつ出して終わる。`q` と、解く選択・絞り込み・検索が無いときの Esc は、何も出さずに終了コード 1(エディタでは開かない)。標準出力がパイプでも動く:

```sh
target/release/mdgrid examples/vault --pick path | tr '\n' '\0' | xargs -0 -o vi    # 選んだノートを開く(空白を含むパスでも)
target/release/mdgrid examples/vault --pick status
```

既定のキーの割り当てと、割り当て直し(設定の `[keys.<モード>]`)に書く動作の名前の一覧は [docs/keys.md](docs/keys.md)(英語)と [docs/keys.ja.md](docs/keys.ja.md)(日本語)にある。`.base` のうち mdgrid が解釈するもの(読む項目・演算子・関数・`file.*`)と解釈しないものは [docs/obsidian-bases.md](docs/obsidian-bases.md)(英語)と [docs/obsidian-bases.ja.md](docs/obsidian-bases.ja.md)(日本語)にある。書き戻しの安全(1バイトも変えない範囲・読むだけのノートの形と理由・外の変更とのぶつかり・一時ファイルと fsync による書き方)は [docs/safety.md](docs/safety.md)(英語)と [docs/safety.ja.md](docs/safety.ja.md)(日本語)にある。

フロントマターの無いノートと空のフロントマター(`---` が2行だけ)のノートにも、既定で書ける(フロントマターかキーの行が足される)。設定のファイルに `add_frontmatter = false` と書くと、この2つは読むだけになり理由が出る。区切りの間に空行やコメントの行があるものは、空ではなく普通のフロントマターとして扱う。

日付の形・月曜始まりのカレンダー・キーの割り当て直し・検索の欄と、ビュー(今週の作業・担当ごと・優先度と見積・完了・全部)の見どころは [examples/showcase/README.md](examples/showcase/README.md) にある。

`.base` のビューに `summaries` があると、表の下に集計の行(`合計`・`平均`・`最早` など。英語の画面では `Sum`・`Average`・`Earliest`)が出る。値は今見えている行(絞り込みのあと)から計算する。

色のテーマは設定の `theme` で選ぶ。端末の色のままの `"default"`(既定)のほかに、`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` の6つがある。テーマごとの設定の見本は [examples/themes/](examples/themes) にある(`mdgrid --config examples/themes/nord.toml examples/vault`)。

最初に使うキー:

| キー | 動き |
|---|---|
| `?` | ヘルプ(キーの一覧) |
| `:` | コマンドのパレット |
| `Enter` | セルを編集する |
| `%` | 列の値ごとの件数を見る。値を選ぶとその値の行だけ残す |
| `A` | 新しいキーの列を足す(値を入れたノートにだけ書く) |
| `:rename_key`・`:delete_key` | 列のキーの名前を全部のノートで変える・消す(保存の前の差分で見られる) |
| `Ctrl+S` | 差分を見て保存する |
| `[` `]` | `.base` のビューを切り替える |
| `q` | 終了 |
