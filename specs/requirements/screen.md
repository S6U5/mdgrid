# 画面と操作(screen) の要求

## 目的

表を見て直すための画面。表の TUI への不満の中心は「機能が足りない」より「あるのに見つからない」と「日本語や端末の違いで崩れる」なので、見つけやすさ(下の帯・ヘルプ・パレット)と崩れない描画を、機能より先に決める。cellops(自作のタスクの TUI)で作り直しを重ねて固まった形のうち、タスクに依らない部分も採る。表の中身は [base-view](base-view.md)、移動と検索は [navigation](navigation.md)、セルの値・文字の幅・色の見せ方は [cell-view](cell-view.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/screen/spec.md](../_history/decidespec/screen/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 旧い SR-7(新しい名前なし)は 2026-09-30 に削除([2026-09-30](../_history/decidespec/_decisions/2026-09-30-v1-review-fixes.md))。移行先: screen.keys-vim-and-arrows。
- 次の段: 端末のダーク / ライトの自動判定でテーマを選ぶこと(テーマは screen.themes のとおり設定で選ぶ)。
- 採らない: 画面の型の追加(タスクの画面などは scope の scope.screen-types)。
- 採らない: スクリーンリーダーへの完全な対応(端末の表示の仕組みの限界。カーソルの位置を安定させ、再描画を減らす範囲で配慮する)。

## 前提(旧い仕様で AI が置いた仮定)

- 構成・2段階のクリック・キーの表・ヘルプの形は cellops の仕様(layout-v1・tasklist-v2・help-v2・pane-api-v1)から、タスクに依らない部分を採った。
- 見た目の状態の置き場所は、OS の状態用のフォルダ(XDG の state 等)を想定する(docs/design.md)。
- キーの流儀(screen.keys-vim-and-arrows)は人が選んだ(両方の流儀と Bases の主なキー、すべて割り当て直せる)。
- screen.default-keys の Esc は、選択があれば選択を、無ければ簡易の絞り込みを解く。状況で動作が変わるが、どちらも「今かかっている一時的なもの」を解く1つの動作で、戻せないものを失わない(key-defaults で却下した「読み込み中だけ Ctrl+C を中止にする」は、コピーのつもりで読み込みが止まる点で違う。ime-keys-marks)。
- screen.default-keys の Ctrl+A は GNU screen の接頭キーと重なり、screen の中では mdgrid に届かない。screen.keys-vim-and-arrows のとおり割り当て直すか、screen の側の `C-a a` で送る。

## 要求

### 構成

#### 画面は上から、上の余白・ヘッダー・ビューの帯・設定の帯(条件が効いているとき。navigation.settings-bar)・表・下の帯・メッセージ行で構成するべき
`req~screen.layout-order~1` P1
- 受け入れ: 80×24 の端末で起動 → 1行目は空いた上の余白、2行目にヘッダー(screen.breadcrumb。範囲と表のパンくずと行数、右の端に画面の型のメニュー(relations.screen-menu)と設定のボタン)、3〜5行目にビューの帯(navigation.search-box。3行目と5行目は空き行、4行目は左にタブ、右に検索の欄と絞り込み・並べ替え・分類・列のボタン、右の端に作れるときは「+ 新規」(cell-edit.new-note))、6行目から表(ゴールデン)。絞り込みが効いているとき → ビューの帯と表のあいだに設定の帯の行が入る。

#### 上の余白は、設定 `display.top_padding` で 0〜2 を 0.5 刻みで選べ、既定を 1 にするべき
`req~screen.layout-top-padding~1` P1
- なぜ: navigation.search-box と同じく、環境で見え方がぶれないようにするため
- 受け入れ: `display.top_padding = 0` と `display.bar_padding = 0` → 1行目にヘッダー、2行目にビューの帯、3行目から表。`display.top_padding` を書かない → ヘッダーの上に空き行が1行。

#### 上の余白の端数は、ビューの帯の余白(navigation.search-box)と同じく半分の塗りの字で描き、そのときはヘッダーの行にもビューの帯と同じ地の色を敷くべき
`req~screen.layout-half-padding~1` P1
- 受け入れ: 端末の地の色が分かる色ありの表示で `display.top_padding = 0.5` → 1行目は上の余白の半行(`▀`)、2行目のヘッダーの行の地はビューの帯と同じ地の色。

#### 下の帯には、今のモード・選んだセルの位置・未保存の数・今押せるキーを出すべき
`req~screen.layout-bottom-bar~1` P1
- 受け入れ: 80×24 の端末で起動 → 下の帯に今のモード、選んだセルの位置、「未保存 0」、今押せるキーが並ぶ(ゴールデン)。

#### ビューの帯のタブと検索の欄、設定の帯は、screen.display-options で隠せる
`req~screen.layout-hide-parts~1` P1
- 受け入れ: タブを出し、検索の欄だけ隠す(`search_bar = false`) → ビューの帯の行は残り、タブとボタンが並び、検索の欄は無い。

#### タブと検索の欄を両方隠せば、ビューの帯の行ごと出さない(そのときの「+ 新規」はヘッダーの画面の型のメニューの左。cell-edit.new-note)
`req~screen.layout-no-view-bar~1` P1
- 受け入れ: タブ(`tabs = "never"`)と検索の欄(`search_bar = false`)を両方隠す → ビューの帯の行もボタンも出ず、ヘッダーのすぐ下の行が列の見出し。

#### `groupBy` のまとまりは、見出しの行で開閉できるべき
`req~screen.group-fold~1` P1
- 受け入れ: 見出しで Enter → そのまとまりの行が畳まれる。

#### 列は横にスクロールでき、選んだセルが常に見えているべき
`req~screen.h-scroll-follow~1` P1
- 受け入れ: 列が画面に収まらない表で右へ移動 → 選んだセルが見える位置まで表が流れる。

#### 列の幅は、列の見出しの境界のドラッグで変えてよい
`req~screen.h-scroll-drag-width~1` P1
- 受け入れ: 列の見出しの右の境界を押して右へ5桁ドラッグ → その列の幅が5桁広がる。

#### 列の幅は、表の行のセルの右の境界のドラッグでも変えてよい
`req~screen.cell-drag-width~1` P1
- なぜ: 見出しまで手を戻さずに、読んでいるセルのそばで幅を直せるように
- 受け入れ: 表の2行目で title の列のセルの右の境界を押して右へ5桁ドラッグ → title の列の幅が5桁広がる。境界を押して動かさずに離す → 幅も、選んでいる行と列も変わらない。

#### 表の左のノートの欄は、末尾の `.md` を除いた名前(screen.note-name の `display.note_name` があればそれで抜き出すか削ったもの)を、設定 `display.note_path`(screen.note-path)に従ったフォルダとともに出すべき
`req~screen.note-column-path~1` P1
- 受け入れ: `Tasks/a.md`・`Tasks/b.md` だけの表 → 欄は `a`・`b`(`"common"`(既定)は読み込んだ全部の行に共通のフォルダを除いたパス)。`Tasks/a.md`・`Notes/c.md` → `Tasks/a`・`Notes/c`。`note_path = "name"`(フォルダを除いた名前だけ)で `todo/a.md`・`archive/todo/b.md` → `a`・`b`。`note_path = "full"`(根からのパス)で `Tasks/a.md`・`Tasks/b.md` → `Tasks/a`・`Tasks/b`。

#### 表の列に `file.name` があるときと `display.note_path = "off"` のときは、ノートの欄に名前と見出しを出さず、行の印(`!`・`~`)だけを出すべき
`req~screen.note-column-marks-only~1` P1
- なぜ: `"off"` は題名の列で行が分かる表のため
- 受け入れ: `note_path = "off"` で `todo/20261015_a.md` → 欄に名前も見出しも出ず、外で変わった行には `!`。order に `file.name` のある `.base` → 欄に名前も見出し「ノート」も出ず、外で変わった行には `!` が出る。

#### ノートの欄に名前を出さないとき(`file.name` の列があるときと `"off"`)も、親子の字下げと ▾・▸、WBS の番号と進み具合(navigation.parent-child・navigation.wbs)は欄に残すべき
`req~screen.note-column-keeps-tree~1` P1
- 受け入れ: 親子で並べた表で `"off"` → ▾・▸ と字下げは残る。

#### 検索(navigation.find)と、メッセージ・詳しく見る画面のノートの名前は、`display.note_path` によらず根からのパスのままにするべき
`req~screen.note-column-full-path-elsewhere~1` P1
- 受け入れ: `"name"` でも `\` の検索で `archive` と打つと `archive/todo/b.md` の行が当たる。

### 操作と見つけやすさ

#### キーは、vim 風(`h` `j` `k` `l`・`/`・`n` `N`・`gg` `G`)と、矢印・Home / End・PageUp / PageDown の両方が効かなければならない
`req~screen.keys-vim-and-arrows-both~1` P1
- 受け入れ: 表で `j` を押したときと ↓ を押したとき → 同じ動き。

#### Obsidian Bases の主なキー(Enter・Tab / Shift+Tab・Ctrl+C / Ctrl+Z・Backspace)も効かなければならない
`req~screen.keys-vim-and-arrows-bases~1` P1
- 受け入れ: 表で Enter → 選んだセルの編集、Tab / Shift+Tab → 右 / 左のセル、Ctrl+C → コピー、Ctrl+Z → 直前の変更の取り消し、Backspace → 選んだセルを空にする。

#### すべてのキーは、設定ファイルで割り当て直せなければならない
`req~screen.keys-vim-and-arrows-remap~1` P1
- 受け入れ: 設定で `j` を外す → ↓ だけが効く。

#### キーの定義は1か所の表に置き、キーの振り分け・下の帯・ヘルプを、すべてその表から作るべき
`req~screen.key-table~1` P1
- 受け入れ: 表にキーを1つ足す → 振り分け・下の帯・ヘルプに同時に出る。

#### 既定のキーの割り当てと、割り当て直し(screen.keys-vim-and-arrows)に書く動作の名前の一覧を、英語(`docs/keys.md`)と日本語(`docs/keys.ja.md`)の文書に置くべき
`req~screen.keys-docs-files~1` P1
- 受け入れ: `docs/keys.md` と `docs/keys.ja.md` → どちらにも既定のキーと動作の名前の一覧がある。文書の動作の名前を設定の `[keys.table]` に書く → その動作に割り当たる。

#### キーの文書は、モードごと(設定の `[keys.<モード>]` の名前)に、キー・動作の名前・説明を並べるべき
`req~screen.keys-docs-by-mode~1` P1
- 受け入れ: `docs/keys.ja.md` → `table` などモードの名前の節ごとに、キー・動作の名前・説明を並べた行があり、説明が空の行は無い。

#### キーの表(screen.key-table)のモード・キー・動作と、動作の名前の一覧が文書と食い違えば、試験で落ちるべき
`req~screen.keys-docs-sync~1` P1
- なぜ: 文書が古くなるのを防ぐため
- 受け入れ: キーの表に `table` の `x` → 何かの動作を足して文書を直さない → 試験が落ちる。文書から1行消す → 落ちる。

#### 表のモードで、選んでいるセルに対して今使える操作の一覧(メニュー)を、キー(既定 `x`。screen.key-table のキーの表で割り当て直せる)で開けるべき
`req~screen.action-menu-open-key~1` P1
- 受け入れ: status のセルで `x` → 操作の一覧が開く。`[keys.table] "m" = "action_menu"` → `m` で開く。

#### 表のモードで、セルの右クリックで、そのセルに対して今使える操作の一覧を開けるべき
`req~screen.action-menu-right-click~1` P1
- 受け入れ: セルを右クリック → そのセルを選んで一覧が開く。

#### 操作の一覧は、セル・列・行・ビューとファイルの節に分けるべき
`req~screen.action-menu-sections~1` P1
- 受け入れ: status のセルで2行に印を付けて `x` → 「セル」「列」「行」「ビューとファイル」の節の一覧が出る(セル: 編集・空にする・コピー・エディタで開く・詳細。列: この列で並べ替え・同じ値で絞る・同じ値を強調・隠す・幅・左右に移す。行: 印・選んだ行にまとめて入れる・行をコピー。ビューとファイル: 新しいノート・ビューの設定・`.base` に書き出す・保存)。

#### 操作の一覧の各行には、動作の名前と、その動作の今のキー(割り当て直したあとのキー)を添えるべき
`req~screen.action-menu-row-keys~1` P1
- 受け入れ: status のセルで `x` → 「編集  Enter」「この列で並べ替え  s」が並ぶ。

#### 操作の一覧には、その場で使えない操作(読むだけのセルの編集と空にする、印の無いときのまとめて入れる、読むだけの起動での保存と新しいノートなど)を出さないべき
`req~screen.action-menu-hide-unavailable~1` P1
- 受け入れ: 読むだけのセルで `x` → 「編集」「空にする」が無い。

#### 操作の一覧では、↑↓(`j` `k`)で選んで Enter で実行するべき
`req~screen.action-menu-enter~1` P1
- 受け入れ: 書けるセルで `x` のあと `j` 6回・`k` 1回 → 「この列で並べ替え」が選ばれ、Enter → その列で並べ替わる。

#### 操作の一覧は、Esc で閉じるべき
`req~screen.action-menu-esc~1` P1
- 受け入れ: 一覧で Esc → 何もせず閉じる。

#### 操作の一覧で、行に添えたキーを押しても、その操作を実行するべき
`req~screen.action-menu-item-key~1` P1
- 受け入れ: 一覧で `s` → 「この列で並べ替え」を実行し、`s` と同じに並ぶ。

#### 操作の一覧は表の上に重ねる窓で出し、窓の外の表は見えたままにするべき
`req~screen.action-menu-overlay~1` P1
- 受け入れ: `x` で一覧を開く → 窓の行のうち窓より左の桁に、開く前と同じ表の文字が残る。

#### 操作の一覧の項目をクリックしたら、その操作を実行するべき
`req~screen.action-menu-click-item~1` P1
- 受け入れ: 一覧の「この列で並べ替え」を左クリック → `s` と同じに並ぶ。

#### 操作の一覧の窓の外をクリックしたら、一覧を閉じるべき
`req~screen.action-menu-click-outside~1` P1
- 受け入れ: 一覧の外(画面の左上の角)を左クリック → 一覧が閉じる。

#### 操作の一覧の窓が入らない低い端末では、一覧を開かずに理由を出すべき
`req~screen.action-menu-too-low~1` P1
- 受け入れ: 80×8 の端末で `x` → 一覧は開かず、メッセージ行に理由が出る。

#### 操作の一覧の「選んだ行にまとめて入れる」は、選んだ行のうち今の列を直せる行が1つも無ければ出さないべき
`req~screen.action-menu-bulk-hidden~1` P1
- 受け入れ: 式の列(読むだけ)のセルで2行に印を付けて `x` → 「選んだ行にまとめて入れる」が無い。

#### 操作の一覧からの実行は、キーで直接行ったときと同じ振る舞いにするべき
`req~screen.action-menu-same-as-key~1` P1
- 受け入れ: 「この列で並べ替え」で Enter → `s` と同じに並ぶ。

#### 前置きのキー(`g g` の `g` のように、キーの表で2つ以上の打鍵の頭になるキー。設定で足したものも含む)を押したら、すぐに下の帯を、続けて押せるキーと動作の名前の並びに替えるべき
`req~screen.prefix-hints-show~1` P1
- 受け入れ: 表で `g` → 下の帯が「g → g 先頭の行」になる。設定で `"z z" = "bottom"` を足す → `z` で「z → z 末尾の行」。

#### 前置きのあと、続きを押すか Esc を押したら、下の帯を元に戻すべき
`req~screen.prefix-hints-restore~1` P1
- 受け入れ: 表で `g` のあと続けて `g` → 先頭の行へ動き、帯が元に戻る。`g` のあと Esc → 動かず帯が戻る。

#### 前置きのあとの Esc は前置きを取り消すだけにし、選択の解除や一覧を閉じることはしないべき
`req~screen.prefix-hints-esc-only~1` P1
- 受け入れ: 2行に印を付けて `g` のあと Esc → 前置きだけが取り消され、2行の印は残る。操作の一覧を開いて `g` のあと Esc → 一覧は開いたまま。

#### 操作の一覧の上では、一覧の項目に続く前置きだけを受けるべき
`req~screen.prefix-hints-menu~1` P1
- 受け入れ: 設定で `[keys.table] "z z" = "bottom"` を足し、操作の一覧の上で `z` → 末尾の行は一覧の項目に無いので、一覧のままで、帯に「z →」が出ない。

#### `?` でヘルプを重ねて出し、先頭に今押せるキー、その下に全部のキーを並べ、色だけで意味を伝えない cell-view.not-color-only の印は見ただけでは分からないので、最後にセルと行の印(`∅`・`""`・`!`・`*`・`#`・`?`・`…⏎`、行の左の `>`・`+`・`!`・`~`)の意味を並べるべき
`req~screen.help~1` P1
- 受け入れ: 表で `?` → 末尾の「印」の節に `∅` と `#` の意味。表で `?` → 先頭に表の操作。

#### コマンドの名前であいまいに探して実行できるパレット(`:` と Ctrl+P)を持つべき
`req~screen.palette-fuzzy~1` P1
- 受け入れ: `:` で「save」と打つ → 保存が候補の先頭に出る。Ctrl+P で開いて「save」と打つ → 同じ。

#### パレットには、各コマンドの割り当てキーも出すべき
`req~screen.palette-keys~1` P1
- 受け入れ: `:` で「save」と打つ → 保存の候補の横に Ctrl+S。

#### 選んでいない行をクリックしたら、その行を選ぶだけにするべき
`req~screen.mouse-click-select~1` P1
- 受け入れ: 別の行のセルを1回クリック → 選ぶだけで編集は始まらない。

#### 選んだ行のセルをクリックしたら、そのセルのその場の編集を始めるべき
`req~screen.mouse-click-edit~1` P1
- 受け入れ: 選んだ行の status のセルをクリック → そのセルの編集が始まる。

#### ホイールで上下に動けるべき
`req~screen.mouse-wheel~1` P1
- 受け入れ: ホイールの下・上 → 次・前の行。

#### 表の画面では、横のホイール(トラックパッドの横のスワイプ)と shift を押したホイールで、`h`・`l` と同じに左右の列へ動けるべき
`req~screen.mouse-h-wheel~1` P1
- なぜ: 縦しか拾わないと、収まらない列を指で流せないため
- 受け入れ: 表で横のホイールを右 → 右の列、左 → 左の列、一番左で左 → そのまま。shift+ホイールの下 → 右の列、上 → 左の列。列が画面に収まらない表で右へ流し続ける → 選んだセルが見える位置まで表が流れる(screen.h-scroll)。本物の実行ファイルに端末の横のスクロールの報告(SGR の 67)を送る → 右の列。

#### 表の画面でない画面(ヘルプなど)では、横のホイールで何もしないべき
`req~screen.mouse-h-wheel-other~1` P1
- 受け入れ: ヘルプで横のホイール → 何も変わらない。

#### 表の画面でない画面(ヘルプなど)では、shift を押したホイールを縦のホイールのままにするべき
`req~screen.mouse-shift-wheel-other~1` P1
- 受け入れ: ヘルプで shift+ホイールの下 → ヘルプが下へ流れる。

#### 選んだ行のノートをエディタで開けるべき
`req~screen.editor-open~1` P1
- 受け入れ: 選んだ行で `e` → そのノートのパスでエディタが起動する。

#### エディタから戻ったら、開いたノートを読み直すべき
`req~screen.editor-reload~1` P1
- 受け入れ: `EDITOR="code -w"` で開く → 戻ると行が更新。

#### エディタは、設定(cli.config-file の設定ファイル)の `editor`、環境変数 `$VISUAL`、`$EDITOR` の順で空でない最初のものにするべき
`req~screen.editor-choice~1` P1
- 受け入れ: 設定に `editor = "nvim"` と `EDITOR=vim` → nvim で開く。設定が無く `VISUAL=hx`・`EDITOR=vim` → hx。設定の `editor = ""` は無いのと同じ。

#### 設定の `editor`・`$VISUAL`・`$EDITOR` のどれも無ければ、エディタを `vi`(Windows では `notepad`)にするべき
`req~screen.editor-fallback~1` P1
- なぜ: Windows では vi が入っていないのがふつうのため
- 受け入れ: どれも無い → Unix では vi、Windows では notepad(OS ごとの既定は文字の値 `"vi"`・`"notepad"` で比べる)。

#### エディタの値が引数つき(`code -w` など)でも動くべき
`req~screen.editor-args~1` P1
- 受け入れ: `EDITOR="code -w"` → `code` に `-w` とノートのパスを渡して起動する。

#### エディタは、シェルを通さずに起動するべき
`req~screen.editor-no-shell~1` P1
- 受け入れ: `EDITOR="ed $HOME;"` → `ed` に `$HOME;` を展開せずにそのまま1つの引数で渡す。

#### ノートのパスは、エディタに1つの引数として渡すべき
`req~screen.editor-path-one-arg~1` P1
- 受け入れ: 空白を含む `/tmp/my notes/a b.md` のノートを開く → パスは分けずに1つの引数で渡る。

#### キーは、モード(表・編集・パレット・保存の確認・終了の確認・ヘルプ・検索・絞り込み・詳細の表示・ビューの設定・設定の値の入力・設定の帯・リストの選択・操作の一覧(screen.action-menu)・頻度表(navigation.frequency))ごとに割り当てるべき
`req~screen.key-modes-per-mode~1` P1
- 受け入れ: 同じ Enter → 表ではセルの編集、詳細の表示では選んだプロパティの編集。

#### 同じモードの中で、1つのキーに2つの動作を割り当てるべきでない
`req~screen.key-modes-no-dup~1` P1
- 受け入れ: キーの表をモードごとに調べて重複が無い。

#### 表のモードの既定のキーは、Enter = 編集(グループの見出しの行では開閉、チェックボックスのセルでは切り替え)、Space = 行の印、`K` = 詳細の表示、Ctrl+R = やり直しにするべき
`req~screen.key-modes-table-defaults~1` P1
- 受け入れ: 表で Enter → セルの編集、Space → 行の印、`K` → 詳細の表示、Ctrl+R → やり直し。

#### 編集のモードでは、Ctrl+R の既定を、編集前の値に戻す動作にするべき
`req~screen.key-modes-edit-defaults~1` P1
- 受け入れ: status が `todo` のセルの編集で `done` と打ち替えて Ctrl+R → 入力が `todo` に戻る。

#### 詳細の表示のモードの既定のキーは、Enter = 選んだプロパティの編集(navigation.detail)、Esc と `K` = 閉じるにするべき
`req~screen.key-modes-detail-defaults~1` P1
- 受け入れ: 詳細の表示で Enter → 選んだプロパティの編集、Esc で閉じる。`K` でも閉じる。

#### パレット(`:` で開く)は、行番号(`:120`)と `w`(保存)と `q`(終了)と `wq`・`x`(保存して終了)も受け付けるべき
`req~screen.key-modes-palette-commands~2` P1
- 受け入れ: `:120` → 120行目が選ばれる。パレットの `w` → 保存(差分の確認へ)、`q` → 終了(ためた変更があれば終了の確認)。未保存 1 でパレットの `wq` → 差分の確認が出て、確定で書いて終わる(差分の確認から戻ると、書かず終わらず表に戻る)。`x` も同じ。ためた変更が無いときの `wq` → そのまま終わる。読むだけの起動(cli.readonly)では `wq`・`x` は `w` と同じく候補に出ない。

#### 表のモードで、日本語入力(IME)が有効なまま打った全角の英数字と記号が届いたら、半角のキーとして扱うべき
`req~screen.ime-fullwidth~1` P1
- 受け入れ: 表で全角の ｊ を送る → j と同じ動き。

#### 表のモードで、日本語の読点と中黒(`、`・`・`)は、同じキーの位置の `,`・`/` として扱うべき
`req~screen.ime-punct~1` P1
- 受け入れ: 表で `、` を送る → `,`(screen.default-keys の同じ値の行だけに絞る)と同じ動き。`・` → `/`(検索)と同じ動き。

#### 入力ボックスでは、端末のカーソルを入力の位置に置き、変換の窓が入力の位置に出るようにするべき
`req~screen.ime-cursor~1` P1
- 受け入れ: 検索の欄に「会議」と打つ → 端末のカーソルは「会議」のすぐ後ろ(欄の語の末尾)にある。

#### 表のモードで、ほかの要件で既定のキーを決めていない動作の既定は、Tab / Shift+Tab = 右 / 左のセルへ移る、Backspace と Delete = 選んだセルの値を空にする(ためる変更。cell-edit.empty-is-null。キーの無いセルでは何もせず、キーを足さない)、`q` = 終わる(write-back.quit-confirm)、Ctrl+A = 全部の行を選ぶ、Esc = 選択を解く(選択が無ければ検索を解く。navigation.select-rows・navigation.search)、`*` = 選んだセルと同じ値の行を強調する、`,` = 同じ値の行だけに絞る(navigation.same-value)、Ctrl+G = 読み込みの中止(base-view.large-load)とするべき
`req~screen.default-keys~1` P1
- 受け入れ: 表で Backspace → そのセルが null のためる変更になり「未保存 1」。読むだけのセルでは理由を表示して何もしない。キーの無いセルで Backspace → 何も変わらず「未保存 0」。読み込み中に Ctrl+C → コピーになり、読み込みは止まらない。

#### 行のファイル名(左の行の名前の欄と、`file.name` の列のセル)をクリックしたとき、選んでいない行なら選ぶだけにし(screen.mouse)、選んだ行なら `e`(screen.editor)を押したのと同じ振る舞いで、そのノートをエディタで開くべき
`req~screen.click-name~1` P1
- 受け入れ: 別の行のファイル名を1回クリック → その行を選ぶだけで開かない。選んだ行のファイル名をクリック → エディタが起動し、戻るとその行を読み直す。`EDITOR` が空のときと読むだけの起動のときも、`e` と同じ結果。

### 設定と見た目の状態

#### 設定の `display.tabs` で、ビューのタブの行を `"always"`(いつも出す。既定)・`"auto"`(ビューが2つ以上のときだけ出す)・`"never"`(出さない)にできるべき
`req~screen.tabs-setting-values~1` P1
- 受け入れ: 既定(always)→ 今のまま。`"never"` → タブの行が無い。ビューを保存していないフォルダを `[display] tabs = "auto"` で開いてビューを1つ保存する → タブの行に2つ。知らない値 → 警告して always。

#### `display.tabs` が `"auto"` でビューが1つのときは、ビューの帯にタブを出さず(検索の欄も出さない設定なら、ビューの帯の行も出さずに表に使い)、ヘッダーにビューの名前も足さない
`req~screen.tabs-setting-auto-single~1` P1
- なぜ: 1つしか無いビューの名前は見分けの役に立たないため
- 受け入れ: ビューを保存していないフォルダを `[display] tabs = "auto"` と `search_bar = false` で開く → タブの行もヘッダーのビューの名前も無く、表が1行上から始まる。

#### ビューの設定の表示の区画(screen.display-options)のタブの項目は、`display.tabs` の always・auto・never の3つを順に切り替えるべき
`req~screen.tabs-setting-cycle~1` P1
- 受け入れ: 表示の区画のタブの項目を押していく → always・auto・never の順に変わる。

#### `display.tabs` でタブを出さないときも、タブの切り替えのキーは効くべき
`req~screen.tabs-setting-hidden-keys~1` P1
- 受け入れ: `[display] tabs = "never"` → タブの行が無く、`]` で切り替わる。

#### 色を使う表示(cell-view.no-color)では、設定の `look.cells` が `"rich"`(既定)のとき、表のセルを値の型に合わせた部品で見せるべき: 真偽の値、リストの要素、種類の少ない短い文字の列(status など)の値、リンク(relations.link-display)、列の見出しの型の印を、screen.style-parts の形(`check`・`tags`・`status`・`links`・`icons`)で見せる
`req~screen.rich-cells-parts~1` P1
- 受け入れ: 色のある表示で、既定の組(sumi)では done(真偽)の列 → `✓`/`·`、tags `[a, b]` → `a · b`、status(doing・todo・done が何行も)→ `● doing`、見出しに型の印なし。組 classic では `☑`/`☐`・札の `a` と `b`・札の `doing`・`◷ due`。`look.cells` の知らない値 → 警告して rich。

#### 表のセルの部品の色は値ごとに決まった色にし、どの値も文字はそのまま見せる(色だけで値を伝えない。cell-view.not-color-only)
`req~screen.rich-cells-value-color~1` P1
- 受け入れ: 組 classic の色のある表示で status が doing の行が2つ → 2つの札の地が同じ色で、どちらの札にも `doing` の文字がある。

#### 部品の種類ごとに部品を使わないときは、screen.style-parts のその部品の形を文字のままの形(`status`・`tags`・`links` は `plain`、`check` は `text`、印は `icons = false`)にする
`req~screen.rich-cells-per-kind~1` P1
- 受け入れ: `[look.style] check = "text"` → 真偽だけ文字のまま。`links = "plain"` → リンクがアクセントの色でない。

#### 列ごとの見せ方は `[look.columns]` に `"rich"`・`"plain"`・`"chip"`(自動で部品にならない列も部品に)で書け、部品の種類ごとの形より優先する
`req~screen.rich-cells-per-column~1` P1
- 受け入れ: `[look.columns]` で `status = "plain"` → その列だけ文字のまま、`owner = "chip"` → owner の値が札。

#### `look.cells = "plain"` と色を使わない表示では、表のセルを今の見た目にする
`req~screen.rich-cells-plain~1` P1
- 受け入れ: `[look] cells = "plain"` か `--no-color` → 今の見た目(`true`・`[a, b]`)。

#### 表のセルを部品で見せても、ファイルに書く値と、編集・検索・出力(`--print`)の値は変えない
`req~screen.rich-cells-values-kept~1` P1
- 受け入れ: 組 classic の色のある表示で done が `☑` のセル → `\` の検索で `true` と打つと当たり、`--print` の done の列は `true`。

#### 設定の `[look.style]` で、表と窓の部品の形を部品の種類ごとに選べるべき
`req~screen.style-parts-choose~1` P1
- 受け入れ: `[look.style] status = "chip"` → 今までの札。`frames = "ascii"` → 枠が ASCII。次の項目と値はどれも警告せずに効く: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、リンク(`links`: `accent`・`plain`)、列の見出しの型の印(`icons`)。

#### `look.preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`・`dozy-pink`)を書けば部品をまとめて選び、`[look.style]` の項目はそれを上書きする
`req~screen.style-parts-preset~1` P1
- 受け入れ: `[look] preset = "saas"` → 淡い地の丸い札と切り替えの枠のタブ。`preset = "dozy-pink"`(テーマ dozy-pink に合う柔らかい組)→ 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ。`preset = "saas"` に `[look.style] status = "chip"` を足す → status だけ今までの札。

#### 部品の形の既定は組 `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)にする
`req~screen.style-parts-default~1` P1
- 受け入れ: 既定(設定なし)で色のある表示 → status の値は `● doing`(点だけに色)、tags は `ui · web`、選んだ行の左の印 `>` はアクセントの色の太字(印の文字は変えない。screen.modern-look)。

