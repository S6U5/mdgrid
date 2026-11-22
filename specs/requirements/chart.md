# グラフ(今のビューの行を数えて描く画面) の要求

## 目的

表(と木・関係マップ・ガント・カンバン)に続く画面の型。今のビューの行を、分ける列の値ごとに数えるか、数の列を集めて、棒・積み上げ棒・折れ線・散布図・ヒートマップで見せる。状態ごとの件数、担当ごとの見積もりの合計、月ごとの件数や累積を、表を数えずに一目で見るため。設定は印の `[[table]]` の `chart = { … }`(複数なら配列)で、無ければ列から推定する(gantt.screen-config の一般の形)。種類と列は画面の中のキーでも切り替えられ、項目から `Enter` で表をその値に絞って戻れる。読むだけの画面で、値は表で直す。扱わない: 円グラフ、印への書き戻し、複数のグラフを並べる画面、`.base` の chart のビュー(scope.table-only)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/chart/spec.md](../_history/decidespec/chart/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 円グラフ(端末の文字の格子では形が崩れる。積み上げ棒の割合 `%` で代える)。
- `.base` の chart のビューを開くこと(scope.table-only)。グラフの設定は印の `[[table]]` の `chart` に書く(gantt.screen-config)。
- グラフの画面で値を直すこと。値は表で直す。

## 次の段(まだ作っていない。P2)

- 印に書いたグラフを1画面に並べる(ダッシュボード)。
- 画面で切り替えたグラフを印に書き出す(設定の画面 `o` から選んで保存する形も)。
- 年の単位、面グラフ、箱ひげ図。

## 前提(旧い仕様で AI が置いた仮定)

- 行の並びと絞り込みは、表(ビューの設定)が作る行をそのまま使う(グラフで独自の絞り込みを持たない)。親子(navigation.parent-child)の畳みは見ず、ビューを通った全部の行を数える。
- 数の列の区間の数(10 まで)と、内訳の数(5 まで)は固定。
- 推定の順(chart.open)と、種類を切り替えたときの列の選び方(その種類に合う最初の列)は固定。外れたら印で書くか、キーで切り替える。
- 平均・最小・最大は、数の値のある行だけで出す。数の値のある行が無い項目は値なし(棒を描かず「-」)とする。

## 要求

### ワークスペースの印の `[[table]]` に、グラフの設定 `chart = { … }`(鍵は kind・x・y・agg・by・unit・cumulative・name)を書いてよい
`req~chart.config-one~1` P1
- 受け入れ: `chart = { kind = "bar", x = "status" }` → `P` で status ごとの件数の棒(鍵の形は `chart = { kind = "<種類>", x = "<列>", y = "<列>", agg = "<集め方>", by = "<列>", unit = "<単位>", cumulative = <真偽>, name = "<名前>" }`)。

### グラフを複数書くときは、`chart` を配列(`chart = [{ … }, { … }]`)にしてよい
`req~chart.config-array~1` P1
- 受け入れ: `chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }]` → `]` で2つ目の折れ線。

### `chart` の種類は bar・stacked・line・scatter・heatmap、集め方は count・sum・avg・min・max、単位は day・week・month とする
`req~chart.config-names~1` P1
- 受け入れ: `kind = "stacked"`・`agg = "avg"`・`unit = "week"` のように、5つの種類(bar 棒・stacked 積み上げ棒・line 折れ線・scatter 散布図・heatmap ヒートマップ)・5つの集め方・3つの単位のどの名前を書いても、警告なしで読む。

### `chart` で省いた鍵は、chart.open の推定で埋めるべき
`req~chart.config-defaults~1` P1
- 受け入れ: `chart = {}`(鍵を全部省く)で status の列がある表 → 警告なしで、`P` で status ごとの件数の棒(chart.open の推定)。

### `chart` の書き方が違うとき(表でない・知らない鍵や値・文字でない列の名前)は、cli.config-file の形で警告してそのグラフを外すべき
`req~chart.config-invalid~1` P1
- 受け入れ: `kind = "pie"` → 警告1行で、そのグラフは出ない。

### `chart` の列が表に無いときは、列が出そろってから cli.config-file の形で1回警告し、そのグラフを外すべき
`req~chart.config-missing-column~1` P1
- 受け入れ: `x = "none"` → 読み終えて警告1行。行が1つも無い表で `x = "none"` → 列があるかを決めず、警告しない(gantt.config と同じ)。

### 設定の `[types]`・`[rules.<列>]` に書いた列は、その列を持つノートが1件も無くても、`chart` の列として表にあると数えるべき
`req~chart.config-declared-column~1` P1
- なぜ: その列をまだ書いていないノートだけのときに、印のグラフが外れないようにするため
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]`、`chart = [{ name = "S", x = "status" }]` の表に、status を持たないノートが1件 → 警告しない。`x = "none"`(設定にもノートにも無い)→ chart.config-missing-column のとおり警告1行。

### `P`(か画面の型のメニューの「グラフ」)で、今のビューの行(絞り込み・並べ替えのあと。tree.screen と同じ)を数えたグラフの画面を開くべき
`req~chart.open-screen~1` P1
- 受け入れ: ビューで絞った表で `P` → グラフの画面が開き、ビューで絞った行だけを数える。

### 印にグラフがあれば、グラフの画面を最初のグラフで開くべき
`req~chart.open-first~1` P1
- 受け入れ: 印に `chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }]` → `P` で1つ目の「状態」のグラフで開く。

### 印にグラフが無ければ棒で開き、分ける列を、状態の列 → 決まりのある列 → チェックボックスの列 → リストの列 → 日付の列 → 最初の列の順に推定してよい
`req~chart.open-infer~1` P1
- 受け入れ: 設定なしで status の列(todo 2・doing 1・done 3)→ status で分けた棒が3本(状態の列は kanban.config の名前)。status の列が無く、決まり(cell-edit.rules の `values`)のある phase の列がある → phase で分ける。

### 印にグラフが無いときは、グラフの値を件数にするべき
`req~chart.open-count~1` P1
- 受け入れ: 設定なしで status の列(todo 2・doing 1・done 3)→ `P` で値は件数になり、todo の棒の値は 2。

### グラフの画面の題には、種類・分ける列・値と数えた行の数を出すべき
`req~chart.open-title~1` P1
- 受け入れ: 設定なしで status の列(todo 2・doing 1・done 3)→ 題に棒・status・件数と、数えた行の数 6。

### 推定した鍵があれば、グラフの画面の題にそのことを出すべき
`req~chart.open-title-inferred~1` P1
- 受け入れ: 設定なしで status の列(todo 2・doing 1・done 3)→ 題に「推定」。

### 列が1つも無い表と、行が1つも無いビューでは、グラフの画面を理由1行で開かないべき
`req~chart.open-refuse~1` P1
- 受け入れ: 列の無い表 → 理由1行で開かない。ビューで絞って行が1つも無い → 理由1行で開かない。

### 分ける列が文字かチェックボックスの列なら、値ごとの項目にするべき
`req~chart.buckets-by-value~1` P1
- 受け入れ: status が todo の2行と done の3行 → 項目は todo(2)と done(3)。

### 分ける列がリストの列なら、要素ごとの項目にするべき(1行が複数の項目に入る。navigation.frequency と同じ)
`req~chart.buckets-list-items~1` P1
- 受け入れ: tags(リスト)で 会議 と 家 を持つ行 → 会議にも家にも1。

### 分ける列の値の無い行は、最後の「(なし)」の項目にするべき
`req~chart.buckets-none~1` P1
- 受け入れ: status の値の無い行が1つ → 最後の「(なし)」の項目に 1。

### 日付の列は日・週・月の区切りで分け、最初から最後までの区切りを、行の無い区切りも 0 として並べるべき(開くときは区切りが幅に収まる最も細かい単位。gantt.open と同じ)
`req~chart.buckets-date~1` P1
- 受け入れ: due が 10/01 と 10/20 の2行で週の単位 → 10/01 の週から 10/20 の週まで4つの区切り(間の2つは 0)。

### 数の列は、等しい幅の区間(10 まで)に分けるべき
`req~chart.buckets-number~1` P1
- 受け入れ: estimate が 0・37・100 の3行で分ける → 区間は 10 以下で、どの区間も同じ幅。

### 日付と数の列で分けるとき、値の無い・読めない行は数えず、外した行の数を示すべき
`req~chart.buckets-x-skip~1` P1
- 受け入れ: due で分けて、due の値が無い行と due が someday の行 → どちらも数えず「外した 2 行」。

### グラフの値は、件数か、数の列(`y`)の合計・平均・最小・最大にするべき
`req~chart.buckets-agg~1` P1
- 受け入れ: todo の行の estimate が 2 と 4 → 件数 2・合計 6・平均 3・最小 2・最大 4。

### グラフの値の計算では、数でない値の行を外して、外した数を示すべき
`req~chart.buckets-y-skip~1` P1
- 受け入れ: estimate の合計で値が "abc" の行 → 合計に入れず「外した 1 行」。

### 項目は、列の決まりがあればその順、日付と数は小さい順、ほかは値の大きい順(同じなら名前の順)に並べるべき
`req~chart.buckets-order~1` P1
- 受け入れ: 決まりの無い status で done 3・todo 2 → done、todo の順。`values = [todo, doing, done]` の決まり → todo、doing、done の順。

### 棒は、1項目を1行の横棒にし、左に項目の名前、右に値を出すべき
`req~chart.bar-rows~1` P1
- 受け入れ: status(todo 2・doing 1・done 3)の棒 → 3行の横棒で、各行の左に項目の名前(todo など)、右に値(2 など)。

### 棒は、最大の値を棒の全長にするべき(負の値は棒を描かず値だけ出す)
`req~chart.bar-scale~1` P1
- 受け入れ: status(todo 2・doing 1・done 3)の棒 → done の棒が全長、todo はその 2/3。値が -2 の項目 → 棒を描かず値「-2」だけ。

### 積み上げ棒は、内訳の列(`by`)の値ごとに棒を塗り分けて、上に凡例を出すべき
`req~chart.bar-stacked~1` P1
- 受け入れ: 積み上げ棒で x = assignee、by = status → 凡例に3つの値と塗りの字。

### 積み上げ棒の内訳は、全体で多い順に5つまでとし、残りを「その他」にまとめるべき
`req~chart.bar-stacked-top5~1` P1
- 受け入れ: by の列の値が o1〜o7 で、多いのが o1〜o5 → 内訳は o1〜o5 の5つで、o6・o7 は「その他」。

### 積み上げ棒は、`%` で、各棒を全長にした内訳の割合に切り替えるべき
`req~chart.bar-percent~1` P1
- 受け入れ: 積み上げ棒で x = assignee、by = status → `%` でどの棒も全長。

### 棒の項目が高さに収まらないときは、選んでいる項目が見える所までずらすべき
`req~chart.bar-scroll~1` P1
- 受け入れ: 項目が 40 個で高さ 30 の画面 → `j` で最後の項目まで選ぶと、最後の項目が見える所までずれる。

### 色なしでも、積み上げ棒の内訳を塗りの字で見分けられるべき(cell-view.not-color-only)
`req~chart.bar-fill-glyph~1` P1
- 受け入れ: `--no-color` でも塗りの字が内訳ごとに違う。

### 色なしでも、棒の選んでいる項目に `>` を付けるべき(cell-view.not-color-only)
`req~chart.bar-cursor~1` P1
- 受け入れ: `--no-color` で、選んでいる項目に `>` が1つ付く。

### 折れ線は、分ける列が日付か数の列のとき、区切りごとの値を点で打って線で結ぶべき
`req~chart.line-points~1` P1
- 受け入れ: due で 2026-10 に3件・2026-11 に1件、単位が月 → 2点を結ぶ線。

### 折れ線は、左に値の目盛り(最小と最大)、下に最初と最後の区切りの名前を出すべき
`req~chart.line-axes~1` P1
- 受け入れ: due で 2026-10 に3件・2026-11 に1件、単位が月 → 左の目盛りの最大は 3、下に 2026-10 と 2026-11。

### 折れ線は、`c` で累積(その区切りまでの合計)に切り替えるべき
`req~chart.line-cumulative~1` P1
- 受け入れ: due で 2026-10 に3件・2026-11 に1件、単位が月 → `c` で 3 と 4。

### 折れ線は、内訳の列(`by`)があれば内訳ごとに線を引くべき(5本まで)
`req~chart.line-series~1` P1
- 受け入れ: by = status → 線が3本。

### 折れ線の内訳の線は、点の字(`*` `+` `o` `x` `#`)と凡例で見分けられるべき(cell-view.not-color-only)
`req~chart.line-series-glyph~1` P1
- 受け入れ: by = status → 3本の線の点の字が違い、凡例にそれぞれの点の字と値が出る。

