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
- Acceptance:
  - Given:
  - When: 2つのセルを直す
  - Then: ファイルは変わらず、下の帯に「未保存 2」。

#### ためる変更は、色と印で示さなければならない
`req~write-back.pending-mark~1` P1
- Acceptance:
  - Given:
  - When: セルを直す
  - Then: そのセルがためる変更の色で描かれ、`*` の印が付く。

#### 保存の操作(既定は Ctrl+S と `:w`)で、ためる変更をまとめて書き込まなければならない
`req~write-back.pending-save~1` P1
- Acceptance:
  - Given:
  - When: 別々のノートの2つのセルを直して `:w`
  - Then: 確定で2ファイルにまとめて書かれる。

#### 保存の前に、ファイルごとの差分を見せて確認をとらなければならない
`req~write-back.pending-diff-confirm~1` P1
- Acceptance:
  - Given:
  - When: 別々のノートの2つのセルを直して Ctrl+S
  - Then: 書く前に2ファイルの差分が出て、確定で書かれる。

#### ためる変更は、取り消しができるべき
`req~write-back.undo-pending~1` P1
- Acceptance:
  - Given:
  - When: 直して `u`
  - Then: 元の値に戻り「未保存 0」。

#### ためる変更は、やり直しができるべき
`req~write-back.redo-pending~1` P1
- Acceptance:
  - Given:
  - When: 直して `u` で取り消してから Ctrl+R
  - Then: 直した値に戻り「未保存 1」。

#### 取り消しの既定のキーは、`u` と Ctrl+Z にするべき
`req~write-back.undo-keys~1` P1
- Acceptance:
  - Given:
  - When: 直して Ctrl+Z
  - Then: `u` と同じく元の値に戻る。

