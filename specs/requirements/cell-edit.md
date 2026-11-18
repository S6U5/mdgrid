# セルの編集とリストからの選択(cell-edit) の要求

## 目的

表のセルを選んで、その場で値を直す。直した値はすぐにはファイルに書かず、保存の操作までためておく([write-back](write-back.md) の write-back.pending-and-diff)。値の種類に合わせて、打つ・リストから選ぶ・切り替える・日付を入れる、の入り方を変える。キーの割り当ては [screen](screen.md)、行の選び方は [navigation](navigation.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/cell-edit/spec.md](../_history/decidespec/cell-edit/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 旧い CE-6(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-list-select.md))。移行先: cell-edit.list-items(リストの値を編集できるようにした)。
- 旧い CE-13(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-list-select.md))。移行先: cell-edit.list-items・cell-edit.list-layout(版1に入れた)。
- 旧い CE-14(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-new-note.md))。移行先: cell-edit.new-note・cell-edit.new-note-form(版1に入れた)。
- 採らない: 式(`formula.*`)の編集と、表計算のようなセル参照・数式。
- キーの名前の変更と削除は cell-edit.rename-delete-key(2026-10-06 に write-back.byte-range・write-back.frontmatter-only を変えて入れた)。列の追加は cell-edit.add-column。

## 前提(旧い仕様で AI が置いた仮定)

- 候補の数の既定 20 は AI の仮置き。使ってから見直す。
- Obsidian には列挙値から選ぶ公式の仕組みが無く、既存の値の候補(サジェスト)だけがある(2026-09-30 の調査)。cell-edit.value-list はその同じ材料で、少ないときはリストにする形。
- 一括の設定(cell-edit.bulk-set)と、編集中の Tab・Ctrl+R(cell-edit.revert-and-tab)は VisiData・lazysql の操作から採った。

## 要求

### 編集の入り方

#### セルの編集は、そのセルの位置に出す入力(入力ボックス・リスト)で行うべき
`req~cell-edit.inline-input~1` P1
- 受け入れ: 選んだセルで Enter → その場に入力ボックス。

#### セルの入力は、Enter で確定し、Esc で取り消すべき
`req~cell-edit.inline-enter-esc~1` P1
- 受け入れ: `title: 案` のセルの入力で `案2` と打つ → Enter なら確定し、Esc なら入力が閉じて `案` のまま(ためる変更なし)。

#### 確定したセルの値は、ためる変更になるべき(write-back.pending-and-diff)
`req~cell-edit.inline-pending~1` P1
- 受け入れ: セルの入力で値を打って Enter → 確定してためる変更になり、ファイルはまだ変わらない。

#### 入力の外をクリックしたら、確定を試みるべき
`req~cell-edit.inline-click-commit~1` P1
- 受け入れ: 入力ボックスで値を打って外をクリック → 確定してためる変更になる。

#### 入力の外をクリックしたとき値が不正なら、入力を閉じずにそのクリックを捨てるべき
`req~cell-edit.inline-click-invalid~1` P1
- 受け入れ: 日付の列の入力で `2026-02-30` と打って外の行をクリック → 入力は開いたままで、クリックした行は選ばれない。

#### 編集中に、編集前の値に戻す操作(Ctrl+R)と、確定して隣のセルへ移って編集を続ける操作(Tab / Shift+Tab)があるべき
`req~cell-edit.revert-and-tab~1` P1
- 受け入れ: Tab → 確定して右のセルの入力ボックスが開く。

#### 列の型は、設定の `[types]`(cell-edit.type-config)、ルートフォルダの型の設定(`.obsidian/types.json`)、Obsidian と同じ規則の推定(最初に見つかった空でない値)の順で決めるべき
`req~cell-edit.column-type-order~1` P1
- 受け入れ: types.json で `due` が date → 日付の入力になる。types.json で date と決めた列の日時の値は今までどおり `!`。

#### 推定で日付になった列に日時の値(`YYYY-MM-DDTHH:MM`)もあれば、その列を日時の列とするべき
`req~cell-edit.column-type-mixed-datetime~1` P1
- なぜ: 日付だけの値も日時の列に合うので、時刻を型の不一致として扱わないため
- 受け入れ: types.json の無い列で 2026-08-01 と 2026-08-03T08:15 が混ざる → 日時の列で、どちらも `!` が付かない。

#### 改行を含む値は、1行の入力では編集できないようにし、エディタ(screen.editor)で開くか読むだけにするべき
`req~cell-edit.multiline-guard~1` P1
- 受け入れ: ブロックスカラーの値のセルで Enter → 1行の入力は開かず、案内を表示。

#### 表の画面では、ビューの帯(screen.layout)の右の端にキーを添えた「+ 新規」(例「a + 新規」)を出し、クリックで新しいノートを作り始められるべき
`req~cell-edit.new-note-button~1` P1
- 受け入れ: ビューの帯を出した表の画面 → 帯の右の端に「a + 新規」が出て、クリックすると名前を打つ欄が出る。

#### ビューの帯を出さないとき(screen.display-options・screen.tabs-setting で、タブも検索の欄も出さないとき)は、「+ 新規」をヘッダーの右の端の画面の型のメニューの左に出すべき
`req~cell-edit.new-note-button-header~1` P1
- 受け入れ: タブも検索の欄も出さない設定 → ヘッダーの右の端の画面の型のメニューの左に「+ 新規」が出る。

#### 新しいノートは、キーの操作(screen.key-table のキーの表で割り当て直せる)とコマンドのパレットからも作り始められるべき
`req~cell-edit.new-note-start-keys~1` P1
- 受け入れ: 表のモードで新しいノートのキー(既定 `a`)→ 名前を打つ欄が出る。パレットで新しいノートを選んでも同じ欄が出る。

#### 新しいノートは、名前を打つ欄に名前を打って Enter で `<名前>.md` として作り、表のその行を選ぶべき
`req~cell-edit.new-note-name~1` P1
- 受け入れ: 名前を打つ欄で「会議の準備」と打って Enter → `会議の準備.md` が作られ、表にその行が選ばれて出る。

#### 新しいノートを作る場所は、開いたフォルダ(`.base` で開いたときはその `.base` のフォルダ)を既定とするべき
`req~cell-edit.new-note-place~1` P1
- 受け入れ: `仕事/tasks.base` で開いて「メモ」と打って Enter → `仕事/メモ.md` が作られる。

#### 新しいノートの名前に `下のフォルダ/名前` と書けば、下のフォルダに作り、そのフォルダが無ければ作るべき
`req~cell-edit.new-note-subfolder~1` P1
- 受け入れ: `プロジェクト/新しい` → `プロジェクト/新しい.md`(`プロジェクト` のフォルダが無ければ作る)。

#### 新しいノートを作る既定のフォルダは、設定で変えられるべき
`req~cell-edit.new-note-default-folder~1` P1
- 受け入れ: 設定で既定のフォルダを `inbox` にする → 「メモ」と打って Enter で `inbox/メモ.md` が作られる。

#### フォルダを2つ以上開いたとき(起動の引数に2つ以上のフォルダや `.base`)は、名前の欄の前に、作る場所を開いたフォルダの一覧から選ぶ欄を出すべき(一覧は起動の引数の順。既定は最初のフォルダ。Esc で取りやめ)
`req~cell-edit.new-note-folder-pick~1` P1
- 受け入れ: フォルダ A と B を開いて「+ 新規」→ A と B の一覧(A が選ばれている)。

#### 作る場所を選ぶ欄で選んだフォルダを、新しいノートを作る場所の根にするべき
`req~cell-edit.new-note-folder-root~1` P1
- 受け入れ: フォルダ A と B を開いて「+ 新規」→ B を選んで「メモ」→ `B/メモ.md` が作られ、その行が選ばれる。

#### フォルダを2つ以上開いたときも、「+ 新規」を出すべき
`req~cell-edit.new-note-folders-button~1` P1
- 受け入れ: フォルダ A と B を開く → ビューの帯に「+ 新規」が出る。

