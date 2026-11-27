# 起動と設定(cli) の要求

## 目的

mdgrid の起動のしかたと設定ファイル。設定が無くても既定で動くことを先にする(細かい設定を必須にしない)。見た目の状態(列の幅など)は設定ではなく、[screen](screen.md) の screen.state-separate・screen.state-store で別に持つ。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/cli/spec.md](../_history/decidespec/cli/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: 設定ファイルでの任意のコードの実行(VisiData の Python の設定の型。安全面で不利)。
- CLI-22(`--check`)は 2026-10-15 に削除し、文と確かめ方を変えずに CLI-23(今の cli.check)へ移した([記録](../_history/decidespec/_decisions/2026-10-15-check-id.md))。CLI-22 の ID は、公開リポで先にマージしたエージェントのスキルの要件(2026-10-14-agent-skill)のもの。(旧い ID のまま残す)

## 前提(旧い仕様で AI が置いた仮定)

- 設定の書式を TOML にしたのは、Rust の TUI の多く(xleak・tabiew)と揃えるため(AI の判断)。

## 要求

### 起動と設定

#### `mdgrid [<.base のパス \| フォルダ> …]` で起動し、引数に渡した `.base` かフォルダの表を開くべき
`req~cli.launch-args~2` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes`
  - Then: notes の既定の表が開く。
- Acceptance:
  - Given:
  - When: `mdgrid notes/Tasks.base`
  - Then: Tasks.base の表が開く。

#### 引数なしで起動し、登録した表(cli.places)が無いときは、今のフォルダを開くべき
`req~cli.launch-no-args-cwd~1` P1
- Acceptance:
  - Given:
  - When: places.toml に登録が無く、引数なしでノートのフォルダから起動
  - Then: そのフォルダの既定の表。

#### 引数なしで起動し、登録した表があるときは、cli.places-list の一覧を出すべき
`req~cli.launch-no-args-list~1` P1
- Acceptance:
  - Given:
  - When: places.toml に登録があり、引数なしで起動
  - Then: 一覧(先頭は今のフォルダ)。

#### 開くフォルダ(引数に渡したフォルダ1つか、引数が無いときの今のフォルダ)の印 `.mdgrid/workspace.toml` の `open` が印の表を指すときは、既定の表と一覧の代わりにその表を開くべき(workspace.marker)
`req~cli.launch-marker-open~3` P1
- Acceptance:
  - Given: 根に `tasks.base` と AGENTS.md のあるフォルダに、`open = "台帳"` と `[[table]] name = "台帳"`・`path = "tasks.base"` の印
  - When:
  - Then: そのフォルダで引数なしの起動・`mdgrid .`・ほかのフォルダからそのフォルダのパスを渡した起動のどれでも tasks.base の表が開き、AGENTS.md の行は無い。places.toml に登録があっても cli.places-list の一覧を出さずにその表。
- Acceptance:
  - Given: 同じ印で、根の下に、どの表のノートのフォルダでもない `memo/`
  - When: `memo/` を開く
  - Then: memo の既定の表(印の `open` は上へたどらない。表のノートのフォルダを開いたときは cli.launch-table-folder)。

#### 画面を出さない起動(`--print`・`--check`・`--print-config --resolved`)でも、印の `open` を同じに使うべき
`req~cli.launch-marker-headless~1` P1
- Rationale: 画面と書き出しで開く表が食い違わないため
- Acceptance:
  - Given:
  - When: 根に `tasks.base` と AGENTS.md があり、`open = "台帳"`(`path = "tasks.base"`)の印のあるフォルダで `mdgrid --print`
  - Then: tasks.base の最初のビューの行。

#### フォルダを2つ以上渡したときと、`.md` のファイルを渡したとき(cli.open-md。そのノートの行を選ぶため)は、印の `open` を使わないべき
`req~cli.launch-marker-skip~2` P1
- Acceptance:
  - Given:
  - When: `open = "台帳"` の印のある根で `mdgrid AGENTS.md`
  - Then: 根の既定の表で AGENTS.md の行を選ぶ。
- Acceptance:
  - Given:
  - When: 根と別のフォルダの2つを渡す
  - Then: 2つの既定の表。

#### `--raw` を添えたときは、印の `open` を使わずに、そのフォルダの既定の表(base-view.source)を開くべき
`req~cli.launch-raw~2` P1
- Rationale: 素の表を見たいときに印を消さずに済むため
- Acceptance:
  - Given:
  - When: `open = "台帳"` の印のある根で `mdgrid --raw` と `mdgrid . --raw`
  - Then: AGENTS.md の行もある既定の表。
- Acceptance:
  - Given:
  - When: `mdgrid --raw --print`
  - Then: 既定の表の行。

#### 印の無いフォルダでの `--raw` は、何も変えないべき
`req~cli.launch-raw-no-marker~1` P1
- Acceptance:
  - Given: 印の無いフォルダで `--raw`
  - When:
  - Then: 今までどおり(`--raw` の無い起動と同じ表)。

#### 印のあるフォルダでも、印の表と別の `.base` は、そのパスを渡して開く
`req~cli.launch-marker-other-base~1` P1
- Acceptance:
  - Given:
  - When: `open = "台帳"`(`path = "tasks.base"`)の印のあるフォルダの別の `other.base` のパスを渡す
  - Then: tasks.base でなく other.base の表が開く。

#### `--view <名前>`・`--readonly`・`--no-color`・`--config <パス>`・`--print-config`(cli.print-config)・`--help`・`--version` を受け付けるべき(セッションの `-s`・`--session`・`--sessions` は、セッション(cli.session-save・cli.session-screen-list)を作るときに足す)
`req~cli.options~2` P1
- Acceptance:
  - Given: `--view 進行中`
  - When:
  - Then: そのビューで開く。

#### 画面を出す起動(`--pick` も)の `--view <名前>` は、`.base` のビューに無ければ mdgrid のビュー(base-view.own-views-tabs)からも名前で探して開き、どちらにも無ければ、ある名前を並べた理由1行と終了コード 2 で終わるべき
`req~cli.view-own-views~2` P1
- Rationale: 開き直す命令(resume.reopen-args)で mdgrid のビューも戻せるようにするため(`--print` はもう mdgrid のビューも探す。cli.print-stdout)
- Acceptance:
  - Given:
  - When: views.toml にビュー「進行中」のある、`.base` の無いフォルダ notes で `mdgrid notes --view 進行中`
  - Then: 「進行中」のタブで開く。
- Acceptance:
  - Given:
  - When: `mdgrid notes --view 無い`
  - Then: `進行中` を並べた理由1行と終了コード 2。

#### 設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`。Windows では `XDG_CONFIG_HOME` が無ければ `%APPDATA%\mdgrid\config.toml`)に1つ置くべき
`req~cli.config-file-location~1` P1
- Acceptance:
  - Given: Windows で `XDG_CONFIG_HOME` が無く `APPDATA` が `C:\Users\a\AppData\Roaming` → `C:\Users\a\AppData\Roaming\mdgrid\config.toml`。`XDG_CONFIG_HOME` が絶対パスならそちら、相対パスは無視して `APPDATA`。どちらも無い・空
  - When:
  - Then: 置き場なし(既定で動く)。

#### 設定の項目は、アプリ全体の項目と、表の見え方と振る舞いを決める表のプロファイルに分けるべき(全項目は cli.config-docs の文書)
`req~cli.config-file-items~1` P1
- Acceptance:
  - Given:
  - When: アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、端末の性質の `[terminal]`(`color`・`ambiguous_wide`・`nerd_font`)、`[workspace]`、`[keys]`)と、表のプロファイル(`[look]`(テーマ・組・セルの部品の切り替え・部品の形 `[look.style]`・列ごとの見せ方 `[look.columns]`・色 `[look.colors]`)・`[display]`・`[dates]`・`[edit]`(候補の数(cell-edit.value-list)・フロントマターを足すか(write-back.add-key-and-frontmatter))・`[new_note]`・土台にするテンプレートの名前 `use`)をその場所に書いた config.toml
  - Then: どの項目も警告なしに読まれる。

#### 名前を付けたプロファイルの断片(テンプレート)を、設定の `[templates.<名前>]` に書けるべき
`req~cli.config-file-templates~1` P1
- Acceptance:
  - Given: config.toml に `[templates.night.look]` の `theme = "nord"` と、最上位の `use = "night"`
  - When:
  - Then: 警告なしに読まれ、`look.theme` が nord。

#### 画面で選んだ設定(screen.look-section)は、同じ置き場の `ui.toml` に config.toml と同じ形で置き、config.toml の上に重ねるべき
`req~cli.config-file-ui-layer~1` P1
- Acceptance:
  - Given: ui.toml に `[look] theme = "nord"`、config.toml に `[look] theme = "sumi"`
  - When:
  - Then: nord。

#### 画面で選んだ設定を置くときは、config.toml を書き換えないべき
`req~cli.config-file-ui-keeps-config~1` P1
- Acceptance:
  - Given:
  - When: 見た目の区画(screen.look-section)で選んだ設定を反映する
  - Then: ui.toml に書かれ、config.toml のバイトは変わらない。

#### 設定が無くても、既定で動くべき
`req~cli.config-file-defaults~1` P1
- Acceptance:
  - Given:
  - When: 設定の無い環境で起動
  - Then: 既定で動く。

#### 設定の知らない項目・型の違う値・その場所に書けない項目は、警告にとどめるべき
`req~cli.config-file-warn-only~1` P1
- Acceptance:
  - Given: 知らない項目がある設定
  - When:
  - Then: 起動して警告。

#### 設定の警告は、どの設定のファイル(config.toml・ui.toml・views.toml・workspaces.toml・ワークスペースの印・places.toml・sessions.toml)でも「ファイルの名前: 項目の道筋(`look.style.status` など): 理由」の形にそろえるべき
`req~cli.config-file-warning-form~2` P1
- Acceptance:
  - Given: `[look.style] status = "neon"`
  - When:
  - Then: `config.toml: look.style.status:` で始まる警告。
