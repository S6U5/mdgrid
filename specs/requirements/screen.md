# 画面と操作(screen) の要求

## 目的

表を見て直すための画面。表の TUI への不満の中心は「機能が足りない」より「あるのに見つからない」と「日本語や端末の違いで崩れる」なので、見つけやすさ(下の帯・ヘルプ・パレット)と崩れない描画を、機能より先に決める。cellops(自作のタスクの TUI)で作り直しを重ねて固まった形のうち、タスクに依らない部分も採る。表の中身は [base-view](base-view.md)、移動と検索は [navigation](navigation.md)、セルの値・文字の幅・色の見せ方は [cell-view](cell-view.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/screen/spec.md](../_history/decidespec/screen/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 旧い SR-7(新しい名前なし)は 2026-09-30 に削除([2026-09-30](../_history/decidespec/_decisions/2026-09-30-v1-review-fixes.md))。移行先: screen.keys-vim-and-arrows。
- 次の段: 端末のダーク / ライトの自動判定でテーマを選ぶこと(テーマは screen.themes のとおり設定で選ぶ)。
- 採らない: 画面の型の追加(タスクの画面などは scope の scope.screen-types)。
- 採らない: 説明書・サイト・README を英語と日本語の外の言語に訳すこと(2026-11-21。台帳で訳すのは画面と起動の文言と設定の説明だけ)。日付の言葉(cell-edit.date-words)も日本語と英語だけ。
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
`req~screen.layout-order~2` P1
- Acceptance:
  - Given:
  - When: 80×24 の端末で起動
  - Then: 1行目は空いた上の余白、2行目にヘッダー(screen.breadcrumb。範囲と表のパンくずと行数、右の端に画面の型のメニュー(relations.screen-menu)と設定のボタン)、3〜5行目にビューの帯(navigation.search-box。3行目と5行目は空き行、4行目は左にタブ、右に検索の欄と絞り込み・並べ替え・分類・列のボタン、右の端に作れるときは「+ 新規」(cell-edit.new-note))、6行目から表(ゴールデン)。
- Acceptance:
  - Given: 絞り込みが効いているとき
  - When:
  - Then: ビューの帯と表のあいだに設定の帯の行が入る。

#### 上の余白は、設定 `display.top_padding` で 0〜2 を 0.5 刻みで選べ、既定を 1 にするべき
`req~screen.layout-top-padding~2` P1
- Rationale: navigation.search-box と同じく、環境で見え方がぶれないようにするため
- Acceptance:
  - Given: `display.top_padding = 0` と `display.bar_padding = 0`
  - When:
  - Then: 1行目にヘッダー、2行目にビューの帯、3行目から表。
- Acceptance:
  - Given: `display.top_padding` を書かない
  - When:
  - Then: ヘッダーの上に空き行が1行。

#### 上の余白の端数は、ビューの帯の余白(navigation.search-box)と同じく半分の塗りの字で描き、そのときはヘッダーの行にもビューの帯と同じ地の色を敷くべき
`req~screen.layout-half-padding~1` P1
- Acceptance:
  - Given: 端末の地の色が分かる色ありの表示で `display.top_padding = 0.5`
  - When:
  - Then: 1行目は上の余白の半行(`▀`)、2行目のヘッダーの行の地はビューの帯と同じ地の色。

#### 下の帯には、今のモード・選んだセルの位置・未保存の数・今押せるキーを出すべき
`req~screen.layout-bottom-bar~1` P1
- Acceptance:
  - Given:
  - When: 80×24 の端末で起動
  - Then: 下の帯に今のモード、選んだセルの位置、「未保存 0」、今押せるキーが並ぶ(ゴールデン)。

#### 読むだけのセルの理由に前置きの「読むだけ: 」を付けて出す所(セルを選んだときの案内・直そうとしたときのメッセージ)では、理由の文に同じ「読むだけ」を重ねないべき(どの言語の文言でも同じ)
`req~screen.read-only-reason-once~1` P1
- Rationale: 式の列のセルを選ぶと「読むだけ: 式の列(読むだけ)」と同じ語が2回出ていたため(2026-11-25 の実地の操作で見つかった。理由の文の側にも「(読むだけ)」「なので読むだけ」が付いていた)
- Acceptance:
  - Given: `.base` の式の列のセル
  - When: 選ぶ
  - Then: 案内は「読むだけ: 式の列」。
- Acceptance:
  - Given: 値が3つで列が2つの CSV の行のセル
  - When: 選ぶ
  - Then: 案内は「読むだけ: この行は値が 3 個で、列は 2 個」。
- Acceptance:
  - Given: 読むだけの理由の文言(全部の言語)
  - When: 試験で読む
  - Then: どれも、その言語の前置きの語(日本語は「読むだけ」、英語は "read-only")を含まない。
- Needs: utest

#### ビューの帯のタブと検索の欄、設定の帯は、screen.display-options で隠せる
`req~screen.layout-hide-parts~1` P1
- Acceptance:
  - Given:
  - When: タブを出し、検索の欄だけ隠す(`search_bar = false`)
  - Then: ビューの帯の行は残り、タブとボタンが並び、検索の欄は無い。

#### タブと検索の欄を両方隠せば、ビューの帯の行ごと出さない(そのときの「+ 新規」はヘッダーの画面の型のメニューの左。cell-edit.new-note)
`req~screen.layout-no-view-bar~1` P1
- Acceptance:
  - Given:
  - When: タブ(`tabs = "never"`)と検索の欄(`search_bar = false`)を両方隠す
  - Then: ビューの帯の行もボタンも出ず、ヘッダーのすぐ下の行が列の見出し。

#### `groupBy` のまとまりは、見出しの行で開閉できるべき
`req~screen.group-fold~1` P1
- Acceptance:
  - Given:
  - When: 見出しで Enter
  - Then: そのまとまりの行が畳まれる。

#### 列は横にスクロールでき、選んだセルが常に見えているべき
`req~screen.h-scroll-follow~1` P1
- Acceptance:
  - Given: 列が画面に収まらない表で右へ移動
  - When:
  - Then: 選んだセルが見える位置まで表が流れる。

#### 列の幅は、列の見出しの境界のドラッグで変えてよい
`req~screen.h-scroll-drag-width~1` P1
- Acceptance:
  - Given:
  - When: 列の見出しの右の境界を押して右へ5桁ドラッグ
  - Then: その列の幅が5桁広がる。

#### 表の左のノートの欄は、末尾の `.md` を除いた名前(screen.note-name の `display.note_name` があればそれで抜き出すか削ったもの)を、設定 `display.note_path`(screen.note-path)に従ったフォルダとともに出すべき
`req~screen.note-column-path~1` P1
- Acceptance:
  - Given: `Tasks/a.md`・`Tasks/b.md` だけの表
  - When:
  - Then: 欄は `a`・`b`(`"common"`(既定)は読み込んだ全部の行に共通のフォルダを除いたパス)。`Tasks/a.md`・`Notes/c.md` → `Tasks/a`・`Notes/c`。`note_path = "name"`(フォルダを除いた名前だけ)で `todo/a.md`・`archive/todo/b.md` → `a`・`b`。`note_path = "full"`(根からのパス)で `Tasks/a.md`・`Tasks/b.md` → `Tasks/a`・`Tasks/b`。

#### 表の列に `file.name` があるときと `display.note_path = "off"` のときは、ノートの欄に名前と見出しを出さず、行の印(`!`・`~`)だけを出すべき
`req~screen.note-column-marks-only~2` P1
- Rationale: `"off"` は題名の列で行が分かる表のため
- Acceptance:
  - Given: `note_path = "off"` で `todo/20261015_a.md`
  - When:
  - Then: 欄に名前も見出しも出ず、外で変わった行には `!`。
- Acceptance:
  - Given: order に `file.name` のある `.base`
  - When:
  - Then: 欄に名前も見出し「ノート」も出ず、外で変わった行には `!` が出る。

#### ノートの欄に名前を出さないとき(`file.name` の列があるときと `"off"`)も、親子の字下げと ▾・▸、WBS の番号と進み具合(navigation.parent-child・navigation.wbs)は欄に残すべき
`req~screen.note-column-keeps-tree~1` P1
- Acceptance:
  - Given: 親子で並べた表で `"off"`
  - When:
  - Then: ▾・▸ と字下げは残る。

#### 検索(navigation.find)と、メッセージ・詳しく見る画面のノートの名前は、`display.note_path` によらず根からのパスのままにするべき
`req~screen.note-column-full-path-elsewhere~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `"name"` でも `\` の検索で `archive` と打つと `archive/todo/b.md` の行が当たる。

### 操作と見つけやすさ

#### キーは、vim 風(`h` `j` `k` `l`・`/`・`n` `N`・`gg` `G`)と、矢印・Home / End・PageUp / PageDown の両方が効かなければならない
`req~screen.keys-vim-and-arrows-both~1` P1
- Acceptance:
  - Given:
  - When: 表で `j` を押したときと ↓ を押したとき
  - Then: 同じ動き。

#### Obsidian Bases の主なキー(Enter・Tab / Shift+Tab・Ctrl+C / Ctrl+Z・Backspace)も効かなければならない
`req~screen.keys-vim-and-arrows-bases~2` P1
- Acceptance:
  - Given: 表
  - When: Enter
  - Then: 選んだセルの編集。
- Acceptance:
  - Given: 表
  - When: Tab / Shift+Tab
  - Then: 右 / 左のセル。
- Acceptance:
  - Given: 表
  - When: Ctrl+C
  - Then: コピー。
- Acceptance:
  - Given: 表
  - When: Ctrl+Z
  - Then: 直前の変更の取り消し。
- Acceptance:
  - Given: 表
  - When: Backspace
  - Then: 選んだセルを空にする。

#### すべてのキーは、設定ファイルで割り当て直せなければならない
`req~screen.keys-vim-and-arrows-remap~1` P1
- Acceptance:
  - Given:
  - When: 設定で `j` を外す
  - Then: ↓ だけが効く。

#### 表の画面では `O`(Shift+o)で、選んだ行の URL を開き(木の画面の `o` と同じ動作)、`[keys.table]` で `open_url` を別のキーに割り当て直せるべき
`req~screen.open-url-key~1` P1
- Rationale: 日常で使うサイトの表で、URL を開くのに操作の一覧かパレットを通るしかなく、自分でキーを割り当てようとしても「表のモードでは使えない」と無視されていたため(2026-11-25 に人が「両方」と決めた)
- Acceptance:
  - Given: `url: https://example.com/mail` の行を選んだ表
  - When: `O`
  - Then: その URL を OS の既定のブラウザに渡す(開く前にメッセージ行に URL を出す)。
- Acceptance:
  - Given: 設定に `[keys.table]` の `"u" = "open_url"`
  - When: 起動して `u`
  - Then: 警告を出さず、選んだ行の URL を開く。
- Needs: utest

#### キーの定義は1か所の表に置き、キーの振り分け・下の帯・ヘルプを、すべてその表から作るべき
`req~screen.key-table~1` P1
- Acceptance:
  - Given:
  - When: 表にキーを1つ足す
  - Then: 振り分け・下の帯・ヘルプに同時に出る。

#### 既定のキーの割り当てと、割り当て直し(screen.keys-vim-and-arrows)に書く動作の名前の一覧を、英語(`docs/keys.md`)と日本語(`docs/keys.ja.md`)の文書に置くべき
`req~screen.keys-docs-files~2` P1
- Acceptance:
  - Given: `docs/keys.md` と `docs/keys.ja.md`
  - When:
  - Then: どちらにも既定のキーと動作の名前の一覧がある。
- Acceptance:
  - Given:
  - When: 文書の動作の名前を設定の `[keys.table]` に書く
  - Then: その動作に割り当たる。

#### キーの文書は、モードごと(設定の `[keys.<モード>]` の名前)に、キー・動作の名前・説明を並べるべき
`req~screen.keys-docs-by-mode~1` P1
- Acceptance:
  - Given: `docs/keys.ja.md`
  - When:
  - Then: `table` などモードの名前の節ごとに、キー・動作の名前・説明を並べた行があり、説明が空の行は無い。

#### キーの表(screen.key-table)のモード・キー・動作と、動作の名前の一覧が文書と食い違えば、試験で落ちるべき
`req~screen.keys-docs-sync~2` P1
- Rationale: 文書が古くなるのを防ぐため
- Acceptance:
  - Given:
  - When: キーの表に `table` の `x` → 何かの動作を足して文書を直さない
  - Then: 試験が落ちる。
- Acceptance:
  - Given:
  - When: 文書から1行消す
  - Then: 落ちる。

#### 表のモードで、選んでいるセルに対して今使える操作の一覧(メニュー)を、キー(既定 `x`。screen.key-table のキーの表で割り当て直せる)で開けるべき
`req~screen.action-menu-open-key~2` P1
- Acceptance:
  - Given:
  - When: status のセルで `x`
  - Then: 操作の一覧が開く。
- Acceptance:
  - Given: `[keys.table] "m" = "action_menu"`
  - When:
  - Then: `m` で開く。

#### 表のモードで、セルの右クリックで、そのセルに対して今使える操作の一覧を開けるべき
`req~screen.action-menu-right-click~1` P1
- Acceptance:
  - Given:
  - When: セルを右クリック
  - Then: そのセルを選んで一覧が開く。

#### 操作の一覧は、セル・列・行・ビューとファイルの節に分けるべき
`req~screen.action-menu-sections~1` P1
- Acceptance:
  - Given:
  - When: status のセルで2行に印を付けて `x`
  - Then: 「セル」「列」「行」「ビューとファイル」の節の一覧が出る(セル: 編集・空にする・コピー・エディタで開く・詳細。列: この列で並べ替え・同じ値で絞る・同じ値を強調・隠す・幅・左右に移す。行: 印・選んだ行にまとめて入れる・行をコピー。ビューとファイル: 新しいノート・ビューの設定・`.base` に書き出す・保存)。

#### 操作の一覧の各行には、動作の名前と、その動作の今のキー(割り当て直したあとのキー)を添えるべき
`req~screen.action-menu-row-keys~1` P1
- Acceptance:
  - Given:
  - When: status のセルで `x`
  - Then: 「編集  Enter」「この列で並べ替え  s」が並ぶ。

#### 操作の一覧には、その場で使えない操作(読むだけのセルの編集と空にする、印の無いときのまとめて入れる、読むだけの起動での保存と新しいノートなど)を出さないべき
`req~screen.action-menu-hide-unavailable~1` P1
- Acceptance:
  - Given:
  - When: 読むだけのセルで `x`
  - Then: 「編集」「空にする」が無い。

#### 操作の一覧では、↑↓(`j` `k`)で選んで Enter で実行するべき
`req~screen.action-menu-enter~1` P1
- Acceptance:
  - Given:
  - When: 書けるセルで `x` のあと `j` 6回・`k` 1回 → 「この列で並べ替え」が選ばれ、Enter
  - Then: その列で並べ替わる。

#### 操作の一覧は、Esc で閉じるべき
`req~screen.action-menu-esc~1` P1
- Acceptance:
  - Given:
  - When: 一覧で Esc
  - Then: 何もせず閉じる。

#### 操作の一覧で、行に添えたキーを押しても、その操作を実行するべき
`req~screen.action-menu-item-key~1` P1
- Acceptance:
  - Given:
  - When: 一覧で `s`
  - Then: 「この列で並べ替え」を実行し、`s` と同じに並ぶ。

#### 操作の一覧は表の上に重ねる窓で出し、窓の外の表は見えたままにするべき
`req~screen.action-menu-overlay~1` P1
- Acceptance:
  - Given:
  - When: `x` で一覧を開く
  - Then: 窓の行のうち窓より左の桁に、開く前と同じ表の文字が残る。

#### 操作の一覧の項目をクリックしたら、その操作を実行するべき
`req~screen.action-menu-click-item~1` P1
- Acceptance:
  - Given:
  - When: 一覧の「この列で並べ替え」を左クリック
  - Then: `s` と同じに並ぶ。

#### 操作の一覧の窓の外をクリックしたら、一覧を閉じるべき
`req~screen.action-menu-click-outside~1` P1
- Acceptance:
  - Given:
  - When: 一覧の外(画面の左上の角)を左クリック
  - Then: 一覧が閉じる。

#### 操作の一覧の窓が入らない低い端末では、一覧を開かずに理由を出すべき
`req~screen.action-menu-too-low~1` P1
- Acceptance:
  - Given:
  - When: 80×8 の端末で `x`
  - Then: 一覧は開かず、メッセージ行に理由が出る。

#### 操作の一覧の「選んだ行にまとめて入れる」は、選んだ行のうち今の列を直せる行が1つも無ければ出さないべき
`req~screen.action-menu-bulk-hidden~1` P1
- Acceptance:
  - Given:
  - When: 式の列(読むだけ)のセルで2行に印を付けて `x`
  - Then: 「選んだ行にまとめて入れる」が無い。

#### 操作の一覧からの実行は、キーで直接行ったときと同じ振る舞いにするべき
`req~screen.action-menu-same-as-key~1` P1
- Acceptance:
  - Given:
  - When: 「この列で並べ替え」で Enter
  - Then: `s` と同じに並ぶ。

#### 前置きのキー(`g g` の `g` のように、キーの表で2つ以上の打鍵の頭になるキー。設定で足したものも含む)を押したら、すぐに下の帯を、続けて押せるキーと動作の名前の並びに替えるべき
`req~screen.prefix-hints-show~2` P1
- Acceptance:
  - Given:
  - When: 表で `g`
  - Then: 下の帯が「g → g 先頭の行」になる。
- Acceptance:
  - Given:
  - When: 設定で `"z z" = "bottom"` を足す
  - Then: `z` で「z → z 末尾の行」。

#### 前置きのあと、続きを押すか Esc を押したら、下の帯を元に戻すべき
`req~screen.prefix-hints-restore~2` P1
- Acceptance:
  - Given:
  - When: 表で `g` のあと続けて `g`
  - Then: 先頭の行へ動き、帯が元に戻る。
- Acceptance:
  - Given:
  - When: `g` のあと Esc
  - Then: 動かず帯が戻る。

#### 前置きのあとの Esc は前置きを取り消すだけにし、選択の解除や一覧を閉じることはしないべき
`req~screen.prefix-hints-esc-only~2` P1
- Acceptance:
  - Given:
  - When: 2行に印を付けて `g` のあと Esc
  - Then: 前置きだけが取り消され、2行の印は残る。
- Acceptance:
  - Given:
  - When: 操作の一覧を開いて `g` のあと Esc
  - Then: 一覧は開いたまま。

#### 操作の一覧の上では、一覧の項目に続く前置きだけを受けるべき
`req~screen.prefix-hints-menu~1` P1
- Acceptance:
  - Given:
  - When: 設定で `[keys.table] "z z" = "bottom"` を足し、操作の一覧の上で `z`
  - Then: 末尾の行は一覧の項目に無いので、一覧のままで、帯に「z →」が出ない。

#### `?` でヘルプを重ねて出し、先頭に今押せるキー、その下に全部のキーを並べ、色だけで意味を伝えない cell-view.not-color-only の印は見ただけでは分からないので、最後にセルと行の印(`∅`・`""`・`!`・`*`・`#`・`?`・`…⏎`、行の左の `>`・`+`・`!`・`~`)の意味を並べるべき
`req~screen.help~2` P1
- Acceptance:
  - Given:
  - When: 表で `?`
  - Then: 末尾の「印」の節に `∅` と `#` の意味。
- Acceptance:
  - Given:
  - When: 表で `?`
  - Then: 先頭に表の操作。

#### `?` のヘルプの先頭の「今押せるキー」の行は、↑↓ とクリックで選べ、Enter でヘルプを閉じてその操作をするべき(lazygit の `?` と同じく、覚えていない操作もヘルプから選んで使える)
`req~screen.help-run~1` P1
- Rationale: 操作を覚えなくても使えるようにするため。ヘルプでキーを探して、閉じてから押し直す手間をなくす(2026-11-25 に人が「lazygit みたいに操作方法を覚えなくても使えるようにしたい」と言った)
- Acceptance:
  - Given: `url` のある行を選んだ表
  - When: `?` で「今押せるキー」の「URL を開く」の行を選んで Enter
  - Then: ヘルプが閉じ、その行の URL を開く(`O` を押したのと同じ)。
- Acceptance:
  - Given: ヘルプを開いた
  - When: 「今押せるキー」より下の節(全部のキー・印の意味)の行で Enter
  - Then: 何もせず、ヘルプは開いたまま。
- Acceptance:
  - Given: ヘルプで行を選んだ
  - When: Esc
  - Then: 何もせずにヘルプを閉じる。
- Needs: utest

#### チェックを付け外しする一覧の窓(リストの値の付け外し・絞り込みの値の一覧・スターターのオプションの一覧)では、Enter と Space で選んだ行のチェックを付け外しし、窓の一番下の「[ 決める ]」の行での Enter かクリック、またはどの行でも Tab で決め、Esc でやめるべき(打った字で絞る欄に打っているあいだの Space は空白のまま)
`req~screen.checklist-keys~1` P1
- Rationale: チェックを入れるつもりで Enter を押すと、入れずに窓が閉じたため。Enter で入れ、決めるのは見える「[ 決める ]」の行か Tab にする(2026-11-25 に人が「Enter で入れる」と決めた)
- Acceptance:
  - Given: tags の候補に 仕事・開発・家 があり、仕事が付いたセル
  - When: 付け外しの窓で「開発」を選んで Enter
  - Then: 開発にチェックが付き、窓は開いたまま。
- Acceptance:
  - Given: 同じ窓で開発にチェックを付けた
  - When: Tab
  - Then: 窓が閉じ、tags は「仕事・開発」のためる変更。
- Acceptance:
  - Given: 同じ窓で開発にチェックを付けた
  - When: 「[ 決める ]」の行で Enter
  - Then: Tab と同じく決める。
- Acceptance:
  - Given: 同じ窓で開発にチェックを付けた
  - When: Esc
  - Then: 窓が閉じ、何もためない。
- Acceptance:
  - Given: 絞り込みの窓の status の値の一覧
  - When: 「done」で Enter、続けて Tab
  - Then: done のチェックが入り、「status が done を除く」(か「done だけ」。一覧の先頭の切り替えのとおり)の条件ができる。
- Needs: utest

#### 一覧の窓(値の候補・リストの値の付け外し・絞り込みの窓・並べ替えの窓・切り替えの一覧・引き出し・画面の型のメニュー・操作の一覧)の下の縁に、その窓で効く主なキーを「キー 動作」で並べ、クリックでその動作をし、幅が足りなければ後ろから省くべき
`req~screen.window-key-hints~1` P1
- Rationale: 下の帯は画面の端にあって目が届きにくく、窓の中で何を押せばよいかが分からなかったため。窓のそばにキーを出し、押せるようにする(2026-11-25 に人が「lazygit みたいに」と言った)
- Acceptance:
  - Given: リストの値の付け外しの窓
  - When: 窓を描く
  - Then: 下の縁に「Enter 入れる・外す」「Tab 決める」「Esc やめる」が並ぶ。
- Acceptance:
  - Given: 同じ窓
  - When: 下の縁の「Tab 決める」をクリック
  - Then: Tab と同じく決める。
- Acceptance:
  - Given: 幅の狭い窓
  - When: 窓を描く
  - Then: 入らないキーは後ろから省き、縁の線と字が重ならない。
- Needs: utest

#### コマンドの名前であいまいに探して実行できるパレット(`:` と Ctrl+P)を持つべき
`req~screen.palette-fuzzy~2` P1
- Acceptance:
  - Given:
  - When: `:` で「save」と打つ
  - Then: 保存が候補の先頭に出る。
- Acceptance:
  - Given:
  - When: Ctrl+P で開いて「save」と打つ
  - Then: 同じ。

#### パレットには、各コマンドの割り当てキーも出すべき
`req~screen.palette-keys~1` P1
- Acceptance:
  - Given:
  - When: `:` で「save」と打つ
  - Then: 保存の候補の横に Ctrl+S。

#### 選んでいない行をクリックしたら、その行を選ぶだけにするべき
`req~screen.mouse-click-select~1` P1
- Acceptance:
  - Given:
  - When: 別の行のセルを1回クリック
  - Then: 選ぶだけで編集は始まらない。

#### 選んだ行のセルをクリックしたら、そのセルのその場の編集を始めるべき(行のファイル名(screen.click-name)と、開ける URL のセル(screen.url-click-open)を除く)
`req~screen.mouse-click-edit~2` P1
- Acceptance:
  - Given:
  - When: 選んだ行の status のセルをクリック
  - Then: そのセルの編集が始まる。

#### ホイールで上下に動けるべき
`req~screen.mouse-wheel~1` P1
- Acceptance:
  - Given:
  - When: ホイールの下・上
  - Then: 次・前の行。

#### 表の画面では、横のホイール(トラックパッドの横のスワイプ)と shift を押したホイールで、`h`・`l` と同じに左右の列へ動けるべき
`req~screen.mouse-h-wheel~2` P1
- Rationale: 縦しか拾わないと、収まらない列を指で流せないため
- Acceptance:
  - Given: 表
  - When: 横のホイールを右
  - Then: 右の列。
- Acceptance:
  - Given: 表
  - When: 横のホイールを左
  - Then: 左の列。
- Acceptance:
  - Given: 表の一番左の列
  - When: 横のホイールを左
  - Then: そのまま。
- Acceptance:
  - Given: 表
  - When: shift+ホイールの下
  - Then: 右の列。
- Acceptance:
  - Given: 表
  - When: shift+ホイールの上
  - Then: 左の列。
- Acceptance:
  - Given: 列が画面に収まらない表
  - When: 右へ流し続ける
  - Then: 選んだセルが見える位置まで表が流れる(screen.h-scroll)。
- Acceptance:
  - Given: 本物の実行ファイルの表
  - When: 端末の横のスクロールの報告(SGR の 67)を送る
  - Then: 右の列。

#### 表の画面でない画面(ヘルプなど)では、横のホイールで何もしないべき
`req~screen.mouse-h-wheel-other~1` P1
- Acceptance:
  - Given:
  - When: ヘルプで横のホイール
  - Then: 何も変わらない。

#### 表の画面でない画面(ヘルプなど)では、shift を押したホイールを縦のホイールのままにするべき
`req~screen.mouse-shift-wheel-other~1` P1
- Acceptance:
  - Given:
  - When: ヘルプで shift+ホイールの下
  - Then: ヘルプが下へ流れる。

#### 選んだ行のノートをエディタで開けるべき
`req~screen.editor-open~1` P1
- Acceptance:
  - Given:
  - When: 選んだ行で `e`
  - Then: そのノートのパスでエディタが起動する。

#### エディタから戻ったら、開いたノートを読み直すべき
`req~screen.editor-reload~1` P1
- Acceptance:
  - Given:
  - When: `EDITOR="code -w"` で開く
  - Then: 戻ると行が更新。

#### エディタは、設定(cli.config-file の設定ファイル)の `editor`、環境変数 `$VISUAL`、`$EDITOR` の順で空でない最初のものにするべき
`req~screen.editor-choice~2` P1
- Acceptance:
  - Given: 設定に `editor = "nvim"` と `EDITOR=vim`
  - When:
  - Then: nvim で開く。
- Acceptance:
  - Given: 設定が無く `VISUAL=hx`・`EDITOR=vim`
  - When:
  - Then: hx。設定の `editor = ""` は無いのと同じ。

#### 設定の `editor`・`$VISUAL`・`$EDITOR` のどれも無ければ、エディタを `vi`(Windows では `notepad`)にするべき
`req~screen.editor-fallback~1` P1
- Rationale: Windows では vi が入っていないのがふつうのため
- Acceptance:
  - Given: どれも無い
  - When:
  - Then: Unix では vi、Windows では notepad(OS ごとの既定は文字の値 `"vi"`・`"notepad"` で比べる)。

#### エディタの値が引数つき(`code -w` など)でも動くべき
`req~screen.editor-args~1` P1
- Acceptance:
  - Given: `EDITOR="code -w"`
  - When:
  - Then: `code` に `-w` とノートのパスを渡して起動する。

#### エディタは、シェルを通さずに起動するべき
`req~screen.editor-no-shell~1` P1
- Acceptance:
  - Given: `EDITOR="ed $HOME;"`
  - When:
  - Then: `ed` に `$HOME;` を展開せずにそのまま1つの引数で渡す。

#### ノートのパスは、エディタに1つの引数として渡すべき
`req~screen.editor-path-one-arg~1` P1
- Acceptance:
  - Given:
  - When: 空白を含む `/tmp/my notes/a b.md` のノートを開く
  - Then: パスは分けずに1つの引数で渡る。

#### キーは、モード(表・編集・パレット・保存の確認・終了の確認・ヘルプ・検索・絞り込み・詳細の表示・ビューの設定・設定の値の入力・設定の帯・リストの選択・操作の一覧(screen.action-menu)・頻度表(navigation.frequency))ごとに割り当てるべき
`req~screen.key-modes-per-mode~1` P1
- Acceptance:
  - Given:
  - When: 同じ Enter
  - Then: 表ではセルの編集、詳細の表示では選んだプロパティの編集。

#### 同じモードの中で、1つのキーに2つの動作を割り当てるべきでない
`req~screen.key-modes-no-dup~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: キーの表をモードごとに調べて重複が無い。

#### 表のモードの既定のキーは、Enter = 編集(グループの見出しの行では開閉、チェックボックスのセルでは切り替え)、Space = 行の印、`K` = 詳細の表示、Ctrl+R = やり直しにするべき
`req~screen.key-modes-table-defaults~2` P1
- Acceptance:
  - Given: 表
  - When: Enter
  - Then: セルの編集。
- Acceptance:
  - Given: 表
  - When: Space
  - Then: 行の印。
- Acceptance:
  - Given: 表
  - When: `K`
  - Then: 詳細の表示。
- Acceptance:
  - Given: 表
  - When: Ctrl+R
  - Then: やり直し。

#### 編集のモードでは、Ctrl+R の既定を、編集前の値に戻す動作にするべき
`req~screen.key-modes-edit-defaults~1` P1
- Acceptance:
  - Given:
  - When: status が `todo` のセルの編集で `done` と打ち替えて Ctrl+R
  - Then: 入力が `todo` に戻る。

#### 詳細の表示のモードの既定のキーは、Enter = 選んだプロパティの編集(navigation.detail)、Esc と `K` = 閉じるにするべき
`req~screen.key-modes-detail-defaults~1` P1
- Acceptance:
  - Given:
  - When: 詳細の表示で Enter
  - Then: 選んだプロパティの編集、Esc で閉じる。`K` でも閉じる。

#### パレット(`:` で開く)は、行番号(`:120`)と `w`(保存)と `q`(終了)と `wq`・`x`(保存して終了)も受け付けるべき
`req~screen.key-modes-palette-commands~3` P1
- Acceptance:
  - Given: 表
  - When: パレットで `:120`
  - Then: 120行目が選ばれる。
- Acceptance:
  - Given: 表
  - When: パレットの `w`
  - Then: 保存(差分の確認へ)。
- Acceptance:
  - Given: 表
  - When: パレットの `q`
  - Then: 終了(ためた変更があれば終了の確認)。
- Acceptance:
  - Given: 未保存 1
  - When: パレットの `wq`
  - Then: 差分の確認が出て、確定で書いて終わる(差分の確認から戻ると、書かず終わらず表に戻る)。`x` も同じ。
- Acceptance:
  - Given: ためた変更が無い
  - When: パレットの `wq`
  - Then: そのまま終わる。
- Acceptance:
  - Given: 読むだけの起動(cli.readonly)
  - When: パレットを開く
  - Then: `wq`・`x` は `w` と同じく候補に出ない。

#### 表のモードで、日本語入力(IME)が有効なまま打った全角の英数字と記号が届いたら、半角のキーとして扱うべき
`req~screen.ime-fullwidth~1` P1
- Acceptance:
  - Given:
  - When: 表で全角の ｊ を送る
  - Then: j と同じ動き。

#### 表のモードで、日本語の読点と中黒(`、`・`・`)は、同じキーの位置の `,`・`/` として扱うべき
`req~screen.ime-punct~1` P1
- Acceptance:
  - Given:
  - When: 表で `、` を送る
  - Then: `,`(screen.default-keys の同じ値の行だけに絞る)と同じ動き。`・` → `/`(検索)と同じ動き。

#### 入力ボックスでは、端末のカーソルを入力の位置に置き、変換の窓が入力の位置に出るようにするべき
`req~screen.ime-cursor~1` P1
- Acceptance:
  - Given:
  - When: 検索の欄に「会議」と打つ
  - Then: 端末のカーソルは「会議」のすぐ後ろ(欄の語の末尾)にある。

#### 表のモードで、ほかの要件で既定のキーを決めていない動作の既定は、Tab / Shift+Tab = 右 / 左のセルへ移る、Backspace と Delete = 選んだセルの値を空にする(ためる変更。cell-edit.empty-is-null。キーの無いセルでは何もせず、キーを足さない)、`q` = 終わる(write-back.quit-confirm)、Ctrl+A = 全部の行を選ぶ、Esc = 選択を解く(選択が無ければ検索を解く。navigation.select-rows・navigation.search)、`*` = 選んだセルと同じ値の行を強調する、`,` = 同じ値の行だけに絞る(navigation.same-value)、Ctrl+G = 読み込みの中止(base-view.large-load)とするべき
`req~screen.default-keys~2` P1
- Acceptance:
  - Given:
  - When: 表で Backspace
  - Then: そのセルが null のためる変更になり「未保存 1」。読むだけのセルでは理由を表示して何もしない。
- Acceptance:
  - Given:
  - When: キーの無いセルで Backspace
  - Then: 何も変わらず「未保存 0」。
- Acceptance:
  - Given:
  - When: 読み込み中に Ctrl+C
  - Then: コピーになり、読み込みは止まらない。

#### 行のファイル名(左の行の名前の欄と、`file.name` の列のセル)をクリックしたとき、選んでいない行なら選ぶだけにし(screen.mouse)、選んだ行なら `e`(screen.editor)を押したのと同じ振る舞いで、そのノートをエディタで開くべき
`req~screen.click-name~2` P1
- Acceptance:
  - Given:
  - When: 別の行のファイル名を1回クリック
  - Then: その行を選ぶだけで開かない。
- Acceptance:
  - Given:
  - When: 選んだ行のファイル名をクリック
  - Then: エディタが起動し、戻るとその行を読み直す。`EDITOR` が空のときと読むだけの起動のときも、`e` と同じ結果。

#### 表の画面で、値の全体が `http://` か `https://` の URL(tree.open-url-scheme で開いてよい形)のセルは、選んでいないセルのクリックではそのセルを選ぶだけにし、選んでいるセルをもう一度クリックしたら、`O`(screen.open-url-key)と同じ守り(tree.open-url-show・tree.open-url-disguised)で、そのセルの切る前の URL の全体を開くべき
`req~screen.url-click-open~1` P1
- Rationale: 端末を束ねる道具(Herdr など)の中では、切って見せた URL に付けた行き先(cell-view.url-full-target の OSC 8)が外の端末へ届かず、外の端末が見えている文字から途中までの URL を開いたため(2026-11-25 の報告)。⌘+クリックは外の端末が受け取って mdgrid に届かないので、mdgrid が受けるふつうのクリックで開く。木の画面の右の行(tree.mouse-row-reclick)・関係マップの表と同じく、1回目で選び、2回目で開く。縦に並べた表(ページの区画)の表も同じ
- Acceptance:
  - Given: 別の行を選んでいて、url が `https://docs.example.com/guides/workspace-setup/advanced-configuration/a-very-long-page-name`(url の列の幅 24 桁で `…` に切って見せる)の行がある表
  - When: その行の url のセルを1回クリック
  - Then: その url のセルが選ばれ、開く口に何も渡らず、編集も始まらない。
- Acceptance:
  - Given: 同じ行の title のセルを選んでいる表
  - When: その行の url のセルを1回クリック
  - Then: url のセルが選ばれるだけで、編集は始まらず、開く口に何も渡らない。
- Acceptance:
  - Given: その url のセルを選んでいる表
  - When: そのセルをもう一度クリック
  - Then: 開く前にメッセージ行に URL が出て、開く口に切る前の URL の全体が1回渡り、編集は始まらない。
- Acceptance:
  - Given: url が `https://example.com/files/report.pdf.exe` のセルを選んでいる表
  - When: そのセルをもう一度クリック
  - Then: 開く口に渡らず、実行ファイル(.exe)を指すので開かないという理由が出る。
- Acceptance:
  - Given: url が `file:///etc/passwd` のセルを選んでいる表
  - When: そのセルをもう一度クリック
  - Then: 開く口に渡らず、URL のセルとして扱わずに、そのセルの編集が始まる(screen.mouse-click-edit)。
- Acceptance:
  - Given: 2つの表を縦に並べたページで、下の表の url のセルを選んでいる
  - When: そのセルをもう一度クリック
  - Then: 開く口にそのセルの URL の全体が渡る。
- Needs: utest, stest

#### 表の画面の `O`(screen.open-url-key)は、選んでいるセルの値が開ける URL なら、行のほかの列の URL より先に、そのセルの URL を開くべき
`req~screen.open-url-cell-first~1` P1
- Rationale: 開ける URL のセルを選んだときの案内(screen.url-click-hint)が「もう一度クリックか O で開く」と言うので、URL の列が2つある行で `O` が別の列の URL を開くと、案内とクリックの結果が食い違うため
- Acceptance:
  - Given: url が `https://example.com/a`、mirror が `https://example.com/b` の行で、mirror のセルを選んでいる表(列は url・mirror の順)
  - When: `O`
  - Then: 開く口に `https://example.com/b` が渡る。
- Acceptance:
  - Given: 同じ行で、URL でない title のセルを選んでいる表
  - When: `O`
  - Then: 開く口に行の最初の URL `https://example.com/a` が渡る。
- Needs: utest

#### 開ける URL(値の全体が `http://` か `https://` の URL で、tree.open-url-disguised で止めないもの)を選んでいるとき(表の画面のセル・木の画面の右の行・詳細の表示の項目)、ほかの知らせが無ければ、メッセージ行に「URL: もう一度クリックか O で開く」(キーはその画面で URL を開くキー)の案内を出し、幅に入らなければ案内を出さないべき
`req~screen.url-click-hint~1` P1
- Rationale: もう一度のクリックで開くこと(screen.url-click-open)はクリックするまで見えないので、マウスで使う人にも開き方が分かるようにする。キーを外したときはクリックだけを言う
- Acceptance:
  - Given: 幅 100 の表で、url が `https://example.com/mail` のセルを選んでいる
  - When: 表を描く
  - Then: メッセージ行に `URL: もう一度クリックか O で開く` が出る。
- Acceptance:
  - Given: 同じ行の、URL でない title のセルを選んでいる
  - When: 表を描く
  - Then: メッセージ行に案内は出ない。
- Acceptance:
  - Given: url が `https://example.com/files/report.pdf.exe` のセルを選んでいる
  - When: 表を描く
  - Then: メッセージ行に案内は出ない(開かない URL なので)。
- Acceptance:
  - Given: 幅 20 の表で、url のセルを選んでいる
  - When: 表を描く
  - Then: メッセージ行に案内の切れ端も出さない。
- Acceptance:
  - Given: 木の画面で、url が `https://example.com/alpha` の右の行を選んでいる
  - When: 木の画面を描く
  - Then: メッセージ行に `URL: もう一度クリックか o で開く` が出る。
- Acceptance:
  - Given: 詳細の表示で、url の項目を選んでいる
  - When: 詳細の表示を描く
  - Then: メッセージ行に `URL: もう一度クリックか O で開く` が出る。
- Acceptance:
  - Given: 設定で表の `O` を外し(`[keys.table]` の `"O" = "none"`)、url のセルを選んでいる
  - When: 表を描く
  - Then: メッセージ行に `URL: もう一度クリックで開く` が出る。
- Needs: utest, stest

### vim モード

#### 設定の `[keys]` の `vim` を `true` にしたときだけ vim モードにし、既定は `false` にするべきで、`false` のときはキーの振る舞いを今と変えないべき
`req~screen.vim-setting~2` P1
- Rationale: vim に慣れた人には入力の欄のモードと行の演算子を足し、慣れない人には、Esc で入力が閉じない・`d` で行が消えるといった驚きを与えないため(2026-11-20 に人が挙げ、2026-11-21 に「世間の vi モードを調べて」と任せた。シェルの vi モード・Obsidian・表計算の sc-im も、設定で入れる形)
- Acceptance:
  - Given: 設定が無い
  - When:
  - Then: セルの入力の欄で Esc を押すと、今どおり入力を取りやめて閉じる。
- Acceptance:
  - Given: `vim = true`
  - When:
  - Then: 同じ Esc で入力の欄がノーマルのモードになり、閉じない。
- Needs: stest

#### vim モードでは、1行の入力の欄(セルの編集・検索・絞り込み・パレット・設定の値の入力・新しいノートの名前)を挿入のモードで始め、Esc でノーマルのモードにし、ノーマルのモードの Esc で入力を取りやめ、Enter はどちらのモードでも決めるべき
`req~screen.vim-input-modes~2` P1
- Rationale: シェルの vi モードと同じく、打ち始めはそのまま文字を入れ、直すときだけノーマルのモードに移れるようにするため
- Acceptance:
  - Given:
  - When: `vim = true` で title のセルを Enter
  - Then: 打った字が入る。
- Acceptance:
  - Given:
  - When: Esc
  - Then: 欄は開いたまま。
- Acceptance:
  - Given:
  - When: もう一度 Esc
  - Then: 取りやめて閉じ、値は前のまま。
- Acceptance:
  - Given:
  - When: ノーマルのモードで Enter
  - Then: 決まる。

#### vim モードの入力の欄は、下の帯に今のモード(挿入・ノーマル)を出し、カーソルの形も変えるべき
`req~screen.vim-mode-shown~2` P1
- Rationale: 今打った字が文字として入るのかキーとして効くのかを、いつも見て分かるようにするため
- Acceptance:
  - Given:
  - When: `vim = true` で入力の欄を開く
  - Then: 下の帯に「-- 挿入 --」。
- Acceptance:
  - Given:
  - When: Esc
  - Then: 「-- ノーマル --」。

#### vim モードの入力の欄のノーマルのモードでは、動き(`h`・`l`・`0`・`$`・`w`・`b`・`e`)、挿入への移り(`i`・`a`・`I`・`A`)、消す(`x`・`X`・`dw`・`db`・`d$`・`D`・`dd`)、替える(`cw`・`c$`・`C`・`cc`)、取り消し(`u`)が、vim と同じに効くべき
`req~screen.vim-normal-keys~2` P1
- Rationale: 入力の欄の中の直しを、vim と同じ指の動きで行うため(シェルの vi モードと Obsidian の vim モードにもある、よく使うキーの組)
- Acceptance:
  - Given:
  - When: `会議の見積もり` の欄で Esc・`0`・`dw` → `の見積もり`(語の切れ目は vim の `w` と同じ)。`$`・`x`
  - Then: 最後の1字が消える。
- Acceptance:
  - Given:
  - When: `会議の見積もり` の欄で Esc・`0`・`dw` → `の見積もり`(語の切れ目は vim の `w` と同じ)。`cc`
  - Then: 欄が空になり挿入のモード。
- Acceptance:
  - Given:
  - When: `会議の見積もり` の欄で Esc・`0`・`dw` → `の見積もり`(語の切れ目は vim の `w` と同じ)。`u`
  - Then: 1つ前の形に戻る。
- Needs: utest

#### vim モードの表の画面では、`dd`(範囲を選んでいるときは `d`)で、印を付けた行(印が無ければ選んでいる行)のノートを、パレットの「ノートを消す」(cell-edit のノートの削除)と同じく、名前を確かめてからごみ箱へ動かすべき
`req~screen.vim-dd~2` P1
- Rationale: vim の行を消すキーで、要らないノートを片付けるため。確かめと取り消しは「ノートを消す」と同じにし、キーによって消え方が変わらないようにする
- Acceptance:
  - Given:
  - When: `vim = true` で a.md の行で `dd` → 名前の確かめ → `y`
  - Then: a.md が `.trash/` へ動き、表から消える。
- Acceptance:
  - Given:
  - When: `u`
  - Then: 元の場所に戻る。
- Acceptance:
  - Given:
  - When: `v` で2行選んで `d`
  - Then: 2つともごみ箱へ、`u` 1回で2つとも戻る。
- Acceptance:
  - Given:
  - When: 読むだけの起動(cli.readonly)
  - Then: 何もせず理由を出す。
- Acceptance:
  - Given:
  - When: `vim` が無い設定で `dd`
  - Then: 何も変わらない。

#### vim モードの表の画面では、`p` で、mdgrid の中で最後に写した値(`y` のセルか `Y` の行)を、選んでいるセル(行を写したときは印を付けた行か選んでいる行の同じ名前の列)に、ためる変更として入れるべき
`req~screen.vim-paste~2` P1
- Rationale: 写した値を別の行へ、打ち直さずに入れるため(sc-im・ranger と同じ `y` と `p` の組)
- Acceptance:
  - Given:
  - When: `vim = true` で a.md の status(done)で `y`、b.md の status で `p`
  - Then: b.md の status が done のためる変更。
- Acceptance:
  - Given:
  - When: a.md の行で `Y`、c.md の行で `p`
  - Then: c.md の、a.md と同じ名前の書ける列が a.md の値になり、`file.name` と計算の列は変わらない。
- Acceptance:
  - Given: 何も写していない
  - When:
  - Then: 何もせず理由を出す。

#### vim モードの表の画面では、`.` で、直前に1つのセルに入れた値を、選んでいる行の同じ列にもう一度入れるべき
`req~screen.vim-repeat~2` P1
- Rationale: 同じ値(status を done にするなど)を下の行へ続けて入れるのを、1打で繰り返すため(vim の `.` と同じ考え)
- Acceptance:
  - Given:
  - When: `vim = true` で a.md の status を done にしたあと `j` で b.md へ移り、選んでいる列が due でも `.`
  - Then: b.md の status が done のためる変更。
- Acceptance:
  - Given: まだ何も直していない
  - When:
  - Then: 何も変わらない。

#### パレットの `q!` は、ためた変更を確かめずに捨てて終わるべき(vim モードによらない)
`req~screen.quit-discard~2` P1
- Rationale: vim と同じ打ち方で、ためた変更を書かずにすぐ終わるため。パレットの `q`(screen.key-modes-palette-commands)はこれまでどおり確かめる
- Acceptance:
  - Given:
  - When: 未保存 2 でパレットの `q!`
  - Then: 確かめずに終わり、ノートのファイルは前のまま。
- Acceptance:
  - Given:
  - When: 未保存 0
  - Then: そのまま終わる。

#### vim モードでも、数字のキーは回数にせず、名前の条件の切り替え(condition.digit-keys)に使うべき
`req~screen.vim-no-count~1` P1
- Rationale: 数字だけのキーは条件の入れ切りに使うと決めたため(理由は condition.digit-keys)。行を大きく動かすのは `gg`・`G`・`Ctrl+d`・`Ctrl+u`・`:120` で足りる
- Acceptance:
  - Given:
  - When: `vim = true` で `5` `j`
  - Then: 5 に条件があればその条件を切り替え、`j` で1行だけ下へ。

#### vim モードで足すキーも、ほかのキーと同じく設定で割り当て直せ(screen.keys-vim-and-arrows-remap)、ヘルプとキーの文書(screen.keys-docs-files)に出すべき
`req~screen.vim-remap~1` P1
- Rationale: vim モードのキーが、人の割り当てやほかの道具のキーとぶつかったときに外せるようにし、何が効くかを調べられるようにするため
- Acceptance:
  - Given:
  - When: `vim = true` で `[keys.table]` の `delete_note` を外す
  - Then: `dd` が何もしない。ヘルプに `dd`・`p`・`.` の行(`vim` が無い設定では出ない)。

### 設定と見た目の状態

#### 設定の `display.tabs` で、ビューのタブの行を `"always"`(いつも出す。既定)・`"auto"`(ビューが2つ以上のときだけ出す)・`"never"`(出さない)にできるべき
`req~screen.tabs-setting-values~2` P1
- Acceptance:
  - Given: 既定(always)
  - When:
  - Then: 今のまま。
- Acceptance:
  - Given: `"never"`
  - When:
  - Then: タブの行が無い。
- Acceptance:
  - Given:
  - When: ビューを保存していないフォルダを `[display] tabs = "auto"` で開いてビューを1つ保存する
  - Then: タブの行に2つ。
- Acceptance:
  - Given: 知らない値
  - When:
  - Then: 警告して always。

#### `display.tabs` が `"auto"` でビューが1つのときは、ビューの帯にタブを出さず(検索の欄も出さない設定なら、ビューの帯の行も出さずに表に使い)、ヘッダーにビューの名前も足さない
`req~screen.tabs-setting-auto-single~1` P1
- Rationale: 1つしか無いビューの名前は見分けの役に立たないため
- Acceptance:
  - Given:
  - When: ビューを保存していないフォルダを `[display] tabs = "auto"` と `search_bar = false` で開く
  - Then: タブの行もヘッダーのビューの名前も無く、表が1行上から始まる。

#### ビューの設定の表示の区画(screen.display-options)のタブの項目は、`display.tabs` の always・auto・never の3つを順に切り替えるべき
`req~screen.tabs-setting-cycle~1` P1
- Acceptance:
  - Given:
  - When: 表示の区画のタブの項目を押していく
  - Then: always・auto・never の順に変わる。

#### `display.tabs` でタブを出さないときも、タブの切り替えのキーは効くべき
`req~screen.tabs-setting-hidden-keys~1` P1
- Acceptance:
  - Given: `[display] tabs = "never"`
  - When:
  - Then: タブの行が無く、`]` で切り替わる。

#### 色を使う表示(cell-view.no-color)では、設定の `look.cells` が `"rich"`(既定)のとき、表のセルを値の型に合わせた部品で見せるべき: 真偽の値、リストの要素、種類の少ない短い文字の列(status など)の値、リンク(relations.link-display)、列の見出しの型の印を、screen.style-parts の形(`check`・`tags`・`status`・`links`・`icons`)で見せる
`req~screen.rich-cells-parts~2` P1
- Acceptance:
  - Given: 色のある表示で、既定の組(sumi)
  - When: done(真偽)・tags `[a, b]`・status(doing・todo・done が何行も)の列の表を出す
  - Then: done の列は `✓`/`·`、tags は `a · b`、status は `● doing`、見出しに型の印なし。
- Acceptance:
  - Given: 色のある表示で、組 classic
  - When: 同じ表を出す
  - Then: `☑`/`☐`・札の `a` と `b`・札の `doing`・`◷ due`。
- Acceptance:
  - Given: `look.cells` の知らない値
  - When: 起動する
  - Then: 警告して rich。

#### 表のセルの部品の色は値ごとに決まった色にし、どの値も文字はそのまま見せる(色だけで値を伝えない。cell-view.not-color-only)
`req~screen.rich-cells-value-color~1` P1
- Acceptance:
  - Given:
  - When: 組 classic の色のある表示で status が doing の行が2つ
  - Then: 2つの札の地が同じ色で、どちらの札にも `doing` の文字がある。

#### 部品の種類ごとに部品を使わないときは、screen.style-parts のその部品の形を文字のままの形(`status`・`tags`・`links` は `plain`、`check` は `text`、印は `icons = false`)にする
`req~screen.rich-cells-per-kind~2` P1
- Acceptance:
  - Given: `[look.style] check = "text"`
  - When:
  - Then: 真偽だけ文字のまま。
- Acceptance:
  - Given: `links = "plain"`
  - When:
  - Then: リンクがアクセントの色でない。

#### 列ごとの見せ方は `[look.columns]` に `"rich"`・`"plain"`・`"chip"`(自動で部品にならない列も部品に)で書け、部品の種類ごとの形より優先する
`req~screen.rich-cells-per-column~2` P1
- Acceptance:
  - Given: `[look.columns]` で `status = "plain"`
  - When: 表を出す
  - Then: status の列だけ文字のまま。
- Acceptance:
  - Given: `[look.columns]` で `owner = "chip"`
  - When: 表を出す
  - Then: owner の値が札。

#### `look.cells = "plain"` と色を使わない表示では、表のセルを今の見た目にする
`req~screen.rich-cells-plain~1` P1
- Acceptance:
  - Given: `[look] cells = "plain"` か `--no-color`
  - When:
  - Then: 今の見た目(`true`・`[a, b]`)。

#### 表のセルを部品で見せても、ファイルに書く値と、編集・検索・出力(`--print`)の値は変えない
`req~screen.rich-cells-values-kept~1` P1
- Acceptance:
  - Given: 組 classic の色のある表示で done が `☑` のセル
  - When:
  - Then: `\` の検索で `true` と打つと当たり、`--print` の done の列は `true`。

#### 種類の少ない短い文字の列の値を札にするか(screen.rich-cells-parts)は、今のビューの行でなく表の全部の行(`.base` の全体の絞り込みを通った行。ビューの絞り込みの前)で決め、同じ表の同じ列はどのビュー(タブ)でも同じ見せ方にするべき(`[look.columns]` で決めた列はそのとおり)
`req~screen.rich-cells-table-wide~1` P1
- Rationale: 見えている行で値がくり返すかで決めていたので、同じ列が、行の多いタブでは札、行の少ないタブでは文字になり、タブを替えるたびに見え方が変わったため(2026-11-25 の台帳3つの実地の操作で見つかった)
- Acceptance:
  - Given: status が todo・todo・doing・done の4行の表と、全部の行のビュー「全部」と、`status != "todo"` で絞ったビュー「残り」(色のある表示)
  - When: ビュー「残り」を開く
  - Then: status の doing と done のセルは札(ビュー「全部」と同じ)。
- Acceptance:
  - Given: `.base` の全体の絞り込みが `file.inFolder("A")` で、A の中の行の status がくり返さず(doing・done)、A の外の行に todo がくり返す表
  - When: 表を描く
  - Then: status のセルは文字のまま(全体の絞り込みの外の行は数えない)。
- Acceptance:
  - Given: 1つ目の受け入れと同じ表に `[look.columns] status = "plain"`
  - When: どちらのビューを開く
  - Then: status のセルは文字のまま。
- Needs: utest

#### 設定の `[look.style]` で、表と窓の部品の形を部品の種類ごとに選べるべき
`req~screen.style-parts-choose~2` P1
- Acceptance:
  - Given: `[look.style] status = "chip"`
  - When:
  - Then: 今までの札。
- Acceptance:
  - Given: `frames = "ascii"`
  - When:
  - Then: 枠が ASCII。次の項目と値はどれも警告せずに効く: くり返す短い値(`status`: `dot`・`shape`・`text`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`)、リスト(`tags`: `dots`・`hash`・`brackets`・`pill`・`tint`・`solid`・`soft`・`chip`・`plain`。`solid` は値の色をそのまま地に塗った、透けない札)、真偽(`check`: `box`・`tick`・`bracket`・`text`)、選び(`select`: `bar`・`cross`・`tint`・`outline`・`fill`・`reverse`)、区切り(`rules`: `none`・`header`・`columns`・`grid`)、ビューのタブ(`tabs`: `underline`・`pill`・`segment`・`brackets`・`dim`)、窓の枠(`frames`: `rounded`・`square`・`heavy`・`ascii`・`none`)、下の帯(`band`: `keys`・`boxed`・`quiet`)、リンク(`links`: `accent`・`plain`)、列の見出しの型の印(`icons`)。