#### 開いたフォルダの外(`..` や絶対パス)を指す名前では、新しいノートを作らず理由を出すべき
`req~cell-edit.new-note-refuse-outside~1` P1
- 受け入れ: `../外` → 作らず理由。

#### 既にある名前では、新しいノートを作らず理由を出すべき
`req~cell-edit.new-note-refuse-existing~1` P1
- 受け入れ: `会議の準備.md` のあるフォルダで同じ名前「会議の準備」→ 作らず理由。

#### 関係マップの画面(relations.screen-menu)でも、「+ 新規」を表の画面と同じ位置にいつも出すべき
`req~cell-edit.new-note-relmap-button~1` P1
- なぜ: 画面ごとにボタンが出たり消えたりしないため
- 受け入れ: 関係マップの画面 → ヘッダーの下に帯の上の余白と同じ行の数を空け、その下のマップの窓の上の縁の右の端に「+ 新規」が重なって出る(表の画面のビューの帯の真ん中の行の右の端と同じ位置。関係マップにはビューの帯が無いため)。ビューの帯を出さないとき → 表の画面と同じくヘッダーの画面の型のメニューの左に出る。

#### 関係マップの画面で「+ 新規」を押すか同じキーの操作をしたら、関係マップで選んでいる表を開いて、その表で名前を打つ欄を出すべき
`req~cell-edit.new-note-relmap-start~1` P1
- 受け入れ: 関係マップで「タスク」の表を選んで「+ 新規」→「タスク」の表が開き、名前を打つ欄が出る。同じキーの操作(既定 `a`)でも同じ。

#### 読むだけの起動(`--readonly`)では、新しいノートを作らないべき
`req~cell-edit.new-note-readonly~1` P1
- 受け入れ: `--readonly` で起動して「+ 新規」かキーの操作 → 作らない(ファイルは増えない)。

#### 新しいノートの作成では新しいファイルだけを作り、既にあるファイルは書き換えないべき(write-back.frontmatter-only)
`req~cell-edit.new-note-no-overwrite~1` P1
- 受け入れ: 「会議の準備」を作る → 増えるのは `会議の準備.md` だけで、既にあるファイルは1バイトも変わらない。

#### ビューの絞り込みの条件のうち値が1つに決まるもの(`status == "todo"`・タグを含む など)は、その値をフロントマターに入れて新しいノートを作るべき
`req~cell-edit.new-note-filter-values~1` P1
- なぜ: 作った行がそのビューに残るようにするため
- 受け入れ: 「status が todo」のビューで「+ 新規」→「会議の準備」と打って Enter → `会議の準備.md` が `---\nstatus: todo\n---\n` で作られ、表にその行が選ばれて出る。

#### 新しいノートは、作る場所(cell-edit.new-note の作る場所を選ぶ欄があるとき)・名前・入力させる項目を、1つの窓(フォーム)に縦に並べて作るべき
`req~cell-edit.new-note-form-fields~1` P1
- 受け入れ: フォルダ A と B を開き、設定に ask = [priority] で「+ 新規」→ 1つの窓に 作る場所・名前・priority の欄が縦に並ぶ。

#### 新しいノートの窓で入力させる項目は、設定の `ask` があればその列、無ければ今の表で見えている列(`file.*`・`formula.*` を除く)とするべき
`req~cell-edit.new-note-form-ask~1` P1
- 受け入れ: 設定に ask = [priority] → 窓に 名前・priority の欄が並ぶ。ask の無い設定 → 見えている列が欄に並ぶ。

#### 新しいノートの窓で、前もって入れる値(cell-edit.new-note の絞り込みの値・設定の値)を入れた欄には、出どころを示すべき
`req~cell-edit.new-note-form-source~1` P1
- 受け入れ: 設定に ask = [priority]、set に tags = [inbox] → 窓に 名前・priority・tags の欄が並び、tags に「設定から」。

#### 新しいノートの窓を開くと名前の欄の入力が開き、Enter・Tab で今の欄を確定して次の欄、Shift+Tab で前の欄に移るべき
`req~cell-edit.new-note-form-move~1` P1
- 受け入れ: 設定に ask = [priority] で窓を開く → 名前の欄の入力が開いている。名前を打って Enter か Tab → priority の欄へ、Shift+Tab → 名前の欄へ戻る。

#### 新しいノートの窓の各欄は、型に合った入力(cell-edit.column-type)で直すべき
`req~cell-edit.new-note-form-typed~1` P1
- 受け入れ: ask = [priority, due](priority が数・due が日付の列)→ priority の欄で `12a` は閉じずに不正を表示し、due の欄はカレンダーつきの日付の入力で開く。

#### 新しいノートの窓は、Ctrl+S(どの欄からでも)と最後の欄の Enter でノートを作るべき
`req~cell-edit.new-note-form-create~1` P1
- 受け入れ: 名前の欄で「メモ」と打って Ctrl+S → `メモ.md` が作られる。最後の欄で Enter でも作られる。

#### 新しいノートの窓は、Esc でノートを作らずにやめるべき
`req~cell-edit.new-note-form-esc~1` P1
- 受け入れ: 名前を打ってから Esc → 窓が閉じ、ノートは作られない。

#### 新しいノートの窓の空の欄は、ノートに書かないべき
`req~cell-edit.new-note-form-skip-empty~1` P1
- 受け入れ: 設定に ask = [priority]、set に tags = [inbox] → priority を空のまま名前を打って Ctrl+S → priority の行が無く `tags: [inbox]`(縦の形)のノート。

#### 新しいノートの窓で作れない名前のときは、作らずに名前の欄に理由を示すべき
`req~cell-edit.new-note-form-bad-name~1` P1
- 受け入れ: 既にある名前で Ctrl+S → 名前の欄に理由、作らない。

#### 新しいノートの窓で必須の欄(cell-edit.new-note-config)が空のときは、作らずにその欄に理由を示すべき
`req~cell-edit.new-note-form-required~1` P1
- 受け入れ: `required = ["due"]` で due を空のまま Ctrl+S → 作らずに due の欄に理由。

#### 新しいノートの窓の設定は、ビューごとにも決められてよい(mdgrid のビュー、base-view.own-views)
`req~cell-edit.new-note-form-per-view~1` P1
- 受け入れ: 設定に ask = [priority]、あるビューの `[table.view.new_note]` に ask = [due] → そのビューの窓は 名前・due の欄、ほかのビューは 名前・priority の欄。

#### 新しいノートを作る既定のキーは、表のモードの `a` としてよい
`req~cell-edit.new-note-config-key~1` P1
- なぜ: `n` は検索の次の一致に使うため
- 受け入れ: 既定で `a` → 名前の欄。

#### 新しいノートを作る動作の名前は `new_note` としてよい
`req~cell-edit.new-note-config-action~1` P1
- 受け入れ: `[keys.table] "n" = "new_note"` で `n` でも作れる。

#### 設定の `[new_note]` の表に、`folder`(既定のフォルダ。開いたフォルダからの相対)を持ってよい
`req~cell-edit.new-note-config-folder~1` P1
- 受け入れ: 開いたフォルダ `notes` で `[new_note] folder = "inbox"` → 「メモ」は `notes/inbox/メモ.md` に作られる。

#### 設定の `[new_note]` の表に、`name`(名前の雛形。`{date}` で今日の日付)を持ってよい
`req~cell-edit.new-note-config-name~1` P1
- 受け入れ: 今日が 2026-10-02 のとき `name = "{date} "` → 名前の欄に `2026-10-02 ` が入って出る。

#### 設定の `[new_note]` の表に、`ask`(窓に出す列の並び)を持ってよい
`req~cell-edit.new-note-config-ask~1` P1
- 受け入れ: `ask = ["priority", "due"]` → 窓に 名前・priority・due の順で欄が並ぶ。

#### 設定の `[new_note]` の表に、`required`(空では作らない列)を持ってよい
`req~cell-edit.new-note-config-required~1` P1
- 受け入れ: `required = ["due"]` で due を空にして Ctrl+S → 作らずに理由。

#### 設定の `[new_note]` の表に、`hidden`(窓に出さずに値だけ入れる列)を持ってよい
`req~cell-edit.new-note-config-hidden~1` P1
- 受け入れ: `hidden = ["status"]`・`[new_note.set] status = "todo"` → 窓に status の欄は出ず、作ったノートに `status: todo`。