#### やり直しの既定のキーは、Ctrl+R と `U` と Ctrl+Shift+Z にするべき(Ctrl+Shift+Z は、区別して受け取れる端末だけで効く)
`req~write-back.redo-keys~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: Ctrl+Shift+Z を区別できない端末でも `U` でやり直せる。

#### ためる変更があるまま終わろうとしたら、保存する・捨てる・戻る を確かめるべき
`req~write-back.quit-confirm~2` P0
- Rationale: 保存していない変更は人の作業そのものなので、終わるときに黙って失わないため
- Acceptance:
  - Given:
  - When: 未保存 1 で `q`
  - Then: 確認が出る。

#### 保存が止まったとき(外部の変更など)は、ためる変更を捨てずに残し、読み直してからやり直せるべき
`req~write-back.keep-on-stop~2` P0
- Rationale: 保存が止まっても、ためた作業を黙って失わないため
- Acceptance:
  - Given:
  - When: 保存の途中で外部の変更
  - Then: 止まり、未保存の数は減らない。

#### 外やエディタ(screen.editor)でノートが変わったとき、そのノートにためた変更が無ければ読み直すべき
`req~write-back.external-reload~1` P1
- Acceptance:
  - Given:
  - When: ためた変更の無いノートの title を外で直す
  - Then: 表の title が外で直した値になる。

#### ためた変更のあるノートが外やエディタで変わったときは、表の値を読み直しても、ためた変更は残すべき
`req~write-back.external-keep-pending~2` P0
- Rationale: 外の変更を読み直すときに、ためた作業を黙って失わないため
- Acceptance:
  - Given:
  - When: status をためたまま外で title を直す
  - Then: 表の title は外で直した値になり、status のためた値と「未保存 1」は残る。

#### ためた変更のあるノートが外やエディタで変わったときは、その行に「外で変更」の印を付けるべき
`req~write-back.external-mark~1` P1
- Acceptance:
  - Given:
  - When: status をためたまま外で title を直す
  - Then: 行に印。

#### 外で変わったノートの保存で、ためたキーを外でも変えていたときは、差分を見せて、外の変更の上に書くか、ためた変更を捨てるかを選ばせるべき(保存の基準は write-back.conflict-stop のとおり。別のキーだけなら write-back.merge-other-keys)
`req~write-back.external-save-choice~2` P1
- Acceptance:
  - Given: status を done にためている
  - When: 外で同じノートの status を doing に直してから保存する
  - Then: 止まり、差分と2つの選択肢が出る。
- Needs: utest

#### 外で変わったノートでも、ためたキーの今の値が、どれもそのキーを直し始めたときに読んでいた値と同じ(外の変更が別のキーだけ)なら、止めずに今のファイルの内容を基準にして、ためたキーの値だけを書き、外の変更と合わせて書いたことを知らせるべき
`req~write-back.merge-other-keys~1` P0
- Rationale: 共有のフォルダ(クラウドの同期・git)で何人かと AI が同じ台帳を書くと、別の人が別のキーを直しただけで保存が止まり、毎回選ばされていた。ためたキーに触れていなければ、どちらの変更も失わずに書ける。同じキーを両方が変えたときだけ人が決める
- Acceptance:
  - Given: status を done にためている
  - When: 外で同じノートの title を直してから保存する
  - Then: 止まらずに書き、ファイルの status は done、title は外で直した値。知らせに、外の変更と合わせて書いたノートの名前が出る。
- Acceptance:
  - Given: status を done にためたあと、外でその status を doing に直し、また todo(直し始めたときの値)に戻した
  - When: 保存する
  - Then: 止まらずに書き、status は done。
- Acceptance:
  - Given: CSV の台帳の1行目の status を done にためている
  - When: 外で2行目のセルを直してから保存する
  - Then: 止まらずに書き、1行目の status は done、2行目は外で直した値。
- Acceptance:
  - Given: CSV の台帳の1行目の status を done にためている
  - When: 外で1行目のほかのセルを直してから保存する(行の中身が変わり、どの行かを引き継げない。source の CSV の読み直しの決まり)
  - Then: 止まり、ためた変更は残る。
- Needs: utest

#### ためた値が、そのセルの最後に読んだ値と同じになったら(直して元の値に戻した、一括の設定で元と同じ値を入れた、など)、そのセルをためる変更から外し、未保存の数・保存の差分・ためる変更の印に出すべきでない
`req~write-back.revert-drops-pending~2` P1
- Acceptance:
  - Given:
  - When: `status: todo` を done に直す
  - Then: 「未保存 1」。
- Acceptance:
  - Given:
  - When: もう一度 todo に直す
  - Then: 「未保存 0」で、セルに `*` が無く、Ctrl+S は「変更は無い」。
- Acceptance:
  - Given:
  - When: 数の列の `3` に 3 を入れる
  - Then: ためない。数の `3` と文字の `"3"` は違う値として扱う。
- Acceptance:
  - Given:
  - When: `done: false` のチェックボックスで Enter を2回
  - Then: true から元の false に戻って「未保存 0」。

#### 元の値に戻したセルは、ファイルに書くべきでない
`req~write-back.revert-not-written~1` P0
- Rationale: 戻したつもりでも書き戻すと書式やクオートが変わりうるため
- Acceptance:
  - Given:
  - When: クオートの無い日付 `due: 2026-09-30` を別の日付にしてから 2026-09-30 に戻して保存
  - Then: ファイルは1バイトも変わらない。

#### ためる変更から外す操作も、取り消し・やり直しの1手にするべき
`req~write-back.revert-undoable~1` P1
- Acceptance:
  - Given:
  - When: `status: todo` を done に直してから、もう一度 todo に直して `u`
  - Then: done に戻って「未保存 1」。

### 書き換える範囲

#### 値を書き戻すときは、対象のキーの値の範囲だけを置き換え、ファイルの他のバイト(他のキー・キーの順・コメント・空行・改行コード・本文)を1バイトも変えてはならない
`req~write-back.byte-range-value~2` P0
- Rationale: 人のノートを壊さないため。他の人と共同で使える基盤として育てるには、直した値のほかは変えないと言い切れることが要る
- Acceptance:
  - Given:
  - When: ブロックリスト・ネストした map・行末コメント・CRLF・日本語のキーを含むノートの `status` を書く
  - Then: 書いた前後の差分が `status` の値の範囲だけ。

#### 値を空にする(null を書く)ときは、キーの後ろの区切りの空白(`:` のあとから値の前まで)も値の範囲に含め、Obsidian と同じく `key:` と書く(cell-edit.empty-is-null)
`req~write-back.byte-range-null~1` P1
- Acceptance:
  - Given:
  - When: `status: done` を空にする
  - Then: `status:`(区切りの空白も消え、行末のコメントと改行はそのまま)。

#### キーを足すとき(write-back.add-key-and-frontmatter)は、そのキーの行(リストの値は cell-edit.list-layout の縦の形の、キーの行と要素の行)を足すことだけを許さなければならない
`req~write-back.byte-range-add-key~2` P0
- Rationale: キーを足すときも、人のノートのほかの所を変えず、他の人と共同で使える基盤にするため
- Acceptance:
  - Given:
  - When: tags の無いノートに tags の a を足す
  - Then: `tags:` と `  - a` の2行だけが増え、ほかのバイトは同じ。

#### リストに要素を足すとき(cell-edit.list-items・cell-edit.list-layout)は、その要素の行を足すことだけを許さなければならない
`req~write-back.byte-range-add-item~2` P0
- Rationale: リストに要素を足すときも、人のノートのほかの所を変えず、他の人と共同で使える基盤にするため
- Acceptance:
  - Given:
  - When: 字下げ2の `- a` の縦のリストに c を足す
  - Then: `- c` の1行だけが増え、ほかのバイトは同じ。

#### フロントマターの無いノートに足すとき(write-back.add-key-and-frontmatter)は、先頭に足すフロントマターの行だけを許さなければならない
`req~write-back.byte-range-add-frontmatter~2` P0
- Rationale: フロントマターを足すときも、人が書いた本文を変えず、他の人と共同で使える基盤にするため
- Acceptance:
  - Given:
  - When: 本文だけのノートの status に todo を書いて保存
  - Then: 先頭に `---`・`status: todo`・`---` の3行が増え、その後ろのバイトは元のファイルと同じ。

#### 足す行(キーの行・要素の行・フロントマターの行)の改行コードは、ファイルに合わせなければならない
`req~write-back.byte-range-added-newline~2` P0
- Rationale: 足した行だけ改行コードの違うノートにして、人のノートを壊さないため
- Acceptance:
  - Given:
  - When: CRLF のノートの縦のリストに要素を足す
  - Then: 足した要素の行の改行も CRLF。

#### キーの名前を変えるとき(cell-edit.rename-delete-key)は、そのキーの行のキーの文字だけを置き換えることを許さなければならない
`req~write-back.byte-range-rename-key~2` P0
- Rationale: キーの名前を変えるときも、値とほかの行を変えず、人のノートを壊さないため
- Acceptance:
  - Given: `a: 1` と `b: [x, y]` の `b` の名前を `c` に
  - When:
  - Then: `c: [x, y]` で、ほかのバイトは同じ。

#### キーを消すとき(cell-edit.rename-delete-key)は、そのキーの行と、その値の続きの行(ブロックのリストの要素の行・複数行の値の行)だけを消すことを許さなければならない(続きの行の後ろの空行とコメントの行は、次のキーのものとして残す)
`req~write-back.byte-range-delete-key~2` P0
- Rationale: キーを消すときも、次のキーに付いた空行とコメントを巻き込まず、人のノートを壊さないため
- Acceptance:
  - Given:
  - When: ブロックのリストの `tags` を消す
  - Then: `tags:` の行と要素の行だけが消え、次のキーの前のコメントは残る。

#### 書き込んでよいのは、ノートのフロントマターの第1階層のキーの値と、キーの名前の変更・キーの削除(cell-edit.rename-delete-key)だけにしなければならない
`req~write-back.frontmatter-only-top-level~2` P0
- Rationale: 書く所を、範囲を言い切れる第1階層のキーに絞り、人のノートを壊さないため
- Acceptance:
  - Given:
  - When: 値がネストした map の `meta` のセルに書こうとする
  - Then: 読むだけで、ファイルは変わらない。

#### mdgrid は、ノートの本文・`.base`・他のファイルを書き換えてはならない(AI の案のファイル(ai.proposal-file)を、受けたか捨てたときに消すのは除く)
`req~write-back.frontmatter-only-no-body~3` P0
- Rationale: 本文と `.base` とほかのファイルは人やほかの道具が書いたものなので、mdgrid を通しただけで変えず、他の人と共同で使える基盤にするため
- Acceptance:
  - Given:
  - When:
  - Then: 表からどう操作しても、フロントマターの外のバイトが変わらない。

#### 新しいファイルを作るのは、新しいノート(cell-edit.new-note)と新しい `.base` への書き出し(base-view.export-base)と AI の案(ai.proposal-file)だけにしなければならない(保存の一時ファイル(write-back.atomic-replace)と、設定と見た目の状態の置き場のファイル(cli.config-file・screen.state-store・base-view.views-toml)は除く)
`req~write-back.frontmatter-only-new-files~2` P1
- Acceptance:
  - Given:
  - When: セルを直して保存する
  - Then: 保存のあと、ノートのフォルダのファイルの数は変わらない。

#### 既にあるファイルを置き換えてはならない(保存の一時ファイルと名前の変更によるノートの置き換え(write-back.atomic-replace)と、設定と見た目の状態の置き場のファイル(cli.config-file・screen.state-store・base-view.views-toml)は除く)
`req~write-back.frontmatter-only-no-replace~2` P0
- Rationale: 既にあるファイルは人や自分が作ったものなので、置き換えて壊さないため(安全のため)
- Acceptance:
  - Given:
  - When: 新しいノートの名前に既にある `a.md` と同じ名前を打つ
  - Then: 作らず、`a.md` は1バイトも変わらない。

#### キーがノートに無ければ、フロントマターの末尾にそのキーの行(リストの値は cell-edit.list-layout の縦の形の、キーの行と要素の行)を足してよい
`req~write-back.add-key-at-end~1` P1
- Acceptance:
  - Given:
  - When: `---\ntitle: x\n---\n本文\n` の status に todo を書いて保存
  - Then: `---\ntitle: x\nstatus: todo\n---\n本文\n`。

#### フロントマターの無いノート(1行目が `---` でないノート。空のファイルも)のセルに書くときは、ファイルの先頭に `---`・そのキーの行・`---` のフロントマターを足してよい(本文のバイトは変えない。write-back.byte-range-add-frontmatter)
`req~write-back.add-frontmatter-at-head~2` P1
- Acceptance:
  - Given:
  - When: 本文だけの `# 見出し\n本文\n` の status に todo を書いて保存
  - Then: `---\nstatus: todo\n---\n# 見出し\n本文\n`。