#### `look.preset` に組の名前(`sumi`・`slate`・`saas`・`paper`・`grid`・`classic`・`dozy-pink`)を書けば部品をまとめて選び、`[look.style]` の項目はそれを上書きする
`req~screen.style-parts-preset~2` P1
- Acceptance:
  - Given: `[look] preset = "saas"`
  - When:
  - Then: 淡い地の丸い札と切り替えの枠のタブ。
- Acceptance:
  - Given: `preset = "dozy-pink"`(テーマ dozy-pink に合う柔らかい組)
  - When:
  - Then: 丸い札の状態とタグ・左の線の選び・線なし・塗ったタブ。
- Acceptance:
  - Given:
  - When: `preset = "saas"` に `[look.style] status = "chip"` を足す
  - Then: status だけ今までの札。

#### 部品の形の既定は組 `sumi`(点と文字・`·` で区切るタグ・`✓`・左の線の選び・見出しの下の線・下線のタブ・角の丸い枠・薄い下の帯・印なし)にする
`req~screen.style-parts-default~1` P1
- Acceptance:
  - Given: 既定(設定なし)で色のある表示
  - When:
  - Then: status の値は `● doing`(点だけに色)、tags は `ui · web`、選んだ行の左の印 `>` はアクセントの色の太字(印の文字は変えない。screen.modern-look)。