#### 設定の `[new_note.set]` に、前もって入れる値(列 = 値)を持ってよい
`req~cell-edit.new-note-config-set~1` P1
- 受け入れ: 絞り込みの無いビューで `[new_note.set] status = "doing"` → 窓の status に doing が入り、作ったノートに `status: doing`。

#### `[new_note.set]` の文字の値には、`{date}`(今日)・`{date+N}`(N 日後。`{date-N}` は前)・`{now}`(今の日時)を書いてよい
`req~cell-edit.new-note-config-set-vars~1` P1
- 受け入れ: `[new_note.set] due = "{date+7}"` → 窓の due に7日後の日付。

#### mdgrid のビュー(views.toml)の `[table.view.new_note]` に `[new_note]` と同じ形で書けば、そのビューではそちらを使ってよい
`req~cell-edit.new-note-config-view~1` P1
- 受け入れ: 設定に `[new_note] folder = "inbox"`、あるビューの `[table.view.new_note]` に `folder = "done"` → そのビューで作ると `done/` に、ほかのビューでは `inbox/` に作られる。

#### 前もって入れる値は、同じ列の設定の値より絞り込みの値を先にしてよい
`req~cell-edit.new-note-config-filter-first~1` P1
- なぜ: 作った行が絞り込みに残るようにするため
- 受け入れ: 「status が todo」のビューで `[new_note.set] status = "doing"` → 作ったノートは `status: todo`。

### 型ごとの入り方

#### テキストの列で、読み込んだノート全部(base-view.root-folder の範囲。ビューの絞り込みの前)の同じキーに現れる異なる値が一定数以下(既定 20)なら、その値のリストから選べるようにするべき
`req~cell-edit.value-list-open~1` P1
- 受け入れ: status の値 `doing` のセルで Enter → リスト。ビューの絞り込みで見えていないノートの値も候補に入る。異なる値が 21 個の列 → リストを出さず1行の入力。

#### 値のリストからは、自由入力にも切り替えられるべき
`req~cell-edit.value-list-free-input~1` P1
- 受け入れ: status の値 `doing` のセルのリストで `blocked` と打って Enter → `blocked` のためる変更。

#### 値のリストでは、今の値に印を付けるべき
`req~cell-edit.value-list-mark~1` P1
- 受け入れ: `status` の値が todo・doing・done だけで今の値が doing → リストが出て、今の値の doing に印。

#### 値のリストには、空の値を「なし」として並べるべき
`req~cell-edit.value-list-none~1` P1
- 受け入れ: `status` の値が todo・doing・done だけ → 3つと「なし」のリストが出る。

#### 値のリストを開いた直後(リストで選び直したあとも)に文字を打ったときは、今の値を消して打った文字から自由入力を始めるべき
`req~cell-edit.value-list-type-replaces~1` P1
- なぜ: 候補に無い新しい値を打つのに、今の値が邪魔をしないため
- 受け入れ: status の値 `doing` のセルのリストで ↓ のあと `x` と打つ → `x`。← を押してから `!` と打つ → `doing!`。Ctrl+R のあとに打つ → 今の値の続き。

#### 自由入力で打っている間は、候補のうち打った文字を含むもの(大文字小文字を問わない)を、選んでいない形でリストに出し続けるべき
`req~cell-edit.value-list-narrow~1` P1
- 受け入れ: `do` と打つ → doing と done だけの候補が選びなしで出る(`DO` でも同じ)。どの候補にも当たらない文字 → 候補は出ない。

#### 自由入力で打っている間は、↑↓ で絞り込んだ候補の中から選べるべき(打った文字のまま Enter なら打った文字を書く)
`req~cell-edit.value-list-narrow-pick~1` P1
- 受け入れ: `do` のあと ↓ → doing を選び Enter → doing。`do` と打ったまま Enter → `do`。

#### チェックボックスの列は、1回の操作で、真と偽を切り替えるべき
`req~cell-edit.checkbox-toggle~1` P1
- なぜ: Obsidian のチェックボックスと同じ手触りにするため
- 受け入れ: `done: false` のセルで Enter → `done: true`。`done: true` で Enter → `done: false`。全部の行を選んで Enter → 今の行の次の状態(true の次は false、ほかは true)が全部の行に入る(cell-edit.bulk-set)。

#### チェックボックスの列で、値が空かキーの無いノートを切り替えたら真にするべき
`req~cell-edit.checkbox-empty-true~1` P1
- 受け入れ: キーの無いノートで Enter → `done: true`、もう一度 → `done: false`。`done:` の空のセルで Enter → `done: true`。

#### チェックボックスの列を空(cell-edit.empty-is-null と同じく `key:`)にするのは、空にする操作(BS・Delete)だけにするべき
`req~cell-edit.checkbox-clear~1` P1
- 受け入れ: `done: true` のセルで BS → `done:`。

#### 日付の列は、`YYYY-MM-DD` と、今日からの日数(`+3`)と、空(値を消す。キーは残して null と書く。cell-edit.empty-is-null)を受け付けるべき
`req~cell-edit.date-input-forms~1` P1
- 受け入れ: due のセルで `2026-11-03` → `due: 2026-11-03`、`+3` → 今日の3日後、空で確定 → `due:` の行が残る。

#### 日付の列は、実在しない日付を受け付けるべきでない
`req~cell-edit.date-input-invalid~1` P1
- 受け入れ: `2026-02-30` → 閉じずに不正を表示。

#### 日付の入力を開いた直後に文字を打ったときは、今の日付の続きでなく打った文字から入力を始めるべき
`req~cell-edit.date-input-type-replaces~1` P1
- なぜ: `+3` が今日からの日数として読めるようにするため
- 受け入れ: `due: 2026-10-05` のセルで Enter のあと `+3` → 今日の3日後。

#### 日付の列のセルを編集するときは、入力ボックスに加えてカレンダーを出し、カレンダーで日を選んで決めても、入力ボックスに打ち込んでも(cell-edit.date-input の形)値を入れられるべき
`req~cell-edit.calendar~1` P1
- 受け入れ: due のセルで Enter → 入力ボックスの下に今の値の月のカレンダーが出て、今の値の日が選ばれている。→ で次の日 → Enter → その日がためる変更。`2026-11-03` と打って Enter → 打った日がためる変更。打っている間はカレンダーが打った日の月に動く。

#### カレンダーは、←→で1日、↑↓で1週、PageUp・PageDown で1か月動かすべき
`req~cell-edit.calendar-keys-move~1` P1
- 受け入れ: 2026-10-30 で ↓ → 2026-11-06。PageDown → 翌月の同じ日(無ければ月末)。

#### カレンダーは、今日に戻す操作を持つべき
`req~cell-edit.calendar-keys-today~1` P1
- 受け入れ: 2026-10-30 のセルのカレンダーで今日に戻す → 今日の日。

#### カレンダーは、空にする操作(cell-edit.date-input の空と同じ)を持つべき
`req~cell-edit.calendar-keys-clear~1` P1
- 受け入れ: due のセルのカレンダーで空にする → null のためる変更。

#### カレンダーは、今日・元の値・選んでいる日を、色のほかに印でも見分けられるようにするべき(cell-view.not-color-only)
`req~cell-edit.calendar-keys-marks~1` P1
- 受け入れ: 今日が 2026-10-15 で、元の値 2026-10-30 のセルのカレンダーで 2026-10-31 を選ぶ → 色を除いた画面の文字だけで、今日・元の値・選んでいる日の3つが違う印で見分けられる。

#### カレンダーの週の始まりは既定で日曜とし、設定で月曜にできてよい
`req~cell-edit.calendar-keys-week-start~1` P1
- 受け入れ: 設定が無い → 1列目が日。設定で月曜始まり → 1列目が月。

#### 値が空か型の合わない値(cell-view.type-mismatch)のセルでは、カレンダーに今日の月を出すべき
`req~cell-edit.calendar-keys-empty-month~1` P1
- 受け入れ: `someday` のセル → 今日の月が出る。