- Acceptance:
  - Given: 空のファイル
  - When:
  - Then: `---\nstatus: todo\n---\n`。
- Acceptance:
  - Given:
  - When: 2つのキーを書いて保存
  - Then: フロントマターは1つで2行。

#### フロントマターの無いノートに足す行の改行コードは、ファイルに合わせる(改行の無いファイルは LF)
`req~write-back.add-frontmatter-newline~2` P0
- Rationale: 足したフロントマターの行だけ改行コードの違うノートにして、人のノートを壊さないため
- Acceptance:
  - Given: CRLF のノート
  - When:
  - Then: 足す3行も CRLF。

#### 1行目が `---` で始まるが読めないノートは、これまでどおり write-back.refuse-unsafe-notes で書くべきでない
`req~write-back.add-frontmatter-skip-broken~2` P0
- Rationale: 読めないフロントマターの上にもう1つ足して、人のノートをさらに壊さないため
- Acceptance:
  - Given: BOM で始まるノート
  - When:
  - Then: 読むだけのまま。

#### 空のフロントマター(`---` が2行だけ)のノートには、2つの区切りの間にキーの行を足してよい
`req~write-back.add-key-empty-frontmatter~1` P1
- Acceptance:
  - Given: `---\n---\n本文\n` の status に todo
  - When:
  - Then: `---\nstatus: todo\n---\n本文\n`。