#### 部品の形の `pill` は丸い端の字(U+E0B6・U+E0B4)を使うので、丸い端を描けるときだけ使い、そうでなければ `soft` で描くべき
`req~screen.style-parts-pill~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `[look.style] status = "pill"` で `terminal.nerd_font` なし(`TERM_PROGRAM` が ghostty でも WezTerm でもない)→ `soft`。

#### 丸い端を描けるかは設定の `terminal.nerd_font` で決め、`true`(Nerd Font を使っている)・`false`・`"auto"`(既定)を書ける
`req~screen.style-parts-nerd-font~1` P1
- Acceptance:
  - Given: `[look.style] status = "pill"` と `[terminal] nerd_font = true`
  - When:
  - Then: status の札の両端が丸い端の字。`nerd_font = false` → `soft`。

#### `terminal.nerd_font` の `"auto"` は、丸い端の字を字体に頼らず自分で描く端末(環境変数 `TERM_PROGRAM` が `ghostty` か `WezTerm`)のときだけ描けるとみなし、ほかの端末では描けないとみなすべき
`req~screen.style-parts-nerd-auto~1` P1
- Rationale: どの端末でも、描けなければ字が化けない形に落とすため
- Acceptance:
  - Given: `[look.style] status = "pill"` と `nerd_font = "auto"` で `TERM_PROGRAM=ghostty`
  - When:
  - Then: 丸い端の札。`TERM_PROGRAM=tmux` → `soft`。

#### 部品の形・組・丸い札の端の設定(`[look.style]`・`look.preset`・`terminal.nerd_font`)の知らない値は、警告して既定にするべき
`req~screen.style-parts-unknown~1` P1
- Acceptance:
  - Given: `[look.style] status = "neon"`
  - When:
  - Then: 警告して、status は既定の形のまま。

#### テーマ(screen.themes)に、落ち着いた色の組 `sumi`(墨。暗い地・薄い灰・青緑のアクセント)・`slate`(石板。紫寄りのアクセント)・`saas`(明るい灰の地・藍のアクセント)・`paper`(明るい地・青のアクセント)・`saas-dark`(saas の暗い地の版)を足すべき
`req~screen.calm-themes-add~1` P1
- Acceptance:
  - Given: `theme = "sumi"`
  - When:
  - Then: 地が #16171b。

#### 落ち着いた色の組のテーマも、screen.themes と同じく色だけを変え、画面の文字・印・構成は変えない
`req~screen.calm-themes-color-only~1` P1
- Acceptance:
  - Given: `sumi`・`slate`・`saas`・`paper`・`saas-dark` のどのテーマでも
  - When:
  - Then: 画面の文字はテーマなしと同じ。

#### リポの `docs/catalog/index.html` に、部品の形とテーマを見本の表と関係マップで見比べて選び、その設定(`config.toml` に貼れる、cli.config-file の形の文。`[look]`・`[look.style]`・`[look.colors]`)を作れるカタログを置くべき
`req~screen.style-catalog-page~1` P1
- Acceptance:
  - Given:
  - When: docs/catalog/index.html を開く
  - Then: 組とテーマを選ぶと見本が変わり、設定の文が出る。

#### カタログ(`docs/catalog/index.html`)では、色の役割(screen.color-overrides)と値の色(screen.value-colors)も選べ、その設定の文も作れるべき
`req~screen.style-catalog-colors~2` P1
- Acceptance:
  - Given:
  - When: カタログで役割 accent に #e0a458 を選ぶ
  - Then: 設定の文に `[look.colors]` と `accent = "#e0a458"` が出る。
- Acceptance:
  - Given:
  - When: 値 進行中 に色を選ぶ
  - Then: 設定の文に `[look.colors.values]` の行が出る。

#### カタログが出す設定の項目と値は、本体の設定(screen.style-parts・screen.calm-themes・screen.themes・screen.color-overrides)が受けるものと同じにし、ずれたら試験で落とすべき
`req~screen.style-catalog-sync~1` P1
- Acceptance:
  - Given: カタログの `status` の候補に本体に無い値
  - When:
  - Then: 試験が落ちる。

#### カタログ(`docs/catalog/index.html`)は1つのファイルで、外の何にも頼らず開けるべき
`req~screen.style-catalog-offline~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: docs/catalog/index.html に、外の URL と外のファイルの読み込みが無い。