#### 日時の列では、カレンダーで日を選んでも時刻は元の値を保つべき(元に時刻が無ければ、時刻は打ち込みで入れる)
`req~cell-edit.calendar-keys-keep-time~1` P1
- 受け入れ: `2026-10-30T09:00` の日時のセルで → → `2026-10-31T09:00`。

#### カレンダーは、画面に収まらないときは出さず、打ち込みだけにするべき(cell-view.grapheme-width)
`req~cell-edit.calendar-keys-too-small~1` P1
- 受け入れ: 幅 30 の端末 → カレンダーが出ず、打ち込みはできる。

#### 日付の表での見せ方と打ち込みの形は、設定(cli.config-file の設定ファイル)の日付の形(区切りが `-` か `/` か、年を出すかなど)で選べるべき
`req~cell-edit.date-format-display~1` P1
- 受け入れ: 設定で `date_format = "YYYY/MM/DD"` → 表の due が `2026/10/05` と出る。設定が無い → 今までどおり `2026-10-05`。読めない形の設定 → 起動して警告し、既定の形を使う。

#### 日付の打ち込みは、設定の日付の形と `YYYY-MM-DD` の両方を受け付けるべき
`req~cell-edit.date-format-input~1` P1
- 受け入れ: 設定で `date_format = "YYYY/MM/DD"` → `2026/11/03` と打っても `2026-11-03` と打っても、閉じずに確定できる(不正を表示しない)。

#### ノートに書く日付の形は、Obsidian が日付と見なす `YYYY-MM-DD` のまま変えないべき
`req~cell-edit.date-format-write~1` P1
- なぜ: ほかの形で書くと Obsidian で不正な値になるため
- 受け入れ: 設定で `date_format = "YYYY/MM/DD"` → `2026/11/03` と打っても `2026-11-03` と打っても、ファイルには `due: 2026-11-03` と書かれる。

#### カレンダーは、PageUp・PageDown のほかに Shift+←・Shift+→ でも1か月動かせるべき
`req~cell-edit.calendar-month-keys-shift~1` P1
- なぜ: PageUp・PageDown の無いキーボードでも月を飛ばせるようにするため
- 受け入れ: 2026-10-30 のセルでカレンダーを出し Shift+→ → 2026-11-30。2026-10-30 から Shift+← を2回 → 2026-08-30。2026-03-31 で Shift+← → 2026-02-28(無ければ月末)。カレンダーが出ていないときは文字のカーソルが動くだけ。

#### 月を動かすキーを、カレンダーの下の縁に示すべき
`req~cell-edit.calendar-month-keys-hint~1` P1
- なぜ: PageUp・PageDown の無いキーボードでも、月を動かすキーに気づけるようにするため
- 受け入れ: 日付のセルでカレンダーを出す → カレンダーの下の縁に `Shift+←→ 月` の文字。

#### 月を動かすキーは、設定の割り当て直し(screen.key-table)で変えられてよい
`req~cell-edit.calendar-month-keys-rebind~1` P1
- 受け入れ: キーの表で翌月へ動かす操作を別のキーに割り当て直す → 2026-10-30 のカレンダーでそのキー → 2026-11-30。

#### カレンダーは、Shift+↑・Shift+↓ で1年動かしてよく、そのキーをカレンダーの下の縁に示してよい
`req~cell-edit.calendar-year-keys~1` P1
- 受け入れ: 2026-10-30 で Shift+↓ → 2027-10-30。2028-02-29 で Shift+↓ → 2029-02-28。

#### 数値の列は、数として読める値だけを受け付けるべき
`req~cell-edit.number-only~1` P1
- 受け入れ: `12a` → 閉じずに不正を表示。

#### 数値の入力を開いた直後に文字を打ったときは、今の数の続きでなく打った文字から入力を始めるべき
`req~cell-edit.number-type-replaces~1` P1
- 受け入れ: `priority: 1` のセルで Enter のあと `5` → 5(15 ではない)。

#### 値を空にしたときは、Obsidian と同じく `key:`(null)と書き、キーは消さないべき
`req~cell-edit.empty-is-null~1` P1
- 受け入れ: `status: done` のセルを空にして保存 → `status:` の行が残る。

#### リストの値の列(tags に限らず、列の型がリストの列)は、表から要素を付け外しできるべき
`req~cell-edit.list-items-toggle~1` P1
- 受け入れ: tags のセルで 会議 を付けて確定 → tags に 会議 が足されたためる変更。会議 を外して確定 → tags から 会議 が消えたためる変更。

#### リストの要素の候補は、読み込んだノート全部(base-view.root-folder の範囲)のその列に今ある要素とするべき
`req~cell-edit.list-items-candidates~1` P1
- 受け入れ: tags のセルで Enter → ルートフォルダの中のタグの一覧が出る。

#### リストの要素の候補は、打つたびに絞り込んで出すべき
`req~cell-edit.list-items-narrow~1` P1
- 受け入れ: tags のセルで「会」と打つ → 会議・会計 だけに絞られる。

#### リストの値の列には、候補に無い要素も打って足せるべき
`req~cell-edit.list-items-new~1` P1
- 受け入れ: 一覧に無い 新規 を打って足す → 新規 が足される。

#### 選んだ複数の行(navigation.select-rows)のリストの値の列に、要素を一括で足す・外すことができ、各行のほかの要素は残すべきで、一部の行だけが持つ要素は、その旨の印で示すべき
`req~cell-edit.list-bulk~1` P1
- 受け入れ: 3行を選び、tags に 会議 を足す → 3行とも 会議 が足され、元のタグは残る。1行だけが持つ 本 は「一部」の印。本 を外す → その1行から 本 が消える。

#### 要素を足し引きして書き戻すときは、元の書き方(1行の `[a, b]` か、複数行の `- a`)と字下げを保つべきで、キーが無い・値が空のノートに初めて書くときは、Obsidian と同じ複数行の形(`key:` の次の行から `  - a`)で書くべき
`req~cell-edit.list-layout~1` P1
- 受け入れ: `tags: [a, b]` に c を足す → `tags: [a, b, c]`。字下げ2の `- a` の縦のリストに c を足す → 同じ字下げの `- c` が1行増え、ほかの行は1バイトも変わらない。tags の無いノートに a を足す → `tags:` と `  - a` の2行が足される。

#### リストの値の付け外しでは、候補を件数の多い順(同じ件数は文字の順)に並べるべき
`req~cell-edit.list-candidates-order~1` P1
- 受け入れ: 会議 が5件・本 が2件 → 会議 が先。

#### リストの値の付け外しでは、今の値に付いている要素に印を付けるべき
`req~cell-edit.list-candidates-mark~1` P1
- 受け入れ: `tags: [会議]` のセルで Enter → 一覧の 会議 に印が付き、本 には付かない。

#### リストの要素の候補の絞り込みは、大文字小文字を区別しない部分一致にするべき
`req~cell-edit.list-candidates-match~1` P1
- 受け入れ: 候補に Meeting・notebook・会議 → `MEET` と打つ → Meeting だけに絞られる。

#### リストの値の付け外しでは、同じ要素を2つ足さないべき
`req~cell-edit.list-candidates-no-dup~1` P1
- 受け入れ: 会議 を持つ行で 会議 を打って足す → 既にあれば足さない(会議 は1つのまま)。

#### リストの要素を全部外したら、cell-edit.empty-is-null と同じく `key:` と書いてキーは消さないべき
`req~cell-edit.list-candidates-empty~1` P1
- 受け入れ: 全部を外して保存 → `tags:` の行が残る。

#### tags の列では、打った先頭の `#` を除いて扱うべき
`req~cell-edit.list-candidates-hash~1` P1
- 受け入れ: `#会議` と打つ → 会議 として扱う。

#### リストの列で1つの文字列で書かれた値(`tags: x`)は、1つの要素のリストとして開くべき
`req~cell-edit.list-candidates-scalar-open~1` P1
- 受け入れ: `tags: x` のセルで Enter → x に印の付いた要素の一覧が開く。

