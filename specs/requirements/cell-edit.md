# セルの編集とリストからの選択(cell-edit) の要求

## 目的

表のセルを選んで、その場で値を直す。直した値はすぐにはファイルに書かず、保存の操作までためておく([write-back](write-back.md) の write-back.pending-and-diff)。値の種類に合わせて、打つ・リストから選ぶ・切り替える・日付を入れる、の入り方を変える。キーの割り当ては [screen](screen.md)、行の選び方は [navigation](navigation.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/cell-edit/spec.md](../_history/decidespec/cell-edit/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 旧い CE-6(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-list-select.md))。移行先: cell-edit.list-items(リストの値を編集できるようにした)。
- 旧い CE-13(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-list-select.md))。移行先: cell-edit.list-items・cell-edit.list-layout(版1に入れた)。
- 旧い CE-14(新しい名前なし)は 2026-10-02 に削除([2026-10-02](../_history/decidespec/_decisions/2026-10-02-new-note.md))。移行先: cell-edit.new-note・cell-edit.new-note-form(版1に入れた)。
- 採らない: 式(`formula.*`)の編集と、表計算のようなセル参照・数式。
- 採らない: 取り消しとやり直しの手を一覧で見て、選んだ所まで戻す・進む「履歴」の画面(2026-11-18。`u`・Ctrl+R と「ごみ箱から戻す」(cell-edit.trash-restore-list)でまかなう)。
- 採らない: ごみ箱のノートを mdgrid から完全に消す操作と、古いごみの自動の片づけ(2026-11-18。mdgrid はごみ箱の中を消さず、片づけは人がファイルの道具でする)。
- キーの名前の変更と削除は cell-edit.rename-delete-key(2026-10-06 に write-back.byte-range・write-back.frontmatter-only を変えて入れた)。列の追加は cell-edit.add-column。

## 前提(旧い仕様で AI が置いた仮定)

- 候補の数の既定 20 は AI の仮置き。使ってから見直す。
- Obsidian には列挙値から選ぶ公式の仕組みが無く、既存の値の候補(サジェスト)だけがある(2026-09-30 の調査)。cell-edit.value-list はその同じ材料で、少ないときはリストにする形。
- 一括の設定(cell-edit.bulk-set)と、編集中の Tab・Ctrl+R(cell-edit.revert-and-tab)は VisiData・lazysql の操作から採った。

## 要求

### 編集の入り方

#### セルの編集は、そのセルの位置に出す入力(入力ボックス・リスト)で行うべき
`req~cell-edit.inline-input~1` P1
- Acceptance:
  - Given:
  - When: 選んだセルで Enter
  - Then: その場に入力ボックス。

#### セルの入力は、Enter で確定し、Esc で取り消すべき
`req~cell-edit.inline-enter-esc~1` P1
- Acceptance:
  - Given:
  - When: `title: 案` のセルの入力で `案2` と打つ
  - Then: Enter なら確定し、Esc なら入力が閉じて `案` のまま(ためる変更なし)。

#### 確定したセルの値は、ためる変更になるべき(write-back.pending-and-diff)
`req~cell-edit.inline-pending~1` P1
- Acceptance:
  - Given:
  - When: セルの入力で値を打って Enter
  - Then: 確定してためる変更になり、ファイルはまだ変わらない。

#### 入力の外をクリックしたら、確定を試みるべき
`req~cell-edit.inline-click-commit~1` P1
- Acceptance:
  - Given:
  - When: 入力ボックスで値を打って外をクリック
  - Then: 確定してためる変更になる。

#### 入力の外をクリックしたとき値が不正なら、入力を閉じずにそのクリックを捨てるべき
`req~cell-edit.inline-click-invalid~1` P1
- Acceptance:
  - Given:
  - When: 日付の列の入力で `2026-02-30` と打って外の行をクリック
  - Then: 入力は開いたままで、クリックした行は選ばれない。

#### 編集中に、編集前の値に戻す操作(Ctrl+R)と、確定して隣のセルへ移って編集を続ける操作(Tab / Shift+Tab)があるべき
`req~cell-edit.revert-and-tab~1` P1
- Acceptance:
  - Given:
  - When: Tab
  - Then: 確定して右のセルの入力ボックスが開く。

#### 列の型は、設定の `[types]`(cell-edit.type-config)、ルートフォルダの型の設定(`.obsidian/types.json`)、Obsidian と同じ規則の推定(最初に見つかった空でない値)の順で決めるべき
`req~cell-edit.column-type-order~1` P1
- Acceptance:
  - Given: types.json で `due` が date
  - When:
  - Then: 日付の入力になる。types.json で date と決めた列の日時の値は今までどおり `!`。

#### 推定で日付になった列に日時の値(`YYYY-MM-DDTHH:MM`)もあれば、その列を日時の列とするべき
`req~cell-edit.column-type-mixed-datetime~1` P1
- Rationale: 日付だけの値も日時の列に合うので、時刻を型の不一致として扱わないため
- Acceptance:
  - Given:
  - When: types.json の無い列で 2026-08-01 と 2026-08-03T08:15 が混ざる
  - Then: 日時の列で、どちらも `!` が付かない。

#### 改行を含む値は、1行の入力では編集できないようにし、エディタ(screen.editor)で開くか読むだけにするべき
`req~cell-edit.multiline-guard~1` P1
- Acceptance:
  - Given:
  - When: ブロックスカラーの値のセルで Enter
  - Then: 1行の入力は開かず、案内を表示。

#### 表の画面では、ビューの帯(screen.layout)の右の端にキーを添えた「+ 新規」(例「a + 新規」)を出し、クリックで新しいノートを作り始められるべき
`req~cell-edit.new-note-button~1` P1
- Acceptance:
  - Given: ビューの帯を出した表の画面
  - When:
  - Then: 帯の右の端に「a + 新規」が出て、クリックすると名前を打つ欄が出る。

#### ビューの帯を出さないとき(screen.display-options・screen.tabs-setting で、タブも検索の欄も出さないとき)は、「+ 新規」をヘッダーの右の端の画面の型のメニューの左に出すべき
`req~cell-edit.new-note-button-header~1` P1
- Acceptance:
  - Given: タブも検索の欄も出さない設定
  - When:
  - Then: ヘッダーの右の端の画面の型のメニューの左に「+ 新規」が出る。

#### 新しいノートは、キーの操作(screen.key-table のキーの表で割り当て直せる)とコマンドのパレットからも作り始められるべき
`req~cell-edit.new-note-start-keys~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 表のモードで新しいノートのキー(既定 `a`)→ 名前を打つ欄が出る。パレットで新しいノートを選んでも同じ欄が出る。

#### 新しいノートは、名前を打つ欄に名前を打って Enter で `<名前>.md` として作り、表のその行を選ぶべき
`req~cell-edit.new-note-name~1` P1
- Acceptance:
  - Given:
  - When: 名前を打つ欄で「会議の準備」と打って Enter
  - Then: `会議の準備.md` が作られ、表にその行が選ばれて出る。

#### 新しいノートを作る場所は、開いたフォルダ(`.base` で開いたときはその `.base` のフォルダ)を既定とするべき
`req~cell-edit.new-note-place~1` P1
- Acceptance:
  - Given:
  - When: `仕事/tasks.base` で開いて「メモ」と打って Enter
  - Then: `仕事/メモ.md` が作られる。

#### 新しいノートの名前に `下のフォルダ/名前` と書けば、下のフォルダに作り、そのフォルダが無ければ作るべき
`req~cell-edit.new-note-subfolder~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `プロジェクト/新しい` → `プロジェクト/新しい.md`(`プロジェクト` のフォルダが無ければ作る)。

#### 新しいノートを作る既定のフォルダは、設定で変えられるべき
`req~cell-edit.new-note-default-folder~1` P1
- Acceptance:
  - Given:
  - When: 設定で既定のフォルダを `inbox` にする
  - Then: 「メモ」と打って Enter で `inbox/メモ.md` が作られる。

#### フォルダを2つ以上開いたとき(起動の引数に2つ以上のフォルダや `.base`)は、名前の欄の前に、作る場所を開いたフォルダの一覧から選ぶ欄を出すべき(一覧は起動の引数の順。既定は最初のフォルダ。Esc で取りやめ)
`req~cell-edit.new-note-folder-pick~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: フォルダ A と B を開いて「+ 新規」→ A と B の一覧(A が選ばれている)。

#### 作る場所を選ぶ欄で選んだフォルダを、新しいノートを作る場所の根にするべき
`req~cell-edit.new-note-folder-root~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: フォルダ A と B を開いて「+ 新規」→ B を選んで「メモ」→ `B/メモ.md` が作られ、その行が選ばれる。

#### フォルダを2つ以上開いたときも、「+ 新規」を出すべき
`req~cell-edit.new-note-folders-button~1` P1
- Acceptance:
  - Given:
  - When: フォルダ A と B を開く
  - Then: ビューの帯に「+ 新規」が出る。

#### 開いたフォルダの外(`..` や絶対パス)を指す名前では、新しいノートを作らず理由を出すべき
`req~cell-edit.new-note-refuse-outside~1` P1
- Acceptance:
  - Given:
  - When: `../外`
  - Then: 作らず理由。

#### 既にある名前では、新しいノートを作らず理由を出すべき
`req~cell-edit.new-note-refuse-existing~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `会議の準備.md` のあるフォルダで同じ名前「会議の準備」→ 作らず理由。

#### 関係マップの画面(relations.screen-menu)でも、「+ 新規」を表の画面と同じ位置にいつも出すべき
`req~cell-edit.new-note-relmap-button~2` P1
- Rationale: 画面ごとにボタンが出たり消えたりしないため
- Acceptance:
  - Given: 関係マップの画面
  - When:
  - Then: ヘッダーの下に帯の上の余白と同じ行の数を空け、その下のマップの窓の上の縁の右の端に「+ 新規」が重なって出る(表の画面のビューの帯の真ん中の行の右の端と同じ位置。関係マップにはビューの帯が無いため)。
- Acceptance:
  - Given: ビューの帯を出さないとき
  - When:
  - Then: 表の画面と同じくヘッダーの画面の型のメニューの左に出る。

#### 関係マップの画面で「+ 新規」を押すか同じキーの操作をしたら、関係マップで選んでいる表を開いて、その表で名前を打つ欄を出すべき
`req~cell-edit.new-note-relmap-start~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 関係マップで「タスク」の表を選んで「+ 新規」→「タスク」の表が開き、名前を打つ欄が出る。同じキーの操作(既定 `a`)でも同じ。

#### 読むだけの起動(`--readonly`)では、新しいノートを作らないべき
`req~cell-edit.new-note-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で起動して「+ 新規」かキーの操作
  - Then: 作らない(ファイルは増えない)。

#### 新しいノートの作成では新しいファイルだけを作り、既にあるファイルは書き換えないべき(write-back.frontmatter-only)
`req~cell-edit.new-note-no-overwrite~1` P1
- Acceptance:
  - Given:
  - When: 「会議の準備」を作る
  - Then: 増えるのは `会議の準備.md` だけで、既にあるファイルは1バイトも変わらない。

#### ビューの絞り込みの条件のうち値が1つに決まるもの(`status == "todo"`・タグを含む など)は、その値をフロントマターに入れて新しいノートを作るべき
`req~cell-edit.new-note-filter-values~1` P1
- Rationale: 作った行がそのビューに残るようにするため
- Acceptance:
  - Given:
  - When: 「status が todo」のビューで「+ 新規」→「会議の準備」と打って Enter
  - Then: `会議の準備.md` が `---\nstatus: todo\n---\n` で作られ、表にその行が選ばれて出る。

#### 新しいノートは、作る場所(cell-edit.new-note の作る場所を選ぶ欄があるとき)・名前・入力させる項目を、1つの窓(フォーム)に縦に並べて作るべき
`req~cell-edit.new-note-form-fields~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: フォルダ A と B を開き、設定に ask = [priority] で「+ 新規」→ 1つの窓に 作る場所・名前・priority の欄が縦に並ぶ。

#### 新しいノートの窓で入力させる項目は、設定の `ask` があればその列、無ければ今の表で見えている列(`file.*`・`formula.*` を除く)とするべき
`req~cell-edit.new-note-form-ask~2` P1
- Acceptance:
  - Given: 設定に ask = [priority]
  - When:
  - Then: 窓に 名前・priority の欄が並ぶ。
- Acceptance:
  - Given: ask の無い設定
  - When:
  - Then: 見えている列が欄に並ぶ。

#### 新しいノートの窓で、前もって入れる値(cell-edit.new-note の絞り込みの値・設定の値)を入れた欄には、出どころを示すべき
`req~cell-edit.new-note-form-source~1` P1
- Acceptance:
  - Given: 設定に ask = [priority]、set に tags = [inbox]
  - When:
  - Then: 窓に 名前・priority・tags の欄が並び、tags に「設定から」。

#### 新しいノートの窓を開くと名前の欄の入力が開き、Enter・Tab で今の欄を確定して次の欄、Shift+Tab で前の欄に移るべき。リストの列の欄は、リストの値の付け外しの窓(screen.checklist-keys)と同じく、Enter と Space で選んだ値を入れる・外し、Tab か「[ 決める ]」の行で決めて次の欄に移るべき
`req~cell-edit.new-note-form-move~3` P1
- Rationale: リストの欄だけ Enter で次の欄へ進み、最後の欄ではノートができたため、値を入れるつもりの Enter で、何も選ばないままノートができた(2026-11-25 に調べ物の台帳で見つかった。セルのチェックの窓は人が「Enter で入れる」と決めた)
- Acceptance:
  - Given:
  - When: 設定に ask = [priority] で窓を開く
  - Then: 名前の欄の入力が開いている。
- Acceptance:
  - Given:
  - When: 名前を打って Enter か Tab → priority の欄へ、Shift+Tab
  - Then: 名前の欄へ戻る。
- Acceptance:
  - Given: ask = [tags, priority](tags はリストの列で、候補に 仕事・家)の窓で、tags の欄にいる
  - When: 「仕事」を選んで Enter
  - Then: 仕事にチェックが付き、tags の欄のまま(候補の窓は開いたまま)。
- Acceptance:
  - Given: 同じ窓で、仕事にチェックを付けた
  - When: Tab
  - Then: priority の欄に移り、tags の欄には「仕事」が見える。
- Acceptance:
  - Given: 同じ窓で、tags の欄にいる
  - When: Shift+Tab
  - Then: 名前の欄に移る。
- Needs: utest, stest