#### 設定の `look.theme` が `"auto"` か明暗の表(`{ light = "<テーマ>", dark = "<テーマ>" }`)のとき、端末の地が明るいか暗いかで、明るいときのテーマ(`light`。既定 `saas`)と暗いときのテーマ(`dark`。既定 `sumi`)のどちらかを使うべき(`"auto"` は `{ light = "saas", dark = "sumi" }` と同じ。色を使わない表示ではテーマを使わない。screen.themes)
`req~screen.theme-auto-pick~2` P1
- Acceptance:
  - Given: `[look] theme = "auto"` で `COLORFGBG=0;15`(明るい地)
  - When:
  - Then: saas。
- Acceptance:
  - Given: `COLORFGBG=15;0`
  - When:
  - Then: sumi。

#### `look.theme` の明暗の表の片方を省けば、省いた側はその既定(`light` は `saas`、`dark` は `sumi`)を使う
`req~screen.theme-auto-partial~2` P1
- Acceptance:
  - Given: `theme = { light = "paper" }`
  - When: 明るい地の端末で起動する
  - Then: paper。
- Acceptance:
  - Given: `theme = { light = "paper" }`
  - When: 暗い地の端末で起動する
  - Then: sumi。

#### 端末の地の明るさは、環境変数 `COLORFGBG` か端末への問い合わせ(短い待ち時間で、答えが無ければ諦める)で決め、分からなければ暗い地とみなす
`req~screen.theme-auto-detect~1` P1
- Acceptance:
  - Given: `[look] theme = "auto"` で `COLORFGBG` が無く、問い合わせに答えない
  - When:
  - Then: sumi。

#### 設定の `[look.colors]` で、テーマ(screen.themes)の色のうち役割ごとに色を上書きできるべき
`req~screen.color-overrides-roles~1` P1
- Acceptance:
  - Given: `[look] theme = "sumi"` と `[look.colors] accent = "#e0a458"`
  - When:
  - Then: 列の見出しと下の帯のキーが #e0a458。上書きできる役割は `background`(地)・`text`(文字)・`header`(1行目)・`selection`(選んだ行の地)・`selection_text`・`band`(下の帯の地)・`band_text`・`accent`(見出し・キー・枠)・`strong`(強調)・`zebra`(一行おきの地)・`zebra_text`・`mark`(左端の印)・`added`(足した行)・`removed`(消した行)・`pending`(ためた変更)・`highlight`(検索の一致の地)で、どれを書いても警告しない。

#### `[look.colors]` の色は `"#rrggbb"`・`"#rgb"` か、色の名前(`black`・`white`・`gray`・`red`・`orange`・`yellow`・`green`・`teal`・`cyan`・`blue`・`purple`・`magenta`・`pink`・`brown`)で書く
`req~screen.color-overrides-format~2` P1
- Acceptance:
  - Given: `accent = "orange"`
  - When:
  - Then: 名前の色。
- Acceptance:
  - Given: `accent = "#e0a458"`
  - When:
  - Then: #e0a458。

#### `[look.colors]` で上書きしなかった役割は、テーマの色のまま
`req~screen.color-overrides-rest~1` P1
- Acceptance:
  - Given: `[look] theme = "sumi"` と `[look.colors] accent = "#e0a458"`
  - When:
  - Then: 地は sumi のまま。

#### テーマが `default`(色の表なし)のときは、端末の地と文字は変えず、`[look.colors]` の `accent`・`selection` と値の色(screen.value-colors)だけを使うべき
`req~screen.color-overrides-default-theme~1` P1
- Acceptance:
  - Given: `[look] theme = "default"` と `[look.colors] accent = "orange"`・`background = "#000000"`
  - When:
  - Then: 列の見出しは orange の色で、地の色の指定は無い(端末の地のまま)。

#### 色を使わない表示では、`[look.colors]` は効かない(cell-view.no-color)
`req~screen.color-overrides-no-color~1` P1
- Acceptance:
  - Given: `[look.colors] accent = "#e0a458"` と `--no-color`
  - When:
  - Then: 効かない(色の指定が無い)。

#### `[look.colors]` の知らない役割と読めない色は、警告して無視するべき
`req~screen.color-overrides-invalid~1` P1
- Acceptance:
  - Given: `accent = "#zz"`・`foo = "red"`
  - When:
  - Then: 警告して無視。

#### 設定の `[look.colors.values]` で、値ごとの色を決められるべき(キーは値の文字、値は screen.color-overrides と同じ色の書き方)
`req~screen.value-colors-set~2` P1
- Acceptance:
  - Given: `[look.colors.values] doing = "#d6a85c"`
  - When:
  - Then: status が doing の点・札がその色。
- Acceptance:
  - Given: 読めない色
  - When:
  - Then: 警告して無視。

#### `[look.colors.values]` で決めた値は、部品(screen.style-parts の `status`・`tags` のどの形でも、窓の候補でも)でその色を使い、決めていない値は今までの色(状態の語の意味の色か、値の文字から決まる色)のまま
`req~screen.value-colors-apply~2` P1
- Acceptance:
  - Given: `[look.colors.values] doing = "#d6a85c"` と `[look.style] status = "chip"`
  - When:
  - Then: doing の札の地と、窓の候補の doing がその色。
- Acceptance:
  - Given: 決めていない値
  - When:
  - Then: 今までの色。

#### `[look.colors.values]` の値の文字の照合は、前後の空白を除き、大文字・小文字を区別しない
`req~screen.value-colors-match~1` P1
- Acceptance:
  - Given: `Done = "green"`
  - When:
  - Then: done の札が緑(大文字・小文字によらない)。

#### `[look.colors.values]` で、日本語の値にも色を付けられる
`req~screen.value-colors-japanese~1` P1
- Acceptance:
  - Given: `[look.colors.values] "進行中" = "#d6a85c"`
  - When:
  - Then: status が 進行中 の点・札がその色。

#### ヘッダー(screen.layout)の左に、今の範囲と表を「範囲 › 表」の形(パンくず)で見せ、その右に行の数を見せ、範囲がワークスペースでなければ範囲を省くべき
`req~screen.breadcrumb-path~3` P1
- Acceptance:
  - Given:
  - When: ワークスペース「仕事」の tasks を開く
  - Then: 1行目の左が「W 仕事 › w tasks ▾  3行」。
- Acceptance:
  - Given: 範囲が無い
  - When:
  - Then: 「w tasks ▾  3行」。

