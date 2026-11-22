# ワークスペース(workspace) の要求

## 目的

ワークスペースは、名前の付いた表(フォルダか `.base`)の集まりで、リンクの行き先・つながった行・表どうしのつながり・関係マップ([relations](relations.md))の範囲になる。設定のフォルダの `workspaces.toml` に持ち、画面のパレットとコマンドの引数で管理する。フォルダに `.mdgrid/workspace.toml` の印があれば、そちらを先に使う(workspace.marker)。書いたワークスペースが無いときは、全体の設定で選んだ検知(.obsidian/ のあるルートフォルダ・git のリポ)で範囲を決め、それも無ければ登録した表([cli](cli.md) の cli.places。よく開く場所のブックマーク)を使う。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/workspace/spec.md](../_history/decidespec/workspace/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: 根の `mdgrid.toml` の形の印。印は `.mdgrid/workspace.toml` だけ(workspace.marker)。
- 採らない: 登録した表(cli.places)の分類をワークスペースにする形。登録はブックマークとして別に残す。

## 前提(旧い仕様で AI が置いた仮定)

- 表がいくつのワークスペースに入っていてもよい。`-w` が無ければ、`workspaces.toml` の順で最初のものを使う。

## 要求

### ワークスペース

#### ワークスペース(名前の付いた表の集まり)を、設定のフォルダの `workspaces.toml`(`[[workspace]]` の並び。`name` と、表の並び `table`(`name`・`path`(フォルダか `.base`)・`view`)と、ワークスペースと表それぞれの表のプロファイル(screen.profile-scopes。`[workspace.look]`・`[workspace.table.look]` など))に持つべき
`req~workspace.app-side-file~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `[[workspace]]` に name = "Product"、表を2つ → 2つの表のワークスペース。`[workspace.look] theme = "nord"` → そのワークスペースの表が nord。

#### ワークスペースの設定は、ノートのフォルダには何も書かない
`req~workspace.app-side-no-note-write~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid ~/notes/tasks --add-to Product`
  - Then: workspaces.toml だけが変わり、~/notes/tasks の中のファイルとフォルダは前後で同じ。

#### `workspaces.toml` の表の `path` の先頭の `~` は、ホームのフォルダを指す
`req~workspace.app-side-home~1` P1
- Acceptance:
  - Given: `path = "~/notes/tasks"` の表
  - When:
  - Then: ホームのフォルダの下の notes/tasks を表として開く。

#### `workspaces.toml` の読めない行は警告にして飛ばし、起動を止めてはならない
`req~workspace.app-side-bad-entry~1` P1
- Acceptance:
  - Given: path の無い表
  - When:
  - Then: 警告して飛ばし、起動は止まらない。

#### 画面のパレットから、ワークスペースを作る・今の表をワークスペースに足す(今のワークスペースから選ぶか新しく作る)・今の表をワークスペースから外す・ワークスペースを開く(一覧から選び、その表を開く)ができるべき
`req~workspace.palette~1` P1
- Acceptance:
  - Given:
  - When: 「この表をワークスペースに足す」→ 名前を選ぶ
  - Then: workspaces.toml にその表が入る。「ワークスペースを外す」→ 消える。

#### コマンドの引数 `mdgrid --workspaces` で、ワークスペースの一覧を出せるべき
`req~workspace.cli-list~1` P1
- Acceptance:
  - Given: Product に tasks と projects
  - When:
  - Then: `mdgrid --workspaces` に Product とその表 tasks・projects が出る。

#### コマンドの引数 `mdgrid <パス> --add-to <名前> [--as <表の名前>]` で、表をワークスペースに足せるべき(ワークスペースが無ければ作る)
`req~workspace.cli-add~1` P1
- Acceptance:
  - Given:
  - When: Product が無いときに `mdgrid ~/notes/tasks --add-to Product`
  - Then: Product ができ、`mdgrid --workspaces` に出る。

#### コマンドの引数 `mdgrid <パス> --remove-from <名前>` で、表をワークスペースから外せるべき
`req~workspace.cli-remove~1` P1
- Acceptance:
  - Given: Product に tasks がある
  - When: `mdgrid ~/notes/tasks --remove-from Product`
  - Then: `mdgrid --workspaces` の Product に tasks が出ない。

#### コマンドの引数 `mdgrid --remove-workspace <名前>` で、ワークスペースごと消せるべき
`req~workspace.cli-remove-workspace~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --remove-workspace Product`
  - Then: `mdgrid --workspaces` に Product が出ない。

#### コマンドの引数 `mdgrid -w <名前> [<パス>]` で、ワークスペースで表を開けるべき(パスが無ければ最初の表)
`req~workspace.cli-open~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `mdgrid -w Product` → 最初の表で開く。無い名前 → 理由1行と 0 以外の終了コード。

#### ワークスペースの操作は、開くパス(位置の引数)と重ならない旗で書くべき
`req~workspace.cli-flags~1` P1
- Rationale: 同じ名前のフォルダを開けなくしないため
- Acceptance:
  - Given:
  - When:
  - Then: `workspace` という名前のフォルダは `mdgrid workspace` で開ける。

#### ワークスペースの操作のうち画面を出さないものは、標準出力がパイプでも動くべき
`req~workspace.cli-pipe~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid --workspaces | cat`
  - Then: 一覧が出て、終了コードは 0。

#### 開いた表が属するワークスペース(`-w` で選んだもの、無ければ開いた表を含む最初のワークスペース)があれば、リンクの行き先を探す範囲・つながった行・表どうしのつながり・関係マップ・上の端のタブは、そのワークスペースの表(と今の表)だけを使うべき
`req~workspace.scope-use~1` P1
- Acceptance:
  - Given: ワークスペース Work に tasks と projects、Hobby に books
  - When:
  - Then: tasks で関係マップを開くと tasks と projects だけ。books を指すリンクは行き先の表に数えない。

#### 全体の設定 `[workspace]` の `detect` で、書いたワークスペースが無い表を開いたときに、開いたフォルダから上へたどって最初に見つかったルートフォルダ(`.obsidian/` のあるフォルダ)か git のリポの根(`.git` のあるフォルダ)を、名前の無いワークスペースとみなすべき(`detect` は `".obsidian"`・`".git"` の並びで、既定は `[".obsidian"]`。旧い値 `"vault"`・`"git"` も同じ意味で読み、移ったと警告する)
`req~workspace.detect-roots-find~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `.obsidian/` のあるルートフォルダの tasks を開く → ルートフォルダの直下の projects も関係マップに出る。`[workspace] detect = []` → 検知しない。

#### 検知した名前の無いワークスペースの表は、根の直下の、ノートのあるフォルダにするべき(`.base` は入れない。要れば書いて足す)
`req~workspace.detect-roots-tables~1` P1
- Rationale: `.base` の表はルートフォルダを指すので、自動で入れるとほかの表と重なるため
- Acceptance:
  - Given: ルートフォルダの直下に tasks・projects(ノートあり)・empty(ノートなし)・`tasks.base`
  - When:
  - Then: 表は tasks と projects だけで、empty と `tasks.base` は入らない。

#### 範囲の決め方は、`-w` で選んだワークスペース → 開いたフォルダから上へたどって最初に見つかった `.mdgrid/workspace.toml`(workspace.marker)→ 開いた表を含む `workspaces.toml` のワークスペース → 検知(workspace.detect-roots)→ 登録した表(cli.places)の順にするべき
`req~workspace.scope-order-rank~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 書いたワークスペースとルートフォルダの両方に入る表 → 書いたほう。どれにも入らない → 登録した表(今までどおり)。

#### 今どの範囲かを、ヘッダーのパンくず(screen.breadcrumb)に範囲の名前で示すべき(検知した範囲は、関係マップの画面でだけ示してよい)
`req~workspace.scope-order-crumb~1` P1
- Acceptance:
  - Given:
  - When: ワークスペース Work の tasks を開く
  - Then: ヘッダーにワークスペースの名前 Work。

#### フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき
`req~workspace.marker-root~1` P1
- Acceptance:
  - Given: notes/.mdgrid/workspace.toml がある
  - When:
  - Then: notes/tasks を開くと、notes を根とするワークスペースが範囲になる。

#### 印の形は、アプリの側の1つのワークスペースと同じにする(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対。ワークスペースの表のプロファイルは最上位に、表のものは `[table.look]` などに書く。screen.profile-scopes)
`req~workspace.marker-form~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: table を1つ書く → その表だけ。印に `[look] theme = "nord"` → その下の表が nord。

#### 印に表を書かなければ、根の直下の、ノートのあるフォルダを表にする(`.base` は workspace.detect-roots と同じ理由で入れない)
`req~workspace.marker-auto-tables~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: notes/.mdgrid/workspace.toml(表なし)→ notes の直下の tasks・projects がワークスペースの表。notes/tasks.base は入らない。

#### 印の最上位の `open` に印の表の名前(書いた表なら `name`、自動の表ならフォルダの名前)を書けば、根のフォルダを開いたときにその表を開くべき(cli.launch)
`req~workspace.marker-open~1` P1
- Acceptance:
  - Given: 表なしの印に `open = "projects"`
  - When:
  - Then: `mdgrid notes` が projects の表。

#### 印の `open` で開く表に `view` があれば、そのビューで開くべき
`req~workspace.marker-open-view~1` P1
- Acceptance:
  - Given: `open = "tasks"` と `[[table]] name = "tasks"`・`path = "tasks.base"`・`view = "人だけ"`
  - When:
  - Then: `mdgrid notes` が tasks.base の「人だけ」のビュー。

#### 印の `open` が印の表のどれも指さないときは、cli.config-file の形で警告して、`open` が無いときと同じに開くべき
`req~workspace.marker-open-unknown~1` P1
- Acceptance:
  - Given: `open = "無い"`
  - When:
  - Then: `.mdgrid/workspace.toml: open:` で始まる警告と、notes の既定の表。

#### `mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき
`req~workspace.marker-init~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `mdgrid notes --init-workspace` → notes/.mdgrid/workspace.toml ができる。もう一度 → 書かずに理由を出す。

#### 今の範囲(workspace.scope-order)の表の一覧を、画面の左から表の上に重ねて出す引き出しで見せるべき
`req~workspace.drawer-overlay~1` P1
- Acceptance:
  - Given: ワークスペース「仕事」に tasks・projects
  - When: tasks で開き `w`
  - Then: 左に重ねて tasks・projects が出る。

#### 引き出しで選んだ表へ移れるべき
`req~workspace.drawer-switch~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: projects を Enter → projects の表になる。「趣味」を開いて books をクリック → books の表で範囲は「趣味」。

#### 引き出しは、普段は出さないべき
`req~workspace.drawer-hidden~1` P1
- Rationale: 出しっぱなしにすると、左に自分の一覧を持つ端末の道具の中で動かしたとき左に帯が2本並び、表の幅も減るため
- Acceptance:
  - Given:
  - When:
  - Then: 普段の画面に引き出しの帯が無い。

#### 引き出しは、表の画面と関係マップのどちらでも、ヘッダーのパンくずの範囲か表の名前(screen.breadcrumb)・表の画面のキー `w`(割り当て直せる。screen.keys-vim-and-arrows)・パレットで開くべき
`req~workspace.drawer-open~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: tasks で開き `w` → 引き出しが開く。パンくずの範囲の名前をクリック → 引き出しが開く。

#### 引き出しは、表を選ぶ(クリックか Enter)・Esc・引き出しの外のクリック・開いたのと同じボタンかキーで閉じるべき
`req~workspace.drawer-close~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: projects を Enter → 引き出しは閉じる。開いて Esc → 何も変わらずに閉じる。

#### 引き出しに並べた今の範囲の表のうち、今の表に印を付けるべき
`req~workspace.drawer-current-mark~1` P1
- Acceptance:
  - Given:
  - When: tasks で開き `w`
  - Then: tasks(印付き)・projects。

#### 引き出しの今の範囲の表の下に、ほかのワークスペースを名前の見出しだけ畳んで並べるべき
`req~workspace.drawer-others~1` P1
- Acceptance:
  - Given: 「趣味」に books
  - When: tasks で開き `w`
  - Then: その下に畳んだ「趣味」があり、books は出ていない。

#### 引き出しのほかのワークスペースの見出しを開くと、そのワークスペースの表が並ぶべき
`req~workspace.drawer-expand~1` P1
- Acceptance:
  - Given:
  - When: 「趣味」を開く
  - Then: books が並ぶ。

#### 範囲がワークスペースでない(検知(workspace.detect-roots)か登録した表(cli.places))ときは、引き出しにその範囲の表を並べるべき
`req~workspace.drawer-non-ws~1` P1
- Acceptance:
  - Given: ワークスペースが無く登録した表だけ
  - When:
  - Then: 登録した表が並ぶ。

#### ためた変更があるときに引き出しで別の表へ移るなら、登録した表へ移るとき(cli.places-list)と同じく保存か取りやめを確かめるべき
`req~workspace.drawer-unsaved~1` P1
- Acceptance:
  - Given:
  - When: 未保存の変更があるときに projects を選ぶ
  - Then: 保存の確かめ。

#### 関係マップで引き出しを開いたら、表の画面に戻ってから開くべき
`req~workspace.drawer-from-map~1` P1
- Acceptance:
  - Given:
  - When: 関係マップでヘッダーのパンくずの範囲の名前を押す
  - Then: 表の画面に戻り、引き出しが開く。

#### 印の `[[table]]` には、`path` の `.base` のファイルの代わりに、`.base` と同じ中身(`filters`・`formulas`・`properties`・`summaries`・`views`。キーの名前と式は `.base` と同じ)を TOML の表 `base` に書いてよく、そのときその表は、印の根に置いた同じ中身の `.base` を開いたのと同じに振る舞うべき(ビューのタブ・絞り込み・計算の列・表示名・グループ分け・並べ替え・`--print`・`--check`・`--view`)
`req~workspace.marker-base-inline~1` P1
- Acceptance:
  - Given:
  - When: 根に `tasks.base` を置かず、印に `open = "台帳"`、`[[table]] name = "台帳"` と `[table.base]` に `filters = { or = ['file.inFolder("todo")'] }`・`[table.base.formulas] 担当 = 'if(actor == "ai", "AI", "人")'`・`[table.base.properties."note.title"] displayName = "題名"`・`[[table.base.views]]` を2つ(`name = "未完了"` で `status != "done"`、`name = "終わった"` で `status == "done"`、どちらも `groupBy = { property = "formula.担当", direction = "ASC" }`)→ 根で `mdgrid --print` が未完了のビューの行で、見出しに「題名」。`--view 終わった`
  - Then: done の行だけ。画面でタブに「未完了」「終わった」があり、担当でまとまりに分かれる。AGENTS.md の行は無い。

#### 印の表の `base` でノートを探す範囲は、印の根からにするべき(base-view.root-folder)
`req~workspace.marker-base-root~1` P1
- Acceptance:
  - Given: notes の親に `.obsidian/` があり、notes の隣の other にもノートがある
  - When:
  - Then: notes で `mdgrid --print` の行は notes の下のノートだけ。

#### 印に書いた `base` の中身は、mdgrid が書き換えないべき(base-view.base-untouched と同じ理由)
`req~workspace.marker-base-untouched~1` P1
- Acceptance:
  - Given:
  - When: 根で `--print`・`--view 終わった`・`--check` と画面で開く
  - Then: 開く前後で印のバイトが同じ。

#### `base` を書いた印の表には、`path` は要らない
`req~workspace.marker-base-no-path~1` P1
- Acceptance:
  - Given: `[[table]] name = "台帳"` と `[table.base]` だけで `path` を書かない
  - When:
  - Then: 警告せずに台帳の表を開く。

#### 印の表に `base` と `path` の両方を書いたときは、`base` を使って警告するべき(cli.config-file の形)
`req~workspace.marker-base-both~1` P1
- Acceptance:
  - Given:
  - When: `base` と `path = "x.base"` を両方書く
  - Then: `.mdgrid/workspace.toml: table[1].path:` で始まる警告で `base` を使う。

#### 印の表の `base` の中身が `.base` として読めないとき(base-view.source の読めない `.base`)は、`.base` のときと同じく理由を出して開かないべき
`req~workspace.marker-base-invalid~1` P1
- Acceptance:
  - Given: `[table.base] views = "x"` のような読めない中身
  - When:
  - Then: 理由1行と 0 以外の終了コード。

#### 印の表の `base`(workspace.marker-base)では、最上位とビューに `where`(列の名前を鍵に、値の配列(要素は文字列・数・真偽)を値にした表)を書いてよく、その列の値が配列のどれかと同じ行だけを残すべき(リストの列は、要素のどれかが当たれば残す)
`req~workspace.marker-where-match~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `[[table.base.views]]` に `name = "未完了"`・`where = { status = ["todo", "doing"] }` → todo と doing の行だけ。最上位の `where = { kind = ["todo"] }` → どのビューも kind が todo の行だけ。tags が `[a, b]` のノートに `where = { tags = ["b"] }` → 残る。

#### 印の表の `base` の `where` に鍵を2つ以上書いたときは、全部を満たす行だけにするべき
`req~workspace.marker-where-keys-and~1` P1
- Acceptance:
  - Given: `where = { status = ["done"], repo = ["tasks"] }`
  - When:
  - Then: done で repo が tasks の行だけ。

#### 印の表の `base` の同じ所に `filters` と `where` の両方を書いたときは、両方を満たす行だけにするべき
`req~workspace.marker-where-filters-and~1` P1
- Acceptance:
  - Given: ビューに `filters = 'repo == "tasks"'` と `where = { status = ["todo"] }`
  - When:
  - Then: todo で repo が tasks の行だけ。

#### 印の表の `base` の `where` の値が空の配列か配列でない値のときは、その鍵を cli.config-file の形で警告して使わないべき
`req~workspace.marker-where-invalid~1` P1
- Acceptance:
  - Given: `where = { status = [] }`
  - When:
  - Then: `.mdgrid/workspace.toml: table[1].base.views[1].where.status:` で始まる警告と、絞らない表。

### お気に入りのワークスペースと切り替え

#### ワークスペース(`workspaces.toml` のものと、印 workspace.marker の根)を、お気に入りに入れたり外したりできるべきで、入れるのは引き出しのワークスペースの見出しとパレットの「今のワークスペースをお気に入りにする」から行うべき
`req~workspace.favorite-mark~1` P1
- Rationale: よく行き来するワークスペースを、切り替えの一覧(workspace.switch-open)の先頭に置くため(2026-11-20 に人が挙げた)
- Acceptance:
  - Given:
  - When:
  - Then: 引き出しの「趣味」の見出しでお気に入りのキー → 見出しに ★ が付く。もう一度 → ★ が消える。印のワークスペース lab を開いてパレットの「今のワークスペースをお気に入りにする」→ 切り替えの一覧に lab。

#### お気に入りは、設定のフォルダの `workspaces.toml` の最上位の `favorites`(`workspaces.toml` のワークスペースは名前、印のワークスペースは根のパス)に入れた順で持ち、ノートのフォルダと印には書かないべき
`req~workspace.favorite-store~1` P1
- Rationale: お気に入りは人ごとの好みで、ノートのフォルダを共にする人やほかのマシンの印を変えないため(workspace.app-side-no-note-write と同じ考え)
- Acceptance:
  - Given:
  - When:
  - Then: 「趣味」と印の lab をお気に入りにする → `workspaces.toml` に `favorites = ["趣味", "~/lab"]`、lab の印のバイトは同じ。お気に入りの印の根が無くなった → 一覧から外して警告(cli.config-file-warning-form)。

#### 表・ガント・カンバン・グラフ・木・関係マップのどの画面からでも、キーとパレットの「ワークスペースを切り替える」で、お気に入りのワークスペースを入れた順に番号つきで並べた一覧を開けるべき
`req~workspace.switch-open~1` P1
- Rationale: 今は引き出しが表の画面と関係マップでしか開けず、ガントやカンバンから別のワークスペースへ移れないため(2026-11-20 に人が「どこからでも切り替え」と挙げた)
- Acceptance:
  - Given:
  - When:
  - Then: お気に入りが work・趣味 → カンバンの画面で切り替えのキー → 「1 work」「2 趣味」の一覧。お気に入りが無い → 一覧に、お気に入りの入れ方の案内の1行。
- Needs: stest

#### 切り替えの一覧は、起動したときに開いた所(ワークスペースでないフォルダ・`.base`・検知した根・ワークスペースのどれでも)から今は離れているなら、その所を「最初に開いた所」として一覧の先頭に置くべき
`req~workspace.switch-origin~1` P1
- Rationale: ワークスペースでない所から開いて別のワークスペースへ移ったあとでも、同じ切り替えの操作で最初の所へ戻れるようにするため(2026-11-20 に人が条件として挙げた)
- Acceptance:
  - Given:
  - When:
  - Then: ワークスペースでないフォルダ ~/scratch を開き、切り替えで work へ移る → もう一度切り替えを開くと先頭に「最初に開いた所: scratch」→ 選ぶ → ~/scratch の表に戻る。~/scratch にいるあいだ → 一覧に「最初に開いた所」は無い。
- Needs: stest

#### 切り替えの一覧で選んだワークスペースは、この起動のあいだにそのワークスペースで最後に見ていた表とビューで開き、まだ見ていなければ、印の `open`(workspace.marker-open)か最初の表で開くべき
`req~workspace.switch-target~1` P1
- Rationale: 行き来するたびに、さっき見ていた表を探し直さなくてよくするため
- Acceptance:
  - Given:
  - When:
  - Then: work の projects(進行中)を見てから趣味へ移り、また work を選ぶ → projects の進行中。この起動でまだ開いていない趣味を選ぶ → 趣味の最初の表。「最初に開いた所」を選ぶ → 起動したときの表で、そこで最後に見ていたビュー。

#### 切り替えの一覧では、数字のキーでその番号のワークスペースを選び、Esc で何もせずに閉じるべき
`req~workspace.switch-keys~1` P1
- Rationale: 一覧を開いて1打で移れるようにするため
- Acceptance:
  - Given:
  - When:
  - Then: 一覧で `2` → 趣味へ移る。Esc → 元の画面のまま。

#### ためた変更があるときに切り替えでほかのワークスペースへ移るなら、引き出しで移るとき(workspace.drawer-unsaved)と同じく保存か取りやめを確かめるべき
`req~workspace.switch-unsaved~1` P1
- Rationale: 移るついでに変更を黙って捨てないため
- Acceptance:
  - Given:
  - When:
  - Then: 未保存の変更があるときに一覧で趣味を選ぶ → 保存の確かめ。取りやめ → 元の画面のまま。