- Acceptance:
  - Given: views.toml の知らない項目
  - When:
  - Then: `views.toml: table.view.foo:` で始まる警告。

#### 起動できないとき(パスが無い・`.base` が読めない)は、理由を1行で出し、0 以外の終了コードで終わるべき
`req~cli.fail-fast~1` P1
- Acceptance:
  - Given:
  - When: 無いパスを渡す
  - Then: 理由1行と終了コード 2。

#### 読むだけのファイル(`.base`・設定ファイル・`--apply` の入力・新しいノートの雛形)も、文字のファイルでない(write-back.refuse-not-text-any-file と同じ決まり)ときは中身を使わず、そのファイルの名前とテキストでないという理由を出すべき
`req~cli.read-text-only~1` P1
- Rationale: 名前だけ `.base` や `.toml` のほかのファイルを読んで、意味の無い表や設定にしないため。UTF-8 でないファイルと同じに止め、理由で見分けられるようにする
- Acceptance:
  - Given: NUL のバイトを含む `tasks.base`
  - When: `mdgrid tasks.base --print`
  - Then: 表を出さず、`tasks.base` とテキストでないという理由の1行と、終了コード 2。
- Acceptance:
  - Given: NUL のバイトを含む設定ファイル
  - When: `mdgrid --config <その設定ファイル> notes --print`
  - Then: 表を出さず、設定ファイルのパスとテキストでないという理由の1行と、終了コード 2(UTF-8 でない設定ファイルと同じ)。
- Acceptance:
  - Given: NUL のバイトを含む `in.csv`
  - When: `mdgrid notes --apply in.csv`
  - Then: どのノートも書かず、`in.csv` とテキストでないという理由の1行と、終了コード 2。
- Acceptance:
  - Given: `[new_note] body` の雛形が NUL のバイトを含む
  - When: `mdgrid notes --new-note 会議`
  - Then: ノートを作らず、雛形のパスとテキストでないという理由の1行と、0 以外の終了コード。
- Needs: stest

#### `mdgrid --print-config` は、設定の全項目を、区画ごとに既定値と説明のコメント付きの TOML で標準出力に出し、終了コード 0 で終わるべき
`req~cli.print-config-output~1` P1
- Rationale: dotfiles に貼る出発点にするため
- Acceptance:
  - Given:
  - When: `mdgrid --print-config`
  - Then: 終了コード 0 で、出力に全項目の名前(区画(アプリ全体の最上位・`[terminal]`・`[workspace]`・`[look]`・`[look.style]`・`[look.colors]`・`[display]`・`[dates]`・`[edit]`・`[new_note]`・`[keys]`・`[templates]`)の見出しの下)と既定値とコメントがあり、旧い名前(`theme_light`・`borders`・`view_tabs` など)が無い。

#### `--print-config` の各項目の説明には、書ける範囲(全体だけか、ワークスペース・表・ビューでも書けるか)を添えるべき
`req~cli.print-config-scopes~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --print-config`
  - Then: 出力の各項目の説明のコメントに、書ける範囲がある。

#### `--print-config` が出したものは、そのまま設定ファイルとして警告なしで読めて、既定と同じ振る舞いになるべき
`req~cli.print-config-reads-back~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --print-config > c.toml`
  - Then: `mdgrid --config c.toml` で警告なしに起動し、既定と同じ。

#### 設定の全項目を、区画ごとに、項目の道筋(`look.theme` など)・型・既定値・書ける範囲・説明で並べた文書を、英語(docs/config.md)と日本語(docs/config.ja.md)で置き、旧い名前(cli.legacy-config)と移った先の対応の表も置くべきで、文書の項目と、実装が読む項目と、`--print-config` の項目と、旧い名前の対応が食い違えば試験で落ちるべき
`req~cli.config-docs~2` P1
- Rationale: 文書が古くなるのを防ぐため
- Acceptance:
  - Given:
  - When: 実装に項目を足して文書に足さない
  - Then: 試験が落ちる。
- Acceptance:
  - Given:
  - When: 旧い名前の対応を実装に足して文書の表に足さない
  - Then: 試験が落ちる。

#### `mdgrid --completions <シェル>`(bash・zsh・fish・elvish・powershell)は、そのシェルの補完の定義を標準出力に出して終了コード 0 で終わってよい
`req~cli.completions-man-shells~1` P1
- Acceptance:
  - Given: `--completions zsh`
  - When:
  - Then: `#compdef mdgrid` で始まり、終了コード 0。

#### `mdgrid --man` は、man ページ(roff)を標準出力に出して終了コード 0 で終わってよい
`req~cli.completions-man-roff~1` P1
- Acceptance:
  - Given:
  - When: `--man`
  - Then: `.TH` を含み、終了コード 0。

#### `--completions` に知らないシェルの名前を渡したときは、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.completions-man-unknown-shell~1` P1
- Acceptance:
  - Given: `--completions nushell`
  - When:
  - Then: 理由1行と終了コード 2。

#### 補完と man は、受け付けるオプション(cli.options)と食い違わないよう、引数の定義から作るべき
`req~cli.completions-man-from-args~2` P1
- Acceptance:
  - Given: `--completions zsh`
  - When:
  - Then: `--readonly` を含む。
- Acceptance:
  - Given:
  - When: `--man`
  - Then: `--print-config` を含む。

#### `--print` に `--with-path` を付けたときは、表の先頭に見出し `path` の列を足し、各行にそのノートのパスを出すべき
`req~cli.with-path-column~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid examples/demo/Tasks.base --print --with-path`
  - Then: 見出しの行が `path,status,…`、各行の先頭が `examples/demo/Tasks/Fix login redirect bug.md` のようなパス(output.pick の `--pick path` と同じ形: 起動の引数のフォルダにノートの相対のパスをつないだもの)。フロントマターの無いノートの行もパスで見分けられる。

#### `--print --with-path` の json では、各オブジェクトの最初の鍵を `path` にするべき
`req~cli.with-path-json-first~1` P1
- Acceptance:
  - Given: `--print --with-path --format json`
  - When:
  - Then: 各オブジェクトの最初が `"path"`。

#### `--print --with-path` の json で、ビューに同じ見出し `path` の列があれば、先頭に足した `path` を鍵 `path` に、ビューの列を鍵 `path (<その列の id>)` に出し、それでも重なれば後ろに ` #<列の番号>`(1から)を添えるべき
`req~cli.with-path-name-clash~2` P1
- Acceptance:
  - Given:
  - When: `.base` の無いフォルダで、ノートのキー `path`(値 `中の値`)を持つノートを `--print --with-path --format json`
  - Then: オブジェクトに、鍵 `path`(足したノートのパス)と鍵 `path (path)`(値 `中の値`)の両方がある。

#### `--print --with-path` の csv と json と、画面の表の書き出し(output.export)の csv と json には、最後に見出し `_rev` の列を足し、各行に、そのノートの値(フロントマターのキーと値。本文は入れない)から作った印を出すべき(表に見出しが `_rev` の列があれば足さず、標準エラーに1行で知らせる)
`req~cli.print-rev~1` P0
- Rationale: 書き出してから `--apply` で戻すまでのあいだに、ほかの人(や同期で届いた変更)がノートを直していないかを、戻すときに確かめるため(cli.apply-stale-row)。印は中身を言わない文字にし、形は約束しない(あとで細かくできるように)
- Acceptance:
  - Given: `status: todo`・`due: 2026-12-01` のノート
  - When: `--print --with-path` で書き出す
  - Then: 見出しの行の最後が `_rev` で、その行に空でない印がある。
- Acceptance:
  - Given: 同じノートを書き出したあと、本文だけを直した
  - When: もう一度書き出す
  - Then: 印は同じ。
- Acceptance:
  - Given: 同じノートを書き出したあと、due を直した
  - When: もう一度書き出す
  - Then: 印が違う。
- Acceptance:
  - Given: `--print --with-path --format json`
  - When: 書き出す
  - Then: 各オブジェクトの最後の鍵が `_rev`。`--format md` と `tsv` には `_rev` が無い。
- Acceptance:
  - Given: 画面で表を `out.csv` に書き出す
  - When: 書き出したファイルを読む
  - Then: 見出しの最後が `_rev` で、`--print --with-path` と同じ印。
- Needs: utest・stest

#### `--print` の無い `--with-path` は、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.with-path-needs-print~1` P1
- Acceptance:
  - Given: `--with-path` だけ
  - When:
  - Then: 理由1行と終了コード 2。

#### 起動の引数に `.md` のファイルを渡したときは、そのファイルのあるフォルダを渡したのと同じに開くべき
`req~cli.open-md-folder~1` P1
- Rationale: ファイル名を補完で渡す人が「フォルダでない」で止められないようにするため
- Acceptance:
  - Given:
  - When: `mdgrid notes/b.md`
  - Then: notes の既定の表が開く。

#### 起動の引数に `.md` のファイルを渡したときは、画面ではそのノートの行を選んで始めるべき
`req~cli.open-md-select~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes/b.md`
  - Then: b.md の行が選ばれている。

#### 起動の引数に渡した `.md` のファイルが無いときは、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.open-md-missing~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes/無い.md`
  - Then: 理由1行と終了コード 2。

#### `--print` に `.md` のファイルを渡したときは、そのフォルダの表のうち、渡したノートの行だけを出すべき
`req~cli.open-md-print-rows~2` P1
- Rationale: 1つのノートの値を取り出すため
- Acceptance:
  - Given:
  - When: `mdgrid notes/b.md --print`
  - Then: notes の表と同じ列で、b の1行だけ。
- Acceptance:
  - Given: `notes/a.md notes/b.md --print`
  - When:
  - Then: a と b の2行。

#### 読むだけの起動(`--readonly`)では、保存の操作を出さず、どのファイルも書くべきでない
`req~cli.readonly~1` P0
- Acceptance:
  - Given:
  - When: `--readonly` で起動して Ctrl+S
  - Then: 何も書かれず、読むだけと表示。