#### パンくずの表の名前の所(後ろの「▾」を含む)は、表の一覧の引き出し(workspace.drawer)を開け閉めするボタンにするべき
`req~screen.breadcrumb-drawer~2` P1
- Rationale: 引き出しはワークスペースの中の表の切り替えなので、表の名前の所から開く(2026-11-25 に人が「ビューを押した時はビューの切り替えを」と決めた。前はパンくずの全体が引き出しの1つのボタンだった)
- Acceptance:
  - Given: ワークスペース「仕事」の tasks
  - When: パンくずの「tasks ▾」をクリック
  - Then: 表の一覧の引き出しが開き、もう一度で閉じる。
- Acceptance:
  - Given: 切り替えの一覧(workspace.switch-open)を開いている
  - When: パンくずの「tasks ▾」をクリック
  - Then: 切り替えの一覧が閉じ、引き出しが開く。

#### パンくずの範囲の名前の所は、ワークスペースの切り替えの一覧(workspace.switch-open)を開け閉めするボタンにし、表でない画面の型(関係マップ・木・ガント・カンバン・グラフ)ではその画面の上に開くべき
`req~screen.breadcrumb-switch~1` P1
- Rationale: 左上のワークスペースの名前を押したらワークスペースの切り替え、表の名前を押したらワークスペースの中の切り替えにして、押す所と開くものを合わせるため(2026-11-25 に人が決めた)
- Acceptance:
  - Given: ワークスペース「仕事」の tasks、お気に入りが 仕事・趣味
  - When: パンくずの「仕事」をクリック
  - Then: 切り替えの一覧が開き、もう一度で閉じる。
- Acceptance:
  - Given: 同じ表で、カンバンの画面
  - When: パンくずの「仕事」をクリック
  - Then: カンバンの画面のまま、その上に切り替えの一覧が開く。
- Acceptance:
  - Given: 引き出しを開いている
  - When: パンくずの「仕事」をクリック
  - Then: 引き出しが閉じ、切り替えの一覧が開く。
- Acceptance:
  - Given: ワークスペースでなく検知した範囲「notes」の表で、関係マップの画面(範囲の名前が出る)、お気に入りが無い
  - When: パンくずの「notes」をクリック
  - Then: 関係マップの上に、お気に入りの入れ方の案内の1行だけの切り替えの一覧が開く。
- Needs: utest・stest

#### パンくずの範囲の名前の所と表の名前の所は、それぞれビューの帯のボタン(navigation.search-box)と同じ見た目の別のボタンにして、範囲の名前の頭に切り替えの一覧のキー、表の名前の頭に引き出しのキー(screen.keys-vim-and-arrows で割り当てたキー。割り当てが無ければ添えない)、表の名前の後ろに「▾」を添えるべき
`req~screen.breadcrumb-button-look~3` P1
- Rationale: 押せることと、どこを押すと何が開くかが見て分かるように(2026-11-25 に、範囲の名前と表の名前で開くものを分けた)
- Acceptance:
  - Given:
  - When: ワークスペース「仕事」の tasks を開く
  - Then: 「W 仕事」と「w tasks ▾」がそれぞれボタンの見た目で、間の「›」はボタンの見た目でない。
- Acceptance:
  - Given:
  - When: 引き出しのキーを外す
  - Then: 「W 仕事 › tasks ▾」。

#### ヘッダーの右の端には、画面の型のメニュー(relations.screen-menu)と、ビューの設定の画面(`o`)を開くボタン「⚙ 設定」(キーを添える)をこの順に置くべき(ヘッダーには、どこで何として見るかに関わるものを置き、行を足す「+ 新規」はビューの帯に置く。ビューの帯を出さないときだけ、画面の型のメニューの左に置く。cell-edit.new-note)
`req~screen.breadcrumb-right-buttons~1` P1
- Acceptance:
  - Given:
  - When: 1行目の右の端に「▦ 表 ▾」と「o ⚙ 設定」があり、「o ⚙ 設定」を押す
  - Then: ビューの設定の画面。

#### パンくずとヘッダーの右の端のボタンは、表の画面と関係マップで同じ位置に出し、関係マップで押したら表の画面に戻ってから開くべき(パンくずの範囲の名前の所で開く切り替えの一覧は、screen.breadcrumb-switch のとおり関係マップの上に開く)
`req~screen.breadcrumb-relation-map~2` P1
- Acceptance:
  - Given:
  - When: 関係マップで、パンくずの表の名前を押す
  - Then: パンくずは表の画面と同じ桁で、表の画面に戻って引き出しが開く。
- Acceptance:
  - Given:
  - When: 関係マップで、パンくずの範囲の名前を押す
  - Then: 関係マップの画面のまま、切り替えの一覧が開く。

#### 端末の幅が足りないときは、ヘッダーの設定のボタンを先に省き、次に「+ 新規」を省き、パンくずと画面の型のメニューを残すべき(設定は `o` とパレットでも開ける)
`req~screen.breadcrumb-narrow~1` P1
- Acceptance:
  - Given: 幅 60
  - When:
  - Then: 設定のボタンが無く、パンくずと「▦ 表 ▾」は出る。

#### ビューの設定の画面(navigation.settings-layout)の左の一覧に「見た目」の区画を持ち、テーマ(screen.themes・screen.calm-themes。`auto` と、一覧の最後の「カスタム」(screen.look-section-custom)を含む)・組(screen.style-parts の `look.preset`)・丸い札の端(`terminal.nerd_font`: `auto`・`true`・`false`)を選べるべき
`req~screen.look-section-items~2` P1
- Acceptance:
  - Given:
  - When: `o`
  - Then: 左の一覧に「見た目」があり、その区画でテーマ(`auto` と、一覧の最後の「カスタム」を含む)・組・丸い札の端(`auto`・`true`・`false`)を選べる。

#### 「見た目」の区画の上で保存先の範囲(全体・このワークスペース・この表・このビュー。screen.profile-scopes)を選べ、既定は全体にするべき
`req~screen.look-section-scope~1` P1
- Acceptance:
  - Given:
  - When: ワークスペースの表で `o`
  - Then: 保存先は「全体」で、全体・このワークスペース・この表・このビューから選べる。

#### 「見た目」の区画のテーマと組の横には、今効いている値がどこから来たか(例「dracula(この表)」「sumi(config.toml)」。ワークスペースの印から来た値は「dozy-pink(印)」)を出すべき
`req~screen.look-section-origin~3` P1
- Rationale: 印の見た目は台帳の作り手の勧める既定で、人の選びより先に負ける(screen.profile-scopes-marker-look)。「このワークスペース」と出すと、保存先の「このワークスペース」と同じ所に見え、全体を選んでも変わらない理由が分からなかったため
- Acceptance:
  - Given:
  - When: config.toml に `[look] theme = "sumi"` で `o`
  - Then: テーマの横に「sumi(config.toml)」。
- Acceptance:
  - Given:
  - When: 保存先「この表」でテーマ nord を反映
  - Then: テーマの横に「(この表)」。
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "dozy-pink"` と `preset = "dozy-pink"`
  - When: notes/tasks で `o`
  - Then: テーマと組の横に「(印)」が出て、「(このワークスペース)」は出ない。

#### 「見た目」の区画で選んだ見た目は、「反映」で画面にすぐ効かせるべき
`req~screen.look-section-apply~2` P1
- Acceptance:
  - Given:
  - When: テーマ dracula・組 dozy-pink を選ぶ
  - Then: 上に「未反映 1」。
- Acceptance:
  - Given:
  - When: 反映
  - Then: 画面の地が dracula の色、札が dozy-pink の形。

#### 「見た目」の区画で反映した見た目は、選んだ範囲の画面が書く場所(全体は ui.toml、ワークスペースは workspaces.toml のそのワークスペース、表は views.toml の `[[table]]`、ビューはビューの設定)に残すべき(config.toml とワークスペースの印は人が書くファイルなので書き換えない)
`req~screen.look-section-save~2` P1
- Acceptance:
  - Given:
  - When: 保存先「全体」でテーマ dracula・組 dozy-pink を反映 → ui.toml の `[look]` に theme = "dracula"・preset = "dozy-pink"、終了して開き直す
  - Then: 同じ見た目。
- Acceptance:
  - Given:
  - When: 保存先「この表」でテーマ nord を反映
  - Then: views.toml の対象の `[table.look]` に theme = "nord"、ほかの表は dracula のまま。
- Acceptance:
  - Given: 壊れた ui.toml
  - When:
  - Then: 警告して config.toml のとおりに起動。

#### 「見た目」の区画の丸い札の端(`terminal.nerd_font`)は、どの範囲を選んでいても ui.toml に残すべき
`req~screen.look-section-nerd-font~1` P1
- Rationale: 丸い札の端は端末の性質のため
- Acceptance:
  - Given:
  - When: 保存先「この表」で丸い札の端を `true` にして反映
  - Then: ui.toml に nerd_font = true があり、views.toml には書かない。

#### 「見た目」の区画で組を変えたときは、選んだ範囲より広い範囲の部品ごとの形は重ねず、その組の形にするべき(screen.profile-scopes)
`req~screen.look-section-preset~1` P1
- Acceptance:
  - Given:
  - When: config.toml に `[look.style] status = "chip"`、保存先「この表」で組 saas を反映
  - Then: その表の status は saas の形(chip の札でない)。

#### 「見た目」の区画で、今のテーマと組の組み合わせに名前を付けて、テンプレートとして ui.toml の `[templates.<名前>]` に保存できるべき
`req~screen.look-section-template-save~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 「名前を付けて保存」で「夜」→ ui.toml に `[templates.夜.look]`、テンプレートに「夜」。

#### 「見た目」の区画のテンプレートの一覧(config.toml と ui.toml のテンプレート)から選んで、写しに当てられるべき
`req~screen.look-section-template-use~1` P1
- Acceptance:
  - Given:
  - When: テーマ dracula・組 dozy-pink を「夜」で保存し、組を sumi に戻して反映 → テンプレートの「夜」を選んで反映
  - Then: dracula と dozy-pink。

#### 「見た目」の区画のテンプレートの一覧のうち、ui.toml のテンプレートは消せるべき
`req~screen.look-section-template-delete~1` P1
- Acceptance:
  - Given:
  - When: ui.toml のテンプレート「夜」を消す
  - Then: ui.toml に `[templates.夜]` が無く、テンプレートの一覧に「夜」が無い。

#### 「見た目」の区画の「この範囲の上書きを外す」で、選んだ範囲に画面が書いた見た目を外し、それより広い範囲のとおりに戻せるべき
`req~screen.look-section-clear~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 全体に dracula、保存先「この表」で nord を反映したあと「この範囲の上書きを外す」→ その表も dracula。

#### ワークスペースの範囲が無いとき(screen.profile-scopes)は、「見た目」の区画の保存先にワークスペースを出さないべき
`req~screen.look-section-no-workspace~1` P1
- Acceptance:
  - Given:
  - When: 登録した表(ワークスペースの範囲が無い)で `o`
  - Then: 保存先に「このワークスペース」が無い。

#### ワークスペースが印から来たときにそれを「見た目」の保存先に選んで反映したら、印には書かないので印に手で書くことを理由として出し、書かないべき
`req~screen.look-section-marker~1` P1
- Acceptance:
  - Given:
  - When: 印のワークスペースで保存先「このワークスペース」を反映
  - Then: 理由が出て、印は変わらない。

#### 読むだけの起動(`--readonly`)では、「見た目」の区画の反映で画面には効かせるが、どのファイルにも書かないべき(cli.readonly)
`req~screen.look-section-readonly~1` P1
- Acceptance:
  - Given: `--readonly`
  - When:
  - Then: 反映で効くがファイルは無い。

#### ビューの設定の画面の左の一覧の「見た目」の区画の印は、今の組の名前にするべき
`req~screen.look-section-mark~2` P1
- Acceptance:
  - Given:
  - When: 既定の組で `o`
  - Then: 左に「見た目 sumi」。
- Acceptance:
  - Given: 組 dozy-pink を反映
  - When:
  - Then: 左に「見た目 dozy-pink」。

#### 「見た目」の区画のテーマの一覧の「カスタム」を選ぶと、今のテーマを元のテーマにして、区画に元のテーマの行と、役割ごとの色(screen.color-overrides-roles の16の役割)の行を出し、それぞれを選んで直せるべき(名前のテーマを選び直せば、役割の行と上書きの色を外す)
`req~screen.look-section-custom~1` P1
- Rationale: 自分で色を決めたい人が、設定のファイルを書かずに、今のテーマから少しだけ変えられるように
- Needs: utest・stest
- Acceptance:
  - Given: テーマ dozy-pink
  - When: 「見た目」の区画のテーマの一覧で「カスタム」を選ぶ
  - Then: テーマの欄が「カスタム(元: dozy-pink)」になり、元のテーマの行(dozy-pink)と、地・文字・アクセントなど16の役割の行が出る。
- Acceptance:
  - Given: カスタムでアクセントを orange にした写し
  - When: 元のテーマの行で nord を選ぶ
  - Then: テーマの欄が「カスタム(元: nord)」で、アクセントは orange のまま。
- Acceptance:
  - Given: カスタムでアクセントを orange にした写し
  - When: テーマの一覧で sumi を選ぶ
  - Then: テーマの欄が「sumi」で、役割の行が無く、写しに上書きの色が無い。

#### 「見た目」の区画のカスタムの役割の色は、「テーマのまま」・見本の色の一覧(screen.color-overrides-format の名前の色)・`#rrggbb` を打つ、から選べ、読めない色は理由を出して入れないべき
`req~screen.look-section-custom-colors~1` P1
- Needs: utest
- Acceptance:
  - Given: カスタムの役割の行が出ている
  - When: アクセントの行で見本の色の orange を選ぶ
  - Then: アクセントの行が #ea8c34。
- Acceptance:
  - Given: カスタムの役割の行が出ている
  - When: アクセントの行で `#e0a458` を打って決める
  - Then: アクセントの行が #e0a458。
- Acceptance:
  - Given: カスタムの役割の行が出ている
  - When: アクセントの行で `#zz` を打って決める
  - Then: 読めない理由が出て、入力の欄は開いたまま、アクセントの行は変わらない。
- Acceptance:
  - Given: アクセントを #e0a458 にした写し
  - When: アクセントの行で「テーマのまま」を選ぶ
  - Then: アクセントの行は元のテーマの色で、写しにアクセントの上書きが無い。

