# ガント(時間軸の横棒の画面) の要求

## 目的

表(と木・関係マップ)に続く画面の型。今のビューの行を、始まりと終わりの日付の列から時間軸の横棒にして見せる。親子(navigation.parent-child)と WBS が効いていれば、その並び・字下げ・番号・進み具合をそのまま使い、親の棒は子の範囲になる。設定は印の `[[table]]` の `gantt = { … }` で、無ければ日付の列の名前から推定する(gantt.screen-config の一般の形)。読むだけの画面で、日付は表で直す。扱わない: 棒のドラッグでの編集、依存の矢印、クリティカルパス、履歴(バーンダウン)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/gantt/spec.md](../_history/decidespec/gantt/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 履歴を要する図(バーンダウン)。
- 時間の単位は日・週・月。時刻の単位(時間)は扱わない。

## 次の段(まだ作っていない。P2)

- カレンダー(日付の列を月のます目に)・マトリクス(2つの列で4象限)。どれも gantt.screen-config の形(`[[table]]` の型の名前のキー、無ければ推定)で足す(カンバンは `kanban`、集計のグラフは `chart` に移した)。
- 依存の矢印(`depends_on` の列)とクリティカルパス。棒の端のドラッグで日付を直す(キーで動かす・伸ばすのは gantt.edit-move・gantt.edit-resize)。期間を始まりと終わりの日付で打って決めること。

## 前提(旧い仕様で AI が置いた仮定)

- 行の並びと親子・WBS は、表(ビューの設定)が作る行の木をそのまま使う(ガントで独自の並びを持たない)。
- 推定の列の名前の一覧は固定(gantt.config)。外れたら印で上書きする。

## 要求

### ワークスペースの印の `[[table]]` に `gantt = { start = "<列>", end = "<列>" }` を書いてよく、`start` と `end` の列の日付をその行の棒の始まりと終わりとみなすべき
`req~gantt.config-start-end~1` P1
- Acceptance:
  - Given: `gantt = { start = "start", end = "due" }` の表
  - When:
  - Then: メニューに「ガント」が出て `C` で開き、start 2026-10-01・due 2026-10-07 の行の棒は 10/01〜10/07。

### ガントの設定では、`end` の代わりに `days = "<列>"`(始まりからの日数)を書いてもよい
`req~gantt.config-days~1` P1
- Acceptance:
  - Given: `gantt = { start = "start", days = "days" }` の表で start 2026-10-01・days 3 の行
  - When:
  - Then: 日の単位で3マスの棒(10/01〜10/03)。

### ガントの設定が無いときは、始まりと終わりの列を日付の列の名前から推定してよい
`req~gantt.config-infer~1` P1
- Acceptance:
  - Given: 設定なしで start と due の日付の列がある
  - When:
  - Then: 推定して開き、画面の題に推定した列を示す(推定する名前は、始まりが start・begin・from・開始、終わりが end・due・finish・until・to・期限・終了で、大文字小文字を問わない)。

### ガントの設定が無く、推定でも列が決まらないときは、理由を出してガントを出さないべき
`req~gantt.config-infer-fail~1` P1
- Acceptance:
  - Given: 設定なしで日付の列が無い
  - When:
  - Then: 理由1行で開かない。

### ガントの設定の `start`・`end`・`days` の列が表に無いときは、cli.config-file の形で警告してガントを出さないべき
`req~gantt.config-missing-column~1` P1
- Acceptance:
  - Given: `start = "none"`
  - When:
  - Then: 警告1行で、メニューに出ない。

### 設定の `[types]`・`[rules.<列>]` に書いた列は、その列を持つノートが1件も無くても、ガントの設定の列として表にあると数えるべき
`req~gantt.config-declared-column~2` P1
- Rationale: スターターから作った台帳で、開始や期限をまだ書いていないノートだけのときに、設定で決めた列が無いと言わないようにするため
- Acceptance:
  - Given: `[types] start = "date"`・`due = "date"`、`gantt = { start = "start", end = "due" }` の表に、start も due も持たないノートが1件
  - When:
  - Then: 警告せず、`C` でガントが開く。
- Acceptance:
  - Given:
  - When: `[types]` にも `[rules]` にもノートにも無い列 `begin` を `start` に書く
  - Then: gantt.config-missing-column のとおり警告1行で開かない。

### 行が1つも無い表(見本なしで作った台帳など)では、ガントの設定の列があるかを決めず、警告しないべき
`req~gantt.config-empty-table~1` P1
- Rationale: 表の列はノートのキーから作るので、行が無いと設定の列があるかを決められないため
- Acceptance:
  - Given: 行が1つも無い表
  - When:
  - Then: 読み終えても警告せず、画面の型のメニューに出て、開くと出す行が無いと理由1行。外からノートを足すと `C` で開く。

### `C`(か画面の型のメニューの「ガント」)で、今のビューの行(絞り込み・並べ替えのあと)を、左に行の名前、右に時間軸と横棒で見せるべき
`req~gantt.open-bars~1` P1
- Acceptance:
  - Given:
  - When: start 2026-10-01・due 2026-10-07 の行で `C`
  - Then: 左に行の名前、右に日の単位で7マスの棒。

### ガントの時間軸は日・週・月の単位を持ち、開くときは行の範囲が幅に収まる最も細かい単位にするべき
`req~gantt.open-unit~1` P1
- Acceptance:
  - Given: 幅 100 で範囲 400 日
  - When:
  - Then: 週か月の単位で開く。

### ガントは今日の列に印を付けるべき
`req~gantt.open-today~1` P1
- Acceptance:
  - Given: MDGRID_TODAY=2026-10-06
  - When:
  - Then: 10/06 の列に今日の印。

### ガントでは、始まりか終わりの片方しか無い行は、その日に印(◆)を置くべき
`req~gantt.open-milestone~1` P1
- Acceptance:
  - Given: due だけの行
  - When:
  - Then: その due の日に ◆が1つ。

### ガントでは、始まりも終わりも無い行は、棒なしで並べるべき
`req~gantt.open-no-dates~1` P1
- Acceptance:
  - Given: start も due も無い行
  - When:
  - Then: ガントにその行の名前が出て、棒も ◆ も無い。

### ガントは、色なしでも棒・印・今日・選んでいる行を見分けられるべき(cell-view.not-color-only)
`req~gantt.open-no-color~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `--no-color` でも棒と印が文字で出る。

### ビューの設定の親子(navigation.parent-child)が効いているときは、ガントの行も表と同じ並び・字下げ・畳みにするべき
`req~gantt.tree-wbs-order~2` P1
- Acceptance:
  - Given: parent で子2件の親
  - When:
  - Then: ガントでも表と同じく親の下に子2件が字下げして並ぶ。
- Acceptance:
  - Given:
  - When: 表で親を畳んでから `C`
  - Then: ガントでも子の行は出ない。

### WBS が効いていれば、ガントの行も表と同じ番号と進み具合を使うべき
`req~gantt.tree-wbs-number~1` P1
- Acceptance:
  - Given:
  - When: WBS の効いた表で `C`
  - Then: ガントの各行の番号と進み具合が、表のその行の番号と進み具合と同じ。

### ガントの親の棒は、親に自分の日付があればそれ、無ければ子孫の始まりの最小から終わりの最大までにするべき
`req~gantt.tree-wbs-parent-range~1` P1
- Acceptance:
  - Given: parent で子2件(10/01〜10/03、10/05〜10/07)、親に日付なし
  - When:
  - Then: 親の棒は 10/01〜10/07。

### `children` の表(navigation.parent-child-config)で、自分の日付を持つ親の期間から子孫の棒がはみ出したら、`children` の `gantt`(書かなければ `"mark"`)が `"mark"` なら、はみ出した子と親の行に色に頼らない印を付け、選んだ行の下の帯にはみ出した日数を出し、`"none"` なら何もしないべき
`req~gantt.children-span-mark~1` P1
- Rationale: 親の期限のあとまで子が続く計画の崩れに、ガントの画面で気づくため。期間をどう扱うかは設定で選べるようにする(2026-11-24 に人が「ガントチャートの期間に関してもどうするかは考えて」と言った。既定は tbd~gantt.children-span-default)
- Acceptance:
  - Given: 親が 2026-11-01〜11-10、子が 11-05〜11-12
  - When: ガントで開く
  - Then: 子と親の行に印が付き、子を選ぶと下の帯に「親の期間から 2 日はみ出す」旨の1行が出る。
- Acceptance:
  - Given: 同じ表で、子が 11-05〜11-10
  - When: ガントで開く
  - Then: どの行にも印が無い。
- Acceptance:
  - Given: `children = { key = "parent", gantt = "none" }` で、子が 11-05〜11-12
  - When: ガントで開く
  - Then: どの行にも印が無い。
- Needs: utest

### `children` の `gantt = "extend"` では、子の棒を動かす・伸ばす(gantt.edit-move・gantt.edit-resize)と親の期間からはみ出すとき、親の始まりか終わりを子孫の範囲まで広げる変更を、子の変更と同じ1手としてためるべき
`req~gantt.children-span-extend~1` P1
- Rationale: 子の予定を延ばしたら、親の期間も合わせて延ばす使い方のため(要約のタスクの期間を子に合わせる、計画の道具の慣習)。親の棒を動かしたときに子も動かすかは tbd~gantt.children-move
- Acceptance:
  - Given: `gantt = "extend"`、親が 11-01〜11-10、子が 11-05〜11-10、日の単位
  - When: 子を選んで `>` を2回
  - Then: 子の終わりが 11-12、親の終わりも 11-12 のためる変更になる。
- Acceptance:
  - Given: その変更のあと
  - When: `u` を2回
  - Then: 子と親の終わりが 11-10 に戻る。
- Acceptance:
  - Given: 同じ設定で、自分の日付の無い親(棒は子孫の範囲。gantt.tree-wbs-parent-range)
  - When: 子を `>` で伸ばす
  - Then: 親には何もためない。
- Needs: utest

### ガントの親の棒の塗りの長さは、WBS の進み具合の割合にするべき
`req~gantt.tree-wbs-progress-fill~1` P1
- Acceptance:
  - Given: WBS で子の割合が 100 と 0
  - When:
  - Then: 親の棒の塗りは半分。

### ガントの画面では、`Z` で親の行を畳む・開く
`req~gantt.tree-wbs-fold~1` P1
- Acceptance:
  - Given:
  - When: `Z` で親を畳む → 子の行が消え、もう一度 `Z`
  - Then: 子の行が戻る。

### ガントの画面では、`j` `k` で行を選ぶべき
`req~gantt.keys-select~1` P1
- Acceptance:
  - Given:
  - When: `C` → `j` `j` → 3行目が選ばれ、`k`
  - Then: 2行目が選ばれる。

### ガントの画面では、`h` `l` で時間軸を1単位動かすべき
`req~gantt.keys-pan~1` P1
- Acceptance:
  - Given:
  - When: `l` を3回 → 時間軸が3単位進み、`h` を1回
  - Then: 1単位戻る。

### ガントの画面では、`+` `-` で時間軸の単位を細かく・粗くするべき
`req~gantt.keys-zoom~2` P1
- Acceptance:
  - Given:
  - When: `-`
  - Then: 日から週へ。
- Acceptance:
  - Given:
  - When: `+`
  - Then: 週から日へ。

### ガントの画面では、`t` で今日が見える位置に戻すべき
`req~gantt.keys-today~1` P1
- Acceptance:
  - Given:
  - When: MDGRID_TODAY=2026-10-06 で `l` を押して 10/06 が時間軸の外に出てから `t`
  - Then: 10/06 の今日の印が見える。

### ガントの画面では、`Enter` で表に戻ってその行を選ぶべき
`req~gantt.keys-enter~1` P1
- Acceptance:
  - Given:
  - When: `C` → `j` `j`
  - Then: `Enter` で表の3行目が選ばれる。

### ガントの画面では、`Esc`・`q`・`C` で表に戻るべき
`req~gantt.keys-back~1` P1
- Acceptance:
  - Given:
  - When: ガントで `Esc`
  - Then: 表に戻る。`q`・`C` でも同じ。

### ガントの画面では、行のクリックでその行を選ぶべき
`req~gantt.keys-mouse-click~1` P1
- Acceptance:
  - Given:
  - When: 1行目を選んだガントで3行目をクリック
  - Then: 3行目が選ばれる。

### ガントの画面では、選んでいる行をもう一度クリックすると、表のその行へ移るべき
`req~gantt.keys-mouse-click-again~1` P1
- Acceptance:
  - Given:
  - When: ガントで3行目をクリックして選び、もう一度クリック
  - Then: 表に戻り、表の3行目が選ばれる。

### ガントの画面では、ホイールで行を動かすべき
`req~gantt.keys-mouse-wheel~1` P1
- Acceptance:
  - Given:
  - When: ガントでホイールを下へ回す
  - Then: 選んでいる行が下の行へ動く。

### ガントの画面では、横のホイールで時間軸を動かすべき
`req~gantt.keys-mouse-hwheel~1` P1
- Acceptance:
  - Given:
  - When: ガントで横のホイールを右へ回す
  - Then: 時間軸が進む。

### ガントの窓の下の縁に操作の帯を出し、時間軸を動かす `◀`・`▶`、単位の `日`・`週`・`月`(今の単位は `[週]` のように囲む)、期間の `今日`・`今週`・`今月`・`四半期`・`全体` をこの順に並べるべき
`req~gantt.controls-bar~1` P1
- Rationale: キーを知らなくても、何ができるかが画面から分かるようにするため(2026-11-24 に人が「期間指定とか操作 UI を付ける」と決めた)。下の縁に置くので、行と時間軸の位置は変わらない
- Acceptance:
  - Given: 行の範囲が 2026-10-01〜2026-10-07 の表(幅 120)
  - When: `C` でガントを開く
  - Then: 窓の下の縁の行に `◀`・`▶`・`[日]`・`週`・`月`・`今日`・`今週`・`今月`・`四半期`・`全体` がこの順に並ぶ。
- Acceptance:
  - Given: 同じガント
  - When: `-`
  - Then: 下の縁の単位は `日`・`[週]`・`月`。
- Needs: utest

### ガントの操作の帯の項目をクリックしたら、その項目の動作をするべき(`◀`・`▶` は `h`・`l`、単位はその単位にする、`今日` は `t`、期間は gantt.range-presets)
`req~gantt.controls-click~1` P1
- Acceptance:
  - Given: 日の単位で開いたガント
  - When: 操作の帯の `週` をクリック
  - Then: 単位が週になり、帯は `[週]`。
- Acceptance:
  - Given: 左端が 2026-10-01 の日の単位のガント
  - When: 操作の帯の `▶` をクリック
  - Then: 左端が 2026-10-02。
- Acceptance:
  - Given: 今日が 2026-10-06 のガント
  - When: 操作の帯の `今月` をクリック
  - Then: 左端が 2026-10-01。
- Needs: utest

### ガントの期間は、`今週`(今日を含む週の月曜を左端にした日の単位)・`今月`(今月の1日を左端に、月の末日までが幅に収まる最も細かい単位)・`四半期`(今日を含む四半期の最初の日(1・4・7・10月の1日)を左端に、四半期の末日までが幅に収まる最も細かい単位)・`全体`(開いたときと同じ。行の範囲の最初の日を左端に、最後の日までが収まる最も細かい単位)から選べるべき
`req~gantt.range-presets~1` P1
- Rationale: 見たい期間に、1単位ずつ動かさずに1手で合わせるため。期間を選んでも行は隠さない(棒が期間の外の行も並べたままにする。絞り込みと区別するため)
- Acceptance:
  - Given: 今日が 2026-10-06(火)のガント
  - When: 期間を `今週` にする
  - Then: 単位は日で、左端は 2026-10-05(月)。
- Acceptance:
  - Given: 今日が 2026-10-06 で、時間軸が 60 字より広いガント
  - When: 期間を `今月` にする
  - Then: 単位は日で、左端は 2026-10-01。
- Acceptance:
  - Given: 今日が 2026-11-20 で、時間軸が 60 字より広く 92 字より狭いガント
  - When: 期間を `四半期` にする
  - Then: 単位は週で、左端は 2026-10-01 を含む週の月曜(2026-09-28)。
- Acceptance:
  - Given: 行の範囲が 2026-10-01〜2026-10-07 のガントで、`-` で単位を週にし、`l` を5回押した
  - When: 期間を `全体` にする
  - Then: 単位は日で、左端は 2026-10-01。
- Acceptance:
  - Given: 行の範囲が 2026-10-01〜2026-10-07 の4行のガント
  - When: 期間を `今週` にする(10/03 だけの行は期間の外)
  - Then: 行は4行のまま。
- Needs: utest

### ガントの画面では、`r` で期間を `今週` → `今月` → `四半期` → `全体` の順に切り替えるべき
`req~gantt.range-key~1` P1
- Acceptance:
  - Given: 今日が 2026-10-06 で開いたガント
  - When: `r`
  - Then: 左端は 2026-10-05(今週)。
- Acceptance:
  - Given: 同じガントで `r` を1回押した
  - When: `r`
  - Then: 左端は 2026-10-01(今月)。
- Acceptance:
  - Given: 同じガントで `r` を4回押した(全体)
  - When: `r`
  - Then: 左端は 2026-10-05(今週に戻る)。
- Needs: utest

### ガントの画面では、`H`・`L` で、選んでいる行の棒を1単位(日の単位は1日、週は7日、月は1か月。月の末日を越える日は末日に寄せる)前・後ろへ動かし、始まりと終わりの列の値を、ためる変更(write-back.pending)として書くべき
`req~gantt.edit-move~1` P1
- Rationale: 予定をずらすたびに表へ戻って日付を2つ打ち直さずに済むようにするため(2026-11-24 に人が決めた。棒の端のドラッグは次の段)。ためる変更なので、保存の前に差分で確かめられ、取り消せる
- Acceptance:
  - Given: start が 2026-10-01、due が 2026-10-07 の行を選んだ日の単位のガント
  - When: `L`
  - Then: ためた値は start = 2026-10-02、due = 2026-10-08。ノートのファイルは変わらない。
- Acceptance:
  - Given: 同じ行を選んだ週の単位のガント
  - When: `H`
  - Then: ためた値は start = 2026-09-24、due = 2026-09-30。
- Acceptance:
  - Given: start が 2026-01-31、due が 2026-01-31 の行を選んだ月の単位のガント
  - When: `L`
  - Then: ためた値は start = 2026-02-28、due = 2026-02-28。
- Acceptance:
  - Given: due だけが 2026-10-03 の行(印 ◆)を選んだ日の単位のガント
  - When: `L`
  - Then: ためた値は due = 2026-10-04 だけで、start は書かない。
- Acceptance:
  - Given: start が `2026-10-01T09:30` の日時の行を選んだ日の単位のガント
  - When: `L`
  - Then: ためた値は start = `2026-10-02T09:30`(時刻は残す)。
- Acceptance:
  - Given: `gantt = { start = "start", days = "days" }` で start が 2026-10-01、days が 3 の行
  - When: `L`
  - Then: ためた値は start = 2026-10-02 だけで、days は書かない。
- Needs: utest

### ガントの画面では、`>`・`<` で、選んでいる行の棒の終わりを1単位伸ばす・縮めるべき(終わりが始まりより前になる縮め方はせず、理由を出す。`days` の設定では日数の列を書く)
`req~gantt.edit-resize~1` P1
- Acceptance:
  - Given: start が 2026-10-01、due が 2026-10-07 の行を選んだ日の単位のガント
  - When: `>`
  - Then: ためた値は due = 2026-10-08 だけ。
- Acceptance:
  - Given: 同じ行
  - When: `<`
  - Then: ためた値は due = 2026-10-06 だけ。
- Acceptance:
  - Given: start も due も 2026-10-02 の行
  - When: `<`
  - Then: 何もためず、これ以上縮められない理由が出る。
- Acceptance:
  - Given: `gantt = { start = "start", days = "days" }` で start が 2026-10-01、days が 3 の行を選んだ日の単位のガント
  - When: `>`
  - Then: ためた値は days = 4 だけ。
- Needs: utest

### ガントの画面で、動かせない棒(自分の日付が無い行(子孫の範囲を見せる親・日付の無い行)、読むだけの起動、読むだけの列)は動かさず、理由を出すべき
`req~gantt.edit-refuse~1` P1
- Acceptance:
  - Given: 自分の日付が無く、子の範囲(10/01〜10/07)が棒になっている親の行を選んだガント
  - When: `L`
  - Then: 何もためず、この行に日付が無いという理由が出る。
- Acceptance:
  - Given: `--readonly` で開いたガント
  - When: `L`
  - Then: 何もためず、読むだけの理由が出る。
- Acceptance:
  - Given: `gantt = { start = "start", end = "file.mtime" }`(終わりが読むだけのファイルの列)で、start が 2026-10-01 の行を選んだガント
  - When: `>`
  - Then: 何もためず、その列を直せない理由が出る。
- Needs: utest

### ガントの画面では、`u` で取り消し、`U` でやり直しができ、Ctrl+S で表と同じ保存の確認(write-back)へ進むべき
`req~gantt.edit-undo~1` P1
- Acceptance:
  - Given: start が 2026-10-01、due が 2026-10-07 の行で `L` を押したガント
  - When: `u`
  - Then: ためた変更が無くなり、ガントのまま。
- Acceptance:
  - Given: その取り消しのあと
  - When: `U`
  - Then: ためた値は start = 2026-10-02、due = 2026-10-08 に戻る。
- Acceptance:
  - Given: `L` を押して変更をためたガント
  - When: Ctrl+S
  - Then: 保存の確認の画面が出る。
- Needs: utest

### ガントの画面の下の帯には、棒を動かすキー(`H`・`L`)と伸ばす・縮めるキー(`>`・`<`)を、動かせる棒を選んでいるときに出すべき
`req~gantt.band-edit-keys~1` P1
- Rationale: 棒を動かせるのに、下の帯にそのキーが出ず、キーを知らない人には動かせることが分からなかったため(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: 自分の日付のある行を選んだガント
  - When: 下の帯を見る
  - Then: `H`・`L` と `>`・`<` の説明がある。
- Acceptance:
  - Given: 読むだけの起動のガント
  - When: 下の帯を見る
  - Then: 棒を動かすキーは出ない。
- Needs: utest

### ガントの画面では、棒を動かす・伸ばす・縮めるキー(gantt.edit-move・gantt.edit-resize)のほかでは値を直せないべき(日付を打つ入力と、棒の端のドラッグはしない)
`req~gantt.read-only-no-edit~2` P1
- Acceptance:
  - Given:
  - When:
  - Then: ガントで編集のキー(`e`・Space・Backspace・Delete・文字)を押しても何も変わらない。

### ガントを開いているあいだにノートが外で変わったら、組み直して同じ行を選び直すべき
`req~gantt.read-only-external~1` P1
- Acceptance:
  - Given:
  - When: 開いたまま外でノートの due を変える
  - Then: 棒が変わり、選んでいた行は同じ。

### 見せ方の一覧(screen.views-page)・できることの一覧(screen.features-page)・キーの文書(screen.keys-docs)・設定の文書にガントの項を持ち、組み込みの tasks のスターターに `start` の列と `gantt` の設定を同梱し、場面の台本にガントの場面を持つべき
`req~gantt.docs~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: docs/manual/ja・en の views.md に「ガント」の節(画像 images/gantt.svg)。starters/tasks の印に `gantt = { start = "start", end = "due" }`、見本のノートに start。docs/manual-scenarios.toml に id = "gantt"(読んで判定と)。

### 画面の型ごとの設定は、`[[table]]` に型の名前のキー(`tree`・`gantt`・以後の `kanban` など)で同じ形(列の名前を指す表)で書き、設定が無いときの推定の規則を型ごとに持ち、画面の型のメニューには設定か推定で開ける型だけを並べるべき
`req~gantt.screen-config~2` P1
- Rationale: 型が増えても台帳ごとの設定を増やさないため
- Acceptance:
  - Given: 印に `tree` と `gantt` の両方がある表
  - When:
  - Then: メニューに 表・木・ガント。
- Acceptance:
  - Given: どちらも無く日付の列が start・due の表
  - When:
  - Then: 表・ガント(推定)。読んで判定: 設定の読み方が同じ形。

## 未確定事項

### ガントで選んだ期間と単位を、ビューごとに覚えるか
`tbd~gantt.range-remember`
- Decider: 人
- Provisional: 覚えない(開くたびに「全体」から。開いている間だけ効く)。覚えるなら、見た目の状態(screen.view-state)に期間の名前(今週・今月・四半期・全体)で残す

### 子が親の期間からはみ出したときの扱いの既定
`tbd~gantt.children-span-default`
- Decider: 人
- Provisional: `"mark"`(印を付けるだけで、値は書かない)

### 親の棒を動かしたときに子も動かすか
`tbd~gantt.children-move`
- Decider: 人
- Provisional: 動かさない