#### 新しいノートの窓の各欄は、型に合った入力(cell-edit.column-type)で直すべき
`req~cell-edit.new-note-form-typed~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: ask = [priority, due](priority が数・due が日付の列)→ priority の欄で `12a` は閉じずに不正を表示し、due の欄はカレンダーつきの日付の入力で開く。

#### 新しいノートの窓で、決まりに `values` のある列(cell-edit.rules-values)の欄は、その列を持つノートが1件も無くても、その値を決まりの順で候補のリストに並べるべき
`req~cell-edit.new-note-form-rule-values~2` P1
- Rationale: 見本なしで作った台帳の最初のノートでも、選べる値から選べるようにするため
- Acceptance:
  - Given: ノートが1件も無い表で `[rules.priority] values = ["p1", "p2", "p3"]`、ask = [priority]
  - When:
  - Then: priority の欄の候補のリストに p1・p2・p3 がこの順で並び、最後に「なし」。
- Acceptance:
  - Given: 決まりの無いテキストの列で、どのノートにも値が無い
  - When:
  - Then: リストを出さず1行の入力。

#### 新しいノートの窓は、Ctrl+S(どの欄からでも)と、最後の欄の確定(リストの列の欄では Tab か「[ 決める ]」の行。ほかの欄では Enter か Tab)でノートを作るべき
`req~cell-edit.new-note-form-create~2` P1
- Rationale: リストの欄の Enter は値を入れる・外す(cell-edit.new-note-form-move)ので、最後の欄でも Enter では作らない
- Acceptance:
  - Given:
  - When: 名前の欄で「メモ」と打って Ctrl+S
  - Then: `メモ.md` が作られる。最後の欄で Enter でも作られる。
- Acceptance:
  - Given: ask = [tags](tags はリストの列で、候補に 仕事・家)の窓で、名前に「メモ」と打って tags の欄にいる
  - When: 「仕事」を選んで Enter
  - Then: ノートは作られず、仕事にチェックが付く。
- Acceptance:
  - Given: 同じ窓で、仕事にチェックを付けた
  - When: Tab
  - Then: `メモ.md` が `tags` の要素「仕事」で作られる。
- Needs: utest

#### 新しいノートの窓は、欄の上に候補の窓(cell-edit.new-note-form-esc-popup)が出ていないときの Esc で、ノートを作らずにやめるべき(打った値があっても確かめない)
`req~cell-edit.new-note-form-esc~2` P1
- Rationale: 候補の窓を閉じるつもりの Esc で窓ごと閉じないように、閉じるのは候補の窓の出ていないときだけにする。窓を閉じるときに確かめないのは、セルの入力の Esc(打った字を確かめずに捨てる)と同じにするため
- Acceptance:
  - Given:
  - When: 名前を打ってから Esc
  - Then: 窓が閉じ、ノートは作られない。
- Needs: utest

#### 新しいノートの窓の空の欄は、ノートに書かないべき
`req~cell-edit.new-note-form-skip-empty~1` P1
- Acceptance:
  - Given: 設定に ask = [priority]、set に tags = [inbox]
  - When: priority を空のまま名前を打って Ctrl+S
  - Then: priority の行が無く `tags: [inbox]`(縦の形)のノート。

#### 新しいノートの窓で作れない名前のときは、作らずに名前の欄に理由を示すべき
`req~cell-edit.new-note-form-bad-name~1` P1
- Acceptance:
  - Given:
  - When: 既にある名前で Ctrl+S
  - Then: 名前の欄に理由、作らない。

#### 新しいノートの窓で必須の欄(cell-edit.new-note-config)が空のときは、作らずにその欄に理由を示すべき
`req~cell-edit.new-note-form-required~1` P1
- Acceptance:
  - Given:
  - When: `required = ["due"]` で due を空のまま Ctrl+S
  - Then: 作らずに due の欄に理由。

#### 設定の `[new_note]` の表に、`name_head`(名前の雛形 `name` を埋めた文字の見せ方)を持ち、`"show"`(既定)では名前の欄に入れて出し、`"hide"` では名前の欄に(名前の欄を離れたあとも)出さず、打った名前の前に付けて作るべき
`req~cell-edit.new-note-name-head~2` P1
- Rationale: 名前の頭に付ける日付を、初めから欄に入れて見せたい人と、見ずに題だけを打ちたい人がいるため(2026-11-24 に人が「選べるようにする」と決めた)。表で頭を隠す `display.note_name`(screen.note-name)と `"hide"` を組むと、表でも打つ欄でも頭を見ない。名前の欄を離れたとたんに頭の付いた名前が出ると、隠したはずの頭が見えた(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: 今日が 2026-10-02、`[new_note] name = "{date:YYYYMMDD}_"`・`name_head = "hide"`
  - When: 新しいノートの窓を開く
  - Then: 名前の欄は空で、`20261002_` が出ない。
- Acceptance:
  - Given: 同じ設定
  - When: 名前の欄に「買い物」と打って Ctrl+S
  - Then: `20261002_買い物.md` が作られる。
- Acceptance:
  - Given: 同じ設定
  - When: 名前の欄に「sub/買い物」と打って Ctrl+S
  - Then: `sub/20261002_買い物.md` が作られる(頭は最後の名前の部分に付く)。
- Acceptance:
  - Given: 同じ設定
  - When: 名前の欄に何も打たずに Ctrl+S
  - Then: ノートを作らず、窓は開いたまま。
- Acceptance:
  - Given: 同じ設定に `[display] note_name = '^\d{8}_'` を足す
  - When: 名前の欄に `20250101_古い`(`note_name` に当たる頭を自分で打った名前)と打って Ctrl+S
  - Then: `20250101_古い.md` が作られる(頭を重ねない)。
- Acceptance:
  - Given: 同じ設定で、ask = [priority]
  - When: 名前の欄に「買い物」と打って Enter
  - Then: priority の欄に移り、名前の欄には「買い物」が見え、`20261002_` は出ない。
- Acceptance:
  - Given: 今日が 2026-10-02、`[new_note] name = "{date:YYYYMMDD}_"` で、`name_head` が無いか `"show"`(`display.note_name` があっても同じ)
  - When: 新しいノートの窓を開く
  - Then: 名前の欄に `20261002_` が入って出る(cell-edit.new-note-config-name のまま)。
- Acceptance:
  - Given: `[new_note] name_head = "maybe"`
  - When: 設定を読む
  - Then: `"show"` でも `"hide"` でもないと警告し、`"show"` として扱う。
- Needs: utest

#### 新しいノートの窓の欄の行をクリックしたら、今の欄を確定して、クリックした欄に移るべき(今の欄の値が読めなければ、移らずに理由を出す)
`req~cell-edit.new-note-form-click-field~1` P1
- Rationale: 前の欄へ戻るのが Shift+Tab だけで、打ち直したい欄へ直に行けないため(2026-11-24 の報告)
- Acceptance:
  - Given: ask = [priority, due] の窓で、名前に「メモ」と打って Enter を2回押し、due の欄にいる
  - When: 名前の欄の行をクリック
  - Then: 名前の欄の入力が「メモ」で開く。
- Acceptance:
  - Given: ask = [priority, due] の窓で、名前に「メモ」と打った
  - When: due の欄の行をクリック
  - Then: due の欄の入力が開き、名前の欄には「メモ」が見える。
- Acceptance:
  - Given: ask = [priority, due](priority は数の列)の窓で、priority の欄に `12a` と打った
  - When: 名前の欄の行をクリック
  - Then: priority の欄のまま、数として読めない理由が出る。
- Needs: utest

#### 新しいノートの窓のキーの行(次へ・作る・前の項目・やめる・エディタで)の項目をクリックしたら、そのキーを押したのと同じ振る舞いにするべき
`req~cell-edit.new-note-form-click-keys~1` P1
- Rationale: マウスで欄を選んだあと、作る・やめるのためにキーへ手を戻さずに済むようにするため(2026-11-24 の報告)
- Acceptance:
  - Given: 名前に「メモ」と打った窓
  - When: キーの行の「作る」をクリック
  - Then: `メモ.md` が作られる。
- Acceptance:
  - Given: 名前に「メモ」と打った窓
  - When: キーの行の「やめる」をクリック
  - Then: 窓が閉じ、ノートは作られない。
- Acceptance:
  - Given: ask = [priority] の窓で、名前に「メモ」と打って Enter を押し、priority の欄にいる
  - When: キーの行の「前の項目」をクリック
  - Then: 名前の欄の入力が「メモ」で開く。
- Needs: utest

#### 新しいノートの窓の欄に出た候補のリストの項目をクリックしたら、その値に決めて次の欄へ移るべき(最後の欄ならノートを作る)
`req~cell-edit.new-note-form-click-list~1` P1
- Rationale: 表のセルの編集(候補のクリックで決まる)と同じにするため
- Acceptance:
  - Given: `[rules.status] values = ["todo", "doing", "done"]`、ask = [status, priority] の窓で、status の欄にいる
  - When: 候補のリストの `doing` をクリック
  - Then: priority の欄に移り、status の欄には `doing` が見える。
- Needs: utest

#### 新しいノートの窓のリンクの列(relations.link-input-candidates)の欄は、セルを直すときと同じく、行き先のフォルダのノートを候補にし、選んだノートへのリンクを列の形(relations.link-input-form)で書き、欄にはリンクの行き先の名前を見せるべき(リンクのリストの列も同じ)
`req~cell-edit.new-note-form-link-candidates~1` P1
- Rationale: 新しいノートの窓のリンクの欄だけ、候補がその列に今ある値(`[[…]]` の生の形。切れたリンクも混ざる)で、まだどのノートからも指されていないノートを選べなかった(2026-11-25 に読書の記録の台帳で、新しい抜き書きの出典にまだ抜き書きの無い本を選べなかった)
- Acceptance:
  - Given: Library/ に「霧の灯台守」と「雨の日の数学」があり、Quotes/ の抜き書きの source はどれも `[[霧の灯台守]]`(雨の日の数学を指す抜き書きは無い)
  - When: 新しいノートの窓で source の欄を開く
  - Then: 候補に「霧の灯台守」と「雨の日の数学」が名前で並び、`[[…]]` の形の候補は無い。
- Acceptance:
  - Given: 同じ窓
  - When: 候補の「雨の日の数学」を選んで決め、作る
  - Then: 作ったノートに `source: "[[雨の日の数学]]"`、決めたあとの source の欄には「雨の日の数学」が見える。
- Acceptance:
  - Given: News/ のニュースの question がどれも Questions/ の問いへのリンクのリスト
  - When: 新しいノートの窓で question の欄を開く
  - Then: 付け外しの窓の候補は Questions/ の問いの名前で、どのニュースからも指されていない問いも並ぶ。
- Needs: utest, stest

#### 新しいノートの窓の欄は、同じ列のセルを直すとき(cell-edit.column-type・cell-edit.value-list-open・cell-edit.calendar・cell-edit.list-select)と同じ候補と入れ方で開くべき(チェックボックスの列は true・false・なし の候補、日付・日時・数と候補のある欄は、開いた直後に打つと前もって入れた値を置き換える)
`req~cell-edit.new-note-form-same-input~1` P1
- Rationale: 同じ列でも、セルと新しいノートの窓とで候補と打ったときの振る舞いが違い、窓の中だけ別の操作を覚える要があったため(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: ask = [fav](fav はチェックボックスの列)の窓
  - When: fav の欄を開く
  - Then: 候補に true・false・なし が並ぶ(セルに Tab で着いたときと同じ)。
- Acceptance:
  - Given: `[new_note.set] due = "{date+7}"`、ask = [due](due は日付の列)の窓で、due の欄に 7日後の日付が入っている
  - When: `2026-12-01` と打って Enter
  - Then: 作ったノートの due は 2026-12-01(前もって入れた日付に続けて打たない)。
- Acceptance:
  - Given: ask = [status](`[rules.status] values = ["todo", "done"]`)、`[new_note.set] status = "todo"` の窓
  - When: status の欄で `done` と打って Enter
  - Then: 作ったノートの status は done。
- Needs: utest

#### 新しいノートの窓で、欄の上に候補の窓(値の候補のリスト・カレンダー・リストの値の付け外しの窓)が出ているときの Esc は、その候補の窓だけを閉じ、新しいノートの窓と、ほかの欄の値・その欄に打った文字を残すべき
`req~cell-edit.new-note-form-esc-popup~1` P1
- Rationale: 候補を閉じるつもりの Esc で新しいノートの窓ごと閉じ、打った名前や URL が消えた(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: `[rules.status] values = ["todo", "done"]`、ask = [url, status] の窓で、名前と url を打ち、status の欄に候補のリストが出ている
  - When: Esc
  - Then: 候補のリストが閉じ、窓は開いたまま status の欄の入力が開いていて、名前と url の欄の値が見える。
- Acceptance:
  - Given: 同じ窓で候補のリストを閉じた
  - When: Esc
  - Then: 窓が閉じ、ノートは作られない(cell-edit.new-note-form-esc)。
- Acceptance:
  - Given: ask = [due](due は日付の列)の窓で、due の欄にカレンダーが出ている
  - When: Esc
  - Then: カレンダーが閉じ、due の欄の入力は開いたまま。
- Acceptance:
  - Given: ask = [tags, priority](tags はリストの列)の窓で、tags の欄の付け外しの窓が出ていて、「仕事」にチェックを付けた
  - When: Esc
  - Then: 付け外しの窓が閉じて付けたチェックは入らず、新しいノートの窓は開いたまま tags の欄にいる。
- Acceptance:
  - Given: 同じ窓で付け外しの窓を閉じた
  - When: Enter
  - Then: tags の欄の付け外しの窓がまた開く(Tab なら priority の欄へ移る)。
- Needs: utest, stest

#### 新しいノートの窓の欄に出した作れない理由の印(cell-edit.new-note-form-required・cell-edit.new-note-form-bad-name)は、その欄の値を直して決めたら消すべき(直した値でもまだ作れなければ、作るときにまた出す)
`req~cell-edit.new-note-form-error-clear~1` P1
- Rationale: 必須の欄を空のまま作ろうとして出た「! 必須」の印が、値を入れたあとも残り、まだ作れないように見えた(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: `required = ["due"]`、ask = [due, priority] の窓で、due を空のまま Ctrl+S して due の欄に「! 必須」の印が出た
  - When: due の欄に `2026-10-05` と打って Enter
  - Then: priority の欄に移り、due の欄に `!` の印は無い。
- Acceptance:
  - Given: 名前を「c」に決めて priority の欄にいるあいだに、ほかの道具で `c.md` が作られ、Ctrl+S で名前の欄に戻って理由の印が出た
  - When: 名前を「d」に直して Enter
  - Then: priority の欄に移り、名前の欄に `!` の印は無い。
- Acceptance:
  - Given: 同じ `required = ["due"]` の窓で due の欄に「! 必須」の印が出た
  - When: due の欄を空のまま Enter
  - Then: 印は残る(まだ空のため)。
- Needs: utest

#### 新しいノートの窓の欄に出す候補の窓(値の候補のリスト・カレンダー・リストの値の付け外しの窓)は、新しいノートの窓に重ねず、窓の右(今の欄の行の下から。入らなければ上へ)に置き、右に入らなければ窓の下、それも入らなければ窓の上に置くべき。どこにも入らなければ、今の欄の行の下(入らなければ上)に、欄の名前より右に重ねるべき
`req~cell-edit.new-note-form-popup-place~1` P1
- Rationale: 読書の記録で新しい抜き書きの出典の欄を開くと、29 件の候補の窓が新しいノートの窓の左の欄の名前(「ページ」が「ペー」に)と窓の下の部分に重なった(2026-11-25 の実地の操作で見つかった)。セルを直すときの候補が直しているセルを隠さないのと同じく、どの欄に何を入れているかを見たまま選べるようにする。右を先にするのは、今の欄の行のすぐ横で目を動かさずに選べるため。右に入るかは、候補の窓の幅を 24 桁(それより狭い窓はその幅)まで縮めて入るかで決め、縮めたときは長い候補の名前の後ろを切る。上下に入るかは、セルを直すときの候補と同じく、少なくとも3行(候補が少なければその数)見せられるかで決める
- Acceptance:
  - Given: 幅 160・高さ 40 の画面で、ask = [source, page, quote] の新しいノートの窓
  - When: source の欄を開き、29 件の候補(1つは 70 字の名前)のリストが出る
  - Then: 候補の窓の左の縁は新しいノートの窓の右の縁より右にあり、新しいノートの窓の欄の名前と下の縁の行はどれも欠けない。
- Acceptance:
  - Given: 幅 90・高さ 40 の画面で同じ窓
  - When: source の欄を開く
  - Then: 候補の窓の上の縁は、新しいノートの窓の下の縁より下にある。
- Acceptance:
  - Given: 幅 90・高さ 24 の画面で、欄が 14 ある新しいノートの窓(窓の右にも下にも候補の窓が入らない)
  - When: 2つ目の欄(source)を開く
  - Then: 候補の窓の下の縁は、新しいノートの窓の上の縁より上にあり、新しいノートの窓は欠けない。
- Acceptance:
  - Given: 新しいノートの窓の右にも下にも上にも候補の窓が入らない
  - When: 欄の候補の窓を置く
  - Then: 候補の窓は今の欄の行より下にあり、左の縁は欄の値の桁より左に出ない(欄の名前は欠けない)。
- Acceptance:
  - Given: 幅 90・高さ 40 の画面で、ask = [due](due は日付の列)の窓
  - When: due の欄を開く
  - Then: カレンダーは新しいノートの窓に重ならない。
- Acceptance:
  - Given: 幅 90・高さ 40 の画面で、ask = [tags](tags はリストの列)の窓
  - When: tags の欄を開く
  - Then: 付け外しの窓は新しいノートの窓に重ならない。
- Needs: utest, stest

#### 新しいノートの窓の設定は、ビューごとにも決められてよい(mdgrid のビュー、base-view.own-views)
`req~cell-edit.new-note-form-per-view~1` P1
- Acceptance:
  - Given: 設定に ask = [priority]、あるビューの `[table.view.new_note]` に ask = [due]
  - When:
  - Then: そのビューの窓は 名前・due の欄、ほかのビューは 名前・priority の欄。

#### 新しいノートを作る既定のキーは、表のモードの `a` としてよい
`req~cell-edit.new-note-config-key~1` P1
- Rationale: `n` は検索の次の一致に使うため
- Acceptance:
  - Given:
  - When: 既定で `a`
  - Then: 名前の欄。

#### 新しいノートを作る動作の名前は `new_note` としてよい
`req~cell-edit.new-note-config-action~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `[keys.table] "n" = "new_note"` で `n` でも作れる。