#### 1つの文字列で書かれたリストの値は、確定したらフローのリスト(`tags: [x, y]`)に書き換えるべき
`req~cell-edit.list-candidates-scalar-write~1` P1
- 受け入れ: `tags: x` で y を足して保存 → `tags: [x, y]`、行のほかのバイトはそのまま。

#### cell-edit.read-only-forms の読むだけの形(複数行にまたがるフローのリスト、アンカーや別名、入れ子の要素)のリストは、開かずに形ごとの理由を出すべき
`req~cell-edit.list-candidates-read-only~1` P1
- 受け入れ: `[a,\n b]` のセルで Enter → 開かずに理由。

#### 元の書き方を保って書けない形のリストは、開かずに形ごとの理由を出すべき(要素の間の空行やコメント、要素の行末のコメント、字下げの違う要素、`-` のあとのタブ、文字列でない要素や値(書くと型が変わる)、ブロックの文字で書かれた値)
`req~cell-edit.list-candidates-unkeepable~1` P1
- 受け入れ: 要素の間にコメントのある縦のリスト・`- 2024` を含むリスト・`tags: 2024` → どれも開かずに、形ごとの理由。

#### リストの要素ごとのクオートは、write-back.quoting に従うべき
`req~cell-edit.list-candidates-quoting~1` P1
- 受け入れ: `no` を足す → `"no"` と書かれる。

### 読むだけのセルと一括の編集

#### ファイルの属性(`file.*`)・式の列(`formula.*`)・ネストした map・ブロックスカラー・アンカーや別名を使った値・タグつきの値(`!!str` など)・複数行にまたがる値(素のスカラー・引用符つきの値・フロー形式)のセルは、編集できないようにし、なぜ読むだけかを表示するべき
`req~cell-edit.read-only-forms~1` P0
- 受け入れ: `file.mtime` の列で Enter → 読むだけと理由を表示。`title: !!str 12` と2行にまたがる `[a,\n b]` のセル → どちらも読むだけと理由。

#### 選んだ複数の行の同じ列に、同じ値を、ためる変更としてまとめて入れられるべき
`req~cell-edit.bulk-set-same-value~1` P1
- 受け入れ: 3行を選び status に done → 3つのためる変更、保存前の差分に3ファイル。チェックボックスの列で全部を選んで切り替え → 今の行の次の状態(true の次は false、ほかは true)を全部の行にそろえ、元と同じになる行は外す(write-back.revert-drops-change)。

#### まとめて入れるときは、その列のセルを直せない行(読むだけのノート(write-back.refuse-unsafe-notes)と、読むだけのセル(cell-edit.read-only-forms・cell-edit.multiline-guard))を飛ばし、飛ばした数と理由を知らせるべき
`req~cell-edit.bulk-set-skip-read-only~1` P1
- なぜ: 元の値を黙って消さないため
- 受け入れ: 3行のうち1行が読めないフロントマターのノート(write-back.refuse-unsafe-notes) → 2つのためる変更と「1行を飛ばした(理由)」。フロントマターの無いノートは飛ばさず、ためる変更になる。

#### チェックボックスの列の一括の切り替え(cell-edit.checkbox)では、ためた変更が無く値の型が合わない行(cell-view.type-mismatch。1行なら入力を開く行)を飛ばし、飛ばした数と理由を知らせるべき
`req~cell-edit.bulk-set-skip-mismatch~1` P1
- なぜ: 元の値を黙って消さないため
- 受け入れ: a `done: true`・b `done: yes` を選んで切り替え → a だけ `done: false`、b はそのままで「1行を飛ばした(型が合わない)」。

### 旧い仕様では次の段の表にあり、作ってテストのあるもの

#### 表のモードで、まだどのノートにも無いキーの列を、名前を打って表に足せるようにするべき(キー(既定 `A`。screen.key-table のキーの表で割り当て直せる)・パレット・操作の一覧(screen.action-menu の列の節)から)
`req~cell-edit.add-column-start~1` P1
- 受け入れ: 表のモードで `A` → 列の名前を打つ欄が出る。パレットと操作の一覧の列の節からも同じ欄が出る。

#### 足した列は、空のセルの列として表の右に出すべき
`req~cell-edit.add-column-empty~1` P1
- 受け入れ: `A` → `reviewer` と打って Enter → 右に `reviewer` の空の列。

#### 足した列は、セルに値を入れて保存したノートにだけキーの行を足すべき(write-back.byte-range・write-back.add-key-and-frontmatter。値を入れないノートは変えない)
`req~cell-edit.add-column-write~1` P1
- 受け入れ: 足した `reviewer` の列の1行に `ken` を入れて保存 → そのノートにだけ `reviewer: ken` の行。ほかのノートは1バイトも変わらない。

#### 足した列は起動の間だけ残し、ノートに値が入れば普通の列になるべき
`req~cell-edit.add-column-session~1` P1
- 受け入れ: `reviewer` を足して値を入れずに終了し、起動し直す → `reviewer` の列は無い。1行に `ken` を入れて保存してから起動し直す → `reviewer` は普通の列として出る。

#### 足す列の名前が空・`file.`・`formula.` で始まる・改行を含むときは、足さずに理由を出すべき
`req~cell-edit.add-column-bad-name~1` P1
- 受け入れ: `file.x` → 理由が出て足さない。空の名前でも同じ。

#### 足す列の名前がもうある列の名前なら、その列に移るべき
`req~cell-edit.add-column-existing~1` P1
- 受け入れ: `status` の列がある表で `A` → `status` と打って Enter → 列は増えず、status の列に移る。

#### 読むだけの起動(cli.readonly)では、列を足さないべき
`req~cell-edit.add-column-readonly~1` P1
- 受け入れ: `--readonly` で起動して `A` → 列は足されない。

#### 表のモードで、列のキーの名前を、読み込んだ全部のノート(base-view.root-folder の範囲。ビューの絞り込みの前)に対して、新しい名前を打ってためる変更としてまとめて変えられるべき(操作の一覧(screen.action-menu)の列の節とパレットから。キーは既定で割り当てない)
`req~cell-edit.rename-delete-key-rename~1` P1
- 受け入れ: status の列で「名前を変える」→ `state` と打つ → status を持つ全部のノートに、キーの名前の変更のためる変更。

#### 表のモードで、列のキーを、読み込んだ全部のノート(base-view.root-folder の範囲。ビューの絞り込みの前)から、ためる変更としてまとめて消せるべき(操作の一覧(screen.action-menu)の列の節とパレットから。キーは既定で割り当てない)
`req~cell-edit.rename-delete-key-delete~1` P1
- 受け入れ: status を持つノートが3個の列で「キーを消す」→ 確かめて y → 3つのノートから status の行を消すためる変更。

#### キーの名前の変更では、もうその名前のキーを持つノートを飛ばして、数と理由を知らせるべき
`req~cell-edit.rename-delete-key-skip-existing~1` P0
- 受け入れ: status の名前を `state` に変える → `state` を既に持つノートは飛ばして「1行を飛ばした(その名前のキーがある)」。

#### キーの名前の変更では、キーを書き換えられないノート(読むだけのノート(write-back.refuse-unsafe-notes)・読むだけの値(cell-edit.read-only-forms))を飛ばして、数と理由を知らせるべき
`req~cell-edit.rename-delete-key-skip-unsafe~1` P0
- 受け入れ: status を持つノートの1つが `chmod 444` のノート(write-back.refuse-unsafe-notes)で、status の名前を `state` に変える → そのノートは飛ばして「1行を飛ばした(理由)」。

#### キーの削除は、数を示して確かめてから(`y`)行うべき
`req~cell-edit.rename-delete-key-confirm~1` P0
- 受け入れ: 「キーを消す」→「3個のノートから消す? y」→ y で3つのためる変更、y 以外ではためる変更は0。

#### キーの名前の変更と削除は、どちらも1手で取り消せるべき
`req~cell-edit.rename-delete-key-undo~1` P0
- 受け入れ: キーの名前の変更で3つのためる変更ができたあと `u` → 3つとも戻る(`u` で全部を戻す)。

