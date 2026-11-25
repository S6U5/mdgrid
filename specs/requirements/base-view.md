# 表の組み立て(base-view) の要求

## 目的

ノートのフォルダと `.base` の定義から、表の行(ノート)と列(プロパティ)を決める。`.base` には公開された文法が無く、Obsidian の版で意味が変わりうるので、互換は「table ビューのよく使う部分を正しく、それ以外は正直に未対応と出す」を方針にする(`.base` の table ビューの一部と互換)。セルの編集は [cell-edit](cell-edit.md)、画面の中での移動・検索・一時的な並べ替えは [navigation](navigation.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/base-view/spec.md](../_history/decidespec/base-view/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 次の段: ノートの中の ```` ```base ```` のブロックの解釈と、そのときの `this`(`.base` を直接開いたときの this は base-view.links-eval)。
- 旧い BV-15(新しい名前なし)は 2026-09-30 に削除([2026-09-30](../_history/decidespec/_decisions/2026-09-30-v1-review-fixes.md))。移行先: base-view の範囲外(次の段: `this` を使う式とノートの中の base のブロック)。
- 採らない: table 以外のビュー(cards・list・map・kanban)を表の見た目の切り替えで作ること(scope の scope.table-only・scope.screen-types)。
- 採らない: `.base` への書き戻し(列の順・幅・並べ替えの保存を含む。base-view.base-untouched)。見た目の状態は screen の screen.state-separate・screen.state-store で `.base` の外に持つ。
- 採らない: Dataview の inline フィールド・FLATTEN・GROUP BY の集計(Bases 自体が持たない)。

## 前提(旧い仕様で AI が置いた仮定)

- `.base` の形式と型の置き場所は 2026-09-30 の調査による(Obsidian 1.13.7〜1.14.3。公開の文法・JSON Schema は無く、`sort`・`columnSize`・`rowHeight` はヘルプに無いが実在)。Obsidian の版が上がって意味が変わったら、提案で直す。
- 関数の一覧(68個)のうち、版1で入れるのは base-view.expr-eval の範囲。

## 要求

### 何を読むか

#### `.base` のファイルを渡されたら、その定義で表を作るべき
`req~base-view.source-base-file~1` P1
- Acceptance:
  - Given:
  - When: filters が `status == "done"` の `.base` を渡す
  - Then: status が done のノートだけの表。

#### フォルダだけを渡されたら、その中の Markdown のノートを行、フロントマターのキーを列にした既定の表を作るべき
`req~base-view.source-folder~1` P1
- Acceptance:
  - Given:
  - When: 設定も `.base` も無いフォルダを渡す
  - Then: 全ノートと全キーの表。

#### ノートを探す範囲(ルートフォルダ)は、`.obsidian/` を持つ最寄りの上のフォルダとし、無ければ渡したフォルダにするべき
`req~base-view.root-folder-nearest~2` P1
- Acceptance:
  - Given:
  - When: `.obsidian/` を持つ `vault/` の下の `vault/sub` を渡す
  - Then: ルートフォルダは `vault/`。
- Acceptance:
  - Given:
  - When: 上のどこにも `.obsidian/` が無い `notes/` を渡す
  - Then: ルートフォルダは `notes/`。

#### ノートを探す範囲(ルートフォルダ)は、引数で1つ以上のフォルダを並べて広げてよい
`req~base-view.root-folder-multiple~1` P1
- Acceptance:
  - Given:
  - When: 2つのリポのフォルダを並べて渡す
  - Then: 両方のノートが1つの表に入る。

#### `.base` のファイルと、ノートの中の ````base` のブロックを書き換えてはならない
`req~base-view.base-untouched~1` P0
- Acceptance:
  - Given:
  - When:
  - Then: 表の操作の前後で `.base` のバイトが同じ。

#### 渡したフォルダの重なりやシンボリックリンクで同じノートが2度見つかったら、実体のパスで1行にするべき
`req~base-view.dedupe-real-path~1` P1
- Acceptance:
  - Given:
  - When: 同じフォルダを2回と、そこを指すリンクを渡す
  - Then: 行は1つ。

#### ルートフォルダが複数あって `.obsidian/types.json` の型が列で食い違うときは、その列をテキストとして扱い、読むだけにするべき
`req~base-view.types-conflict~1` P1
- Acceptance:
  - Given:
  - When: 根 A で date・根 B で text の `due`
  - Then: テキストで表示し、編集できない理由を出す。

### 解釈

#### Obsidian のヘルプの Bases の構文どおり、`filters`(`and`・`or`・`not` の入れ子と式の文字列)を、全体とビューの両方で解釈し、両者を AND でつなぐべき
`req~base-view.filters~1` P1
- Acceptance:
  - Given: 全体 `file.hasTag("a")`・ビュー `status == "done"`
  - When:
  - Then: 両方を満たすノートだけ。

#### table ビューの `order`・`sort`・`groupBy`・`limit` と、`properties` の `displayName` を解釈するべき
`req~base-view.table-keys~1` P1
- Acceptance:
  - Given: `groupBy: {property: status, direction: ASC}`
  - When:
  - Then: status ごとのまとまりで並ぶ。

#### `properties` に displayName の無い列の見出しは、Obsidian の既定に合わせるべき
`req~base-view.default-headings-obsidian~1` P1
- Acceptance:
  - Given:
  - When: 式の列(`formula.x`)は式の名前(`x`)、`file.name` → `file name`・`file.ctime` → `created time`・`file.mtime` → `modified time`・`file.ext` → `file extension`、それ以外の `file.x` → `file x`、ノートのキーの列はキーの名前。order に `file.name`・`formula.価格`・`status` を並べた `.base`
  - Then: 見出しは `file name`・`価格`・`status`。`--print --format json` の鍵も同じ見出し。

#### `properties` に displayName があれば、列の見出しにそれを使うべき
`req~base-view.default-headings-display-name~1` P1
- Acceptance:
  - Given: `file.name` に displayName `名前`
  - When:
  - Then: 見出しは `名前`。

#### 列の名前を見せる所(まとまりの見出し・ビューの帯の条件と並べ替えとグループの札・ビューの設定の絞り込みの一覧・カンバンとガントとグラフとリストの値の付け外しの窓の題・新しいノートの窓の欄・木の画面の右の欄の列の見出し)は、列の見出しと同じく `properties` の displayName があればそれを使い、ノートに書く値・式・設定・`--print` の鍵は変えないべき
`req~base-view.display-name-everywhere~1` P1
- Rationale: 列の見出しは「状態」なのに、まとまりの見出しや帯の札やカンバンの題には生の列名(`status`)が出て、同じ列が2つの名前で見えていたため(2026-11-25 に、まとまりの見出しの「actor:」を直したときに、同じ形の所を洗い出した)
- Acceptance:
  - Given: `properties` で `note.status` に displayName `状態` の表で、絞り込み「status が done を除く」
  - When: 表を描く
  - Then: ビューの帯の札は「状態: done を除く」。
- Acceptance:
  - Given: 同じ表で `kanban = { column = "status" }`
  - When: カンバンを開く
  - Then: 窓の題は「カンバン: 状態」。
- Acceptance:
  - Given: `note.start` に displayName `開始`、`note.due` に `期限` で、`gantt = { start = "start", end = "due" }`
  - When: ガントを開く
  - Then: 窓の題は「ガント: 開始 → 期限(…)」。
- Acceptance:
  - Given: 同じ表で、新しいノートの窓で status を聞く(`ask = ["status"]`)
  - When: `a` で窓を開く
  - Then: 欄の名前は「状態」。
- Acceptance:
  - Given: displayName の無い列 owner で絞り込む
  - When: 表を描く
  - Then: 帯の札は「owner: …」のまま。
- Needs: utest

#### 式は、比較・論理・算術、`note.`・`file.`・`formula.` の参照、よく使う関数(`file.hasTag`・`file.inFolder`・`file.hasProperty`・`contains` 系・`if`・`date`・`now`・`today` と期間の足し引き)を評価するべき
`req~base-view.expr-eval~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 関数ごとに Obsidian での結果を写した小さな例で一致。

#### 評価できない式・関数・ビューの型に出会ったら、推測で行を作ってはならない
`req~base-view.unsupported-no-guess~1` P1
- Acceptance:
  - Given: 未実装の関数を使った filters のビュー
  - When:
  - Then: 全ノートの行も、その関数を飛ばして絞った行も作らない。

#### 絞り込みが評価できないビューは開かずに、どの式が未対応かを示す理由を出さなければならない
`req~base-view.unsupported-filters~1` P1
- Acceptance:
  - Given: 未実装の関数を使った filters
  - When:
  - Then: そのビューは開かず、関数名つきで未対応と表示。

#### 評価できない列は、値の代わりに未対応の印(`?`)を薄い表示で出さなければならない
`req~base-view.unsupported-column-mark~2` P1
- Rationale: 空・null・キーが無い の見せ方(cell-view.null-empty-missing)とも、本当の値の `?` とも見分けられるようにするため
- Acceptance:
  - Given: 評価できない式の列
  - When:
  - Then: 全行に薄い `?` が出て、キーが無いときの空欄と違う。
- Acceptance:
  - Given: 値が `?` のテキストの列
  - When:
  - Then: 普通の `?`。

#### 評価できない列は、読むだけにしなければならない
`req~base-view.unsupported-column-readonly~1` P1
- Acceptance:
  - Given:
  - When: 評価できない式の列のセルで Enter
  - Then: 入力が開かず、セルは `?` のまま。

#### 評価できない列のセルを選んだら、下の帯に未対応の式を出さなければならない(色だけに頼らない。cell-view.not-color-only)
`req~base-view.unsupported-cell-band~1` P1
- Acceptance:
  - Given:
  - When: 評価できない式のセルを選ぶ
  - Then: 下の帯に未対応の式が出る(`TERM=dumb` で薄い表示が出なくても、選べば見分けられる)。

#### `.base` のうち mdgrid が解釈するものと解釈しないものを、英語(`docs/obsidian-bases.md`)と日本語(`docs/obsidian-bases.ja.md`)の文書に並べるべき
`req~base-view.support-docs-list~2` P1
- Acceptance:
  - Given:
  - When: 解釈するものは読む項目・ビューの型・演算子・関数とメソッド・`file.*` の値、解釈しないものは base-view.unsupported で未対応と出すもの。文書の対応する関数を使った絞り込み
  - Then: base-view.unsupported の未対応にならない。
- Acceptance:
  - Given:
  - When: 解釈するものは読む項目・ビューの型・演算子・関数とメソッド・`file.*` の値、解釈しないものは base-view.unsupported で未対応と出すもの。文書の解釈しないものに書いた関数を使った絞り込み
  - Then: 未対応になる。

#### 文書の関数・メソッド・`file.*` の一覧と、式の評価が受け付ける一覧が食い違えば、試験で落ちるべき
`req~base-view.support-docs-check~2` P1
- Rationale: 文書が古くなるのを防ぐため
- Acceptance:
  - Given:
  - When: 式の評価に関数を1つ足して文書を直さない
  - Then: 試験が落ちる。
- Acceptance:
  - Given:
  - When: 文書に評価できない関数を書く
  - Then: 落ちる。

#### `.base` の中の知らないキーは、エラーにせず無視し、消するべきでない
`req~base-view.unknown-keys~1` P1
- Acceptance:
  - Given:
  - When: ビューに独自のキーを足した `.base`
  - Then: 開ける。

#### `.base` の views が2つ以上あれば、ビューを切り替えられるべき
`req~base-view.view-switch-tabs~1` P1
- Acceptance:
  - Given: ビューが3つの `.base`
  - When:
  - Then: タブに3つ並び、切り替えると行と列が変わる。

#### 起動時は、既定に選んだビュー(navigation.default-view)を、無ければ先頭のビューを開くべき
`req~base-view.view-switch-startup~2` P1
- Acceptance:
  - Given:
  - When: 既定のビューを選んでいない、ビューが3つの `.base`
  - Then: 先頭が選ばれている。
- Acceptance:
  - Given:
  - When: 「完了」を既定のビューにして開き直す
  - Then: 「完了」が選ばれている。

#### 式は `file.links` を、そのノートの本文とフロントマターにあるリンクの行き先のノートのリストとして評価するべき
`req~base-view.links-eval-links~1` P1
- Acceptance:
  - Given: リンクは `[[名前]]`・`[[名前|表示]]`・`[[名前#見出し]]`・`[文字](相対のパス.md)`。a.md の本文に `[[b]]` と `[x](sub/c.md)`
  - When:
  - Then: a の file.links に b と sub/c。

#### `file.links` の行き先は、Obsidian と同じく、ルートフォルダからのパス、無ければ同じ名前のノートに解き、解けないリンクは文字のまま入れるべき
`req~base-view.links-eval-resolve~1` P1
- Acceptance:
  - Given: 同じ名前の `b.md` と `sub/b.md` があって `[[b]]`
  - When:
  - Then: 根に近い・パスの短い `b.md` に解く。`[[無い]]` は文字 "無い" のまま入る。

#### 式は `file.backlinks` を、そのノートを行き先に持つノートのリストとして評価するべき
`req~base-view.links-eval-backlinks~1` P1
- Acceptance:
  - Given:
  - When: b.md を a と c が指す
  - Then: b の file.backlinks に a と c。

#### 式は `file.hasLink(x)`(x はノートの名前・パス・`this.file`)を評価するべき
`req~base-view.links-eval-has-link~1` P1
- Acceptance:
  - Given: `file.hasLink("b")`
  - When:
  - Then: b を指すノートで真。

#### `.base` を直接開いたときの `this` はその `.base` のファイルとし、`this.file.name`・`this.file.folder` などと `file.hasLink(this.file)` を評価するべき
`req~base-view.links-eval-this~2` P1
- Acceptance:
  - Given:
  - When: Projects.base を開き `file.inFolder(this.file.folder)`
  - Then: .base と同じフォルダのノート。
- Acceptance:
  - Given: `file.links.contains(this.file)`
  - When:
  - Then: .base を指すノート。
- Acceptance:
  - Given: `file.hasLink(this.file)`
  - When:
  - Then: .base を指すノート。

#### ノートを探すときは、名前が `.` で始まるフォルダ(Obsidian がノートに数えない隠しフォルダ)と `node_modules` の下を探さないべき
`req~base-view.skip-hidden-dirs~1` P1
- Rationale: コードのリポを開いたときに依存のパッケージの文書を行にしないため
- Acceptance:
  - Given: `a.md`・`.hidden/b.md`・`node_modules/p/README.md`・`sub/c.md` のフォルダを --print
  - When:
  - Then: a と sub/c の2行。

#### 渡したフォルダそのものが隠しフォルダでも、その中は探してよい
`req~base-view.skip-hidden-given~1` P1
- Acceptance:
  - Given:
  - When: `.hidden/b.md` のある `.hidden` を直接渡す
  - Then: b の1行。

#### `.base` のビューの `summaries` のうち、Obsidian の組み込みの集計を、表の下に1行で、集計した列の下に「集計の名前 値」で出すべき
`req~base-view.summaries-builtin~2` P1
- Acceptance:
  - Given: `summaries` は列の id
  - When:
  - Then: 集計の名前で、組み込みは Average・Min・Max・Sum・Range・Median・Stddev・Earliest・Latest・Checked・Unchecked・Empty・Filled・Unique。
- Acceptance:
  - Given: 英語の画面で `summaries: {estimate: Sum, due: Earliest, done: Checked}`
  - When:
  - Then: 表の下に「Sum 18」「Earliest 2026-10-05」「Checked 2」。

#### 集計は、今のビューの行(絞り込みのあと)で計算するべき
`req~base-view.summaries-filtered~1` P1
- Acceptance:
  - Given:
  - When: `estimate: Sum` の集計のあるビューで、絞り込みで行が減る
  - Then: 値も変わり、残った行の estimate の合計になる。

#### 集計の名前は、英語と日本語で見せるべき
`req~base-view.summaries-names~2` P1
- Acceptance:
  - Given: 英語の画面
  - When:
  - Then: 集計の名前は Obsidian の名前(`Sum`)。
- Acceptance:
  - Given: 日本語の画面
  - When:
  - Then: 同じ集計の名前を日本語で出す。

#### 集計は、型の合わない値を数えないべき
`req~base-view.summaries-type-mismatch~1` P1
- Acceptance:
  - Given: 数の列 `estimate` が 10・8 と文字 `abc` の3行で Sum
  - When:
  - Then: 値は 18(`abc` を数えない)。

#### 式で書いた集計(最上位の `summaries` の formula)とまとまりごとの集計は解釈しないで、未対応の理由を出すべき
`req~base-view.summaries-unsupported~1` P1
- Acceptance:
  - Given: `values.mean` の集計
  - When:
  - Then: 未対応の理由が出て、ビューは開く。

### 今日・今からの隔たり(`relative()`)

#### 日付に付けた `relative()` は、今日からの隔たりを今の言語の文字にし、暦で1か月に満たなければ日の数で言うべき
`req~base-view.relative-days~1` P1
- Rationale: 期限まであと何日か、何日過ぎたかを、日付を見て数えずに読めるようにするため
- Acceptance:
  - Given:
  - When:
  - Then: 今日が 2026-10-06(日本語)→ `date("2026-10-20").relative()` は `あと14日`、`date("2026-10-01").relative()` は `5日前`、`date("2026-10-06").relative()` は `今日`、`date("2026-11-05").relative()` は `あと30日`。英語では `in 3 days`・`5 days ago`。

#### 日付に付けた `relative()` は、暦で1か月以上なら月の数、12か月以上なら年の数で言い、端数は切り捨てるべき
`req~base-view.relative-months~1` P1
- Acceptance:
  - Given: 今日が 2026-10-06
  - When:
  - Then: 2026-11-06 は `あと1か月`、2027-01-15 は `あと3か月`、2026-09-06 は `1か月前`、2026-09-07 は `29日前`、2027-10-05 は `あと11か月`、2027-10-06 は `あと1年`、2024-03-01 は `2年前`。

#### 日時に付けた `relative()` は、今からの隔たりを言い、1日に満たなければ時間、1時間に満たなければ分、1分に満たなければ「今」にするべき
`req~base-view.relative-time~1` P1
- Acceptance:
  - Given: 今が 2026-10-06 00:00
  - When:
  - Then: 2026-10-06T03:20 は `あと3時間`、2026-10-06T00:20 は `あと20分`、2026-10-05T23:15 は `45分前`、2026-10-06T00:00:30 は `今`、2026-10-08T12:00 は `あと2日`。

#### 日付・日時でない値に付けた `relative()` は `null` にするべき
`req~base-view.relative-not-a-date~1` P1
- Acceptance:
  - Given: `"a".relative()`・`(5).relative()`・文字の列 `status.relative()`
  - When:
  - Then: どれも `null`。

### 集まりの変化

#### ノートの変化は、ファイルの更新時刻と大きさを一定の間隔(既定 1 秒、設定で変えられる)で見て、変わったノートだけを読み直して表に反映するべき
`req~base-view.watch-poll~1` P1
- Acceptance:
  - Given:
  - When: 外でノートを1つ直す
  - Then: 次の間隔で、そのノートの行だけが変わる。

#### ノートの変化を知るのに、ファイルの変更通知は、使うなら補助にとどめるべき
`req~base-view.watch-notify-aux~1` P1
- Acceptance:
  - Given:
  - When: 変更通知が届かない状態で外でノートを1つ直す
  - Then: 次の間隔で、そのノートの行が変わる。

#### 編集中・リストを開いている間は、ノートの読み直しを止めるべき
`req~base-view.watch-pause~1` P1
- Acceptance:
  - Given:
  - When: セルの編集中に外でノートを直す
  - Then: 間隔が過ぎても、編集中は表が変わらない。

#### ノートが 2,000 を超えるルートフォルダでは、1回の間隔で見るノートを 2,000 に限り、順に回して全部を見てよい
`req~base-view.watch-round-robin~1` P1
- Rationale: 何もしていなくても CPU を使い続けないため
- Acceptance:
  - Given:
  - When: 2,500 ノートのルートフォルダで後ろのほうのノートを直す
  - Then: 2回の間隔のうちに変わる。

#### 読み直したあとも、同じノートの行を選んだままにし、その行が消えたら最寄りの行を選ぶべき
`req~base-view.keep-selection~1` P1
- Acceptance:
  - Given:
  - When: 選んだ行のノートを消す
  - Then: 下の行が選ばれる。

#### ノートが1万あっても、最初の画面は読み込みを待たずに出し、読み込み中は進捗を出し、読み込みの中止の操作(既定は Ctrl+G。Ctrl+C はコピー(output.copy))で読み込みを止められるべき
`req~base-view.large-load~2` P1
- Acceptance:
  - Given: 1万ノートの材料
  - When:
  - Then: 1秒以内に画面が出て、進捗が増える。
- Acceptance:
  - Given:
  - When: Ctrl+G
  - Then: 読み込みが止まり、読んだ分の表が残る。

### mdgrid のビュー

#### `.base` を使わずに、開いた対象(フォルダか `.base`)ごとに、mdgrid 独自のビューを名前つきで複数持ち、ビューのタブで切り替えられるべき
`req~base-view.own-views-tabs~1` P1
- Acceptance:
  - Given:
  - When: `.base` の無いフォルダで開き、設定の画面の「名前を付けて保存」でビューを2つ作る
  - Then: タブに「既定の表」と2つの名前が並び、切り替えると絞り込みや列が変わる。

#### mdgrid のビューの定義は、mdgrid の設定の置き場の、人が読み書きできる TOML に置くべき
`req~base-view.own-views-toml~2` P1
- Acceptance:
  - Given:
  - When: ビューを2つ作り、終了して開き直す
  - Then: 同じ2つのタブが並ぶ。
- Acceptance:
  - Given:
  - When: 設定の置き場の TOML を手で書き足す
  - Then: タブが増える。

#### mdgrid のビューの定義は、ノートのフォルダには書かないべき
`req~base-view.own-views-no-note-files~1` P1
- Acceptance:
  - Given:
  - When: ビューを2つ作る
  - Then: ノートのフォルダに新しいファイルが無い。

#### mdgrid のビューは、`.base` のビューより多くの機能を持ってよい
`req~base-view.own-views-beyond-base~1` P1
- Acceptance:
  - Given:
  - When: `.base` で表せない設定(グループの空の値のまとまりを隠す。navigation.group-options)を持つビューを mdgrid のビューとして保存し、開き直す
  - Then: その設定が効いている。

#### ビューの設定の画面(navigation.view-settings)から、今の見せ方を名前を付けて mdgrid のビューとして保存できるべき
`req~base-view.save-view-save-as~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `o` で done を外し「名前を付けて保存」で「未完了」→ タブに「未完了」が増える。

#### ビューの設定の画面から、今の見せ方で今の mdgrid のビューを上書きできるべき
`req~base-view.save-view-overwrite~1` P1
- Acceptance:
  - Given:
  - When: 「未完了」で別の絞り込みにして上書き
  - Then: 開き直しても上書きした見せ方。

#### ビューの設定の画面から、mdgrid のビューの名前を変えられるべき
`req~base-view.save-view-rename~1` P1
- Acceptance:
  - Given:
  - When: 「未完了」の名前を「残り」に変える
  - Then: タブが変わる。

#### ビューの設定の画面から、mdgrid のビューを削除できるべき
`req~base-view.save-view-delete~1` P1
- Acceptance:
  - Given:
  - When: 「未完了」を消して `y`
  - Then: 「未完了」のタブが消える。

#### mdgrid のビューの削除は、消すビューの名前を示して確かめるべき(`y` で消す。ほかの打ち込みと Esc ではやめる)
`req~base-view.save-view-delete-confirm~1` P1
- Acceptance:
  - Given:
  - When: 「未完了」を消す
  - Then: 「未完了」を消すかを聞かれ、Esc ではそのまま。

#### mdgrid のビューを消した直後の取り消し(`u`)で、定義・タブの順と隠す・既定のビュー・見た目の状態ごと元に戻せるべき
`req~base-view.save-view-undo-delete~1` P1
- Acceptance:
  - Given:
  - When: 消した直後に表で `u`
  - Then: 「未完了」のタブが元の位置に戻り、views.toml にも戻る。

#### mdgrid のビューを `.base` のビューとして書き出せてよい
`req~base-view.export-base-write~1` P1
- Acceptance:
  - Given:
  - When: 絞り込みと並べ替えとグループのあるビューを書き出す
  - Then: Obsidian で開ける `.base` ができ、同じ行が出る。

#### `.base` のビューを mdgrid のビューとして取り込めてよい
`req~base-view.export-base-import~1` P1
- Acceptance:
  - Given:
  - When: tasks.base のビュー「進行」を取り込む
  - Then: mdgrid のビューのタブ「進行」が増え、`.base` の絞り込みが効く。

#### 書き出しと取り込みでは、`.base` で表せない部分(または mdgrid で解釈できない部分)は落とし、何を落としたかを知らせてよい
`req~base-view.export-base-dropped~1` P1
- Acceptance:
  - Given: 表せない設定を持つビュー
  - When:
  - Then: 書き出しで落とした項目の一覧が出る。

#### `.base` への書き出しは、利用者が名前を決めた新しい `.base` に書き、既存の `.base` を書き換えないべき(base-view.base-untouched)
`req~base-view.export-base-new-file~1` P1
- Acceptance:
  - Given:
  - When: 既存の `.base` の名前を選ぶ
  - Then: 書かずに理由(読んで判定)。

#### mdgrid のビューの定義は、設定の置き場の `views.toml` の、開いた対象の実体のパスごとの `[[table]]` の項目(`path`)に、ビューの名前・列の並びと隠す列・ビューの設定を持つべき
`req~base-view.views-toml-layout~1` P1
- Acceptance:
  - Given:
  - When: ビューの設定は navigation.view-settings から navigation.settings-reset までのフィルター・並べ替え・グループ。シンボリックリンクで渡したフォルダでビュー「未完了」を保存する
  - Then: `views.toml` に `path` が実体のパスの `[[table]]` ができ、名前「未完了」・列の並び・隠す列・ビューの設定が入る。

#### `views.toml` の知らない項目は、警告にとどめるべき(cli.config-file)
`req~base-view.views-toml-unknown-items~1` P1
- Acceptance:
  - Given: 知らない項目 `color = 1`
  - When:
  - Then: 警告して読む。

#### 壊れた `views.toml` は、警告して読まずに起動するべき
`req~base-view.views-toml-broken~1` P1
- Acceptance:
  - Given:
  - When: `views.toml` を手で壊す
  - Then: 起動して警告、mdgrid のビューのタブは出ない。

#### `.base` で開いたときのタブは、`.base` のビューのあとに mdgrid のビューを並べるべき(起動時に開くのは base-view.view-switch のとおり、既定に選んだビューか先頭のビュー)
`req~base-view.views-toml-tab-order~1` P1
- Acceptance:
  - Given: `.base` と mdgrid のビューの両方がある対象
  - When:
  - Then: タブは `.base` のビューが先。

#### `views.toml` の保存は、一時ファイル → 名前の変更で書くべき
`req~base-view.views-toml-atomic-save~1` P1
- Acceptance:
  - Given:
  - When: ビューの保存で一時ファイルを書いている途中で止まる
  - Then: `views.toml` は前の中身のまま。

#### `--readonly`(cli.readonly)では、`views.toml` を書かないべき
`req~base-view.views-toml-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で保存
  - Then: 何も書かれず「読むだけ」。

#### 見た目の状態(screen.state-store)の列の幅と畳んだまとまりは、mdgrid のビューにも名前ごとに当てるべき
`req~base-view.views-toml-view-state~1` P1
- Acceptance:
  - Given:
  - When: mdgrid のビュー「全部」で title の列の幅を変え、「まとめ」で先頭のまとまりを畳む
  - Then: ほかのビューに移って戻っても、「全部」は同じ幅、「まとめ」は畳んだまま。

## 未確定事項

### `relative()` の言い回しと数え方
`tbd~relative-wording`
- Decider: 人
- Provisional: 言い回しは mdgrid の言語(日本語は「あと14日」「5日前」「今日」、英語は "in 14 days"・"5 days ago"・"today")。日付は今日から日の単位で数え、月と年は暦で数えて端数を切り捨てる。Obsidian の `relative()` の表示とは合わせていない。
- Basis: 2026-11-18 の話: あと何日・あと何か月を計算して読めるようにする。前の仕様では、言い回しが Obsidian の言語の設定に依るので未対応にしていた(specs/_history/decidespec/_changes/2026-10-06-common-methods.md の仮定)。