#### 設定(cli.config-file の設定ファイル)の `[edit] add_frontmatter = false` では、フロントマターの無いノートと空のフロントマターのノートを読むだけにし、理由を表示するべき(既定は true)
`req~write-back.add-frontmatter-setting~1` P1
- Acceptance:
  - Given: `[edit] add_frontmatter = false`
  - When:
  - Then: フロントマターの無いノートと空のフロントマターのノートのセルは読むだけで理由を表示。

### 書かずに止める

#### 書く直前に、保存の基準(そのノートに最初に変更をためた時点で表が持っていた内容、つまりその時点までに最後に読んだ内容で、ためたあとの読み直し(write-back.external-change)では変えない。自分で書いたあとは write-back.new-baseline の新しい基準。write-back.external-change の保存の選択で外の変更の上に書くを選んだときは、選んだ時点のファイルの内容)から、ファイルの更新時刻・大きさ・中身のハッシュのどれかが変わっていたら、外の変更と合わせられるとき(write-back.merge-other-keys)のほかは、書かずに止めて知らせなければならない
`req~write-back.conflict-stop~3` P0
- Rationale: 外で入った変更を知らずに上書きして、意図しない変更を引き起こさないため
- Acceptance:
  - Given: status を done にためている
  - When: 外でノートの status を doing に直してから保存する
  - Then: 書かれず、差分と外部の変更を示す。
- Needs: utest


#### 止まった保存で「外の変更の上に書く」を選んだときは、選んだ時点のファイルの内容を基準にして検査し、ためたキーの値だけを書いて外の変更は残さなければならない
`req~write-back.conflict-stop-overwrite-choice~2` P0
- Rationale: 人が見て選んだ内容の上にためたキーだけを書き、外の変更に意図しない変更を引き起こさないため
- Acceptance:
  - Given:
  - When: 止まった保存で「外の変更の上に書く」を選ぶ
  - Then: ためたキーの値だけが書かれて外の変更は残る。

#### 自分で書いたあとは、書いた内容の更新時刻・大きさ・ハッシュを新しい基準にするべき(同じノートの2回目の保存を外部の変更と誤らない)
`req~write-back.new-baseline-after-write~1` P1
- Acceptance:
  - Given:
  - When: 同じノートの別のキーを続けて2回保存
  - Then: 両方書ける。

#### ノートを名前の変更で置き換える直前にも、write-back.conflict-stop の検査をもう一度するべき
`req~write-back.new-baseline-recheck~2` P0
- Rationale: 検査から置き換えまでのあいだに入った外の変更を上書きして、意図しない変更を引き起こさないため
- Acceptance:
  - Given:
  - When: 一時ファイルに書いたあと、名前を変えて置き換える前に外でノートを直す
  - Then: 置き換えずに止まり、外で直した内容が残る。

#### 同じキーが2回以上ある・BOM で始まる・YAML として読めない・閉じの区切りが無い・UTF-8 として読めない・改行コードが混ざる・ハードリンクのあるノートには、書き込んではならず、その行を読むだけにして理由を表示しなければならない(設定で変えられてはならない)
`req~write-back.refuse-unsafe-read-only~1` P0
- Rationale: 書くと壊すか、どちらの値かを決められないため
- Acceptance:
  - Given:
  - When: `a: 1\na: 2` を持つノートの `a`
  - Then: 編集できず理由を表示。