#### 部品の形の `pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき
`req~screen.style-parts-pill~1` P1
- 受け入れ: `[look.style] status = "pill"` で `terminal.nerd_font` なし(`TERM_PROGRAM` が ghostty でも WezTerm でもない)→ `soft`。

#### 丸い端を描けるかは設定の `terminal.nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける
`req~screen.style-parts-nerd-font~1` P1
- 受け入れ: `[look.style] status = "pill"` と `[terminal] nerd_font = true` → status の札の両端が丸い端の字。`nerd_font = false` → `soft`。

#### `terminal.nerd_font` の `"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき
`req~screen.style-parts-nerd-auto~1` P1
- なぜ: どの端末でも、描けなければ字が化けない形に落とすため
- 受け入れ: `[look.style] status = "pill"` と `nerd_font = "auto"` で `TERM_PROGRAM=ghostty` → 丸い端の札。`TERM_PROGRAM=tmux` → `soft`。

#### 部品の形・組・丸い札の端の設定(`[look.style]`・`look.preset`・`terminal.nerd_font`)の知らない値は、警告して既定にするべき
`req~screen.style-parts-unknown~1` P1
- 受け入れ: `[look.style] status = "neon"` → 警告して、status は既定の形のまま。

#### テーマ(screen.themes)に、落ち着いた色の組 `sumi`(墨。暗い地・薄い灰・青緑のアクセント)・`slate`(石板。紫寄りのアクセント)・`saas`(明るい灰の地・藍のアクセント)・`paper`(明るい地・青のアクセント)・`saas-dark`(saas の暗い地の版)を足すべき
`req~screen.calm-themes-add~1` P1
- 受け入れ: `theme = "sumi"` → 地が #16171b。