#### 設定の `[new_note]` の表に、`folder`(既定のフォルダ。開いたフォルダからの相対)を持ってよい
`req~cell-edit.new-note-config-folder~1` P1
- Acceptance:
  - Given: 開いたフォルダ `notes` で `[new_note] folder = "inbox"`
  - When:
  - Then: 「メモ」は `notes/inbox/メモ.md` に作られる。

#### 設定の `[new_note]` の表に、`name`(名前の雛形。`{date}` で今日の日付)を持ってよい
`req~cell-edit.new-note-config-name~1` P1
- Acceptance:
  - Given: 今日が 2026-10-02 のとき `name = "{date} "`
  - When:
  - Then: 名前の欄に `2026-10-02 ` が入って出る。

#### 設定の `[new_note]` の表に、`ask`(窓に出す列の並び)を持ってよい
`req~cell-edit.new-note-config-ask~1` P1
- Acceptance:
  - Given: `ask = ["priority", "due"]`
  - When:
  - Then: 窓に 名前・priority・due の順で欄が並ぶ。

#### 設定の `[new_note]` の表に、`required`(空では作らない列)を持ってよい
`req~cell-edit.new-note-config-required~1` P1
- Acceptance:
  - Given:
  - When: `required = ["due"]` で due を空にして Ctrl+S
  - Then: 作らずに理由。

#### 設定の `[new_note]` の表に、`hidden`(窓に出さずに値だけ入れる列)を持ってよい
`req~cell-edit.new-note-config-hidden~1` P1
- Acceptance:
  - Given: `hidden = ["status"]`・`[new_note.set] status = "todo"`
  - When:
  - Then: 窓に status の欄は出ず、作ったノートに `status: todo`。

#### 設定の `[new_note.set]` に、前もって入れる値(列 = 値)を持ってよい
`req~cell-edit.new-note-config-set~1` P1
- Acceptance:
  - Given:
  - When: 絞り込みの無いビューで `[new_note.set] status = "doing"`
  - Then: 窓の status に doing が入り、作ったノートに `status: doing`。

#### `[new_note.set]` の文字の値には、`{date}`(今日)・`{date+N}`(N 日後。`{date-N}` は前)・`{now}`(今の日時)を書いてよい
`req~cell-edit.new-note-config-set-vars~1` P1
- Acceptance:
  - Given: `[new_note.set] due = "{date+7}"`
  - When:
  - Then: 窓の due に7日後の日付。

#### mdgrid のビュー(views.toml)の `[table.view.new_note]` に `[new_note]` と同じ形で書けば、そのビューではそちらを使ってよい
`req~cell-edit.new-note-config-view~1` P1
- Acceptance:
  - Given: 設定に `[new_note] folder = "inbox"`、あるビューの `[table.view.new_note]` に `folder = "done"`
  - When:
  - Then: そのビューで作ると `done/` に、ほかのビューでは `inbox/` に作られる。

#### 前もって入れる値は、同じ列の設定の値より絞り込みの値を先にしてよい
`req~cell-edit.new-note-config-filter-first~1` P1
- Rationale: 作った行が絞り込みに残るようにするため
- Acceptance:
  - Given: 「status が todo」のビューで `[new_note.set] status = "doing"`
  - When:
  - Then: 作ったノートは `status: todo`。

### 型ごとの入り方

#### テキストの列で、読み込んだノート全部(base-view.root-folder の範囲。ビューの絞り込みの前)の同じキーに現れる異なる値が一定数以下(既定 20)なら、その値のリストから選べるようにするべき
`req~cell-edit.value-list-open~2` P1
- Acceptance:
  - Given:
  - When: status の値 `doing` のセルで Enter
  - Then: リスト。ビューの絞り込みで見えていないノートの値も候補に入る。
- Acceptance:
  - Given: 異なる値が 21 個の列
  - When:
  - Then: リストを出さず1行の入力。

#### 値のリストからは、自由入力にも切り替えられるべき
`req~cell-edit.value-list-free-input~1` P1
- Acceptance:
  - Given:
  - When: status の値 `doing` のセルのリストで `blocked` と打って Enter
  - Then: `blocked` のためる変更。

#### 値のリストでは、今の値に印を付けるべき
`req~cell-edit.value-list-mark~1` P1
- Acceptance:
  - Given: `status` の値が todo・doing・done だけで今の値が doing
  - When:
  - Then: リストが出て、今の値の doing に印。

#### 値のリストには、空の値を「なし」として並べるべき
`req~cell-edit.value-list-none~1` P1
- Acceptance:
  - Given: `status` の値が todo・doing・done だけ
  - When:
  - Then: 3つと「なし」のリストが出る。

#### 値のリストを開いた直後(リストで選び直したあとも)に文字を打ったときは、今の値を消して打った文字から自由入力を始めるべき
`req~cell-edit.value-list-type-replaces~2` P1
- Rationale: 候補に無い新しい値を打つのに、今の値が邪魔をしないため
- Acceptance:
  - Given:
  - When: status の値 `doing` のセルのリストで ↓ のあと `x` と打つ
  - Then: `x`。
- Acceptance:
  - Given:
  - When: ← を押してから `!` と打つ
  - Then: `doing!`。
- Acceptance:
  - Given:
  - When: Ctrl+R のあとに打つ
  - Then: 今の値の続き。

#### 自由入力で打っている間は、候補のうち打った文字を含むもの(大文字小文字を問わない)を、選んでいない形でリストに出し続けるべき
`req~cell-edit.value-list-narrow~2` P1
- Acceptance:
  - Given:
  - When: `do` と打つ
  - Then: doing と done だけの候補が選びなしで出る(`DO` でも同じ)。
- Acceptance:
  - Given: どの候補にも当たらない文字
  - When:
  - Then: 候補は出ない。

#### 自由入力で打っている間は、↑↓ で絞り込んだ候補の中から選べるべき(打った文字のまま Enter なら打った文字を書く)
`req~cell-edit.value-list-narrow-pick~2` P1
- Acceptance:
  - Given:
  - When: `do` のあと ↓ → doing を選び Enter
  - Then: doing。
- Acceptance:
  - Given:
  - When: `do` と打ったまま Enter
  - Then: `do`。

#### チェックボックスの列は、1回の操作で、真と偽を切り替えるべき
`req~cell-edit.checkbox-toggle~2` P1
- Rationale: Obsidian のチェックボックスと同じ手触りにするため
- Acceptance:
  - Given:
  - When: `done: false` のセルで Enter
  - Then: `done: true`。
- Acceptance:
  - Given:
  - When: `done: true` で Enter
  - Then: `done: false`。
- Acceptance:
  - Given:
  - When: 全部の行を選んで Enter
  - Then: 今の行の次の状態(true の次は false、ほかは true)が全部の行に入る(cell-edit.bulk-set)。

#### チェックボックスの列で、値が空かキーの無いノートを切り替えたら真にするべき
`req~cell-edit.checkbox-empty-true~2` P1
- Acceptance:
  - Given:
  - When: キーの無いノートで Enter → `done: true`、もう一度
  - Then: `done: false`。
- Acceptance:
  - Given:
  - When: `done:` の空のセルで Enter
  - Then: `done: true`。

#### チェックボックスの列を空(cell-edit.empty-is-null と同じく `key:`)にするのは、空にする操作(BS・Delete)だけにするべき
`req~cell-edit.checkbox-clear~1` P1
- Acceptance:
  - Given: `done: true` のセルで BS
  - When:
  - Then: `done:`。

#### 日付の列は、`YYYY-MM-DD` と、今日からの日数(`+3`)と、空(値を消す。キーは残して null と書く。cell-edit.empty-is-null)を受け付けるべき
`req~cell-edit.date-input-forms~2` P1
- Acceptance:
  - Given: due のセル
  - When: `2026-11-03` と打って確定
  - Then: `due: 2026-11-03`。
- Acceptance:
  - Given: due のセル
  - When: `+3` と打って確定
  - Then: 今日の3日後。
- Acceptance:
  - Given: due のセル
  - When: 空で確定
  - Then: `due:` の行が残る。

#### 日付の列は、実在しない日付を受け付けるべきでない
`req~cell-edit.date-input-invalid~1` P1
- Acceptance:
  - Given: `2026-02-30`
  - When:
  - Then: 閉じずに不正を表示。

#### 日付の入力を開いた直後に文字を打ったときは、今の日付の続きでなく打った文字から入力を始めるべき
`req~cell-edit.date-input-type-replaces~1` P1
- Rationale: `+3` が今日からの日数として読めるようにするため
- Acceptance:
  - Given:
  - When: `due: 2026-10-05` のセルで Enter のあと `+3`
  - Then: 今日の3日後。

#### 日付の列のセルを編集するときは、入力ボックスに加えてカレンダーを出し、カレンダーで日を選んで決めても、入力ボックスに打ち込んでも(cell-edit.date-input の形)値を入れられるべき
`req~cell-edit.calendar~2` P1
- Acceptance:
  - Given:
  - When: due のセルで Enter
  - Then: 入力ボックスの下に今の値の月のカレンダーが出て、今の値の日が選ばれている。
- Acceptance:
  - Given:
  - When: → で次の日 → Enter
  - Then: その日がためる変更。
- Acceptance:
  - Given:
  - When: `2026-11-03` と打って Enter
  - Then: 打った日がためる変更。打っている間はカレンダーが打った日の月に動く。

#### カレンダーは、←→で1日、↑↓で1週、PageUp・PageDown で1か月動かすべき
`req~cell-edit.calendar-keys-move~2` P1
- Acceptance:
  - Given: 2026-10-30 で ↓
  - When:
  - Then: 2026-11-06。
- Acceptance:
  - Given:
  - When: PageDown
  - Then: 翌月の同じ日(無ければ月末)。

#### カレンダーは、今日に戻す操作を持つべき
`req~cell-edit.calendar-keys-today~1` P1
- Acceptance:
  - Given:
  - When: 2026-10-30 のセルのカレンダーで今日に戻す
  - Then: 今日の日。

#### カレンダーは、空にする操作(cell-edit.date-input の空と同じ)を持つべき
`req~cell-edit.calendar-keys-clear~1` P1
- Acceptance:
  - Given:
  - When: due のセルのカレンダーで空にする
  - Then: null のためる変更。