#### 利用者に書き込みの権限が無い(`chmod 444` など)ノートには、書き込んではならず、その行を読むだけにして理由を表示しなければならない(設定で変えられてはならない)
`req~write-back.refuse-unsafe-no-permission~2` P0
- Rationale: 書き込みを許していないノートに、意図しない変更を引き起こさないため。理由を表示するのは、直せないのが故障に見えず、利用者が不安にならないようにするため
- Acceptance:
  - Given: `chmod 444` のノートの status
  - When:
  - Then: 編集できず「書き込めない権限」と理由を表示。

#### 先頭の 8,000 バイトに NUL(0x00)のバイトがある `.md` は、名前だけが Markdown のほかのファイル(実行ファイル・圧縮ファイル・画像など)とみなし、値を読まず、書き込まず、エディタにも渡さず、その行を読むだけにして理由を表示しなければならない(設定で変えられてはならない)
`req~write-back.refuse-not-text~1` P0
- Rationale: 拡張子を `.md` に変えたほかのファイル(偽装した拡張子)に、フロントマターを足したりエディタで開いたりして壊さないため。UTF-8 の文には NUL が出ないので、ふつうのノートを誤って止めない。8,000 バイトは、git がファイルを文字か否か見分けるのと同じ幅(2026-11-23 に人が「偽装拡張子は開かない。誤検知に気をつけて」と決めた)
- Acceptance:
  - Given: `PK\x03\x04` で始まり NUL のバイトを含み、UTF-8 としては読める `archive.md`
  - When: その行のセルで Enter
  - Then: 入力が開かず、中身がテキストでないという理由が出る。
- Acceptance:
  - Given: `archive.md` と、ふつうのノート2つの行を選ぶ
  - When: status に done をまとめて入れて保存する
  - Then: ふつうのノート2つだけが書かれ、`archive.md` は1バイトも変わらない。
- Acceptance:
  - Given: `archive.md` の行
  - When: `e`
  - Then: エディタは起動せず、メッセージ行に中身がテキストでないという理由が出る。
- Acceptance:
  - Given: 先頭から 8,000 番目のバイトが NUL のノートと、8,001 番目のバイトが NUL のノート
  - When: そのフォルダを開く
  - Then: 8,000 番目のノートは読むだけで、8,001 番目のノートは NUL の無いノートと同じに扱う。
- Acceptance:
  - Given: 本文が「MZ-700 の思い出」で始まり、NUL の無いノート
  - When: その行の status を直して保存する
  - Then: ふつうのノートと同じに書ける。
- Needs: utest

#### UTF-16 の BOM(`FF FE` か `FE FF`)で始まる `.md` は、その行を読むだけにし、UTF-16 で書かれていて mdgrid は UTF-8 だけを読むという理由を表示するべき
`req~write-back.refuse-utf16~1` P1
- Rationale: UTF-16 の文にも NUL が入るので、write-back.refuse-not-text の「テキストでない」と取り違えず、UTF-8 で保存し直せばよいと分かる理由を出すため。UTF-16 は文字のファイルなので、エディタには渡してよい
- Acceptance:
  - Given: `FF FE` で始まる UTF-16 のノート
  - When: その行のセルで Enter
  - Then: 入力が開かず、UTF-16 で書かれている(UTF-8 だけを読む)という理由が出る。
- Acceptance:
  - Given: 同じノートの行
  - When: `e`
  - Then: エディタが起動する。
- Needs: utest

#### ノートのほかに mdgrid が読んで書き戻すファイル(CSV・TSV の台帳と、設定のフォルダの TOML)も、文字のファイルでない(先頭の 8,000 バイトに NUL がある・UTF-16 の BOM で始まる・UTF-8 でない)ときは、中身を使わず、そのファイルに書かず、理由を出さなければならない(設定で変えられてはならない)
`req~write-back.refuse-not-text-any-file~1` P0
- Rationale: 名前だけ `.csv` や `.toml` のほかのファイルを、台帳や設定として直して上書きし、壊さないため。write-back.refuse-not-text の決まりを、読むどのファイルにも当てる(2026-11-23 に人が「どのファイルでも読み込むなら対策を入れる」と決めた)
- Acceptance:
  - Given: 2行目に NUL のバイトがあり、UTF-8 としては読める `tasks.csv`
  - When: `mdgrid tasks.csv --print`
  - Then: 表を出さず、テキストでないという理由1行と終了コード 2 で終わり、`tasks.csv` は1バイトも変わらない。
- Acceptance:
  - Given: 設定のフォルダの `places.toml` が UTF-8 でない
  - When: 画面から表を登録する
  - Then: `places.toml` は1バイトも変わらず、メッセージ行に書かない理由が出る。
- Needs: utest, stest

#### 読むだけにしたノートの行では、操作の一覧(screen.action-menu)にも編集と空にするを出してはならない
`req~write-back.refuse-unsafe-action-menu~1` P1
- Acceptance:
  - Given:
  - When: `a: 1\na: 2` を持つノートの行で操作の一覧を開く
  - Then: 編集と空にするが無い。