#### 落ち着いた色の組のテーマも、screen.themes と同じく色だけを変え、画面の文字・印・構成は変えない
`req~screen.calm-themes-color-only~1` P1
- 受け入れ: `sumi`・`slate`・`saas`・`paper`・`saas-dark` のどのテーマでも → 画面の文字はテーマなしと同じ。

#### リポの `docs/catalog/index.html` に、部品の形とテーマを見本の表と関係マップで見比べて選び、その設定(`config.toml` に貼れる、cli.config-file の形の文。`[look]`・`[look.style]`・`[look.colors]`)を作れるカタログを置くべき
`req~screen.style-catalog-page~1` P1
- 受け入れ: docs/catalog/index.html を開く → 組とテーマを選ぶと見本が変わり、設定の文が出る。

#### カタログ(`docs/catalog/index.html`)では、色の役割(screen.color-overrides)と値の色(screen.value-colors)も選べ、その設定の文も作れるべき
`req~screen.style-catalog-colors~1` P1
- 受け入れ: カタログで役割 accent に #e0a458 を選ぶ → 設定の文に `[look.colors]` と `accent = "#e0a458"` が出る。値 進行中 に色を選ぶ → 設定の文に `[look.colors.values]` の行が出る。

#### カタログが出す設定の項目と値は、本体の設定(screen.style-parts・screen.calm-themes・screen.themes・screen.color-overrides)が受けるものと同じにし、ずれたら試験で落とすべき
`req~screen.style-catalog-sync~1` P1
- 受け入れ: カタログの `status` の候補に本体に無い値 → 試験が落ちる。