### 折れ線は、`h` `l` で区切りを選び、選んだ区切りの名前と値を出すべき
`req~chart.line-select~1` P1
- 受け入れ: due で 2026-10 に3件・2026-11 に1件、単位が月 → `l` で「2026-11: 1」。

### 散布図は、分ける列(横)と値の列(縦。`y`)がどちらも数か日付の列のとき、1行を1点で打つべき
`req~chart.scatter-points~1` P1
- 受け入れ: estimate と actual の数の列で3行 → 3点。

### 散布図は、内訳の列(`by`)があれば点の字で見分けられるべき
`req~chart.scatter-series-glyph~1` P1
- 受け入れ: estimate と actual の散布図で by = status → status の値ごとに点の字が違う。

### 散布図は、横か縦の値が無い・読めない行を外して、外した数を示すべき
`req~chart.scatter-skip~1` P1
- 受け入れ: estimate と actual の数の列で actual が空の行 → 外した 1 行。

### 散布図は、`j` `k` で点(横の値の順)を選び、選んだ点の行の名前と2つの値を出すべき
`req~chart.scatter-select~1` P1
- 受け入れ: estimate と actual の数の列で3行 → `j` で estimate の小さい順に次の点を選び、その行の名前と estimate・actual の値を出す。

### 散布図は、`Enter` で表に戻って、選んだ点の行を選ぶべき
`req~chart.scatter-enter~1` P1
- 受け入れ: `j` → `Enter` → 表でその行が選ばれる。