#### キーの名前の変更と削除は、保存の前の差分(write-back.pending-and-diff)で全部のファイルを見られるべき
`req~cell-edit.rename-delete-key-diff~1` P0
- 受け入れ: status の名前を `state` に変えて Ctrl+S → 差分に各ファイルの `-status: todo` と `+state: todo`。

#### キーの名前の変更と削除は、`file.*`・`formula.*` の列と、読むだけの起動(cli.readonly)では出さないべき
`req~cell-edit.rename-delete-key-hidden~1` P1
- 受け入れ: `file.mtime` の列の操作の一覧と、`--readonly` の起動 → 「名前を変える」と「キーを消す」が出ない。

#### 日時の列のカレンダーは時刻の欄を持ち、日と時刻のどちらを選んでいるかを切り替えられるべき
`req~cell-edit.datetime-calendar-time-field~1` P1
- 受け入れ: `at: 2026-10-30T09:00` のセルで Enter → 時刻の欄に 09:00。時刻に切り替えると ↑ が時刻を動かし、日に戻すと → が日を動かす。

#### カレンダーで時刻を選んでいるときは、↑↓ で15分、Shift+↑↓ で1時間動かすべき
`req~cell-edit.datetime-calendar-time-step~1` P1
- 受け入れ: `at: 2026-10-30T09:00` のセルで時刻に切り替えて ↑ → 09:15、Shift+↑ → 10:15、確定 → `2026-10-30T10:15`。

#### カレンダーで時刻を動かしたら、入力ボックスの文字もその時刻にするべき
`req~cell-edit.datetime-calendar-time-text~1` P1
- 受け入れ: `at: 2026-10-30T09:00` のセルで時刻に切り替えて ↑ → 入力ボックスの文字が `2026-10-30T09:15`。

#### 時刻の無い値では、カレンダーの時刻の欄を空で始めるべき
`req~cell-edit.datetime-calendar-empty-time~1` P1
- 受け入れ: 日時の列の `at: 2026-10-30` のセルで Enter → 時刻の欄は空。

#### 時刻の無い値で時刻を動かさずに確定したら、日だけを選んだときと同じく時刻を書かないべき
`req~cell-edit.datetime-calendar-keep-date-only~1` P1
- 受け入れ: `at: 2026-10-30` で時刻を動かさずに確定 → `2026-10-30` のまま。

#### 日時の列の時刻は、打ち込みでも今までどおり入れられるべき
`req~cell-edit.datetime-calendar-typing~1` P1
- 受け入れ: `at: 2026-10-30` のセルで `2026-10-30T18:30` と打って Enter → `2026-10-30T18:30` のためる変更。

#### カレンダーの時刻の欄への切り替えは、既定で Ctrl+O にするべき(Ctrl+T は今日に戻す。割り当て直せる)
`req~cell-edit.time-keys-switch~1` P1
- 受け入れ: `at: 2026-10-30T09:00` のカレンダーで Ctrl+O のあと ↑ → `2026-10-30T09:15`(日でなく時刻が動く)。

#### 時刻の欄へ切り替えるキーを、カレンダーの下の縁に出すべき
`req~cell-edit.time-keys-hint~1` P1
- 受け入れ: 日時の列のセルでカレンダーを出す → カレンダーの下の縁に Ctrl+O の文字。

#### 時刻の無い値で最初に時刻を動かすときは、09:00 から動かすべき
`req~cell-edit.time-keys-first-time~1` P1
- 受け入れ: `2026-10-30` で Ctrl+O・↑ → `2026-10-30T09:15`。

#### カレンダーの時刻の ↑↓ は、15分の刻みにそろえるべき(09:07 で ↑ → 09:15)
`req~cell-edit.time-keys-snap~1` P1
- 受け入れ: `2026-10-30T09:07` で時刻に切り替えて ↑ → `2026-10-30T09:15`。

#### カレンダーで時刻が0時をまたいでも、日は変えないべき
`req~cell-edit.time-keys-midnight~1` P1
- 受け入れ: `2026-10-30T23:45` で時刻に切り替えて ↑ → 00:00、日は 30 のまま。

#### 秒やタイムゾーンの付いた時刻は、カレンダーで時と分だけを変えて残りを保つべき
`req~cell-edit.time-keys-seconds-tz~1` P1
- 受け入れ: `2026-10-30T09:00:30+09:00` で ↑ → `2026-10-30T09:15:30+09:00`。

#### 一括の編集で時刻を動かしたら、打ち込んだ時刻と同じく全部の行にその時刻を入れるべき(cell-edit.bulk-set)
`req~cell-edit.time-keys-bulk~1` P1
- 受け入れ: 日時の列で2行を選んで一括の編集を開き、`2026-10-30` から Ctrl+O・↑ で確定 → 2行とも `2026-10-30T09:15` のためる変更。

#### 新しいノートの名前の雛形と前もって入れる値(cell-edit.new-note-config)には、変数(`{date}`・`{date:形}`・`{date+N}`・`{date-N}`・`{time}`・`{now}`・`{weekday}`・`{name}`・`{folder}`)を書けるべき
`req~cell-edit.template-vars-list~1` P1
- 受け入れ: 今が 2026-10-07 20:15、窓で決めた名前が 打ち合わせ、作る場所のフォルダが Tasks → `{date}` は 2026-10-07(今日)、`{date:YYYY/MM/DD}` は 2026/10/07(cell-edit.date-format の日付の形)、`{date+7}` は 2026-10-14、`{date-1}` は 2026-10-06(N 日後・前)、`{time}` は 20:15(今の時刻 HH:MM)、`{now}` は 2026-10-07T20:15(今の日時)、`{weekday}` はその日の曜日、`{name}` は 打ち合わせ、`{folder}` は Tasks。

#### `[new_note] body` に雛形のファイルのパス(開いたフォルダからの相対)を書けば、その中身の変数を埋めて新しいノートの本文に書くべき
`req~cell-edit.template-vars-body~1` P1
- 受け入れ: `body = "templates/task.md"` の中身 `# {name}\n作成: {date}` → 本文が `# 打ち合わせ\n作成: 2026-10-07`。

#### 窓に出さない欄(`hidden`)の値も変数を埋めて書き、作成日などを作るたびに自動で入れられるべき
`req~cell-edit.template-vars-hidden~1` P1
- 受け入れ: `hidden = ["created"]`・`set` に `created = "{now}"` → 作ったノートに `created: 2026-10-07T20:15`(今の日時)。

#### 雛形の知らない変数は、文字のまま残すべき
`req~cell-edit.template-vars-unknown~1` P1
- 受け入れ: `{unknown}` はそのまま。名前の雛形 `{unknown}` → 名前の欄に `{unknown}` と出る。

#### 新しいノートの作り方は、設定の `[new_note] mode` で `form`(既定。cell-edit.new-note-form の窓)か `editor` を選べるべき
`req~cell-edit.new-note-mode-select~1` P1
- 受け入れ: mode を書かない → `a` で cell-edit.new-note-form の窓。`mode = "editor"` → `a` で名前だけの窓。

#### 新しいノートの作り方は、ビューごとにも決められてよい(base-view.own-views)
`req~cell-edit.new-note-mode-per-view~1` P1
- 受け入れ: 設定に `[new_note] mode = "form"`、あるビューの `[table.view.new_note]` に `mode = "editor"` → そのビューで `a` → 名前だけの窓、ほかのビューではフォーム。

#### `editor` の作り方では、作る場所(cell-edit.new-note)と名前だけを聞くべき
`req~cell-edit.new-note-mode-editor-ask~1` P1
- 受け入れ: `mode = "editor"` で `a` → 名前だけの窓(見えている列は欄にしない)。

#### `editor` の作り方では、前もって入れる値(絞り込み・set・hidden。cell-edit.template-vars の変数を埋める)と本文の雛形でノートを作るべき
`req~cell-edit.new-note-mode-editor-create~1` P1
- 受け入れ: `mode = "editor"`・`hidden = ["created"]`・set に `created = "{date}"` で、今日が 2026-10-02 のとき名前を打って Enter → 作ったノートに `created: 2026-10-02`。