#### カレンダーは、今日・元の値・選んでいる日を、色のほかに印でも見分けられるようにするべき(cell-view.not-color-only)
`req~cell-edit.calendar-keys-marks~1` P1
- Acceptance:
  - Given:
  - When: 今日が 2026-10-15 で、元の値 2026-10-30 のセルのカレンダーで 2026-10-31 を選ぶ
  - Then: 色を除いた画面の文字だけで、今日・元の値・選んでいる日の3つが違う印で見分けられる。

#### カレンダーの週の始まりは既定で日曜とし、設定で月曜にできてよい
`req~cell-edit.calendar-keys-week-start~2` P1
- Acceptance:
  - Given: 設定が無い
  - When:
  - Then: 1列目が日。
- Acceptance:
  - Given: 設定で月曜始まり
  - When:
  - Then: 1列目が月。

#### 値が空か型の合わない値(cell-view.type-mismatch)のセルでは、カレンダーに今日の月を出すべき
`req~cell-edit.calendar-keys-empty-month~1` P1
- Acceptance:
  - Given: `someday` のセル
  - When:
  - Then: 今日の月が出る。

#### 日時の列では、カレンダーで日を選んでも時刻は元の値を保つべき(元に時刻が無ければ、時刻は打ち込みで入れる)
`req~cell-edit.calendar-keys-keep-time~1` P1
- Acceptance:
  - Given: `2026-10-30T09:00` の日時のセルで
  - When:
  - Then: → `2026-10-31T09:00`。

#### カレンダーは、画面に収まらないときは出さず、打ち込みだけにするべき(cell-view.grapheme-width)
`req~cell-edit.calendar-keys-too-small~1` P1
- Acceptance:
  - Given: 幅 30 の端末
  - When:
  - Then: カレンダーが出ず、打ち込みはできる。

#### 日付の表での見せ方と打ち込みの形は、設定(cli.config-file の設定ファイル)の日付の形(区切りが `-` か `/` か、年を出すかなど)で選べるべき
`req~cell-edit.date-format-display~2` P1
- Acceptance:
  - Given: 設定で `date_format = "YYYY/MM/DD"`
  - When:
  - Then: 表の due が `2026/10/05` と出る。
- Acceptance:
  - Given: 設定が無い
  - When:
  - Then: 今までどおり `2026-10-05`。
- Acceptance:
  - Given: 読めない形の設定
  - When:
  - Then: 起動して警告し、既定の形を使う。

#### 日付の打ち込みは、設定の日付の形と `YYYY-MM-DD` の両方を受け付けるべき
`req~cell-edit.date-format-input~1` P1
- Acceptance:
  - Given: 設定で `date_format = "YYYY/MM/DD"`
  - When:
  - Then: `2026/11/03` と打っても `2026-11-03` と打っても、閉じずに確定できる(不正を表示しない)。

#### ノートに書く日付の形は、Obsidian が日付と見なす `YYYY-MM-DD` のまま変えないべき
`req~cell-edit.date-format-write~1` P1
- Rationale: ほかの形で書くと Obsidian で不正な値になるため
- Acceptance:
  - Given: 設定で `date_format = "YYYY/MM/DD"`
  - When:
  - Then: `2026/11/03` と打っても `2026-11-03` と打っても、ファイルには `due: 2026-11-03` と書かれる。

#### カレンダーは、PageUp・PageDown のほかに Shift+←・Shift+→ でも1か月動かせるべき
`req~cell-edit.calendar-month-keys-shift~2` P1
- Rationale: PageUp・PageDown の無いキーボードでも月を飛ばせるようにするため
- Acceptance:
  - Given:
  - When: 2026-10-30 のセルでカレンダーを出し Shift+ → 
  - Then: 2026-11-30。
- Acceptance:
  - Given:
  - When: 2026-10-30 から Shift+← を2回
  - Then: 2026-08-30。
- Acceptance:
  - Given:
  - When: 2026-03-31 で Shift+←
  - Then: 2026-02-28(無ければ月末)。カレンダーが出ていないときは文字のカーソルが動くだけ。

#### 月を動かすキーを、カレンダーの下の縁に示すべき
`req~cell-edit.calendar-month-keys-hint~1` P1
- Rationale: PageUp・PageDown の無いキーボードでも、月を動かすキーに気づけるようにするため
- Acceptance:
  - Given:
  - When: 日付のセルでカレンダーを出す
  - Then: カレンダーの下の縁に `Shift+←→ 月` の文字。

#### 月を動かすキーは、設定の割り当て直し(screen.key-table)で変えられてよい
`req~cell-edit.calendar-month-keys-rebind~1` P1
- Acceptance:
  - Given:
  - When: キーの表で翌月へ動かす操作を別のキーに割り当て直す → 2026-10-30 のカレンダーでそのキー
  - Then: 2026-11-30。

#### カレンダーは、Shift+↑・Shift+↓ で1年動かしてよく、そのキーをカレンダーの下の縁に示してよい
`req~cell-edit.calendar-year-keys~2` P1
- Acceptance:
  - Given:
  - When: 2026-10-30 で Shift+↓
  - Then: 2027-10-30。
- Acceptance:
  - Given:
  - When: 2028-02-29 で Shift+↓
  - Then: 2029-02-28。

#### 数値の列は、数として読める値だけを受け付けるべき
`req~cell-edit.number-only~1` P1
- Acceptance:
  - Given:
  - When: `12a`
  - Then: 閉じずに不正を表示。

#### 数値の入力を開いた直後に文字を打ったときは、今の数の続きでなく打った文字から入力を始めるべき
`req~cell-edit.number-type-replaces~1` P1
- Acceptance:
  - Given:
  - When: `priority: 1` のセルで Enter のあと `5`
  - Then: 5(15 ではない)。

#### 値を空にしたときは、Obsidian と同じく `key:`(null)と書き、キーは消さないべき
`req~cell-edit.empty-is-null~1` P1
- Acceptance:
  - Given:
  - When: `status: done` のセルを空にして保存
  - Then: `status:` の行が残る。

#### リストの値の列(tags に限らず、列の型がリストの列)は、表から要素を付け外しできるべき
`req~cell-edit.list-items-toggle~2` P1
- Acceptance:
  - Given:
  - When: tags のセルで 会議 を付けて確定
  - Then: tags に 会議 が足されたためる変更。
- Acceptance:
  - Given:
  - When: 会議 を外して確定
  - Then: tags から 会議 が消えたためる変更。

#### リストの要素の候補は、読み込んだノート全部(base-view.root-folder の範囲)のその列に今ある要素とするべき
`req~cell-edit.list-items-candidates~1` P1
- Acceptance:
  - Given:
  - When: tags のセルで Enter
  - Then: ルートフォルダの中のタグの一覧が出る。

#### リストの要素の候補は、打つたびに絞り込んで出すべき
`req~cell-edit.list-items-narrow~1` P1
- Acceptance:
  - Given:
  - When: tags のセルで「会」と打つ
  - Then: 会議・会計 だけに絞られる。

#### リストの値の列には、候補に無い要素も打って足せるべき
`req~cell-edit.list-items-new~1` P1
- Acceptance:
  - Given:
  - When: 一覧に無い 新規 を打って足す
  - Then: 新規 が足される。

#### 選んだ複数の行(navigation.select-rows)のリストの値の列に、要素を一括で足す・外すことができ、各行のほかの要素は残すべきで、一部の行だけが持つ要素は、その旨の印で示すべき
`req~cell-edit.list-bulk~2` P1
- Acceptance:
  - Given:
  - When: 3行を選び、tags に 会議 を足す
  - Then: 3行とも 会議 が足され、元のタグは残る。1行だけが持つ 本 は「一部」の印。
- Acceptance:
  - Given:
  - When: 本 を外す
  - Then: その1行から 本 が消える。

#### 要素を足し引きして書き戻すときは、元の書き方(1行の `[a, b]` か、複数行の `- a`)と字下げを保つべきで、キーが無い・値が空のノートに初めて書くときは、Obsidian と同じ複数行の形(`key:` の次の行から `  - a`)で書くべき
`req~cell-edit.list-layout~2` P1
- Acceptance:
  - Given:
  - When: `tags: [a, b]` に c を足す
  - Then: `tags: [a, b, c]`。
- Acceptance:
  - Given:
  - When: 字下げ2の `- a` の縦のリストに c を足す
  - Then: 同じ字下げの `- c` が1行増え、ほかの行は1バイトも変わらない。
- Acceptance:
  - Given:
  - When: tags の無いノートに a を足す
  - Then: `tags:` と `  - a` の2行が足される。

#### リストの値の付け外しでは、候補を件数の多い順(同じ件数は文字の順)に並べるべき
`req~cell-edit.list-candidates-order~1` P1
- Acceptance:
  - Given: 会議 が5件・本 が2件
  - When:
  - Then: 会議 が先。

#### リストの値の付け外しでは、今の値に付いている要素に印を付けるべき
`req~cell-edit.list-candidates-mark~1` P1
- Acceptance:
  - Given:
  - When: `tags: [会議]` のセルで Enter
  - Then: 一覧の 会議 に印が付き、本 には付かない。

#### リストの要素の候補の絞り込みは、大文字小文字を区別しない部分一致にするべき
`req~cell-edit.list-candidates-match~1` P1
- Acceptance:
  - Given: 候補に Meeting・notebook・会議
  - When: `MEET` と打つ
  - Then: Meeting だけに絞られる。

#### リストの値の付け外しでは、同じ要素を2つ足さないべき
`req~cell-edit.list-candidates-no-dup~1` P1
- Acceptance:
  - Given:
  - When: 会議 を持つ行で 会議 を打って足す
  - Then: 既にあれば足さない(会議 は1つのまま)。

#### リストの要素を全部外したら、cell-edit.empty-is-null と同じく `key:` と書いてキーは消さないべき
`req~cell-edit.list-candidates-empty~1` P1
- Acceptance:
  - Given:
  - When: 全部を外して保存
  - Then: `tags:` の行が残る。

#### tags の列では、打った先頭の `#` を除いて扱うべき
`req~cell-edit.list-candidates-hash~1` P1
- Acceptance:
  - Given:
  - When: `#会議` と打つ
  - Then: 会議 として扱う。

#### リストの列で1つの文字列で書かれた値(`tags: x`)は、1つの要素のリストとして開くべき
`req~cell-edit.list-candidates-scalar-open~1` P1
- Acceptance:
  - Given:
  - When: `tags: x` のセルで Enter
  - Then: x に印の付いた要素の一覧が開く。

#### 1つの文字列で書かれたリストの値は、確定したらフローのリスト(`tags: [x, y]`)に書き換えるべき
`req~cell-edit.list-candidates-scalar-write~1` P1
- Acceptance:
  - Given:
  - When: `tags: x` で y を足して保存
  - Then: `tags: [x, y]`、行のほかのバイトはそのまま。

#### cell-edit.read-only-forms の読むだけの形(複数行にまたがるフローのリスト、アンカーや別名、入れ子の要素)のリストは、開かずに形ごとの理由を出すべき
`req~cell-edit.list-candidates-read-only~1` P1
- Acceptance:
  - Given:
  - When: `[a,\n b]` のセルで Enter
  - Then: 開かずに理由。

#### 元の書き方を保って書けない形のリストは、開かずに形ごとの理由を出すべき(要素の間の空行やコメント、要素の行末のコメント、字下げの違う要素、`-` のあとのタブ、文字列でない要素や値(書くと型が変わる)、ブロックの文字で書かれた値)
`req~cell-edit.list-candidates-unkeepable~1` P1
- Acceptance:
  - Given: 要素の間にコメントのある縦のリスト・`- 2024` を含むリスト・`tags: 2024`
  - When:
  - Then: どれも開かずに、形ごとの理由。

#### リストの要素ごとのクオートは、write-back.quoting に従うべき
`req~cell-edit.list-candidates-quoting~1` P1
- Acceptance:
  - Given:
  - When: `no` を足す
  - Then: `"no"` と書かれる。

### 読むだけのセルと一括の編集

#### ファイルの属性(`file.*`)・式の列(`formula.*`)・ネストした map・ブロックスカラー・アンカーや別名を使った値・タグつきの値(`!!str` など)・複数行にまたがる値(素のスカラー・引用符つきの値・フロー形式)のセルは、編集できないようにし、なぜ読むだけかを表示するべき
`req~cell-edit.read-only-forms~2` P0
- Disposition: 踏襲(形だけを Given・When・Then に移した。振る舞いは前の版のまま)
- Acceptance:
  - Given: `file.mtime` の列のセル
  - When: Enter
  - Then: 入力が開かず、読むだけと理由を表示。
- Acceptance:
  - Given: `title: !!str 12` のセル
  - When: Enter
  - Then: 入力が開かず、読むだけと理由を表示。
- Acceptance:
  - Given: 2行にまたがる `[a,\n b]` のセル
  - When: Enter
  - Then: 入力が開かず、読むだけと理由を表示。

#### 選んだ複数の行の同じ列に、同じ値を、ためる変更としてまとめて入れられるべき
`req~cell-edit.bulk-set-same-value~2` P1
- Acceptance:
  - Given:
  - When: 3行を選び status に done
  - Then: 3つのためる変更、保存前の差分に3ファイル。
- Acceptance:
  - Given:
  - When: チェックボックスの列で全部を選んで切り替え
  - Then: 今の行の次の状態(true の次は false、ほかは true)を全部の行にそろえ、元と同じになる行は外す(write-back.revert-drops-change)。

#### まとめて入れるときは、その列のセルを直せない行(読むだけのノート(write-back.refuse-unsafe-notes)と、読むだけのセル(cell-edit.read-only-forms・cell-edit.multiline-guard))を飛ばし、飛ばした数と理由を知らせるべき
`req~cell-edit.bulk-set-skip-read-only~1` P1
- Rationale: 元の値を黙って消さないため
- Acceptance:
  - Given: 3行のうち1行が読めないフロントマターのノート(write-back.refuse-unsafe-notes)
  - When:
  - Then: 2つのためる変更と「1行を飛ばした(理由)」。フロントマターの無いノートは飛ばさず、ためる変更になる。

#### チェックボックスの列の一括の切り替え(cell-edit.checkbox)では、ためた変更が無く値の型が合わない行(cell-view.type-mismatch。1行なら入力を開く行)を飛ばし、飛ばした数と理由を知らせるべき
`req~cell-edit.bulk-set-skip-mismatch~1` P1
- Rationale: 元の値を黙って消さないため
- Acceptance:
  - Given:
  - When: a `done: true`・b `done: yes` を選んで切り替え
  - Then: a だけ `done: false`、b はそのままで「1行を飛ばした(型が合わない)」。