### ヒートマップは、分ける列(縦)と内訳の列(横。`by`)の値の組ごとの値を、ます目に数で出すべき
`req~chart.heatmap-cells~1` P1
- 受け入れ: status × assignee で todo かつ alice が2行 → そのます目に 2。

### ヒートマップは、値の大きさを地の濃さで示すべき
`req~chart.heatmap-shade~1` P1
- 受け入れ: status × assignee で todo かつ alice が2行、todo かつ bob が1行 → todo × alice のます目の地が todo × bob より濃い。

### ヒートマップは、色なしでも濃さを字(` ` `░` `▒` `▓` `█`)で示すべき(cell-view.not-color-only)
`req~chart.heatmap-no-color~1` P1
- 受け入れ: `--no-color` で最大のます目に `█`。

### ヒートマップは、`h` `j` `k` `l` でます目を選び、選んだます目の組と値を出すべき
`req~chart.heatmap-select~1` P1
- 受け入れ: status × assignee で todo × alice のます目を選ぶ → 組(todo と alice)と値 2 を出す。`l` → 右のます目を選ぶ。

### ヒートマップは、`Enter` で選んだます目の2つの値の行だけに絞って表に戻るべき(日付と数の区切りでは絞らずに理由を出す)
`req~chart.heatmap-enter~1` P1
- 受け入れ: todo × alice で `Enter` → 表が status = todo かつ assignee = alice の行だけ。縦が due の月の区切りで `Enter` → 絞らずに理由を出す。

