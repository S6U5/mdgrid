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

#### mdgrid は組み込みのスターター `tasks`・`tasks-ai`・`bookmarks`・`reading`・`people` を実行ファイルの中に持つべき
`req~starter.builtin-list~2` P1
- 受け入れ: 設定の置き場が無い → 組み込みのスターターは `tasks`(タスク)・`tasks-ai`(人と AI で分けるタスクとルーチン)・`bookmarks`(ブックマーク)・`reading`(読書・視聴の記録)・`people`(人・連絡先・会議メモ)の5つで、どれからも `--init-workspace --starter` で作れる。

#### 組み込みのスターターは、ユーザーのスターター(starter.user)と同じ形・同じ読み方の見本とし、組み込みだけに効く特例を作らないべき
`req~starter.builtin-same-shape~1` P1
- 受け入れ: 組み込みの `reading` と同じ中身を設定の置き場の `starters/reading2/` に置く → `--init-workspace --starter reading2` で、`--starter reading` と同じパスと中身の印と雛形ができる。

#### 組み込みのスターターは、どの言語の変種でも、印が警告なしに読めるべき(印の読み込み・木の設定・`open` の警告が空)
`req~starter.builtin-no-warning~2` P1
- 受け入れ: 5つの組み込みのどれからでも見本つきで作る → `--print-config --resolved` の標準エラーに `mdgrid:` の行が無い。`LANG=en_US.UTF-8` でも同じ。

#### 組み込みのスターターから見本なしで作った台帳は、どの言語の変種でも、行が無いことの知らせ・警告を出さずに開けるべき(Obsidian を使わないフォルダで `.obsidian/` の話をしない)
`req~starter.builtin-empty-quiet~1` P1
- なぜ: 印のある台帳の根は印のフォルダで、`.obsidian/` から推した根ではないため
- 受け入れ: 5つの組み込みのどれからでも見本なしで作る(上のどこにも `.obsidian/` が無い)→ `--print` の標準エラーが空で、終了コード 0。`LANG=en_US.UTF-8` でも同じ。印の無い場所の `.base` が 0 行のときは、今までどおり探した根を標準エラーに出す(base-view.root-folder-nearest)。

#### 組み込みのスターターは、どの言語の変種でも、印の `open` が印の表を指すべき
`req~starter.builtin-open~2` P1
- 受け入れ: 5つの組み込みのどれからでも見本つきで作る → `--print` が見本の行を出す。`LANG=en_US.UTF-8` でも同じ。

#### 組み込みのスターターの見本のノートは、どの言語の変種でも `--check` で問題 0 になるべき
`req~starter.builtin-samples-check~2` P1
- 受け入れ: 5つの組み込みのどれからでも見本つきで作る → `--check` の問題が 0。`LANG=en_US.UTF-8` でも同じ。

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
`req~starter.user-list~2` P1
- 受け入れ: 置き場なし → `--starters` が組み込みの5つを、名前と説明の1行ずつ標準出力に出し、終了コード 0。

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
`req~starter.palette-create~2` P1
- 受け入れ: `:` → 「スターター」→ 一覧(名前と説明。打った字で絞る)に組み込みの5つ → reading を Enter → 「作る場所」の欄に `reading`(既定は開いたフォルダの下のスターターの名前)が入っているのを `~/notes/books`(開いたフォルダからの相対・`~`・絶対のパスを打てる)に直して Enter → 「見本のノートも入れる?(y/n)」→ `y` → books の Reading の表が開き、パンくずに books。

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

### スターターを元に重ねる

台帳の印に「元にするスターターとオプション」だけを書き、実行ファイルの中のスターター(と設定の置き場の自分のスターター)をその下に毎回重ねる(2026-11-18 に人が決めた向き)。印に自分で書いた項目が常に勝つ。読む側(重ねて開く)は作った(P1)。作る側は今も丸ごと写す(starter.init-ledger)ままで、重ねる形の印は手で書く。作る側・選ぶ画面・書き出し・知らせは次の段(P2)で、そのときに、写す形を前提にした starter.init-ledger・starter.init-ledger-verbatim・starter.same-as-manual-keys・starter.builtin-same-shape の版を上げて書き直す。

