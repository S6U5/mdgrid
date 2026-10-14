---
type: Spec
id: 01M3SBGVX3YSG433EGT4BGQBCH
title: 起動と設定(cli)
description: 起動のしかた(引数・オプション)と、設定ファイルの置き場所と中身、起動できないときの振る舞い。
status: active
load_when: 起動の引数・オプション・設定ファイル・終了コードを作る・変えるとき
created: 2026-09-30
updated: 2026-10-15
---

# 起動と設定(cli)

mdgrid の起動のしかたと設定ファイル。設定が無くても既定で動くことを先にする(細かい設定を必須にしない)。見た目の状態(列の幅など)は設定ではなく、[screen](../screen/spec.md) の SR-11・SR-12 で別に持つ。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 起動と設定

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CLI-1 | `mdgrid [<.base のパス \\| フォルダ> …]` で起動し、引数が無ければ、登録した表(CLI-18)が無いときは今のフォルダを開き、あるときは CLI-19 の一覧を出すべき。 | 引数なしでノートのフォルダから起動 → そのフォルダの既定の表。places.toml に登録がある → 一覧(先頭は今のフォルダ)(`test_CLI_1`・`test_cli_19_*`) | [2026-10-08](../_decisions/2026-10-08-places.md)、[2026-09-30](../_decisions/2026-09-30-rename-mdgrid.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-2 | `--view <名前>`・`--readonly`・`--no-color`・`--config <パス>`・`--print-config`(CLI-11)・`-s` / `--session`(CLI-6)・`--sessions`(CLI-9)・`--help`・`--version` を受け付けるべき。 | `--view 進行中` → そのビューで開く(`test_CLI_2`) | [2026-10-03](../_decisions/2026-10-03-stale-text.md)、[2026-10-02](../_decisions/2026-10-02-cli2-print-config.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-3 | 設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`)に1つ置くべき。項目は、アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、端末の性質の `[terminal]`(`color`・`ambiguous_wide`・`nerd_font`)、`[workspace]`、`[keys]`)と、表の見え方と振る舞いを決める表のプロファイル(`[look]`(テーマ・組・セルの部品の切り替え・部品の形 `[look.style]`・列ごとの見せ方 `[look.columns]`・色 `[look.colors]`)・`[display]`・`[dates]`・`[edit]`(候補の数(CE-3)・フロントマターを足すか(WB-3))・`[new_note]` と、土台にするテンプレートの名前 `use`)に分け、名前を付けたプロファイルの断片(テンプレート)を `[templates.<名前>]` に書けるべき(全項目は CLI-12 の文書)。画面で選んだ設定(SR-43)は、同じ置き場の `ui.toml` に config.toml と同じ形で置いて config.toml の上に重ね、config.toml は書き換えないべき。設定が無くても既定で動くべきで、知らない項目・型の違う値・その場所に書けない項目は警告にとどめるべき。警告は、どの設定のファイル(config.toml・ui.toml・views.toml・workspaces.toml・ワークスペースの印・places.toml・sessions.toml)でも「ファイルの名前: 項目の道筋(`look.style.status` など): 理由」の形にそろえるべき。 | 設定の無い環境で起動 → 既定で動く。知らない項目がある設定 → 起動して警告。`[look.style] status = "neon"` → `config.toml: look.style.status:` で始まる警告。views.toml の知らない項目 → `views.toml: table.view.foo:` で始まる警告。ui.toml に `[look] theme = "nord"`、config.toml に `[look] theme = "sumi"` → nord(`test_CLI_3`・`test_cli_3_*`) | [2026-10-10](../_decisions/2026-10-10-config-v2.md)、[2026-10-03](../_decisions/2026-10-03-stale-text.md)、[2026-09-30](../_decisions/2026-09-30-rename-mdgrid.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-4 | 起動できないとき(パスが無い・`.base` が読めない)は、理由を1行で出し、0 以外の終了コードで終わるべき。 | 無いパスを渡す → 理由1行と終了コード 2(`test_CLI_4`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-11 | `mdgrid --print-config` は、設定の全項目を、区画(アプリ全体の最上位・`[terminal]`・`[workspace]`・`[look]`・`[look.style]`・`[look.colors]`・`[display]`・`[dates]`・`[edit]`・`[new_note]`・`[keys]`・`[templates]`)ごとに、既定値と説明のコメント付きの TOML として標準出力に出し、終了コード 0 で終わるべき。各項目の説明には、書ける範囲(全体だけか、ワークスペース・表・ビューでも書けるか)を添えるべき。出したものは、そのまま設定ファイルとして警告なしで読めて、既定と同じ振る舞いになるべき(dotfiles に貼る出発点にするため)。 | `mdgrid --print-config > c.toml` → `mdgrid --config c.toml` で警告なしに起動し、既定と同じ。出力に全項目の名前(区画の見出しの下)とコメントと書ける範囲があり、旧い名前(`theme_light`・`borders`・`view_tabs` など)が無い(`test_CLI_11`・`test_cli_11_*`) | [2026-10-10](../_decisions/2026-10-10-config-v2.md)、[2026-10-02](../_decisions/2026-10-02-config-docs.md) |
| CLI-12 | 設定の全項目を、区画ごとに、項目の道筋(`look.theme` など)・型・既定値・書ける範囲・説明で並べた文書を、英語(docs/config.md)と日本語(docs/config.ja.md)で置き、旧い名前(CLI-20)と移った先の対応の表も置くべきで、文書の項目と、実装が読む項目と、`--print-config` の項目と、旧い名前の対応が食い違えば試験で落ちるべき(文書が古くなるのを防ぐため)。 | 実装に項目を足して文書に足さない → 試験が落ちる。旧い名前の対応を実装に足して文書の表に足さない → 試験が落ちる(`test_CLI_12`・`test_cli_12_*`) | [2026-10-10](../_decisions/2026-10-10-config-v2.md)、[2026-10-02](../_decisions/2026-10-02-config-docs.md) |
| CLI-13 | `mdgrid --completions <シェル>`(bash・zsh・fish・elvish・powershell)はそのシェルの補完の定義を、`mdgrid --man` は man ページ(roff)を、標準出力に出して終了コード 0 で終わってよい。知らないシェルの名前は CLI-4 と同じく理由1行と終了コード 2 にするべき。補完と man は、受け付けるオプション(CLI-2)と食い違わないよう、引数の定義から作るべき。 | `--completions zsh` → `#compdef mdgrid` で始まり `--readonly` を含む。`--completions nushell` → 理由1行と終了コード 2。`--man` → `.TH` を含み `--print-config` を含む(`test_CLI_13`) | [2026-10-03](../_decisions/2026-10-03-completions-man.md) |
| CLI-14 | `--print` に `--with-path` を付けたときは、表の先頭に見出し `path` の列を足し、各行にそのノートのパス(OUT-3 の `--pick path` と同じ形: 起動の引数のフォルダにノートの相対のパスをつないだもの)を出すべき。json では各オブジェクトの最初の鍵 `path` にし、ビューに同じ見出しの列があれば CLI-5 の重なりの決まりで見分けるべき。`--print` の無い `--with-path` は CLI-4 と同じく理由1行と終了コード 2 にするべき。 | `mdgrid examples/demo/Tasks.base --print --with-path` → 見出しの行が `path,status,…`、各行の先頭が `examples/demo/Tasks/Fix login redirect bug.md` のようなパス。`--format json` → 各オブジェクトの最初が `"path"`。フロントマターの無いノートの行もパスで見分けられる。`--with-path` だけ → 理由1行と終了コード 2(`test_CLI_14`) | [2026-10-06](../_decisions/2026-10-06-print-with-path.md) |
| CLI-15 | 起動の引数に `.md` のファイルを渡したときは、ファイル名を補完で渡す人が「フォルダでない」で止められないよう、そのファイルのあるフォルダを渡したのと同じに開き、画面ではそのノートの行を選んで始めるべき。ファイルが無いときは CLI-4 と同じく理由1行と終了コード 2 にするべき。`--print` に `.md` のファイルを渡したときは、そのフォルダの表のうち、渡したノートの行だけを出すべき(1つのノートの値を取り出すため)。 | `mdgrid notes/b.md` → notes の既定の表が開き、b.md の行が選ばれている。`mdgrid notes/b.md --print` → notes の表と同じ列で、b の1行だけ。`notes/a.md notes/b.md --print` → a と b の2行。`mdgrid notes/無い.md` → 理由1行と終了コード 2(`test_CLI_15`) | [2026-10-06](../_decisions/2026-10-06-print-md-rows.md)、[2026-10-06](../_decisions/2026-10-06-open-md-file.md) |
| WB-15 | 読むだけの起動(`--readonly`)では、保存の操作を出さず、どのファイルも書くべきでない。 | `--readonly` で起動して Ctrl+S → 何も書かれず、読むだけと表示(`test_WB_15`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-5 | `mdgrid [<.base \| フォルダ> …] --print [--format csv\|tsv\|json\|md] [--view <名前>]` は、画面を出さずに、開いたビュー(`.base` のビューか mdgrid のビュー。BV-17)の表を標準出力に出して終了コード 0 で終わるべき。行はビューの絞り込みと並べ替えのあと(グループ分けは並びだけに効かせ、見出しの行は出さない)、列はビューの列の並びとし、画面の見た目の状態(SR-12)と画面で掛けた絞り込みは使わないべき。既定の形は csv(RFC 4180、見出しの行つき、改行は LF)、tsv はタブ区切り(見出しの行つき、セルの中のタブと改行は空白)、json は列の名前を鍵にしたオブジェクトの配列(数・真偽・null・リストは JSON の型、日付と日時は文字列)、md は見出しと区切りの行のある表(セルの中の縦棒はバックスラッシュで逃がし、改行は空白)とするべき。評価できない列(BV-7)は csv と md で空、json で null にし、標準エラーに1行で知らせるべき。ノートを書き換えないべき(WB-15 と同じ)。起動できないとき・知らない形の名前は CLI-4 と同じく理由1行と終了コード 2 にするべき。 | `--print --format csv` → 見出しの行と行ごとの CSV が出て終わる。`--format json` → `[{"title": "会議", "priority": 2, "tags": ["a"]}]` の形。`--format md` → 見出しの行と区切りの行のある表。`--format tsv` → タブ区切り。`--format xml` → 理由1行と終了コード 2。標準出力がパイプでも動く(`test_CLI_5`) | [2026-10-07](../_decisions/2026-10-07-export-table.md)、[2026-10-03](../_decisions/2026-10-03-print-pick.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| CLI-20 | 0.3.0 までの書き方は、読んで新しい場所に当て、項目ごとに「旧い名前は新しい道筋に移った」と警告するべき: `color`・`ambiguous_wide`・`nerd_font` → `[terminal]`、`workspace_detect` → `workspace.detect`、`theme`・`theme_light`・`theme_dark` → `look.theme`(`"auto"` に明暗のどちらかを書いていれば明暗の表)、`look` → `look.mode`、`borders` → `look.style.frames`、`[style]` → `[look.style]`(`preset` は `look.preset`)、`cells` の文字列と `[cells] style` → `look.cells`、`[cells]` の部品ごとの真偽 → `[look.style]` のその部品の `plain`(真偽は `check = "text"`、リンクは `links = "plain"`、印は `icons`)、`[cells.columns]` → `[look.columns]`、`[colors]` → `[look.colors]`、`view_tabs` → `display.tabs`、`[display] tabs = false` → `display.tabs = "never"`、`search_bar` → `display.search_bar`、`date_format`・`week_start` → `dates.format`・`dates.week_start`、`candidates`・`add_frontmatter` → `[edit]`。旧い名前と新しい道筋の両方に書いてあれば、新しい道筋を使うべき。画面が書くファイルの旧い形(`look.toml`、views.toml の `[[target]]`)も読み、次に書くときに新しい形(ui.toml、`[[table]]`)で書き、`look.toml` は ui.toml に移したあと消してよい。`mdgrid --migrate-config` は、設定ファイル(`--config` があればそのファイル)を新しい形にした TOML を標準出力に出して終了コード 0 で終わり、ファイルは書かないべき。注釈は移らないことを、出す文の先頭のコメントで知らせるべき。 | `theme = "auto"` と `theme_light = "paper"` → 明るい地で paper、警告に `theme_light` と `look.theme`。`borders = "ascii"` → 枠が ASCII で、警告に `look.style.frames`。`view_tabs = "auto"` と `[display] tabs = "always"` → always。`mdgrid --migrate-config` → `[look]` の下に `theme = { light = "paper", dark = "sumi" }`、出した文を読み直すと警告なしで同じ振る舞い。look.toml だけがある → その見た目で起動し、見た目の区画で反映すると ui.toml ができて look.toml が無い(`test_cli_20_*`) | [2026-10-10](../_decisions/2026-10-10-config-v2.md) |
| CLI-21 | `mdgrid [<表>] --print-config --resolved` は、その表(引数が無ければ今のフォルダ。`--view` があればそのビュー)を開いたときに効く表のプロファイルとアプリ全体の項目の全部を、CLI-11 と同じ区画の TOML で標準出力に出し、各項目の上に、その値がどこから来たか(既定・config.toml・ui.toml・ワークスペースの印か workspaces.toml とその名前・views.toml の表・ビューの名前・テンプレートの名前)をコメントで添えて、終了コード 0 で終わるべき。出したものは設定ファイルとして警告なしに読めるべき。画面を開かず、ファイルは書かないべき。 | SR-44 の例で `mdgrid notes/tasks --print-config --resolved` → `theme = "dracula"` の上に `# views.toml (notes/tasks)`、`preset` の上に `# default`。出した文を `--config` で読む → 警告なし(`test_cli_21_*`) | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md) |
| CLI-22 | リポは、AI エージェント(Claude Code・Codex)が mdgrid を CLI で使うための手引きのスキル(`SKILL.md`)を、プラグインとして入れられる形(Claude Code と Codex のマーケットプレイスの索引とマニフェスト)で置くべき。スキルは、頼まれた条件を `--print` の `--filter`・`--sort`・`--view`・`--format`・`--with-path` に組み立てて表をファイルに出す手順と、`--apply` で書き戻すときは差分を人に見せてから `--yes` で書く手順を書く。スキルに書いた例のコマンドは、そのまま動かして通るべき(古い旗や書けない式を AI に教えないため)。マニフェストの版は Cargo.toml の版と揃えるべき。 | SKILL.md の例のコマンドを見本のフォルダ(status が todo・doing・done のノート)で動かす → 全部が終了コード 0 で、todo と doing の絞り込みの例は2行だけを出す(`test_cli_22_skill_examples_run`)。マニフェストの版が Cargo.toml の版と違う → 落ちる(`test_cli_22_plugin_versions_match_crate`) | [2026-10-14](../_decisions/2026-10-14-agent-skill.md) |
| CLI-23 | `mdgrid [<.base \| フォルダ> …] --check [--view <名前>]` は、画面を出さずに、開いたビュー(CLI-5 と同じく、ビューの絞り込みのあとの行。画面の見た目の状態は使わない)の各ノートの全部のフロントマターのキーと、決まり(CE-36)のある列を確かめ、問題(CV-9)を1つ1行で `<パス>:<行>: <列>: <理由>`(キーが無いときは行を付けず `<パス>: <列>: <理由>`)の形でパスと行の順に標準出力に出し、問題の数を標準エラーに1行で出すべき。列の型は、画面と同じく CE-2 の順(設定・ルートフォルダの型の設定・読み込んだ全部のノート(BV-2 の範囲)からの推定)で決めるべき(画面と `--check` で同じ値の判定が食い違わないため)。起動の引数に `.md` のファイルを渡したときは、`--print` と同じく(CLI-15)そのノートの行だけを確かめるべき。問題が無ければ終了コード 0、あれば 1、起動できないときは CLI-4 と同じく理由1行と 2 にするべき(CI と pre-commit で止められるようにするため)。パスは `--pick path`(OUT-3)と同じ形にし、ノートを書き換えないべき。 | `[types] due = "date"` で `due: someday` のノートと、`[rules.status] required = true` で status の無いノート → `notes/a.md:3: due: 型が合わない(日付ではない)` と `notes/b.md: status: 必須のキーが無い`、標準エラーに「問題 2」、終了コード 1。直したあと → 出力なし、終了コード 0。無いパス → 理由1行と終了コード 2。ビューの絞り込みで日付の値の行だけが残っても、ほかの行の値で決まる列の型は画面と同じ。`notes/a.md --check` → a.md の問題だけ(`test_cli_23_*`) | [2026-10-15](../_decisions/2026-10-15-check-id.md)、[2026-10-14](../_decisions/2026-10-14-check-kinds.md)、[2026-10-14](../_decisions/2026-10-14-type-check.md) |

## 要件: 版1 — セッション

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| CLI-6 | 表の表示を名前を付けたセッションとして保存し、`mdgrid -s <名前>`(`--session <名前>`)で、その表示のまま起動できるべき。 | フォルダを開き、絞り込み・並び・列を決めて名前 `進行中` で保存して終了 → `mdgrid -s 進行中` で同じフォルダ・同じ絞り込み・並び・列の表が開く。無い名前 `mdgrid -s 無い` → 理由1行と終了コード 2(`test_CLI_6`) | [2026-10-01](../_decisions/2026-10-01-sessions.md) |
| CLI-7 | セッションは、開くフォルダ(1つ以上)、`.base` のパスとビューの名前(任意)、画面で掛けた絞り込み(検索 NV-2・同じ値の絞り込み NV-8。次の段の式の絞り込み NV-10 は入ったら同じく)、並び(一時的な並べ替え NV-3 も、保存したときのものを持つ)、列の並びと隠す列、読むだけで開くか(`--readonly`)を持つべき。セッションで開いたときは、列の並びと隠す列を SR-12 の見た目の状態より先に当て、表の行と値は、値をセッションに写さず、起動のたびに今のノートから作り直すべき。 | `.base` の無いフォルダで status のセルが「進行中」の行だけに絞り(NV-8)、`\` で「会議」と打って絞り(NV-2)、`due` の昇順、`title`・`status`・`due` の3列にして保存 → 起動し直すと同じ3列・同じ並び・同じ2つの絞り込みで、あとから直したノートの値が出る。セッションにあって今の表に無い列 → その列だけ飛ばして警告(`test_CLI_7`) | [2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-10-01](../_decisions/2026-10-01-sessions-filters.md)、[2026-10-01](../_decisions/2026-10-01-sessions-details.md) |
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
- CLI-22(`--check`)は 2026-10-15 に削除し、文と確かめ方を変えずに CLI-23 へ移した([記録](../_decisions/2026-10-15-check-id.md))。CLI-22 の ID は、公開リポで先にマージしたエージェントのスキルの要件(2026-10-14-agent-skill)のもの。

## 前提(AI の仮定)

- 設定の書式を TOML にしたのは、Rust の TUI の多く(xleak・tabiew)と揃えるため(AI の判断)。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-15 | [--check の要件の ID を CLI-22 から CLI-23 に付け替える(check-id)](../_decisions/2026-10-15-check-id.md) | accepted | AI | 検証の指摘 | CLI-22, CLI-23 |
| 2026-10-14 | [型と決まりに合わない値を「問題」として探し、画面と CLI で確かめられるようにする(type-check)](../_decisions/2026-10-14-type-check.md) | accepted | 人 | 人の発言 | CE-36, CV-9, NV-30, CLI-22 |
| 2026-10-14 | [--check の列の型を画面と同じ推定にし、.md を渡したらそのノートだけを確かめる(check-kinds)](../_decisions/2026-10-14-check-kinds.md) | accepted | AI | 検証の指摘 | CLI-22 |
| 2026-10-14 | [mdgrid を CLI で使う AI エージェント向けのスキルを、プラグインとして置く(agent-skill)](../_decisions/2026-10-14-agent-skill.md) | accepted | 人 | 人の発言 | CLI-22 |
| 2026-10-13 | [表示の項目が11になって変わる試験の錠を掛け直す(row-lines-locks)](../_decisions/2026-10-13-row-lines-locks.md) | accepted | 人 | AI の提案 | SR-20, SR-21, CLI-11 ほか 4 |
| 2026-10-12 | [余白の既定を 1 にして変わる試験の錠を掛け直す(padding-default-one-locks)](../_decisions/2026-10-12-padding-default-one-locks.md) | accepted | 人 | AI の提案 | NV-23, SR-1, SR-26, CLI-11, SR-21 |
| 2026-10-12 | [右上のメニュー・3行のビューの帯で行の位置が変わる試験の錠を、まとめて掛け直す(header-menu-locks)](../_decisions/2026-10-12-header-menu-locks.md) | accepted | 人 | AI の提案 | BV-3, BV-7, BV-13 ほか 26 |
| 2026-10-12 | [余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)](../_decisions/2026-10-12-half-row-locks.md) | accepted | 人 | AI の提案 | BV-7, BV-13, BV-17 ほか 28 |
| 2026-10-11 | [絞り込みを並べ替えと同じボタンと窓にし、上の欄の検索と呼び分ける(filter-button)](../_decisions/2026-10-11-filter-button.md) | accepted | 人 | 人の発言 | NV-29, NV-23, NV-24 ほか 6 |
| 2026-10-10 | [表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)](../_decisions/2026-10-10-scope-overrides.md) | accepted | 人 | 人の発言 | SR-44, SR-43, WS-1, WS-7, CLI-21 |
| 2026-10-10 | [設定の形を作り直し、アプリ全体の項目と表のプロファイルに分け、重なる項目をまとめる(config-v2)](../_decisions/2026-10-10-config-v2.md) | accepted | 人 | 人の発言 | CLI-3, CLI-11, CLI-12 ほか 16 |
| 2026-10-10 | [公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)](../_decisions/2026-10-10-config-v2-relock.md) | accepted | 人 | 人の発言 | BV-17, BV-19, BV-20 ほか 42 |
| 2026-10-10 | [設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)](../_decisions/2026-10-10-config-v2-locks.md) | accepted | 人 | 人の発言 | BV-17, BV-20, CE-16 ほか 34 |
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

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| CLI-1 | [2026-10-08](../_decisions/2026-10-08-places.md) | src/test_main.rs::test_cli_1_no_args_opens_current_folder_default_table, src/test_main.rs::test_cli_1_paths_or_current_folder, src/ui/test_grid.rs::test_cli_2_view_option_opens_named_view ほか 2 | [2026-10-10-release-0-3-0](../_changes/2026-10-10-release-0-3-0.md) 2026-10-10 |
| CLI-2 | [2026-10-03](../_decisions/2026-10-03-stale-text.md) | src/test_main.rs::test_cli_11_print_config_arg, src/test_main.rs::test_cli_2_config_path, src/test_main.rs::test_cli_2_options ほか 5 | [2026-10-09-release-v0.2.0](../_changes/2026-10-09-release-v0.2.0.md) 2026-10-09 |
| CLI-3 | [2026-10-10](../_decisions/2026-10-10-config-v2.md) | src/test_config_unit.rs::test_ce_22_config_date_items_wrong_types_warn, src/test_config_unit.rs::test_nv_23_config_search_bar, src/test_main.rs::test_cli_3_config_missing_unknown_broken ほか 36 | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| CLI-4 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/test_main.rs::test_cli_4_missing_path_one_line_reason, src/ui/test_grid.rs::test_cli_4_unknown_view_one_line_exit_2, tests/test_open_md_file.rs::test_cli_15_missing_md_file_fails_with_one_line_and_code_2 ほか 3 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CLI-11 | [2026-10-10](../_decisions/2026-10-10-config-v2.md) | src/test_main.rs::test_cli_11_print_config_arg, tests/test_config_docs.rs::test_cli_11_output_has_every_item_with_comment, tests/test_config_docs.rs::test_cli_11_output_reads_back_without_warnings_as_default ほか 8 | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| CLI-12 | [2026-10-10](../_decisions/2026-10-10-config-v2.md) | src/test_config_unit.rs::test_cli_12_every_item_is_read, src/test_config_unit.rs::test_cli_12_items_have_descriptions, src/ui/test_keymap.rs::test_cli_12_keys_item_lists_every_mode ほか 11 | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| CLI-13 | [2026-10-03](../_decisions/2026-10-03-completions-man.md) | なし | [2026-10-06-help-wording](../_changes/2026-10-06-help-wording.md) 2026-10-06 |
| CLI-14 | [2026-10-06](../_decisions/2026-10-06-print-with-path.md) | tests/test_print_with_path.rs::test_cli_14_base_with_path_is_relative_to_base_folder, tests/test_print_with_path.rs::test_cli_14_csv_with_path_first_column_is_note_path, tests/test_print_with_path.rs::test_cli_14_json_with_path_first_key_is_path ほか 4 | [2026-10-07-windows-path-slash](../_changes/2026-10-07-windows-path-slash.md) 2026-10-07 |
| CLI-15 | [2026-10-06](../_decisions/2026-10-06-print-md-rows.md) | tests/test_open_md_file.rs::test_cli_15_md_file_opens_folder_and_selects_its_row, tests/test_open_md_file.rs::test_cli_15_missing_md_file_fails_with_one_line_and_code_2, tests/test_open_md_file.rs::test_cli_15_print_md_file_extension_is_case_insensitive ほか 3 | [2026-10-06-print-md-rows](../_changes/2026-10-06-print-md-rows.md) 2026-10-06 |
| WB-15 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_action_menu_gaps.rs::test_sr_24_gaps_readonly_launch_hides_save_new_export, src/ui/test_click_name.rs::test_sr_19_readonly_click_same_as_e, src/ui/test_look_section.rs::test_sr_43_readonly_applies_without_writing ほか 13 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| CLI-5 | [2026-10-07](../_decisions/2026-10-07-export-table.md) | tests/test_language_lib.rs::test_sr_23_lib_print_warning_is_english_with_lang_en, tests/test_language_lib.rs::test_sr_23_lib_print_warning_is_japanese_with_lang_ja, tests/test_print.rs::test_cli_5_default_csv_header_rows_and_rfc4180 ほか 12 | [2026-10-07-export-table](../_changes/2026-10-07-export-table.md) 2026-10-07 |
| CLI-20 | [2026-10-10](../_decisions/2026-10-10-config-v2.md) | src/test_uifile_unit.rs::test_cli_20_old_look_toml_is_read_and_moved, tests/test_config_docs.rs::test_cli_12_docs_list_old_names, tests/test_config_v2.rs::test_cli_20_migrate_config_prints_the_new_layout ほか 3 | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| CLI-21 | [2026-10-10](../_decisions/2026-10-10-scope-overrides.md) | src/test_profile_unit.rs::test_cli_21_resolved_toml_names_the_origin, tests/test_config_v2.rs::test_cli_21_print_resolved_names_the_origin | [2026-10-10-config-v2](../_changes/2026-10-10-config-v2.md) 2026-10-10 |
| CLI-22 | [2026-10-14](../_decisions/2026-10-14-agent-skill.md) | tests/test_agent_skill.rs::test_cli_22_plugin_versions_match_crate, tests/test_agent_skill.rs::test_cli_22_skill_examples_run | [2026-10-14-agent-skill](../_changes/2026-10-14-agent-skill.md) 2026-10-14 |
| CLI-23 | [2026-10-15](../_decisions/2026-10-15-check-id.md) | tests/test_check_cli.rs::test_cli_23_missing_path_exits_2, tests/test_check_cli.rs::test_cli_23_no_problems_prints_nothing_and_exits_0, tests/test_check_cli.rs::test_cli_23_reports_type_and_required_problems ほか 3 | [2026-10-14-type-check](../_changes/2026-10-14-type-check.md) 2026-10-15 |
| CLI-6 | [2026-10-01](../_decisions/2026-10-01-sessions.md) | なし | なし |
| CLI-7 | [2026-10-11](../_decisions/2026-10-11-filter-button.md) | なし | [2026-10-11-filter-button](../_changes/2026-10-11-filter-button.md) 2026-10-11 |
| CLI-8 | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) | なし | なし |
| CLI-9 | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) | なし | なし |
| CLI-10 | [2026-10-01](../_decisions/2026-10-01-sessions-details.md) | なし | なし |
| CLI-16 | [2026-10-06](../_decisions/2026-10-06-print-filter-sort.md) | tests/test_cli_polish.rs::test_cli_16_sort_unknown_column, tests/test_cli_polish.rs::test_cli_17_ignored_columns_noted, tests/test_cli_polish.rs::test_cli_17_wording ほか 10 | [2026-10-06-review6-fixes](../_changes/2026-10-06-review6-fixes.md) 2026-10-06 |
| CLI-17 | [2026-10-06](../_decisions/2026-10-06-apply.md) | tests/test_apply.rs::test_cli_17_any_problem_writes_nothing, tests/test_apply.rs::test_cli_17_csv_round_trip_from_stdin, tests/test_apply.rs::test_cli_17_empty_csv_cell_does_not_add_key ほか 16 | [2026-10-07-refactor-maintainability](../_changes/2026-10-07-refactor-maintainability.md) 2026-10-07 |
| CLI-18 | [2026-10-08](../_decisions/2026-10-08-places.md) | src/test_places_keep_unit.rs::test_cli_18_add_keeps_hand_written_text, src/test_places_keep_unit.rs::test_cli_18_broken_block_is_kept, src/test_places_keep_unit.rs::test_cli_18_replace_only_that_block ほか 9 | [2026-10-09-review-fixes](../_changes/2026-10-09-review-fixes.md) 2026-10-09 |
| CLI-19 | [2026-10-08](../_decisions/2026-10-08-places.md) | src/test_places_unit.rs::test_cli_19_grouped_order, src/ui/test_places.rs::test_cli_19_here_closes_list, src/ui/test_places.rs::test_cli_19_no_places_message ほか 7 | [2026-10-09-places-list-align](../_changes/2026-10-09-places-list-align.md) 2026-10-09 |
<!-- decidespec:history:end -->