### グラフの画面では、`Tab`・`Shift+Tab` で種類を切り替えてよい
`req~chart.keys-kind~1` P1
- 受け入れ: 棒で `Tab` → 積み上げ棒。積み上げ棒で `Shift+Tab` → 棒。

### グラフの画面では、`x` で分ける列を切り替えてよい
`req~chart.keys-x~1` P1
- 受け入れ: status で分けた棒で `x` → 分ける列が次の列に変わる。

### グラフの画面では、`y` で値(件数 → 数の列)を切り替えてよい
`req~chart.keys-y~1` P1
- 受け入れ: 値が件数の棒で `y` → 値が数の列 estimate に変わる。

### グラフの画面では、`a` で集め方を切り替えてよい
`req~chart.keys-agg~1` P1
- 受け入れ: 値が estimate の合計の棒で `a` → 集め方が合計から次の集め方に変わる。

### グラフの画面では、`b` で内訳の列(なし → 列)を切り替えてよい
`req~chart.keys-by~1` P1
- 受け入れ: `Tab` で積み上げ棒にしてから `b` → 内訳が次の列。

### グラフの画面では、`+` `-` で日付の単位を切り替えてよい
`req~chart.keys-unit~1` P1
- 受け入れ: due で分けた週の単位の棒で `+` と `-` → 日付の単位がそれぞれ週から別の単位(日か月)に変わる。