### 旧い仕様では次の段の表にあり、作ってテストのあるもの

#### 表のモードで、まだどのノートにも無いキーの列を、名前を打って表に足せるようにするべき(キー(既定 `A`。screen.key-table のキーの表で割り当て直せる)・パレット・操作の一覧(screen.action-menu の列の節)から)
`req~cell-edit.add-column-start~1` P1
- Acceptance:
  - Given:
  - When: 表のモードで `A`
  - Then: 列の名前を打つ欄が出る。パレットと操作の一覧の列の節からも同じ欄が出る。

#### 足した列は、空のセルの列として表の右に出すべき
`req~cell-edit.add-column-empty~1` P1
- Acceptance:
  - Given:
  - When: `A` → `reviewer` と打って Enter
  - Then: 右に `reviewer` の空の列。

#### 足した列は、セルに値を入れて保存したノートにだけキーの行を足すべき(write-back.byte-range・write-back.add-key-and-frontmatter。値を入れないノートは変えない)
`req~cell-edit.add-column-write~1` P1
- Acceptance:
  - Given:
  - When: 足した `reviewer` の列の1行に `ken` を入れて保存
  - Then: そのノートにだけ `reviewer: ken` の行。ほかのノートは1バイトも変わらない。

#### 足した列は起動の間だけ残し、ノートに値が入れば普通の列になるべき
`req~cell-edit.add-column-session~2` P1
- Acceptance:
  - Given:
  - When: `reviewer` を足して値を入れずに終了し、起動し直す
  - Then: `reviewer` の列は無い。
- Acceptance:
  - Given:
  - When: 1行に `ken` を入れて保存してから起動し直す
  - Then: `reviewer` は普通の列として出る。

#### 足す列の名前が空・`file.`・`formula.` で始まる・改行を含むときは、足さずに理由を出すべき
`req~cell-edit.add-column-bad-name~1` P1
- Acceptance:
  - Given:
  - When: `file.x`
  - Then: 理由が出て足さない。空の名前でも同じ。

#### 足す列の名前がもうある列の名前なら、その列に移るべき
`req~cell-edit.add-column-existing~1` P1
- Acceptance:
  - Given:
  - When: `status` の列がある表で `A` → `status` と打って Enter
  - Then: 列は増えず、status の列に移る。

#### 読むだけの起動(cli.readonly)では、列を足さないべき
`req~cell-edit.add-column-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で起動して `A`
  - Then: 列は足されない。

#### 表のモードで、列のキーの名前を、読み込んだ全部のノート(base-view.root-folder の範囲。ビューの絞り込みの前)に対して、新しい名前を打ってためる変更としてまとめて変えられるべき(操作の一覧(screen.action-menu)の列の節とパレットから。キーは既定で割り当てない)
`req~cell-edit.rename-delete-key-rename~1` P1
- Acceptance:
  - Given:
  - When: status の列で「名前を変える」→ `state` と打つ
  - Then: status を持つ全部のノートに、キーの名前の変更のためる変更。

#### 表のモードで、列のキーを、読み込んだ全部のノート(base-view.root-folder の範囲。ビューの絞り込みの前)から、ためる変更としてまとめて消せるべき(操作の一覧(screen.action-menu)の列の節とパレットから。キーは既定で割り当てない)
`req~cell-edit.rename-delete-key-delete~1` P1
- Acceptance:
  - Given:
  - When: status を持つノートが3個の列で「キーを消す」→ 確かめて y
  - Then: 3つのノートから status の行を消すためる変更。

#### キーの名前の変更では、もうその名前のキーを持つノートを飛ばして、数と理由を知らせるべき
`req~cell-edit.rename-delete-key-skip-existing~1` P0
- Acceptance:
  - Given:
  - When: status の名前を `state` に変える
  - Then: `state` を既に持つノートは飛ばして「1行を飛ばした(その名前のキーがある)」。

#### キーの名前の変更では、キーを書き換えられないノート(読むだけのノート(write-back.refuse-unsafe-notes)・読むだけの値(cell-edit.read-only-forms))を飛ばして、数と理由を知らせるべき
`req~cell-edit.rename-delete-key-skip-unsafe~1` P0
- Acceptance:
  - Given:
  - When: status を持つノートの1つが `chmod 444` のノート(write-back.refuse-unsafe-notes)で、status の名前を `state` に変える
  - Then: そのノートは飛ばして「1行を飛ばした(理由)」。

#### キーの削除は、数を示して確かめてから(`y`)行うべき
`req~cell-edit.rename-delete-key-confirm~1` P0
- Acceptance:
  - Given:
  - When:
  - Then: 「キーを消す」→「3個のノートから消す? y」→ y で3つのためる変更、y 以外ではためる変更は0。

#### キーの名前の変更と削除は、どちらも1手で取り消せるべき
`req~cell-edit.rename-delete-key-undo~1` P0
- Acceptance:
  - Given:
  - When: キーの名前の変更で3つのためる変更ができたあと `u`
  - Then: 3つとも戻る(`u` で全部を戻す)。

#### キーの名前の変更と削除は、保存の前の差分(write-back.pending-and-diff)で全部のファイルを見られるべき
`req~cell-edit.rename-delete-key-diff~1` P0
- Acceptance:
  - Given:
  - When: status の名前を `state` に変えて Ctrl+S
  - Then: 差分に各ファイルの `-status: todo` と `+state: todo`。

#### キーの名前の変更と削除は、`file.*`・`formula.*` の列と、読むだけの起動(cli.readonly)では出さないべき
`req~cell-edit.rename-delete-key-hidden~1` P1
- Acceptance:
  - Given:
  - When: `file.mtime` の列の操作の一覧と、`--readonly` の起動
  - Then: 「名前を変える」と「キーを消す」が出ない。

#### 日時の列のカレンダーは時刻の欄を持ち、日と時刻のどちらを選んでいるかを切り替えられるべき
`req~cell-edit.datetime-calendar-time-field~2` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30T09:00` のセルで Enter
  - Then: 時刻の欄に 09:00。
- Acceptance:
  - Given:
  - When: 時刻に切り替えると ↑ が時刻を動かし、日に戻すと
  - Then: が日を動かす。

#### カレンダーで時刻を選んでいるときは、↑↓ で15分、Shift+↑↓ で1時間動かすべき
`req~cell-edit.datetime-calendar-time-step~1` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30T09:00` のセルで時刻に切り替えて ↑ → 09:15、Shift+↑ → 10:15、確定
  - Then: `2026-10-30T10:15`。

#### カレンダーで時刻を動かしたら、入力ボックスの文字もその時刻にするべき
`req~cell-edit.datetime-calendar-time-text~1` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30T09:00` のセルで時刻に切り替えて ↑
  - Then: 入力ボックスの文字が `2026-10-30T09:15`。

#### 時刻の無い値では、カレンダーの時刻の欄を空で始めるべき
`req~cell-edit.datetime-calendar-empty-time~1` P1
- Acceptance:
  - Given:
  - When: 日時の列の `at: 2026-10-30` のセルで Enter
  - Then: 時刻の欄は空。

#### 時刻の無い値で時刻を動かさずに確定したら、日だけを選んだときと同じく時刻を書かないべき
`req~cell-edit.datetime-calendar-keep-date-only~1` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30` で時刻を動かさずに確定
  - Then: `2026-10-30` のまま。

#### 日時の列の時刻は、打ち込みでも今までどおり入れられるべき
`req~cell-edit.datetime-calendar-typing~1` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30` のセルで `2026-10-30T18:30` と打って Enter
  - Then: `2026-10-30T18:30` のためる変更。

#### カレンダーの時刻の欄への切り替えは、既定で Ctrl+O にするべき(Ctrl+T は今日に戻す。割り当て直せる)
`req~cell-edit.time-keys-switch~1` P1
- Acceptance:
  - Given:
  - When: `at: 2026-10-30T09:00` のカレンダーで Ctrl+O のあと ↑
  - Then: `2026-10-30T09:15`(日でなく時刻が動く)。

#### 時刻の欄へ切り替えるキーを、カレンダーの下の縁に出すべき
`req~cell-edit.time-keys-hint~1` P1
- Acceptance:
  - Given:
  - When: 日時の列のセルでカレンダーを出す
  - Then: カレンダーの下の縁に Ctrl+O の文字。

#### 時刻の無い値で最初に時刻を動かすときは、09:00 から動かすべき
`req~cell-edit.time-keys-first-time~1` P1
- Acceptance:
  - Given:
  - When: `2026-10-30` で Ctrl+O・↑
  - Then: `2026-10-30T09:15`。

#### カレンダーの時刻の ↑↓ は、15分の刻みにそろえるべき(09:07 で ↑ → 09:15)
`req~cell-edit.time-keys-snap~1` P1
- Acceptance:
  - Given:
  - When: `2026-10-30T09:07` で時刻に切り替えて ↑
  - Then: `2026-10-30T09:15`。

#### カレンダーで時刻が0時をまたいでも、日は変えないべき
`req~cell-edit.time-keys-midnight~1` P1
- Acceptance:
  - Given:
  - When: `2026-10-30T23:45` で時刻に切り替えて ↑
  - Then: 00:00、日は 30 のまま。

#### 秒やタイムゾーンの付いた時刻は、カレンダーで時と分だけを変えて残りを保つべき
`req~cell-edit.time-keys-seconds-tz~1` P1
- Acceptance:
  - Given: `2026-10-30T09:00:30+09:00` で ↑
  - When:
  - Then: `2026-10-30T09:15:30+09:00`。

#### 一括の編集で時刻を動かしたら、打ち込んだ時刻と同じく全部の行にその時刻を入れるべき(cell-edit.bulk-set)
`req~cell-edit.time-keys-bulk~1` P1
- Acceptance:
  - Given:
  - When: 日時の列で2行を選んで一括の編集を開き、`2026-10-30` から Ctrl+O・↑ で確定
  - Then: 2行とも `2026-10-30T09:15` のためる変更。

#### 新しいノートの名前の雛形と前もって入れる値(cell-edit.new-note-config)には、変数(`{date}`・`{date:形}`・`{date+N}`・`{date-N}`・`{time}`・`{now}`・`{weekday}`・`{name}`・`{folder}`)を書けるべき
`req~cell-edit.template-vars-list~1` P1
- Acceptance:
  - Given:
  - When: 今が 2026-10-07 20:15、窓で決めた名前が 打ち合わせ、作る場所のフォルダが Tasks
  - Then: `{date}` は 2026-10-07(今日)、`{date:YYYY/MM/DD}` は 2026/10/07(cell-edit.date-format の日付の形)、`{date+7}` は 2026-10-14、`{date-1}` は 2026-10-06(N 日後・前)、`{time}` は 20:15(今の時刻 HH:MM)、`{now}` は 2026-10-07T20:15(今の日時)、`{weekday}` はその日の曜日、`{name}` は 打ち合わせ、`{folder}` は Tasks。

#### `[new_note] body` に雛形のファイルのパス(開いたフォルダからの相対)を書けば、その中身の変数を埋めて新しいノートの本文に書くべき
`req~cell-edit.template-vars-body~1` P1
- Acceptance:
  - Given: `body = "templates/task.md"` の中身 `# {name}\n作成: {date}`
  - When:
  - Then: 本文が `# 打ち合わせ\n作成: 2026-10-07`。

#### 窓に出さない欄(`hidden`)の値も変数を埋めて書き、作成日などを作るたびに自動で入れられるべき
`req~cell-edit.template-vars-hidden~1` P1
- Acceptance:
  - Given: `hidden = ["created"]`・`set` に `created = "{now}"`
  - When:
  - Then: 作ったノートに `created: 2026-10-07T20:15`(今の日時)。

#### 雛形の知らない変数は、文字のまま残すべき
`req~cell-edit.template-vars-unknown~1` P1
- Acceptance:
  - Given: `{unknown}` はそのまま。名前の雛形 `{unknown}`
  - When:
  - Then: 名前の欄に `{unknown}` と出る。

#### 雛形の変数 `{uuid}` は、ノートごとに新しく作った版4の UUID(小文字の16進を `-` で 8-4-4-4-12 に区切った36字)にするべき
`req~cell-edit.template-vars-uuid~1` P1
- Rationale: 名前を変えても変わらない印を、作るたびに手で考えずにノートに持たせるため
- Acceptance:
  - Given:
  - When: `hidden = ["id"]`・`set` に `id = "{uuid}"` でノートを2件作る
  - Then: どちらのノートにも `id: 3f2b8c1e-9a4d-4e7b-8c21-5d6f7a8b9c0d` の形(3つ目の組の頭が `4`、4つ目の組の頭が `8`・`9`・`a`・`b` のどれか)の値が入り、2件の値は違う。

#### 雛形の変数 `{uid}` は、ノートごとに新しく作った、英小文字で始まる数字と英小文字の8字にするべき
`req~cell-edit.template-vars-uid~1` P1
- Rationale: UUID より短く、ファイルの名前の頭にも書ける印にするため(頭を英字にするのは、フロントマターで数として読まれないため)
- Acceptance:
  - Given:
  - When: `name = "{uid}_"` でノートを2件作り始める
  - Then: 名前の欄に `k3f9x2ab_` の形(英小文字1字と、数字か英小文字の7字、続けて `_`)が入って出て、2件の8字は違う。

#### 1つのノートの中では、名前の雛形・前もって入れる値・本文の雛形の `{uuid}` と `{uid}` を、それぞれ同じ値にするべき
`req~cell-edit.template-vars-id-same~1` P1
- Rationale: ファイルの名前の印とフロントマターの印を同じにして、片方からもう片方を引けるようにするため
- Acceptance:
  - Given:
  - When: `name = "{uid}_"`・`hidden = ["id"]`・`set` に `id = "{uid}"` で、名前の欄の `k3f9x2ab_` に続けて `t` と打って作る
  - Then: `k3f9x2ab_t.md` に `id: k3f9x2ab`。

#### `[new_note] name_head = "hide"`(cell-edit.new-note-name-head)のとき、変数 `{name}` は、隠した頭を付ける前の、打った名前(最後の名前の部分)にするべき(本文の雛形・前もって入れる値・`--new-note` で同じ)
`req~cell-edit.template-vars-name-head~1` P1
- Rationale: 名前の頭の日付を隠して作る台帳で、本文の雛形の `# {name}` の見出しに `20261125_テスト` と頭が入り、表で見える名前と見出しが違った(2026-11-25 の実地の操作で見つかった)
- Acceptance:
  - Given: 今日が 2026-11-25、`[new_note] name = "{date:YYYYMMDD}_"`・`name_head = "hide"`・`body` の雛形が `# {name}`
  - When: 新しいノートの窓で名前に「テスト」と打って作る
  - Then: `20261125_テスト.md` ができ、本文は `# テスト`。