#### カタログ(`docs/catalog/index.html`)は1つのファイルで、外の何にも頼らず開けるべき
`req~screen.style-catalog-offline~1` P1
- 受け入れ: docs/catalog/index.html に、外の URL と外のファイルの読み込みが無い。

#### 設定の `look.theme` が `"auto"` か明暗の表(`{ light = "<テーマ>", dark = "<テーマ>" }`)のとき、端末の地が明るいか暗いかで、明るいときのテーマ(`light`。既定 `saas`)と暗いときのテーマ(`dark`。既定 `sumi`)のどちらかを使うべき(`"auto"` は `{ light = "saas", dark = "sumi" }` と同じ。色を使わない表示ではテーマを使わない。screen.themes)
`req~screen.theme-auto-pick~1` P1
- 受け入れ: `[look] theme = "auto"` で `COLORFGBG=0;15`(明るい地)→ saas。`COLORFGBG=15;0` → sumi。

#### `look.theme` の明暗の表の片方を省けば、省いた側はその既定(`light` は `saas`、`dark` は `sumi`)を使う
`req~screen.theme-auto-partial~1` P1
- 受け入れ: `theme = { light = "paper" }` で明るい地 → paper、暗い地 → sumi。

#### 端末の地の明るさは、環境変数 `COLORFGBG` か端末への問い合わせ(短い待ち時間で、答えが無ければ諦める)で決め、分からなければ暗い地とみなす
`req~screen.theme-auto-detect~1` P1
- 受け入れ: `[look] theme = "auto"` で `COLORFGBG` が無く、問い合わせに答えない → sumi。

#### 設定の `[look.colors]` で、テーマ(screen.themes)の色のうち役割ごとに色を上書きできるべき
`req~screen.color-overrides-roles~1` P1
- 受け入れ: `[look] theme = "sumi"` と `[look.colors] accent = "#e0a458"` → 列の見出しと下の帯のキーが #e0a458。上書きできる役割は `background`(地)・`text`(文字)・`header`(1行目)・`selection`(選んだ行の地)・`selection_text`・`band`(下の帯の地)・`band_text`・`accent`(見出し・キー・枠)・`strong`(強調)・`zebra`(一行おきの地)・`zebra_text`・`mark`(左端の印)・`added`(足した行)・`removed`(消した行)・`pending`(ためた変更)・`highlight`(検索の一致の地)で、どれを書いても警告しない。

#### `[look.colors]` の色は `"#rrggbb"`・`"#rgb"` か、色の名前(`black`・`white`・`gray`・`red`・`orange`・`yellow`・`green`・`teal`・`cyan`・`blue`・`purple`・`magenta`・`pink`・`brown`)で書く
`req~screen.color-overrides-format~1` P1
- 受け入れ: `accent = "orange"` → 名前の色。`accent = "#e0a458"` → #e0a458。

#### `[look.colors]` で上書きしなかった役割は、テーマの色のまま
`req~screen.color-overrides-rest~1` P1
- 受け入れ: `[look] theme = "sumi"` と `[look.colors] accent = "#e0a458"` → 地は sumi のまま。

#### テーマが `default`(色の表なし)のときは、端末の地と文字は変えず、`[look.colors]` の `accent`・`selection` と値の色(screen.value-colors)だけを使うべき
`req~screen.color-overrides-default-theme~1` P1
- 受け入れ: `[look] theme = "default"` と `[look.colors] accent = "orange"`・`background = "#000000"` → 列の見出しは orange の色で、地の色の指定は無い(端末の地のまま)。

#### 色を使わない表示では、`[look.colors]` は効かない(cell-view.no-color)
`req~screen.color-overrides-no-color~1` P1
- 受け入れ: `[look.colors] accent = "#e0a458"` と `--no-color` → 効かない(色の指定が無い)。

#### `[look.colors]` の知らない役割と読めない色は、警告して無視するべき
`req~screen.color-overrides-invalid~1` P1
- 受け入れ: `accent = "#zz"`・`foo = "red"` → 警告して無視。

#### 設定の `[look.colors.values]` で、値ごとの色を決められるべき(キーは値の文字、値は screen.color-overrides と同じ色の書き方)
`req~screen.value-colors-set~1` P1
- 受け入れ: `[look.colors.values] doing = "#d6a85c"` → status が doing の点・札がその色。読めない色 → 警告して無視。

#### `[look.colors.values]` で決めた値は、部品(screen.style-parts の `status`・`tags` のどの形でも、窓の候補でも)でその色を使い、決めていない値は今までの色(状態の語の意味の色か、値の文字から決まる色)のまま
`req~screen.value-colors-apply~1` P1
- 受け入れ: `[look.colors.values] doing = "#d6a85c"` と `[look.style] status = "chip"` → doing の札の地と、窓の候補の doing がその色。決めていない値 → 今までの色。

#### `[look.colors.values]` の値の文字の照合は、前後の空白を除き、大文字・小文字を区別しない
`req~screen.value-colors-match~1` P1
- 受け入れ: `Done = "green"` → done の札が緑(大文字・小文字によらない)。

#### `[look.colors.values]` で、日本語の値にも色を付けられる
`req~screen.value-colors-japanese~1` P1
- 受け入れ: `[look.colors.values] "進行中" = "#d6a85c"` → status が 進行中 の点・札がその色。

#### ヘッダー(screen.layout)の左に、今の範囲と表を「範囲 › 表」の形(パンくず)で見せ、その右に行の数を見せ、範囲がワークスペースでなければ範囲を省くべき
`req~screen.breadcrumb-path~1` P1
- 受け入れ: ワークスペース「仕事」の tasks を開く → 1行目の左が「w 仕事 › tasks ▾  3行」。範囲が無い → 「w tasks ▾  3行」。

#### パンくずは、表の一覧の引き出し(workspace.drawer)を開け閉めする1つのボタンにするべき
`req~screen.breadcrumb-drawer~1` P1
- 受け入れ: パンくずをクリック → 表の一覧の引き出しが開き、もう一度で閉じる。