### グラフの画面では、`[` `]` で印に書いたグラフを切り替えてよい
`req~chart.keys-charts~1` P1
- 受け入れ: 印に `chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }]` → `]` で2つ目の折れ線、`[` で1つ目の「状態」に戻る。

### グラフの画面での切り替えは画面の中だけにして、印は書き換えないべき
`req~chart.keys-no-write~1` P1
- 受け入れ: `Tab`・`x`・`b` で切り替えてから表に戻る → 印は書き換わらない。

### 種類に合う列が無いとき(折れ線に日付と数の列が無い、散布図に数か日付の列が2つ無い、積み上げ棒とヒートマップに内訳にできる列が無い)は、その種類にせずに理由を出すべき
`req~chart.keys-kind-unfit~1` P1
- 受け入れ: 日付と数の列の無い表で折れ線へ `Tab` → 理由1行で次の種類へ。

### 棒と積み上げ棒では、`j` `k` で項目を選ぶべき
`req~chart.keys-item~1` P1
- 受け入れ: status の棒で最初の項目を選んでいる → `j` で次の項目、続けて `k` で最初の項目に戻る。

### 棒と積み上げ棒では、`Enter` で選んだ項目の値の行だけに絞って表に戻るべき(navigation.frequency の頻度表の Enter と同じ絞り込み)
`req~chart.keys-enter~1` P1
- 受け入れ: done の棒で `Enter` → 表が status = done の行だけになり、ヘッダーに絞り込み。日付の区切りの項目で `Enter` → 絞らずに理由を出す(日付と数の区切りの項目では絞らない)。

### グラフの画面では、`Esc`・`q`・`P` で表に戻るべき
`req~chart.keys-back~1` P1
- 受け入れ: グラフの画面で `Esc`・`q`・`P` のどれか → 表の画面に戻る。

### グラフの画面では、項目のクリックでその項目を選ぶべき
`req~chart.keys-click~1` P1
- 受け入れ: todo の棒をクリック → todo を選び、グラフの画面のまま。

### グラフの画面では、選んでいる項目をもう一度クリックすると、`Enter` と同じにするべき
`req~chart.keys-click-again~1` P1
- 受け入れ: 選んでいる todo の棒をもう一度クリック → `Enter` と同じく、表が status = todo の行だけ。

### グラフの画面では、ホイールで選んでいる項目を動かすべき
`req~chart.keys-wheel~1` P1
- 受け入れ: 棒で最初の項目を選んでいてホイールを1つ回す → 選んでいる項目が隣の項目に動く。

### グラフの画面では、値を直せない
`req~chart.keys-read-only~1` P1
- 受け入れ: グラフの画面で直すキー(`e` など)を押す → ためた変更は 0 で、ノートは変わらない。

### グラフの画面を開いているあいだにノートが外で変わったら、数え直すべき
`req~chart.keys-recount~1` P1
- 受け入れ: グラフを開いたまま外で status を書き換える → 数が変わる。

### 見せ方の一覧(screen.views-page)・できることの一覧(screen.features-page)・キーの文書(screen.keys-docs)・設定の文書にグラフの項を持ち、組み込みの tasks のスターターに `chart` の設定を同梱し、場面の台本にグラフの場面を、録画の台本にグラフの録画を持つべき
`req~chart.docs~1` P1
- 受け入れ: docs/manual/ja・en の views.md に「グラフ」の節(画像 images/chart.svg)、features.md に項。starters/tasks の印に `chart`。docs/manual-scenarios.toml に id = "chart"。demos/chart.tape(読んで判定と)。