#### 台帳の印に `starter = "<名前>"` と `options = ["<名前>", …]` を書くと、そのスターターとオプションの中身(印の項目・表・ビュー)を元にして開くべき
`req~starter.layer-marker~1` P1
- なぜ: スターターが新しくなったときに、台帳を書き換えずに取り込めるようにするため
- 受け入れ: 印が `starter = "tasks"` と `options = ["ai"]` の2行だけの台帳 → tasks にオプション ai を付けたときの表・ビュー・値の候補で開く。`starter` の無い印 → 今までどおり印に書いた中身だけ。

#### スターターから来た項目と印に自分で書いた項目が同じとき、印の項目が勝つべき
`req~starter.layer-own-wins~1` P1
- なぜ: 自分の直しが、スターターの更新で消えないようにするため
- 受け入れ: スターター tasks の `[rules.priority] values = ["p1", "p2", "p3"]`、印に `[rules.priority] values = ["high", "medium", "low"]` → 優先度の候補は high・medium・low。印に書いていない `[rules.status]` はスターターの値のまま。

#### スターターの表とビューは名前で印の表とビューに突き合わせ、同じ名前は項目ごとに印が勝ち、印にだけある表とビューは後ろに足すべき
`req~starter.layer-tables~1` P1
- 受け入れ: 印に `[[table]] name = "タスク"` と `kanban = { column = "phase" }` だけ → 表「タスク」のカンバンの列は phase、ビューはスターターのもの。印に新しい名前のビュー「今週」→ スターターのビューの後ろに「今週」のタブ。

#### mdgrid を新しい版にすると、印を1バイトも書き換えずに、新しいスターターの中身が効くべき
`req~starter.layer-update~1` P1
- 受け入れ: 印が `starter = "tasks"` と自分の上書きだけの台帳を、スターター tasks にビューが1つ増えた版で開く → 増えたビューのタブが出て、印のバイトは前と同じ。

#### 元にしたスターターの中身が前に開いたときから変わっていたら、開いたときに1回だけ、変わったことを1行で知らせるべき
`req~starter.layer-notice~1` P2
- なぜ: 版を上げたときに、黙って見え方が変わらないようにするため
- 受け入れ: スターターにビューが1つ増えた版で初めて開く → 「スターター tasks が新しくなった」の1行。閉じてもう一度開く → 知らせは出ない(読んで判定)。

#### `--print-config --resolved` は、スターターから来た項目の出どころに、スターターとオプションの名前を出すべき
`req~starter.layer-origin~1` P2
- 受け入れ: 印が `starter = "tasks"`・`options = ["ai"]` の台帳 → オプション ai から来た `[rules.actor]` の出どころのコメントに `tasks` と `ai`、印に書いた項目の出どころは印(読んで判定)。

#### スターターは、付け外しできる部品(オプション)を `options/<名前>.toml`(印に重ねる断片)に持て、`options/<名前>.<言語>.toml` があればその言語ではそれを使うべき
`req~starter.option-shape~1` P1
- 受け入れ: `starters/kb/options/who.toml`(`[rules.actor]` と、表 Board のビュー All に actor の列を足し、ビュー AI を足す断片)→ 印に `options = ["who"]` と書いた台帳で、All に actor の列が出て、ビュー AI が使える。`options` に書かなければ、どちらも無い。

#### 印の `starter_lang = "<言語>"` で、元にするスターターの言語の変種を決められ、無ければ今の言語の変種を使うべき
`req~starter.layer-lang~1` P1
- なぜ: 表とビューの名前が言語で変わると、印の `open` や自分の上書きの名前が当たらなくなるため
- 受け入れ: `ledger.ja/` のあるスターターで、`LANG` が ja・`starter_lang` なし → ja の変種。`starter_lang = "en"` → `LANG` が ja でも接尾の無い `ledger/`。

#### 新しいノートの本文の雛形は、台帳に同じパスのファイルがあればそれを、無ければ元にするスターターの `ledger/` の同じパスのファイルを使うべき
`req~starter.layer-files~1` P1
- 受け入れ: スターターの `[new_note] body = ".mdgrid/templates/card.md"`(中身 `# {name}`)、台帳にそのファイルが無い → 新しいノート x の本文は `# x`。台帳に同じパスで `mine {name}` を置く → 新しいノート y の本文は `mine y`。