#### パンくずのボタンは、ビューの帯のボタン(navigation.search-box)と同じ見た目にして、頭に引き出しのキー(screen.keys-vim-and-arrows で割り当てたキー。割り当てが無ければ添えない)、後ろに「▾」を添えるべき
`req~screen.breadcrumb-button-look~1` P1
- なぜ: 押せることと、何が開くかが見て分かるように
- 受け入れ: ワークスペース「仕事」の tasks を開く → 「w 仕事 › tasks ▾」がボタンの見た目。引き出しのキーを外す → 「仕事 › tasks ▾」。

#### ヘッダーの右の端には、画面の型のメニュー(relations.screen-menu)と、ビューの設定の画面(`o`)を開くボタン「⚙ 設定」(キーを添える)をこの順に置くべき(ヘッダーには、どこで何として見るかに関わるものを置き、行を足す「+ 新規」はビューの帯に置く。ビューの帯を出さないときだけ、画面の型のメニューの左に置く。cell-edit.new-note)
`req~screen.breadcrumb-right-buttons~1` P1
- 受け入れ: 1行目の右の端に「▦ 表 ▾」と「o ⚙ 設定」があり、「o ⚙ 設定」を押す → ビューの設定の画面。

#### パンくずとヘッダーの右の端のボタンは、表の画面と関係マップで同じ位置に出し、関係マップで押したら表の画面に戻ってから開くべき
`req~screen.breadcrumb-relation-map~1` P1
- 受け入れ: 関係マップでもパンくずは表の画面と同じ桁で、押すと表の画面に戻って開く。

#### 端末の幅が足りないときは、ヘッダーの設定のボタンを先に省き、次に「+ 新規」を省き、パンくずと画面の型のメニューを残すべき(設定は `o` とパレットでも開ける)
`req~screen.breadcrumb-narrow~1` P1
- 受け入れ: 幅 60 → 設定のボタンが無く、パンくずと「▦ 表 ▾」は出る。

#### ビューの設定の画面(navigation.settings-layout)の左の一覧に「見た目」の区画を持ち、テーマ(screen.themes・screen.calm-themes。`auto` を含む)・組(screen.style-parts の `look.preset`)・丸い札の端(`terminal.nerd_font`: `auto`・`true`・`false`)を選べるべき
`req~screen.look-section-items~1` P1
- 受け入れ: `o` → 左の一覧に「見た目」があり、その区画でテーマ(`auto` を含む)・組・丸い札の端(`auto`・`true`・`false`)を選べる。

#### 「見た目」の区画の上で保存先の範囲(全体・このワークスペース・この表・このビュー。screen.profile-scopes)を選べ、既定は全体にするべき
`req~screen.look-section-scope~1` P1
- 受け入れ: ワークスペースの表で `o` → 保存先は「全体」で、全体・このワークスペース・この表・このビューから選べる。

#### 「見た目」の区画のテーマと組の横には、今効いている値がどこから来たか(例「dracula(この表)」「sumi(config.toml)」)を出すべき
`req~screen.look-section-origin~1` P1
- 受け入れ: config.toml に `[look] theme = "sumi"` で `o` → テーマの横に「sumi(config.toml)」。保存先「この表」でテーマ nord を反映 → テーマの横に「(この表)」。

#### 「見た目」の区画で選んだ見た目は、「反映」で画面にすぐ効かせるべき
`req~screen.look-section-apply~1` P1
- 受け入れ: テーマ dracula・組 dozy-pink を選ぶ → 上に「未反映 1」。反映 → 画面の地が dracula の色、札が dozy-pink の形。

#### 「見た目」の区画で反映した見た目は、選んだ範囲の画面が書く場所(全体は ui.toml、ワークスペースは workspaces.toml のそのワークスペース、表は views.toml の `[[table]]`、ビューはビューの設定)に残すべき(config.toml とワークスペースの印は人が書くファイルなので書き換えない)
`req~screen.look-section-save~1` P1
- 受け入れ: 保存先「全体」でテーマ dracula・組 dozy-pink を反映 → ui.toml の `[look]` に theme = "dracula"・preset = "dozy-pink"、終了して開き直す → 同じ見た目。保存先「この表」でテーマ nord を反映 → views.toml の対象の `[table.look]` に theme = "nord"、ほかの表は dracula のまま。壊れた ui.toml → 警告して config.toml のとおりに起動。

#### 「見た目」の区画の丸い札の端(`terminal.nerd_font`)は、どの範囲を選んでいても ui.toml に残すべき
`req~screen.look-section-nerd-font~1` P1
- なぜ: 丸い札の端は端末の性質のため
- 受け入れ: 保存先「この表」で丸い札の端を `true` にして反映 → ui.toml に nerd_font = true があり、views.toml には書かない。

#### 「見た目」の区画で組を変えたときは、選んだ範囲より広い範囲の部品ごとの形は重ねず、その組の形にするべき(screen.profile-scopes)
`req~screen.look-section-preset~1` P1
- 受け入れ: config.toml に `[look.style] status = "chip"`、保存先「この表」で組 saas を反映 → その表の status は saas の形(chip の札でない)。

#### 「見た目」の区画で、今のテーマと組の組み合わせに名前を付けて、テンプレートとして ui.toml の `[templates.<名前>]` に保存できるべき
`req~screen.look-section-template-save~1` P1
- 受け入れ: 「名前を付けて保存」で「夜」→ ui.toml に `[templates.夜.look]`、テンプレートに「夜」。

#### 「見た目」の区画のテンプレートの一覧(config.toml と ui.toml のテンプレート)から選んで、写しに当てられるべき
`req~screen.look-section-template-use~1` P1
- 受け入れ: テーマ dracula・組 dozy-pink を「夜」で保存し、組を sumi に戻して反映 → テンプレートの「夜」を選んで反映 → dracula と dozy-pink。

#### 「見た目」の区画のテンプレートの一覧のうち、ui.toml のテンプレートは消せるべき
`req~screen.look-section-template-delete~1` P1
- 受け入れ: ui.toml のテンプレート「夜」を消す → ui.toml に `[templates.夜]` が無く、テンプレートの一覧に「夜」が無い。

#### 「見た目」の区画の「この範囲の上書きを外す」で、選んだ範囲に画面が書いた見た目を外し、それより広い範囲のとおりに戻せるべき
`req~screen.look-section-clear~1` P1
- 受け入れ: 全体に dracula、保存先「この表」で nord を反映したあと「この範囲の上書きを外す」→ その表も dracula。

#### ワークスペースの範囲が無いとき(screen.profile-scopes)は、「見た目」の区画の保存先にワークスペースを出さないべき
`req~screen.look-section-no-workspace~1` P1
- 受け入れ: 登録した表(ワークスペースの範囲が無い)で `o` → 保存先に「このワークスペース」が無い。

#### ワークスペースが印から来たときにそれを「見た目」の保存先に選んで反映したら、印には書かないので印に手で書くことを理由として出し、書かないべき
`req~screen.look-section-marker~1` P1
- 受け入れ: 印のワークスペースで保存先「このワークスペース」を反映 → 理由が出て、印は変わらない。

#### 読むだけの起動(`--readonly`)では、「見た目」の区画の反映で画面には効かせるが、どのファイルにも書かないべき(cli.readonly)
`req~screen.look-section-readonly~1` P1
- 受け入れ: `--readonly` → 反映で効くがファイルは無い。

#### ビューの設定の画面の左の一覧の「見た目」の区画の印は、今の組の名前にするべき
`req~screen.look-section-mark~1` P1
- 受け入れ: 既定の組で `o` → 左に「見た目 sumi」。組 dozy-pink を反映 → 左に「見た目 dozy-pink」。

#### 設定(ユーザーが書くもの)と見た目の状態(列の幅・選んだビュー・畳んだまとまりなど、消えても困らないもの)を分け、見た目の状態は、ノートのフォルダの中に書くべきでない
`req~screen.state-separate~1` P1
- 受け入れ: 列の幅を変えて終了 → ノートのフォルダに新しいファイルが無い。

#### 見た目の状態は、`.base` のパス(`.base` なしで開いたときはフォルダのパス)とビューの名前ごとに保存するべき
`req~screen.state-store-key~1` P1
- 受け入れ: ビュー「全部」で title の幅を変えて終了し、開き直す → 「全部」の title は変えた幅で、ほかのビューの title の幅は変わらない。

#### 見た目の状態が壊れていたら、捨てて既定で起動するべき
`req~screen.state-store-broken~1` P1
- 受け入れ: 状態のファイルを壊して起動 → 既定で起動し、エラーで止まらない。

#### 見た目の状態の列の並びと隠す列は、今の `.base` にある列にだけ当て、`.base` に増えた列は右に足すべき
`req~screen.state-store-columns~1` P1
- 受け入れ: 列 a・b・c を c・a・b に並べて終了し、`.base` に列 d を足して開き直す → c・a・b・d の順。

#### 見出しからの並べ替え(navigation.header-sort)は、設定の並べ替えとして残すべき
`req~screen.state-store-header-sort~1` P1
- 受け入れ: 見出しから title で並べ替えて終了し、同じ対象で開き直す → title で並んだまま。

#### 表の見せ方の次の4つの既定を、設定(cli.config-file の設定ファイル)で決められるべき: 行番号(表の左に1始まりの番号。既定は出さない)・一行おきの色(既定は付けない)・列の区切り線(既定は今の見た目)・上の帯(ビューの帯のタブと検索の欄(navigation.search-box)、設定の帯(navigation.settings-bar)をそれぞれ出すか。タブと検索の欄を両方出さなければ、ビューの帯の行ごと出さない)
`req~screen.display-options-config~1` P1
- 受け入れ: 設定で行番号を出す → 各行の左に 1・2・3。一行おきの色 → 偶数の行の背景が違う。区切り線を入れる → 列の間に `│` が出る。既定(今の見た目)では `│` が無い。検索の欄を隠す → 表が1行広い。

#### 表の見せ方の4つ(行番号・一行おきの色・列の区切り線・上の帯)は、ビューの設定(navigation.view-settings)の「表示」の区画(navigation.settings-layout)でビューごとに切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.display-options-per-view~1` P1
- 受け入れ: 設定で行番号を出し、ビューの設定の表示の区画で消して保存 → そのビューだけ番号が無く、開き直しても同じ。

