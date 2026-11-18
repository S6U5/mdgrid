# 台帳のスターター(starter) の要求

## 目的

タスク・ブックマーク・読書の記録・人と会議のように、よくある台帳の「見せ方」(ワークスペースの印 `.mdgrid/workspace.toml`・新しいノートの本文の雛形・見本のノート)をひとまとまりにして持ち、そこから新しい台帳を作る仕組み。組み込みのスターターは実行ファイルの中にあり、自分のスターターは設定の置き場の `starters/` に同じ形で置く。CLI(`--init-workspace --starter <名前>`)と画面のコマンドのパレットの両方から作れる。スターターは見せ方だけを持ち、ノートの形の決まり(スキーマ・lint)は別の道具の役目で、ここでは扱わない。版1 はスターターから作るだけで、今の表からスターターを書き出すのは次の段。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/starter/spec.md](../_history/decidespec/starter/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 次の段: 今の表(印・ビュー)からスターターを書き出すこと(逆の向き)。版1 はスターターから作るだけ。
- 採らない: スキーマ・lint・ノートの形の検査(ノートの形を検査する道具の役目。starter.shape)。
- 採らない: ネットからスターターを取ってくること(置き場は組み込みと設定のフォルダだけ。starter.user)。

## 前提(旧い仕様で AI が置いた仮定)

- 組み込みのスターターの印には `name` を書かない(作った先のフォルダの名前がワークスペースの名前になる。workspace.marker)。
- 言語の変種(`ledger.ja/` など)は丸ごとの差し替えで、`ledger/` に重ねない(見本のノートの名前が英語と日本語で混ざらないため)。英語(接尾なし)が正本。
- 呼び名: 「型」は列の型・画面の型に、「テンプレート」は cli.config-file の `[templates]` と cell-edit.template-vars の雛形に使っているので、この機能は「スターター」(`starter`)と呼ぶ。

## 要求

### スターターの形と組み込み

#### スターターは、印と新しいノートの本文の雛形を `ledger/` に、見本のノートを `samples/` に、台帳の根からの相対パスのまま置いた、台帳の見せ方のひとまとまりとするべき
`req~starter.shape-layout~1` P1
- 受け入れ: `starters/tasks/` に `starter.toml`・`ledger/.mdgrid/workspace.toml`(印。workspace.marker・workspace.marker-base・workspace.marker-where・tree.config の中身)・`ledger/.mdgrid/templates/task.md`(本文の雛形。cell-edit.template-vars の `body` のファイル)・`samples/tasks/Write the plan.md` → スターターとして読める。

#### スターターは見せ方だけを持ち、ノートの形の決まり(スキーマ・lint)は持たないべき
`req~starter.shape-no-schema~1` P1
- なぜ: ノートの形を検査する道具の役目で、mdgrid が台帳ごとの知識を持たないため
- 受け入れ: スターターの形(`starter.toml`・`ledger/`・`samples/`)に、スキーマ・lint の項目が無い(読んで判定)。

#### スターターの名前と1行の説明は、言語ごとに `starter.toml` に持ち、無ければフォルダの名前と空の説明にするべき
`req~starter.shape-name~1` P1
- 受け入れ: `starters/tasks/` の `starter.toml`(`[name]`・`[description]` に、文言の台帳と同じ言語の符号の鍵)→ 名前は ja で「タスク」、en で「Tasks」。`starter.toml` の無いユーザーのスターター → 名前はフォルダの名前、説明は空。

#### スターターに `ledger.<言語>/`・`samples.<言語>/` があれば、その言語のときは `ledger/`・`samples/` に重ねずに、それを丸ごと代わりに使うべき
`req~starter.shape-lang-variant~1` P1
- 受け入れ: `ledger.ja/` があり言語が ja → `ledger/` は使わず `ledger.ja/` の中身だけ。

#### mdgrid は組み込みのスターター `tasks`・`bookmarks`・`reading`・`people` を実行ファイルの中に持つべき
`req~starter.builtin-list~1` P1
- 受け入れ: 設定の置き場が無い → 組み込みのスターターは `tasks`(タスク)・`bookmarks`(ブックマーク)・`reading`(読書・視聴の記録)・`people`(人・連絡先・会議メモ)の4つで、どれからも `--init-workspace --starter` で作れる。

#### 組み込みのスターターは、ユーザーのスターター(starter.user)と同じ形・同じ読み方の見本とし、組み込みだけに効く特例を作らないべき
`req~starter.builtin-same-shape~1` P1
- 受け入れ: 組み込みの `reading` と同じ中身を設定の置き場の `starters/reading2/` に置く → `--init-workspace --starter reading2` で、`--starter reading` と同じパスと中身の印と雛形ができる。

#### 組み込みのスターターは、どの言語の変種でも、印が警告なしに読めるべき(印の読み込み・木の設定・`open` の警告が空)
`req~starter.builtin-no-warning~1` P1
- 受け入れ: 4つの組み込みから見本つきで作る → `--print-config --resolved` の標準エラーに `mdgrid:` の行が無い。`LANG=en_US.UTF-8` でも同じ。

#### 組み込みのスターターは、どの言語の変種でも、印の `open` が印の表を指すべき
`req~starter.builtin-open~1` P1
- 受け入れ: 4つの組み込みから見本つきで作る → `--print` が見本の行を出す。`LANG=en_US.UTF-8` でも同じ。

#### 組み込みのスターターの見本のノートは、どの言語の変種でも `--check` で問題 0 になるべき
`req~starter.builtin-samples-check~1` P1
- 受け入れ: 4つの組み込みから見本つきで作る → `--check` の問題が 0。`LANG=en_US.UTF-8` でも同じ。

#### 組み込みのスターター `tasks` から作った台帳は、新しいノートの名前を今日の日付8桁と `_` で始め、ノートの欄にはその頭の部分を出さないべき
`req~starter.tasks-name-prefix~1` P1
- なぜ: 同じ題のタスクを別の日にも作れ、ファイルが作った順に並び、表では題だけを読めるようにするため
- 受け入れ: 今日が 2026-10-06 で、`tasks` から作った台帳の表で新しいノートを作り始める → 名前の欄に `20261006_` が入って出る。その台帳の `Tasks/20261006_請求書を送る.md` → ノートの欄は `請求書を送る`。頭に8桁と `_` の無い `Tasks/メモ.md` → ノートの欄は `メモ`。`LANG=en_US.UTF-8` で作った台帳でも同じ。

#### ユーザーのスターターは、設定の置き場(cli.config-file)の `starters/<名前>/` に、組み込みと同じ形で置けるべき
`req~starter.user-place~1` P1
- 受け入れ: 設定の置き場の `starters/kanban/` に組み込みと同じ形のスターターを置く → `--starters` に kanban が出る。

#### スターターの一覧は、ユーザーのスターターを名前の順に先に、組み込みをその後に並べるべき
`req~starter.user-order~1` P1
- 受け入れ: `starters/kanban/` を置く → `--starters` の先頭に kanban、その後に組み込み。

#### ユーザーのスターターと組み込みに同じ名前があれば、一覧はユーザーのものを使い、組み込みを出さないべき
`req~starter.user-shadow~1` P1
- 受け入れ: `starters/tasks/` を置く → tasks は1つで、ユーザーのもの。

#### `mdgrid --starters` は、スターターの名前と今の言語の1行の説明を1行ずつ標準出力に出して終了コード 0 で終わり、設定の置き場が無くても組み込みを出すべき
`req~starter.user-list~1` P1
- 受け入れ: 置き場なし → `--starters` が組み込みの4つを、名前と説明の1行ずつ標準出力に出し、終了コード 0。

#### 読めないスターター(壊れた `starter.toml`・`ledger/.mdgrid/workspace.toml` の無いもの)は、cli.config-file の形で警告して飛ばすべき
`req~starter.user-skip-broken~1` P1
- 受け入れ: 壊れた `starter.toml` → `starters/kanban/starter.toml:` で始まる警告1行で、ほかは出る。

#### `mdgrid --starters` は画面を開かず、ファイルを書かないべき
`req~starter.user-no-side-effect~1` P1
- 受け入れ: `mdgrid --starters` → 画面を出さずに終わり、今のフォルダと設定の置き場のファイルは前と同じ。

### 作る(CLI と画面)

#### `mdgrid [<フォルダ>] --init-workspace --starter <名前>` は、そのフォルダにスターターの `ledger/` の中身を同じ相対のパスで書くべき
`req~starter.init-ledger~1` P1
- 受け入れ: `mdgrid books --init-workspace --starter reading` → `books/.mdgrid/workspace.toml` と `books/.mdgrid/templates/` の雛形ができる(フォルダを渡さなければ今のフォルダ。フォルダが無ければ作る)。

#### `--init-workspace --starter` に `--with-samples` を添えたときは、スターターの見本のノートも書くべき
`req~starter.init-samples~1` P1
- 受け入れ: `mdgrid books --init-workspace --starter reading --with-samples` → 見本のノートも書かれる。`--with-samples` なしでは見本は無い。

#### スターターから作るとき、見本のノートの中の `{date}`・`{date+N}`・`{now}` などは、cell-edit.template-vars と同じ変数として書くときに埋めるべき
`req~starter.init-sample-vars~1` P1
- 受け入れ: `--with-samples` と `MDGRID_TODAY=2026-10-06` → 見本の `{date-10}` が `2026-09-26` で書かれる。

#### スターターから作るとき、`ledger/` の中身(印と本文の雛形)はそのまま写すべき
`req~starter.init-ledger-verbatim~1` P1
- なぜ: 雛形の変数は新しいノートを作るときに埋まるため
- 受け入れ: `--with-samples` と `MDGRID_TODAY=2026-10-06` → 雛形の `{date}` は文字のまま。

#### `--init-workspace --starter` は、書く先に同じパスのファイルが1つでもあれば、どれも書かずに理由を出して終了コード 2 にするべき(workspace.marker と同じく上書きしない)
`req~starter.init-no-overwrite~1` P0
- 受け入れ: `mdgrid books --init-workspace --starter reading` をもう一度 → 理由1行と終了コード 2 で何も変わらない。

#### `--starter` に無い名前・2つ以上のフォルダ・`--init-workspace` の無い `--starter`・`--starter` の無い `--with-samples` は、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~starter.init-bad-args~1` P1
- 受け入れ: `--starter nope` → 理由1行(`--starters` で一覧を見られることを添える)と終了コード 2。`--init-workspace` の無い `--starter reading` → 理由1行と終了コード 2。

#### `--init-workspace --starter` は、作ったファイルを1行ずつ標準出力に出し、画面は出さないべき
`req~starter.init-report~1` P1
- 受け入れ: `mdgrid books --init-workspace --starter reading` → 標準出力に、作ったファイル(印と雛形)が1行に1つずつ出て、画面は出ない。

#### コマンドのパレット(screen.palette)の「スターターから台帳を作る」で、スターター・作る場所・見本の有無を答えると、starter.init と同じに作り、その台帳の印の `open` の表(無ければ根)を開くべき
`req~starter.palette-create~1` P1
- 受け入れ: `:` → 「スターター」→ 一覧(名前と説明。打った字で絞る)に組み込みの4つ → reading を Enter → 「作る場所」の欄に `reading`(既定は開いたフォルダの下のスターターの名前)が入っているのを `~/notes/books`(開いたフォルダからの相対・`~`・絶対のパスを打てる)に直して Enter → 「見本のノートも入れる?(y/n)」→ `y` → books の Reading の表が開き、パンくずに books。

#### パレットの「スターターから台帳を作る」は、ためた変更があれば、cli.places-list と同じく保存か取りやめを確かめるべき
`req~starter.palette-unsaved~1` P1
- 受け入れ: 表で未保存の変更があるときに「スターターから台帳を作る」→ 保存の確かめ。

#### パレットの「スターターから台帳を作る」の Esc は、どの段でも取りやめにし、何も作らないべき
`req~starter.palette-esc~1` P1
- 受け入れ: 一覧・作る場所・見本の問いのどの段でも、途中の Esc → 「やめた」で何も作らない。

#### 読むだけの起動(cli.readonly)では、パレットに「スターターから台帳を作る」を出さないべき
`req~starter.palette-readonly~1` P1
- 受け入れ: `--readonly` で起動 → パレットに「スターターから台帳を作る」が無い。

#### スターターから作った台帳は、ふだんの印のあるフォルダと同じに扱い、開いて見るだけでは印・雛形・見本のバイトを変えないべき
`req~starter.same-as-manual-bytes~1` P1
- なぜ: スターターから作った台帳と手で書いた台帳の間に差を作らないため
- 受け入れ: 作った直後と、開いて閉じたあとの印のバイトが同じ(雛形と見本のバイトも同じ)。

#### スターターの中身は、文書にある印の項目(cli.config-docs)と雛形の変数(cell-edit.template-vars)の範囲に収め、スターターだけに効く項目を作らないべき
`req~starter.same-as-manual-keys~1` P1
- なぜ: スターターから作った台帳と手で書いた台帳の間に差を作らないため
- 受け入れ: 組み込みの印に知らない項目が無い(starter.builtin の警告なしで確かめる)。

## 未確定事項

### 組み込みの `tasks` の見本のノートの名前にも、日付8桁と `_` の頭を付けるか
`tbd~starter-sample-name-prefix`
- 決める人: 人
- 仮: 付けない(見本のノートは題だけの名前のままにし、ノートの欄にはそのまま出る)
- 根拠: 見本の `created` は作る日からの相対(`{date-2}`)で書くとき埋まるが、ファイルの名前は埋めない。名前に決まった日付を書くと `created` とずれ、名前も書くとき埋めるには `{date-2:YYYYMMDD}` のような形が要るが、`:` は Windows のファイルの名前に書けない。