#### 変更をためたあとに書き込みの権限が無くなったノートは、保存で書かずに止めて知らせ、ためた変更を残さなければならない(write-back.keep-on-stop。設定で変えられてはならない)
`req~write-back.refuse-unsafe-perm-lost~2` P0
- Rationale: 書き込みを許さなくなったノートに意図しない変更を引き起こさず、ためた作業も黙って失わないため
- Acceptance:
  - Given: status をためたあとに外で `chmod 444`
  - When:
  - Then: 保存で止まり、ファイルの中身と権限は変わらず、未保存の数は減らない。

#### 書く前に、一時ファイルに書いた内容を読み直して、対象のキーが意図した値になり、他のキーの値が変わらないことを確かめなければならず、違えば元のファイルを置き換えずに止めなければならない
`req~write-back.verify-before-replace~2` P0
- Rationale: 意図した値のほかが変わった内容でノートを置き換えて、意図しない変更を引き起こさないため
- Acceptance:
  - Given: 値の書き方の誤りを仕込んだ試験
  - When:
  - Then: 元のファイルが変わらない。

#### 同期の道具の競合ファイル(`.sync-conflict-` や `(conflicted copy)` を名前に含むもの)を見つけたら、表に印を付けて知らせるべき
`req~write-back.sync-conflict~1` P1
- Acceptance:
  - Given: `note.sync-conflict-20260930-1.md` があるフォルダ
  - When:
  - Then: その行に印と案内。

### 書き方

#### 新しい値は、元の値と同じクオートの種類と同じ形(フロー形式かブロック形式か)で書かなければならない
`req~write-back.quoting-keep-style~2` P1
- Acceptance:
  - Given:
  - When: `title: 'a'` を b にする
  - Then: `title: 'b'`(単一引用符のまま)。
- Acceptance:
  - Given:
  - When: `tags: [a, b]` から b を外す
  - Then: `tags: [a]`(フロー形式のまま)。

#### 元のクオートで安全に表せない値は、二重引用符で書かなければならない
`req~write-back.quoting-double-fallback~2` P0
- Rationale: 書いた値でフロントマターが壊れるのを防ぐため。クオートを付けても困ることは少ない
- Acceptance:
  - Given:
  - When: クオートの無い `title: abc` を `a: b` にする
  - Then: `title: "a: b"`。

#### 改行を含む入力は、受け付けてはならない
`req~write-back.quoting-no-newline~2` P0
- Rationale: 改行の入った値でフロントマターが壊れるのを防ぐため
- Acceptance:
  - Given:
  - When: 改行を含む文字をセルに貼って確定する
  - Then: 受け付けず「未保存 0」のまま。

#### `no`・`yes`・`on`・数字に見える文字列・`: `・` #` を含む値・先頭が記号の値はクオートし、YAML 1.1 と 1.2 のどちらで読んでも型が変わらない書き方にしなければならない
`req~write-back.quoting-type-safe~2` P0
- Rationale: 書いた値が別の型に読まれたり、フロントマターが壊れたりするのを防ぐため。クオートを付けても困ることは少ない
- Acceptance:
  - Given:
  - When: Text 型の列に `no` を入れる
  - Then: `"no"` と書かれ、読み直すと文字列の no。

#### ノートを書くときは、同じフォルダの一時ファイルに書いて fsync してから名前を変えて置き換えなければならない
`req~write-back.atomic-replace-temp~2` P0
- Rationale: 同期の道具や Obsidian など、ほかの道具が書きかけのノートを読まないようにするため
- Acceptance:
  - Given:
  - When:
  - Then: 書き込みの途中で落としても元のファイルが壊れない。

#### シンボリックリンクのノートは、辿った先のファイルを書かなければならない
`req~write-back.atomic-replace-symlink~2` P0
- Rationale: 一時ファイルからの置き換えで、リンクをふつうのファイルに変えて、人のノートの置き方を壊さないため
- Acceptance:
  - Given:
  - When: シンボリックリンクのノートの status を書く
  - Then: リンクはリンクのまま残り、辿った先のファイルの status が変わる。

#### ノートを書くときは、ファイルの権限を保たなければならない
`req~write-back.atomic-replace-mode~2` P0
- Rationale: 一時ファイルからの置き換えで、人が決めたノートの権限を変えないため
- Acceptance:
  - Given:
  - When: 権限が 600 のノートの status を書く
  - Then: 書いたあとも 600。

#### 日付・日時の列(列の型が date か datetime)に書く日付は、元の値が囲んでいなければ(キーが無い・空のときも)、Obsidian と同じく囲まずに `YYYY-MM-DD`(日時は元の形の `YYYY-MM-DDTHH:MM` など)で書くべき
`req~write-back.date-quoting-plain~2` P1
- Rationale: 書くと型が変わるため
- Acceptance:
  - Given:
  - When: `due: 2026-10-05` の date の列を 2026-11-03 にして保存
  - Then: `due: 2026-11-03`。