#### 上の帯を隠しても、その帯の操作(タブの切り替えのキー・`\`・`f`)は効くべき
`req~screen.display-options-hidden-keys~1` P1
- 受け入れ: 検索の欄を隠す → `\` は最下行で打てる。タブを隠す → `]` で切り替わる。

#### 一行おきの色は、色を使わない表示(`--no-color`・NO_COLOR)では付けないべき(cell-view.not-color-only)
`req~screen.display-options-zebra-no-color~1` P1
- 受け入れ: 一行おきの色を付ける設定で `--no-color` → 偶数の行の背景も同じ。

#### screen.display-options の設定は、`[display]` の表に15の項目を持つべき
`req~screen.display-config-keys~1` P1
- 受け入れ: `[display] row_numbers = true` → 番号が出て、設定の画面の表示の区画でも「行番号」がオン。`[display] search_bar = false` → 検索の欄が無い。`--print-config` の `[display]` に15項目: `row_numbers`(既定 false)・`zebra`(既定 false)・`column_lines`(既定 false。true で列の間に `│` を引く)・`group_gap`(既定 false。screen.group-gap)・`tabs`(既定 `"always"`。screen.tabs-setting)・`search_bar`(既定 true。navigation.search-box)・`chips`(既定 true)・`buttons`(既定 `"plain"`。navigation.search-box)・`bar_padding`(既定 1。navigation.search-box)・`top_padding`(既定 1。screen.layout)・`row_extra_lines`(既定 0。cell-view.row-lines)・`note_path`(既定 `"common"`。screen.note-column・screen.note-path)・`note_name`(既定なし。screen.note-name)・`tab_wrap`(既定 false。navigation.tab-wrap)・`tab_wrap_gap`(既定 false。navigation.tab-wrap)。

#### ビューごとの表示の値は、ビューの設定(settings)の `display` に、設定と違う項目だけを持ってよい
`req~screen.display-config-view-diff~1` P1
- 受け入れ: 設定が `[display] row_numbers = true` で、ビューの settings の `display` に `zebra = true` だけを書く → そのビューは行番号と一行おきの色の両方が効く。

#### 表の見せ方(screen.display-options)の、まとまりの見出しの上の空きは、`groupBy`(navigation.settings-group の設定のグループ分けを含む)のまとまりがあるとき、2つ目以降の見出しの行の上に空きの行を1つ入れるべき
`req~screen.group-gap-line~1` P1
- 受け入れ: `[display] group_gap = true` で status でまとめたビューを開く → 2つ目の見出しの上に空きの行があり、1つ目の見出しの上には無い。まとまりの無いビュー → 空きは出ない。

#### まとまりの見出しの上の空きは、既定では空けず、設定の `[display]` の `group_gap`(既定 false)で既定を決めるべき
`req~screen.group-gap-config~1` P1
- 受け入れ: `group_gap` の無い設定 → 今と同じ画面。

#### まとまりの見出しの上の空きは、screen.display-options のほかの項目と同じく、ビューの設定(navigation.view-settings)でビューごとに切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.group-gap-per-view~1` P1
- 受け入れ: `group_gap` の無い設定で、status でまとめたビューの表示の区画で空きを入れて保存 → そのビューだけ2つ目の見出しの上に空きの行があり、開き直しても同じ。

#### まとまりの見出しの上の空きの行は、行として数えない(行の数・行番号・`--print` に出さない)べき
`req~screen.group-gap-not-row~1` P1
- なぜ: 空きは見た目のためだけにあるため
- 受け入れ: `[display] group_gap = true` と `row_numbers = true` で status でまとめた3行のビュー → ヘッダーの行の数は 3行で、行番号は 1・2・3、空きの行に番号が無い。

#### まとまりの見出しの上の空きの行には、カーソルを止めないべき
`req~screen.group-gap-cursor~1` P1
- なぜ: 空きは見た目のためだけにあるため
- 受け入れ: 1つ目のまとまりの最後の行で `j` → 空きを飛ばして2つ目の見出しへ。

#### まとまりの見出しの上の空きの行は、クリックしても選びを変えないべき
`req~screen.group-gap-click~1` P1
- なぜ: 空きは見た目のためだけにあるため
- 受け入れ: 空きの行をクリック → 選びは変わらない。

#### 表のセルの編集の入力欄は、地の色で欄と分かるようにするべき(色を使わない設定では今までどおり太字と下線。太字と下線は screen.themes のテーマの強い色の目印なので残す)
`req~screen.input-box-fill~1` P1
- 受け入れ: status のセルで Enter → 入力欄に地の色が付く。`--no-color` で同じ操作 → 入力欄は太字と下線。

#### 表のセルの編集の入力欄の幅は、列・打った文字・16桁のうち広いもの(右の列に重ね、画面の幅まで)にするべき
`req~screen.input-box-width~1` P1
- 受け入れ: status のセルで Enter → 入力欄は16桁の幅。

#### 新しいノートの窓(cell-edit.new-note-form)は、枠のある窓として表の上に重ね、上の縁に題と作る場所のフォルダの名前、下の縁の上に押せるキーを出すべき
`req~screen.input-box-form-frame~1` P1
- 受け入れ: 新しいノートの窓 → `╭─ 新しいノート` の枠、窓の中に `^S 作る`。

#### 新しいノートの窓の、まだ値の無い欄には、型の手がかり(日付は `YYYY-MM-DD`、日時は `YYYY-MM-DDTHH:MM`、数・リスト・チェック)を薄く出すべき
`req~screen.input-box-form-hints~1` P1
- 受け入れ: 新しいノートの窓 → due の欄に `YYYY-MM-DD`。

#### 新しいノートの窓の枠の線は、あいまいな幅の文字を2と数える設定(cell-view.ambiguous-width)では ASCII の線にするべき
`req~screen.input-box-ascii~1` P1
- 受け入れ: `ambiguous_wide = true` → 新しいノートの窓の枠は `+-`。

#### 表に重ねる窓(候補・操作の一覧・リストの選択・カレンダー・頻度表・新しいノートの窓など)の枠は、既定でつながった罫線(角の丸い `╭ ╮ ╰ ╯` と `─ │`)で描き、設定の `look.style.frames` が `"ascii"` のときと、East Asian Ambiguous の文字を幅2とする設定(cell-view.ambiguous-width)のときは、幅がずれないように ASCII の `+ - |` で描くべき
`req~screen.frames~1` P1
- 受け入れ: 既定で候補の窓を開く → 枠が `╭─…─╮`。`[look.style] frames = "ascii"` か `[terminal] ambiguous_wide = true` → `+-…-+`。

#### 色を使える表示で、設定の `look.mode` が `"modern"`(既定)のときは、今どきの道具のような見た目として、窓の枠を薄い色にし、今操作している窓の枠と題をアクセントの色にするべき
`req~screen.modern-look-frames~1` P1
- 受け入れ: 色のある表示で候補の窓を開く → 今操作している候補の窓の枠はアクセントの色。

#### `look.mode` が `"modern"` のとき、選んでいる行・候補・項目は、反転でなく背景の色と太字(形は screen.style-parts の `select`)で見せる
`req~screen.modern-look-select~1` P1
- 受け入れ: 色のある表示で候補の窓を開く → 選んだ候補のセルに反転が無く背景の色が付く。

#### `look.mode` が `"modern"` のとき、下の帯・ビューのタブ・列の見出しの形は screen.style-parts の `band`・`tabs`・`icons` に従う
`req~screen.modern-look-parts~1` P1
- 受け入れ: 色のある表示で、下の帯のキーは組の帯の形(screen.style-parts の `band`)で、説明は薄い色(既定の sumi は `quiet` でキーは太字、`classic` の組ではアクセントの色の太字)。

#### `look.mode` が `"modern"` のとき、色はテーマ(screen.themes)のものを使い、テーマが既定(色の表が無い)なら端末の地の色を変えない控えめな組を使う
`req~screen.modern-look-colors~1` P1
- 受け入れ: テーマを決めない色のある表示で候補の窓を開く → 画面の地の色の指定が無く(端末の地のまま)、選んだ候補には背景の色が付く。

#### `look.mode` が `"classic"` のときと色を使わない表示(cell-view.no-color)では、今までの見た目(反転)にする
`req~screen.modern-look-classic~1` P1
- 受け入れ: `[look] mode = "classic"` → 今の見た目(反転)。`--no-color` → 色の指定が無い。

#### `look.mode` が `"modern"` でも `"classic"` でも、値の文字・印・構成は変えない(screen.themes)
`req~screen.modern-look-same-text~1` P1
- 受け入れ: どの mode でも、同じ表と候補の窓の画面の文字は同じ。

#### 画面と起動の文言(使い方(`--help`)・起動できない理由・ヘルプ・パレット・下の帯とメッセージ行・キーの表示名・確認の画面・ビューの設定の画面(区画の名前と説明。navigation.settings-layout)・操作の一覧(screen.action-menu)・前置きのキーの続きの案内(screen.prefix-hints)・読むだけや未対応や保存が止まった理由・`--print` と `--pick` の警告)を、英語と日本語で出せるべき
`req~screen.locales-two-languages~1` P1
- 受け入れ: 台帳の全部の文言が英語と日本語の両方にある。

#### 文言の言語は、設定(cli.config-file)の `language`(`"auto"`・`"en"`・`"ja"`。既定 `"auto"`)で選ぶべき
`req~screen.locales-setting~1` P1
- 受け入れ: `LANG=en_US.UTF-8` で設定に `language = "ja"` → 日本語。

#### `language` が `auto` のときは、環境変数 `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値が `ja` で始まれば日本語、それ以外(どれも無いときも)英語にするべき
`req~screen.locales-auto~1` P1
- 受け入れ: `LANG=en_US.UTF-8` で起動 → 下の帯と `--help` が英語。`LANG=ja_JP.UTF-8` → 日本語。どの環境変数も無い → 英語。

#### 文言は言語ごとの台帳(`locales/<言語の符号>.toml`。英語の `en.toml` が文言の名前と並びの正本)に置き、どれかの台帳で文言が欠けるか余るか空であるか、差し込みの番号(`{0}` など)の組が英語と違えば、ビルドで落ちるべき
`req~screen.locales-ledger~1` P1
- 受け入れ: `locales/ja.toml` で文言が1つ欠ける、または差し込みの番号が英語と違う → ビルドが落ちる。

