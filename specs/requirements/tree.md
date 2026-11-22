# 木の画面(tree) の要求

## 目的

ブックマークの台帳のように、1件1ノートで、置き場の階層を「根 > フォルダ > …」のような文字の列で持つ表を、フォルダの木と、選んだフォルダの中の並びで見せる画面の型。表の画面とは別の画面の型(scope.screen-types)で、関係マップと同じくヘッダーのメニューから切り替える。どの列が階層でどの列が並びかは、ワークスペースの印に書く。ブックマーク専用ではなく、階層を文字の列で持つどの台帳にも使う。版1 は見るだけで、行を別のフォルダへ移す・並べ替えるのは次の段。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/tree/spec.md](../_history/decidespec/tree/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 次の段: フォルダの並びをノートの外の一覧(例: 台帳の設定のフォルダの順)から読むこと。版1 は名前の順(tree.folders)。
- 採らない: ブックマーク専用の画面や、ブラウザのブックマークのファイルを直接読むこと(scope.core・scope.standalone-oss。台帳の側の道具の役目)。
- 採らない: 表のテーマの切り替えで木を作ること(scope.screen-types)。

## 前提(旧い仕様で AI が置いた仮定)

- 木の画面への切り替えのキーは設計で決める(関係マップの `R` と同じく、窓を開かずに切り替えるキーを1つ持つ)。
- 階層の列が文字のリスト(`[a, b]`)のときは、版1 では最初の要素だけを使う。

## 要求

### 木の画面

#### 表の画面とは別の画面の型(scope.screen-types)として木の画面を持ち、画面の型のメニューから切り替えられるべき
`req~tree.screen-type~1` P1
- Acceptance:
  - Given:
  - When: 印に階層の列を書いた表で「▦ 表 ▾」を押して「木」を選ぶ
  - Then: 木の画面になる。

#### 表に階層の列の設定(tree.config)があるときだけ、画面の型のメニュー(relations.screen-menu)に「木」を出すべき(設定が無い表では出さない)
`req~tree.screen-menu~2` P1
- Acceptance:
  - Given:
  - When: 印に階層の列を書いた表で「▦ 表 ▾」を押す
  - Then: 窓に「木」がある。
- Acceptance:
  - Given: 設定の無い表
  - When:
  - Then: 窓に「木」が無い。

#### 木の画面は、今の表のビュー(絞り込み・where・計算の列。base-view.table-keys・workspace.marker-base・workspace.marker-where)を通った行だけを扱うべき
`req~tree.screen-view-rows~1` P1
- Rationale: ビューで外した行を木に出さないため
- Acceptance:
  - Given:
  - When: ビュー「使う」(status が active だけ)で木を開く
  - Then: archived の行が木に出ない。

#### ワークスペースの印の `[[table]]` に `tree = { path = "<列>", separator = "<区切り>", order = "<列>" }` を書いてよく、`path` の列の値を `separator`(省略は ` > `)で切ったものを、その行の置き場の階層とみなすべき(`order` は省略してよい)
`req~tree.config-path~1` P1
- Acceptance:
  - Given: `tree = { path = "folder", order = "order" }` と folder が `bar > 開発 > Web` の行
  - When:
  - Then: bar の下の 開発 の下の Web に置かれる。

#### `tree` の `path` の列が無い表や、区切りが空の文字のときは、cli.config-file の形で警告して木を出さないべき
`req~tree.config-invalid~2` P1
- Acceptance:
  - Given: `separator = ""`
  - When:
  - Then: 警告1行で、メニューに「木」が出ない。
- Acceptance:
  - Given: `path` の列が表に無い
  - When:
  - Then: 読み終えたときに警告1行で、木を出さない。

#### 設定の `[types]`・`[rules.<列>]` に書いた列は、その列を持つノートが1件も無くても、`tree` の `path` の列として表にあると数えるべき
`req~tree.config-declared-column~2` P1
- Rationale: 階層をまだ書いていないノートだけのときに、木の設定が外れないようにするため
- Acceptance:
  - Given: `[types] folder = "text"`、`tree = { path = "folder" }` の表に、folder を持たないノートが1件
  - When:
  - Then: 警告しない。
- Acceptance:
  - Given: `path = "none"`(設定にもノートにも無い)
  - When:
  - Then: tree.config-invalid のとおり警告1行。

#### `tree` を書いた表に行が1つも無いとき(見本なしで作った台帳など)は、`path` の列があるかを決めず、警告しないべき(画面の型のメニューには出し、開くときに出す行が無いと理由を出す)
`req~tree.config-empty-table~1` P1
- Rationale: 行が無い表の列はノートのキーから作るので、`path` の列があるかをまだ決められないため
- Acceptance:
  - Given: 行が1つも無い表
  - When:
  - Then: 読み終えても警告せず、メニューに出て、開くと理由1行。外からノートを足すと `T` で開く。

#### 木の画面の左に、階層の木を描くべき
`req~tree.folders-pane~1` P1
- Acceptance:
  - Given: `bar > 開発`(2行)・`bar > 開発 > Web`(1行)・`other`(1行)・空(1行)
  - When:
  - Then: 左の欄に bar・開発・Web・other・(なし) のフォルダの木が出る。

#### 行の値に出てくる途中の階層も、木のフォルダとして作るべき(`a > b > c` だけでも a と b を作る)
`req~tree.folders-intermediate~1` P1
- Acceptance:
  - Given: `a > b > c` の行だけの表
  - When:
  - Then: 木に c だけでなく a と b のフォルダもできる。

#### 木の各フォルダに、その下にある行の数(下の階層も含む)を添えるべき
`req~tree.folders-count~1` P1
- Acceptance:
  - Given: `bar > 開発`(2行)・`bar > 開発 > Web`(1行)・`other`(1行)・空(1行)
  - When:
  - Then: 木は bar(3)・開発(3)・Web(1)・other(1)・(なし)(1)。開発を閉じても数は3のまま。

#### 木のフォルダを開く・閉じるができるべき
`req~tree.folders-toggle~2` P1
- Acceptance:
  - Given:
  - When: 開発を閉じる
  - Then: Web が隠れる。
- Acceptance:
  - Given:
  - When: 開発を開く
  - Then: Web がまた出る。

#### 階層の列が空の行は、木の根の「(なし)」にまとめるべき
`req~tree.folders-none~1` P1
- Acceptance:
  - Given: 階層の列が空の行が1行
  - When:
  - Then: 根に (なし)(1) ができ、(なし) を選ぶと右にその行が出る。

#### 同じ名前のフォルダは、親が違えば別のフォルダにするべき
`req~tree.folders-same-name~1` P1
- Acceptance:
  - Given: `a > x`(1行)・`b > x`(1行)
  - When:
  - Then: a の下の x(1) と b の下の x(1) の2つのフォルダになる。

#### 木のフォルダの並びは、名前の順にするべき
`req~tree.folders-order~1` P1
- Acceptance:
  - Given: ノート z.md の階層が `bar`、a.md の階層が `other`
  - When:
  - Then: 木は bar・other の順(ノートの順でなく名前の順)。

#### 木の画面の右に、選んだフォルダの直下にある行を、今のビューの列で並べるべき
`req~tree.rows-direct~1` P1
- Acceptance:
  - Given:
  - When: `bar > 開発`(2行)・`bar > 開発 > Web`(1行)で 開発 を選ぶ
  - Then: 右に 開発 の直下の2行が今のビューの列で出て、Web の行は出ない。

#### 右の行の並びは、tree.config の `order` の列の小さい順にするべき(数として読めない値と空は後ろに、同じ値はノートの名前の順)
`req~tree.rows-order~1` P1
- Acceptance:
  - Given:
  - When: 開発 を選ぶ
  - Then: order 10・20・空 の3行がその順に出る。order が `abc` の行は空と同じく後ろ。

#### tree.config に `order` が無ければ、右の行をノートの名前の順にするべき
`req~tree.rows-name-order~1` P1
- Acceptance:
  - Given: `tree = { path = "folder" }`(order なし)で 開発 の直下に b.md と a.md
  - When:
  - Then: 右は a・b の順。

#### 右の欄で行を選んで Enter で、表の画面のその行に移れるべき
`req~tree.rows-enter~1` P1
- Acceptance:
  - Given:
  - When: 行で Enter
  - Then: 表の画面に戻り、その行が選ばれている。

#### 右の欄で選んだ行に `http://` か `https://` で始まる値の列があれば、その URL を OS の既定のブラウザで開く操作を、操作の一覧とパレットから出せるべき
`req~tree.open-url-action~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: url が `https://example.com` の行で「URL を開く」→ OS の開くコマンドに渡る(試験では開く口を差し替えて渡った値を確かめる)。

#### URL を開く前に、その URL を下の帯に見せるべき
`req~tree.open-url-show~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: url が `https://example.com` の行で「URL を開く」→ 開く前に帯に URL が出る。

#### URL を開く操作で、`http`・`https` 以外の値は開いてはならない
`req~tree.open-url-scheme~1` P1
- Rationale: ノートの値から任意のコマンドやファイルを開かないため
- Acceptance:
  - Given: `file:///etc/passwd`・`javascript:` の行
  - When:
  - Then: 出さないか、理由を出して開かない。

#### 木の画面の段組み(左の木・右の行)は、端末の幅と高さに合わせて変えるべき(広ければ並べ、狭ければ1つずつにしてキーかタブで切り替える)
`req~tree.layout-width~1` P1
- Acceptance:
  - Given: 幅 160・100・60 で木の画面を出す
  - When:
  - Then: 160・100 は並べ、60 は1つずつ。

#### 木の画面は、どの幅でも、文字が重なったり切れて読めなくなったりしてはならない
`req~tree.layout-legible~1` P1
- Acceptance:
  - Given: 幅 160・100・60 で木の画面を出す
  - When:
  - Then: どれも重ならず、切れて読めなくなる文字も無い。

#### 木の画面で、左のフォルダをクリックすると、そのフォルダを選ぶべき
`req~tree.mouse-folder-click~1` P1
- Acceptance:
  - Given:
  - When: Web をクリック
  - Then: 選ぶ。

#### 木の画面で、フォルダの開閉の印をクリックすると、そのフォルダを開閉するべき
`req~tree.mouse-toggle~2` P1
- Acceptance:
  - Given:
  - When: 開発 の印をクリック
  - Then: 閉じる。
- Acceptance:
  - Given:
  - When: もう一度クリック
  - Then: 開く。

#### 木の画面で、右の行をクリックすると、その行を選ぶべき
`req~tree.mouse-row-click~1` P1
- Acceptance:
  - Given:
  - When: 右の行をクリック1回
  - Then: その行を選び、表の画面には移らない。

#### 木の画面で、選んでいる右の行をもう一度クリックすると、表の画面のその行に移るべき
`req~tree.mouse-row-reclick~1` P1
- Acceptance:
  - Given:
  - When: 右の行をクリック2回
  - Then: 表のその行へ。

#### 木の画面で、マウスのホイールで選びを上下に動かすべき
`req~tree.mouse-wheel~2` P1
- Acceptance:
  - Given:
  - When: Web を選んでホイールを上へ
  - Then: 開発 を選ぶ。
- Acceptance:
  - Given:
  - When: ホイールを下へ
  - Then: Web を選ぶ。

### 次の段(まだ作っていない。P2)

#### 木の画面で、右の行を別のフォルダへ移せるべき(左のフォルダを選んで移す)
`req~tree.move-row-pick~1` P2
- Acceptance:
  - Given:
  - When: folder が `other` の行で移す操作 → 左で Web を選ぶ
  - Then: その行が Web の行として出る(読んで判定)。

#### 行を移すと、その行の `path` の列の値を新しいフォルダの階層の文字に書き換える変更としてため、保存の流れ(差分を見せてから書く。write-back.pending-and-diff)に乗せるべき
`req~tree.move-row-pending~1` P2
- Acceptance:
  - Given:
  - When: folder が `other` の行を Web へ移す
  - Then: その行の folder を `bar > 開発 > Web` に書き換える変更がたまり、保存では差分を見せてから書く(読んで判定)。

#### 木の画面で、右の行を上下に動かして並びを変えられるべき
`req~tree.reorder-move~1` P2
- Acceptance:
  - Given:
  - When: 開発 の右で3行目を上へ動かす
  - Then: 2行目と入れ替わって並ぶ(読んで判定)。

#### 行を動かすと、そのフォルダの行の `order` の列を、並びどおりの値(10 刻み)に振り直す変更としてため、保存の流れに乗せるべき
`req~tree.reorder-renumber~1` P2
- Acceptance:
  - Given: a・b・c の順の行で c を一番上へ
  - When:
  - Then: c・a・b の順に order が 10 ずつ増える値に振り直す変更がたまり、保存では差分を見せてから書く(読んで判定)。