#### 「見た目」の区画でカスタムの色を反映すると、画面にすぐ効かせ、選んだ範囲の画面が書く場所(screen.look-section-save)に、`theme` に元のテーマ、`[look.colors]` に上書きした役割の色(`"#rrggbb"`)を書くべき(上書きの無い役割は書かない。値の色 `[look.colors.values]` は変えない)
`req~screen.look-section-custom-save~1` P1
- Needs: utest・stest
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "dozy-pink"`
  - When: 保存先「全体」、カスタムでアクセントを #e0a458 にして反映
  - Then: 画面のアクセントの色が #e0a458 で、ui.toml の `[look]` に theme = "dozy-pink"、`[look.colors]` に accent = "#e0a458" だけがあり、開き直しても同じ。
- Acceptance:
  - Given: ui.toml に theme = "dozy-pink" と `[look.colors] accent = "#e0a458"`・`[look.colors.values] done = "green"`
  - When: 保存先「全体」でテーマ nord を選んで反映
  - Then: ui.toml の theme が nord で、`[look.colors]` に accent が無く、values の done は残る。
- Acceptance:
  - Given: カスタムでアクセントを #e0a458 にした写し
  - When: 「名前を付けて保存」で「夜」
  - Then: ui.toml の `[templates.夜.look]` に元のテーマ、`[templates.夜.look.colors]` に accent = "#e0a458"。

#### 今の見た目が役割の色の上書き(`[look.colors]` の役割)を持つときは、「見た目」の区画のテーマの欄に「カスタム(元: <テーマ>)」と出し、役割の行を出すべき
`req~screen.look-section-custom-label~1` P1
- Needs: utest
- Acceptance:
  - Given: config.toml に `[look] theme = "sumi"` と `[look.colors] accent = "orange"`
  - When: `o` で「見た目」の区画を見る
  - Then: テーマの欄が「カスタム(元: sumi)」で、アクセントの行が #ea8c34。
- Acceptance:
  - Given: config.toml に `[look] theme = "sumi"` と `[look.colors.values] done = "green"` だけ
  - When: `o` で「見た目」の区画を見る
  - Then: テーマの欄は「sumi」(値の色はカスタムに数えない)。

#### 設定(ユーザーが書くもの)と見た目の状態(列の幅・選んだビュー・畳んだまとまりなど、消えても困らないもの)を分け、見た目の状態は、ノートのフォルダの中に書くべきでない
`req~screen.state-separate~1` P1
- Acceptance:
  - Given:
  - When: 列の幅を変えて終了
  - Then: ノートのフォルダに新しいファイルが無い。

#### 見た目の状態は、`.base` のパス(`.base` なしで開いたときはフォルダのパス)とビューの名前ごとに保存するべき
`req~screen.state-store-key~1` P1
- Acceptance:
  - Given:
  - When: ビュー「全部」で title の幅を変えて終了し、開き直す
  - Then: 「全部」の title は変えた幅で、ほかのビューの title の幅は変わらない。

#### 見た目の状態が壊れていたら、捨てて既定で起動するべき
`req~screen.state-store-broken~1` P1
- Acceptance:
  - Given:
  - When: 状態のファイルを壊して起動
  - Then: 既定で起動し、エラーで止まらない。

#### 見た目の状態の列の並びと隠す列は、今の `.base` にある列にだけ当て、`.base` に増えた列は `.base` のビューの `order` の中の位置に入れ(screen.state-store-follow-order。`order` の終わりに足した列は右)、`order` の無い表で増えた列は右に足すべき
`req~screen.state-store-columns~2` P1
- Rationale: 増えた列をいつも右に足すと、印の `order` の途中に足した列が、印を直した人の置いた位置に出ないため(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given:
  - When: 列 a・b・c を c・a・b に並べて終了し、`.base` に列 d を足して開き直す
  - Then: c・a・b・d の順。
- Acceptance:
  - Given: `order` が status・due・title の表で、title を先頭に動かして終了した(title・status・due)
  - When: `order` を file.name・status・title にして開き直す
  - Then: title・file.name・status の順(消えた due は出ず、足した file.name は印の位置の status の前)。

#### 見出しからの並べ替え(navigation.header-sort)は、設定の並べ替えとして残すべき
`req~screen.state-store-header-sort~1` P1
- Acceptance:
  - Given:
  - When: 見出しから title で並べ替えて終了し、同じ対象で開き直す
  - Then: title で並んだまま。

#### 見た目の状態の列の並びと隠す列は、人が列を動かすか隠したときだけ覚え、開いただけ・幅を変えただけ・畳んだだけでは覚えないべき(幅と畳んだまとまりは、人が変えたものだけを今までどおり覚える)
`req~screen.state-store-arranged-only~1` P1
- Rationale: ビューを一度開いただけで、そのときの列の並びが状態に書かれ、あとで印(`.base`・印の `[table.base]`)の `order` に列を足しても、足した列が右の端に回っていたため(2026-11-25 の実地の操作で見つかった。人は印を手で直す)
- Acceptance:
  - Given: 印のビューの `order` が a・b・c の表
  - When: 開いて何もせずに終了し、`order` を a・b・d・c にして開き直す
  - Then: 列は a・b・d・c の順。
- Acceptance:
  - Given: 同じ表
  - When: b の幅を変えて終了し、`order` を a・b・d・c にして開き直す
  - Then: 列は a・b・d・c の順で、b は変えた幅のまま。
- Needs: utest

#### 列の並びを覚えたあとで、印(`.base`・印の `[table.base]`)のビューの `order` が覚えたときから変わったら、`order` に合わせ、覚えた列の動かし方のうち `order` の変更と食い違わないものだけを残すべき(隠した列と幅は残す)
`req~screen.state-store-follow-order~1` P1
- Rationale: 印を直した人の意図(列を足す・動かす)を、前に覚えた並びで消さないため。覚えた並びを丸ごと捨てると、人が動かした列まで戻ってしまう
- Acceptance:
  - Given: `order` が a・b・c の表で、c を先頭に動かして終了した(c・a・b)
  - When: `order` を a・x・b・c にして開き直す
  - Then: 列は c・a・x・b の順(足した x は印の位置に入り、先頭に動かした c は先頭のまま)。
- Acceptance:
  - Given: 同じ表で c を先頭に動かして終了した
  - When: `order` を変えずに開き直す
  - Then: 列は c・a・b の順(screen.state-store-columns と同じ)。
- Acceptance:
  - Given: `order` が a・b・c の表で、c を先頭に動かし、b を隠して終了した
  - When: `order` を b・a・c にして開き直す
  - Then: 列は c・a の順で、b は隠したまま。
- Needs: utest

#### 表の見せ方の次の4つの既定を、設定(cli.config-file の設定ファイル)で決められるべき: 行番号(表の左に1始まりの番号。既定は出さない)・一行おきの色(既定は付けない)・列の区切り線(既定は今の見た目)・上の帯(ビューの帯のタブと検索の欄(navigation.search-box)、設定の帯(navigation.settings-bar)をそれぞれ出すか。タブと検索の欄を両方出さなければ、ビューの帯の行ごと出さない)
`req~screen.display-options-config~2` P1
- Acceptance:
  - Given:
  - When: 設定で行番号を出す
  - Then: 各行の左に 1・2・3。
- Acceptance:
  - Given: 一行おきの色
  - When:
  - Then: 偶数の行の背景が違う。
- Acceptance:
  - Given:
  - When: 区切り線を入れる
  - Then: 列の間に `│` が出る。既定(今の見た目)では `│` が無い。
- Acceptance:
  - Given:
  - When: 検索の欄を隠す
  - Then: 表が1行広い。

#### 表の見せ方の4つ(行番号・一行おきの色・列の区切り線・上の帯)は、ビューの設定(navigation.view-settings)の「表示」の区画(navigation.settings-layout)でビューごとに切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.display-options-per-view~1` P1
- Acceptance:
  - Given:
  - When: 設定で行番号を出し、ビューの設定の表示の区画で消して保存
  - Then: そのビューだけ番号が無く、開き直しても同じ。

#### 上の帯を隠しても、その帯の操作(タブの切り替えのキー・`\`・`f`)は効くべき
`req~screen.display-options-hidden-keys~2` P1
- Acceptance:
  - Given:
  - When: 検索の欄を隠す
  - Then: `\` は最下行で打てる。
- Acceptance:
  - Given:
  - When: タブを隠す
  - Then: `]` で切り替わる。

#### 一行おきの色は、色を使わない表示(`--no-color`・NO_COLOR)では付けないべき(cell-view.not-color-only)
`req~screen.display-options-zebra-no-color~1` P1
- Acceptance:
  - Given:
  - When: 一行おきの色を付ける設定で `--no-color`
  - Then: 偶数の行の背景も同じ。

#### screen.display-options の設定は、`[display]` の表に15の項目を持つべき
`req~screen.display-config-keys~2` P1
- Acceptance:
  - Given: `[display] row_numbers = true`
  - When:
  - Then: 番号が出て、設定の画面の表示の区画でも「行番号」がオン。
- Acceptance:
  - Given: `[display] search_bar = false`
  - When:
  - Then: 検索の欄が無い。`--print-config` の `[display]` に15項目: `row_numbers`(既定 false)・`zebra`(既定 false)・`column_lines`(既定 false。true で列の間に `│` を引く)・`group_gap`(既定 false。screen.group-gap)・`tabs`(既定 `"always"`。screen.tabs-setting)・`search_bar`(既定 true。navigation.search-box)・`chips`(既定 true)・`buttons`(既定 `"plain"`。navigation.search-box)・`bar_padding`(既定 1。navigation.search-box)・`top_padding`(既定 1。screen.layout)・`row_extra_lines`(既定 0。cell-view.row-lines)・`note_path`(既定 `"common"`。screen.note-column・screen.note-path)・`note_name`(既定なし。screen.note-name)・`tab_wrap`(既定 false。navigation.tab-wrap)・`tab_wrap_gap`(既定 false。navigation.tab-wrap)。

#### ビューごとの表示の値は、ビューの設定(settings)の `display` に、設定と違う項目だけを持ってよい
`req~screen.display-config-view-diff~1` P1
- Acceptance:
  - Given:
  - When: 設定が `[display] row_numbers = true` で、ビューの settings の `display` に `zebra = true` だけを書く
  - Then: そのビューは行番号と一行おきの色の両方が効く。

#### 表の見せ方(screen.display-options)の、まとまりの見出しの上の空きは、`groupBy`(navigation.settings-group の設定のグループ分けを含む)のまとまりがあるとき、2つ目以降の見出しの行の上に空きの行を1つ入れるべき
`req~screen.group-gap-line~2` P1
- Acceptance:
  - Given:
  - When: `[display] group_gap = true` で status でまとめたビューを開く
  - Then: 2つ目の見出しの上に空きの行があり、1つ目の見出しの上には無い。
- Acceptance:
  - Given: まとまりの無いビュー
  - When:
  - Then: 空きは出ない。

#### まとまりの見出しの上の空きは、既定では空けず、設定の `[display]` の `group_gap`(既定 false)で既定を決めるべき
`req~screen.group-gap-config~1` P1
- Acceptance:
  - Given: `group_gap` の無い設定
  - When:
  - Then: 今と同じ画面。

#### まとまりの見出しの上の空きは、screen.display-options のほかの項目と同じく、ビューの設定(navigation.view-settings)でビューごとに切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.group-gap-per-view~1` P1
- Acceptance:
  - Given:
  - When: `group_gap` の無い設定で、status でまとめたビューの表示の区画で空きを入れて保存
  - Then: そのビューだけ2つ目の見出しの上に空きの行があり、開き直しても同じ。

#### まとまりの見出しの上の空きの行は、行として数えない(行の数・行番号・`--print` に出さない)べき
`req~screen.group-gap-not-row~1` P1
- Rationale: 空きは見た目のためだけにあるため
- Acceptance:
  - Given: `[display] group_gap = true` と `row_numbers = true` で status でまとめた3行のビュー
  - When:
  - Then: ヘッダーの行の数は 3行で、行番号は 1・2・3、空きの行に番号が無い。

#### まとまりの見出しの上の空きの行には、カーソルを止めないべき
`req~screen.group-gap-cursor~1` P1
- Rationale: 空きは見た目のためだけにあるため
- Acceptance:
  - Given:
  - When: 1つ目のまとまりの最後の行で `j`
  - Then: 空きを飛ばして2つ目の見出しへ。

#### まとまりの見出しの上の空きの行は、クリックしても選びを変えないべき
`req~screen.group-gap-click~1` P1
- Rationale: 空きは見た目のためだけにあるため
- Acceptance:
  - Given:
  - When: 空きの行をクリック
  - Then: 選びは変わらない。

#### 表のセルの編集の入力欄は、地の色で欄と分かるようにするべき(色を使わない設定では今までどおり太字と下線。太字と下線は screen.themes のテーマの強い色の目印なので残す)
`req~screen.input-box-fill~2` P1
- Acceptance:
  - Given:
  - When: status のセルで Enter
  - Then: 入力欄に地の色が付く。
- Acceptance:
  - Given: `--no-color` で同じ操作
  - When:
  - Then: 入力欄は太字と下線。

#### 表のセルの編集の入力欄の幅は、列・打った文字・16桁のうち広いもの(右の列に重ね、画面の幅まで)にするべき
`req~screen.input-box-width~1` P1
- Acceptance:
  - Given:
  - When: status のセルで Enter
  - Then: 入力欄は16桁の幅。

#### 新しいノートの窓(cell-edit.new-note-form)は、枠のある窓として表の上に重ね、上の縁に題と作る場所のフォルダの名前、下の縁の上に押せるキーを出すべき
`req~screen.input-box-form-frame~1` P1
- Acceptance:
  - Given: 新しいノートの窓
  - When:
  - Then: `╭─ 新しいノート` の枠、窓の中に `^S 作る`。

#### 新しいノートの窓の、まだ値の無い欄には、型の手がかり(日付は `YYYY-MM-DD`、日時は `YYYY-MM-DDTHH:MM`、数・リスト・チェック)を薄く出すべき
`req~screen.input-box-form-hints~1` P1
- Acceptance:
  - Given: 新しいノートの窓
  - When:
  - Then: due の欄に `YYYY-MM-DD`。

#### 新しいノートの窓の枠の線は、あいまいな幅の文字を2と数える設定(cell-view.ambiguous-width)では ASCII の線にするべき
`req~screen.input-box-ascii~1` P1
- Acceptance:
  - Given: `ambiguous_wide = true`
  - When:
  - Then: 新しいノートの窓の枠は `+-`。

#### 表に重ねる窓(候補・操作の一覧・リストの選択・カレンダー・頻度表・新しいノートの窓など)の枠は、既定でつながった罫線(角の丸い `╭ ╮ ╰ ╯` と `─ │`)で描き、設定の `look.style.frames` が `"ascii"` のときと、East Asian Ambiguous の文字を幅2とする設定(cell-view.ambiguous-width)のときは、幅がずれないように ASCII の `+ - |` で描くべき
`req~screen.frames~1` P1
- Acceptance:
  - Given:
  - When: 既定で候補の窓を開く
  - Then: 枠が `╭─…─╮`。`[look.style] frames = "ascii"` か `[terminal] ambiguous_wide = true` → `+-…-+`。

#### 表に重ねる窓(候補・操作の一覧・リストの選択・カレンダー・頻度表・絞り込みの窓・新しいノートの窓など)の縁の外に、窓に隠れた札の丸い端の字だけを残さないべき
`req~screen.window-chip-remnants~1` P1
- Rationale: ブックマークの台帳で新しいノートの窓を開くと、窓の右の縁の外に、窓に隠れた下の表の札の丸い端だけが残り、何かの印のように見えたため(2026-11-25 の実地の操作)。札の体が見えていれば、丸い端は札の一部として読めるので残す
- Acceptance:
  - Given: 丸い札(`[look.style] status = "pill"` と `nerd_font = true`)の status の札の体がちょうど窓に隠れ、右の丸い端の字(U+E0B4)が窓の右の縁の外にくる表
  - When: 窓を開く
  - Then: 窓の右の縁の外のその桁は空白で、丸い端の字が無い。
- Acceptance:
  - Given: 札の左の丸い端の字(U+E0B6)だけが窓の左の縁の外にくる表
  - When: 窓を開く
  - Then: その桁は空白。
- Acceptance:
  - Given: 窓の外にある札と、体の一部が窓の外に見える札
  - When: 窓を開く
  - Then: どちらの札も、見えている体の隣の丸い端の字は残る。
- Needs: utest

#### 色を使える表示で、設定の `look.mode` が `"modern"`(既定)のときは、今どきの道具のような見た目として、窓の枠を薄い色にし、今操作している窓の枠と題をアクセントの色にするべき
`req~screen.modern-look-frames~1` P1
- Acceptance:
  - Given:
  - When: 色のある表示で候補の窓を開く
  - Then: 今操作している候補の窓の枠はアクセントの色。

#### `look.mode` が `"modern"` のとき、選んでいる行・候補・項目は、反転でなく背景の色と太字(形は screen.style-parts の `select`)で見せる
`req~screen.modern-look-select~1` P1
- Acceptance:
  - Given:
  - When: 色のある表示で候補の窓を開く
  - Then: 選んだ候補のセルに反転が無く背景の色が付く。

#### `look.mode` が `"modern"` のとき、下の帯・ビューのタブ・列の見出しの形は screen.style-parts の `band`・`tabs`・`icons` に従う
`req~screen.modern-look-parts~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 色のある表示で、下の帯のキーは組の帯の形(screen.style-parts の `band`)で、説明は薄い色(既定の sumi は `quiet` でキーは太字、`classic` の組ではアクセントの色の太字)。

#### `look.mode` が `"modern"` のとき、色はテーマ(screen.themes)のものを使い、テーマが既定(色の表が無い)なら端末の地の色を変えない控えめな組を使う
`req~screen.modern-look-colors~1` P1
- Acceptance:
  - Given:
  - When: テーマを決めない色のある表示で候補の窓を開く
  - Then: 画面の地の色の指定が無く(端末の地のまま)、選んだ候補には背景の色が付く。

#### `look.mode` が `"classic"` のときと色を使わない表示(cell-view.no-color)では、今までの見た目(反転)にする
`req~screen.modern-look-classic~2` P1
- Acceptance:
  - Given: `[look] mode = "classic"`
  - When:
  - Then: 今の見た目(反転)。
- Acceptance:
  - Given: `--no-color`
  - When:
  - Then: 色の指定が無い。

#### `look.mode` が `"modern"` でも `"classic"` でも、値の文字・印・構成は変えない(screen.themes)
`req~screen.modern-look-same-text~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: どの mode でも、同じ表と候補の窓の画面の文字は同じ。