#### `mdgrid [<.base | フォルダ> …] --print [--format csv|tsv|json|md] [--view <名前>]` は、画面を出さずに、開いたビュー(`.base` のビューか mdgrid のビュー。base-view.own-views)の表を標準出力に出して終了コード 0 で終わるべき
`req~cli.print-stdout~1` P1
- Acceptance:
  - Given: `--print --view 進行中`
  - When:
  - Then: 画面を出さずに進行中のビューの表が標準出力に出て、終了コード 0 で終わる。標準出力がパイプでも動く。

#### `--print` の行は、ビューの絞り込みと並べ替えのあとの行にするべき(グループ分けは並びだけに効かせ、見出しの行は出さない)
`req~cli.print-rows~2` P1
- Acceptance:
  - Given:
  - When: 絞り込み `status != "done"` と `due` の昇順の並べ替えのビュー
  - Then: done でない行だけが due の順に出る。
- Acceptance:
  - Given: グループ分けのあるビュー
  - When:
  - Then: グループの見出しの行は出ない。

#### `--print` の行の並びは、グループ分けの列に値の候補(`[rules.<列>] values`)があれば、画面と同じく候補の順のまとまりで並べるべき(navigation.group-values-order)
`req~cli.print-group-order~1` P1
- Rationale: 画面ではまとまりを候補の順(todo・doing・waiting)に並べるのに、`--print` は値の文字の順(doing・todo・waiting)で、画面と出力で行の並びが違っていたため(2026-11-25 に、画面と命令の食い違いを洗い出した)。スクリプトや AI のエージェントが読む順を画面とそろえる
- Acceptance:
  - Given: 印に `[rules.status] values = ["todo", "doing", "waiting"]` と、status でグループ分けしたビュー、status が doing・todo・waiting の3行
  - When: `--print --view <そのビュー>`
  - Then: 行は todo・doing・waiting の順。
- Acceptance:
  - Given: 値の候補の無い列でグループ分けしたビュー
  - When: `--print`
  - Then: 今までどおり、値の文字の順。
- Needs: stest

#### `--print` の列は、ビューの列の並びにするべき
`req~cli.print-columns~1` P1
- Acceptance:
  - Given: 列が `title`・`due`・`status` の順のビュー
  - When:
  - Then: 見出しの行が `title,due,status`。

#### `--print` は、画面の見た目の状態(screen.state-store)と画面で掛けた絞り込みを使わないべき
`req~cli.print-no-screen-state~1` P1
- Acceptance:
  - Given:
  - When: 画面で列を隠し、検索で行を絞って終えた表を `--print`
  - Then: 隠した列も、検索で外れた行も、ビューのとおりに出る。

#### `--print` の既定の形は csv(RFC 4180、見出しの行つき、改行は LF)にするべき
`req~cli.print-csv~1` P1
- Acceptance:
  - Given: `--print`(`--format` なし)と `--print --format csv`
  - When:
  - Then: どちらも見出しの行と行ごとの CSV が出て終わる。

#### `--print --format tsv` は、タブ区切り(見出しの行つき、セルの中のタブと改行は空白)にするべき
`req~cli.print-tsv~1` P1
- Acceptance:
  - Given: `--format tsv`
  - When:
  - Then: 見出しの行つきのタブ区切りで、セルの中のタブと改行は空白。

#### `--print --format json` は、列の名前を鍵にしたオブジェクトの配列(数・真偽・null・リストは JSON の型、日付と日時は文字列)にするべき
`req~cli.print-json~1` P1
- Acceptance:
  - Given: `--format json`
  - When:
  - Then: `[{"title": "会議", "priority": 2, "tags": ["a"]}]` の形。

#### `--print --format md` は、見出しと区切りの行のある表(セルの中の縦棒はバックスラッシュで逃がし、改行は空白)にするべき
`req~cli.print-md~1` P1
- Acceptance:
  - Given: `--format md`
  - When:
  - Then: 見出しの行と区切りの行のある表。

#### `--print` の評価できない列(base-view.unsupported)は、csv と md で空、json で null にするべき
`req~cli.print-unsupported-empty~2` P1
- Acceptance:
  - Given: 評価できない式の列のあるビューを `--print --format csv`
  - When:
  - Then: その列のセルが空。
- Acceptance:
  - Given: `--format json`
  - When:
  - Then: その列が null。

#### `--print` で評価できない列があるときは、標準エラーに1行で知らせるべき
`req~cli.print-unsupported-warn~1` P1
- Acceptance:
  - Given: 評価できない式の列のあるビューを `--print`
  - When:
  - Then: 標準エラーに1行の知らせが出る。

#### `--print` は、ノートを書き換えないべき(cli.readonly と同じ)
`req~cli.print-no-write~1` P1
- Acceptance:
  - Given:
  - When: `--print` で表を出す
  - Then: ノートのフォルダのファイルは変わらず、新しいファイルもできない。

#### `--print` で起動できないとき・知らない形の名前は、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.print-fail~2` P1
- Acceptance:
  - Given: `--format xml`
  - When:
  - Then: 理由1行と終了コード 2。
- Acceptance:
  - Given: 無いパスで `--print`
  - When:
  - Then: 理由1行と終了コード 2。

#### 0.3.0 までの設定の書き方は、読んで新しい場所に当てるべき
`req~cli.legacy-config-apply~2` P1
- Acceptance:
  - Given: 次の旧い名前で書いた設定(Then の対応の左側)
  - When: 起動する
  - Then: それぞれ移った先に書いたのと同じに効く: `color`・`ambiguous_wide`・`nerd_font` → `[terminal]`、`workspace_detect` → `workspace.detect`、`theme`・`theme_light`・`theme_dark` → `look.theme`(`"auto"` に明暗のどちらかを書いていれば明暗の表)、`look` → `look.mode`、`borders` → `look.style.frames`、`[style]` → `[look.style]`(`preset` は `look.preset`)、`cells` の文字列と `[cells] style` → `look.cells`、`[cells]` の部品ごとの真偽 → `[look.style]` のその部品の `plain`(真偽は `check = "text"`、リンクは `links = "plain"`、印は `icons`)、`[cells.columns]` → `[look.columns]`、`[colors]` → `[look.colors]`、`view_tabs` → `display.tabs`、`[display] tabs = false` → `display.tabs = "never"`、`search_bar` → `display.search_bar`、`date_format`・`week_start` → `dates.format`・`dates.week_start`、`candidates`・`add_frontmatter` → `[edit]`。
- Acceptance:
  - Given: `theme = "auto"` と `theme_light = "paper"`
  - When: 明るい地の端末で起動する
  - Then: paper。
- Acceptance:
  - Given: `borders = "ascii"`
  - When: 起動する
  - Then: 枠が ASCII。

#### 0.3.0 までの書き方を読んだときは、項目ごとに「旧い名前は新しい道筋に移った」と警告するべき
`req~cli.legacy-config-warn~2` P1
- Acceptance:
  - Given: `theme = "auto"` と `theme_light = "paper"`
  - When:
  - Then: 警告に `theme_light` と `look.theme`。
- Acceptance:
  - Given: `borders = "ascii"`
  - When:
  - Then: 警告に `look.style.frames`。

#### 旧い名前と新しい道筋の両方に書いてあれば、新しい道筋を使うべき
`req~cli.legacy-config-new-wins~1` P1
- Acceptance:
  - Given: `view_tabs = "auto"` と `[display] tabs = "always"`
  - When:
  - Then: always。

#### 画面が書くファイルの旧い形(`look.toml`、views.toml の `[[target]]`)も読む
`req~cli.legacy-config-old-screen-files~1` P1
- Acceptance:
  - Given: look.toml だけがある
  - When:
  - Then: その見た目で起動する。

#### 画面が書くファイルを旧い形から読んだときは、次に書くときに新しい形(ui.toml、`[[table]]`)で書く
`req~cli.legacy-config-rewrite-new-form~1` P1
- Acceptance:
  - Given: look.toml だけがある
  - When:
  - Then: 見た目の区画で反映すると ui.toml ができる。

#### `look.toml` は、ui.toml に移したあと消してよい
`req~cli.legacy-config-remove-look-toml~1` P1
- Acceptance:
  - Given: look.toml だけがある
  - When:
  - Then: 見た目の区画で反映すると ui.toml ができて look.toml が無い。

#### `mdgrid --migrate-config` は、設定ファイル(`--config` があればそのファイル)を新しい形にした TOML を標準出力に出して、終了コード 0 で終わるべき
`req~cli.legacy-config-migrate~1` P1
- Acceptance:
  - Given:
  - When: `theme = "auto"` と `theme_light = "paper"` の設定で `mdgrid --migrate-config`
  - Then: `[look]` の下に `theme = { light = "paper", dark = "sumi" }`、出した文を読み直すと警告なしで同じ振る舞い。

#### `mdgrid --migrate-config` は、設定ファイルを書かないべき
`req~cli.legacy-config-migrate-no-write~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --migrate-config` のあと
  - Then: 設定ファイルのバイトは変わらない。

#### `mdgrid --migrate-config` は、注釈は移らないことを、出す文の先頭のコメントで知らせるべき
`req~cli.legacy-config-migrate-note~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --migrate-config`
  - Then: 出す文が `# ` のコメントで始まり、注釈が移らないことを書いている。

#### `mdgrid [<表>] --print-config --resolved` は、その表(引数が無ければ今のフォルダ。`--view` があればそのビュー)を開いたときに効く表のプロファイルとアプリ全体の項目の全部を、cli.print-config と同じ区画の TOML で標準出力に出し、終了コード 0 で終わるべき
`req~cli.resolved-config-values~1` P1
- Acceptance:
  - Given:
  - When: screen.profile-scopes の例で `mdgrid notes/tasks --print-config --resolved`
  - Then: `theme = "dracula"` が出て、終了コード 0。

#### `--print-config --resolved` は、各項目の上に、その値がどこから来たかをコメントで添えるべき
`req~cli.resolved-config-origin~1` P1
- Acceptance:
  - Given:
  - When: screen.profile-scopes の例で `mdgrid notes/tasks --print-config --resolved`
  - Then: `theme = "dracula"` の上に `# views.toml (notes/tasks)`、`preset` の上に `# default`(来たところは、既定・config.toml・ui.toml・ワークスペースの印か workspaces.toml とその名前・views.toml の表・ビューの名前・テンプレートの名前)。

