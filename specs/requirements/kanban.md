# カンバン(状態の列の値ごとのレーンの画面) の要求

## 目的

表(と木・関係マップ・ガント)に続く画面の型。今のビューの行を、状態の列の値ごとのレーン(縦の列)に札として並べ、全部の状態を1枚で見せる。レーンの順は列の決まり(cell-edit.rules の `values`)から取り、札を隣のレーンへ動かすと、表のセルの直しと同じためる変更として状態の値を書き換える。設定は印の `[[table]]` の `kanban = { … }` で、無ければ列の名前から推定する(gantt.screen-config の一般の形)。扱わない: 札に値を添えること、レーンの中の並べ替え、WIP の上限、`.base` の kanban のビュー(scope.table-only)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/kanban/spec.md](../_history/decidespec/kanban/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- `.base` の kanban のビューを開くこと(scope.table-only)。カンバンの設定は印の `[[table]]` の `kanban` に書く(gantt.screen-config)。
- 札の値を消すこと(「(なし)」のレーンへ動かすこと)。値を消すのは表のセルの直しでする。

## 次の段(まだ作っていない。P2)

- 札に値を添える(`kanban = { column, card = ["due", "priority"] }` で札の2行目に)。
- レーンの中の並べ替えと、WIP の上限の印。

## 前提(旧い仕様で AI が置いた仮定)

- 行の並びは表(ビューの設定)が作る行の順をそのまま使う(カンバンで独自の並びを持たない)。親子(navigation.parent-child)の畳みは見ず、ビューを通った全部の行を札にする。
- 推定の列の名前の一覧は固定(kanban.config)。外れたら印で上書きする。

## 要求

### ワークスペースの印の `[[table]]` に `kanban = { column = "<列>" }` を書いてよく、その列の値ごとに行を分けるべき
`req~kanban.config-column~1` P1
- 受け入れ: `kanban = { column = "status" }` の表 → 画面の型のメニューに「カンバン」が出て `B` で開く。

### カンバンの設定が無いときは、列の名前から分ける列を推定してよい(status・state・stage・状態・ステータス・段階。大文字小文字を問わない。リストの型の列は選ばない)
`req~kanban.config-guess~1` P1
- 受け入れ: 設定なしで status の列がある → 推定して開き、画面の題に推定と示す。

### カンバンの設定が無く、推定でも列が決まらないときは、理由を出してカンバンを出さないべき
`req~kanban.config-guess-none~1` P1
- 受け入れ: 設定なしで状態らしい列が無い → 理由1行で開かない。

### 設定の `column` の列が表に無いか、リストの型の列のときは、cli.config-file の形で警告してカンバンを出さないべき
`req~kanban.config-bad-column~1` P1
- なぜ: 1行が複数のレーンに属さないようにするため
- 受け入れ: `column = "none"` → 警告1行で、メニューに出ない。

### 設定の `[types]`・`[rules.<列>]` に書いた列は、その列を持つノートが1件も無くても、カンバンの設定の列として表にあると数えるべき
`req~kanban.config-declared-column~1` P1
- なぜ: 状態をまだ書いていないノートだけのときに、設定で決めた列が無いと言わないようにするため
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]`、`kanban = { column = "status" }` の表に、status を持たないノートが1件 → 警告せず、`B` でカンバンが開き、レーン todo・doing・done が並ぶ。`column = "none"`(設定にもノートにも無い)→ kanban.config-bad-column のとおり警告1行。

### 行が1つも無い表(見本なしで作った台帳など)では、カンバンの設定の列があるかを決めず、警告しないべき(画面の型のメニューには出し、開くときに出す行が無いと理由を出す)
`req~kanban.config-empty-table~2` P1
- なぜ: 行が1つも無い表では、列をノートのキーから作るため
- 受け入れ: 行が1つも無い表 → 読み終えても警告せず、メニューに出て、開くと理由1行(列に決まりの値があるときは、kanban.open-empty-lanes のとおり空のレーンで開く)。外からノートを足すと `B` で開く。

### 行が1つも無くても、カンバンの列に決まりの値(cell-edit.rules-values)があるときは、その値のレーンを札 0 枚で並べてカンバンを開くべき
`req~kanban.open-empty-lanes~1` P1
- なぜ: 見本なしで作った台帳でも、設定で決めた状態の並びを最初から見られるようにするため
- 受け入れ: ノートが1件も無い表で `[rules.status] values = ["todo", "doing", "done"]`、`kanban = { column = "status" }` → `B` でカンバンが開き、レーン todo (0)・doing (0)・done (0) がこの順で並ぶ。札を動かすキーを押しても何もためない。決まりの値の無い列で行が1つも無い → kanban.config-empty-table のとおり開かずに理由1行。

### `B`(か画面の型のメニューの「カンバン」)で、今のビューの行(絞り込み・並べ替えのあと。tree.screen と同じ)を、列の値ごとのレーン(縦の列)に札として並べて見せるべき
`req~kanban.open-lanes~1` P1
- 受け入れ: status が todo 2件・doing 1件のビューで `B` → todo のレーンに札2枚、doing のレーンに札1枚。絞り込みで外した行は札にならない。

### カンバンのレーンの順は、列の決まり(cell-edit.rules の `values`)があればその順、続いて決まりに無い値を行に現れた順、最後に値の無い行の「(なし)」にするべき
`req~kanban.open-lane-order~1` P1
- 受け入れ: 決まり `values = ["todo", "doing", "waiting", "done"]` の status で todo 2件・doing 1件・done 1件・状態なし1件 → レーンは todo(2)・doing(1)・waiting(0)・done(1)・(なし)(1) の順(行の無い waiting もレーンにする)。決まりに無い値 blocked の行がある → done と (なし) の間に blocked のレーン。状態なしの行が無い → (なし) のレーンは無い。

### カンバンのレーンの見出しには、値と札の数を出すべき
`req~kanban.open-lane-heading~1` P1
- 受け入れ: status が todo の行が2件 → todo のレーンの見出しに、値 todo と札の数 2 が出る。

### カンバンの札は、行の名前(表のノートの欄と同じ見せ方)の1行にするべき
`req~kanban.open-card-label~1` P1
- 受け入れ: 表のノートの欄に `会議` と出る行 → 札は `会議` の1行。

### カンバンのレーンの中の札は、表の行の順に並べるべき
`req~kanban.open-card-order~1` P1
- 受け入れ: 表で上から alpha・beta の順に並ぶ todo の2件 → todo のレーンの札も alpha・beta の順。

### カンバンのレーンが幅に収まらないときは、選んでいるレーンが見える所まで横にずらすべき
`req~kanban.open-scroll~1` P1
- 受け入れ: 幅 40 で5つのレーン → 選んでいるレーンが見える。

### カンバンのレーンが幅に収まらないときは、隠れたレーンがあることを示すべき
`req~kanban.open-hidden-mark~1` P1
- 受け入れ: 幅 40 で5つのレーン → 隠れた側に印。

### 色なしでも、カンバンの選んでいる札とレーン、ためる変更のある札を見分けられるべき(cell-view.not-color-only)
`req~kanban.open-no-color~1` P1
- 受け入れ: `--no-color` でも選んでいる札に `>`。

### カンバンの画面では、`j` `k` でレーンの中の札を選び、`h` `l` でレーンを移るべき
`req~kanban.keys-move~1` P1
- 受け入れ: `B` → `l` → `j` で、doing のレーンの2件目の札が選ばれる。

### カンバンの画面では、`Enter` で表に戻って、選んでいる札の行を選ぶべき
`req~kanban.keys-enter~1` P1
- 受け入れ: `B` → `l` → `j` → `Enter` で、表の doing の2件目の行が選ばれる。

### カンバンの画面では、`Esc`・`q`・`B` で表に戻るべき
`req~kanban.keys-back~1` P1
- 受け入れ: カンバンで `Esc`・`q`・`B` のどれか → 表に戻る。

### カンバンの画面では、札のクリックでその札を選ぶべき
`req~kanban.keys-click-select~1` P1
- 受け入れ: 札のクリック → その札を選ぶ。

### カンバンの画面では、選んでいる札をもう一度クリックすると、表のその行へ移るべき
`req~kanban.keys-click-open~1` P1
- 受け入れ: 選んでいる札をもう一度クリック → 表のその行が選ばれる。

### カンバンの画面では、ホイールで札を、横のホイールでレーンを動かすべき
`req~kanban.keys-wheel~1` P1
- 受け入れ: ホイール → レーンの中で選んでいる札が移る。横のホイール → 選んでいるレーンが移る。

### カンバンの画面では、`H` `L` で選んでいる札を左・右のレーンへ動かし、その行の列の値を、動かした先のレーンの値に書き換えるべき
`req~kanban.move-card~1` P1
- 受け入れ: todo の札で `L` → 札が doing のレーンへ移り、その行の status が doing になる。

### カンバンで札を動かした値の書き換えは、表のセルの直しと同じためる変更(未保存)にするべき
`req~kanban.move-pending~1` P1
- 受け入れ: todo の札で `L` → 下の帯の未保存が 1。

### カンバンで動かした札は、動かした先のレーンで選んだままにするべき
`req~kanban.move-keep-selection~1` P1
- 受け入れ: todo の札で `L` → doing のレーンへ移った札を選んだまま。

### カンバンの画面では、`u` で札の動きを取り消すべき
`req~kanban.move-undo~1` P1
- 受け入れ: todo の札で `L` のあと `u` → todo に戻る。

### カンバンの画面では、`U` で取り消した札の動きをやり直すべき
`req~kanban.move-redo~1` P1
- 受け入れ: todo の札で `L`・`u` のあと `U` → 札が doing のレーンへ移る。

### カンバンの画面では、`Ctrl+s` で表に戻って保存の確かめ(差分)を開くべき
`req~kanban.move-save~1` P1
- 受け入れ: todo の札で `L` のあと `Ctrl+s` → 表に戻り、保存の確かめ(差分)が開く。

### カンバンの札は、「(なし)」のレーンへは動かさないべき(値を消すのは表で)
`req~kanban.move-not-to-none~1` P1
- 受け入れ: 右の隣が (なし) の札で `L` → 動かず理由1行。

### 読むだけのとき(`--readonly`、読むだけの列や行。cli.options・base-view.unsupported・base-view.types-conflict)は、カンバンの札を動かさずに理由を出すべき
`req~kanban.move-readonly~1` P1
- 受け入れ: `--readonly` で `L` → 動かず理由1行。

### カンバンを開いているあいだにノートが外で変わったら、組み直して同じ行の札を選び直すべき(札が別のレーンへ移っていればそのレーンで)
`req~kanban.reload~1` P1
- 受け入れ: 開いたまま外で status を todo から done に書き換える → 札が done のレーンに移り、選んだまま。

### 見せ方の一覧(screen.views-page)・できることの一覧(screen.features-page)・キーの文書(screen.keys-docs)・設定の文書にカンバンの項を持ち、組み込みの tasks のスターターに `kanban` の設定を同梱し、場面の台本にカンバンの場面を持つべき
`req~kanban.docs~1` P1
- 受け入れ: docs/manual/ja・en の views.md に「カンバン」の節(画像 images/kanban.svg)、features.md に札を動かして状態を直す項。starters/tasks の印に `kanban = { column = "status" }`。docs/manual-scenarios.toml に id = "kanban"(読んで判定と)。
