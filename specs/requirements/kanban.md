# カンバン(状態の列の値ごとのレーンの画面) の要求

## 目的

表(と木・関係マップ・ガント)に続く画面の型。今のビューの行を、状態の列の値ごとのレーン(縦の列)に札として並べ、全部の状態を1枚で見せる。レーンの順は列の決まり(cell-edit.rules の `values`)から取り、札を隣のレーンへ動かすと、表のセルの直しと同じためる変更として状態の値を書き換える。設定は印の `[[table]]` の `kanban = { … }` で、無ければ列の名前から推定する(gantt.screen-config の一般の形)。札には名前の下に、印の `card` に書いた列の値を添えられる。扱わない: レーンの中の並べ替え、WIP の上限、`.base` の kanban のビュー(scope.table-only)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/kanban/spec.md](../_history/decidespec/kanban/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- `.base` の kanban のビューを開くこと(scope.table-only)。カンバンの設定は印の `[[table]]` の `kanban` に書く(gantt.screen-config)。
- 札の値を消すこと(「(なし)」のレーンへ動かすこと)。値を消すのは表のセルの直しでする。

## 次の段(まだ作っていない。P2)

- レーンの中の並べ替えと、WIP の上限の印。

## 前提(旧い仕様で AI が置いた仮定)

- 行の並びは表(ビューの設定)が作る行の順をそのまま使う(カンバンで独自の並びを持たない)。親子(navigation.parent-child)の畳みは見ず、ビューを通った全部の行を札にする。
- 推定の列の名前の一覧は固定(kanban.config)。外れたら印で上書きする。

## 要求

### ワークスペースの印の `[[table]]` に `kanban = { column = "<列>" }` を書いてよく、その列の値ごとに行を分けるべき
`req~kanban.config-column~1` P1
- Acceptance:
  - Given: `kanban = { column = "status" }` の表
  - When:
  - Then: 画面の型のメニューに「カンバン」が出て `B` で開く。

### カンバンの設定が無いときは、列の名前から分ける列を推定してよい(status・state・stage・状態・ステータス・段階。大文字小文字を問わない。リストの型の列は選ばない)
`req~kanban.config-guess~1` P1
- Acceptance:
  - Given: 設定なしで status の列がある
  - When:
  - Then: 推定して開き、画面の題に推定と示す。

### カンバンの設定が無く、推定でも列が決まらないときは、理由を出してカンバンを出さないべき
`req~kanban.config-guess-none~1` P1
- Acceptance:
  - Given: 設定なしで状態らしい列が無い
  - When:
  - Then: 理由1行で開かない。

### 設定の `column` の列が表に無いか、リストの型の列のときは、cli.config-file の形で警告してカンバンを出さないべき
`req~kanban.config-bad-column~1` P1
- Rationale: 1行が複数のレーンに属さないようにするため
- Acceptance:
  - Given: `column = "none"`
  - When:
  - Then: 警告1行で、メニューに出ない。

### 設定の `[types]`・`[rules.<列>]` に書いた列は、その列を持つノートが1件も無くても、カンバンの設定の列として表にあると数えるべき
`req~kanban.config-declared-column~2` P1
- Rationale: 状態をまだ書いていないノートだけのときに、設定で決めた列が無いと言わないようにするため
- Acceptance:
  - Given: `[rules.status] values = ["todo", "doing", "done"]`、`kanban = { column = "status" }` の表に、status を持たないノートが1件
  - When:
  - Then: 警告せず、`B` でカンバンが開き、レーン todo・doing・done が並ぶ。
- Acceptance:
  - Given: `column = "none"`(設定にもノートにも無い)
  - When:
  - Then: kanban.config-bad-column のとおり警告1行。

### 行が1つも無い表(見本なしで作った台帳など)では、カンバンの設定の列があるかを決めず、警告しないべき(画面の型のメニューには出し、開くときに出す行が無いと理由を出す)
`req~kanban.config-empty-table~2` P1
- Rationale: 行が1つも無い表では、列をノートのキーから作るため
- Acceptance:
  - Given: 行が1つも無い表
  - When:
  - Then: 読み終えても警告せず、メニューに出て、開くと理由1行(列に決まりの値があるときは、kanban.open-empty-lanes のとおり空のレーンで開く)。外からノートを足すと `B` で開く。

### 行が1つも無くても、カンバンの列に決まりの値(cell-edit.rules-values)があるときは、その値のレーンを札 0 枚で並べてカンバンを開くべき
`req~kanban.open-empty-lanes~2` P1
- Rationale: 見本なしで作った台帳でも、設定で決めた状態の並びを最初から見られるようにするため
- Acceptance:
  - Given: ノートが1件も無い表で `[rules.status] values = ["todo", "doing", "done"]`、`kanban = { column = "status" }`
  - When:
  - Then: `B` でカンバンが開き、レーン todo (0)・doing (0)・done (0) がこの順で並ぶ。札を動かすキーを押しても何もためない。
- Acceptance:
  - Given: 決まりの値の無い列で行が1つも無い
  - When:
  - Then: kanban.config-empty-table のとおり開かずに理由1行。

### `B`(か画面の型のメニューの「カンバン」)で、今のビューの行(絞り込み・並べ替えのあと。tree.screen と同じ)を、列の値ごとのレーン(縦の列)に札として並べて見せるべき
`req~kanban.open-lanes~1` P1
- Acceptance:
  - Given:
  - When: status が todo 2件・doing 1件のビューで `B`
  - Then: todo のレーンに札2枚、doing のレーンに札1枚。絞り込みで外した行は札にならない。

### カンバンのレーンの順は、列の決まり(cell-edit.rules の `values`)があればその順、続いて決まりに無い値を行に現れた順、最後に値の無い行の「(なし)」にするべき
`req~kanban.open-lane-order~2` P1
- Acceptance:
  - Given: 決まり `values = ["todo", "doing", "waiting", "done"]` の status で todo 2件・doing 1件・done 1件・状態なし1件
  - When:
  - Then: レーンは todo(2)・doing(1)・waiting(0)・done(1)・(なし)(1) の順(行の無い waiting もレーンにする)。
- Acceptance:
  - Given: 決まりに無い値 blocked の行がある
  - When:
  - Then: done と (なし) の間に blocked のレーン。
- Acceptance:
  - Given: 状態なしの行が無い
  - When:
  - Then: (なし) のレーンは無い。

### カンバンのレーンの見出しには、値と札の数を出すべき
`req~kanban.open-lane-heading~1` P1
- Acceptance:
  - Given: status が todo の行が2件
  - When:
  - Then: todo のレーンの見出しに、値 todo と札の数 2 が出る。

### カンバンの札の頭の行は、行の名前(表のノートの欄と同じ見せ方)にするべき
`req~kanban.open-card-label~2` P1
- Rationale: 札に値を添えたり長い名前を折り返したりしても、どの行の札かを頭の行で見分けられるようにするため
- Acceptance:
  - Given: 表のノートの欄に `会議` と出る行で、印の `kanban` に `card` を書いていない
  - When: `B`
  - Then: 札は `会議` の1行。

### カンバンのレーンの中の札は、表の行の順に並べるべき
`req~kanban.open-card-order~1` P1
- Acceptance:
  - Given: 表で上から alpha・beta の順に並ぶ todo の2件
  - When:
  - Then: todo のレーンの札も alpha・beta の順。

### カンバンのレーンが幅に収まらないときは、選んでいるレーンが見える所まで横にずらすべき
`req~kanban.open-scroll~1` P1
- Acceptance:
  - Given: 幅 40 で5つのレーン
  - When:
  - Then: 選んでいるレーンが見える。

### カンバンのレーンが幅に収まらないときは、隠れたレーンがあることを示すべき
`req~kanban.open-hidden-mark~1` P1
- Acceptance:
  - Given: 幅 40 で5つのレーン
  - When:
  - Then: 隠れた側に印。

### 色なしでも、カンバンの選んでいる札とレーン、ためる変更のある札を見分けられるべき(cell-view.not-color-only)
`req~kanban.open-no-color~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `--no-color` でも選んでいる札に `>`。

### カンバンの画面では、`j` `k` でレーンの中の札を選び、`h` `l` でレーンを移るべき
`req~kanban.keys-move~1` P1
- Acceptance:
  - Given:
  - When: `B` → `l`
  - Then: `j` で、doing のレーンの2件目の札が選ばれる。

### カンバンの画面では、`Enter` で表に戻って、選んでいる札の行を選ぶべき
`req~kanban.keys-enter~1` P1
- Acceptance:
  - Given:
  - When: `B` → `l` → `j`
  - Then: `Enter` で、表の doing の2件目の行が選ばれる。

### カンバンの画面では、`Esc`・`q`・`B` で表に戻るべき
`req~kanban.keys-back~1` P1
- Acceptance:
  - Given:
  - When: カンバンで `Esc`・`q`・`B` のどれか
  - Then: 表に戻る。

### カンバンの画面では、札のクリックでその札を選ぶべき
`req~kanban.keys-click-select~1` P1
- Acceptance:
  - Given:
  - When: 札のクリック
  - Then: その札を選ぶ。

### カンバンの画面では、選んでいる札をもう一度クリックすると、表のその行へ移るべき
`req~kanban.keys-click-open~1` P1
- Acceptance:
  - Given:
  - When: 選んでいる札をもう一度クリック
  - Then: 表のその行が選ばれる。

### カンバンの画面では、ホイールで札を、横のホイールでレーンを動かすべき
`req~kanban.keys-wheel~2` P1
- Acceptance:
  - Given:
  - When: ホイール
  - Then: レーンの中で選んでいる札が移る。
- Acceptance:
  - Given:
  - When: 横のホイール
  - Then: 選んでいるレーンが移る。

### カンバンの画面では、`H` `L` で選んでいる札を左・右のレーンへ動かし、その行の列の値を、動かした先のレーンの値に書き換えるべき
`req~kanban.move-card~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L`
  - Then: 札が doing のレーンへ移り、その行の status が doing になる。

### カンバンで札を動かした値の書き換えは、表のセルの直しと同じためる変更(未保存)にするべき
`req~kanban.move-pending~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L`
  - Then: 下の帯の未保存が 1。

### カンバンで動かした札は、動かした先のレーンで選んだままにするべき
`req~kanban.move-keep-selection~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L`
  - Then: doing のレーンへ移った札を選んだまま。

### カンバンの画面では、`u` で札の動きを取り消すべき
`req~kanban.move-undo~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L` のあと `u`
  - Then: todo に戻る。

### カンバンの画面では、`U` で取り消した札の動きをやり直すべき
`req~kanban.move-redo~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L`・`u` のあと `U`
  - Then: 札が doing のレーンへ移る。

### カンバンの画面では、`Ctrl+s` で表に戻って保存の確かめ(差分)を開くべき
`req~kanban.move-save~1` P1
- Acceptance:
  - Given:
  - When: todo の札で `L` のあと `Ctrl+s`
  - Then: 表に戻り、保存の確かめ(差分)が開く。

### カンバンの札は、「(なし)」のレーンへは動かさないべき(値を消すのは表で)
`req~kanban.move-not-to-none~1` P1
- Acceptance:
  - Given:
  - When: 右の隣が (なし) の札で `L`
  - Then: 動かず理由1行。

### 読むだけのとき(`--readonly`、読むだけの列や行。cli.options・base-view.unsupported・base-view.types-conflict)は、カンバンの札を動かさずに理由を出すべき
`req~kanban.move-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で `L`
  - Then: 動かず理由1行。

### カンバンを開いているあいだにノートが外で変わったら、組み直して同じ行の札を選び直すべき(札が別のレーンへ移っていればそのレーンで)
`req~kanban.reload~1` P1
- Acceptance:
  - Given:
  - When: 開いたまま外で status を todo から done に書き換える
  - Then: 札が done のレーンに移り、選んだまま。

### 見せ方の一覧(screen.views-page)・できることの一覧(screen.features-page)・キーの文書(screen.keys-docs)・設定の文書にカンバンの項を持ち、組み込みの tasks のスターターに `kanban` の設定を同梱し、場面の台本にカンバンの場面を持つべき
`req~kanban.docs~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: docs/manual/ja・en の views.md に「カンバン」の節(画像 images/kanban.svg)、features.md に札を動かして状態を直す項。starters/tasks の印に `kanban = { column = "status" }`。docs/manual-scenarios.toml に id = "kanban"(読んで判定と)。

### 印の `kanban` に `card = ["<列>", …]` を書いてよく、札の名前の下に、書いた列の値を書いた順に1列1行で出すべき(値の無い列の行は出さない)
`req~kanban.card-columns~1` P1
- Rationale: 札を表に戻らずに読めるようにするため(読書の記録の著者・進み・評価、会議の日時・出席者、定期の支払いの金額・次の支払い日)
- Needs: utest・stest
- Acceptance:
  - Given: `kanban = { column = "status", card = ["author", "rating"] }` の表で、alpha の author が `yamada`・rating が `4`
  - When: `B`
  - Then: alpha の札は `alpha`・`yamada`・`4` の3行がこの順に上から並ぶ。
- Acceptance:
  - Given: 同じ表で、beta の author が `sato` で rating のキーが無い
  - When: `B`
  - Then: beta の札は `beta`・`sato` の2行(rating の行は無い)。
- Acceptance:
  - Given: `kanban = { column = "status" }`(`card` を書かない)
  - When: `B`
  - Then: 札は名前の行だけで、値の行は無い。

### カンバンの札の値は、表のセルと同じ見せ方(決まりの名前とアイコン・設定の日付の形・リンクの行き先の名前・計算の列の値)にするべき
`req~kanban.card-values-shown~1` P1
- Rationale: 同じ値が表と札で違って見えないようにするため
- Needs: utest
- Acceptance:
  - Given: `[rules.kind] values = ["book"]`・`labels = { book = "本" }`、`card = ["kind"]`、kind が `book` の行
  - When: `B`
  - Then: 札の値の行は `本`(`book` でない)。
- Acceptance:
  - Given: `[dates] format = "YYYY/MM/DD"`、`[types] due = "date"`、`card = ["due"]`、due が `2026-12-01` の行
  - When: `B`
  - Then: 札の値の行は `2026/12/01`。
- Acceptance:
  - Given: `card = ["owner"]`、owner が `"[[hanako]]"` の行
  - When: `B`
  - Then: 札の値の行は `hanako`(`[[` `]]` を見せない)。
- Acceptance:
  - Given: 印の式 `twice = 'n * 2'`、`card = ["formula.twice"]`、n が `21` の行
  - When: `B`
  - Then: 札の値の行は `42`。

### 印の `kanban` の `card` が列の名前の文字列の並びでないときは、cli.config-file の形で警告して `card` を外し、札を名前だけにしてカンバンは出すべき
`req~kanban.card-config-bad~1` P1
- Rationale: 書き方を間違えても、カンバン自体は使えるようにするため
- Needs: utest
- Acceptance:
  - Given: `kanban = { column = "status", card = "author" }`
  - When: 開いて `B`
  - Then: `table[1].kanban.card` を指す警告1行が出て、カンバンが開き、札は名前の行だけ。
- Acceptance:
  - Given: `card = ["author", 3]`
  - When: 開く
  - Then: `table[1].kanban.card` を指す警告1行。

### 印の `kanban` の `card` の列が表に無いときは、ノートを読み終えたときに cli.config-file の形で警告し、その列の行を札に出さないべき
`req~kanban.card-missing-column~1` P1
- Rationale: 列の名前の書き違いに気づけるようにするため(計算の列 `formula.*`・ファイルの列 `file.*` は表にあると数える)
- Needs: utest
- Acceptance:
  - Given: `card = ["nosuch", "author"]` で、どのノートにも nosuch のキーが無く、設定の `[types]`・`[rules]` にも無い
  - When: 開いて読み終える
  - Then: `table[1].kanban.card` と `nosuch` を含む警告1行。`B` で開いた札は名前と author の行。
- Acceptance:
  - Given: `card = ["author"]` で、author を持つノートがある
  - When: 開いて読み終える
  - Then: 警告は出ない。

### カンバンの札の名前がレーンの幅に入らないときは、2行まで折り返し、それでも入らなければ2行目の末尾を `…` で切るべき
`req~kanban.card-name-wrap~1` P1
- Rationale: 長い名前(調べ物の問いなど)の札が頭の数文字で切れて読めないのを防ぐため
- Needs: utest・stest
- Acceptance:
  - Given: レーンの札の名前の欄の幅より長く、2行には入る名前の行
  - When: `B`
  - Then: 札の1行目と2行目に名前が続けて出て、つなぐと名前の全部になる(`…` は無い)。
- Acceptance:
  - Given: 2行にも入らない名前の行
  - When: `B`
  - Then: 札の名前は2行で、2行目の末尾が `…`。3行目に名前の続きは無い。
- Acceptance:
  - Given: 幅に入る名前の行
  - When: `B`
  - Then: 札の名前は1行。

### カンバンの札が2行以上になるときは、2行目から(名前の続きと値の行)を、名前の頭より2字下げるべき
`req~kanban.card-indent~1` P1
- Rationale: 色が無くても、札の続きの行と次の札の頭の行を見分けられるようにするため(cell-view.not-color-only)
- Needs: utest
- Acceptance:
  - Given: `card = ["author"]` の表の札
  - When: `B`
  - Then: author の値の行の最初の字は、名前の最初の字より2桁右。
- Acceptance:
  - Given: 名前が2行に折り返す札
  - When: `B`
  - Then: 2行目の最初の字は、1行目の名前の最初の字より2桁右。

### カンバンの札と札の間は、印の `kanban` に `card` を書いたときだけ1行空けるべき
`req~kanban.card-gap~1` P1
- Rationale: 値の行のある札どうしを見分けやすくし、`card` を書かない表は前の札の並びのままにするため
- Needs: utest
- Acceptance:
  - Given: `card = ["author"]` の表で、同じレーンに alpha・beta の札
  - When: `B`
  - Then: alpha の札の最後の行と beta の札の名前の行の間に、何も無い行が1行。
- Acceptance:
  - Given: `card` を書かない表で、同じレーンに名前の短い alpha・beta の札
  - When: `B`
  - Then: alpha と beta の札は続く2行(間の空きの行は無い)。

### 高さの違う札が並ぶレーンでも、選んでいる札の全部の行が見えるように、レーンを縦にずらすべき
`req~kanban.card-scroll~1` P1
- Rationale: 札の高さが値の数と名前の長さで変わっても、選んだ札を読めるようにするため
- Needs: utest・stest
- Acceptance:
  - Given: `card` に2列を書き、1つのレーンに 3行の札が 12枚、体の高さが札 5枚ほどの画面
  - When: `B` のあと `j` を 10回
  - Then: 11枚目の札の名前と値の2行が全部見え、選んでいる印 `>` はその札にある。
- Acceptance:
  - Given: 同じ画面で 11枚目を選んでいる
  - When: `k` を 10回
  - Then: 1枚目の札の全部の行が見え、選んでいる印はその札にある。

### カンバンの札の2行目から(名前の続き・値の行)のクリックでも、その札を選ぶべき(札の間の空きの行のクリックでは何も選ばない)
`req~kanban.card-click~1` P1
- Rationale: 高さのある札のどこを押しても、その札を選べるようにするため
- Needs: utest・stest
- Acceptance:
  - Given: `card = ["author"]` の表で、alpha を選んでいて、同じレーンに beta の札
  - When: beta の札の author の値の行をクリック
  - Then: beta の札を選ぶ(カンバンのまま)。
- Acceptance:
  - Given: 同じ画面
  - When: alpha の札と beta の札の間の空きの行をクリック
  - Then: 選んでいる札は変わらない。

## 未確定事項

### 札の名前の折り返しを、`card` を書かない札にも効かせるか
`tbd~kanban.card-name-wrap-always`
- Decider: 人
- Provisional: 効かせる。幅に入る名前の札は前と同じ1行で、札の間も空けない(kanban.card-gap)。調べ物の問いのような長い名前を、`card` を書かずに読めるようにするため。
- Basis: 2026-11-25 の台帳の確かめで、調べ物の問いの札が切れて読めなかった。「何も書かなければ今の札のまま」とも決めたので、`card` を書かないときに名前を折り返さない読みもある。

### 札の値の並べ方(列の名前を添えるか・幅に入るなら1行に並べるか)
`tbd~kanban.card-value-layout`
- Decider: 人
- Provisional: 1列1行で、値だけを出す(列の名前は添えない)。値の無い列の行は出さない。表の道具の札の多くと同じ形で、狭いレーンでも値の幅を取れるため。
- Basis: 2026-11-25 の決め: 「列ごとに1行(または幅に収まるなら並べて)」。並べる形は選ばなかった。

### 札に出す列を、カンバンの画面かビューの設定(`o`)から選べるようにするか
`tbd~kanban.card-picker`
- Decider: 人
- Provisional: この版では入れず、印の `card` に書く。画面で選んでも残す所が決まらない(札の列は表ごとの印の設定で、画面から印を書き換える口はページの保存だけ。グラフの画面の切り替えのように印を書き換えない形にすると、開き直すと消える)。
- Basis: 2026-11-25 の決め: 「入れる価値を考え、入れるなら同じ PR で」。
