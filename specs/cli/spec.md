---
type: Spec
id: 01M3SBGVX3YSG433EGT4BGQBCH
title: 起動と設定(cli)
description: 起動のしかた(引数・オプション)と、設定ファイルの置き場所と中身、起動できないときの振る舞い。
status: active
load_when: 起動の引数・オプション・設定ファイル・終了コードを作る・変えるとき
created: 2026-09-30
updated: 2026-10-08
---

# 起動と設定(cli)

mdgrid の起動のしかたと設定ファイル。設定が無くても既定で動くことを先にする(細かい設定を必須にしない)。見た目の状態(列の幅など)は設定ではなく、[screen](../screen/spec.md) の SR-11・SR-12 で別に持つ。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 起動と設定

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CLI-1 | `mdgrid [<.base のパス \\| フォルダ> …]` で起動し、引数が無ければ、登録した表(CLI-18)が無いときは今のフォルダを開き、あるときは CLI-19 の一覧を出すべき。 | 引数なしでノートのフォルダから起動 → そのフォルダの既定の表。places.toml に登録がある → 一覧(先頭は今のフォルダ)(`test_CLI_1`・`test_cli_19_*`) | [2026-10-08](../_decisions/2026-10-08-places.md)、[2026-09-30](../_decisions/2026-09-30-rename-mdgrid.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-2 | `--view <名前>`・`--readonly`・`--no-color`・`--config <パス>`・`--print-config`(CLI-11)・`-s` / `--session`(CLI-6)・`--sessions`(CLI-9)・`--help`・`--version` を受け付けるべき。 | `--view 進行中` → そのビューで開く(`test_CLI_2`) | [2026-10-03](../_decisions/2026-10-03-stale-text.md)、[2026-10-02](../_decisions/2026-10-02-cli2-print-config.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-3 | 設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`)に1つ置き、キーの割り当て・色・候補の数(CE-3)・読み直しの間隔(BV-9)・East Asian Ambiguous の幅(CV-6)など(全項目は CLI-12 の文書)を持つべき。設定が無くても既定で動くべきで、知らない設定の項目は警告にとどめるべき。 | 設定の無い環境で起動 → 既定で動く。知らない項目がある設定 → 起動して警告(`test_CLI_3`) | [2026-10-03](../_decisions/2026-10-03-stale-text.md)、[2026-09-30](../_decisions/2026-09-30-rename-mdgrid.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-4 | 起動できないとき(パスが無い・`.base` が読めない)は、理由を1行で出し、0 以外の終了コードで終わるべき。 | 無いパスを渡す → 理由1行と終了コード 2(`test_CLI_4`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-11 | `mdgrid --print-config` は、設定の全項目を既定値と説明のコメント付きの TOML として標準出力に出し、終了コード 0 で終わるべき。出したものは、そのまま設定ファイルとして警告なしで読めて、既定と同じ振る舞いになるべき(dotfiles に貼る出発点にするため)。 | `mdgrid --print-config > c.toml` → `mdgrid --config c.toml` で警告なしに起動し、既定と同じ。出力に全項目の名前とコメントがある(`test_CLI_11`) | [2026-10-02](../_decisions/2026-10-02-config-docs.md) |
| CLI-12 | 設定の全項目を、名前・型・既定値・説明で並べた文書を、英語(docs/config.md)と日本語(docs/config.ja.md)で置くべきで、文書の項目と、実装が読む項目と、`--print-config` の項目が食い違えば試験で落ちるべき(文書が古くなるのを防ぐため)。 | 実装に項目を足して文書に足さない → 試験が落ちる(`test_CLI_12`) | [2026-10-02](../_decisions/2026-10-02-config-docs.md) |
| CLI-13 | `mdgrid --completions <シェル>`(bash・zsh・fish・elvish・powershell)はそのシェルの補完の定義を、`mdgrid --man` は man ページ(roff)を、標準出力に出して終了コード 0 で終わってよい。知らないシェルの名前は CLI-4 と同じく理由1行と終了コード 2 にするべき。補完と man は、受け付けるオプション(CLI-2)と食い違わないよう、引数の定義から作るべき。 | `--completions zsh` → `#compdef mdgrid` で始まり `--readonly` を含む。`--completions nushell` → 理由1行と終了コード 2。`--man` → `.TH` を含み `--print-config` を含む(`test_CLI_13`) | [2026-10-03](../_decisions/2026-10-03-completions-man.md) |
| CLI-14 | `--print` に `--with-path` を付けたときは、表の先頭に見出し `path` の列を足し、各行にそのノートのパス(OUT-3 の `--pick path` と同じ形: 起動の引数のフォルダにノートの相対のパスをつないだもの)を出すべき。json では各オブジェクトの最初の鍵 `path` にし、ビューに同じ見出しの列があれば CLI-5 の重なりの決まりで見分けるべき。`--print` の無い `--with-path` は CLI-4 と同じく理由1行と終了コード 2 にするべき。 | `mdgrid examples/demo/Tasks.base --print --with-path` → 見出しの行が `path,status,…`、各行の先頭が `examples/demo/Tasks/Fix login redirect bug.md` のようなパス。`--format json` → 各オブジェクトの最初が `"path"`。フロントマターの無いノートの行もパスで見分けられる。`--with-path` だけ → 理由1行と終了コード 2(`test_CLI_14`) | [2026-10-06](../_decisions/2026-10-06-print-with-path.md) |
| CLI-15 | 起動の引数に `.md` のファイルを渡したときは、ファイル名を補完で渡す人が「フォルダでない」で止められないよう、そのファイルのあるフォルダを渡したのと同じに開き、画面ではそのノートの行を選んで始めるべき。ファイルが無いときは CLI-4 と同じく理由1行と終了コード 2 にするべき。`--print` に `.md` のファイルを渡したときは、そのフォルダの表のうち、渡したノートの行だけを出すべき(1つのノートの値を取り出すため)。 | `mdgrid notes/b.md` → notes の既定の表が開き、b.md の行が選ばれている。`mdgrid notes/b.md --print` → notes の表と同じ列で、b の1行だけ。`notes/a.md notes/b.md --print` → a と b の2行。`mdgrid notes/無い.md` → 理由1行と終了コード 2(`test_CLI_15`) | [2026-10-06](../_decisions/2026-10-06-print-md-rows.md)、[2026-10-06](../_decisions/2026-10-06-open-md-file.md) |
| WB-15 | 読むだけの起動(`--readonly`)では、保存の操作を出さず、どのファイルも書くべきでない。 | `--readonly` で起動して Ctrl+S → 何も書かれず、読むだけと表示(`test_WB_15`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-5 | `mdgrid [<.base \| フォルダ> …] --print [--format csv\|tsv\|json\|md] [--view <名前>]` は、画面を出さずに、開いたビュー(`.base` のビューか mdgrid のビュー。BV-17)の表を標準出力に出して終了コード 0 で終わるべき。行はビューの絞り込みと並べ替えのあと(グループ分けは並びだけに効かせ、見出しの行は出さない)、列はビューの列の並びとし、画面の見た目の状態(SR-12)と画面で掛けた絞り込みは使わないべき。既定の形は csv(RFC 4180、見出しの行つき、改行は LF)、tsv はタブ区切り(見出しの行つき、セルの中のタブと改行は空白)、json は列の名前を鍵にしたオブジェクトの配列(数・真偽・null・リストは JSON の型、日付と日時は文字列)、md は見出しと区切りの行のある表(セルの中の縦棒はバックスラッシュで逃がし、改行は空白)とするべき。評価できない列(BV-7)は csv と md で空、json で null にし、標準エラーに1行で知らせるべき。ノートを書き換えないべき(WB-15 と同じ)。起動できないとき・知らない形の名前は CLI-4 と同じく理由1行と終了コード 2 にするべき。 | `--print --format csv` → 見出しの行と行ごとの CSV が出て終わる。`--format json` → `[{"title": "会議", "priority": 2, "tags": ["a"]}]` の形。`--format md` → 見出しの行と区切りの行のある表。`--format tsv` → タブ区切り。`--format xml` → 理由1行と終了コード 2。標準出力がパイプでも動く(`test_CLI_5`) | [2026-10-07](../_decisions/2026-10-07-export-table.md)、[2026-10-03](../_decisions/2026-10-03-print-pick.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 版1 — セッション

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CLI-6 | 表の表示を名前を付けたセッションとして保存し、`mdgrid -s <名前>`(`--session <名前>`)で、その表示のまま起動できるべき。 | フォルダを開き、絞り込み・並び・列を決めて名前 `進行中` で保存して終了 → `mdgrid -s 進行中` で同じフォルダ・同じ絞り込み・並び・列の表が開く。無い名前 `mdgrid -s 無い` → 理由1行と終了コード 2(`test_CLI_6`) | [2026-10-01](../_decisions/2026-10-01-sessions.md) |
| CLI-7 | セッションは、開くフォルダ(1つ以上)、`.base` のパスとビューの名前(任意)、画面で掛けた絞り込み(簡易の絞り込み NV-2・同じ値の絞り込み NV-8。次の段の式の絞り込み NV-10 は入ったら同じく)、並び(一時的な並べ替え NV-3 も、保存したときのものを持つ)、列の並びと隠す列、読むだけで開くか(`--readonly`)を持つべき。セッションで開いたときは、列の並びと隠す列を SR-12 の見た目の状態より先に当て、表の行と値は、値をセッションに写さず、起動のたびに今のノートから作り直すべき。 | `.base` の無いフォルダで status のセルが「進行中」の行だけに絞り(NV-8)、`\` で「会議」と打って絞り(NV-2)、`due` の昇順、`title`・`status`・`due` の3列にして保存 → 起動し直すと同じ3列・同じ並び・同じ2つの絞り込みで、あとから直したノートの値が出る。セッションにあって今の表に無い列 → その列だけ飛ばして警告(`test_CLI_7`) | [2026-10-01](../_decisions/2026-10-01-sessions-filters.md)、[2026-10-01](../_decisions/2026-10-01-sessions-details.md) |
| CLI-8 | セッションは、ノートのフォルダや `.base` に書かず(SR-11・BV-3・WB-2 と同じ理由)、設定の置き場(CLI-3)の `sessions.toml` に名前ごとに置くべきで、人が手で書いてもよく、知らない項目は警告にとどめるべき。 | 保存したあと、ノートのフォルダに新しいファイルが無く、`~/.config/mdgrid/sessions.toml` に `[sessions.進行中]` がある。手で `color = "red"` を足す → 起動して警告(`test_CLI_8`) | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) |
| CLI-9 | 画面から、今の表示を名前を付けて保存でき、同じ名前があるときは上書きしてよいか確かめるべき。`mdgrid --sessions` は、保存したセッションの名前と開くフォルダを1行ずつ出して終わるべき。 | 同じ名前で2回保存 → 2回目に上書きの確認が出て、断ると元のまま。`--sessions` → `進行中	~/notes/tasks` の形の行が並び、終了コード 0。セッションが1つも無い → 何も出さず終了コード 0(`test_CLI_9`) | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) |
| CLI-10 | 無い名前・壊れた `sessions.toml`・セッションの開くフォルダが無いときは、CLI-4 と同じく理由を1行で出して終了コード 2 で終わり、どちらを開くか曖昧になる `-s` とパスの引数を一緒に渡されたときも同じにするべき。 | `mdgrid -s 無い` → `セッション「無い」はありません(ある名前: 進行中)` の1行と終了コード 2。`mdgrid -s 進行中 ~/notes` → 理由1行と終了コード 2(`test_CLI_10`) | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) |
| CLI-16 | `--print` には、`--filter <式>`(BV-6 の式。何度でも渡せ、全部を満たす行だけ)と `--sort <列>[:asc\|:desc]`(何度でも渡せ、渡した順に効く。既定は昇順)を添えてよい。`.base` のビューに添えたときは、絞り込みはビューの絞り込みと両方を満たす行に、並べ替えはビューの並べ替えの代わりにするべき。フォルダには、既定の表(BV-1)と同じ列のまま、絞り込みと並べ替えを当てるべき。評価できない式は、評価できないビュー(BV-7)と同じく理由1行と終了コード 2 にし、mdgrid のビュー(BV-17)に添えたときも理由1行と終了コード 2 にするべき。 | `mdgrid notes --print --filter 'status != "done"' --sort due` → done でない行が期日の順。`--sort priority:desc` → 優先度の高い順。`.base` の絞り込みと両方。`--filter 'foo('` → 理由1行と終了コード 2(`test_cli_16_*`) | [2026-10-06](../_decisions/2026-10-06-print-filter-sort.md) |
| CLI-17 | `mdgrid [<.base \| フォルダ> …] --apply <ファイル>`(`-` で標準入力)は、`--print --with-path` の形の CSV か JSON(`path` の列とノートのキーの列。見出しは列の id か表示名)を読み、各行の `path` のノートの、値が今と違うセルだけを変えるべき。既定では書かずに差分(WB-9 と同じ形)を標準出力に出し、`--yes` を添えたときだけ画面の保存と同じ道(WB-1〜WB-8)で書くべき(書く前に差分を見られるようにするため)。知らない `path`・書けないセル(CE-8・WB-5)・列の型に合わない値が1つでもあれば、どれも書かずに全部の理由を標準エラーに出して終了コード 2 にするべき。`path`・`file.*`・`formula.*` の列は読まないべき。CSV の空のセルは値を消す(CE-9)が、キーの無いノートではキーを足さないべきで、リストの列の CSV のセルは `, ` で要素に分けるべき。 | `--print --with-path --format json` の status を done に直して `--apply` → 差分が出てファイルは変わらない。`--yes` → そのセルだけが書かれ、ほかのバイトは同じ。無い path → どれも書かず終了コード 2(`test_cli_17_*`) | [2026-10-06](../_decisions/2026-10-06-apply.md) |
| CLI-18 | よく使う表(フォルダか `.base`。`.base` と mdgrid のビューは名前でビューも)を、名前と1段の分類を付けて、設定のフォルダの `places.toml`(`[[place]]` の並び。`name`・`group`・`path`・`view`)に登録できるべき。コマンドのパレットの「この表を登録」で、今開いている表と今のビューを、名前(既定は今のビューかフォルダの名前)と分類(今ある分類から選ぶか打つ)を打って登録でき、同じ名前が既にあれば確かめてから置き換えるべき。`places.toml` は手で書いてもよく、`path` の先頭の `~` はホームのフォルダとし、読めない行は警告にして飛ばすべき。ノートは書き換えないべき(読むだけの起動でも登録できてよい)。 | パレットの「この表を登録」→ 名前「タスク」・分類「仕事」→ places.toml に `name = "タスク"`・`group = "仕事"`・`path`(実際のパス)。同じ名前でもう一度 → 置き換えの確かめ。`path` の無い行 → 警告して飛ばす(`test_cli_18_*`) | [2026-10-08](../_decisions/2026-10-08-places.md) |
| CLI-19 | 引数なしで起動して登録があるときと、表を開いたあとのパレットの「登録した表を開く」では、登録した表を分類ごとに(places.toml の順で)並べた一覧を出し、先頭に「今のフォルダ」(引数なしの起動のとき)を置くべき。打った文字で名前・分類・パスを大文字小文字を区別せずに絞り、Enter で選んだ表を(ビューがあればそのビューで)開くべき。表を開いたあとに別の表へ移るときは、ためた変更があれば終わるとき(WB-11)と同じく保存か取りやめを確かめるべき。無いパスの行は、理由を出して開かないべき。 | places.toml に 仕事(タスク・朝会)と 趣味(読書)→ 引数なしの起動で `今のフォルダ`・`仕事`・`タスク`・`朝会`・`趣味`・`読書` の一覧。「読」と打つ → 読書だけ。Enter → ~/notes/Books の表。朝会 → Tasks.base の By owner のビュー。表で未保存の変更があるときに「登録した表を開く」→ 保存の確かめ(`test_cli_19_*`) | [2026-10-08](../_decisions/2026-10-08-places.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|

## 範囲外

- 採らない: 設定ファイルでの任意のコードの実行(VisiData の Python の設定の型。安全面で不利)。

## 前提(AI の仮定)

- 設定の書式を TOML にしたのは、Rust の TUI の多く(xleak・tabiew)と揃えるため(AI の判断)。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-08 | [よく使う表を名前と分類で登録し、一覧から開く(places)](../_decisions/2026-10-08-places.md) | accepted | 人 | 人の発言 | CLI-1, CLI-18, CLI-19 |
| 2026-10-07 | [左のノートの欄の名前に合わせて、画面の名前を見る試験の錠を掛け直す(note-column-locks)](../_decisions/2026-10-07-note-column-locks.md) | accepted | 人 | 人の発言 | SR-21, BV-17, CE-16 ほか 9 |
| 2026-10-07 | [group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)](../_decisions/2026-10-07-group-gap-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-21, NV-16 ほか 4 |
| 2026-10-07 | [画面の表をファイルに書き出し、--print に tsv を足す(export-table)](../_decisions/2026-10-07-export-table.md) | accepted | 人 | 人の発言 | OUT-2, CLI-5 |
| 2026-10-06 | [--print に各行のノートのパスの列を足す --with-path(print-with-path)](../_decisions/2026-10-06-print-with-path.md) | accepted | AI | 検証の指摘 | CLI-14 |
| 2026-10-06 | [.md を渡した --print は、渡したノートの行だけを出す(print-md-rows)](../_decisions/2026-10-06-print-md-rows.md) | accepted | 人 | 人の発言 | CLI-15 |
| 2026-10-06 | [--print に --filter と --sort を足す(print-filter-sort)](../_decisions/2026-10-06-print-filter-sort.md) | accepted | AI | 検証の指摘 | CLI-16 |
| 2026-10-06 | [.md のファイルを渡したらそのフォルダを開いてその行を選ぶ(open-md-file)](../_decisions/2026-10-06-open-md-file.md) | accepted | AI | 検証の指摘 | CLI-15 |
| 2026-10-06 | [--print で出した CSV・JSON を直して戻す --apply(apply)](../_decisions/2026-10-06-apply.md) | accepted | AI | 検証の指摘 | CLI-17 |
| 2026-10-03 | [仕様の古い記述を今の決定と実装に合わせる(stale-text)](../_decisions/2026-10-03-stale-text.md) | accepted | AI | 検証の指摘 | SR-1, SR-16, CLI-2 ほか 4 |
| 2026-10-03 | [--print と --pick を版1に入れる(print-pick)](../_decisions/2026-10-03-print-pick.md) | accepted | AI | AI の提案 | CLI-5, OUT-3, SR-10 |
| 2026-10-03 | [シェルの補完と man ページを出す(completions-man)](../_decisions/2026-10-03-completions-man.md) | accepted | AI | AI の提案 | CLI-13 |
| 2026-10-02 | [設定の全項目の文書と、既定の設定の書き出し(config-docs)](../_decisions/2026-10-02-config-docs.md) | accepted | 人 | 人の発言 | CLI-11, CLI-12 |
| 2026-10-02 | [CLI-2 の起動の引数に --print-config を並べる(cli2-print-config)](../_decisions/2026-10-02-cli2-print-config.md) | accepted | AI | 検証の指摘 | CLI-2 |
| 2026-10-01 | [表示をセッションとして保存し、名前で起動する(sessions)](../_decisions/2026-10-01-sessions.md) | accepted | 人 | 人の発言 | CLI-6 |
| 2026-10-01 | [セッションが持つ絞り込みを、版1の画面の絞り込みに合わせる(sessions-filters)](../_decisions/2026-10-01-sessions-filters.md) | accepted | AI | 検証の指摘 | CLI-7 |
| 2026-10-01 | [セッションの中身・置き場・一覧・失敗のとき(sessions-details)](../_decisions/2026-10-01-sessions-details.md) | accepted | AI | AI の提案 | CLI-7, CLI-8, CLI-9, CLI-10 |
| 2026-09-30 | [仕様の点検の指摘を直す(v1-review-fixes)](../_decisions/2026-09-30-v1-review-fixes.md) | accepted | AI | 検証の指摘 | SR-13, SR-10, SR-12 ほか 18 |
| 2026-09-30 | [最初の版の仕様を、表の TUI の調査に合わせて書き切る(v1-full-spec)](../_decisions/2026-09-30-v1-full-spec.md) | accepted | AI | 人の発言 | BV-4, BV-9, CE-1 ほか 52 |
| 2026-09-30 | [TUI の名前を mdgrid にする(rename-mdgrid)](../_decisions/2026-09-30-rename-mdgrid.md) | accepted | AI | 人の発言 | OUT-3, CLI-1, CLI-3 |
<!-- decidespec:history:end -->