- Acceptance:
  - Given:
  - When: キーの無いノートに入れる
  - Then: `due: 2026-11-03` の1行。

#### 日付・日時の列に書く日付は、元の値が引用符で囲んであれば、元の引用符を保つべき
`req~write-back.date-quoting-keep-quotes~1` P1
- Rationale: 書くと型が変わるため
- Acceptance:
  - Given:
  - When: date の列の `due: "2026-10-05"` を 2026-11-03 にして保存
  - Then: `due: "2026-11-03"`。

#### 列の型が決まらない列(型の設定(cell-edit.column-type)が無く、読み込んだどのノートにもその列の空でない値が無い列。新しいノートで聞く列(cell-edit.new-note-form)を含む)に、日付・日時の形だけの値を書くときも、囲まずに書くべき
`req~write-back.date-quoting-untyped~2` P1
- Rationale: Obsidian が日付と推すため
- Acceptance:
  - Given:
  - When: どのノートにも無い列 deadline を新しいノートで聞いて 2026-10-05 と答える
  - Then: `deadline: 2026-10-05`。
- Acceptance:
  - Given:
  - When: 同じ列に `2026-10-05 ごろ` と答える
  - Then: 囲む。

#### テキストの列(型の設定か、空でない値から text と決まった列)に日付に見える文字を書くときは、これまでどおり write-back.quoting で囲むべき
`req~write-back.date-quoting-text-column~1` P1
- Rationale: 書くと型が変わるため
- Acceptance:
  - Given:
  - When:
  - Then: テキストの列 memo に `2026-11-03` → `memo: "2026-11-03"`。