#### `editor` の作り方では、作ったノートをすぐ screen.editor の `e` と同じエディタで開くべき
`req~cell-edit.new-note-mode-editor-open~1` P1
- 受け入れ: `mode = "editor"` で名前を打って Enter → ノートができ、エディタで開く頼み(`e` と同じ)が出る。

#### `editor` の作り方でエディタから戻ったら、そのノートを読み直してその行を選ぶべき
`req~cell-edit.new-note-mode-editor-return~1` P1
- 受け入れ: `mode = "editor"` で「メモ」を作り、エディタで `status: doing` を書いて戻る → 表で `メモ.md` の行が選ばれ、status に doing が出る。

#### 新しいノートのフォームからも、作ってすぐエディタで開くキー(既定 Ctrl+E。割り当て直せる)で作れるべき
`req~cell-edit.new-note-mode-form-ctrl-e~1` P1
- 受け入れ: フォームで Ctrl+E → 作ってエディタで開く頼み。

#### 表のモードで、選んだ行のノートのファイルの名前を、操作の一覧(screen.action-menu)とパレットの「ノートの名前を変える」で変えられるべき(キーは既定で割り当てない)
`req~cell-edit.rename-note-menu~1` P1
- 受け入れ: 「会議の準備」の行で「ノートの名前を変える」→「会議の準備(10月)」と打つ → ファイルが `会議の準備(10月).md` になり、行がその名前で選ばれたまま。

#### ノートの名前の変更は、新しい名前を打ち(`.md` は付けても付けなくてもよい)、同じフォルダの中で名前だけを変えるべき
`req~cell-edit.rename-note-same-folder~1` P1
- 受け入れ: `プロジェクト/会議の準備.md` の行で「会議の準備(10月).md」と打つ → `プロジェクト/会議の準備(10月).md` になる(`.md` を重ねず、フォルダはそのまま)。

#### ノートの新しい名前が空か、フォルダの区切りを含むときは、理由を出して断るべき
`req~cell-edit.rename-note-refuse-bad-name~1` P1
- 受け入れ: `/` を含む名前・空の名前 → 断って理由。

#### ノートの新しい名前がもうあるファイルの名前なら、理由を出して断るべき
`req~cell-edit.rename-note-refuse-existing~1` P0
- 受け入れ: もうある名前 → 断って理由、どちらのファイルもそのまま。

#### そのノートにためた変更がある間は、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-refuse-pending~1` P0
- 受け入れ: status をためた変更のある行で「ノートの名前を変える」→ 断る。

#### 読むだけのノート(write-back.refuse-unsafe-notes)では、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-refuse-unsafe~1` P0
- 受け入れ: `a: 1\na: 2` を持つノートの行で「ノートの名前を変える」→ 断り、ファイルの名前はそのまま。

#### 読むだけの起動(cli.readonly)では、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-readonly~1` P1
- 受け入れ: `--readonly` で起動 → 操作の一覧とパレットに「ノートの名前を変える」が出ない。

#### ノートの名前を変えたら、開いている表と登録した表(cli.places)のノートの、古い名前を指すリンク(`[[古い名前]]`・`[[古い名前|表示]]`・`[[古い名前#見出し]]`)を新しい名前に書き換える変更をためて、その数を知らせるべき(保存の前の差分(write-back.pending-and-diff)で見られ、取り消せる)
`req~cell-edit.rename-note-links~1` P0
- なぜ: 名前の変更でリンクを黙って切らないため
- 受け入れ: 「会議の準備」を「会議の準備(10月)」に変える → ほかのノートの `project: "[[会議の準備]]"` を書き換える変更が2件ためられて2件と知らされ、Ctrl+S の差分に出る。

#### 表のモードで、選んだ行のノートを、操作の一覧(screen.action-menu)とパレットの「ノートを消す」でごみ箱へ動かせるべき(キーは既定で割り当てない)
`req~cell-edit.delete-note-menu~1` P1
- なぜ: 間違って作ったノートを、mdgrid を出ずに片づけるため
- 受け入れ: 「メモ」の行で「ノートを消す」→ 確かめに y → `メモ.md` が元の場所に無く、ごみ箱に同じバイトであり、表から「メモ」の行が無くなって下の行が選ばれる。
- 確かめ: utest・stest

#### ノートの削除は、消すノートの名前を示して確かめ、`y` のときだけ行わなければならない(ほかの打ち込みと Esc ではやめる)
`req~cell-edit.delete-note-confirm~1` P0
- なぜ: 押し間違いでノートを失わないため
- 受け入れ: 「メモ」の行で「ノートを消す」→「メモ」を消すかを聞かれ、Esc → `メモ.md` は元の場所に同じバイトで残る。

#### ノートの削除は、ファイルを消さずにごみ箱へ動かさなければならず、動かせないときはノートを元の場所に残して理由を出さなければならない
`req~cell-edit.delete-note-trash-only~1` P0
- なぜ: 消したあとで要ると分かったノートを、人が戻せるようにするため
- 受け入れ: ごみ箱へ動かせない状態で「ノートを消す」に y → `メモ.md` は元の場所に同じバイトで残り、理由が出る。

#### ノートの削除は、ごみ箱に前からあるファイルを置き換えてはならない
`req~cell-edit.delete-note-trash-keep-old~1` P0
- なぜ: 前に消したノートを、同じ名前の新しい削除で失わないため
- 受け入れ: 中身が「古い」の `メモ.md` を消したあと、中身が「新しい」の `メモ.md` を作って消す → ごみ箱に「古い」と「新しい」の両方が同じバイトである。

#### そのノートにためた変更がある間は、ノートの削除を断らなければならない
`req~cell-edit.delete-note-refuse-pending~1` P0
- なぜ: ためた変更を、行き先のファイルごと黙って捨てないため
- 受け入れ: status をためた変更のある行で「ノートを消す」→ 断って理由が出て、`メモ.md` は元の場所に残り、ためた変更は1件のまま。

#### 読むだけの起動(cli.readonly)では、ノートの削除を出さないべき
`req~cell-edit.delete-note-readonly~1` P1
- 受け入れ: `--readonly` で起動 → 操作の一覧とパレットに「ノートを消す」が出ない。

#### CSV・TSV の表では、ノートの削除を出さないか、理由を出して断るべき
`req~cell-edit.delete-note-csv~1` P1
- 受け入れ: `台帳.csv` で「ノートを消す」→ CSV では使えない理由が出て、`台帳.csv` は1バイトも変わらない。

#### 新しいノートを作ったことは、取り消しの1手(write-back.undo-pending)にし、取り消すと、作ったノートを確かめずにごみ箱へ動かすべき
`req~cell-edit.new-note-undo~1` P1
- なぜ: 間違って作ったノートを、作った直後の1手で片づけるため
- 受け入れ: 「メモ」を作って `u` → `メモ.md` が元の場所に無く、ごみ箱に同じバイトであり、表から「メモ」の行が無くなる。「メモ」を作って status を直してから `u` を1回 → status の直しだけが戻り、`メモ.md` は元の場所に残る。
- 確かめ: utest・stest

#### 取り消したノートの作成は、やり直し(write-back.redo-pending)で、ごみ箱から元の場所へ戻すべき
`req~cell-edit.new-note-redo~1` P1
- なぜ: 間違いだと思って取り消したが間違いでなかったときに、打ち直さずに先へ進むため
- 受け入れ: 「メモ」を作って `u` のあと Ctrl+R → `メモ.md` が元の場所に作ったときと同じバイトで戻り、表で「メモ」の行が選ばれ、ごみ箱に `メモ.md` が無い。

#### ノートの削除は、取り消しの1手(write-back.undo-pending)にし、取り消すとごみ箱から元の場所へ戻し、やり直すとまたごみ箱へ動かすべき
`req~cell-edit.delete-note-undo~1` P1
- 受け入れ: 「メモ」を消して `u` → `メモ.md` が元の場所に同じバイトで戻り、表で「メモ」の行が選ばれる。続けて Ctrl+R → `メモ.md` が元の場所に無く、ごみ箱にある。