#### 画面と起動の文言(使い方(`--help`)・起動できない理由・ヘルプ・パレット・下の帯とメッセージ行・キーの表示名・確認の画面・ビューの設定の画面(区画の名前と説明。navigation.settings-layout)・操作の一覧(screen.action-menu)・前置きのキーの続きの案内(screen.prefix-hints)・読むだけや未対応や保存が止まった理由・`--print` と `--pick` の警告)を、英語と日本語で出せるべき
`req~screen.locales-two-languages~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 台帳の全部の文言が英語と日本語の両方にある。

#### 文言の言語は、設定(cli.config-file)の `language`(`"auto"` か台帳のある言語の符号。既定 `"auto"`)で選び、台帳の無い符号は警告して `"auto"` と同じにするべき
`req~screen.locales-setting~3` P1
- Rationale: 環境変数と違う言語で使いたい人が選べるようにし、足した言語(screen.locales-languages)もそのまま選べるようにするため
- Acceptance:
  - Given: `LANG=en_US.UTF-8` で設定に `language = "ja"`
  - When:
  - Then: 日本語。
- Acceptance:
  - Given: `language = "fr"`
  - When:
  - Then: フランス語。
- Acceptance:
  - Given: `language = "xx"`
  - When:
  - Then: 警告(cli.config-file-warning-form)して、環境変数で選んだ言語。

#### `language` が `auto` のときは、環境変数 `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値から、言語と地域が合う台帳、無ければ言語が合う台帳を選び、どれも合わないとき(環境変数が無いときも)英語にするべき
`req~screen.locales-auto~3` P1
- Rationale: 足した言語の人が、設定を書かずに自分の言語で使えるようにするため
- Acceptance:
  - Given: `LANG=en_US.UTF-8`
  - When:
  - Then: 英語。
- Acceptance:
  - Given: `LANG=ja_JP.UTF-8`
  - When:
  - Then: 日本語。
- Acceptance:
  - Given: `LANG=zh_TW.UTF-8`
  - When:
  - Then: zh-TW。
- Acceptance:
  - Given: `LANG=zh_HK.UTF-8`
  - When:
  - Then: zh-TW(繁体字)。
- Acceptance:
  - Given: `LANG=zh_CN.UTF-8` と `LANG=zh`
  - When:
  - Then: zh-CN。
- Acceptance:
  - Given: `LANG=pt_PT.UTF-8`
  - When:
  - Then: pt-BR。
- Acceptance:
  - Given: `LANG=de_AT.UTF-8`
  - When:
  - Then: de。
- Acceptance:
  - Given: `LANG=nl_NL.UTF-8`
  - When:
  - Then: 英語。
- Acceptance:
  - Given: どの環境変数も無い
  - When:
  - Then: 英語。

#### 文言は言語ごとの台帳(`locales/<言語の符号>.toml`。英語の `en.toml` が文言の名前と並びの正本)に置き、どれかの台帳で文言が欠けるか余るか空であるか、差し込みの番号(`{0}` など)の組が英語と違えば、ビルドで落ちるべき
`req~screen.locales-ledger~1` P1
- Acceptance:
  - Given:
  - When: `locales/ja.toml` で文言が1つ欠ける、または差し込みの番号が英語と違う
  - Then: ビルドが落ちる。

#### 文言の言語の一覧は台帳のファイルから作り、言語を足すときは台帳を足せばよい形にするべき
`req~screen.locales-list~1` P1
- Acceptance:
  - Given:
  - When: `locales/` に en.toml と同じ名前の文言を持つ `fr.toml` を足してビルドする
  - Then: ほかのファイルを変えずに、言語の一覧に fr が入る。

#### ノートの値・列の名前・`.base` の中身・ファイルの名前は、文言の言語によらず訳さないべき
`req~screen.locales-no-translate~1` P1
- Acceptance:
  - Given:
  - When: `LANG=en_US.UTF-8` で、列 担当 と status の値 進行中 のノートを開く
  - Then: 列の名前 担当 と値 進行中 はそのまま出る。

#### 英語と日本語に加えて、中国語(簡体字 `zh-CN`・繁体字 `zh-TW`)・韓国語(`ko`)・スペイン語(`es`)・フランス語(`fr`)・ドイツ語(`de`)・ブラジルのポルトガル語(`pt-BR`)の台帳を持つべき
`req~screen.locales-languages~1` P1
- Rationale: 日本語と英語の外の人も、自分の言語で画面と使い方を読めるようにするため(2026-11-21 に人が「主な言語をまとめて」と決めた)
- Acceptance:
  - Given:
  - When:
  - Then: `locales/` に en・ja・zh-CN・zh-TW・ko・es・fr・de・pt-BR の9つの台帳があり、`LANG=ko_KR.UTF-8` の `mdgrid --help` が韓国語。訳の質は、その言語を読める人が読んで判定。

#### 利用者に見える文言(screen.locales-two-languages に挙げたものに加え、設定の項目の説明(`--print-config` のコメントと設定の画面の説明)・キーの動作の名前・設定の警告)は、すべて台帳から出し、台帳の外に言語ごとの文言を持たないべき
`req~screen.locales-all-in-ledger~1` P1
- Rationale: 言語を足すときに台帳を足すだけで済むようにし(screen.locales-list)、ソースに残った日本語や英語の文言が、ほかの言語の画面に混ざらないようにするため
- Acceptance:
  - Given:
  - When: どの文言の頭にも「⟦」を付けた試しの台帳で起動し、表の画面・ヘルプ・パレット・ビューの設定の画面・`--help`・`--print-config`・設定の警告を出す
  - Then: ノートの値・列の名前・パスのほかの文言には、どれも「⟦」が付いている。
- Needs: stest

#### どの台帳の言語でも、文言が長いか幅の広い字のときに、ヘッダー・ビューの帯・下の帯・ビューの設定の画面・確認の画面の文字を枠や隣の部品に重ねず、収まらない分を `…` で切るべき
`req~screen.locales-fit~1` P1
- Rationale: ドイツ語のような長い語や、中国語・韓国語の幅の広い字で、画面が崩れないようにするため
- Acceptance:
  - Given:
  - When: 80桁・24行の端末で、9つの台帳のそれぞれで表の画面とビューの設定の画面を出す
  - Then: どの行も80桁を超えず、部品の文字が重ならず、切った所に `…`。
- Needs: stest

#### 画面の色の組み合わせ(テーマ)を、見本の組(今の見た目・Nord・Solarized Light・Dracula・Gruvbox・Pink Monster・Dozy Pink と、screen.calm-themes の落ち着いた組)から設定(cli.config-file)で選べるべき
`req~screen.themes-choose~1` P1
- Acceptance:
  - Given:
  - When: 設定でテーマを Nord にして起動
  - Then: 左上の背景が #2e3440。

#### 今の見た目以外のテーマは、画面の地と文字・ヘッダー・選択・下の帯・列の見出し・強調(太字)・一行おきの色(screen.display-options)・行の印 `>` を、テーマごとに決めた色で塗るべき
`req~screen.themes-paint~1` P1
- Acceptance:
  - Given:
  - When: 設定でテーマを Nord にして起動
  - Then: 選んでいるセルの背景が #5e81ac、下の帯の背景が #3b4252。

#### テーマは色だけを変え、画面の構成・文字・印は変えないべき
`req~screen.themes-color-only~1` P1
- Rationale: 色だけで意味を伝えない cell-view.not-color-only を保ち、画面の型を変える scope.screen-types と分けるため
- Acceptance:
  - Given:
  - When:
  - Then: 7つのどのテーマでも、画面の文字はテーマなしと1字も違わない。

#### screen.themes のテーマは、設定の `look.theme` に `"default"`(既定。今の見た目で、端末の既定の色と反転・太字・薄い表示だけを使う)・`"nord"`・`"solarized-light"`・`"dracula"`・`"gruvbox"`・`"pink-monster"`・`"dozy-pink"`・`"sumi"`・`"slate"`・`"saas"`・`"saas-dark"`・`"paper"`(screen.calm-themes)・`"auto"`(screen.theme-auto)のどれかを書いて選んでよい
`req~screen.theme-names-list~2` P1
- Acceptance:
  - Given: 設定なし
  - When:
  - Then: 今と同じ色。
- Acceptance:
  - Given: `[look] theme = "dracula"`
  - When:
  - Then: 警告なしで dracula の色。

#### `look.theme` の知らない名前は、警告にとどめて `default` で起動するべき(cli.config-file)
`req~screen.theme-names-unknown~1` P1
- Acceptance:
  - Given: `[look] theme = "neon"`
  - When:
  - Then: 警告に `look.theme` と出て既定の見た目で起動。

#### 色を使わない表示(cell-view.no-color・`--no-color`・設定の `terminal.color = false`)では、テーマを使わないべき
`req~screen.theme-names-no-color~1` P1
- Acceptance:
  - Given: `[look] theme = "dracula"` と `NO_COLOR=1`
  - When:
  - Then: 色の指定が無い。

#### トゥルーカラーに対応しない端末(cell-view.not-color-only)では、テーマの色を近い 256 色に落とすべき
`req~screen.theme-names-256~1` P1
- Acceptance:
  - Given: `COLORTERM` なしで `[look] theme = "nord"`
  - When:
  - Then: 256 色の番号で塗る。

#### screen.themes のテーマ(今の見た目を除く)では、ためる変更のセルの文字(cell-edit.bulk-set)、保存の確認の差分の足した行と消した行の文字(write-back.pending-and-diff)、検索の一致と同じ値の強調の地(navigation.find・navigation.same-value)も、テーマごとに決めたその地で読める色で塗るべき
`req~screen.theme-meaning-colors-paint~2` P1
- Acceptance:
  - Given: Solarized Light でためる変更のセル
  - When: 表を出す
  - Then: 文字が #b58900。
- Acceptance:
  - Given: Solarized Light
  - When: 差分の確認を出す
  - Then: 足した行が #859900、消した行が #dc322f。
- Acceptance:
  - Given: テーマなし
  - When: 表を出す
  - Then: 今までと同じ色。

#### テーマのためる変更・差分・検索の一致の色は意味を足すだけで、印(下線・`+`・`-`)はテーマなしと同じに保つべき
`req~screen.theme-meaning-colors-marks~1` P1
- Acceptance:
  - Given:
  - When: Solarized Light で保存の確認の差分を開く
  - Then: 足した行の `+` と消した行の `-` は、テーマなしと同じ字と位置。

#### 表のプロファイル(cli.config-file の `[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]` と `use`)は、全体・ワークスペース・表・ビューの4つの範囲に書けるべき
`req~screen.profile-scopes-sections~1` P1
- Acceptance:
  - Given: 表に `[dates] format = "YYYY/MM/DD"`
  - When:
  - Then: その表だけ日付が `/` 区切り。

#### 表のプロファイルの全体は config.toml と ui.toml、ワークスペースはその印 `.mdgrid/workspace.toml`(workspace.marker)か workspaces.toml の `[[workspace]]`(workspace.app-side)、表はワークスペースの表の項目(印の `[[table]]`・`[[workspace.table]]`)と views.toml の `[[table]]`(base-view.views-toml)、ビューはビューの設定(navigation.settings-remember。mdgrid のビューは保存で定義にも入る。base-view.save-view)に書く
`req~screen.profile-scopes-files~1` P1
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "nord"`、その印の tasks の `[table.look]` に theme = "dracula"
  - When:
  - Then: notes/tasks の地が dracula、notes/projects は nord。

#### 表のプロファイルのワークスペースの範囲は workspace.scope-order で決めた範囲のものとし、検知(workspace.detect-roots)と登録した表(cli.places)から来た範囲にはワークスペースの範囲は無い
`req~screen.profile-scopes-workspace~1` P1
- Acceptance:
  - Given:
  - When: 検知(workspace.detect-roots)から来た範囲で表を開く
  - Then: ワークスペースの範囲の値を使わず、表・全体・既定の値で描く。

#### 表のプロファイルの効く値は項目ごとに、狭い範囲を広い範囲より先にし(ビュー → 表 → ワークスペース → 全体 → 既定)、同じ範囲に2つあるときは画面が書いたものを手で書いたものより先にする(全体は ui.toml → config.toml、表は views.toml → ワークスペースの表の項目)べき(ワークスペースの印に書いた `[look]` だけは、screen.profile-scopes-marker-look のとおり全体より広い範囲とする)
`req~screen.profile-scopes-order~3` P1
- Acceptance:
  - Given: config.toml に `[look] theme = "sumi"`、workspaces.toml のワークスペース W(表 notes/tasks・notes/projects)に `[workspace.look] theme = "nord"`、views.toml の notes/tasks の `[table.look] theme = "dracula"`
  - When:
  - Then: notes/tasks の地が dracula、notes/projects は nord、W の外の表は sumi。
- Acceptance:
  - Given: 別に、ui.toml に `[look] theme = "dracula"`、config.toml に `[look] theme = "sumi"` だけ
  - When:
  - Then: どの表も dracula。

#### 組(`look.preset`)を書いた範囲より広い範囲(screen.profile-scopes-order・screen.profile-scopes-marker-look の順で先に重ねる範囲)の部品ごとの形(`[look.style]`)は、使わないべき
`req~screen.profile-scopes-preset-cut~2` P1
- Acceptance:
  - Given: workspaces.toml のワークスペースに `[look] preset = "saas"`、config.toml に `[look.style] status = "chip"`
  - When:
  - Then: ワークスペースの表の status は saas の形。

#### テーマ(`look.theme`)を書いた範囲より広い範囲(screen.profile-scopes-order・screen.profile-scopes-marker-look の順で先に重ねる範囲)の役割の色(`[look.colors]` の役割。値の色 `[look.colors.values]` は除く)は、使わないべき
`req~screen.profile-scopes-theme-cut~2` P1
- Acceptance:
  - Given: workspaces.toml のワークスペースに `[look] theme = "nord"`、config.toml に `[look.colors] accent = "orange"`
  - When:
  - Then: accent は nord の色。

#### `use = "<名前>"` は、テンプレート(`[templates.<名前>]`。同じ名前は ui.toml を config.toml より先にする)を、その範囲で書いた値の下に敷くべきで、無い名前は警告して無視するべき
`req~screen.profile-scopes-use~2` P1
- Acceptance:
  - Given: 表に `use = "夜"`(`[templates.夜.look] theme = "dracula"`)と `[look] preset = "grid"`
  - When:
  - Then: dracula と grid。
- Acceptance:
  - Given: `use = "無い"`
  - When:
  - Then: 警告。

#### アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、`[terminal]`・`[workspace]`・`[keys]`)は全体にだけ書け、ほかの範囲に書けば警告して無視するべき
`req~screen.profile-scopes-app-only~1` P1
- Acceptance:
  - Given: ワークスペースに `[terminal] color = false`
  - When:
  - Then: 警告して無視。

#### 表を切り替えたら(workspace.palette・cli.places-list・関係マップから開く)、その表で効く表のプロファイルの値で画面を描き直すべき
`req~screen.profile-scopes-redraw~2` P1
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に nord、views.toml の notes/tasks に dracula
  - When: notes/tasks を開き、関係マップから notes/projects を開く
  - Then: nord で描き直す。

#### 関係マップのように1つの画面に複数の表が出るときは、今開いている表のプロファイルの値を使うべき
`req~screen.profile-scopes-relation-map~1` P1
- Acceptance:
  - Given:
  - When: notes/tasks に dracula、notes/projects に nord の設定で、notes/tasks から関係マップを開く
  - Then: 関係マップの地は dracula。

#### ワークスペースの印(workspace.marker)に書いた `[look]`(最上位の `[look]` と、`[[table]]` の `[table.look]`)は、台帳の作り手が勧める既定として、重ねるときは全体より広い範囲(既定のすぐ上。印の表の項目は印の最上位より狭い)とし、人の選んだ見た目(config.toml・ui.toml・workspaces.toml・views.toml・ビューの設定)のどれもを印より先にするべき(`[look]` のほかの区画は screen.profile-scopes-order のとおり)
`req~screen.profile-scopes-marker-look~1` P1
- Rationale: 台帳の印に `[look] theme` を書いておくと、人が画面で全体のテーマを選んでも印が勝って変わらなかったため。印は台帳と一緒に配り分け合うもので、見た目は使う人の好みを先にする
- Needs: utest・stest
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "dozy-pink"`、ui.toml に `[look] theme = "sumi"`
  - When: notes/tasks を開く
  - Then: 地が sumi で、「見た目」の区画のテーマの横は「(全体)」。
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "dozy-pink"`、config.toml に `[look] theme = "sumi"`
  - When: notes/tasks を開く
  - Then: 地が sumi。
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "nord"`・`preset = "saas"` と、tasks の `[table.look]` に theme = "dracula"、config.toml と ui.toml に `[look]` が無い
  - When: notes/tasks と notes/projects を開く
  - Then: tasks の地は dracula、projects は nord、どちらも組は saas。
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[look] theme = "nord"`・`preset = "saas"`、config.toml に `[look] theme = "sumi"`
  - When: notes/tasks を開く
  - Then: 地は sumi、組は saas(項目ごとに重ねる)。
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml に `[display] zebra = true`、config.toml に `[display] zebra = false`
  - When: notes/tasks を開く
  - Then: 一行おきの色がある(`[display]` は印のワークスペースの範囲が全体より先のまま)。

#### アイコン用のフォント(Nerd Font)の記号は、設定 `terminal.nerd_icons`(真偽値。既定 false。全体だけ)を true にしたときだけ使うべきで、既定では今の字の記号(設定のボタンの `⚙`・画面の型のメニューのボタンの `▦`・`◇`・列の見出しの型の印 `#`・`◷`・`⋮`・`◉`)のままにするべき
`req~screen.nerd-icons-setting~1` P1
- Acceptance:
  - Given: 既定
  - When:
  - Then: ヘッダーに `⚙ 設定`・`▦ 表 ▾`、日付の列の見出しに `◷`。