#### `--print-config --resolved` の入れ子の表の項目(`[rules.<列>]` の `icons`・`labels`・`then` の中の値など、層が表を丸ごと書く項目)の出どころのコメントは、その表を書いた層にするべき
`req~cli.resolved-config-origin-nested~1` P1
- Rationale: 印に書いた `[rules.status]` の `values` は印から来たと出るのに、同じ所に書いた `icons`・`labels`・`then` の中の値はどれも `# default` と出て、どこを直せばよいかが読めなかったため(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: ワークスペースの印に `[rules.status]` の `values`・`icons`・`labels` と `[rules.status.then]` を書いた表
  - When: `mdgrid <表> --print-config --resolved`
  - Then: `[rules.status.icons]`・`[rules.status.labels]`・`[rules.status.then.<値>]` の中の項目の上のコメントは、どれも `values` の上と同じ印の名前で、`# default` でない。
- Acceptance:
  - Given: config.toml が `[rules.status]` の `icons` を、ワークスペースの印が同じ列の `labels` を書く
  - When: 同じ命令
  - Then: `[rules.status.icons]` の中の項目の上は `# config.toml`、`[rules.status.labels]` の中の項目の上は印の名前。
- Needs: utest

#### `--print-config --resolved` が出したものは、設定ファイルとして警告なしに読めるべき
`req~cli.resolved-config-reads-back~1` P1
- Acceptance:
  - Given:
  - When: 出した文を `--config` で読む
  - Then: 警告なし。

#### `--print-config --resolved` は、画面を開かないべき
`req~cli.resolved-config-no-screen~1` P1
- Acceptance:
  - Given:
  - When: 標準出力がパイプのまま `mdgrid notes/tasks --print-config --resolved`
  - Then: 画面を出さずに TOML を出して終わる。

#### `--print-config --resolved` は、ファイルを書かないべき
`req~cli.resolved-config-no-write~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes/tasks --print-config --resolved` のあと
  - Then: 設定の置き場とノートのフォルダのファイルは変わらない。

#### リポは、AI エージェント(Claude Code・Codex)が mdgrid を CLI で使うための手引きのスキル(`SKILL.md`)を、プラグインとして入れられる形(Claude Code と Codex のマーケットプレイスの索引とマニフェスト)で置くべき
`req~cli.agent-skill-plugin~1` P1
- Acceptance:
  - Given: リポの Claude Code と Codex のマーケットプレイスの索引
  - When:
  - Then: どちらも、mdgrid の CLI の手引きの `SKILL.md` を持つプラグインのマニフェストを指す。

#### mdgrid の CLI の手引きのスキルは、頼まれた条件を `--print` の旗に組み立てて表をファイルに出す手順を書く
`req~cli.agent-skill-print-steps~1` P1
- Acceptance:
  - Given: SKILL.md
  - When:
  - Then: 頼まれた条件を `--print` の `--filter`・`--sort`・`--view`・`--format`・`--with-path` に組み立てて表をファイルに出す手順がある。

#### mdgrid の CLI の手引きのスキルは、`--apply` で書き戻すときは差分を人に見せてから `--yes` で書く手順を書く
`req~cli.agent-skill-apply-steps~1` P1
- Acceptance:
  - Given: SKILL.md
  - When:
  - Then: `--apply` の差分を人に見せてから `--yes` で書く手順がある。

#### mdgrid の CLI の手引きのスキルは、書き戻すときに `_rev` の列を残すことと、書き出したあとに変わったと止まったら書き出し直して直すことを書く
`req~cli.agent-skill-apply-stale~1` P1
- Rationale: AI が `_rev` の列を消すと確かめられず、止まったときに `--yes` を繰り返すと何も進まないため
- Acceptance:
  - Given: SKILL.md
  - When: 読む
  - Then: `_rev` の列を残すことと、止まったら書き出し直して同じ直しをやり直す手順がある。
- Needs: stest

#### mdgrid の CLI の手引きのスキルに書いた例のコマンドは、そのまま動かして通るべき
`req~cli.agent-skill-examples-run~1` P1
- Rationale: 古い旗や書けない式を AI に教えないため
- Acceptance:
  - Given:
  - When: SKILL.md の例のコマンドを見本のフォルダ(status が todo・doing・done のノート)で動かす
  - Then: 全部が終了コード 0 で、todo と doing の絞り込みの例は2行だけを出す。

#### スキルのプラグインのマニフェストの版は、Cargo.toml の版と揃えるべき
`req~cli.agent-skill-version~1` P1
- Acceptance:
  - Given:
  - When: マニフェストの版が Cargo.toml の版と違う
  - Then: 試験が落ちる。

#### `mdgrid [<.base | フォルダ> …] --check [--view <名前>]` は、画面を出さずに、表の各ノート(`--view` を渡したときはそのビューの行)を確かめ、問題を1つ1行で `<パス>:<行>: <列>: <理由>` の形で標準出力に出すべき
`req~cli.check-problems~2` P1
- Rationale: 最初のタブで絞った台帳でも、ほかのタブにしか出ない行の問題を見落とさないため(ブックマークの台帳で、URL の無いノートを見落とした。2026-11-25)
- Acceptance:
  - Given: `[types] due = "date"` で `due: someday` のノートと、`[rules.status] required = true` で status の無いノート
  - When:
  - Then: `notes/a.md:3: due: 型が合わない(日付ではない)` と `notes/b.md: status: 必須のキーが無い`(キーが無いときは行を付けない)。確かめるのは、`--view` が無ければ表の全部の行(`.base` の全体の絞り込みといつも隠す条件(condition.hide-everywhere)は当て、どのビューの絞り込みも、印の表の `view` も当てない)、`--view` があればそのビューの絞り込みのあとの行(どちらも cli.print と同じく、画面の見た目の状態は使わない)の各ノートの、全部のフロントマターのキーと決まり(cell-edit.rules)のある列の問題(cell-view.problems)。
- Acceptance:
  - Given: 最初のビュー「未完了」が `status != "done"` で絞る `.base` の表に、`status: done` と `due: someday` のノート(`[types] due = "date"`)
  - When: `mdgrid <その .base> --check`
  - Then: そのノートの `due` の問題の行が出て、終了コード 1。
- Acceptance:
  - Given: 同じ表
  - When: `mdgrid <その .base> --check --view 未完了`
  - Then: 出力なし、終了コード 0。

#### `--check` の問題の行は、パスと行の順に出すべき
`req~cli.check-order~1` P1
- Acceptance:
  - Given: notes/a.md の 5 行目と 3 行目と、notes/b.md に問題
  - When:
  - Then: `notes/a.md:3:`・`notes/a.md:5:`・`notes/b.md` の順に出る。

#### `--check` は、問題の数を標準エラーに1行で出すべき
`req~cli.check-count~1` P1
- Acceptance:
  - Given: `due: someday` のノートと status の無いノートの2つの問題
  - When:
  - Then: 標準エラーに「問題 2」。

#### `--check` の列の型は、画面と同じく cell-edit.column-type の順(設定・ルートフォルダの型の設定・読み込んだ全部のノート(base-view.root-folder の範囲)からの推定)で決めるべき
`req~cli.check-column-type~1` P1
- Rationale: 画面と `--check` で同じ値の判定が食い違わないため
- Acceptance:
  - Given:
  - When:
  - Then: ビューの絞り込みで日付の値の行だけが残っても、ほかの行の値で決まる列の型は画面と同じ。

#### `--check` の起動の引数に `.md` のファイルを渡したときは、`--print` と同じく(cli.open-md)そのノートの行だけを確かめるべき
`req~cli.check-md-rows~1` P1
- Acceptance:
  - Given: `notes/a.md --check`
  - When:
  - Then: a.md の問題だけ。

#### `--check` は、問題が無ければ終了コード 0、あれば 1 で終わるべき
`req~cli.check-exit-code~2` P1
- Rationale: CI と pre-commit で止められるようにするため
- Acceptance:
  - Given:
  - When: 問題が2つ
  - Then: 終了コード 1。
- Acceptance:
  - Given:
  - When: 直したあと
  - Then: 出力なし、終了コード 0。

#### `--check` で起動できないときは、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.check-fail~1` P1
- Acceptance:
  - Given: 無いパス
  - When:
  - Then: 理由1行と終了コード 2。

#### `--check` の問題の行のパスは、`--pick path`(output.pick)と同じ形にするべき
`req~cli.check-path-form~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --check`
  - Then: 問題の行の先頭が、`--pick path` と同じ `notes/a.md` の形。

#### `--check` は、ノートを書き換えないべき
`req~cli.check-no-write~1` P1
- Acceptance:
  - Given: 問題のあるノートで `--check`
  - When:
  - Then: ノートのファイルのバイトは変わらない。

#### リポは、AI エージェントがフォルダの印 `.mdgrid/workspace.toml` で開く表・ビュー・列の型・選ぶ列の値・ノートの欄の出し方を決めるための手引きのスキル(`SKILL.md`。名前 `mdgrid-views`)を、cli.agent-skill と同じプラグインに置くべき
`req~cli.views-skill-plugin~1` P1
- Acceptance:
  - Given: mdgrid の CLI の手引きのスキルと同じプラグインの中
  - When:
  - Then: 名前 `mdgrid-views` の `SKILL.md` がある。

#### `mdgrid-views` のスキルには、印の決め方の決まりと、終える前の確かめ方を書くべき
`req~cli.views-skill-checklist~1` P1
- Acceptance:
  - Given: スキルの文
  - When:
  - Then: 次が全部ある: 列の型と選ぶ列の値はデータの形の仕様から取り、仕様が無ければ値から推して迷えば型を付けずに文字のままにして人に聞くこと・日付の列に型を付けること・選ぶ列に値の並びを書くこと・行を見分ける名前を隠さないこと・状態のタブを `where` の配列で書くこと・ASCII でないキーを引用符で囲むこと・終える前に `--print-config --resolved`・`--print`・`--check` と画面で確かめること。スキルの手順に「型」「values」「where」「--check」の語がある。

#### `mdgrid-views` のスキルに書いた印の例は、見本のフォルダに置いて警告なしに読めるべき
`req~cli.views-skill-example-reads~1` P1
- Rationale: 古い書き方や読めない設定を AI に教えないため
- Acceptance:
  - Given:
  - When: スキルの印の例を、todo・archive/todo のノートと README.md のあるフォルダに置く
  - Then: `--print-config --resolved` に `mdgrid:` で始まる警告が無い。

#### `mdgrid-views` のスキルに書いた印の例を見本のフォルダに置くと、`--print` が印の表の行を出すべき
`req~cli.views-skill-example-prints~1` P1
- Acceptance:
  - Given:
  - When: スキルの印の例を、todo・archive/todo のノートと README.md のあるフォルダに置く
  - Then: `--print` が Open のビューの行だけで README.md の行が無い。`--print --view Done` が done の行だけ。

#### `mdgrid [<フォルダ>] --new-note <名前>` は、画面を出さずに、そのフォルダ(無ければ今のフォルダ)を開いたフォルダとして、画面の新しいノート(cell-edit.new-note-name)と同じ設定(cell-edit.new-note-config の `folder`・`set`・`hidden`・`body` と cell-edit.template-vars の変数)で `<名前>.md` を作るべき
`req~cli.new-note-create~2` P1
- Rationale: スクリプトや AI のエージェントが、画面と同じ形のノートを足せるようにするため
- Acceptance:
  - Given:
  - When: 印に `[new_note] folder = "Tasks"`・`body`(雛形 `# {name}`)・`[new_note.set] status = "todo"`・`created = "{date}"` のある台帳 `t` で、`MDGRID_TODAY=2026-11-18 mdgrid t --new-note "Buy milk"`
  - Then: `t/Tasks/Buy milk.md` ができ、フロントマターに `status: todo` と `created: 2026-11-18`、本文は `# Buy milk` で始まる。
- Acceptance:
  - Given: 設定も印も無いフォルダ
  - When:
  - Then: 空の `<名前>.md`。

#### `--new-note` は、`[new_note] name_head = "hide"` のときは、渡した名前に、名前の雛形(`name`)を埋めた頭を、新しいノートの窓(cell-edit.new-note-name-head)と同じ決まりで付けて作り、`"show"` か書かないときは渡した名前のまま作るべき
`req~cli.new-note-name-head~1` P1
- Rationale: 画面で作ったノートと、スクリプトや AI のエージェントが命令で作ったノートの名前の形をそろえるため。日付の頭を付けて作る台帳で、命令で足したタスクにだけ頭が無かった(2026-11-24 に理想のワークスペースを組んで気づいた)
- Acceptance:
  - Given: 今日が 2026-11-18、印に `[new_note] folder = "Tasks"`・`name = "{date:YYYYMMDD}_"`・`name_head = "hide"` のある台帳 `t`
  - When: `mdgrid t --new-note 設計`
  - Then: `t/Tasks/20261118_設計.md` ができ、標準出力はそのパスの1行。
- Acceptance:
  - Given: 同じ台帳に `[display] note_name = '^\d{8}_'` を足した
  - When: `mdgrid t --new-note 20250101_古い`
  - Then: `t/Tasks/20250101_古い.md` ができる(頭を重ねない)。
- Acceptance:
  - Given: `name_head` を書かない同じ台帳
  - When: `mdgrid t --new-note 設計`
  - Then: `t/Tasks/設計.md` ができる(今までどおり)。
- Needs: stest

#### `--new-note` の `--set <列>=<値>`(何度でも)は、新しいノートの窓で答えた値(cell-edit.new-note-form-fields)と同じに扱い、値を列の型(cell-edit.column-type)で読むべき
`req~cli.new-note-set~2` P1
- Acceptance:
  - Given: `--set due=2026-12-01 --set points=3 --set tags=home,errand`(due が日付・points が数・tags がリストの列)
  - When:
  - Then: `due: 2026-12-01`・`points: 3` と、要素 home・errand のリストの tags。
- Acceptance:
  - Given: `--set status=`(空の値)
  - When:
  - Then: `set` に status があっても status を書かない(cell-edit.new-note-form-skip-empty)。
- Acceptance:
  - Given: 数の列に `--set points=many`・日付の列に `--set due=someday`・`=` の無い `--set due`
  - When:
  - Then: 理由1行と終了コード 2 で作らない。

#### `--new-note` は、作ったノートのパスを、渡したフォルダにつないだ形で1行だけ標準出力に出し、終了コード 0 で終わるべき
`req~cli.new-note-report~1` P1
- Rationale: 続くコマンドが、作ったノートをそのまま開いたり書き足したりできるようにするため
- Acceptance:
  - Given:
  - When:
  - Then: `mdgrid t --new-note "Buy milk"`(`folder = "Tasks"`)→ 標準出力は `t/Tasks/Buy milk.md` の1行、標準エラーは空、終了コード 0。

#### `--new-note` は、既にある名前では作らず、既にあるファイルを1バイトも変えずに、理由1行と終了コード 2 で終わらなければならない
`req~cli.new-note-no-overwrite~1` P0
- Acceptance:
  - Given:
  - When: `t/Tasks/Buy milk.md` がある台帳で `mdgrid t --new-note "Buy milk" --set priority=p1`
  - Then: 理由1行(そのパスを言う)と終了コード 2 で、`Buy milk.md` のバイトは前と同じ。

#### `--new-note` は、開いたフォルダの外(`..` や絶対パス)を指す名前では作らず、理由1行と終了コード 2 で終わるべき(cell-edit.new-note-refuse-outside と同じ)
`req~cli.new-note-refuse-outside~2` P1
- Acceptance:
  - Given: `--new-note ../x`・絶対パスの名前
  - When:
  - Then: 理由1行と終了コード 2 で、フォルダの外に何もできない。
- Acceptance:
  - Given: `--new-note sub/x`
  - When:
  - Then: 作る場所の下の `sub/x.md` ができる(cell-edit.new-note-subfolder)。

#### `--new-note` は、設定の `required` の列(cell-edit.new-note-config)が空のときは作らず、その列の名前を言う理由1行と終了コード 2 で終わるべき
`req~cli.new-note-required~2` P1
- Acceptance:
  - Given:
  - When: `required = ["priority"]` の台帳で `mdgrid t --new-note a`
  - Then: 理由1行(priority を言う)と終了コード 2 で作らない。
- Acceptance:
  - Given:
  - When: `--set priority=p2` を添える
  - Then: 作る。

#### `--new-note` の無い `--set`・2つ以上のフォルダ・`--print` などほかの命令との組・空の名前は、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.new-note-bad-args~2` P1
- Acceptance:
  - Given:
  - When: `mdgrid t --set priority=p1`
  - Then: 理由1行と終了コード 2。
- Acceptance:
  - Given:
  - When: `mdgrid t u --new-note x`
  - Then: 理由1行と終了コード 2 で、どちらのフォルダにも作らない。

#### `--help` は、`--new-note` と `--set` を、どの言語でも出すべき
`req~cli.new-note-help~1` P1
- Acceptance:
  - Given:
  - When: `LANG=ja_JP.UTF-8` と `LANG=en_US.UTF-8` の `mdgrid --help`
  - Then: どちらにも `--new-note` と `--set` の行がある。

#### `--new-note` に、ワークスペースの印の表のノートを置くフォルダ(その表の `[new_note] folder` か、表の絞り込みの `file.inFolder` が指すフォルダ)を渡したら、その表の決まり(名前の頭・前もって入れる値・本文の雛形・必須の列)で、そのフォルダに作るべき
`req~cli.new-note-table-folder~1` P1
- Rationale: 印の根に表をいくつも書いた台帳で、表のフォルダを渡して `--new-note` を使うと、表の決まりを使わずに空のファイルができた。画面のその表で作るノートと形がそろわなかった(2026-11-25 に調べ物の台帳で見つかった)
- Acceptance:
  - Given: 今日が 2026-11-25、印の根 `r` に表「ニュース」(`[table.new_note] folder = "News"`・`name = "{date:YYYYMMDD}_"`・`name_head = "hide"`・`body` の雛形 `# {name}`・`[table.new_note.set] date = "{date}"`、絞り込み `file.inFolder("News")`)
  - When: `mdgrid r/News --new-note 見出し`
  - Then: `r/News/20261125_見出し.md` ができ、フロントマターに `date: 2026-11-25`、本文は `# 見出し`、標準出力は `r/News/20261125_見出し.md` の1行。
- Acceptance:
  - Given: 同じ印で、どの表のフォルダでもない `r/Other`
  - When: `mdgrid r/Other --new-note x`
  - Then: 表の決まりを使わず、`r/Other/x.md` が空で作られる(今までどおり)。
- Needs: stest

#### `--new-note` は、`[new_note] body` がフォルダを指すときは、画面で雛形を選ばずに作ったとき(cell-edit.body-choose-default)と同じく、名前の順で最初の雛形を本文にするべき(`.md` が1つも無ければ本文なし)
`req~cli.new-note-body-folder~1` P1
- Rationale: 本文の雛形をフォルダに並べた台帳で `--new-note` を使うと、フォルダをファイルとして読もうとして作れなかった(2026-11-25 に読書の記録の台帳で見つかった)
- Acceptance:
  - Given: `[new_note] body = "templates"` で、`templates/` に `book.md`(`# {name}`)と `memo.md`
  - When: `mdgrid t --new-note 本`
  - Then: `本.md` ができ、本文は `# 本`、終了コード 0。
- Acceptance:
  - Given: `body` のフォルダに `.md` が1つも無い
  - When: `mdgrid t --new-note x`
  - Then: 本文の無い `x.md` ができ、終了コード 0。
- Needs: stest

### セッション

#### 表の表示を名前を付けたセッションとして保存し、`mdgrid -s <名前>`(`--session <名前>`)で、その表示のまま起動できるべき
`req~cli.session-save~2` P1
- Acceptance:
  - Given:
  - When: フォルダを開き、絞り込み・並び・列を決めて名前 `進行中` で保存して終了
  - Then: `mdgrid -s 進行中` で同じフォルダ・同じ絞り込み・並び・列の表が開く。
- Acceptance:
  - Given:
  - When: 無い名前 `mdgrid -s 無い`
  - Then: 理由1行と終了コード 2。

#### セッションは、開くフォルダ(1つ以上)、`.base` のパスとビューの名前(任意)、画面で掛けた絞り込み、並び、列の並びと隠す列、読むだけで開くか(`--readonly`)を持つべき
`req~cli.session-content-fields~2` P1
- Acceptance:
  - Given:
  - When: `.base` の無いフォルダで status のセルが「進行中」の行だけに絞り(navigation.same-value)、`\` で「会議」と打って絞り(navigation.search)、`due` の昇順、`title`・`status`・`due` の3列にして保存
  - Then: 起動し直すと同じ3列・同じ並び・同じ2つの絞り込み(絞り込みは検索 navigation.search・同じ値の絞り込み navigation.same-value・式の絞り込み navigation.expr-filter・数字のキーで入れた条件 condition.digit-keys(いつも隠す条件を止めたことも含む。condition.hide-toggle)。並びは一時的な並べ替え navigation.header-sort も、保存したときのものを持つ)。

#### セッションで開いたときは、列の並びと隠す列を、screen.state-store の見た目の状態より先に当てるべき
`req~cli.session-content-columns-first~2` P1
- Acceptance:
  - Given:
  - When: 見た目の状態で `due` を隠した表を、`title`・`status`・`due` の3列のセッションで開く
  - Then: 3列が出る。
- Acceptance:
  - Given: セッションにあって今の表に無い列
  - When:
  - Then: その列だけ飛ばして警告。

#### セッションで開いたときは、表の行と値を、値をセッションに写さず、起動のたびに今のノートから作り直すべき
`req~cli.session-content-fresh-values~1` P1
- Acceptance:
  - Given:
  - When: セッション `進行中` を保存したあとノートの値を直して `mdgrid -s 進行中`
  - Then: あとから直したノートの値が出る。

#### セッションは、ノートのフォルダや `.base` に書かず(screen.state-separate・base-view.base-untouched・write-back.frontmatter-only と同じ理由)、設定の置き場(cli.config-file)の `sessions.toml` に名前ごとに置くべきで、人が手で書いてもよく、知らない項目は警告にとどめるべき
`req~cli.session-store~1` P1
- Acceptance:
  - Given:
  - When: 保存したあと、ノートのフォルダに新しいファイルが無く、`~/.config/mdgrid/sessions.toml` に `[sessions.進行中]` がある。手で `color = "red"` を足す
  - Then: 起動して警告。

#### 画面から、今の表示を名前を付けてセッションとして保存できるべき
`req~cli.session-screen-save~1` P1
- Acceptance:
  - Given:
  - When: 画面で今の表示を名前 `進行中` で保存
  - Then: sessions.toml に `[sessions.進行中]` ができる。

#### 画面からセッションを保存するときに同じ名前があれば、上書きしてよいか確かめるべき
`req~cli.session-screen-overwrite~1` P1
- Acceptance:
  - Given:
  - When: 同じ名前で2回保存
  - Then: 2回目に上書きの確認が出て、断ると元のまま。

#### `mdgrid --sessions` は、保存したセッションの名前と開くフォルダを1行ずつ出して終わるべき
`req~cli.session-screen-list~2` P1
- Acceptance:
  - Given: `--sessions`
  - When:
  - Then: `進行中	~/notes/tasks` の形の行が並び、終了コード 0。
- Acceptance:
  - Given: セッションが1つも無い
  - When:
  - Then: 何も出さず終了コード 0。

#### 無い名前・壊れた `sessions.toml`・セッションの開くフォルダが無いときは、cli.fail-fast と同じく理由を1行で出して終了コード 2 で終わり、どちらを開くか曖昧になる `-s` とパスの引数を一緒に渡されたときも同じにするべき
`req~cli.session-errors~2` P1
- Acceptance:
  - Given:
  - When: `mdgrid -s 無い`
  - Then: `セッション「無い」はありません(ある名前: 進行中)` の1行と終了コード 2。
- Acceptance:
  - Given:
  - When: `mdgrid -s 進行中 ~/notes`
  - Then: 理由1行と終了コード 2。

#### `--print` には、`--filter <式>`(base-view.expr-eval の式。何度でも渡せ、全部を満たす行だけ)を添えてよい
`req~cli.print-filter-sort-filter~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --print --filter 'status != "done"' --filter 'priority > 1'`
  - Then: done でなく priority が 1 より大きい行だけ。

#### `--print` には、`--sort <列>[:asc|:desc]`(何度でも渡せ、渡した順に効く。既定は昇順)を添えてよい
`req~cli.print-filter-sort-order~2` P1
- Acceptance:
  - Given: `--sort due`
  - When:
  - Then: 期日の昇順。
- Acceptance:
  - Given: `--sort priority:desc`
  - When:
  - Then: 優先度の高い順。

#### `.base` のビューに `--filter` を添えたときは、ビューの絞り込みと両方を満たす行にするべき
`req~cli.print-filter-sort-base-filter~1` P1
- Acceptance:
  - Given:
  - When: 絞り込みのある `.base` のビューに `--filter 'status != "done"'`
  - Then: `.base` の絞り込みと両方を満たす行だけ。

#### `.base` のビューに `--sort` を添えたときは、ビューの並べ替えの代わりにするべき
`req~cli.print-filter-sort-base-sort~1` P1
- Acceptance:
  - Given: `due` の昇順で並べる `.base` のビューに `--sort priority:desc`
  - When:
  - Then: ビューの並べ替えでなく優先度の高い順。

#### フォルダには、既定の表(base-view.source)と同じ列のまま、`--filter` と `--sort` を当てるべき
`req~cli.print-filter-sort-folder~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --print --filter 'status != "done"' --sort due`
  - Then: 既定の表と同じ列で、done でない行が期日の順。

#### `--filter` の評価できない式は、評価できないビュー(base-view.unsupported)と同じく理由1行と終了コード 2 にするべき
`req~cli.print-filter-sort-bad-expr~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `--filter 'foo('` → 理由1行と終了コード 2。

#### mdgrid のビュー(base-view.own-views)に `--filter`・`--sort` を添えたときは、理由1行と終了コード 2 にするべき
`req~cli.print-filter-sort-own-view~1` P1
- Acceptance:
  - Given:
  - When: views.toml の mdgrid のビューを `--view` で選んで `--filter 'status != "done"'`
  - Then: 理由1行と終了コード 2。

#### `mdgrid [<.base | フォルダ> …] --apply <ファイル>`(`-` で標準入力)は、`--print --with-path` の形の CSV か JSON(`path` の列とノートのキーの列。見出しは列の id か表示名)を読み、各行の `path` のノートの、値が今と違うセルだけを変えるべき
`req~cli.apply-changed-cells~1` P0
- Acceptance:
  - Given:
  - When: `--print --with-path --format json` の1行の status だけを done に直して `--apply --yes`
  - Then: そのノートの status のセルだけが変わり、ほかの行のノートのファイルは変わらない。

#### `--apply` は、既定では書かずに差分(write-back.pending-and-diff と同じ形)を標準出力に出すべき
`req~cli.apply-dry-run~1` P0
- Rationale: 書く前に差分を見られるようにするため
- Acceptance:
  - Given:
  - When: `--print --with-path --format json` の status を done に直して `--apply`
  - Then: 差分が出てファイルは変わらない。

#### `--apply` は、`--yes` を添えたときだけ、画面の保存と同じ道(write-back.byte-range から write-back.atomic-replace まで)で書くべき
`req~cli.apply-yes-write~1` P0
- Acceptance:
  - Given:
  - When: `--print --with-path --format json` の status を done に直して `--apply --yes`
  - Then: そのセルだけが書かれ、ほかのバイトは同じ。

#### `--apply` は、知らない `path`・書けないセル(cell-edit.read-only-forms・write-back.refuse-unsafe-notes)・列の型に合わない値が1つでもあれば、どれも書かずに終了コード 2 にするべき
`req~cli.apply-reject-all~1` P0
- Acceptance:
  - Given: 無い path
  - When:
  - Then: どれも書かず終了コード 2。

#### `--apply` は、知らない `path`・書けないセル・列の型に合わない値があれば、その全部の理由を標準エラーに出すべき
`req~cli.apply-reject-reasons~1` P1
- Acceptance:
  - Given: 無い path の行と、列の型に合わない値の行のある CSV で `--apply`
  - When:
  - Then: 標準エラーに2つの理由が出る。

#### `--apply` は、`path`・`file.*`・`formula.*` の列を読まないべき
`req~cli.apply-ignored-columns~1` P1
- Acceptance:
  - Given: `file.name` と `formula.x` の列の値を変えた CSV で `--apply --yes`
  - When:
  - Then: どのノートにも `file.name`・`formula.x` の値は書かれない。

#### `--apply` の CSV の空のセルは値を消す(cell-edit.empty-is-null)が、キーの無いノートにはキーを足さないべき
`req~cli.apply-empty-cell~1` P1
- Acceptance:
  - Given: status の無いノートの行の status のセルが空の CSV で `--apply --yes`
  - When:
  - Then: そのノートに status のキーが足されない。

#### `--apply` のリストの列の CSV のセルは、`, ` で要素に分けるべき
`req~cli.apply-list-cell~1` P1
- Acceptance:
  - Given: tags の列のセルが `a, b` の CSV で `--apply --yes`
  - When:
  - Then: そのノートの tags が a と b の2つの要素のリストになる。

#### `--apply` の行に `_rev` があり、そのノートの今の印(cli.print-rev)と違うときは、その行に今と違うセルが1つでもあれば、どれも書かずに終了コード 2 にし、その行が書き出したあとに変わったことと、書き出し直して直すことを理由に出すべき
`req~cli.apply-stale-row~1` P0
- Rationale: `--apply` は今の値と比べて違うセルを書くので、書き出したあとにほかの人が直したセルを、書き出したときの古い値で戻していた(2026-11-28 に本物で確かめた: 書き出したあとに due を 12-20 に直し、status だけを直した CSV を戻すと、due が 12-01 に戻る差分が出た)。どのセルを直したのかは書き出した印だけでは分からないので、行ごと止めて書き出し直してもらう
- Acceptance:
  - Given: `status: todo`・`due: 2026-12-01` のノートを `--print --with-path` で書き出したあと、外で due を 2026-12-20 に直した
  - When: 書き出した CSV の status を done に直して `--apply --yes`
  - Then: どれも書かず(status は todo、due は 2026-12-20 のまま)終了コード 2。標準エラーの理由に、そのノートの path と、書き出し直すことが出る。
- Acceptance:
  - Given: 同じく書き出したあとに外で due を直した
  - When: 書き出した CSV をそのまま `--apply --yes`
  - Then: 今と違うセルは due だけだが、書き出したときの古い値に戻すことになるので、どれも書かずに終了コード 2。
- Acceptance:
  - Given: 列が status だけのビュー(`.base`)で書き出したあと、外でビューに無い due を直した
  - When: 書き出した CSV をそのまま `--apply --yes`
  - Then: 今と違うセルが無いので、何も書かずに終了コード 0。
- Acceptance:
  - Given: 外で直していないノートを書き出した
  - When: status を done に直して `--apply --yes`
  - Then: status が done になり、ノートに `_rev` のキーは足されない。
- Acceptance:
  - Given: `_rev` の列の無い CSV(手で作った `path,status`)
  - When: `--apply --yes`
  - Then: 今までどおり、今と違うセルを書く。
- Needs: stest

#### よく使う表(フォルダか `.base`。`.base` と mdgrid のビューは名前でビューも)を、名前と1段の分類を付けて、設定のフォルダの `places.toml`(`[[place]]` の並び。`name`・`group`・`path`・`view`)に登録できるべき
`req~cli.places-store~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: パレットの「この表を登録」→ 名前「タスク」・分類「仕事」→ places.toml に `name = "タスク"`・`group = "仕事"`・`path`(実際のパス)。

#### コマンドのパレットの「この表を登録」で、今開いている表と今のビューを、名前(既定は今のビューかフォルダの名前)と分類(今ある分類から選ぶか打つ)を打って登録できるべき
`req~cli.places-palette~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `Tasks.base` のビュー「By owner」を開いて「この表を登録」→ 名前の既定が「By owner」で、今ある分類「仕事」を選んで登録すると、places.toml に `group = "仕事"`・`view = "By owner"` の行。

#### 「この表を登録」で同じ名前が既にあれば、確かめてから置き換えるべき
`req~cli.places-replace-confirm~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 名前「タスク」が登録済みで、同じ名前でもう一度「この表を登録」→ 置き換えの確かめ。

#### `places.toml` は、手で書いてもよい
`req~cli.places-hand-written~1` P1
- Acceptance:
  - Given: 手で `[[place]]` に `name = "読書"`・`group = "趣味"`・`path = "~/notes/Books"` を書いた places.toml
  - When:
  - Then: 登録した表の一覧の 趣味 に 読書 が出る。

#### `places.toml` の `path` の先頭の `~` は、ホームのフォルダとするべき
`req~cli.places-tilde~1` P1
- Acceptance:
  - Given:
  - When: `path = "~/notes/Books"` の行を開く
  - Then: ホームのフォルダの下の notes/Books の表。

#### `places.toml` の読めない行は、警告にして飛ばすべき
`req~cli.places-bad-line~1` P1
- Acceptance:
  - Given: `path` の無い行
  - When:
  - Then: 警告して飛ばす。

#### 表の登録では、ノートは書き換えないべき(読むだけの起動でも登録できてよい)
`req~cli.places-no-note-write~1` P1
- Acceptance:
  - Given: 「この表を登録」で登録
  - When:
  - Then: ノートのフォルダのファイルは変わらない。

#### 引数なしで起動して登録があるときと、表を開いたあとのパレットの「登録した表を開く」では、登録した表を分類ごとに(places.toml の順で)並べた一覧を出し、先頭に「今のフォルダ」(引数なしの起動のとき)を置くべき
`req~cli.places-list-order~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: places.toml に 仕事(タスク・朝会)と 趣味(読書)→ 引数なしの起動で `今のフォルダ`・`仕事`・`タスク`・`朝会`・`趣味`・`読書` の一覧。

#### 登録した表の一覧は、打った文字で名前・分類・パスを大文字小文字を区別せずに絞るべき
`req~cli.places-list-filter~1` P1
- Acceptance:
  - Given:
  - When: 仕事(タスク・朝会)と 趣味(読書)の一覧で「読」と打つ
  - Then: 読書だけ。

#### 登録した表の一覧で Enter を押すと、選んだ表を(ビューがあればそのビューで)開くべき
`req~cli.places-list-open~2` P1
- Acceptance:
  - Given:
  - When: 読書を選んで Enter
  - Then: ~/notes/Books の表。
- Acceptance:
  - Given:
  - When: 朝会を選んで Enter
  - Then: Tasks.base の By owner のビュー。

#### 表を開いたあとに別の表へ移るときは、ためた変更があれば、終わるとき(write-back.quit-confirm)と同じく保存か取りやめを確かめるべき
`req~cli.places-list-pending~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 表で未保存の変更があるときに「登録した表を開く」→ 保存の確かめ。

#### 登録した表の一覧の、無いパスの行は、理由を出して開かないべき
`req~cli.places-list-missing-path~1` P1
- Acceptance:
  - Given:
  - When: path のフォルダが無い行を選んで Enter
  - Then: 理由が出て、その表は開かない。

### 最近開いた表

#### 画面で開いた表(フォルダか `.base` と、そのとき見ていたビュー)を、新しい順に 20 件まで、ノートのフォルダではなく見た目の状態の置き場(screen.state-store)に覚えるべき
`req~cli.recent-record~2` P1
- Rationale: 登録(cli.places)やワークスペースに入れていない表にも、あとから戻れるようにするため(2026-11-20 に人が選んだ)
- Acceptance:
  - Given:
  - When: tasks を開き、projects に移り、また tasks を開く
  - Then: 最近の並びは tasks・projects の順で、tasks は1件だけ。
- Acceptance:
  - Given:
  - When: 21 個目の表を開く
  - Then: いちばん古い1件が消える。ノートのフォルダに新しいファイルは無い。
- Needs: stest

#### パレットの「最近開いた表」で、覚えた表を新しい順に(表の名前・置き場のフォルダ・最後に開いた日時を添えて)並べ、選んだ表を最後に見ていたビューで開くべき
`req~cli.recent-list~1` P1
- Rationale: 名前や置き場を思い出さずに、さっきまで見ていた表へ戻るため
- Acceptance:
  - Given: tasks(進行中)・projects の順に覚えている
  - When: 一覧の1行目が tasks → 選ぶ
  - Then: tasks の進行中。打った字で一覧を絞れる(cli.places-list-filter と同じ)。ためた変更があれば cli.places-list-pending と同じく確かめる。

#### 引数なしで起動して登録した表の一覧(cli.places-list-order)を出すときは、「今のフォルダ」の次に「最近」の分類を置き、最近開いた表を新しい順に 5 件まで並べるべき
`req~cli.recent-in-launch-list~2` P1
- Rationale: 起動してすぐ、昨日の続きの表を選べるようにするため
- Acceptance:
  - Given: places.toml に 仕事(タスク)、最近が projects・books
  - When:
  - Then: 引数なしの起動で `今のフォルダ`・`最近`・`projects`・`books`・`仕事`・`タスク` の一覧。
- Acceptance:
  - Given: 最近が無い
  - When:
  - Then: 「最近」の分類を出さない。

#### `mdgrid --recent` は、画面を出さずに、覚えた表を新しい順に「パス(タブ)ビューの名前(タブ)最後に開いた日時」の1件1行で出し、終了コード 0 で終わるべき
`req~cli.recent-cli~2` P1
- Rationale: シェルの道具(fzf など)で選んで `mdgrid <パス> --view <名前>` に渡せるようにするため
- Acceptance:
  - Given:
  - When: `mdgrid --recent`
  - Then: `/home/u/notes/tasks.base	進行中	2026-11-21 09:30` の形の行。
- Acceptance:
  - Given: 覚えた表が無い
  - When:
  - Then: 何も出さず終了コード 0。標準出力がパイプでも同じ。

#### 覚えた表のパスが無くなっていたら、一覧と `--recent` に出さず、覚えた並びから外すべき
`req~cli.recent-missing~1` P1
- Rationale: 開けない表を選ばせないため
- Acceptance:
  - Given:
  - When:
  - Then: 覚えた projects のフォルダを消してから「最近開いた表」→ projects が無い。

#### `--print-config --resolved` は、その表で効く名前の条件・通知の欄・知らせの項目も出し、各項目の上に出どころ(全体の設定なら `# config.toml`、印なら `# .mdgrid/workspace.toml (<ワークスペースの名前>)`)を添えるべき
`req~cli.resolved-config-ledger~1` P1
- Rationale: 台帳の印と全体の設定のどちらの条件と知らせが効いているかを、画面を開かずに確かめるため(2026-11-25 に人が決めた。condition.marker・notice.marker・notify.marker)
- Acceptance:
  - Given: 全体の設定に条件 due、名前が「人と会議」の印に条件 contact と `[[notices]]`
  - When: 印の台帳で `--print-config --resolved`
  - Then: `[conditions.contact]` と `[[notices]]` の上に `# .mdgrid/workspace.toml (人と会議)`、`[conditions.due]` の上に `# config.toml` がある。
- Acceptance:
  - Given: 同じ設定
  - When: 出した文を `--config` で読む
  - Then: 警告なし。
- Needs: stest

### 表を名前で選ぶ

#### `--table <名前>` は、開くフォルダ(無ければ今のフォルダ。`.md` のファイルならそのフォルダ)から上へたどって最初に見つかった印 `.mdgrid/workspace.toml` の、その名前の表(書いた表は `name`、自動の表はフォルダの名前。workspace.marker-open と同じ)を、印の `open` の表と既定の表の代わりに開くべき
`req~cli.table-open~1` P1
- Rationale: 印に表をいくつも書いた台帳で、`open` でない表を、画面で切り替えずに命令で選べるようにするため。`--print`・`--count`・`--check`・`--new-note` が `open` の表にしか効かず、ニュースを集める道具がニュースの表を使えなかった(2026-11-25 に調べ物・お金・読書の記録の台帳で見つかった)
- Acceptance:
  - Given: 印の根 `r` に `open = "問い"` と、表「問い」(絞り込み `file.inFolder("Questions")`)と表「ニュース」(絞り込み `file.inFolder("News")`)
  - When: `mdgrid r --table ニュース --print --with-path`
  - Then: `r/News` のノートの行だけが出て(`r/Questions` のノートの行は無い)、終了コード 0。
- Acceptance:
  - Given: 同じ台帳で、今のフォルダが `r/Questions`
  - When: `mdgrid --table ニュース`(画面)
  - Then: 表「ニュース」のビューのタブで開き、News のノートの行が出る。
- Needs: stest

#### `--table` は、表を使う命令(画面・`--pick`・`--print`・`--count`・`--check`・`--notices`・`--notify`・`--runs`・`--apply`・`--print-config --resolved`・`--page`)のどれでも、選んだ表を開いた表として効かせるべきで、`--view` はその表のビューから探し、`--view` が無ければ印の `open` で開いたときと同じくその表の `view`(無ければ最初のビュー)にするべき
`req~cli.table-commands~1` P1
- Rationale: 画面と命令で、同じ名前が同じ表を指すようにするため。`--print --view 次の支払いの順` が `open` の表のビューだけを探して「ビューは無い」になった(2026-11-25 にお金の台帳で見つかった)
- Acceptance:
  - Given: 印の根 `m` に `open = "支出"` と表「支出」と、ビュー「全部」「次の支払いの順」(`status` が active の行)の順の表「定期の支払い」
  - When: `mdgrid m --table 定期の支払い --print --view 次の支払いの順`
  - Then: 定期の支払いの active の行だけが出て、終了コード 0。
- Acceptance:
  - Given: 同じ台帳
  - When: `mdgrid m --table 定期の支払い --count`
  - Then: 定期の支払いの最初のビュー「全部」の行の数。
- Acceptance:
  - Given: 同じ台帳で、定期の支払いのノートに `[types]` の日付の列の型に合わない値
  - When: `mdgrid m --table 定期の支払い --check`
  - Then: その問題の行が出て、終了コード 1。
- Acceptance:
  - Given: 同じ台帳で、表「定期の支払い」にだけ `[table.rules.status] values`
  - When: `mdgrid m --table 定期の支払い --print-config --resolved`
  - Then: その `values` が出る。
- Needs: stest

#### `--new-note` に `--table <名前>` を添えたら、その表の決まり(作る場所の `[new_note] folder`・名前の頭・前もって入れる値・本文の雛形・必須の列)で印の根からの場所に作り、作ったパスを、渡したフォルダ(無ければ今のフォルダ)から見た道で出すべき
`req~cli.table-new-note~1` P1
- Rationale: ニュースを集める道具が、どのフォルダから呼んでも、ニュースの表の決まり(名前の日付の頭・雛形・既定の値)でノートを足せるようにするため
- Acceptance:
  - Given: 今日が 2026-11-25、印の根 `r` の表「ニュース」に `[table.new_note]` の `folder = "News"`・`name = "{date:YYYYMMDD}_"`・`name_head = "hide"`・`body`(雛形 `# {name}`)と `[table.new_note.set] date = "{date}"`
  - When: `mdgrid r --table ニュース --new-note 見出し`
  - Then: `r/News/20261125_見出し.md` ができ、フロントマターに `date: 2026-11-25`、本文は `# 見出し`、標準出力は `r/News/20261125_見出し.md` の1行。
- Acceptance:
  - Given: 同じ台帳で、今のフォルダが `r/Questions`
  - When: `mdgrid --table ニュース --new-note 別`
  - Then: `r/News/20261125_別.md` ができ、標準出力は `../News/20261125_別.md` の1行。
- Needs: stest

#### `--table` の名前の表が無いとき(開くフォルダから上へたどって印が無い・印の表のどれの名前でもない)は、何も開かず書かずに、理由1行(印の表があればその名前を並べる)と終了コード 2 で終わるべき
`req~cli.table-unknown~1` P1
- Acceptance:
  - Given: 表「問い」と表「ニュース」の印の根 `r`
  - When: `mdgrid r --table 無い --print`
  - Then: 標準出力は空で、標準エラーの理由1行に `問い` と `ニュース` があり、終了コード 2。
- Acceptance:
  - Given: 同じ台帳
  - When: `mdgrid r --table 無い --new-note x`
  - Then: 理由1行と終了コード 2 で、`r` の下のどこにも `x.md` ができない。
- Acceptance:
  - Given: 上へたどっても印の無いフォルダ `plain`
  - When: `mdgrid plain --table ニュース --print`
  - Then: 印が無いことを言う理由1行と終了コード 2。
- Needs: stest

#### `--table` を `--raw`・`-s`・`--continue` と一緒に渡したとき、開くパスを2つ以上添えたとき、`-w` と開くパスの両方と一緒に渡したときは、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~cli.table-bad-args~1` P1
- Rationale: `--raw`(印を使わない)・セッションと `--continue`(開く表を自分で持つ)・2つ目のパスは、どの表を開くかが食い違うため
- Acceptance:
  - Given: 印の根 `r`
  - When: `mdgrid r --table ニュース --raw --print`・`mdgrid r r/News --table ニュース --print`・`mdgrid -s 朝 --table ニュース`・`mdgrid --continue --table ニュース`
  - Then: どれも理由1行と終了コード 2。
- Needs: stest

#### `-w <名前>` と一緒の `--table` は、`workspaces.toml` のそのワークスペースの表(`[[workspace.table]]` の `name`)から選ぶべき
`req~cli.table-workspace~1` P1
- Rationale: `-w` で範囲を決めたときも、表を同じ旗で選べるようにするため
- Acceptance:
  - Given: `workspaces.toml` のワークスペース「仕事」に、表「a」(フォルダ `a`)と表「b」(フォルダ `b`)
  - When: `mdgrid -w 仕事 --table b --print --with-path`
  - Then: `b` のノートの行だけが出る。
- Acceptance:
  - Given: 同じワークスペース
  - When: `mdgrid -w 仕事 --table c --print`
  - Then: `a` と `b` を並べた理由1行と終了コード 2。
- Needs: stest

#### 印の根の下の、印の表のノートを置くフォルダ(その表の `[new_note] folder` か、表の `.base` の絞り込みの `file.inFolder` が指すフォルダ。cli.new-note-table-folder と同じ)を開いたら、画面でも画面を出さない起動でも、そのフォルダの素の表でなくその表を開くべき(`--raw` のときは素の表)
`req~cli.launch-table-folder~1` P1
- Rationale: 表のフォルダを直に開くと、その表の決まり(新しいノートの名前の頭・雛形・ビュー・値の候補)が効かない素の表になり、画面で作るノートの形が `--new-note`(cli.new-note-table-folder)とそろわなかった(2026-11-25 に調べ物の台帳で見つかった)
- Acceptance:
  - Given: 印の根 `r` の表「ニュース」(絞り込み `file.inFolder("News")`、ビュー「未判定」「全部」)
  - When: `mdgrid r/News`(画面)
  - Then: 表「ニュース」のタブ「未判定」「全部」で開く。
- Acceptance:
  - Given: 同じ台帳
  - When: `mdgrid r/News --print --with-path`
  - Then: `mdgrid r --table ニュース --print --with-path` と同じ行。
- Acceptance:
  - Given: 同じ台帳で、どの表のフォルダでもない `r/Other`
  - When: `mdgrid r/Other --print --with-path`
  - Then: `r/Other` の既定の表(今までどおり)。
- Acceptance:
  - Given: 同じ台帳
  - When: `mdgrid r/News --raw --print --with-path`
  - Then: `r/News` の既定の表(表の絞り込みとビューを当てない)。
- Needs: stest

## 未確定事項

### `--new-note` に `--view <名前>` を添えたとき、そのビューの絞り込みで値が1つに決まる列(cell-edit.new-note-filter-values)と、ビューごとの新しいノートの設定(cell-edit.new-note-form-per-view)を使うか
`tbd~cli.new-note-view-values`
- Decider: 人
- Provisional: 使わない(`--new-note` は台帳の `[new_note]` だけを見る。`--view` を添えても変わらない)

### `--new-note` の名前を省いたとき、設定の名前の雛形(`[new_note] name = "{date}"`)から名前を作るか
`tbd~cli.new-note-name-template`
- Decider: 人
- Provisional: 作らない(名前は必ず渡す。空の名前は理由1行と終了コード 2)

### `--table` の表の名前を、シェルの補完の候補に出すか
`tbd~cli.table-completion`
- Decider: 人
- Provisional: 出さない(補完の定義は引数の定義から前もって作る文(cli.completions-man-from-args)で、台帳ごとに変わる表の名前を出すには、補完のたびに mdgrid を呼ぶ形に作り直す必要があるため。名前を間違えたときは、理由の1行に表の名前が並ぶ(cli.table-unknown))

### `--check` で `--view` が無いとき、いつも隠す条件(condition.hide-everywhere)を当てるか
`tbd~cli.check-hidden-rows`
- Decider: 人
- Provisional: 当てる(いつも隠す行は、画面にも `--print` にも `--count` にも無い行なので、確かめる行からも外す。隠した行も確かめたいときは条件を外す)