#### 書き戻しの安全を、英語(`docs/safety.md`)と日本語(`docs/safety.ja.md`)の文書に書くべき
`req~write-back.safety-docs-write~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `docs/safety.md` と `docs/safety.ja.md` のどちらにも、1バイトも変えない範囲(write-back.byte-range・write-back.frontmatter-only)、読むだけにするノートの形と理由(write-back.add-key-and-frontmatter・write-back.refuse-unsafe-notes。形ごとに例のノートを添える)、外の変更とのぶつかり(write-back.conflict-stop・write-back.new-baseline・write-back.external-change)、書き込みの手順(一時ファイル・読み直し・fsync・名前の変更・権限とシンボリックリンク。write-back.verify-before-replace・write-back.atomic-replace)の4つが書いてある(読んで判定)。

#### 文書の読むだけの形の例は試験で読み取りに通し、読むだけにならなければ試験が落ちるべき
`req~write-back.safety-docs-example-test~1` P1
- Rationale: 文書が古くなるのを防ぐため
- Acceptance:
  - Given:
  - When: 文書の BOM の例から BOM を消す
  - Then: 落ちる。

#### 読み取りが返す読むだけの理由の種類が文書に無ければ、試験が落ちるべき
`req~write-back.safety-docs-reason-test~1` P1
- Rationale: 文書が古くなるのを防ぐため
- Acceptance:
  - Given:
  - When: 読み取りに読むだけの理由を1つ足して文書を直さない
  - Then: 試験が落ちる。

#### 1行目が `+++` のノート(Hugo・Zola の TOML のフロントマター)には書き込まず、その行を読むだけにして、YAML でないフロントマターだと理由を表示し、フロントマターを足す設定(cell-edit.list-items)があっても(足すと生成器が読めないページになるため)足さないようにするべき
`req~write-back.toml-frontmatter~2` P0
- Rationale: 書くと壊す形のノートを壊さないため。理由を表示するのは、直せないのが故障に見えず、利用者が不安にならないようにするため
- Acceptance:
  - Given:
  - When: `+++\ntitle = "x"\n+++\n` のノートの列で Enter
  - Then: 編集できず理由。フロントマターを足す設定でも足さない。

### 保存のあとの git

#### 既定(`[git] auto = "off"`)では、保存しても git を走らせてはならない
`req~write-back.git-off-default~1` P1
- Rationale: 頼んでいないコミットや push を、道具が勝手にしないようにするため
- Acceptance:
  - Given:
  - When: 設定の無いまま、git のリポの中のノートを直して保存
  - Then: コミットは増えず、インデックスも前と同じで、メッセージに git の知らせが無い。

#### `[git] auto = "add"` のときは、保存で書いたノートだけをステージするべき
`req~write-back.git-add~1` P1
- Acceptance:
  - Given:
  - When: a.md を直して保存、other.txt は外で変更中
  - Then: ステージされるのは a.md だけで、other.txt は変更中のまま、コミットは増えない。

#### `[git] auto = "commit"` のときは、書いたノートだけを1つのコミットにし、ほかの変更中のファイルと先にステージしてあったファイルを入れないべき
`req~write-back.git-commit~2` P1
- Rationale: 作業の途中のほかのファイルを、mdgrid の保存に巻き込まないようにするため
- Acceptance:
  - Given:
  - When: a.md を直して保存、other.txt は変更中、b.md は先にステージ済み
  - Then: コミットの中身は a.md だけで、other.txt は変更中のまま、b.md はステージ済みのまま。
- Acceptance:
  - Given: 書いたノートが HEAD と同じ中身
  - When:
  - Then: コミットを作らない。

#### `[git] auto = "push"` のときは、コミットのあとに push するべき
`req~write-back.git-push~1` P1
- Acceptance:
  - Given:
  - When: push 先のあるリポで a.md を直して保存
  - Then: リモートのブランチが手元の HEAD と同じになる。

#### `[git] when = "save"`(既定)は保存と画面からのノートの作成のたびに、`"quit"` は終了のときに1回、git に渡すべきで、終了のときに残っているノートはどちらでも渡すべき
`req~write-back.git-when~2` P1
- Acceptance:
  - Given:
  - When: `when = "save"` で保存を2回
  - Then: コミットが2つ。
- Acceptance:
  - Given:
  - When: 画面から新しいノート c を作る
  - Then: c.md だけのコミット。
- Acceptance:
  - Given:
  - When: `when = "quit"` で a.md と b.md を別々に保存
  - Then: そのあいだコミットは増えず、終了のときに a.md と b.md の1つのコミット。

#### `[git] message = "auto"`(既定)のコミットの文は、1行目にノートの数を言う文、空行、リポの根からのパスを1行ずつにするべき
`req~write-back.git-message-auto~2` P1
- Acceptance:
  - Given: a.md と Tasks/x.md の2件
  - When:
  - Then: 1行目「mdgrid: 2 件のノートを更新」(英語は `mdgrid: update 2 notes`)、空行、`a.md`、`Tasks/x.md`。
- Acceptance:
  - Given: 1件
  - When:
  - Then: 「mdgrid: 1 件のノートを更新」。

#### `[git] message = "ask"` のときは、保存のあとに1行の欄を出し、打った文をコミットの文の1行目にするべき
`req~write-back.git-message-ask~2` P1
- Acceptance:
  - Given:
  - When: 保存
  - Then: 欄が開き、打つまでコミットしない。
- Acceptance:
  - Given:
  - When: 「見直し」と打って Enter
  - Then: 1行目が「見直し」のコミット。
- Acceptance:
  - Given:
  - When: 空のまま Enter
  - Then: 自動の文(write-back.git-message-auto)。
- Acceptance:
  - Given:
  - When: Esc
  - Then: コミットせず、終了のときに自動の文でコミットする。

#### git のリポでないフォルダのノートでは、`[git]` の設定によらず何もせず、知らせも欄も出さないべき
`req~write-back.git-not-repo~1` P1
- Acceptance:
  - Given: 上のどこにも `.git` の無いフォルダで `auto = "push"`・`message = "ask"`
  - When:
  - Then: 保存は済み、欄は開かず、メッセージに git の知らせが無い。

#### git が失敗したときは、保存を済んだままにし、理由を1行で知らせるべき
`req~write-back.git-failure~1` P1
- Rationale: git の都合で、直した値を失わせないため
- Acceptance:
  - Given: push 先の無いリポで `auto = "push"`
  - When:
  - Then: ノートは書かれて未保存 0、メッセージに「git が失敗した(保存は済んでいる)」と `git push` の理由が1行。次の保存か終了のときに、もう一度 git に渡す。

#### `[git]` は設定ファイル(config.toml)にだけ書け、台帳の印に書いたものは警告して使わないべき
`req~write-back.git-global-only~2` P1
- Rationale: 写してきた台帳の印が、開いた人のリポで勝手にコミットや push をさせないようにするため
- Acceptance:
  - Given: config.toml の `[git] auto = "push"`
  - When:
  - Then: 効く。
- Acceptance:
  - Given: `auto = "sometimes"`
  - When:
  - Then: `git.auto` を言う警告1行で、既定(off)のまま。
- Acceptance:
  - Given: 印 `.mdgrid/workspace.toml` の `[git] auto = "push"`
  - When:
  - Then: 警告1行で使わない。

## 未確定事項

### 画面を出さないコマンド(`--new-note`・`--apply --yes`)で書いたノートも、`[git]` の設定で git に渡すか
`tbd~write-back.git-cli`
- Decider: 人
- Provisional: 渡さない(画面からの保存と作成だけ。スクリプトは自分で git を呼べる)

### 画面で名前を変えたノート(cell-edit のノートの改名)を、git に渡すか
`tbd~write-back.git-rename`
- Decider: 人
- Provisional: 渡さない(保存で書いたノートと、画面から作ったノートだけ)

### 「保存のたびにコミットし、push は終了のときだけ」の組み合わせを設定で選べるようにするか
`tbd~write-back.git-push-at-quit`
- Decider: 人
- Provisional: 今は無い(`when` は add・commit・push の全部に同じに効く)