#### 文言の言語の一覧は台帳のファイルから作り、言語を足すときは台帳を足せばよい形にするべき
`req~screen.locales-list~1` P1
- 受け入れ: `locales/` に en.toml と同じ名前の文言を持つ `fr.toml` を足してビルドする → ほかのファイルを変えずに、言語の一覧に fr が入る。

#### ノートの値・列の名前・`.base` の中身・ファイルの名前は、文言の言語によらず訳さないべき
`req~screen.locales-no-translate~1` P1
- 受け入れ: `LANG=en_US.UTF-8` で、列 担当 と status の値 進行中 のノートを開く → 列の名前 担当 と値 進行中 はそのまま出る。

#### 画面の色の組み合わせ(テーマ)を、見本の組(今の見た目・Nord・Solarized Light・Dracula・Gruvbox・Pink Monster・Dozy Pink と、screen.calm-themes の落ち着いた組)から設定(cli.config-file)で選べるべき
`req~screen.themes-choose~1` P1
- 受け入れ: 設定でテーマを Nord にして起動 → 左上の背景が #2e3440。

#### 今の見た目以外のテーマは、画面の地と文字・ヘッダー・選択・下の帯・列の見出し・強調(太字)・一行おきの色(screen.display-options)・行の印 `>` を、テーマごとに決めた色で塗るべき
`req~screen.themes-paint~1` P1
- 受け入れ: 設定でテーマを Nord にして起動 → 選んでいるセルの背景が #5e81ac、下の帯の背景が #3b4252。

#### テーマは色だけを変え、画面の構成・文字・印は変えないべき
`req~screen.themes-color-only~1` P1
- なぜ: 色だけで意味を伝えない cell-view.not-color-only を保ち、画面の型を変える scope.screen-types と分けるため
- 受け入れ: 7つのどのテーマでも、画面の文字はテーマなしと1字も違わない。

#### screen.themes のテーマは、設定の `look.theme` に `"default"`(既定。今の見た目で、端末の既定の色と反転・太字・薄い表示だけを使う)・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"`・`"sumi"`・`"slate"`・`"saas"`・`"saas-dark"`・`"paper"`(screen.calm-themes)・`"auto"`(screen.theme-auto)のどれかを書いて選んでよい
`req~screen.theme-names-list~1` P1
- 受け入れ: 設定なし → 今と同じ色。`[look] theme = "dracula"` → 警告なしで dracula の色。

#### `look.theme` の知らない名前は、警告にとどめて `default` で起動するべき(cli.config-file)
`req~screen.theme-names-unknown~1` P1
- 受け入れ: `[look] theme = "neon"` → 警告に `look.theme` と出て既定の見た目で起動。

#### 色を使わない表示(cell-view.no-color・`--no-color`・設定の `terminal.color = false`)では、テーマを使わないべき
`req~screen.theme-names-no-color~1` P1
- 受け入れ: `[look] theme = "dracula"` と `NO_COLOR=1` → 色の指定が無い。

#### トゥルーカラーに対応しない端末(cell-view.not-color-only)では、テーマの色を近い 256 色に落とすべき
`req~screen.theme-names-256~1` P1
- 受け入れ: `COLORTERM` なしで `[look] theme = "nord"` → 256 色の番号で塗る。

#### screen.themes のテーマ(今の見た目を除く)では、ためる変更のセルの文字(cell-edit.bulk-set)、保存の確認の差分の足した行と消した行の文字(write-back.pending-and-diff)、検索の一致と同じ値の強調の地(navigation.find・navigation.same-value)も、テーマごとに決めたその地で読める色で塗るべき
`req~screen.theme-meaning-colors-paint~1` P1
- 受け入れ: Solarized Light でためる変更のセル → 文字が #b58900。差分の足した行 → #859900、消した行 → #dc322f。テーマなし → 今までと同じ色。

#### テーマのためる変更・差分・検索の一致の色は意味を足すだけで、印(下線・`+`・`-`)はテーマなしと同じに保つべき
`req~screen.theme-meaning-colors-marks~1` P1
- 受け入れ: Solarized Light で保存の確認の差分を開く → 足した行の `+` と消した行の `-` は、テーマなしと同じ字と位置。

#### 表のプロファイル(cli.config-file の `[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]` と `use`)は、全体・ワークスペース・表・ビューの4つの範囲に書けるべき
`req~screen.profile-scopes-sections~1` P1
- 受け入れ: 表に `[dates] format = "YYYY/MM/DD"` → その表だけ日付が `/` 区切り。

#### 表のプロファイルの全体は config.toml と ui.toml、ワークスペースはその印 `.mdgrid/workspace.toml`(workspace.marker)か workspaces.toml の `[[workspace]]`(workspace.app-side)、表はワークスペースの表の項目(印の `[[table]]`・`[[workspace.table]]`)と views.toml の `[[table]]`(base-view.views-toml)、ビューはビューの設定(navigation.settings-remember。mdgrid のビューは保存で定義にも入る。base-view.save-view)に書く
`req~screen.profile-scopes-files~1` P1
- 受け入れ: notes/.mdgrid/workspace.toml に `[look] theme = "nord"`、その印の tasks の `[table.look]` に theme = "dracula" → notes/tasks の地が dracula、notes/projects は nord。

#### 表のプロファイルのワークスペースの範囲は workspace.scope-order で決めた範囲のものとし、検知(workspace.detect-roots)と登録した表(cli.places)から来た範囲にはワークスペースの範囲は無い
`req~screen.profile-scopes-workspace~1` P1
- 受け入れ: 検知(workspace.detect-roots)から来た範囲で表を開く → ワークスペースの範囲の値を使わず、表・全体・既定の値で描く。

#### 表のプロファイルの効く値は項目ごとに、狭い範囲を広い範囲より先にし(ビュー → 表 → ワークスペース → 全体 → 既定)、同じ範囲に2つあるときは画面が書いたものを手で書いたものより先にする(全体は ui.toml → config.toml、表は views.toml → ワークスペースの表の項目)べき
`req~screen.profile-scopes-order~1` P1
- 受け入れ: config.toml に `[look] theme = "sumi"`、notes/.mdgrid/workspace.toml に `[look] theme = "nord"`、views.toml の notes/tasks の `[table.look] theme = "dracula"` → notes/tasks の地が dracula、notes/projects は nord、notes の外の表は sumi。別に、ui.toml に `[look] theme = "dracula"`、config.toml に `[look] theme = "sumi"` だけ → どの表も dracula。

#### 組(`look.preset`)を書いた範囲より広い範囲の部品ごとの形(`[look.style]`)は、使わないべき
`req~screen.profile-scopes-preset-cut~1` P1
- 受け入れ: ワークスペースに `[look] preset = "saas"`、config.toml に `[look.style] status = "chip"` → ワークスペースの表の status は saas の形。

#### テーマ(`look.theme`)を書いた範囲より広い範囲の役割の色(`[look.colors]` の役割。値の色 `[look.colors.values]` は除く)は、使わないべき
`req~screen.profile-scopes-theme-cut~1` P1
- 受け入れ: ワークスペースに `[look] theme = "nord"`、config.toml に `[look.colors] accent = "orange"` → accent は nord の色。

#### `use = "<名前>"` は、テンプレート(`[templates.<名前>]`。同じ名前は ui.toml を config.toml より先にする)を、その範囲で書いた値の下に敷くべきで、無い名前は警告して無視するべき
`req~screen.profile-scopes-use~1` P1
- 受け入れ: 表に `use = "夜"`(`[templates.夜.look] theme = "dracula"`)と `[look] preset = "grid"` → dracula と grid。`use = "無い"` → 警告。

#### アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、`[terminal]`・`[workspace]`・`[keys]`)は全体にだけ書け、ほかの範囲に書けば警告して無視するべき
`req~screen.profile-scopes-app-only~1` P1
- 受け入れ: ワークスペースに `[terminal] color = false` → 警告して無視。

#### 表を切り替えたら(workspace.palette・cli.places-list・関係マップから開く)、その表で効く表のプロファイルの値で画面を描き直すべき
`req~screen.profile-scopes-redraw~1` P1
- 受け入れ: config.toml に sumi、notes/.mdgrid/workspace.toml に nord、notes/tasks に dracula の設定で notes/tasks を開き、関係マップから notes/projects を開く → nord で描き直す。

#### 関係マップのように1つの画面に複数の表が出るときは、今開いている表のプロファイルの値を使うべき
`req~screen.profile-scopes-relation-map~1` P1
- 受け入れ: notes/tasks に dracula、notes/projects に nord の設定で、notes/tasks から関係マップを開く → 関係マップの地は dracula。

#### アイコン用のフォント(Nerd Font)の記号は、設定 `terminal.nerd_icons`(真偽値。既定 false。全体だけ)を true にしたときだけ使うべきで、既定では今の字の記号(設定のボタンの `⚙`・画面の型のメニューのボタンの `▦`・`◇`・列の見出しの型の印 `#`・`◷`・`⋮`・`◉`)のままにするべき
`req~screen.nerd-icons-setting~1` P1
- 受け入れ: 既定 → ヘッダーに `⚙ 設定`・`▦ 表 ▾`、日付の列の見出しに `◷`。

#### `terminal.nerd_icons` が true のときは、設定のボタンの歯車・画面の型のメニューのボタンと窓(表・関係マップ)・列の見出しの型の印(screen.rich-cells。数・日付・日時・チェック・リスト・札)・日付のカレンダーの題に、決まったアイコン用のフォントの記号を使うべき
`req~screen.nerd-icons-symbols~1` P1
- 受け入れ: `[terminal] nerd_icons = true` → 設定のボタン・表のボタン・日付の列の見出し・カレンダーの題にアイコン用のフォントの記号(私用の領域の字)が出て、`⚙`・`▦`・`◷` は出ない。

#### `terminal.nerd_icons` が true のときも、アイコンに絵文字は使わないべき
`req~screen.nerd-icons-no-emoji~1` P1
- 受け入れ: `[terminal] nerd_icons = true` → ヘッダー・列の見出し・カレンダーの題に絵文字の字が無い。

#### アイコンの記号の幅は字の幅の決まり(cell-view.grapheme-width・cell-view.ambiguous-width)で数え、列の位置とクリックの当たりをずらさないべき
`req~screen.nerd-icons-width~1` P1
- 受け入れ: `[terminal] nerd_icons = true` → どの行も画面の幅を超えない。