#### `terminal.nerd_icons` が true のときは、設定のボタンの歯車・画面の型のメニューのボタンと窓(表・関係マップ)・列の見出しの型の印(screen.rich-cells。数・日付・日時・チェック・リスト・札)・日付のカレンダーの題に、決まったアイコン用のフォントの記号を使うべき
`req~screen.nerd-icons-symbols~1` P1
- Acceptance:
  - Given: `[terminal] nerd_icons = true`
  - When:
  - Then: 設定のボタン・表のボタン・日付の列の見出し・カレンダーの題にアイコン用のフォントの記号(私用の領域の字)が出て、`⚙`・`▦`・`◷` は出ない。

#### `terminal.nerd_icons` が true のときも、アイコンに絵文字は使わないべき
`req~screen.nerd-icons-no-emoji~1` P1
- Acceptance:
  - Given: `[terminal] nerd_icons = true`
  - When:
  - Then: ヘッダー・列の見出し・カレンダーの題に絵文字の字が無い。

#### アイコンの記号の幅は字の幅の決まり(cell-view.grapheme-width・cell-view.ambiguous-width)で数え、列の位置とクリックの当たりをずらさないべき
`req~screen.nerd-icons-width~1` P1
- Acceptance:
  - Given: `[terminal] nerd_icons = true`
  - When:
  - Then: どの行も画面の幅を超えない。

#### ノートの欄(screen.note-column)のフォルダの出し方 `display.note_path`(`"common"`・`"name"`・`"full"`・`"off"`。既定 `"common"`)を、表のプロファイルの項目として全体・ワークスペース・表・ビューの範囲(screen.profile-scopes)に書けるべき
`req~screen.note-path-scopes~1` P1
- Acceptance:
  - Given: config.toml に `[display] note_path = "name"`
  - When:
  - Then: どの表の欄も名前だけ。`--print-config` の出力と docs/config.md に項目がある。

#### `display.note_path` の4つの値は、ビューの設定の「表示」の区画(navigation.settings-layout)で順に切り替えられ、mdgrid のビュー(base-view.own-views)に保存できるべき
`req~screen.note-path-per-view~2` P1
- Acceptance:
  - Given: ビューの設定の「表示」で「ノートのパス」を `full` にして反映
  - When:
  - Then: そのビューだけ根からのパス。
- Acceptance:
  - Given:
  - When: もう一度押して `off`
  - Then: 欄に名前が出ない。
- Acceptance:
  - Given:
  - When: 名前を付けて保存して開き直す
  - Then: 同じ。

#### `display.note_path` の知らない値は、警告して既定にするべき(cli.config-file)
`req~screen.note-path-unknown~1` P1
- Acceptance:
  - Given: `note_path = "short"`
  - When:
  - Then: `config.toml: display.note_path:` で始まる警告と既定の見た目。

#### ノートの欄(screen.note-column)の名前(`display.note_path` で決めた、`.md` を除いた文字)の出し方を、正規表現 `display.note_name`(文字列。既定なし)で決められるべき
`req~screen.note-name-regex~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `note_path = "common"` と `'^\d{8}_'` で `todo/20261015_a.md`・`archive/todo/20261001_b.md` → `todo/a`・`archive/todo/b`(フォルダは残し、最後の名前の部分に当てる)。

#### ノートの名前を見せる所(関係マップのつながった行とよく指される行・木の画面の葉・横断の検索の結果・リンクの列を直すときの候補・操作の一覧の「つながった行」の一覧・行き先が2つ以上のリンクを開くときの一覧)にも、ノートの欄と同じ見せ方(`display.note_path` と `display.note_name`)を当てるべき(メッセージ・詳しく見る画面・検索で当てる文字・書くリンクの値は変えない)
`req~screen.note-name-everywhere~2` P1
- Rationale: ノートの欄とリンクのセルでは日付の頭(`20261120_`)を隠しているのに、関係マップのつながった行などには頭が出て、同じノートが2つの名前で見えていたため(2026-11-25 に、リンクのセルの頭を直したときに、同じ形の所を洗い出した。同じ日の実地の操作で、操作の一覧の「つながった行」の一覧にも頭が残っていた)
- Acceptance:
  - Given: `[display] note_name = '^\d{8}_'` と、`20261121_画面` が `parent: "[[20261120_設計]]"` で指す表
  - When: 関係マップで parent のつながりを選ぶ
  - Then: つながった行は「画面 → 設計」、よく指される行は「設計」。
- Acceptance:
  - Given: 同じ設定で、木の設定(`tree`)のある表の `20261120_設計`
  - When: 木の画面を開く
  - Then: 葉の名前は「設計」。
- Acceptance:
  - Given: 同じ設定のワークスペース
  - When: 横断の検索で「設計」を探す
  - Then: 結果の名前は「設計」。
- Acceptance:
  - Given: 同じ設定で、parent がリンクの列の表
  - When: 「画面」の parent のセルを直して候補を出す
  - Then: 候補の名前は「設計」で、選ぶと `[[20261120_設計]]` を書く。
- Acceptance:
  - Given: 同じ設定で、`20261121_画面` が parent で `20261120_設計` を指す表「Notes」
  - When: 「設計」の行で、操作の一覧の「つながった行」を開く
  - Then: 一覧の行は「Notes › parent ← 画面」で、選ぶと `20261121_画面` の行を開く。
- Acceptance:
  - Given: 同じ設定で、`20261121_画面` の related が `20261120_設計` と `20261119_要件` を指す
  - When: 「画面」の related のセルで「行き先を開く」
  - Then: 選ぶ一覧の行は「設計」と「要件」。
- Acceptance:
  - Given: `display.note_name` を書かない表
  - When: 関係マップを開く
  - Then: つながった行は「20261121_画面 → 20261120_設計」のまま。
- Needs: utest

#### 印の `[[table]]` に `name` を書いた表は、パンくず・最近開いた表の一覧・通知の欄の今の表に、フォルダの名前でなくその `name` を見せるべき
`req~screen.table-name-marker~1` P1
- Rationale: 引き出しと印では「予定」なのに、開くとパンくずと最近開いた表の一覧が「calendar」(path のフォルダの名前)になり、同じ表が2つの名前で見えていたため(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: 印の `[[table]]` に `name = "予定"`・`path = "calendar"` の表
  - When: その表を開く
  - Then: パンくずは「<ワークスペース> › 予定」。
- Acceptance:
  - Given: 同じ表を開いたことがある
  - When: 最近開いた表の一覧を開く
  - Then: 行の名前は「予定」。
- Acceptance:
  - Given: 印の `[[table]]` に `path = "calendar"` だけを書いた表
  - When: その表を開く
  - Then: パンくずは今までどおり「calendar」。
- Needs: utest

#### `display.note_name` は、表のプロファイルの項目として全体・ワークスペース・表・ビューの範囲(screen.profile-scopes)に書けるべき
`req~screen.note-name-scopes~1` P1
- Acceptance:
  - Given:
  - When: views.toml のある表の `[table.display]` に `note_name = '^\d{8}_'` を書く
  - Then: その表の欄だけ `20261015_資料を作る.md` が `資料を作る`、ほかの表は名前のまま。

#### 名前が `display.note_name` の正規表現に一致したときは、名前の付いた組 `name` があればその部分を、無ければ1つ目の組があればその部分を、組が無ければ一致した部分を除いた残りを出し、一致しないときと、結果が空になるときは名前をそのまま出すべき
`req~screen.note-name-match~2` P1
- Acceptance:
  - Given:
  - When: `note_path = "name"`・`note_name = '^\d{8}_'` で `todo/20261015_資料を作る.md`
  - Then: 欄は `資料を作る`。
- Acceptance:
  - Given: `note_name = '^\d{8}_(?P<name>.+)$'`
  - When:
  - Then: 同じ。`note_name = '^(.+)_v\d+$'` で `企画_v3.md` → `企画`。一致しない `メモ.md` → `メモ`。
- Acceptance:
  - Given: `note_name = '^.*$'`(全部を除くと空)
  - When:
  - Then: 名前のまま。

#### `display.note_name` の読めない正規表現と文字列でない値は、cli.config-file の形で警告して使わないべき
`req~screen.note-name-invalid~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `note_name = '('` → `config.toml: display.note_name:` で始まる警告と、名前のまま。

#### `display.note_name` の項目を、`--print-config` と docs/config.md に置くべき
`req~screen.note-name-docs~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `--print-config` の `[display]` と docs/config.md に `note_name` の項目がある。

#### `display.note_name` は、ビューの設定の「表示」の区画(navigation.settings-layout)では切り替えない(ビューごとの値は views.toml のビューの設定の `display` に手で書ける)
`req~screen.note-name-not-in-settings~1` P1
- Rationale: 自由に書く文字列で、順に切り替える値が無いため
- Acceptance:
  - Given: ビューの設定の「表示」の区画
  - When:
  - Then: 「ノートのパス」の項目はあり、note_name の項目は無い。

#### `display.note_name` で欄の名前を変えても、検索(navigation.find)と、メッセージ・詳しく見る画面のノートの名前は変えないべき
`req~screen.note-name-search-kept~1` P1
- Acceptance:
  - Given:
  - When: `note_name = '^\d{8}_'` で欄が `資料を作る` のノート `20261015_資料を作る.md`
  - Then: `\` の検索で `20261015` と打つと当たる。

#### 表の画面の列の見出しでは、計算の列(`formula.x`)の見出しの頭に `ƒ` と空白を付けて、ノートのキー(フロントマター)の列と見分けられるようにするべきで、色を使わない表示と見た目の設定(screen.rich-cells の型の印の有無)に依らず付けるべき
`req~screen.formula-mark-prefix~1` P1
- Rationale: 色や設定で消えると見分けられないため
- Acceptance:
  - Given:
  - When: 計算の列 `formula.担当` とキーの列 `title`・`actor` を並べた表
  - Then: 見出しの行に「ƒ 担当」があり、ƒ は1つだけ。`--no-color` でも同じ。

#### 計算の列の列の幅は、見出しの `ƒ` の印を含めて数えるべき
`req~screen.formula-mark-width~1` P1
- Acceptance:
  - Given: 計算の列 `formula.担当` の値がどれも2桁以下
  - When:
  - Then: 列の幅は「ƒ 担当」の6桁以上で、見出しが切れない。

#### `--print` などの出力の見出しは、計算の列でも変えない(`ƒ` を付けない)べき
`req~screen.formula-mark-print~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `--print` の見出しは「担当」で ƒ が無い。

#### リポの説明書(`docs/manual/<言語>/views.md`。サイトでは説明書の1章)に、画面の型と見せ方の一覧のページを置くべき
`req~screen.views-page-file~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `docs/manual/ja/views.md` と `docs/manual/en/views.md` があり、`scripts/pages.py` の `PAGES` に `views.md` がある。

#### 見せ方の一覧のページは、1つの見せ方につき、本物の画面の画像1枚(`docs/manual-scenarios.toml` の場面)と、何が見えてどう開くかの説明と、くわしい節へのリンクを並べる
`req~screen.views-page-entry~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: views.md の画像が全部場面の id で、相対のリンク先が全部ある。

#### 見せ方の一覧のページに載せる見せ方は、少なくとも、表・ビューのタブ・まとまり(グループ)・木・関係マップ・値の分布・絞り込みの帯・テーマと部品の形の8つ
`req~screen.views-page-items~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: views.md の見出しに、題の8つの見せ方がある。

#### サイトの入口(`scripts/pages.py` の「見て選ぶ」)から、見せ方の一覧のページへ行けるべき
`req~screen.views-page-link~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 入口の文言に、見せ方の一覧(views.md)へのリンクがある。

#### 画面の型(表・木・関係マップなど)を足す変更は、見せ方の一覧のページに1項を足すべき
`req~screen.views-page-keep~1` P1
- Acceptance:
  - Given: 画面の型を足した変更の記録で、この一覧に項が無い
  - When:
  - Then: 違反(読んで判定)。

#### 説明書の目次(`docs/manual/<言語>/index.md`)と、サイトの説明書の章の並び(`scripts/pages.py` の `PAGES`)は、読む人の順に並べるべき: 誰のための道具で何がどう見えるか(index)→ 見せ方の一覧(views)→ はじめかた(getting-started。1分で試す道を1本)→ 作業の手引き(tasks)→ 考え方(concepts)→ 見た目(appearance・themes)→ 困ったとき(faq)→ 参照
`req~screen.docs-order-toc~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `docs/manual/ja/index.md` と `en/index.md` の目次のリンクが views → getting-started → tasks → concepts → appearance → faq の順に並び、`PAGES` も同じ順である。

#### 説明書とサイトの入口は短く保ち、詳しさは下のページへ送るべき
`req~screen.docs-order-short-entry~1` P1
- Acceptance:
  - Given:
  - When: 入口(`docs/manual/<言語>/index.md` と `scripts/pages.py` の入口の文言)を読む
  - Then: 詳しい手順と設定の説明は無く、下のページへのリンクで送っている(読んで判定)。

#### 説明書・サイト・README の文は `docs/writing.md` の決まり(具体から書く、語尾と長さを変える、接続詞を連ねない、箇条書きは並列のものだけ、大げさな語・対比の型・三つ組・太字の強調・締めの定型句を使わない)で書くべき
`req~screen.docs-order-style~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 入口と説明書の文に、大げさな語(強力・シームレス・直感的・画期的・最適・豊富 / seamless・robust・powerful・intuitive・leverage・empower・delve・crucial・pivotal)が無い。文の決まりに合うかは、`docs/writing.md` の一覧に照らして読んで判定。

#### リポの説明書(`docs/manual/<言語>/features.md`。サイトでは説明書の1章で、見せ方の一覧の次)に、できること(機能)の一覧のページを置くべき
`req~screen.features-page-file~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `docs/manual/ja/features.md` と `docs/manual/en/features.md` があり、`scripts/pages.py` の `PAGES` で `features.md` が `views.md` の直後にある。

#### 機能の一覧のページは、1つの機能につき、本物の画面の画像1枚(`docs/manual-scenarios.toml` の場面)と、何ができてどう始めるかの説明と、くわしい節へのリンクを並べる
`req~screen.features-page-entry~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: features.md の画像が全部場面の id で、どの機能の節にも画像1枚とくわしい節へのリンクがあり、相対のリンク先が全部ある。

#### 機能の一覧のページに載せる機能は、少なくとも、値をその場で直す・差分を見て保存する・まとめて直す・探す/絞る/並べる・新しいノートを作る・ビューを決めて保存する・`.base` を開く・リンクで表をつなぐ・スターターから台帳を作る・ほかのコマンドと組む・読むだけで開く・見た目を選ぶの12
`req~screen.features-page-items~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: features.md に、題の 12 の機能それぞれの節がある。

#### サイトの入口(`scripts/pages.py` の「見て選ぶ」)から、機能の一覧のページへ行けるべき
`req~screen.features-page-link~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 入口の文言に、機能の一覧(features.md)へのリンクがある。

#### 機能を足す変更は、機能の一覧のページに1項を足すべき
`req~screen.features-page-keep~1` P1
- Acceptance:
  - Given: 機能を足した変更の記録で、この一覧に項が無い
  - When:
  - Then: 違反(読んで判定)。

## 未確定事項

### vim モードを入れる設定の名前と置き場
`tbd~screen.vim-setting-name`
- Decider: 人
- Provisional: `[keys]` の `vim`(真偽。既定は `false`)

### 開ける URL のセルをマウスで直す道
`tbd~screen.url-click-edit`
- Decider: 人
- Provisional: 開ける URL のセル(screen.url-click-open)は、クリックでは直さない(1回目は選ぶだけ、2回目は開く)。直すときは、選んで `Enter` か、右クリックの操作の一覧の「直す」から。⌘+クリックで開ける端末でも、mdgrid は ⌘ の付いたクリックを受け取らないので、同じ振る舞いにする
- Basis: 2026-11-25 に人が「選んでいる URL のセルをもう一度クリックすると開く。今、選んでいるセルのクリックに別の振る舞い(編集など)があれば、URL のセルでは開くを先にするか、食い違いを報告して仮に決める」と決めた

### 印の order の控えを持たない、前の版の見た目の状態の列の並び
`tbd~screen.state-store-legacy-order`
- Decider: 人
- Provisional: 前の版は、開いただけでも列の並びを状態に書いていて、人が動かした並びと区別できない。そこで、覚えたときの `order` の控えを持たない状態の並びは、印のビューに `order` があれば捨てて `order` に合わせる(隠した列・幅・畳んだまとまり・ビューの設定は残す)。前の版で人が動かした並びは一度だけ戻る。
- Basis: 2026-11-25 の実地の操作で、印の `order` に足した列が右の端に回った(screen.state-store-follow-order)。人は細かい決めを任せると言った。