#### パレットの「ごみ箱から戻す」で、ごみ箱のノートの一覧(名前・元の場所・消した日時)を出し、選んだノートを元の場所へ戻せるべき
`req~cell-edit.trash-restore-list~1` P1
- なぜ: mdgrid を閉じたあとや、取り消しの手が流れたあとでも、消したノートを戻せるようにするため
- 受け入れ: `プロジェクト/メモ.md` を消して mdgrid を起動し直し、「ごみ箱から戻す」で「メモ」を選ぶ → `プロジェクト/メモ.md` が同じバイトで戻り、表で「メモ」の行が選ばれる。ごみ箱が空のとき → 一覧を出さず、空だと知らせる。
- 確かめ: utest・stest

#### ごみ箱から戻すとき(取り消し・やり直し・「ごみ箱から戻す」のどれでも)、元の場所に同じ名前のファイルがあれば、置き換えずに断らなければならない
`req~cell-edit.trash-restore-keep-existing~1` P0
- なぜ: 戻す操作で、あとから作ったノートを失わないため
- 受け入れ: `メモ.md` を消したあと、別の中身の `メモ.md` を作り、「ごみ箱から戻す」で「メモ」を選ぶ → 断って理由が出て、元の場所の `メモ.md` とごみ箱の `メモ.md` はどちらも同じバイトで残る。

#### 読むだけの起動(cli.readonly)では、「ごみ箱から戻す」を出さないべき
`req~cell-edit.trash-restore-readonly~1` P1
- 受け入れ: `--readonly` で起動 → パレットに「ごみ箱から戻す」が出ない。

#### 表の列の型を、設定の `[types]`(列の名前 = `"text"`・`"number"`・`"checkbox"`・`"date"`・`"datetime"`・`"list"`・`"auto"`)で決められるべきで、決めた型はルートフォルダの型の設定と推定(cell-edit.column-type)より優先するべき(範囲ごとに重ね、狭い範囲が勝つ。screen.profile-scopes)
`req~cell-edit.type-config-types~1` P1
- 受け入れ: `[types] due = "date"` → due の値が文字でも日付の入力になり、日付でない値に `!`。

#### `[types]` の `"auto"` は、その列を推定に戻すべき
`req~cell-edit.type-config-auto~1` P1
- 受け入れ: due の型の窓で「自動」を選ぶ → 推定の型に戻り、views.toml から due が消える。全体の設定で `due = "number"`、表の範囲で `due = "auto"` → due は推定の型になる。

#### 表のモードで、列の見出しの型の印(screen.rich-cells)のクリック・操作の一覧(screen.action-menu)の列の節・パレットから、選んだ列の型を選び直す窓を出すべき
`req~cell-edit.type-config-dialog~1` P1
- 受け入れ: due の見出しの型の印をクリック → 型の窓。

#### 型を選び直す窓には、「自動」(推定の型を添える)と各型を並べ、今の型に印を付けるべき
`req~cell-edit.type-config-dialog-items~1` P1
- 受け入れ: `[types] due = "number"` で、推定なら日付の due の型の窓 → 「自動」に日付が添えられて各型と並び、数に印。

#### 型を選び直す窓で選んだ型は、開いている表の範囲(views.toml の表)の `[types]` に書いて、すぐ表に効かせるべき(セルの入力・型と合わない値の印(cell-view.type-mismatch)・並べ替え・見出しの型の印)
`req~cell-edit.type-config-dialog-write~1` P1
- 受け入れ: due の型の窓で「数」を選ぶ → views.toml の表に `due = "number"`、日付の値に `!`。

#### 列の型を選び直しても、ノートの値は書き換えないべき
`req~cell-edit.type-config-notes-untouched~1` P1
- 受け入れ: 型の窓で due を「数」にする → ノートのファイルは1バイトも変わらない。

#### `file.*`・`formula.*` の列では、型を選び直す窓を出さないべき
`req~cell-edit.type-config-file-formula~1` P1
- 受け入れ: `file.mtime` の列でパレットから型を選び直す → 型の窓は出ない。

#### `[types]` の知らない型の名前は、警告してその列を推定のままにするべき
`req~cell-edit.type-config-unknown~1` P1
- 受け入れ: `[types] due = "when"` → 警告して推定。

#### 表の列の決まりを、設定の `[rules.<列の名前>]` の `required`(真偽)で書けるべきで、真なら、キーが無い・null・空の文字列・空のリストを違反とするべき
`req~cell-edit.rules-required~1` P1
- 受け入れ: `[rules.status] required = true` → status の無いノートが違反(cell-view.problems の印)、`status: done` は違反でない。

#### 表の列の決まりを、`[rules.<列の名前>]` の `values`(文字の配列)で書けるべきで、値の文字(リストなら各要素の文字)がこのどれでもなければ違反とするべき(大文字小文字を区別する)
`req~cell-edit.rules-values~1` P1
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]` → `status: wip` のノートが違反(cell-view.problems の印)、`status: done` は違反でない。

#### 列の決まりは、範囲ごとに項目ごとに重ね、狭い範囲が勝つべき(screen.profile-scopes)
`req~cell-edit.rules-scopes~1` P1
- 受け入れ: 全体で `[rules.status] values = ["todo", "doing", "done"]`、表の範囲で `values = ["todo", "done"]` を重ねる → 表では doing も違反。

#### 列の決まりは、ノートを書き換えないべき
`req~cell-edit.rules-notes-untouched~1` P1
- なぜ: 決まりは確かめるための物で、書いてから直す流れを妨げないため
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]` で `status: wip` のノートを開く → 違反の印が付くだけで、ファイルは1バイトも変わらない。

#### 列の決まりに合わない値を書くことを、止めないべき
`req~cell-edit.rules-no-block~1` P1
- なぜ: 決まりは確かめるための物で、書いてから直す流れを妨げないため
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]` の status のセルで `wip` と打って Enter → `wip` のためる変更になる。

#### `values` のある列の編集では、cell-edit.value-list の数の上限によらず、その値を候補のリストに並べるべき
`req~cell-edit.rules-values-list~1` P1
- 受け入れ: `[rules.status] values = ["todo", "doing", "done"]` → status のセルで Enter → 候補のリストに todo・doing・done。

#### `file.*`・`formula.*` の列の決まりは、警告して使わないべき
`req~cell-edit.rules-file-formula~1` P1
- 受け入れ: `[rules."file.name"] required = true` → 警告して使わない。

#### 列の決まりの知らない項目と型の違う値は、警告してその項目を使わないべき
`req~cell-edit.rules-bad-items~1` P1
- 受け入れ: `required = "yes"` → 警告して required を使わない。`[rules.status] foo = 1` → 警告。

### 次の段(まだ作っていない。P2)

#### 選んだ範囲の値を、ためる変更として正規表現で置き換えてよい
`req~cell-edit.regex-replace~1` P2
- 受け入れ: `v1` → `v2` の置き換えで、選んだ行の値だけが変わる(読んで判定)。

## 未確定事項

### ノートの削除のごみ箱は、保管庫の `.trash/`(Obsidian と同じ。mdgrid は探さないフォルダ)か、OS のごみ箱か
`tbd~cell-edit.delete-note-trash-place`
- 決める人: 人
- 仮: 保管庫の `.trash/`(「ごみ箱から戻す」の一覧と元の場所への戻しを、どの OS でも同じに作れるため)

### 消すノートを指すリンクが、ほかのノートにあるときにどうするか
`tbd~cell-edit.delete-note-links`
- 決める人: 人
- 仮: リンクは書き換えず、確かめの問いに、指しているノートの数を添える

### 取り消しとやり直しの手を一覧で見て、選んだ所まで戻す・進む「履歴」の画面を持つか
`tbd~cell-edit.history-screen`
- 決める人: 人
- 仮: 持たない(`u`・Ctrl+R と「ごみ箱から戻す」で足りるかを使って確かめてから決める)

### ごみ箱のノートを、mdgrid から完全に消す・古いものを自動で片づける操作を持つか
`tbd~cell-edit.trash-empty`
- 決める人: 人
- 仮: 持たない(mdgrid はごみ箱の中を消さない。片づけは人がファイルの道具でする)