#### ノートの欄(screen.note-column)のフォルダの出し方 `display.note_path`(`"common"`・`"name"`・`"full"`・`"off"`。既定 `"common"`)を、表のプロファイルの項目として全体・ワークスペース・表・ビューの範囲(screen.profile-scopes)に書けるべき
`req~screen.note-path-scopes~1` P1
- 受け入れ: config.toml に `[display] note_path = "name"` → どの表の欄も名前だけ。`--print-config` の出力と docs/config.md に項目がある。

#### `display.note_path` の4つの値は、ビューの設定の「表示」の区画(navigation.settings-layout)で順に切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.note-path-per-view~1` P1
- 受け入れ: ビューの設定の「表示」で「ノートのパス」を `full` にして反映 → そのビューだけ根からのパス。もう一度押して `off` → 欄に名前が出ない。名前を付けて保存して開き直す → 同じ。

#### `display.note_path` の知らない値は、警告して既定にするべき(cli.config-file)
`req~screen.note-path-unknown~1` P1
- 受け入れ: `note_path = "short"` → `config.toml: display.note_path:` で始まる警告と既定の見た目。

#### ノートの欄(screen.note-column)の名前(`display.note_path` で決めた、`.md` を除いた文字)の出し方を、正規表現 `display.note_name`(文字列。既定なし)で決められるべき
`req~screen.note-name-regex~1` P1
- 受け入れ: `note_path = "common"` と `'^\d{8}_'` で `todo/20261015_a.md`・`archive/todo/20261001_b.md` → `todo/a`・`archive/todo/b`(フォルダは残し、最後の名前の部分に当てる)。

#### `display.note_name` は、表のプロファイルの項目として全体・ワークスペース・表・ビューの範囲(screen.profile-scopes)に書けるべき
`req~screen.note-name-scopes~1` P1
- 受け入れ: views.toml のある表の `[table.display]` に `note_name = '^\d{8}_'` を書く → その表の欄だけ `20261015_資料を作る.md` が `資料を作る`、ほかの表は名前のまま。

#### 名前が `display.note_name` の正規表現に一致したときは、名前の付いた組 `name` があればその部分を、無ければ1つ目の組があればその部分を、組が無ければ一致した部分を除いた残りを出し、一致しないときと、結果が空になるときは名前をそのまま出すべき
`req~screen.note-name-match~1` P1
- 受け入れ: `note_path = "name"`・`note_name = '^\d{8}_'` で `todo/20261015_資料を作る.md` → 欄は `資料を作る`。`note_name = '^\d{8}_(?P<name>.+)$'` → 同じ。`note_name = '^(.+)_v\d+$'` で `企画_v3.md` → `企画`。一致しない `メモ.md` → `メモ`。`note_name = '^.*$'`(全部を除くと空)→ 名前のまま。

#### `display.note_name` の読めない正規表現と文字列でない値は、cli.config-file の形で警告して使わないべき
`req~screen.note-name-invalid~1` P1
- 受け入れ: `note_name = '('` → `config.toml: display.note_name:` で始まる警告と、名前のまま。

#### `display.note_name` の項目を、`--print-config` と docs/config.md に置くべき
`req~screen.note-name-docs~1` P1
- 受け入れ: `--print-config` の `[display]` と docs/config.md に `note_name` の項目がある。

#### `display.note_name` は、ビューの設定の「表示」の区画(navigation.settings-layout)では切り替えない(ビューごとの値は views.toml のビューの設定の `display` に手で書ける)
`req~screen.note-name-not-in-settings~1` P1
- なぜ: 自由に書く文字列で、順に切り替える値が無いため
- 受け入れ: ビューの設定の「表示」の区画 → 「ノートのパス」の項目はあり、note_name の項目は無い。

#### `display.note_name` で欄の名前を変えても、検索(navigation.find)と、メッセージ・詳しく見る画面のノートの名前は変えないべき
`req~screen.note-name-search-kept~1` P1
- 受け入れ: `note_name = '^\d{8}_'` で欄が `資料を作る` のノート `20261015_資料を作る.md` → `\` の検索で `20261015` と打つと当たる。

#### 表の画面の列の見出しでは、計算の列(`formula.x`)の見出しの頭に `ƒ` と空白を付けて、ノートのキー(フロントマター)の列と見分けられるようにするべきで、色を使わない表示と見た目の設定(screen.rich-cells の型の印の有無)に依らず付けるべき
`req~screen.formula-mark-prefix~1` P1
- なぜ: 色や設定で消えると見分けられないため
- 受け入れ: 計算の列 `formula.担当` とキーの列 `title`・`actor` を並べた表 → 見出しの行に「ƒ 担当」があり、ƒ は1つだけ。`--no-color` でも同じ。

#### 計算の列の列の幅は、見出しの `ƒ` の印を含めて数えるべき
`req~screen.formula-mark-width~1` P1
- 受け入れ: 計算の列 `formula.担当` の値がどれも2桁以下 → 列の幅は「ƒ 担当」の6桁以上で、見出しが切れない。

#### `--print` などの出力の見出しは、計算の列でも変えない(`ƒ` を付けない)べき
`req~screen.formula-mark-print~1` P1
- 受け入れ: `--print` の見出しは「担当」で ƒ が無い。

#### リポの説明書(`docs/manual/<言語>/views.md`。サイトでは説明書の1章)に、画面の型と見せ方の一覧のページを置くべき
`req~screen.views-page-file~1` P1
- 受け入れ: `docs/manual/ja/views.md` と `docs/manual/en/views.md` があり、`scripts/pages.py` の `PAGES` に `views.md` がある。

#### 見せ方の一覧のページは、1つの見せ方につき、本物の画面の画像1枚(`docs/manual-scenarios.toml` の場面)と、何が見えてどう開くかの説明と、くわしい節へのリンクを並べる
`req~screen.views-page-entry~1` P1
- 受け入れ: views.md の画像が全部場面の id で、相対のリンク先が全部ある。

#### 見せ方の一覧のページに載せる見せ方は、少なくとも、表・ビューのタブ・まとまり(グループ)・木・関係マップ・値の分布・絞り込みの帯・テーマと部品の形の8つ
`req~screen.views-page-items~1` P1
- 受け入れ: views.md の見出しに、題の8つの見せ方がある。

#### サイトの入口(`scripts/pages.py` の「見て選ぶ」)から、見せ方の一覧のページへ行けるべき
`req~screen.views-page-link~1` P1
- 受け入れ: 入口の文言に、見せ方の一覧(views.md)へのリンクがある。

#### 画面の型(表・木・関係マップなど)を足す変更は、見せ方の一覧のページに1項を足すべき
`req~screen.views-page-keep~1` P1
- 受け入れ: 画面の型を足した変更の記録で、この一覧に項が無い → 違反(読んで判定)。

#### 説明書の目次(`docs/manual/<言語>/index.md`)と、サイトの説明書の章の並び(`scripts/pages.py` の `PAGES`)は、読む人の順に並べるべき: 誰のための道具で何がどう見えるか(index)→ 見せ方の一覧(views)→ はじめかた(getting-started。1分で試す道を1本)→ 作業の手引き(tasks)→ 考え方(concepts)→ 見た目(appearance・themes)→ 困ったとき(faq)→ 参照
`req~screen.docs-order-toc~1` P1
- 受け入れ: `docs/manual/ja/index.md` と `en/index.md` の目次のリンクが views → getting-started → tasks → concepts → appearance → faq の順に並び、`PAGES` も同じ順である。

#### 説明書とサイトの入口は短く保ち、詳しさは下のページへ送るべき
`req~screen.docs-order-short-entry~1` P1
- 受け入れ: 入口(`docs/manual/<言語>/index.md` と `scripts/pages.py` の入口の文言)を読む → 詳しい手順と設定の説明は無く、下のページへのリンクで送っている(読んで判定)。

#### 説明書・サイト・README の文は `docs/writing.md` の決まり(具体から書く、語尾と長さを変える、接続詞を連ねない、箇条書きは並列のものだけ、大げさな語・対比の型・三つ組・太字の強調・締めの定型句を使わない)で書くべき
`req~screen.docs-order-style~1` P1
- 受け入れ: 入口と説明書の文に、大げさな語(強力・シームレス・直感的・画期的・最適・豊富 / seamless・robust・powerful・intuitive・leverage・empower・delve・crucial・pivotal)が無い。文の決まりに合うかは、`docs/writing.md` の一覧に照らして読んで判定。

#### リポの説明書(`docs/manual/<言語>/features.md`。サイトでは説明書の1章で、見せ方の一覧の次)に、できること(機能)の一覧のページを置くべき
`req~screen.features-page-file~1` P1
- 受け入れ: `docs/manual/ja/features.md` と `docs/manual/en/features.md` があり、`scripts/pages.py` の `PAGES` で `features.md` が `views.md` の直後にある。

#### 機能の一覧のページは、1つの機能につき、本物の画面の画像1枚(`docs/manual-scenarios.toml` の場面)と、何ができてどう始めるかの説明と、くわしい節へのリンクを並べる
`req~screen.features-page-entry~1` P1
- 受け入れ: features.md の画像が全部場面の id で、どの機能の節にも画像1枚とくわしい節へのリンクがあり、相対のリンク先が全部ある。

#### 機能の一覧のページに載せる機能は、少なくとも、値をその場で直す・差分を見て保存する・まとめて直す・探す/絞る/並べる・新しいノートを作る・ビューを決めて保存する・`.base` を開く・リンクで表をつなぐ・スターターから台帳を作る・ほかのコマンドと組む・読むだけで開く・見た目を選ぶの12
`req~screen.features-page-items~1` P1
- 受け入れ: features.md に、題の 12 の機能それぞれの節がある。

#### サイトの入口(`scripts/pages.py` の「見て選ぶ」)から、機能の一覧のページへ行けるべき
`req~screen.features-page-link~1` P1
- 受け入れ: 入口の文言に、機能の一覧(features.md)へのリンクがある。

#### 機能を足す変更は、機能の一覧のページに1項を足すべき
`req~screen.features-page-keep~1` P1
- 受け入れ: 機能を足した変更の記録で、この一覧に項が無い → 違反(読んで判定)。
