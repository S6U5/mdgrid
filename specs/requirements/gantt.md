# ガント(時間軸の横棒の画面) の要求

## 目的

表(と木・関係マップ)に続く画面の型。今のビューの行を、始まりと終わりの日付の列から時間軸の横棒にして見せる。親子(navigation.parent-child)と WBS が効いていれば、その並び・字下げ・番号・進み具合をそのまま使い、親の棒は子の範囲になる。設定は印の `[[table]]` の `gantt = { … }` で、無ければ日付の列の名前から推定する(gantt.screen-config の一般の形)。読むだけの画面で、日付は表で直す。扱わない: 棒のドラッグでの編集、依存の矢印、クリティカルパス、履歴(バーンダウン)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/gantt/spec.md](../_history/decidespec/gantt/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 履歴を要する図(バーンダウン)。
- 時間の単位は日・週・月。時刻の単位(時間)は扱わない。

## 次の段(まだ作っていない。P2)

- カレンダー(日付の列を月のます目に)・マトリクス(2つの列で4象限)。どれも gantt.screen-config の形(`[[table]]` の型の名前のキー、無ければ推定)で足す(カンバンは `kanban`、集計のグラフは `chart` に移した)。
- 依存の矢印(`depends_on` の列)とクリティカルパス。棒の端のドラッグで日付を直す。

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

### ガントの親の棒の塗りの長さは、WBS の進み具合の割合にするべき
`req~gantt.tree-wbs-progress-fill~1` P1
- Acceptance:
  - Given: WBS で子の割合が 100 と 0
  - When:
  - Then: 親の棒の塗りは半分。

### ガントの画面では、`Z` で親の行を畳む・開く
`req~gantt.tree-wbs-fold~2` P1
- Acceptance:
  - Given:
  - When: `Z` で親を畳む
  - Then: 子の行が消え
- Acceptance:
  - Given:
  - When: もう一度 `Z`
  - Then: 子の行が戻る。

### ガントの画面では、`j` `k` で行を選ぶべき
`req~gantt.keys-select~1` P1
- Acceptance:
  - Given:
  - When: `C` → `j` `j` → 3行目が選ばれ、`k`
  - Then: 2行目が選ばれる。

### ガントの画面では、`h` `l` で時間軸を1単位動かすべき
`req~gantt.keys-pan~2` P1
- Acceptance:
  - Given:
  - When: `l` を3回
  - Then: 時間軸が3単位進み
- Acceptance:
  - Given:
  - When: `h` を1回
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

### ガントの画面では値を直せないべき(棒の端のドラッグや日付の入力はしない。日付は表で直す)
`req~gantt.read-only-no-edit~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: ガントで `Enter` 以外の編集のキーを押しても何も変わらない。

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