- Acceptance:
  - Given: 同じ台帳
  - When: `mdgrid <台帳> --new-note テスト`
  - Then: `20261125_テスト.md` の本文は `# テスト`。
- Acceptance:
  - Given: 同じ設定で `name_head` を書かない(名前の欄に `20261125_` が入って出る)
  - When: 名前の欄を `20261125_テスト` のまま作る
  - Then: 本文は `# 20261125_テスト`(今までどおり)。
- Needs: utest, stest

#### 新しいノートの作り方は、設定の `[new_note] mode` で `form`(既定。cell-edit.new-note-form の窓)か `editor` を選べるべき
`req~cell-edit.new-note-mode-select~2` P1
- Acceptance:
  - Given: mode を書かない
  - When:
  - Then: `a` で cell-edit.new-note-form の窓。
- Acceptance:
  - Given: `mode = "editor"`
  - When:
  - Then: `a` で名前だけの窓。

#### 新しいノートの作り方は、ビューごとにも決められてよい(base-view.own-views)
`req~cell-edit.new-note-mode-per-view~1` P1
- Acceptance:
  - Given: 設定に `[new_note] mode = "form"`、あるビューの `[table.view.new_note]` に `mode = "editor"`
  - When: そのビューで `a`
  - Then: 名前だけの窓、ほかのビューではフォーム。

#### `editor` の作り方では、作る場所(cell-edit.new-note)と名前だけを聞くべき
`req~cell-edit.new-note-mode-editor-ask~1` P1
- Acceptance:
  - Given:
  - When: `mode = "editor"` で `a`
  - Then: 名前だけの窓(見えている列は欄にしない)。

#### `editor` の作り方では、前もって入れる値(絞り込み・set・hidden。cell-edit.template-vars の変数を埋める)と本文の雛形でノートを作るべき
`req~cell-edit.new-note-mode-editor-create~1` P1
- Acceptance:
  - Given:
  - When: `mode = "editor"`・`hidden = ["created"]`・set に `created = "{date}"` で、今日が 2026-10-02 のとき名前を打って Enter
  - Then: 作ったノートに `created: 2026-10-02`。

#### `editor` の作り方では、作ったノートをすぐ screen.editor の `e` と同じエディタで開くべき
`req~cell-edit.new-note-mode-editor-open~1` P1
- Acceptance:
  - Given:
  - When: `mode = "editor"` で名前を打って Enter
  - Then: ノートができ、エディタで開く頼み(`e` と同じ)が出る。

#### `editor` の作り方でエディタから戻ったら、そのノートを読み直してその行を選ぶべき
`req~cell-edit.new-note-mode-editor-return~1` P1
- Acceptance:
  - Given:
  - When: `mode = "editor"` で「メモ」を作り、エディタで `status: doing` を書いて戻る
  - Then: 表で `メモ.md` の行が選ばれ、status に doing が出る。

#### 新しいノートのフォームからも、作ってすぐエディタで開くキー(既定 Ctrl+E。割り当て直せる)で作れるべき
`req~cell-edit.new-note-mode-form-ctrl-e~1` P1
- Acceptance:
  - Given:
  - When: フォームで Ctrl+E
  - Then: 作ってエディタで開く頼み。

#### 表のモードで、選んだ行のノートのファイルの名前を、操作の一覧(screen.action-menu)とパレットの「ノートの名前を変える」で変えられるべき(キーは既定で割り当てない)
`req~cell-edit.rename-note-menu~1` P1
- Acceptance:
  - Given:
  - When: 「会議の準備」の行で「ノートの名前を変える」→「会議の準備(10月)」と打つ
  - Then: ファイルが `会議の準備(10月).md` になり、行がその名前で選ばれたまま。

#### ノートの名前の変更は、新しい名前を打ち(`.md` は付けても付けなくてもよい)、同じフォルダの中で名前だけを変えるべき
`req~cell-edit.rename-note-same-folder~1` P1
- Acceptance:
  - Given:
  - When: `プロジェクト/会議の準備.md` の行で「会議の準備(10月).md」と打つ
  - Then: `プロジェクト/会議の準備(10月).md` になる(`.md` を重ねず、フォルダはそのまま)。

#### ノートの新しい名前が空か、フォルダの区切りを含むときは、理由を出して断るべき
`req~cell-edit.rename-note-refuse-bad-name~1` P1
- Acceptance:
  - Given:
  - When: `/` を含む名前・空の名前
  - Then: 断って理由。

#### ノートの新しい名前がもうあるファイルの名前なら、理由を出して断るべき
`req~cell-edit.rename-note-refuse-existing~1` P0
- Acceptance:
  - Given:
  - When: もうある名前
  - Then: 断って理由、どちらのファイルもそのまま。

#### そのノートにためた変更がある間は、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-refuse-pending~1` P0
- Acceptance:
  - Given:
  - When:
  - Then: status をためた変更のある行で「ノートの名前を変える」→ 断る。

#### 読むだけのノート(write-back.refuse-unsafe-notes)では、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-refuse-unsafe~1` P0
- Acceptance:
  - Given:
  - When:
  - Then: `a: 1\na: 2` を持つノートの行で「ノートの名前を変える」→ 断り、ファイルの名前はそのまま。

