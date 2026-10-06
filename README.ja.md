# mdgrid

[![CI](https://github.com/S6U5/mdgrid/actions/workflows/ci.yml/badge.svg)](https://github.com/S6U5/mdgrid/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/mdgrid.svg)](https://crates.io/crates/mdgrid)
[![GitHub release](https://img.shields.io/github/v/release/S6U5/mdgrid)](https://github.com/S6U5/mdgrid/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/mdgrid.svg)](LICENSE-MIT)

**Markdown のノートを、ターミナルの表計算のように。** mdgrid は、フロントマター付きの Markdown のフォルダを表で見せ(ノートが行、フロントマターのキーが列)、その場で値を直せる道具です。保存のときは差分を見せてから、直した値のバイトだけを書き戻します。

[English](README.md)

![mdgrid: 一覧から状態を選ぶ、カレンダーで日付を選ぶ、差分を見て保存する、まとめたビューに切り替える、操作の一覧を開く](docs/assets/ja/demo.gif)

- **フロントマターを表で直す。** ほかのノートが使っている値から選ぶ(打つと候補が絞られる)、カレンダーで日付を選ぶ、タグとチェックボックスを切り替える、たくさんの行に同じ値を一度に入れる、新しいキーの列を足す、キーの名前を全部のノートで変える・消す、ができます。
- **ファイルのほかの部分には触らない。** ほかのキー、キーの順、コメント、改行コード、本文は1バイトも変えません。保存のたびにファイルごとの差分を先に見せ、一時ファイルから置き換えて書き、その間に外でファイルが変わっていれば止まります。
- **Obsidian Bases がわかる。** `.base` を開くと table ビューがタブに並び、絞り込み・並べ替え・まとまり・集計の行(`合計`・`平均`・`最早` など)と式の一部が使えます。Obsidian を入れていなくても動き、`.base` を書き換えることはありません。
- **シェルになじむ。** `--print` はビューを CSV・JSON・Markdown の表で出し、`--filter` と `--sort` でその場の問い合わせもできます。出した CSV を表計算で直して `--apply` で戻すと、差分を見せて、変わったセルだけを書きます。`--pick` は画面で選んだ行のパスを次のコマンドに渡します。
- **キーボードで動かせて、迷わない。** Vim のキーも矢印のキーも使え、全部のキーを割り当て直せます。`x`(かセルの右クリック)で、そこでできる操作とキーが出ます。コマンドのパレット(`:`)とヘルプ(`?`)にほかの全部があります。
- **端末の色のほかに6つの色のテーマ。** 設定に `theme = "nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"` と書きます。既定は端末の色のままです。
- **単体の実行ファイル。** Rust 製。画面は英語と日本語。`NO_COLOR` に従います。

## 📦 入れる

**ビルド済みのバイナリ。** [GitHub の Releases](https://github.com/S6U5/mdgrid/releases) の各リリースに、Linux(x86_64・aarch64)、macOS(Intel・Apple シリコン)、Windows(x86_64)のアーカイブがあります。展開して `mdgrid` を `PATH` の通った場所に置きます。シェルの補完(`completions/`)と man ページ(`mdgrid.1`)も入っています。

**crates.io から。** Rust 1.90 以上で:

```sh
cargo install mdgrid --locked
```

**ソースから。**

```sh
git clone https://github.com/S6U5/mdgrid.git
cd mdgrid
cargo install --path . --locked
```

Homebrew にはまだありません。

## 🚀 試す

日本語の見本の保管庫([examples/vault](examples/vault))で試せます。保存するとファイルが書き換わるので、先に一時フォルダに写します:

```sh
cp -R examples/vault /tmp/mdgrid-sample
mdgrid /tmp/mdgrid-sample/タスク.base       # .base のビュー(進行中・状態ごと・期限・未対応の式・カード)
mdgrid /tmp/mdgrid-sample                   # フォルダの全部のノート
mdgrid /tmp/mdgrid-sample/タスク/本を返す.md  # .md を渡すと、そのフォルダを開いてその行を選ぶ
```

英語の見本([examples/demo](examples/demo)。小さなチームの仕事の一覧)もあり、README の画面はこれで撮っています。見本の期日は 2026年10月の初めなので、画面の写しと同じに見るには今日とみなす日を渡します: `MDGRID_TODAY=2026-10-03 mdgrid /tmp/mdgrid-demo/Tasks.base`。

自分のノートを開くときは、`--readonly` を付ければ書き込む心配なく見られます:

```sh
mdgrid ~/notes --readonly
```

| キー | 動き |
|---|---|
| `Enter` | 選んだセルを編集する |
| `x` | 選んだセルでできる操作 |
| `%` | 列の値ごとの件数を見る。値を選ぶとその値の行だけ残す |
| `A` | 新しいキーの列を足す(値を入れたノートにだけ書く) |
| `:rename_key`・`:delete_key` | 列のキーの名前を全部のノートで変える・消す(保存の前の差分で見られる) |
| `Ctrl+S` | 差分を見て保存する |
| `/`・`\` | 検索・打ちながら行を絞る |
| `o` | ビューの設定: 列・絞り込み・並べ替え・まとまり |
| `[` `]` | ビューを切り替える |
| `:` | コマンドのパレット |
| `?` | ヘルプ: 全部のキーと、セルの印(`∅`・`!`・`*`・`#` など)の意味 |
| `q` | 終了 |

画面の言葉は設定の `language`(`auto`・`en`・`ja`)で選びます。既定の `auto` は、`LC_ALL`・`LC_MESSAGES`・`LANG` が `ja` で始まれば日本語です。

## 🔍 もう少し詳しく

ほかのノートが使っている値から選ぶか、新しい値を打ちます。打つと、その文字を含む値に絞られます:

![一覧から状態の値を選ぶ](docs/assets/ja/demo-edit-list.svg)

日付の列はカレンダーが開きます:

![カレンダーで日付を選ぶ](docs/assets/ja/demo-calendar.svg)

保存するまで何も書かず、何が変わるかをそのまま見せます:

![1行の差分を見せる保存の確認](docs/assets/ja/demo-save.svg)

列で `%` を押すと値ごとの件数が出て、値を選ぶとその行だけが残ります:

![owner の列の値ごとの件数](docs/assets/ja/demo-freq.svg)

`x` で、選んだセルでできることが出ます:

![セルの操作の一覧](docs/assets/ja/demo-menu.svg)

`.base` のビューは行を見出しでまとめられ、見出しは `Enter` で畳めます:

![状態でまとめた .base のビュー](docs/assets/ja/demo-group.svg)

`summaries` のあるビューは、表の下に集計の行が出ます。値は今見えている行から計算します(見本の `Open` のビューでは、期日の `最早` と見積の `合計`)。

色のテーマは設定で選びます(ここでは `theme = "nord"`):

![Nord のテーマの表](docs/assets/ja/demo-theme.svg)

テーマごとの設定の見本は [examples/themes/](examples/themes) にあります: `mdgrid --config examples/themes/nord.toml examples/demo`。

## 🧰 スクリプトから使う

```sh
mdgrid ~/notes/Tasks.base --print                         # 見出しの行つきの CSV
mdgrid ~/notes/Tasks.base --print --format json | jq .
mdgrid ~/notes --print --format md > table.md
mdgrid ~/notes/todo.md --print --format json             # そのノートの1行だけ
mdgrid ~/notes --print --with-path                        # 先頭の列に各行のノートのパス
mdgrid ~/notes --print --filter 'status != "done"' --sort due   # .base を書かずに絞って並べる
mdgrid ~/notes --print --with-path > notes.csv             # notes.csv を表計算で直して:
mdgrid ~/notes --apply notes.csv                          # 差分を見る(何も書かない)
mdgrid ~/notes --apply notes.csv --yes                    # 変わったセルだけを書く
mdgrid ~/notes --pick path | tr '\n' '\0' | xargs -0 -o vi   # ノートを選んで開く(空白を含むパスでも)
```

`--print` が出すのはビューの列だけで、フォルダの表にはノートの名前の列がありません。どの行がどのノートかを知るには `--with-path` を付けます。

`--print` はノートを書き換えません。`--apply` は先に全部の行を確かめ、パス・値・読むだけのセルに1つでも問題があれば何も書きません。`--yes` が無ければ差分を見せるだけです。`--print` が出したのと同じ値は書き直さないので、直さずに戻しても何も書きません。`--apply --yes` で書いている途中にノートが外で変わると、書き終えたノートはそのままで、終了コード 1 で終わります。`--pick` は表を読むだけで開き、印を付けた行ごとに1行を出します。

## 📚 文書

- [説明書](docs/manual/ja/index.md) — はじめかた、やりたいことごとの手順、7つのテーマ、全部の画面の写し。
- [設定](docs/config.ja.md) — 全部の項目と既定値。`mdgrid --print-config` は説明のコメント付きの設定を出すので、出発点にできます。
- [キー](docs/keys.ja.md) — モードごとの既定のキーと、割り当て直し方。
- [Obsidian Bases への対応](docs/obsidian-bases.ja.md) — `.base` のどこを読み、評価し、無視するか。
- [書き戻しの安全](docs/safety.ja.md) — 何を書き、何に触らないか。読むだけになるノートとその理由。
- [見本の設定と保管庫](examples/showcase/README.md) — ほとんどの機能を有効にした見本。`examples/vault` は、null・空・型の合わない値、読むだけのノート、同期の競合ファイルなどの端の形をそろえています。
- シェルの補完は `mdgrid --completions <bash|zsh|fish|elvish|powershell>`、man ページは `mdgrid --man`。

## 🤝 参加する

バグを見つけた、こうなってほしい、という案があれば [Issue を開いて](https://github.com/S6U5/mdgrid/issues/new/choose)ください(日本語で大丈夫です)。mdgrid は作者と AI のコーディングのエージェント(Claude Code)で直しているので、何をして、何を期待し、何が起きたかが分かる Issue があれば、たいていそれだけで直したり機能を足したりできます。Pull request は共同作業者だけにしています。詳しくは [CONTRIBUTING.md](CONTRIBUTING.md) を見てください。セキュリティの問題は [SECURITY.md](SECURITY.md) のとおり非公開で知らせてください。

## 🏗️ 作り方

仕様の正本は [specs/](specs/README.md) にあり、変更はどれも提案と決定の記録を通ります。人が決めた要件を守る試験には錠が掛かっていて、黙って弱めることはできません。

## 📄 ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) と [MIT license](LICENSE-MIT) のどちらかを選んで使えます。

mdgrid は独立のプロジェクトで、Obsidian とは関係がなく、Obsidian の承認を受けたものでもありません。