#### 印の `options` に、そのスターターに無い名前があるときは、cli.config-file の形で警告して、そのオプションだけを飛ばすべき
`req~starter.option-unknown~1` P1
- 受け入れ: `starter = "tasks"`・`options = ["ai", "nope"]` → `options` の nope を言う警告1行で、ai は効く。

#### パレットの「スターターから台帳を作る」は、スターターを選んだあとに、そのスターターのオプションを名前と説明の一覧から印を付けて選べるべき
`req~starter.pick-options~1` P2
- 受け入れ: tasks を選ぶ → オプションの一覧(ai・routines など。名前と1行の説明)→ Space で ai に印 → Enter → 作る場所 → 作った印に `starter = "tasks"` と `options = ["ai"]`。オプションの無いスターターでは、この段を出さない。どの段の Esc も取りやめで何も作らない(読んで判定)。

#### `--init-workspace --starter <名前>` に `--option <名前>`(何度でも)を添えて、オプションを選べるべき
`req~starter.pick-options-cli~1` P2
- 受け入れ: `mdgrid t --init-workspace --starter tasks --option ai --option routines` → 印に `options = ["ai", "routines"]`。無い名前の `--option nope` → 理由1行と終了コード 2 で何も作らない(読んで判定)。

#### パレットの「この台帳のオプションを変える」で、元にしたスターターのオプションをあとから付け外しでき、印の `options` の行だけを、差分を見せてから書き換えるべき
`req~starter.pick-change~1` P2
- 受け入れ: `options = ["ai"]` の台帳で routines に印を付ける → 差分(`options` の1行)を見せ、確かめてから印に `options = ["ai", "routines"]`。印のほかの行のバイトは同じ。読むだけの起動(cli.readonly)ではパレットに出さない(読んで判定)。

#### パレットとコマンド(`--flatten-starter`)の「スターターの中身を印に書き出す」で、今効いている中身を全部印に書いて `starter` と `options` の行を外し、手で書いた台帳にできるべき
`req~starter.flatten~1` P2
- なぜ: スターターから離れて自分で持ちたい人と、スターターが無くなるときのため
- 受け入れ: `starter = "tasks"`・`options = ["ai"]` と自分の上書きのある台帳で書き出す → 差分を見せ、確かめてから書く。書いたあとの `--print` の出力と `--print-config --resolved` の値は書く前と同じで、印に `starter` と `options` の行は無い(読んで判定)。

#### 印の `starter` が指すスターターが無いときは、警告して、印に自分で書いた中身だけで開くべき
`req~starter.retire-missing~1` P1
- 受け入れ: `starter = "nope"` の台帳 → `starter` の nope を言う警告1行で、印に書いた表とビューだけで開く。ノートと印のバイトは変わらない。

## 未確定事項

### 組み込みのスターターをやめるとき、その台帳をどうやって手で書いた台帳に落とすか(中身は実行ファイルにしか無いので、無くなった版では書き出せない)
`tbd~starter.retire-builtin`
- 決める人: 人
- 仮: やめるスターターは、一覧(`--starters`・パレット)から外したあとも1つ先の版まで実行ファイルに残し、その台帳を開いたときに「印に書き出す」(starter.flatten)を勧める1行を出す

### 組み込みの tasks が持つオプションの並び(今の tasks-ai を「tasks + オプション」に畳むか、別のスターターのまま残すか)
`tbd~starter.option-builtin-list`
- 決める人: 人
- 仮: tasks に ai(担当で分ける)・routines(ルーチンの表とリンクの列)・screens(ガント・カンバン・グラフ)・priority-words(優先度を high・medium・low に。選ばなければ p1〜p3 のまま: 2026-11-18 に人が決めた)。tasks-ai は「tasks に ai と routines を付けたもの」の別名として残す

### スターターのビューや表を、印の側から出さないようにする書き方(上書きは足すことと置き換えることしかできない)
`tbd~starter.layer-hide`
- 決める人: 人
- 仮: 印の表・ビューに `hidden = true` を書くと、同じ名前のスターターの表・ビューを出さない

### `starter` の行を知らない旧い版の mdgrid で、重ねる形の台帳を開いたときの振る舞い
`tbd~starter.layer-old-version`
- 決める人: 人
- 仮: 旧い版は知らない項目として警告し、印に書いた中身だけで開く(旧い版と行き来する人は starter.flatten で書き出しておく)