#### 読むだけの起動(cli.readonly)では、ノートの名前の変更を断るべき
`req~cell-edit.rename-note-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で起動
  - Then: 操作の一覧とパレットに「ノートの名前を変える」が出ない。

#### ノートの名前を変えたら、開いている表と登録した表(cli.places)のノートの、古い名前を指すリンク(`[[古い名前]]`・`[[古い名前|表示]]`・`[[古い名前#見出し]]`)を新しい名前に書き換える変更をためて、その数を知らせるべき(保存の前の差分(write-back.pending-and-diff)で見られ、取り消せる)
`req~cell-edit.rename-note-links~1` P0
- Rationale: 名前の変更でリンクを黙って切らないため
- Acceptance:
  - Given:
  - When: 「会議の準備」を「会議の準備(10月)」に変える
  - Then: ほかのノートの `project: "[[会議の準備]]"` を書き換える変更が2件ためられて2件と知らされ、Ctrl+S の差分に出る。

#### 表のモードで、選んだ行のノートを、操作の一覧(screen.action-menu)とパレットの「ノートを消す」でごみ箱へ動かせるべき(キーは既定で割り当てない)
`req~cell-edit.delete-note-menu~1` P1
- Rationale: 間違って作ったノートを、mdgrid を出ずに片づけるため
- Acceptance:
  - Given:
  - When: 「メモ」の行で「ノートを消す」→ 確かめに y
  - Then: `メモ.md` が元の場所に無く、ごみ箱に同じバイトであり、表から「メモ」の行が無くなって下の行が選ばれる。

#### ノートの削除は、消すノートの名前を示して確かめ、`y` のときだけ行わなければならない(ほかの打ち込みと Esc ではやめる)
`req~cell-edit.delete-note-confirm~1` P0
- Rationale: 押し間違いでノートを失わないため
- Acceptance:
  - Given:
  - When: 「メモ」の行で「ノートを消す」→「メモ」を消すかを聞かれ、Esc
  - Then: `メモ.md` は元の場所に同じバイトで残る。

#### ノートの削除は、ファイルを消さずにごみ箱へ動かさなければならず、動かせないときはノートを元の場所に残して理由を出さなければならない
`req~cell-edit.delete-note-trash-only~1` P0
- Rationale: 消したあとで要ると分かったノートを、人が戻せるようにするため
- Acceptance:
  - Given:
  - When: ごみ箱へ動かせない状態で「ノートを消す」に y
  - Then: `メモ.md` は元の場所に同じバイトで残り、理由が出る。

#### ノートの削除は、ごみ箱に前からあるファイルを置き換えてはならない
`req~cell-edit.delete-note-trash-keep-old~1` P0
- Rationale: 前に消したノートを、同じ名前の新しい削除で失わないため
- Acceptance:
  - Given:
  - When: 中身が「古い」の `メモ.md` を消したあと、中身が「新しい」の `メモ.md` を作って消す
  - Then: ごみ箱に「古い」と「新しい」の両方が同じバイトである。

#### そのノートにためた変更がある間は、ノートの削除を断らなければならない
`req~cell-edit.delete-note-refuse-pending~1` P0
- Rationale: ためた変更を、行き先のファイルごと黙って捨てないため
- Acceptance:
  - Given:
  - When:
  - Then: status をためた変更のある行で「ノートを消す」→ 断って理由が出て、`メモ.md` は元の場所に残り、ためた変更は1件のまま。

#### 読むだけの起動(cli.readonly)では、ノートの削除を出さないべき
`req~cell-edit.delete-note-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で起動
  - Then: 操作の一覧とパレットに「ノートを消す」が出ない。

#### CSV・TSV の表では、ノートの削除を出さないか、理由を出して断るべき
`req~cell-edit.delete-note-csv~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `台帳.csv` で「ノートを消す」→ CSV では使えない理由が出て、`台帳.csv` は1バイトも変わらない。

#### 新しいノートを作ったことは、取り消しの1手(write-back.undo-pending)にし、取り消すと、作ったノートを確かめずにごみ箱へ動かすべき
`req~cell-edit.new-note-undo~2` P1
- Rationale: 間違って作ったノートを、作った直後の1手で片づけるため
- Acceptance:
  - Given:
  - When: 「メモ」を作って `u`
  - Then: `メモ.md` が元の場所に無く、ごみ箱に同じバイトであり、表から「メモ」の行が無くなる。
- Acceptance:
  - Given:
  - When: 「メモ」を作って status を直してから `u` を1回
  - Then: status の直しだけが戻り、`メモ.md` は元の場所に残る。

#### 取り消したノートの作成は、やり直し(write-back.redo-pending)で、ごみ箱から元の場所へ戻すべき
`req~cell-edit.new-note-redo~1` P1
- Rationale: 間違いだと思って取り消したが間違いでなかったときに、打ち直さずに先へ進むため
- Acceptance:
  - Given:
  - When: 「メモ」を作って `u` のあと Ctrl+R
  - Then: `メモ.md` が元の場所に作ったときと同じバイトで戻り、表で「メモ」の行が選ばれ、ごみ箱に `メモ.md` が無い。

#### ノートの削除は、取り消しの1手(write-back.undo-pending)にし、取り消すとごみ箱から元の場所へ戻し、やり直すとまたごみ箱へ動かすべき
`req~cell-edit.delete-note-undo~2` P1
- Acceptance:
  - Given:
  - When: 「メモ」を消して `u`
  - Then: `メモ.md` が元の場所に同じバイトで戻り、表で「メモ」の行が選ばれる。
- Acceptance:
  - Given:
  - When: 続けて Ctrl+R
  - Then: `メモ.md` が元の場所に無く、ごみ箱にある。

#### パレットの「ごみ箱から戻す」で、ごみ箱のノートの一覧(名前・元の場所・消した日時)を出し、選んだノートを元の場所へ戻せるべき
`req~cell-edit.trash-restore-list~2` P1
- Rationale: mdgrid を閉じたあとや、取り消しの手が流れたあとでも、消したノートを戻せるようにするため
- Acceptance:
  - Given:
  - When: `プロジェクト/メモ.md` を消して mdgrid を起動し直し、「ごみ箱から戻す」で「メモ」を選ぶ
  - Then: `プロジェクト/メモ.md` が同じバイトで戻り、表で「メモ」の行が選ばれる。
- Acceptance:
  - Given: ごみ箱が空のとき
  - When:
  - Then: 一覧を出さず、空だと知らせる。

#### ごみ箱から戻すとき(取り消し・やり直し・「ごみ箱から戻す」のどれでも)、元の場所に同じ名前のファイルがあれば、置き換えずに断らなければならない
`req~cell-edit.trash-restore-keep-existing~1` P0
- Rationale: 戻す操作で、あとから作ったノートを失わないため
- Acceptance:
  - Given:
  - When: `メモ.md` を消したあと、別の中身の `メモ.md` を作り、「ごみ箱から戻す」で「メモ」を選ぶ
  - Then: 断って理由が出て、元の場所の `メモ.md` とごみ箱の `メモ.md` はどちらも同じバイトで残る。

#### 読むだけの起動(cli.readonly)では、「ごみ箱から戻す」を出さないべき
`req~cell-edit.trash-restore-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` で起動
  - Then: パレットに「ごみ箱から戻す」が出ない。

#### ノートの削除のごみ箱は、根のフォルダの `.trash/` の下に置くべき(OS のごみ箱は使わない)
`req~cell-edit.delete-note-trash-place~1` P1
- Rationale: 「ごみ箱から戻す」の一覧と元の場所への戻しを、どの OS でも同じにできて、Obsidian のごみ箱と同じ置き場で、mdgrid が中を探さない(base-view の隠しフォルダ)ため
- Acceptance:
  - Given:
  - When: `プロジェクト/メモ.md` を消す
  - Then: 根のフォルダの `.trash/` の下に `メモ.md` が同じバイトであり、表に「メモ」の行が出ない。

#### ノートの削除は、消すノートを指すリンクを書き換えず、確かめの問いに、リンクで指しているノートの数を添えるべき
`req~cell-edit.delete-note-links~2` P1
- Rationale: リンクが切れることを消す前に知らせ、消したノートを戻せばリンクもそのまま生きるようにするため
- Acceptance:
  - Given:
  - When: `project: "[[メモ]]"` を持つノートが2つあるとき「メモ」の行で「ノートを消す」
  - Then: 確かめに 2 個と出て、y のあとも2つのノートは同じバイトで、ためた変更は0件。
- Acceptance:
  - Given: 指すノートが無いとき
  - When:
  - Then: 確かめに数を出さない。

#### 表の列の型を、設定の `[types]`(列の名前 = `"text"`・`"number"`・`"checkbox"`・`"date"`・`"datetime"`・`"list"`・`"auto"`)で決められるべきで、決めた型はルートフォルダの型の設定と推定(cell-edit.column-type)より優先するべき(範囲ごとに重ね、狭い範囲が勝つ。screen.profile-scopes)
`req~cell-edit.type-config-types~1` P1
- Acceptance:
  - Given: `[types] due = "date"`
  - When:
  - Then: due の値が文字でも日付の入力になり、日付でない値に `!`。

#### `[types]` の `"auto"` は、その列を推定に戻すべき
`req~cell-edit.type-config-auto~2` P1
- Acceptance:
  - Given:
  - When: due の型の窓で「自動」を選ぶ
  - Then: 推定の型に戻り、views.toml から due が消える。
- Acceptance:
  - Given: 全体の設定で `due = "number"`、表の範囲で `due = "auto"`
  - When:
  - Then: due は推定の型になる。

#### 表のモードで、列の見出しの型の印(screen.rich-cells)のクリック・操作の一覧(screen.action-menu)の列の節・パレットから、選んだ列の型を選び直す窓を出すべき
`req~cell-edit.type-config-dialog~1` P1
- Acceptance:
  - Given:
  - When: due の見出しの型の印をクリック
  - Then: 型の窓。

#### 型を選び直す窓には、「自動」(推定の型を添える)と各型を並べ、今の型に印を付けるべき
`req~cell-edit.type-config-dialog-items~1` P1
- Acceptance:
  - Given: `[types] due = "number"` で、推定なら日付の due の型の窓
  - When:
  - Then: 「自動」に日付が添えられて各型と並び、数に印。

#### 型を選び直す窓で選んだ型は、開いている表の範囲(views.toml の表)の `[types]` に書いて、すぐ表に効かせるべき(セルの入力・型と合わない値の印(cell-view.type-mismatch)・並べ替え・見出しの型の印)
`req~cell-edit.type-config-dialog-write~1` P1
- Acceptance:
  - Given:
  - When: due の型の窓で「数」を選ぶ
  - Then: views.toml の表に `due = "number"`、日付の値に `!`。

#### 列の型を選び直しても、ノートの値は書き換えないべき
`req~cell-edit.type-config-notes-untouched~1` P1
- Acceptance:
  - Given:
  - When: 型の窓で due を「数」にする
  - Then: ノートのファイルは1バイトも変わらない。

#### `file.*`・`formula.*` の列では、型を選び直す窓を出さないべき
`req~cell-edit.type-config-file-formula~1` P1
- Acceptance:
  - Given:
  - When: `file.mtime` の列でパレットから型を選び直す
  - Then: 型の窓は出ない。

#### `[types]` の知らない型の名前は、警告してその列を推定のままにするべき
`req~cell-edit.type-config-unknown~1` P1
- Acceptance:
  - Given: `[types] due = "when"`
  - When:
  - Then: 警告して推定。

#### 表の列の決まりを、設定の `[rules.<列の名前>]` の `required`(真偽)で書けるべきで、真なら、キーが無い・null・空の文字列・空のリストを違反とするべき
`req~cell-edit.rules-required~1` P1
- Acceptance:
  - Given: `[rules.status] required = true`
  - When:
  - Then: status の無いノートが違反(cell-view.problems の印)、`status: done` は違反でない。

#### 表の列の決まりを、`[rules.<列の名前>]` の `values`(文字の配列)で書けるべきで、値の文字(リストなら各要素の文字)がこのどれでもなければ違反とするべき(大文字小文字を区別する)
`req~cell-edit.rules-values~1` P1
- Acceptance:
  - Given: `[rules.status] values = ["todo", "doing", "done"]`
  - When:
  - Then: `status: wip` のノートが違反(cell-view.problems の印)、`status: done` は違反でない。

#### 列の決まりは、範囲ごとに項目ごとに重ね、狭い範囲が勝つべき(screen.profile-scopes)
`req~cell-edit.rules-scopes~1` P1
- Acceptance:
  - Given:
  - When: 全体で `[rules.status] values = ["todo", "doing", "done"]`、表の範囲で `values = ["todo", "done"]` を重ねる
  - Then: 表では doing も違反。

#### 列の決まりは、ノートを書き換えないべき
`req~cell-edit.rules-notes-untouched~1` P1
- Rationale: 決まりは確かめるための物で、書いてから直す流れを妨げないため
- Acceptance:
  - Given:
  - When: `[rules.status] values = ["todo", "doing", "done"]` で `status: wip` のノートを開く
  - Then: 違反の印が付くだけで、ファイルは1バイトも変わらない。

#### 列の決まりに合わない値を書くことを、止めないべき
`req~cell-edit.rules-no-block~1` P1
- Rationale: 決まりは確かめるための物で、書いてから直す流れを妨げないため
- Acceptance:
  - Given:
  - When: `[rules.status] values = ["todo", "doing", "done"]` の status のセルで `wip` と打って Enter
  - Then: `wip` のためる変更になる。

#### `values` のある列の編集では、cell-edit.value-list の数の上限によらず、その値を候補のリストに並べるべき
`req~cell-edit.rules-values-list~1` P1
- Acceptance:
  - Given: `[rules.status] values = ["todo", "doing", "done"]`
  - When: status のセルで Enter
  - Then: 候補のリストに todo・doing・done。

#### `file.*`・`formula.*` の列の決まりは、警告して使わないべき
`req~cell-edit.rules-file-formula~1` P1
- Acceptance:
  - Given: `[rules."file.name"] required = true`
  - When:
  - Then: 警告して使わない。

#### 列の決まりの知らない項目と型の違う値は、警告してその項目を使わないべき
`req~cell-edit.rules-bad-items~2` P1
- Acceptance:
  - Given: `required = "yes"`
  - When:
  - Then: 警告して required を使わない。
- Acceptance:
  - Given: `[rules.status] foo = 1`
  - When:
  - Then: 警告。

### 本文の雛形を選ぶ

#### `[new_note] body` がフォルダを指すときは、新しいノートの窓の名前の次に「雛形」の欄を出し、そのフォルダの直下の `.md` の名前を名前の順に並べるべき
`req~cell-edit.body-choose-field~2` P1
- Rationale: バグ用・調査用のように、同じ表で本文の形を選び分けて作れるようにするため
- Acceptance:
  - Given: `body = "templates"` で、`templates/` に `bug.md`・`idea.md`・`memo.txt` がある
  - When:
  - Then: 窓の名前の次に「雛形」の欄があり、候補は `bug`・`idea` の順で、`memo` は無い。
- Acceptance:
  - Given: `body` がファイルを指すとき
  - When:
  - Then: 「雛形」の欄は無い。

#### 「雛形」の欄で選んだ雛形の中身を、変数を埋めて新しいノートの本文にし、雛形の名前はノートに書かないべき
`req~cell-edit.body-choose-pick~1` P1
- Acceptance:
  - Given: `templates/idea.md` が `# {name}` と `## ねらい` の2つの見出し
  - When: 名前 `t`、雛形 `idea` で作る
  - Then: `t.md` の本文は `# t` と `## ねらい` で、ノートのどこにも `idea` の字は無い。

#### 「雛形」の欄で選ばずに作ったときは、名前の順で最初の雛形を本文にするべき
`req~cell-edit.body-choose-default~1` P1
- Acceptance:
  - Given: `templates/` に `bug.md`・`idea.md`
  - When: 名前だけ打って作る
  - Then: 本文は `bug.md` の中身。

#### 「雛形」の欄で「なし」を選んで作ったときは、本文の無いノートにするべき
`req~cell-edit.body-choose-none~1` P1
- Acceptance:
  - Given:
  - When: 「雛形」の欄の候補の最後の「なし」を選んで作る
  - Then: ノートはフロントマターだけで、本文は空。

### ある値に直したとき、一緒に入れる値

#### 設定の `[rules.<列>.then]` に `値 = { 列 = "入れる文字" }` を書いてよく、その列のセルをその値に直したら、同じ行のその列にも値をためるべき
`req~cell-edit.then-set~2` P1
- Rationale: 完了にした日のように、ある値にしたとき決まって入れる値を、手で入れずに済ませるため
- Acceptance:
  - Given:
  - When: 今日が 2026-10-06、`[rules.status.then]` に `done = { done_at = "{date}" }`(入れる文字には cell-edit.template-vars の変数を書ける)で、`status: todo` の行の status を `done` に直す
  - Then: その行に `status: done` と `done_at: 2026-10-06` のためる変更。
- Acceptance:
  - Given:
  - When: 別の行の status を `doing` に直す
  - Then: その行のためる変更は `status: doing` だけ。

#### 一緒にためた値は、元の直しと同じ1手として取り消せるべき
`req~cell-edit.then-undo~1` P1
- Acceptance:
  - Given:
  - When: status を `done` に直して `done_at` もたまったあと、取り消しを1回
  - Then: status と done_at のためる変更がどちらも無い。

#### ためた値を別の値に直すか元の値に戻したら、前の値で一緒にためた列の変更を外すべき
`req~cell-edit.then-unset~2` P1
- Rationale: 間違えて完了にした行を直したときに、完了日だけが残らないようにするため
- Acceptance:
  - Given:
  - When: status を `done` に直し、続けて `doing` に直す
  - Then: ためる変更は `status: doing` だけ。
- Acceptance:
  - Given:
  - When: status を `done` に直し、続けて元の `todo` に戻す
  - Then: その行のためる変更は無い。

#### 一括の編集でも、直した行のそれぞれに一緒に入れる値をためるべき
`req~cell-edit.then-bulk~1` P1
- Acceptance:
  - Given:
  - When: `status: todo` の2行に印を付けて status をまとめて `done` に直す
  - Then: 2行とも `status: done` と `done_at: 2026-10-06` のためる変更。

#### カンバンで札をレーンへ動かしたときも、そのレーンの値で一緒に入れる値をためるべき
`req~cell-edit.then-kanban~2` P1
- Acceptance:
  - Given:
  - When: カンバンで `todo` の札を `done` のレーンまで動かす
  - Then: その行に `done_at: 2026-10-06` のためる変更。
- Acceptance:
  - Given:
  - When: 取り消しを1回
  - Then: 札は1つ前のレーンに戻り、done_at のためる変更は無い。

#### `then` の入れる文字が空のときは、その列を空にする変更をためるべき
`req~cell-edit.then-clear~1` P1
- Acceptance:
  - Given:
  - When: `[rules.status.then]` に `todo = { done_at = "" }` で、`status: done`・`done_at: 2026-10-01` の行の status を `todo` に直す
  - Then: `status: todo` と、done_at を空にするためる変更。

#### 表でない `then` は、cli.config-file の形で警告して使わないべき
`req~cell-edit.then-invalid~1` P1
- Acceptance:
  - Given: `[rules.status]` に `then = "done"`
  - When:
  - Then: `config.toml: rules.status.then:` で始まる警告1行で、status を `done` に直してもためる変更は `status: done` だけ。

#### `then` の入れる値は、文字の代わりに `{ value = "入れる文字", if_empty = true }` の表で書いてよく、`if_empty = true` なら、入れる先の列が空(キーが無い・null・空の文字列・空のリスト。ためた値があればその値で見る)のときだけ値をためるべき(セルの直し・一括の編集・カンバンで札を動かしたときのどれでも)
`req~cell-edit.then-if-empty~1` P1
- Rationale: 読書の記録で「中断」から「読んでる」に戻すと始めた日が今日で上書きされ、調べ物で「調べ中」と別の状態を行き来すると調べ始めた日が上書きされたため(2026-11-25 の実地の操作で見つかった)。書き方は、入れる文字に印を付ける形(`"{date}?"` など)でなく表の形にした。印の形は、`?` で終わる文字を入れたいときと区別できず、印を書き落としても気づけない。表の形は、項目の名前で何をするかが読め、名前の書き誤りは警告で気づける
- Acceptance:
  - Given: 今日が 2026-10-06、`[rules.status.then]` に `reading = { started = { value = "{date}", if_empty = true } }`、`status: paused`・`started: 2026-09-01` の行と、`status: todo` で started の無い行
  - When: 2行とも status を `reading` に直す
  - Then: 1行目のためる変更は `status: reading` だけ(started は 2026-09-01 のまま)、2行目は `status: reading` と `started: 2026-10-06`。
- Acceptance:
  - Given: 同じ設定と2行で、2行に印を付けた
  - When: status をまとめて `reading` に直す
  - Then: 1行目のためる変更は `status: reading` だけ、2行目は `status: reading` と `started: 2026-10-06`。
- Acceptance:
  - Given: 同じ設定で、status のカンバン
  - When: `paused` の札(started は 2026-09-01)を `reading` のレーンへ動かす
  - Then: その行のためる変更は `status: reading` だけ。
- Acceptance:
  - Given: 同じ設定で、started の無い `status: todo` の行の started に `2026-10-01` を手で入れた(ためる変更)
  - When: status を `reading` に直し、続けて `paused` に直す
  - Then: started の `2026-10-01` のためる変更は残る(手で入れた値は、上書きも、前の値で一緒にためた値として外すことも(cell-edit.then-unset)しない)。
- Acceptance:
  - Given: 同じ設定で、started の無い `status: todo` の行
  - When: status を `reading` に直し、続けて `paused` に直す
  - Then: ためる変更は `status: paused` だけ(cell-edit.then-unset と同じく、一緒にためた started は外れる)。
- Needs: utest

#### `then` の表の形の入れる値で、`value` が無いか文字列でないもの・`if_empty` が真偽でないもの・知らない項目のあるものは、cli.config-file の形でその道筋を言う警告を出し、その列には入れないべき
`req~cell-edit.then-if-empty-invalid~1` P1
- Rationale: 書き誤った表を「空でなくても入れる」と読むと、残したい値を上書きするため。入れない方に倒し、警告で直す所を知らせる
- Acceptance:
  - Given: `[rules.status.then]` に `done = { done_at = { value = "{date}", if_emtpy = true }, note = "済" }`(`if_empty` の書き誤り)
  - When: 起動して `status: todo` の行の status を `done` に直す
  - Then: `config.toml: rules.status.then.done.done_at.if_emtpy:` で始まる警告1行で、ためる変更は `status: done` と `note: 済`(done_at には入れない)。
- Acceptance:
  - Given: `done = { done_at = { if_empty = true } }`(`value` が無い)
  - When: 起動して `status: todo` の行の status を `done` に直す
  - Then: `config.toml: rules.status.then.done.done_at:` で始まる警告1行で、ためる変更は `status: done` だけ。
- Acceptance:
  - Given: `done = { done_at = { value = "{date}", if_empty = "yes" } }`
  - When: 起動して `status: todo` の行の status を `done` に直す
  - Then: `config.toml: rules.status.then.done.done_at.if_empty:` で始まる警告1行で、ためる変更は `status: done` だけ。
- Needs: utest

### 親を完了にしたときの子

#### `children` の表(navigation.parent-child-config)で、親の行の状態の列(`children` の `status`。書かなければ kanban の column、無ければ `status`)を完了の値(`done`。書かなければ `"done"`)に直したら、`on_done`(書かなければ `"complete"`)のとおりに、まだ終わっていない(`closed` に無い。書かなければ `["done"]`)子孫を扱うべき
`req~cell-edit.children-done~1` P1
- Rationale: 親を済ませたのに子が残ると、済んでいない子が表に散らばるため。どう扱うかは使い方で違うので、設定で選べるようにする(2026-11-24 に人が「子も完了にするがデフォルト。選べてもいい。子を完了にするまでダメとか、子が完了になっていないけど大丈夫とか。設定で柔軟に変えられるのが思想」と決めた)
- Acceptance:
  - Given: `children = { key = "parent", on_done = "nope" }`
  - When: 起動する
  - Then: cli.config-file-warning-form の形で `on_done` を言う警告1行で、`"complete"` として扱う。
- Acceptance:
  - Given: `children = { key = "parent", closed = ["done", "dropped"] }` で、「設計」の子が dropped の「旧案」だけ
  - When: 「設計」を done に直す
  - Then: 「旧案」は dropped のまま(終わっているとみなす)。
- Needs: utest

#### `on_done = "complete"` では、まだ終わっていない子孫の状態も完了の値にする変更を、親と同じ1手としてため、子孫にも一緒に入れる値(cell-edit.then-set)をためるべき
`req~cell-edit.children-done-complete~1` P1
- Rationale: 保存の前の差分で、親と一緒に何が完了になるかを確かめ、取り消し1回で戻せるようにするため
- Acceptance:
  - Given: doing の「設計」の子に todo の「画面」と done の「API」、「画面」の子に todo の「部品」、`[rules.status.then] done = { done_at = "{date}" }`、今日が 2026-11-24
  - When: 「設計」の status を done に直す
  - Then: 「設計」「画面」「部品」の status が done・done_at が 2026-11-24 のためる変更になり、「API」は変わらない。
- Acceptance:
  - Given: その直しのあと
  - When: 取り消しを1回
  - Then: 4つの行とも直す前の値に戻る。
- Acceptance:
  - Given: 同じ表をカンバンで開いている
  - When: 「設計」の札を done のレーンへ動かす
  - Then: 「画面」「部品」も done のためる変更になる。
- Needs: utest

#### `on_done = "refuse"` では、まだ終わっていない子孫があれば親の状態を直さず、残っている子孫の数を添えた理由を出すべき
`req~cell-edit.children-done-refuse~1` P1
- Rationale: 子を全部済ませるまで親を済ませない使い方のため
- Acceptance:
  - Given: `on_done = "refuse"` と、todo の子が2つある「設計」
  - When: 「設計」の status を done に直す
  - Then: 何もためず、「終わっていない子が 2 件ある」旨の理由が出る。
- Acceptance:
  - Given: 同じ表で、子の2つを done にした
  - When: 「設計」の status を done に直す
  - Then: done のためる変更になる。
- Acceptance:
  - Given: 同じ設定で、「設計」と子の無い「メモ」を選んでいる
  - When: 一括の編集で status を done にする
  - Then: 「メモ」だけが done になり、直さなかった行の数(1)が理由と一緒に出る。
- Needs: utest

#### `on_done = "keep"` では、親の状態だけを直し、まだ終わっていない子孫の数を下の帯に知らせるべき
`req~cell-edit.children-done-keep~1` P1
- Rationale: 子が残っていても親を済ませてよい使い方のため。黙って済ませると、残った子に気づけない
- Acceptance:
  - Given: `on_done = "keep"` と、todo の子が2つある「設計」
  - When: 「設計」の status を done に直す
  - Then: 「設計」だけが done のためる変更になり、下の帯に「終わっていない子が 2 件ある」旨の1行が出る。
- Needs: utest

#### `children` に `closed_by = "<列>"` を書いた表では、親の完了で一緒に完了にした子孫(cell-edit.children-done-complete)に、その列へ親へのリンクを一緒にためて、親の完了の操作で閉じたと分かるようにするべき
`req~cell-edit.children-closed-by~1` P1
- Rationale: 親を戻したときに、どの子が親の操作で閉じたのか(自分で済ませた子か)が分からないと、子を戻すかどうかを決められないため(2026-11-25 に人が「その操作で閉じたと分かる」ようにと言った)
- Acceptance:
  - Given: `children = { key = "parent", closed_by = "closed_by" }` で、doing の「設計」の子に todo の「画面」と done の「API」
  - When: 「設計」を done に直す
  - Then: 「画面」は status が done、closed_by が `[[設計]]` のためる変更になり、「API」には closed_by を書かない。
- Needs: utest

#### `children` の `on_reopen`(書かなければ `"keep"`)で、完了の親を完了でない値に戻したときの子孫を決め、`"keep"` は子孫を変えず、`"reopen"` は `closed_by` がその親を指す子孫を親と同じ値に戻して `closed_by` を空にし、`"ask"` はその場で「子も戻す」「親だけ」を選ぶ窓を出すべき
`req~cell-edit.children-reopen~1` P1
- Rationale: 親を戻したとき、親の操作で一緒に閉じた子だけを戻したい使い方と、子は済んだままにしたい使い方があるため。設定で選べるようにする(2026-11-25 に人が「制御できてもいい」と言った)
- Acceptance:
  - Given: `on_reopen = "reopen"`・`closed_by = "closed_by"` で、親の完了で閉じた「画面」(closed_by が `[[設計]]`)と、自分で済ませた「API」(closed_by が空)
  - When: 「設計」を todo に戻す
  - Then: 「画面」は status が todo、closed_by が空のためる変更になり、「API」は done のまま。取り消し1回で戻る。
- Acceptance:
  - Given: `on_reopen = "ask"` の同じ表
  - When: 「設計」を todo に戻す
  - Then: 「子も戻す」「親だけ」を選ぶ窓が出て、「親だけ」なら「設計」だけが todo になる。
- Acceptance:
  - Given: `on_reopen = "reopen"` で `closed_by` を書かない
  - When: 起動する
  - Then: cli.config-file-warning-form の形で、`closed_by` が無いと戻す子が分からない旨の警告1行を出し、`"keep"` として扱う。
- Needs: utest

### 日付を言葉で入れる

#### 日付の入力(セルの編集・一括の設定・絞り込みの比べる値)は、cell-edit.date-input-forms の形に加えて、日本語と英語の日付の言葉を、文言の言語によらず受け付けるべき
`req~cell-edit.date-words~1` P1
- Rationale: 「明日」「来週の金曜」のように、カレンダーを数えずに思ったままの言葉で期限を入れるため(2026-11-20 に人が選んだ)
- Acceptance:
  - Given: 今日が 2026-11-21(土)、週の始まりが日曜
  - When:
  - Then: `明日`・`tomorrow` は 2026-11-22、`昨日`・`yesterday` は 2026-11-20、`明後日` は 2026-11-23、`3日後`・`in 3 days`・`3d`・`+3d` は 2026-11-24、`2日前`・`-2d` は 2026-11-19、`2週間後`・`+2w` は 2026-12-05、`1か月後`・`+1m` は 2026-12-21。`LANG=en_US.UTF-8` でも `明日` は 2026-11-22。
- Needs: utest

#### 曜日の言葉は、`金曜`・`金曜日`・`fri`・`friday` を今日より後の最初のその曜日、`今週の金曜`・`this fri` を今の週(dates.week_start で区切る)のその曜日、`来週の金曜`・`next fri` を次の週のその曜日とするべき
`req~cell-edit.date-words-weekday~2` P1
- Rationale: 曜日の言葉が今日を含むか、どの週を指すかで日がずれないように、決まりを1つにするため
- Acceptance:
  - Given: 今日が 2026-11-20(金)、週の始まりが日曜
  - When:
  - Then: `金曜` は 2026-11-27、`今週の金曜` は 2026-11-20、`来週の金曜` は 2026-11-27、`来週の月曜` は 2026-11-23。
- Acceptance:
  - Given: 週の始まりが月曜で今日が 2026-11-22(日)
  - When:
  - Then: `来週の月曜` は 2026-11-23、`今週の月曜` は 2026-11-16。

#### 週と月の区切りの言葉は、`来週`・`next week` を次の週の初めの日、`来月`・`next month` を次の月の1日、`月末`・`eom` を今月の最後の日、`来月末` を次の月の最後の日とするべき
`req~cell-edit.date-words-period~2` P1
- Rationale: 「来月のどこか」を、決まった日として入れられるようにするため
- Acceptance:
  - Given: 今日が 2026-11-21、週の始まりが日曜
  - When:
  - Then: `来週` は 2026-11-22、`来月` は 2026-12-01、`月末` は 2026-11-30、`来月末` は 2026-12-31。
- Acceptance:
  - Given: 今日が 2027-01-31
  - When:
  - Then: `1か月後` は 2027-02-28(月の終わりを越えない)。

#### 日付の言葉を打っているあいだは、入力の欄の横に読んだ日付(`2026-11-27(金)` の形)を出し、カレンダー(cell-edit.calendar)をその日へ動かし、読めない言葉では読めないと出すべき
`req~cell-edit.date-words-preview~2` P1
- Rationale: 確定する前に、言葉がどの日になるかを確かめられるようにするため
- Acceptance:
  - Given:
  - When: 今日が 2026-11-21、週の始まりが日曜で `来週の金` まで打つ
  - Then: 読めないと出る。
- Acceptance:
  - Given:
  - When: 続けて `曜`
  - Then: 横に `2026-11-27(金)`、カレンダーが 27 日を選ぶ。
- Acceptance:
  - Given:
  - When: Enter
  - Then: `due: 2026-11-27` をためる。
- Needs: stest

#### 読めない日付の言葉で確定しようとしたら、cell-edit.date-input-invalid と同じく閉じずに不正を出し、ノートには書かないべき
`req~cell-edit.date-words-invalid~1` P1
- Rationale: 打ち損じの言葉を、黙って別の日や文字の値として書かないため
- Acceptance:
  - Given:
  - When: `あさっての次` で Enter
  - Then: 欄は閉じず、不正を出し、ためた変更は増えない。

#### 日時の列で日付の言葉を入れたときは、日付だけを言葉の日にし、時刻は元の値を保つべき(cell-edit.calendar と同じ)
`req~cell-edit.date-words-datetime~1` P1
- Rationale: 日を動かしたいだけのときに、決めてあった時刻を消さないため
- Acceptance:
  - Given: `2026-11-21T15:00` のセルで `明日` → `2026-11-22T15:00`。元に時刻が無い
  - When:
  - Then: `2026-11-22`。

### 次の段(まだ作っていない。P2)

#### 選んだ範囲の値を、ためる変更として正規表現で置き換えてよい
`req~cell-edit.regex-replace~1` P2
- Acceptance:
  - Given:
  - When:
  - Then: `v1` → `v2` の置き換えで、選んだ行の値だけが変わる(読んで判定)。

## 未確定事項

### 本文の雛形を選ぶ欄の決め方
`tbd~body-choose-shape`
- Decider: 人
- Provisional: 選べる雛形は `body` に書いたフォルダの直下の `.md`。選ばなければ名前の順で最初の雛形。`mode = "editor"`(名前だけを聞く)では欄を出さず、最初の雛形で作る。前に選んだ雛形は覚えない。
- Basis: 2026-11-18 の話: 作るときにノートの中の雛形も選べるようにする。既定の雛形の決め方と、エディタで作るときの扱いは決まっていない。

### 一緒に入れる値(`then`)の置き場と、どこまで効かせるか
`tbd~then-scope`
- Decider: 人
- Provisional: 置き場は `[rules.<列>.then]`。効くのは画面の直し(セル・一括・チェックボックス・空にする・カンバン)で、リストの列の直しと `--apply` には効かない。入れる先に値があっても上書きする(表の形で `if_empty = true` を書いたときだけ、空の列にだけ入れる。cell-edit.then-if-empty)。入れる文字の `{name}`・`{folder}` は空になる。
- Basis: 2026-11-18 の話: あると便利な機能を探して入れていく。完了にしたら完了日を入れるのを最初に入れた。

### 新しいノートの名前の空白を、決めた文字に置き換える設定を持つか
`tbd~new-note-name-space-fill`
- Decider: 人
- Basis: 2026-11-18 の話: ファイルの名前に空白を入れるかは設定で決められ、置き換える文字も選べるとよい。既定の文字は `-` の見込み。決まっていないのは、既定で置き換えるか(今は打った名前のまま作る)、置き換えた名前のノートの欄での見せ方、設定の名前と範囲。

### 印の変数 `{uuid}`・`{uid}` の形と、ほかに用意する印の種類
`tbd~id-variable-forms`
- Decider: 人
- Provisional: `{uuid}` は版4(乱数)、`{uid}` は英小文字で始まる数字と英小文字の8字。ほかの種類(作った順に並ぶ UUID の版7・連番など)は用意しない。
- Basis: 2026-11-18 の話: UUID や uid など印を複数用意し、ファイルの名前にもフロントマターにも作れるとよい。形と、ほかの種類は決まっていない。

### 名前の雛形の `{time}`・`{now}`・`{name}`・`{folder}` をどう扱うか
`tbd~name-template-other-vars`
- Decider: 人
- Provisional: 文字のまま残す(`{time}`・`{now}` は名前に書けない `:` が入り、`{name}`・`{folder}` は名前を決める前で値が無いため)。日付の変数(`{date}`・`{date:形}`・`{date+N}`・`{date-N}`・`{weekday}`)と印の変数(`{uuid}`・`{uid}`)は埋める。

### `name_head = "hide"` で、`display.note_name` に当たる頭を自分で打った名前(`20250101_古い`)の `{name}` から、その頭を除くか
`tbd~template-vars-name-own-head`
- Decider: 人
- Provisional: 除かない(`{name}` は打った名前のまま `20250101_古い`。頭を付けなかった名前には、隠した頭が無いため)

### 親の完了で子孫を扱うのを、印の `children` が無く、ビューの設定で親子をオンにしただけの表でも効かせるか
`tbd~cell-edit.children-done-scope`
- Decider: 人
- Provisional: 効かせない(書いた `children` があるときだけ。ビューの設定の親子は並べ方だけを変える)

### `--apply` と `--set` で親を完了にしたときも子孫を扱うか
`tbd~cell-edit.children-done-cli`
- Decider: 人
- Provisional: 扱わない(画面の直しだけ。次の段)

### 親の完了で閉じた印の列の既定の名前
`tbd~cell-edit.children-closed-by-name`
- Decider: 人
- Provisional: 既定は無し(`closed_by` を書いた表だけが書く。ノートに列が黙って増えないように)
