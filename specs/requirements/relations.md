# リレーション(relations) の要求

## 目的

ノートのフロントマターに書いたリンク(`[[名前]]`・`[文字](パス)`・ただのパス)を、表と表をつなぐリレーションとして扱う。行を見分けるキーはノートのファイル名(パス)で、id の列は使わない。多対多はリンクのリストで書き、中間の表は使わない。登録した表([cli](cli.md) の cli.places)をワークスペースの表とみなし、どの表がどの表を指すかはリンクの行き先から見つける。どのノートの集まりも表の形にそろうわけではないので、表にならないノートへのリンクも、リンクのまま扱う。最初は表の中のリレーション(見せる・入れる・たどる・つながった行)を作り、関係マップの画面と画面の切り替えのタブ、幅に合わせた段組みは次の段。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/relations/spec.md](../_history/decidespec/relations/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: id の列(P-01 など)で行き先を指す形。キーはファイル名(relations.link-forms)。
- 採らない: 多対多のための中間の表。リンクのリストで書く(relations.link-input)。

## 前提(旧い仕様で AI が置いた仮定)

- リンクの列かどうか(relations.link-input)は、その列の空でない値の過半数がリンクかで決める。
- relations.follow-link の「行き先を開く」の既定のキーは設計で決める(Enter は今のまま編集)。

## 要求

### 表の中のリレーション

#### フロントマターの値のうち、`[[…]]` の形、`[文字](相対のパス)` の形、実際にあるノートを指すただのパスを、行き先のノートへのリンク(リレーション)とみなすべき
`req~relations.link-forms-kinds~2` P1
- Rationale: 行を見分けるキーをノートのファイル名(パス)にし、id の列を要らなくするため
- Acceptance:
  - Given: 値が文字か文字のリストで、`project: "[[mdgrid]]"` で projects/mdgrid.md がある
  - When:
  - Then: リンク。
- Acceptance:
  - Given: `parent: ../projects/mdgrid`
  - When:
  - Then: projects/mdgrid.md へのリンク。`[[…]]` は `[[名前]]`・`[[パス|表示]]`・`[[名前#見出し]]` のどれも、ただのパスは `projects/mdgrid.md` のように `.md` を書いても省いてもリンク。

#### リンクの行き先は、値を持つノートのフォルダからの相対、開いた根からのパス、ファイル名の順で解く
`req~relations.link-forms-resolve~2` P1
- Acceptance:
  - Given: tasks/a.md の `project: "[[mdgrid]]"` で tasks/mdgrid.md と projects/mdgrid.md がある
  - When:
  - Then: フォルダからの相対の tasks/mdgrid.md。
- Acceptance:
  - Given: `project: "[[projects/mdgrid]]"`
  - When:
  - Then: 開いた根からのパスで projects/mdgrid.md。
- Acceptance:
  - Given: ファイル名で解くときに projects/mdgrid.md と archive/old/mdgrid.md がある
  - When:
  - Then: 根に近い・パスの短い projects/mdgrid.md(base-view.links-eval と同じ)。

#### `.base` を開いたときもフォルダを開いたときも、リンクを同じに扱う
`req~relations.link-forms-same-open~1` P1
- Acceptance:
  - Given: 同じノートの `project: "[[mdgrid]]"`
  - When:
  - Then: フォルダを開いても `.base` を開いても projects/mdgrid.md へのリンク。

#### リンクの行き先は、開いたフォルダの外(登録した表のフォルダ)にあってもよい
`req~relations.link-forms-outside~1` P1
- Acceptance:
  - Given:
  - When: tasks/ のフォルダを開き、projects/ が登録した表のフォルダ
  - Then: tasks/a.md の `project: "[[mdgrid]]"` は、開いたフォルダの外の projects/mdgrid.md へのリンク。

#### どのノートも指さないただの文字は、リンクとみなしてはならない
`req~relations.link-forms-plain-text~1` P1
- Rationale: 普通の値を誤ってリンクにしないため
- Acceptance:
  - Given: `status: done` で done.md が無い
  - When:
  - Then: ただの文字。

#### リンクの値のセルは、行き先のノートの名前(拡張子なし。`[[…|表示]]` なら表示の文字)で見せ、リストのときは名前を並べて見せるべき
`req~relations.link-display~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `project: "[[projects/mdgrid|mdgrid]]"` → `mdgrid`。`tags: ["[[tui]]", "[[relations]]"]` → `[tui, relations]`。

#### `[[…]]` か `[…](…)` の形なのに行き先のノートが見つからないリンクは、書いた文字のまま、行き先が無いと分かる印(cell-view.not-color-only と同じく色に頼らない)を付けて見せるべき
`req~relations.broken-link~1` P1
- Acceptance:
  - Given: `project: "[[nothing]]"` で nothing.md が無い
  - When:
  - Then: 印付きの `nothing`。

#### リンクの列(値の多くがリンクの列)のセルを直すときは、候補に、その列のリンクの行き先があるフォルダのノートの名前を出すべき
`req~relations.link-input-candidates~1` P1
- Acceptance:
  - Given:
  - When: project の列の値がどれも `[[…]]` で projects/ を指す
  - Then: 候補は projects/ のノート。

#### リンクの列のセルで候補から選んだら、選んだノートへのリンクを書くべき
`req~relations.link-input-write~1` P1
- Acceptance:
  - Given:
  - When: project の列の値がどれも `[[…]]` で projects/ を指す
  - Then: 候補の mdgrid を選ぶと `"[[mdgrid]]"` と書く。

#### リンクの列に書く形は、その列で使われている形(`[[…]]`・`[…](…)`・パス)に合わせ、無ければ `[[名前]]` にする
`req~relations.link-input-form~1` P1
- Acceptance:
  - Given: project の列の値がどれも `[文字](相対のパス)` の形
  - When:
  - Then: 選んだノートを `[文字](相対のパス)` の形で書く。

#### リンクの列で同じ名前のノートが複数あるときは、パスで区別できる形で書く
`req~relations.link-input-same-name~1` P1
- Acceptance:
  - Given: projects/mdgrid.md と archive/mdgrid.md がある
  - When:
  - Then: projects/mdgrid.md を選ぶと、名前だけでなくパスで区別できる形で書き、そのリンクの行き先は projects/mdgrid.md。

#### リストの列(多対多)では、複数のノートを選んで書けるべき
`req~relations.link-input-list~1` P1
- Acceptance:
  - Given:
  - When: 値が `[[…]]` のリストの tags の列で tui と relations を選ぶ
  - Then: `["[[tui]]", "[[relations]]"]` と書く。

#### リンクのセルから、行き先のノートを含む表(登録した表のうち行き先を含むもの、無ければ行き先のフォルダ)を開いて、その行を選べるべき(操作の一覧とパレットから)
`req~relations.follow-link-open~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: project のセルで「行き先を開く」→ projects の表が開き mdgrid の行が選ばれている。

#### リンクのセルから行き先へ移るとき、保存していない変更があれば、移る前に cli.places-list と同じに確かめる
`req~relations.follow-link-unsaved~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 保存していない変更がある表の project のセルで「行き先を開く」→ 移る前に、保存か取りやめの確かめ(cli.places-list と同じ)が出る。

#### リンクの行き先が無いときは、理由を出し、移ってはならない
`req~relations.follow-link-missing~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 無い行き先(`project: "[[nothing]]"` で nothing.md が無い)のセルで「行き先を開く」→ 理由を出す。表は移らない。

#### 選んだノートを指しているノート(つながった行・被リンク)を、どの表のどの列から指しているかと一緒に一覧で出すべき
`req~relations.backlinks-list~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: projects/mdgrid.md を選んで「つながった行」→ tasks の project の列から指す3行が出る。

#### つながった行の一覧から選んだノートを開けるべき
`req~relations.backlinks-open~1` P1
- Acceptance:
  - Given:
  - When: projects/mdgrid.md の「つながった行」で tasks の1行を選ぶ
  - Then: その行を開く。

#### つながった行を探す範囲は、開いたフォルダと、ワークスペース(workspace.scope-order の順で決めたもの。無ければ登録した表)の表にするべき
`req~relations.backlinks-scope~1` P1
- Acceptance:
  - Given: tasks のフォルダを開き、ワークスペースに meetings の表がある
  - When:
  - Then: projects/mdgrid.md の「つながった行」に、tasks の project の列と meetings の project の列から指す行が出る。

#### ワークスペースの表で、列の値の行き先が別の表のノートなら、設定を書かなくても、その列をその表へのつながり(リストの列なら多対多、そうでなければ多対一)とみなすべき
`req~relations.table-links-detect~2` P1
- Acceptance:
  - Given: ワークスペース(workspace.scope-order の順で決めたもの)が無く、tasks と projects を登録(cli.places)
  - When:
  - Then: tasks.project は projects への多対一。
- Acceptance:
  - Given: tags の列(リスト)
  - When:
  - Then: 多対多。

#### 行き先がどの表にも無いリンクや、列の形がそろっていないノートの集まりでも、リンクとして扱い、動きを止めてはならない
`req~relations.table-links-loose~1` P1
- Rationale: どのノートの集まりも表の形にそろうわけではないため
- Acceptance:
  - Given: 行き先がどの表にも無いリンク
  - When:
  - Then: リンクとして見せ、つながりには数えない。

### 関係マップ

#### 表どうしのつながり(relations.table-links)を図で見せる関係マップの画面を、表の画面とは別の画面の型(scope.screen-types)として持ち、表ごとの箱(名前・行の数・列)と、列から行き先の表への矢印(1対多・多対多)を描き、選んだつながりの詳細(元の列・行き先の表・種類・逆向きの列)と、つながった行を見せるべき
`req~relations.map~2` P1
- Acceptance:
  - Given:
  - When: 表の画面で「▦ 表 ▾」を押す
  - Then: 窓に「● 表」「関係マップ R」。
- Acceptance:
  - Given:
  - When: 「関係マップ」を選ぶ
  - Then: 関係マップの画面で、ボタンは「関係マップ ▾」。
- Acceptance:
  - Given:
  - When: Esc
  - Then: 窓が閉じ、画面は変わらない。
- Acceptance:
  - Given: つながる表が無い
  - When:
  - Then: 窓に関係マップが無い。

#### 画面の上の端の、画面の型のメニュー(今の型の名前と「▾」のボタン。例「▦ 表 ▾」)を押すと、画面の型(表・関係マップ)を並べた小さな窓をボタンのすぐ下に開くべき
`req~relations.screen-menu-open~2` P1
- Rationale: 型が増えてもヘッダーが伸びないようにするため
- Acceptance:
  - Given:
  - When: 表の画面で「▦ 表 ▾」を押す
  - Then: ボタンのすぐ下の窓に「表」と「関係マップ」。
- Acceptance:
  - Given: つながる表が無い
  - When:
  - Then: 窓に関係マップが無い(関係マップは、つながる表があるときだけ)。(読んで判定)

#### 画面の型の窓で選んだ型に切り替えられるべき
`req~relations.screen-menu-switch~1` P1
- Acceptance:
  - Given:
  - When: 窓で「関係マップ」を選ぶ
  - Then: 関係マップの画面になり、ボタンは「関係マップ ▾」。(読んで判定)

#### 画面の型の窓では、今の型に印を付けるべき
`req~relations.screen-menu-current~1` P1
- Acceptance:
  - Given:
  - When: 表の画面で窓を開く
  - Then: 「表」に今の型の印が付く(例「● 表」)。(読んで判定)

#### 画面の型の窓では、型に切り替えのキーがあれば添えるべき
`req~relations.screen-menu-key-hint~1` P1
- Acceptance:
  - Given:
  - When: 窓を開く
  - Then: 関係マップの行に切り替えのキー `R` が添えてある(例「関係マップ R」)。(読んで判定)

#### 画面の型の窓は、Esc と窓の外のクリックで閉じるべき
`req~relations.screen-menu-close~2` P1
- Acceptance:
  - Given:
  - When: 窓を開いて Esc
  - Then: 窓が閉じ、画面は変わらない。
- Acceptance:
  - Given:
  - When: 窓を開いて窓の外をクリック
  - Then: 窓が閉じる。(読んで判定)

#### 切り替えのキー(関係マップの `R`)は、画面の型の窓を開かずに効くべき
`req~relations.screen-menu-direct-key~1` P1
- Acceptance:
  - Given:
  - When: 表の画面で窓を開かずに `R`
  - Then: 関係マップの画面になる。(読んで判定)

#### 左の欄(ワークスペースの表の一覧)・真ん中・右の詳細・下のつながった行のような段組みは、端末の幅と高さに合わせて変えるべき(広ければ並べ、狭ければ1つずつにしてタブかキーで切り替える)
`req~relations.layout-adapt~1` P1
- Acceptance:
  - Given:
  - When: 幅 160・100・60 の端末で関係マップの画面を出す
  - Then: それぞれ並べる・2つ・1つになる。

#### 関係マップの段組みは、どの幅でも、文字が重なったり切れて読めなくなったりしてはならない
`req~relations.layout-readable~1` P1
- Acceptance:
  - Given:
  - When: 幅 160・100・60 の端末で関係マップの画面を出す
  - Then: どれも文字が重ならず、切れて読めなくならない。

#### 関係マップの画面で、表(左の一覧の行か盤の箱)をクリックしたら、その表を選ぶべき
`req~relations.map-mouse-select-table~2` P1
- Acceptance:
  - Given:
  - When: 盤の projects の箱をクリック
  - Then: projects を選ぶ。
- Acceptance:
  - Given:
  - When: 枠や空きのクリック
  - Then: 何も変わらない。

#### 関係マップの画面で、選んでいる表をもう一度クリックしたら、その表を開くべき
`req~relations.map-mouse-open-table~1` P1
- Acceptance:
  - Given:
  - When: 盤の projects の箱をクリックして選び、もう一度クリック
  - Then: projects の表へ移る。

#### 関係マップの画面で、盤の線・矢印・札か狭い画面のつながりの行をクリックしたら、そのつながりを選ぶべき
`req~relations.map-mouse-select-link~1` P1
- Acceptance:
  - Given:
  - When: tasks.project の矢印をクリック
  - Then: そのつながりの詳細。

#### 関係マップの画面で、つながった行をクリックしたら、そのノートを開くべき
`req~relations.map-mouse-open-note~1` P1
- Acceptance:
  - Given:
  - When: tasks.project のつながりを選び、下のつながった行の1行をクリック
  - Then: その行のノートを開く。

#### 関係マップの画面で、ホイールで表を上下に選ぶべき
`req~relations.map-mouse-wheel~1` P1
- Acceptance:
  - Given:
  - When: 表の一覧で tasks を選んでホイールを下へ1つ
  - Then: tasks の次の表を選ぶ。

#### 同じ表の中のノートを指す列(タスクの `parent` など)も、その表から自分への矢印(輪)のつながりとして関係マップに描き、詳細に元の列と種類(多対一・多対多)を出すべき
`req~relations.map-self-links~1` P1
- Rationale: 親子のように表の中でつながる形も、表どうしの図(ER 図)で見えるようにするため(2026-11-24 に人が「表どうしの図」を選んだ)。relations.table-links-detect は別の表へのつながりだけを数えていた
- Acceptance:
  - Given: tasks の parent が tasks のノートを指す
  - When: 関係マップを開く
  - Then: tasks の箱から tasks の箱へ戻る矢印があり、選ぶと詳細に「parent」と「多対一」。
- Acceptance:
  - Given: tasks の routine が routines のノートを、parent が tasks のノートを指す
  - When: 関係マップを開く
  - Then: つながりは2つ(tasks → routines と tasks → tasks)。
- Needs: utest

#### 関係マップの表の箱の列に、列の見出しと同じ型の印(数・日付・日時・チェック・リスト・札の列)と、リンクの列の印を添え、つながりの元の列を見分けられるようにするべき
`req~relations.map-column-types~1` P1
- Rationale: 表どうしの図で、列がどんな値を持つかと、どの列がほかの表を指すかを、箱を見るだけで分かるようにするため
- Acceptance:
  - Given: tasks の due が日付の列、parent と routine がリンクの列
  - When: 関係マップを開く
  - Then: tasks の箱の due に日付の印、parent と routine にリンクの印が付く。
- Acceptance:
  - Given: 色を使わない表示(`--no-color`)
  - When: 関係マップを開く
  - Then: 同じ印が字で出る(cell-view.not-color-only)。
- Needs: utest
### 次の段(まだ作っていない。P2)

#### mdgrid でノートの名前(ファイル名)を変えるときは、そのノートを指すリンク(開いたフォルダと登録した表の中)を、差分を見せてから新しい名前に書き直せるべき
`req~relations.rename-links~1` P2
- Acceptance:
  - Given:
  - When:
  - Then: 読んで判定。
