# ノートへの書き戻し(write-back) の要求

## 目的

mdgrid は、直した値をすぐには書かない。ためておき、保存の操作で、ファイルごとの差分を見せてからまとめて書く。書くのは直したキーの値(と、無いキーやフロントマターを足す行。write-back.add-key-and-frontmatter)だけで、ほかは1バイトも変えない。他の人のノートやフォルダを壊さないことが、この道具の信用の芯になる。どのセルが編集できるかは [cell-edit](cell-edit.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/write-back/spec.md](../_history/decidespec/write-back/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: ネストした map・ブロックスカラーの値の書き換え(読むだけ)。
- 採らない: 表からの本文の編集(エディタに渡す。screen の screen.editor)。
- 次の段: 保存したあとの取り消し(ファイルを元に戻す)。

## 前提(旧い仕様で AI が置いた仮定)

- cellops の `edit.rs`(1キーを行単位で置換)は、ブロックリスト・ブロックスカラー・ネストした map・キーの重複を壊すことを実測で確かめた(2026-09-30)。そのため行単位ではなく、YAML の値の範囲(バイトの位置)を特定して置き換える方式にする(docs/design.md)。
- ためてから保存する形(write-back.pending-and-diff)は人が選んだ(推しの案で進めてよいとの返事。読みに ⚠)。既存の表の TUI(VisiData・lazysql)には保存前の差分の画面が無く、これが強みになる(2026-09-30 の調査)。

## 要求

### ためて保存する

#### 確定した値は、すぐにファイルに書かず、ためる変更にしなければならない
`req~write-back.pending-hold~1` P1
- 受け入れ: 2つのセルを直す → ファイルは変わらず、下の帯に「未保存 2」。

#### ためる変更は、色と印で示さなければならない
`req~write-back.pending-mark~1` P1
- 受け入れ: セルを直す → そのセルがためる変更の色で描かれ、`*` の印が付く。

#### 保存の操作(既定は Ctrl+S と `:w`)で、ためる変更をまとめて書き込まなければならない
`req~write-back.pending-save~1` P1
- 受け入れ: 別々のノートの2つのセルを直して `:w` → 確定で2ファイルにまとめて書かれる。

#### 保存の前に、ファイルごとの差分を見せて確認をとらなければならない
`req~write-back.pending-diff-confirm~1` P1
- 受け入れ: 別々のノートの2つのセルを直して Ctrl+S → 書く前に2ファイルの差分が出て、確定で書かれる。

#### ためる変更は、取り消しができるべき
`req~write-back.undo-pending~1` P1
- 受け入れ: 直して `u` → 元の値に戻り「未保存 0」。

#### ためる変更は、やり直しができるべき
`req~write-back.redo-pending~1` P1
- 受け入れ: 直して `u` で取り消してから Ctrl+R → 直した値に戻り「未保存 1」。

#### 取り消しの既定のキーは、`u` と Ctrl+Z にするべき
`req~write-back.undo-keys~1` P1
- 受け入れ: 直して Ctrl+Z → `u` と同じく元の値に戻る。

#### やり直しの既定のキーは、Ctrl+R と `U` と Ctrl+Shift+Z にするべき(Ctrl+Shift+Z は、区別して受け取れる端末だけで効く)
`req~write-back.redo-keys~1` P1
- 受け入れ: Ctrl+Shift+Z を区別できない端末でも `U` でやり直せる。

#### ためる変更があるまま終わろうとしたら、保存する・捨てる・戻る を確かめるべき
`req~write-back.quit-confirm~1` P0
- 受け入れ: 未保存 1 で `q` → 確認が出る。

#### 保存が止まったとき(外部の変更など)は、ためる変更を捨てずに残し、読み直してからやり直せるべき
`req~write-back.keep-on-stop~1` P0
- 受け入れ: 保存の途中で外部の変更 → 止まり、未保存の数は減らない。

#### 外やエディタ(screen.editor)でノートが変わったとき、そのノートにためた変更が無ければ読み直すべき
`req~write-back.external-reload~1` P1
- 受け入れ: ためた変更の無いノートの title を外で直す → 表の title が外で直した値になる。

#### ためた変更のあるノートが外やエディタで変わったときは、表の値を読み直しても、ためた変更は残すべき
`req~write-back.external-keep-pending~1` P0
- 受け入れ: status をためたまま外で title を直す → 表の title は外で直した値になり、status のためた値と「未保存 1」は残る。

#### ためた変更のあるノートが外やエディタで変わったときは、その行に「外で変更」の印を付けるべき
`req~write-back.external-mark~1` P1
- 受け入れ: status をためたまま外で title を直す → 行に印。

#### 外で変わったノートの保存では、差分を見せて、外の変更の上に書くか、ためた変更を捨てるかを選ばせるべき(保存の基準は write-back.conflict-stop のとおり)
`req~write-back.external-save-choice~1` P1
- 受け入れ: status をためたまま外で title を直して保存 → 止まり、差分と2つの選択肢。

#### ためた値が、そのセルの最後に読んだ値と同じになったら(直して元の値に戻した、一括の設定で元と同じ値を入れた、など)、そのセルをためる変更から外し、未保存の数・保存の差分・ためる変更の印に出すべきでない
`req~write-back.revert-drops-pending~1` P1
- 受け入れ: `status: todo` を done に直す → 「未保存 1」。もう一度 todo に直す → 「未保存 0」で、セルに `*` が無く、Ctrl+S は「変更は無い」。数の列の `3` に 3 を入れる → ためない。数の `3` と文字の `"3"` は違う値として扱う。`done: false` のチェックボックスで Enter を2回 → true から元の false に戻って「未保存 0」。

#### 元の値に戻したセルは、ファイルに書くべきでない
`req~write-back.revert-not-written~1` P0
- なぜ: 戻したつもりでも書き戻すと書式やクオートが変わりうるため
- 受け入れ: クオートの無い日付 `due: 2026-09-30` を別の日付にしてから 2026-09-30 に戻して保存 → ファイルは1バイトも変わらない。

#### ためる変更から外す操作も、取り消し・やり直しの1手にするべき
`req~write-back.revert-undoable~1` P1
- 受け入れ: `status: todo` を done に直してから、もう一度 todo に直して `u` → done に戻って「未保存 1」。

### 書き換える範囲

#### 値を書き戻すときは、対象のキーの値の範囲だけを置き換え、ファイルの他のバイト(他のキー・キーの順・コメント・空行・改行コード・本文)を1バイトも変えてはならない
`req~write-back.byte-range-value~1` P0
- 受け入れ: ブロックリスト・ネストした map・行末コメント・CRLF・日本語のキーを含むノートの `status` を書く → 書いた前後の差分が `status` の値の範囲だけ。

#### 値を空にする(null を書く)ときは、キーの後ろの区切りの空白(`:` のあとから値の前まで)も値の範囲に含め、Obsidian と同じく `key:` と書く(cell-edit.empty-is-null)
`req~write-back.byte-range-null~1` P1
- 受け入れ: `status: done` を空にする → `status:`(区切りの空白も消え、行末のコメントと改行はそのまま)。

#### キーを足すとき(write-back.add-key-and-frontmatter)は、そのキーの行(リストの値は cell-edit.list-layout の縦の形の、キーの行と要素の行)を足すことだけを許さなければならない
`req~write-back.byte-range-add-key~1` P0
- 受け入れ: tags の無いノートに tags の a を足す → `tags:` と `  - a` の2行だけが増え、ほかのバイトは同じ。

#### リストに要素を足すとき(cell-edit.list-items・cell-edit.list-layout)は、その要素の行を足すことだけを許さなければならない
`req~write-back.byte-range-add-item~1` P0
- 受け入れ: 字下げ2の `- a` の縦のリストに c を足す → `- c` の1行だけが増え、ほかのバイトは同じ。

#### フロントマターの無いノートに足すとき(write-back.add-key-and-frontmatter)は、先頭に足すフロントマターの行だけを許さなければならない
`req~write-back.byte-range-add-frontmatter~1` P0
- 受け入れ: 本文だけのノートの status に todo を書いて保存 → 先頭に `---`・`status: todo`・`---` の3行が増え、その後ろのバイトは元のファイルと同じ。

#### 足す行(キーの行・要素の行・フロントマターの行)の改行コードは、ファイルに合わせなければならない
`req~write-back.byte-range-added-newline~1` P0
- 受け入れ: CRLF のノートの縦のリストに要素を足す → 足した要素の行の改行も CRLF。

#### キーの名前を変えるとき(cell-edit.rename-delete-key)は、そのキーの行のキーの文字だけを置き換えることを許さなければならない
`req~write-back.byte-range-rename-key~1` P0
- 受け入れ: `a: 1` と `b: [x, y]` の `b` の名前を `c` に → `c: [x, y]` で、ほかのバイトは同じ。

#### キーを消すとき(cell-edit.rename-delete-key)は、そのキーの行と、その値の続きの行(ブロックのリストの要素の行・複数行の値の行)だけを消すことを許さなければならない(続きの行の後ろの空行とコメントの行は、次のキーのものとして残す)
`req~write-back.byte-range-delete-key~1` P0
- 受け入れ: ブロックのリストの `tags` を消す → `tags:` の行と要素の行だけが消え、次のキーの前のコメントは残る。

#### 書き込んでよいのは、ノートのフロントマターの第1階層のキーの値と、キーの名前の変更・キーの削除(cell-edit.rename-delete-key)だけにしなければならない
`req~write-back.frontmatter-only-top-level~1` P0
- 受け入れ: 値がネストした map の `meta` のセルに書こうとする → 読むだけで、ファイルは変わらない。

#### mdgrid は、ノートの本文・`.base`・他のファイルを書き換えてはならない
`req~write-back.frontmatter-only-no-body~1` P0
- 受け入れ: 表からどう操作しても、フロントマターの外のバイトが変わらない。

#### 新しいファイルを作るのは、新しいノート(cell-edit.new-note)と新しい `.base` への書き出し(base-view.export-base)だけにしなければならない(保存の一時ファイル(write-back.atomic-replace)と、設定と見た目の状態の置き場のファイル(cli.config-file・screen.state-store・base-view.views-toml)は除く)
`req~write-back.frontmatter-only-new-files~1` P1
- 受け入れ: セルを直して保存する → 保存のあと、ノートのフォルダのファイルの数は変わらない。

#### 既にあるファイルを置き換えてはならない(保存の一時ファイルと名前の変更によるノートの置き換え(write-back.atomic-replace)と、設定と見た目の状態の置き場のファイル(cli.config-file・screen.state-store・base-view.views-toml)は除く)
`req~write-back.frontmatter-only-no-replace~1` P0
- 受け入れ: 新しいノートの名前に既にある `a.md` と同じ名前を打つ → 作らず、`a.md` は1バイトも変わらない。

#### キーがノートに無ければ、フロントマターの末尾にそのキーの行(リストの値は cell-edit.list-layout の縦の形の、キーの行と要素の行)を足してよい
`req~write-back.add-key-at-end~1` P1
- 受け入れ: `---\ntitle: x\n---\n本文\n` の status に todo を書いて保存 → `---\ntitle: x\nstatus: todo\n---\n本文\n`。

#### フロントマターの無いノート(1行目が `---` でないノート。空のファイルも)のセルに書くときは、ファイルの先頭に `---`・そのキーの行・`---` のフロントマターを足してよい(本文のバイトは変えない。write-back.byte-range-add-frontmatter)
`req~write-back.add-frontmatter-at-head~1` P1
- 受け入れ: 本文だけの `# 見出し\n本文\n` の status に todo を書いて保存 → `---\nstatus: todo\n---\n# 見出し\n本文\n`。空のファイル → `---\nstatus: todo\n---\n`。2つのキーを書いて保存 → フロントマターは1つで2行。

#### フロントマターの無いノートに足す行の改行コードは、ファイルに合わせる(改行の無いファイルは LF)
`req~write-back.add-frontmatter-newline~1` P0
- 受け入れ: CRLF のノート → 足す3行も CRLF。

#### 1行目が `---` で始まるが読めないノートは、これまでどおり write-back.refuse-unsafe-notes で書くべきでない
`req~write-back.add-frontmatter-skip-broken~1` P0
- 受け入れ: BOM で始まるノート → 読むだけのまま。

#### 空のフロントマター(`---` が2行だけ)のノートには、2つの区切りの間にキーの行を足してよい
`req~write-back.add-key-empty-frontmatter~1` P1
- 受け入れ: `---\n---\n本文\n` の status に todo → `---\nstatus: todo\n---\n本文\n`。

#### 設定(cli.config-file の設定ファイル)の `[edit] add_frontmatter = false` では、フロントマターの無いノートと空のフロントマターのノートを読むだけにし、理由を表示するべき(既定は true)
`req~write-back.add-frontmatter-setting~1` P1
- 受け入れ: `[edit] add_frontmatter = false` → フロントマターの無いノートと空のフロントマターのノートのセルは読むだけで理由を表示。

### 書かずに止める

#### 書く直前に、保存の基準(そのノートに最初に変更をためた時点で表が持っていた内容、つまりその時点までに最後に読んだ内容で、ためたあとの読み直し(write-back.external-change)では変えない。自分で書いたあとは write-back.new-baseline の新しい基準。write-back.external-change の保存の選択で外の変更の上に書くを選んだときは、選んだ時点のファイルの内容)から、ファイルの更新時刻・大きさ・中身のハッシュのどれかが変わっていたら、書かずに止めて知らせなければならない
`req~write-back.conflict-stop~1` P0
- 受け入れ: status をためたあとに外でノートを直し、保存 → 書かれず、差分と外部の変更を示す。


#### 止まった保存で「外の変更の上に書く」を選んだときは、選んだ時点のファイルの内容を基準にして検査し、ためたキーの値だけを書いて外の変更は残さなければならない
`req~write-back.conflict-stop-overwrite-choice~1` P0
- 受け入れ: 止まった保存で「外の変更の上に書く」を選ぶ → ためたキーの値だけが書かれて外の変更は残る。

#### 自分で書いたあとは、書いた内容の更新時刻・大きさ・ハッシュを新しい基準にするべき(同じノートの2回目の保存を外部の変更と誤らない)
`req~write-back.new-baseline-after-write~1` P1
- 受け入れ: 同じノートの別のキーを続けて2回保存 → 両方書ける。

#### ノートを名前の変更で置き換える直前にも、write-back.conflict-stop の検査をもう一度するべき
`req~write-back.new-baseline-recheck~1` P0
- 受け入れ: 一時ファイルに書いたあと、名前を変えて置き換える前に外でノートを直す → 置き換えずに止まり、外で直した内容が残る。

#### 同じキーが2回以上ある・BOM で始まる・YAML として読めない・閉じの区切りが無い・UTF-8 として読めない・改行コードが混ざる・ハードリンクのあるノートには、書き込んではならず、その行を読むだけにして理由を表示しなければならない(設定で変えられてはならない)
`req~write-back.refuse-unsafe-read-only~1` P0
- なぜ: 書くと壊すか、どちらの値かを決められないため
- 受け入れ: `a: 1\na: 2` を持つノートの `a` → 編集できず理由を表示。

#### 利用者に書き込みの権限が無い(`chmod 444` など)ノートには、書き込んではならず、その行を読むだけにして理由を表示しなければならない(設定で変えられてはならない)
`req~write-back.refuse-unsafe-no-permission~1` P0
- 受け入れ: `chmod 444` のノートの status → 編集できず「書き込めない権限」と理由を表示。

#### 読むだけにしたノートの行では、操作の一覧(screen.action-menu)にも編集と空にするを出してはならない
`req~write-back.refuse-unsafe-action-menu~1` P1
- 受け入れ: `a: 1\na: 2` を持つノートの行で操作の一覧を開く → 編集と空にするが無い。

#### 変更をためたあとに書き込みの権限が無くなったノートは、保存で書かずに止めて知らせ、ためた変更を残さなければならない(write-back.keep-on-stop。設定で変えられてはならない)
`req~write-back.refuse-unsafe-perm-lost~1` P0
- 受け入れ: status をためたあとに外で `chmod 444` → 保存で止まり、ファイルの中身と権限は変わらず、未保存の数は減らない。

#### 書く前に、一時ファイルに書いた内容を読み直して、対象のキーが意図した値になり、他のキーの値が変わらないことを確かめなければならず、違えば元のファイルを置き換えずに止めなければならない
`req~write-back.verify-before-replace~1` P0
- 受け入れ: 値の書き方の誤りを仕込んだ試験 → 元のファイルが変わらない。

#### 同期の道具の競合ファイル(`.sync-conflict-` や `(conflicted copy)` を名前に含むもの)を見つけたら、表に印を付けて知らせるべき
`req~write-back.sync-conflict~1` P1
- 受け入れ: `note.sync-conflict-20260930-1.md` があるフォルダ → その行に印と案内。

### 書き方

#### 新しい値は、元の値と同じクオートの種類と同じ形(フロー形式かブロック形式か)で書かなければならない
`req~write-back.quoting-keep-style~1` P1
- 受け入れ: `title: 'a'` を b にする → `title: 'b'`(単一引用符のまま)。`tags: [a, b]` から b を外す → `tags: [a]`(フロー形式のまま)。

#### 元のクオートで安全に表せない値は、二重引用符で書かなければならない
`req~write-back.quoting-double-fallback~1` P0
- 受け入れ: クオートの無い `title: abc` を `a: b` にする → `title: "a: b"`。

#### 改行を含む入力は、受け付けてはならない
`req~write-back.quoting-no-newline~1` P0
- 受け入れ: 改行を含む文字をセルに貼って確定する → 受け付けず「未保存 0」のまま。

#### `no`・`yes`・`on`・数字に見える文字列・`: `・` #` を含む値・先頭が記号の値はクオートし、YAML 1.1 と 1.2 のどちらで読んでも型が変わらない書き方にしなければならない
`req~write-back.quoting-type-safe~1` P0
- 受け入れ: Text 型の列に `no` を入れる → `"no"` と書かれ、読み直すと文字列の no。

#### ノートを書くときは、同じフォルダの一時ファイルに書いて fsync してから名前を変えて置き換えなければならない
`req~write-back.atomic-replace-temp~1` P0
- 受け入れ: 書き込みの途中で落としても元のファイルが壊れない。

#### シンボリックリンクのノートは、辿った先のファイルを書かなければならない
`req~write-back.atomic-replace-symlink~1` P0
- 受け入れ: シンボリックリンクのノートの status を書く → リンクはリンクのまま残り、辿った先のファイルの status が変わる。

#### ノートを書くときは、ファイルの権限を保たなければならない
`req~write-back.atomic-replace-mode~1` P0
- 受け入れ: 権限が 600 のノートの status を書く → 書いたあとも 600。

#### 日付・日時の列(列の型が date か datetime)に書く日付は、元の値が囲んでいなければ(キーが無い・空のときも)、Obsidian と同じく囲まずに `YYYY-MM-DD`(日時は元の形の `YYYY-MM-DDTHH:MM` など)で書くべき
`req~write-back.date-quoting-plain~1` P1
- なぜ: 書くと型が変わるため
- 受け入れ: `due: 2026-10-05` の date の列を 2026-11-03 にして保存 → `due: 2026-11-03`。キーの無いノートに入れる → `due: 2026-11-03` の1行。

#### 日付・日時の列に書く日付は、元の値が引用符で囲んであれば、元の引用符を保つべき
`req~write-back.date-quoting-keep-quotes~1` P1
- なぜ: 書くと型が変わるため
- 受け入れ: date の列の `due: "2026-10-05"` を 2026-11-03 にして保存 → `due: "2026-11-03"`。

#### 列の型が決まらない列(型の設定(cell-edit.column-type)が無く、読み込んだどのノートにもその列の空でない値が無い列。新しいノートで聞く列(cell-edit.new-note-form)を含む)に、日付・日時の形だけの値を書くときも、囲まずに書くべき
`req~write-back.date-quoting-untyped~1` P1
- なぜ: Obsidian が日付と推すため
- 受け入れ: どのノートにも無い列 deadline を新しいノートで聞いて 2026-10-05 と答える → `deadline: 2026-10-05`。同じ列に `2026-10-05 ごろ` と答える → 囲む。

#### テキストの列(型の設定か、空でない値から text と決まった列)に日付に見える文字を書くときは、これまでどおり write-back.quoting で囲むべき
`req~write-back.date-quoting-text-column~1` P1
- なぜ: 書くと型が変わるため
- 受け入れ: テキストの列 memo に `2026-11-03` → `memo: "2026-11-03"`。

#### 書き戻しの安全を、英語(`docs/safety.md`)と日本語(`docs/safety.ja.md`)の文書に書くべき
`req~write-back.safety-docs-write~1` P1
- 受け入れ: `docs/safety.md` と `docs/safety.ja.md` のどちらにも、1バイトも変えない範囲(write-back.byte-range・write-back.frontmatter-only)、読むだけにするノートの形と理由(write-back.add-key-and-frontmatter・write-back.refuse-unsafe-notes。形ごとに例のノートを添える)、外の変更とのぶつかり(write-back.conflict-stop・write-back.new-baseline・write-back.external-change)、書き込みの手順(一時ファイル・読み直し・fsync・名前の変更・権限とシンボリックリンク。write-back.verify-before-replace・write-back.atomic-replace)の4つが書いてある(読んで判定)。

#### 文書の読むだけの形の例は試験で読み取りに通し、読むだけにならなければ試験が落ちるべき
`req~write-back.safety-docs-example-test~1` P1
- なぜ: 文書が古くなるのを防ぐため
- 受け入れ: 文書の BOM の例から BOM を消す → 落ちる。

#### 読み取りが返す読むだけの理由の種類が文書に無ければ、試験が落ちるべき
`req~write-back.safety-docs-reason-test~1` P1
- なぜ: 文書が古くなるのを防ぐため
- 受け入れ: 読み取りに読むだけの理由を1つ足して文書を直さない → 試験が落ちる。

#### 1行目が `+++` のノート(Hugo・Zola の TOML のフロントマター)には書き込まず、その行を読むだけにして、YAML でないフロントマターだと理由を表示し、フロントマターを足す設定(cell-edit.list-items)があっても(足すと生成器が読めないページになるため)足さないようにするべき
`req~write-back.toml-frontmatter~1` P0
- 受け入れ: `+++\ntitle = "x"\n+++\n` のノートの列で Enter → 編集できず理由。フロントマターを足す設定でも足さない。

## 未確定事項

