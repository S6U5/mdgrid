# mdgrid

[![CI](https://github.com/S6U5/mdgrid/actions/workflows/ci.yml/badge.svg)](https://github.com/S6U5/mdgrid/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/mdgrid.svg)](https://crates.io/crates/mdgrid)
[![GitHub release](https://img.shields.io/github/v/release/S6U5/mdgrid)](https://github.com/S6U5/mdgrid/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/mdgrid.svg)](LICENSE-MIT)

**Markdown のフォルダを、ひとことで開ける自分の表に。**

mdgrid にフォルダを渡すと、ノートが行、フロントマターのキーが列の表になり、その場で値を直せます。フォルダに短い alias を付ければ、`tasks`・`books`・`meetings` がそのまま自分のコマンドになり、前に使ったときの表がそのまま開きます。

[English](README.md) · [説明書](https://s6u5.github.io/mdgrid/manual/ja/) · [見た目のカタログ](https://s6u5.github.io/mdgrid/catalog/?lang=ja) · [画面の一覧](https://s6u5.github.io/mdgrid/gallery.html?lang=ja)

![同じ表を、既定の暗い見た目(墨)と明るい SaaS の見た目で](docs/assets/ja/hero.png)

![mdgrid: 一覧から状態を選ぶ、カレンダーで日付を選ぶ(日時の列は時刻の欄も)、差分を見て保存する、まとめたビューに切り替える、操作の一覧を開く](docs/assets/ja/demo.gif)

## ✨ できること

- **フォルダごとに1つのコマンド。** 用意は `alias tasks='mdgrid ~/notes/Tasks'` の1行だけです。mdgrid はフォルダごとに、列の並び・幅、絞り込み・並べ替え・まとまり、保存したビューを覚えているので、`tasks` と打てば毎回同じ表が開きます。よく使う表を名前と分類を付けて登録しておけば、引数なしの `mdgrid` で一覧から選んで切り替えられます。データベースも取り込みも要らず、ノートはただの Markdown のままです。
- **つながった表と関係マップ。** ノートどうしをリンクでつなげば(`project: "[[mdgrid]]"`)、フォルダをつながった表として扱えます。リンクは名前で出て、Enter で行き先を名前から選べ、行き先へ移ったり、ここを指すノートを並べたりできます。`R` で表とつながりの図(関係マップ)が出ます。仕事ごとの表をワークスペースにまとめれば(Obsidian の保管庫や git のリポを自動で見つけることも)、リンクはその中だけで探します。
- **フロントマターを表計算のように直す。** セルは GUI の表のような部品で見せます: チェックボックス、タグや状態の色の付いた札、列ごとの型の印。ほかのノートが使っている値から選ぶ、カレンダーで日付を選ぶ、タグとチェックボックスを切り替える、たくさんの行に同じ値を一度に入れる、新しいキーの列を足す、キーの名前を全部のノートで変える、ができます。
- **ファイルのほかの部分には触らない。** ほかのキー、キーの順、コメント、改行コード、本文は1バイトも変えません。書く前に、ファイルごとの差分を必ず見せます。
- **ほかのコマンドと組み合わせられる。** 表を CSV・TSV・JSON・Markdown で出す(画面の表もパレットからファイルに書き出せる)、出した CSV を表計算で直して戻す、画面で選んだノートのパスを次のコマンドに渡す、ができます。
- **キーボードで覚えやすい。** Vim のキーも矢印のキーも使えます。どのセルでも `x` を押せばそこでできることが、`?` で全部のキーが出ます。

ほかに: 7つの色のテーマ、英語と日本語の画面、Rust 製の単体の実行ファイル。Obsidian を使っているなら `.base` も開けます([下を参照](#-obsidian-を使っている人へ))。

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

## 🚀 はじめる

### 1. 見本で試す

日本語の見本([examples/vault](examples/vault))で試せます。保存するとファイルが書き換わるので、先に一時フォルダに写します:

```sh
cp -R examples/vault /tmp/mdgrid-sample
mdgrid /tmp/mdgrid-sample/タスク
```

矢印のキーで動き、`Enter` で値を直し、`Ctrl+S` で差分を見て保存し、`q` で終わります。

英語の見本([examples/demo](examples/demo)。小さなチームの仕事の一覧)もあり、README の画面はこれで撮っています。見本の期日は 2026年10月の初めなので、画面の写しと同じに見るには今日とみなす日を渡します: `MDGRID_TODAY=2026-10-03 mdgrid /tmp/mdgrid-demo/Tasks`。

リポを clone すれば、見本を全部1つのコマンドで開けます。`demos/try.sh` は mdgrid をビルドし、見本を一時フォルダに写し、その中の設定で開き、終わると写しを消します。自分のノートと設定には触りません:

```sh
git clone https://github.com/S6U5/mdgrid && cd mdgrid
sh demos/try.sh vault       # 上の日本語の見本(空・食い違う値・読むだけのノートなども)
sh demos/try.sh relations   # [[リンク]] でつないだ tasks → projects → members。R で関係マップ
sh demos/try.sh workspace   # 同じ表をワークスペースにまとめたもの(-w Work)
sh demos/try.sh showcase    # 多くの機能を入れた設定と保管庫
sh demos/try.sh ai-human    # 人の仕事と AI の仕事に分ける .base(英語)
sh demos/try.sh demo        # 英語の見本(README の画面)
```

`--` のあとは mdgrid に渡ります。たとえば `sh demos/try.sh demo -- --readonly`。

### 2. 自分のノートを開く

フォルダを渡します。はじめは `--readonly` を付ければ、書き込む心配なく見て回れます:

```sh
mdgrid ~/notes/Tasks --readonly
```

`.md` のファイルを1つ渡すと、そのフォルダを開いてそのノートの行を選びます。

### 3. 自分のコマンドにする

よく開くフォルダごとに、`~/.zshrc` か `~/.bashrc` に alias を書きます:

```sh
alias tasks='mdgrid ~/notes/Tasks'
alias books='mdgrid ~/notes/Books'
alias meetings='mdgrid ~/notes/Meetings --readonly'   # 見るだけ
```

これで `tasks` と打てばタスクの表が開きます。要らない列を隠し、`o` で絞り込みと並べ替えを決め、いくつかのビューをタブとして保存しておけば、次の `tasks` はその全部が当たった状態で開きます。(列の見出しや **並べ替え** のボタンでの並べ替えも覚えます。終わるまでだけなのは `\` の簡易の絞り込みです。)

出す側も、シェルの関数で同じように自分のコマンドにできます:

```sh
todo() { mdgrid ~/notes/Tasks --print --format md --filter 'status != "done"' --sort due; }
```

fish では `alias --save tasks 'mdgrid ~/notes/Tasks'` です。

## ⌨️ まず覚えるキー

| したいこと | キー |
|---|---|
| 動く | 矢印のキー、か `h` `j` `k` `l` |
| 選んだ値を直す | `Enter` |
| このセルでできることを見る | `x`(か右クリック) |
| 変えたところを見て保存する | `Ctrl+S` |
| 終わる | `q` |
| 検索する・打ちながら行を絞る | `/`・`\` |
| 列の値ごとの件数を見て、1つの値だけ残す | `%` |
| 列・絞り込み・並べ替え・まとまりを決める | `o` |
| ビュー(タブ)を切り替える | `[`・`]` |
| 新しいキーの列を足す | `A` |
| コマンドを名前で呼ぶ | `:`(たとえば `:rename_key`・`:delete_key`) |
| 全部のキーと、セルの印の意味を見る | `?` |

キーはどれも設定で変えられます。[キー](docs/keys.ja.md)を見てください。画面の言葉は設定の `language`(`auto`・`en`・`ja`)で選びます。既定の `auto` は、`LC_ALL`・`LC_MESSAGES`・`LANG` が `ja` で始まれば日本語です。

## 🔍 もう少し詳しく

ほかのノートが使っている値から選ぶか、新しい値を打ちます。打つと候補が絞られます:

![一覧から状態の値を選ぶ](docs/assets/ja/demo-edit-list.svg)

日付の列はカレンダーが開きます(日時の列は時刻の欄も):

![カレンダーで日付を選ぶ](docs/assets/ja/demo-calendar.svg)

保存するまで何も書かず、何が変わるかをそのまま見せます:

![1行の差分を見せる保存の確認](docs/assets/ja/demo-save.svg)

列で `%` を押すと値ごとの件数が出て、値を選ぶとその行だけが残ります:

![owner の列の値ごとの件数](docs/assets/ja/demo-freq.svg)

`x` で、選んだセルでできることが出ます:

![セルの操作の一覧](docs/assets/ja/demo-menu.svg)

表は、自分の仕事のやり方に合わせて組み替えられます。[見本 ai-human](examples/ai-human/README.md) は、`.base` の式1つと数行の設定で、タスクを人の手が要るもの(払う・署名する・提出する・送る)と AI エージェントに任せられるものに分け、人のまとまりを上に並べます:

![人と AI に分けたタスク](docs/assets/ja/demo-ai-human.svg)

`R` で関係マップ: ワークスペースの表を箱にし、リンクの列から行き先の表へ矢印を引きます(`N:1`、リンクのリストなら `N:N`)。表やつながりはクリックか矢印のキーで選べます:

![tasks・projects・members の関係マップ](docs/assets/ja/demo-relmap.svg)

色のテーマは設定の `theme` で選びます: `"nord"`(下の画面)・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"`。既定は端末の色のままで、`NO_COLOR` に従います。

![Nord のテーマの表](docs/assets/ja/demo-theme.svg)

テーマごとの設定の見本は [examples/themes/](examples/themes) にあります: `mdgrid --config examples/themes/nord.toml examples/demo`。

## 🧰 ほかのコマンドと使う

**表を出す。** `--print` は表を標準出力に出し、ノートは書き換えません。`--with-path` を付けると、先頭の列に各行のノートのパスが入ります。

```sh
mdgrid ~/notes/Tasks --print                       # 見出しの行つきの CSV
mdgrid ~/notes/Tasks --print --format json | jq .
mdgrid ~/notes/Tasks --print --format md > tasks.md
mdgrid ~/notes/Tasks --print --filter 'status != "done"' --sort due
```

**表計算で直して戻す。** `--with-path` を付けて出し、ファイルを直して `--apply` で戻します。`--yes` が無ければ差分を見せるだけです。

```sh
mdgrid ~/notes/Tasks --print --with-path > tasks.csv
mdgrid ~/notes/Tasks --apply tasks.csv          # 差分を見る(何も書かない)
mdgrid ~/notes/Tasks --apply tasks.csv --yes    # 変わったセルだけを書く
```

mdgrid は書く前に全部の行を確かめ、パスや値に1つでも問題があれば何も書きません。直していないセルは書き直しません。

**ノートを選んで渡す。** `--pick` は表を読むだけで開きます。行に印を付けて `Enter` を押すと、そのパスが1行ずつ出ます:

```sh
mdgrid ~/notes/Tasks --pick path | tr '\n' '\0' | xargs -0 -o vi
```

## 🪨 Obsidian を使っている人へ

mdgrid はどんな Markdown のフォルダでも使えますが、Obsidian の [Bases](https://help.obsidian.md/bases) も読めます。`.base` を開くと table ビューがタブに並び、絞り込み・並べ替え・まとまり・集計の行(`合計`・`平均`・`最早` など)と式の一部が使えます。Obsidian を入れていなくても動き、`.base` を書き換えることはありません。

```sh
mdgrid /tmp/mdgrid-sample/タスク.base                # ビュー: 進行中・状態ごと・期限・未対応の式・カード
alias standup='mdgrid ~/vault/タスク.base --view 状態ごと'
```

まとめたビューは見出しが付き、`Enter` で畳めます:

![状態でまとめた .base のビュー](docs/assets/ja/demo-group.svg)

コマンドのパレットの `:export_base` は今のビューを新しい `.base` に書き出し、`:import_base` は `.base` のビューを mdgrid のビューとして取り込みます。`.obsidian/types.json` のプロパティの型も読みます。どこを読み、評価し、無視するかは [Obsidian Bases への対応](docs/obsidian-bases.ja.md)にあります。

## 📚 文書

- [説明書](https://s6u5.github.io/mdgrid/manual/ja/)([元の文](docs/manual/ja/index.md))— はじめかた、やりたいことごとの手順、テーマ、全部の画面の写し。初めてなら、まず [mdgrid の考え方](docs/manual/ja/concepts.md) を。見た目は [見た目を整える](docs/manual/ja/appearance.md)、よくある疑問は [困ったとき](docs/manual/ja/faq.md) に。
- [設定](docs/config.ja.md) — 全部の項目と既定値。`mdgrid --print-config` は説明のコメント付きの設定を出すので、出発点にできます。
- [機能紹介の録画](demos/README.md) — 型ごとの入力・新しいノートのフォーム・書き出し・登録した表・リレーション・関係マップ・ワークスペース・テーマの短い GIF(画面は英語)。リリースのたびに撮り直します。`sh demos/try.sh <見本>` で、見本を使い捨ての写しで開けます。
- [キー](docs/keys.ja.md) — モードごとの既定のキーと、変え方。
- [書き戻しの安全](docs/safety.ja.md) — 何を書き、何に触らないか。読むだけになるノートとその理由。
- [Obsidian Bases への対応](docs/obsidian-bases.ja.md) — `.base` のどこを読み、評価し、無視するか。
- [見本の設定と保管庫](examples/showcase/README.md) — ほとんどの機能を有効にした見本。`examples/vault` は、null・空・型の合わない値、読むだけのノート、同期の競合ファイルなどの端の形をそろえています。
- [人と AI のタスクを分ける見本](examples/ai-human/README.md) — AI エージェントに任せきれるタスクと、人の手が要るタスク(払う・署名する・提出する・送る)をまとまりに分け、人のまとまりを上に並べる `.base` と設定。
- シェルの補完は `mdgrid --completions <bash|zsh|fish|elvish|powershell>`、man ページは `mdgrid --man`。

## 🤝 参加する

バグを見つけた、こうなってほしい、という案があれば [Issue を開いて](https://github.com/S6U5/mdgrid/issues/new/choose)ください(日本語で大丈夫です)。mdgrid は作者と AI のコーディングのエージェントで直しているので、何をして、何を期待し、何が起きたかが分かる Issue があれば、たいていそれだけで直したり機能を足したりできます。Pull request は共同作業者だけにしています。詳しくは [CONTRIBUTING.md](CONTRIBUTING.md) を見てください。セキュリティの問題は [SECURITY.md](SECURITY.md) のとおり非公開で知らせてください。

## 🏗️ 作り方

仕様の正本は [specs/](specs/README.md) にあり、変更はどれも提案と決定の記録を通ります。人が決めた要件を守る試験には錠が掛かっていて、黙って弱めることはできません。

## 📄 ライセンス

[Apache License, Version 2.0](LICENSE-APACHE) と [MIT license](LICENSE-MIT) のどちらかを選んで使えます。

mdgrid は独立のプロジェクトで、Obsidian とは関係がなく、Obsidian の承認を受けたものでもありません。
