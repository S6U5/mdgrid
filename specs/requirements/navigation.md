# 移動・検索・選択・一時的な並べ替え(navigation) の要求

## 目的

表の中の移動と、探す・絞る・並べ替える・列を隠す・行を選ぶ操作。`.base` の定義(base-view)は変えず、画面の中だけで効く(見た目の状態として残すものは screen の screen.state-separate・screen.state-store、ビューとして残すものは base-view の mdgrid のビュー base-view.own-views・base-view.views-toml)。csvlens・tabiew・VisiData・Obsidian Bases の操作から、表のノートに要るものを採った。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/navigation/spec.md](../_history/decidespec/navigation/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: ピボット・縦持ち化・転置(VisiData にあるが、フロントマターの編集という目的から外れる)。グラフは、人の依頼(2026-11-06)で別の画面の型として `chart`(chart.config〜chart.docs)に足した。
- 採らない: 表のシートを積むスタック(VisiData の型)。版1は `.base` と mdgrid のビュー(base-view.own-views)の切り替えと詳細の表示で足りる。

## 前提(旧い仕様で AI が置いた仮定)

- キーの既定は、vim 風と Bases の主なキーの両方(screen の screen.keys-vim-and-arrows)。ここに書いたキーは既定で、設定で変えられる。
- 簡易の絞り込みの既定のキー `\` は VisiData の「選択を外す」とぶつかるが、mdgrid は選択を Space と v で持つので、ぶつからない(AI の判断)。

## 要求

### 移動と検索

#### 行番号での移動(`:<番号>`)、先頭・末尾(`gg` `G`)、列の先頭・末尾(`0` `$` と Home / End)、画面の半分・1画面の移動(Ctrl+D / Ctrl+U、PageDown / PageUp)があるべき
`req~navigation.jump~1` P1
- 受け入れ: `:120` → 120行目が選ばれる。

#### `/` で表の文字を探し(画面の呼び名は「探す」。行を減らさないので、行を絞る検索(navigation.search)とは呼び分ける)、一致をすべて強調して `n` / `N` で前後に移るべきで、探す語に大文字が無ければ大文字小文字を区別しない(smartcase)べき
`req~navigation.find~1` P1
- 受け入れ: `/Todo` は大文字小文字を区別、`/todo` は区別しない。

#### 打つたびに行を絞り込む検索があるべき
`req~navigation.search-incremental~1` P1
- 受け入れ: `\` で上の欄に入り「会議」と打つ → 「会議」を含む行だけ。

#### 検索の入力は、表の上の検索の欄(navigation.search-box。`\` かクリックで入る)があればそこで、欄を出さない設定なら最下行で行うべき
`req~navigation.search-input-place~1` P1
- 受け入れ: 検索の欄のある表で `\` → 上の欄に入る。設定で欄を消して `\` を押す → 最下行で打てる。

#### 行を絞る検索は Esc で解除するべき
`req~navigation.search-esc~1` P1
- 受け入れ: `\` で「会議」と打って絞ったあと Esc → 元に戻る。

#### 検索は `.base` の絞り込みに重ねて効くべき
`req~navigation.search-over-base~1` P1
- 受け入れ: `.base` の filters で 10 行のビューで `\` で「会議」と打つ → 10 行のうち「会議」を含む行だけ。

#### 画面では、行を絞る検索を「検索」と呼び、行を減らさない `/`(navigation.find。「探す」)と、列の条件で絞る絞り込み(navigation.filter-window)と呼び分けるべき
`req~navigation.search-naming~1` P1
- 受け入れ: 起動(80×24)→ 行を絞る検索の欄の頭は「\ 検索…」で、「探す」とも「絞り込み」とも書かない。

#### ビューの帯(screen.layout)の、ビューのタブの右に、検索(navigation.search)の欄を常に出すべき
`req~navigation.search-box-place~1` P1
- 受け入れ: 起動(80×24)→ 真ん中の行にタブ「既定の表」、その右に空の検索の欄。

#### 検索の欄の頭に、キーと「検索」を示すべき(例「\ 検索」)
`req~navigation.search-box-label~1` P1
- 受け入れ: 起動(80×24)→ 空の検索の欄に「\ 検索…」。

#### 検索の欄に、効いている語と、合う行の数(例「12/80行」)を見せるべき
`req~navigation.search-box-hits~1` P1
- 受け入れ: 「会議」で絞る → 欄に「会議」と「3/16行」。

#### 検索の欄の右に、ボタン「絞り込み」(navigation.filter-window)・「並べ替え」(navigation.sort-window)・「分類」・「列」をこの順に並べるべき
`req~navigation.search-box-buttons~1` P1
- 受け入れ: 起動(80×24)→ 検索の欄の右に、絞り込み・並べ替えのボタン、「分類」「列」がこの順に並ぶ。設定の ⚙ は無い。

#### ビューの帯の右の端には、作れるときは「+ 新規」(cell-edit.new-note)を置くべき
`req~navigation.search-box-new~1` P1
- 受け入れ: 起動(80×24)→ 右の端に「a + 新規」。

#### ビューの帯のキーのあるボタンには、そのキーを添えるべき(例「S 並べ替え」)
`req~navigation.search-box-button-keys~1` P1
- なぜ: マウスで使う人もキーを覚えられるように
- 受け入れ: 起動(80×24)→ キーを添えた絞り込み・並べ替えのボタン(例「S 並べ替え」)。

#### ボタンの頭のキーの字は、名前と見分けられる見た目(太字で、名前と違う色。色を使わない表示では太字。ヘッダーのボタン(screen.breadcrumb)も同じ)にするべき
`req~navigation.search-box-key-glyph~1` P1
- なぜ: キーの字を名前の一部や数字と取り違えないように
- 受け入れ: 起動(80×24)→ ボタンの頭のキーの字は太字で、名前と見た目が違う。

#### ビューの帯の「分類」と「列」のボタンは、ビューの設定の画面(navigation.settings-layout)をその区画で開くべき
`req~navigation.search-box-settings-buttons~1` P1
- 受け入れ: 「分類」を押す → ビューの設定の画面の分類の区画。

#### 端末の幅が足りないときは、列・分類のボタン、今のでないビューのタブ(`display.tab_wrap` が true なら省かずに、検索の欄の行の下へ並べる。navigation.tab-wrap)の順に省き、今のビューのタブと検索の欄と、絞り込み・並べ替え・「+ 新規」のボタンをなるべく残すべき
`req~navigation.search-box-narrow~1` P1
- 受け入れ: 幅 60 → 列・分類が無く、絞り込みと並べ替えと「+ 新規」は出る。

#### ビューの帯は、既定で上下に1行ずつ余白を取った3行の高さにし、タブとボタンを真ん中の行に並べるべき
`req~navigation.search-box-padding-default~1` P1
- なぜ: 表の上が詰まって見えないように
- 受け入れ: 起動(80×24)→ ビューの帯は既定の余白 1 で、上下に1行ずつ空き、真ん中の行にタブ「既定の表」とボタン。

#### ビューの帯の上下の余白は、設定 `display.bar_padding`(0〜2 を 0.5 刻み。既定 1)で変えられるべき
`req~navigation.search-box-padding-setting~1` P1
- なぜ: 端数は端末とテーマによって半行にも1行にもなり、環境で見え方がぶれるので、既定は端数にしない
- 受け入れ: `display.bar_padding = 1` → 上下に1行ずつ空く。`2` → 上下に2行ずつ空く。

#### `display.bar_padding` が 0 では、余白を取らずにビューの帯を1行にするべき
`req~navigation.search-box-padding-zero~1` P1
- 受け入れ: `display.bar_padding = 0` → ビューの帯は1行。

#### 余白の 0.5 の端数は、帯の外側の1行を半行の余白に見せるべき
`req~navigation.search-box-padding-half~1` P1
- なぜ: 端末は行の単位でしか空けられないため
- 受け入れ: `display.bar_padding = 0.5` → 帯の上下の行が半分だけ帯の地の色で塗られる。

#### 半行の余白の行は、帯の行と同じくセルの地の色で帯の色を塗り、帯の外側の半分を、画面の地の色の半分の塗りの字(上の余白は `▀`、下の余白は `▄`)で消して描くべき
`req~navigation.search-box-half-glyphs~1` P1
- なぜ: 字の色と地の色を端末が少し違えて描いても、帯の中に継ぎ目が出ないように
- 受け入れ: `display.bar_padding = 0.5` → 上の余白の行はセルの地の色が帯の色で、字が画面の地の色の `▀`。下の余白の行は同じく `▄`。

#### 画面の地の色は、テーマが画面の地を塗るときはその色、塗らないときは端末への問い合わせ(screen.theme-auto と同じ OSC 11。短い待ち時間で、答えが無ければ諦める)で知った端末の地の色とするべき
`req~navigation.search-box-screen-bg~1` P1
- 受け入れ: テーマが画面の地を塗らず、端末が OSC 11 に `rgb:0000/0000/0000` と答える → `display.bar_padding = 0.5` の半分の塗りの字は黒で描かれる。テーマが画面の地を塗る → 半分の塗りの字はテーマの地の色。

#### 余白に端数があるときは、帯の行に地の色を敷くべき
`req~navigation.search-box-half-fill~1` P1
- 受け入れ: `display.bar_padding = 0.5` → 帯は地の色の付いた帯に見える。

#### 色を使わない表示・幅があいまいな字を全角として数える設定(半分の塗りの字が2桁になりうる)・画面の地の色が分からないときは、余白の端数を1行に切り上げて空き行にするべき
`req~navigation.search-box-half-fallback~1` P1
- 受け入れ: `--no-color` で `0.5` → 上下に1行ずつ空く。

#### ビューの帯の囲み方は設定 `display.buttons` で選べ、`"plain"`(線で囲まず、欄とボタンを地の色で示す。既定)か `"box"`(検索の欄と、絞り込み・並べ替えのボタンを、それぞれ上下の余白の行に線を引いて四角く囲み、線の中は地の色で塗らない。余白が 0 なら囲まない)にできるべき
`req~navigation.search-box-style~1` P1
- 受け入れ: 設定 `display.buttons = "box"` → 欄と絞り込み・並べ替えが線で四角く囲まれ、線の中は地の色が無い。

#### 並べ替えと絞り込みの窓(navigation.sort-window・navigation.filter-window)は、ビューの帯のすぐ下(`"box"` では囲みの下の線のすぐ下)に開くべき
`req~navigation.search-box-window-below~1` P1
- 受け入れ: 「| 絞り込み」を押す → 窓は帯のすぐ下に開く。

#### 端末の高さが16行に満たないときは、ビューの帯の余白を 0 として描くべき
`req~navigation.search-box-low-height~1` P1
- なぜ: 表の行を残すため
- 受け入れ: 高さ 14 → ビューの帯は1行。

#### 検索の欄は設定で出さないこともでき、出さないときはビューの帯を1行にしてよい
`req~navigation.search-box-hide~1` P1
- 受け入れ: 設定で欄を消す → 欄が無い。

#### 設定 `display.tab_wrap`(真偽。既定 false)を true にした表では、ビューのタブが帯の真ん中の行(navigation.search-box)に入りきらないとき、今のでないタブを省かずに、タブを全部真ん中の行から外し、真ん中の行のすぐ下に、左から幅いっぱいに並べて要る行の数だけ折り返すべき
`req~navigation.tab-wrap-below~1` P1
- なぜ: タブが多くても全部を見て選べ、タブの並びが途中で切れないように
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true` → 検索の欄の行は動かず、すぐ下の行から間を空けずにタブの行が続き、7つのタブの名前が全部画面にある。

#### `tab_wrap` でタブを折り返すとき、真ん中の行は検索の欄とボタンだけにし、検索の欄を左の端に寄せるべき
`req~navigation.tab-wrap-middle-row~1` P1
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true` → 検索の欄の行にタブは無く、検索の欄は左の端にある。

#### 設定 `display.tab_wrap_gap`(真偽。既定 false)を true にすると、真ん中の行とタブの行の間に空き行を1つ入れるべき
`req~navigation.tab-wrap-gap~1` P1
- なぜ: 詰めるか空けるかは人の好みで選べるように
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true`、`tab_wrap_gap = true` → 検索の欄の行とタブの行の間に空き行が1つ、表はさらに1行下がる。

#### 囲む帯(`buttons = "box"`)では、折り返したタブの行を囲みの下の線の行の下に置くべき
`req~navigation.tab-wrap-box~1` P1
- なぜ: 囲みを切らないため
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true`、`buttons = "box"` → タブの行は囲みの下の線の下。

#### タブを折り返すときは、帯の下の余白(`bar_padding`)をタブの行のさらに下に置くべき
`req~navigation.tab-wrap-padding~1` P1
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true`、`bar_padding = 1` → タブの行の下に空き行が1つあり、その下から表。

#### 余白の端数で帯の行に地の色を敷くときは、折り返したタブの行と空き行にも同じ地の色を敷くべき
`req~navigation.tab-wrap-fill~1` P1
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true`、`tab_wrap_gap = true`、`bar_padding = 0.5` → タブの行と空き行も、帯の行と同じ地の色。

#### タブを折り返して足した行のぶん、表は下がるべき
`req~navigation.tab-wrap-shift~1` P1
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true` → 表はタブの行の数だけ下がる。

#### タブを折り返して足した行のタブも、クリックで選べるべき
`req~navigation.tab-wrap-click~1` P1
- 受け入れ: 余白 2 と空き行のあとのタブをクリック → そのビュー、空き行のクリックでは変わらない。

#### タブを出さないとき(screen.tabs-setting の `tabs = "never"` と、1つだけで隠す `"auto"`)は、`tab_wrap` は何もしないべき
`req~navigation.tab-wrap-no-tabs~1` P1
- 受け入れ: 7つのタブの表を幅 60 の端末で `tab_wrap = true`、`tabs = "never"` → タブの行を足さない。

#### `tab_wrap` は、ビューの設定の「表示」の区画(navigation.settings-layout)では切り替えない(ビューごとの値は views.toml のビューの `display` に書ける)
`req~navigation.tab-wrap-not-in-display~1` P1
- なぜ: screen.note-name と同じく、表示の区画の数とその写しを増やさないため
- 受け入れ: ビューの設定の「表示」の区画 → `tab_wrap` の項目が無い。

#### タブが真ん中の行に入りきるときと `tab_wrap` が false のときは、今の1行のまま(navigation.search-box)にするべき
`req~navigation.tab-wrap-fits~1` P1
- 受け入れ: 幅 200 → 足す行は無く今と同じ。`tab_wrap = false`(既定)で幅 60 → 今のタブだけ(navigation.search-box のまま)。

#### タブが真ん中の行に入りきるときと `tab_wrap` が false のときは、`tab_wrap_gap` も何もしないべき
`req~navigation.tab-wrap-gap-idle~1` P1
- 受け入れ: 幅 200 か `tab_wrap = false` では `tab_wrap_gap` があっても何も変わらない。

#### `display.tab_wrap` は、表のプロファイルの項目として全体・ワークスペース・表・ビューの範囲(screen.profile-scopes)に書けるべき
`req~navigation.tab-wrap-profile~1` P1
- 受け入れ: notes/.mdgrid/workspace.toml に `[display] tab_wrap = true` → notes の7つのタブの表を幅 60 の端末で開くと、タブが折り返す。

#### `.base` のビュー(ワークスペースの印の `[table.base]` のビューを含む。workspace.marker-base)に `tab_break = true`(YAML では `tab_break: true`)を書くと、タブの行をそのタブの前で改めるべき(mdgrid のビュー(views.toml)には今は書けない)
`req~navigation.tab-break-row~1` P1
- なぜ: 期限で選ぶタブと状態で選ぶタブのように、まとまりごとに行を分けて見られるように(どこで折り返すかを幅に任せると、まとまりが行の途中で混ざるため)
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true` → すぐ下の行に A と B、その下の行に C と D がある。

#### タブの行に出すタブ(navigation.view-tabs の見せる順で、隠したタブを除いたもの)の先頭でないものに切れ目があるときは、端末の幅と `tab_wrap` に依らず、タブを全部真ん中の行から外すべき(navigation.search-box の今のでないタブを省く決まりと、navigation.tab-wrap の入りきるとき・`display.tab_wrap = false` のときは1行のままの決まりより先に効く)
`req~navigation.tab-break-precedence~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、幅 200・`tab_wrap = false` → 検索の欄の行にタブが無く、表はタブの行の2行ぶん下がる。

#### 切れ目があるときのタブは、navigation.tab-wrap と同じ置き場(真ん中の行の下。`tab_wrap_gap` の空き行(タブの行の前に1つだけで、行の間には入れない)・`buttons = "box"` の囲みの下・下の余白と地の色・クリックで選べること・検索の欄を出さないときと低い端末の扱いも navigation.tab-wrap と同じ)に、切れ目のタブの前で行を改めて左から並べるべき
`req~navigation.tab-break-place~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、`tab_wrap_gap = true` → 検索の欄の行とタブの行の間に空き行が1つで、2つのタブの行の間には空き行が無い。下の行の D をクリック → D のビュー。

#### 切れ目で分けた1つのタブの行が幅に入らないときは、その行の中で折り返し、タブを省かないべき(`tab_break` は `tab_wrap` を含む)
`req~navigation.tab-break-overflow~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、幅を狭めて A・B の行が入らない → その行の中で折り返し、4つのタブの名前が全部画面にある。

#### 隠したタブの `tab_break` は、次に見せるタブに移すべき
`req~navigation.tab-break-hidden~1` P1
- なぜ: まとまりの先頭を隠しても、まとまりが混ざらないように
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、C を隠す → 下の1行目に A と B、2行目に D。

#### 先頭のタブの `tab_break` は何もしないべき(そのときは navigation.search-box・navigation.tab-wrap のまま)
`req~navigation.tab-break-first~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で A にだけ `tab_break` → 今のまま、真ん中の行にタブ(navigation.search-box)。

#### タブを出さないとき(screen.tabs-setting)は、`tab_break` は何もしないべき
`req~navigation.tab-break-no-tabs~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、`tabs = "never"` → タブの行は無い。

#### `tab_break` はビューと一緒に動くべき(navigation.view-tabs で並べ替えると切れ目もそのタブと動く)
`req~navigation.tab-break-moves~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、ビューの区画で C を B の前へ上げる → 下の1行目に A、2行目に C と B と D。

#### `tab_break` は行の置き方だけを変え、`[` `]` の順(navigation.view-tabs)は変えないべき
`req~navigation.tab-break-key-order~1` P1
- 受け入れ: 4つのタブ A・B・C・D の表で C に `tab_break = true`、A から `]` を3回 → B・C・D の順に移る。

#### 選んだセルと同じ値の行を強調する操作と、同じ値の行だけに絞る操作があるべき
`req~navigation.same-value~1` P1
- 受け入れ: status が done のセルで絞る → done の行だけ。

### 並べ替えと列

#### 列の見出しから並べ替え(昇順 → 降順 → 解除)ができるべき
`req~navigation.header-sort-cycle~1` P1
- 受け入れ: 見出しを2回 → 降順、3回 → `.base` の並びに戻る。

#### 見出しからの並べ替えは、今のビューの設定の並べ替え(navigation.view-settings)として当て、見た目の状態(screen.state-store)に保存して、次に開いても同じ並びにするべき
`req~navigation.header-sort-save~1` P1
- なぜ: よくある表の道具と同じく、選んだ並びを覚えるため
- 受け入れ: 見出しを1回押して due の昇順にし、終了して開き直す → due の昇順のまま。

#### 見出しからの並べ替えで、`.base` の並べ替えとファイルは変えるべきでない
`req~navigation.header-sort-base-untouched~1` P1
- 受け入れ: 見出しを2回 → `.base` のバイトは変わらない。

#### 列を隠す・戻す、並びを変える、幅を変える(ドラッグと `<` `>`)、左の列(既定はファイル名)を固定する操作があるべき
`req~navigation.columns~1` P1
- 受け入れ: 列を隠して終了し再起動 → 隠したまま(見た目の状態)。

#### ビューの帯(navigation.search-box)の「絞り込み」のボタンの右に「並べ替え」のボタンを出し、押すと、ボタンのすぐ下に、今のビューの並べ替えの決まり(列と向き。複数)を並べた小さな窓を開くべき
`req~navigation.sort-window-open~1` P1
- 受け入れ: 検索の欄の右の「並べ替え」を押す → 窓が開き「+ 並べ替えを足す」。

#### 並べ替えの窓では、決まりを足す(列を選ぶ)・外す・向きを変える・順を入れ替えることができるべき
`req~navigation.sort-window-edit~1` P1
- 受け入れ: 窓で due を足したあと due の行で Enter → 降順「↓ due」。窓で決まりを外す → 元の並び。読むだけの起動でも並べ替えられる(ノートは書かない)。

#### 並べ替えの窓で変えると、すぐ表に効かせるべき
`req~navigation.sort-window-live~1` P1
- 受け入れ: 窓で due を選ぶ → すぐ due の昇順になる。

#### 並べ替えの窓での変更は、設定の並べ替え(navigation.view-settings)として見た目の状態(screen.state-store)に保存するべき
`req~navigation.sort-window-save~1` P1
- 受け入れ: 窓で due を選び、終了して開き直す → 同じ並び。

#### 並べ替えが効いている間は、並べ替えのボタンに先頭の決まりを見せる(例「↑ due」、2つ以上なら「↑ due +1」)
`req~navigation.sort-window-label~1` P1
- 受け入れ: 窓で due を選ぶ → ボタンに「↑ due」。続けて2つめの決まりを足す → ボタンに「↑ due +1」。

#### 並べ替えの窓は、窓の外のクリックか Esc で閉じる
`req~navigation.sort-window-close~1` P1
- 受け入れ: 並べ替えの窓を開いて Esc → 窓が閉じる。窓の外をクリック → 窓が閉じる。

#### 検索の欄を出さない設定(navigation.search-box)では、並べ替えの窓を操作の一覧とコマンドのパレットから開けるべき
`req~navigation.sort-window-palette~1` P1
- 受け入れ: 検索の欄を出さない設定でパレットから並べ替えの窓を開く → 「+ 並べ替えを足す」のある窓が開く。

#### ビューのタブのうち1つを「既定のビュー」に選べるべきで、操作の一覧とコマンドのパレットの「既定のビューにする」で今のビューを選び、mdgrid の設定の置き場の `views.toml` の、開いた対象の項目に名前で残すべき(ノートのフォルダと `.base` には書かない)
`req~navigation.default-view-set~1` P1
- 受け入れ: ビューが3つの対象で「進行中」のタブを選び「既定のビューにする」→ views.toml の対象に default_view = "進行中"。

#### 起動時は、ビューを名前で指定していなければ(`--view`・登録した表のビュー)、既定のビューを開くべき
`req~navigation.default-view-open~1` P1
- 受け入れ: 「進行中」を既定のビューにして終了して開き直す → 「進行中」が選ばれている。`--view 既定の表` で開く → 既定の表。

#### 既定のビューの名前が無くなっていたら、警告せずに先頭のビューを開くべき
`req~navigation.default-view-missing~1` P1
- 受け入れ: 既定のビューの「進行中」を消す → 次の起動は先頭。

#### 読むだけの起動(`--readonly`)では、既定のビューを選べないべき
`req~navigation.default-view-readonly~1` P1
- 受け入れ: `--readonly` → 「既定のビューにする」が出ない。

#### ビューの設定の画面(navigation.settings-layout)の左の一覧に「ビュー」の区画を持ち、開いた対象のビューのタブ(`.base` のビューと mdgrid のビュー)を今のタブの順に並べるべき
`req~navigation.view-tabs-section~1` P1
- 受け入れ: ビューが3つ(既定の表・進行中・未完了)の対象で `o` からビューの区画 → 既定の表・進行中・未完了の順に並ぶ。

#### ビューの区画で、タブの順を変えられるべき
`req~navigation.view-tabs-order~1` P1
- 受け入れ: ビューの区画で「進行中」を K で上へ → タブが「進行中 既定の表」の順。

#### ビューの区画で、タブに出すか隠すかを変えられるべき
`req~navigation.view-tabs-hide~1` P1
- 受け入れ: ビューの区画で「未完了」を隠す → すぐ表の上のタブが2つになる。

#### ビューの区画で、既定のビュー(navigation.default-view)を変えられるべき
`req~navigation.view-tabs-default~1` P1
- 受け入れ: ビューの区画で「進行中」を既定にする → 「既定」の印が付き、views.toml に default_view = "進行中"。

#### ビューの区画で、mdgrid のビューの名前を変えられるべき(base-view.save-view)
`req~navigation.view-tabs-rename~1` P1
- 受け入れ: ビューの区画で「進行中」の名前を「作業中」に変える → タブも変わる。

#### ビューの区画で、mdgrid のビューを消せるべき(base-view.save-view)
`req~navigation.view-tabs-delete~1` P1
- 受け入れ: ビューの区画で mdgrid のビュー「進行中」を消す → タブの行から「進行中」が無くなる。

#### ビューの区画で、タブの行の切り替えの案内(「[ ] で切り替え」)を出すかを変えられるべき
`req~navigation.view-tabs-hint~1` P1
- 受け入れ: 切り替えの案内をオフ → タブの行に「[ ] で切り替え」が無い。

#### ビューの区画の変更は、反映を待たずにすぐ `views.toml` の開いた対象の項目に残すべき(ノートのフォルダと `.base` には書かない。base-view.base-untouched)
`req~navigation.view-tabs-save~1` P1
- なぜ: この区画の変更はビューの一覧の管理で、今のビューの見せ方ではないため
- 受け入れ: ビューの区画でタブの順と隠し方を変え、終了して開き直す → 同じ順・同じ隠し方。

#### 隠したタブは、タブの行にも `[` `]` の切り替えにも出さないべき
`req~navigation.view-tabs-hidden-skip~1` P1
- 受け入れ: ビューの区画で「未完了」を隠す → `]` で未完了に移らない。

#### 隠したタブも、`--view` とビューの区画からは開けるべき
`req~navigation.view-tabs-hidden-open~1` P1
- 受け入れ: 「未完了」を隠したあと `--view 未完了` で開く → 未完了のビュー。

#### 今開いているビューは隠せず、すべてを隠すことはできないべき
`req~navigation.view-tabs-keep-current~1` P1
- 受け入れ: ビューの区画で今のビューを隠そうとする → 隠さず理由が出る。

#### `.base` のビューは、名前の変更と削除をできないべき
`req~navigation.view-tabs-base-fixed~1` P1
- なぜ: `.base` を書き換えないため
- 受け入れ: ビューの区画で `.base` のビューの名前の変更 → 出来ない理由が出る。

#### 読むだけの起動(`--readonly`)では、ビューの区画を見せるが変えられないべき(cli.readonly)
`req~navigation.view-tabs-readonly~1` P1
- 受け入れ: `--readonly` → 区画は見えて、変える操作で読むだけの理由。

#### ビューの区画の印は、タブに出している数と全部の数(例「3/4」)にするべき
`req~navigation.view-tabs-count~1` P1
- 受け入れ: ビューが3つ(既定の表・進行中・未完了)の対象で `o` → 左に「ビュー 3/3」。「未完了」を隠す → 「ビュー 2/3」。

#### ビューの設定の画面(navigation.settings-layout)の左の一覧に「親子」の区画を持ち、そこでオンにしたビューでは、ノートのキー(既定 `parent`。区画で選べる)に書いた親のノートへのリンク(relations.link-forms と同じ書き方と解き方。リストなら先頭のリンク)で、子の行を親の行のすぐ下に並べるべき
`req~navigation.parent-child-rows~1` P1
- 受け入れ: `parent: "[[設計]]"` の「画面」と「API」、`parent: "[[画面]]"` の「部品」がある表で親子をオン → 「設計」の下に「画面」・その下に「部品」・「API」。

#### 親子で並べた行は、深さ1つごとに2桁字下げしてノートの欄に見せるべき
`req~navigation.parent-child-indent~1` P1
- 受け入れ: `parent: "[[設計]]"` の「画面」と「API」、`parent: "[[画面]]"` の「部品」がある表で親子をオン → 「画面」(字下げ2桁)・「部品」(4桁)・「API」(2桁)。

#### 親子で並べたとき、兄弟の順は今の並べ替え(navigation.header-sort・navigation.sort-window・`.base` の sort)のままにするべき
`req~navigation.parent-child-sibling-order~1` P1
- 受け入れ: 親子をオンにした表を title の降順で並べる → 兄弟の「画面」と「API」だけが入れ替わる。

#### 親子で並べたとき、子を持つ行の名前の前に ▾(開いている)か ▸(畳んでいる)を付けるべき
`req~navigation.parent-child-marker~1` P1
- 受け入れ: 親子をオン → 「▾ 設計」。「設計」で `Z` → 「▸ 設計」。

#### 親子で並べたとき、`Z`(Shift+z。とその印のクリック)で子孫を畳む・開くべき
`req~navigation.parent-child-fold~1` P1
- 受け入れ: 「設計」で `Z` → 子孫が隠れる。もう一度 `Z` → 戻る。

#### 親が今の表に無い子(絞り込みで外れた・別のまとまり・解けないリンク)と、親子が輪になった行は、一番上の段に並べるべき
`req~navigation.parent-child-orphans~1` P1
- なぜ: 行を落とさないため
- 受け入れ: 「設計」を絞り込みで外す → 「画面」と「API」が一番上の段。A の親が B・B の親が A → 両方とも一番上の段。

#### グループ分け(navigation.settings-group)があるときは、まとまりの中で親子に並べるべき
`req~navigation.parent-child-groups~1` P1
- 受け入れ: 「設計」「画面」「API」が同じ status のまとまりにある表を status でグループ分けし、親子をオン → そのまとまりの中で「設計」の下に「画面」と「API」。

#### 親子のオン・オフとキーは、ビューの設定として反映で効くべき
`req~navigation.parent-child-apply~1` P1
- 受け入れ: 親子をオフにして反映 → 今までの並び。

#### 親子のオン・オフとキーは、見た目の状態と mdgrid のビュー(base-view.own-views)に残るべき
`req~navigation.parent-child-save~1` P1
- 受け入れ: 親子をオンにして反映し、終了して開き直す → 親子の並びのまま。

#### ビューの設定の画面の「親子」の区画で、親子で並べているビュー(navigation.parent-child)に WBS をオンにできるべき
`req~navigation.wbs-on~1` P1
- 受け入れ: 親子をオンにしたビューで、親子の区画の WBS をオンにして反映 → 行の名前の前に番号が付く。

#### WBS がオンのとき、行の名前の前に WBS の番号(まとまりの中で一番上の段を 1 から、子は親の番号に `.` と兄弟の中の順をつなぐ。例 `1.2.1`)を付けるべき
`req~navigation.wbs-number~1` P1
- 受け入れ: 設計の下に画面(その下に部品・ボタン)と API がある表で、親子と WBS をオン → 「1 設計」「1.1 画面」「1.1.1 部品」「1.1.2 ボタン」「1.2 API」。

#### 親子の区画では、進み具合のキー(既定 `status`)と、その値ごとの割合(0〜100)とラベルの対応表を作れるべき
`req~navigation.wbs-map~1` P1
- 受け入れ: 区画で進み具合のキーを status にし、done = 100 完了・doing = 50 作業中・todo = 0 未着手 と打つ → 対応表にその3つの行ができる。

#### 対応表を作るときは、表の行に出てくる値を件数つきで並べ、値ごとに割合とラベルを打って決める
`req~navigation.wbs-map-values~1` P1
- 受け入れ: status が done・doing・todo の行が1つずつの表 → 区画に done・doing・todo が、それぞれ件数 1 を添えて並ぶ。

#### WBS がオンのとき、子の無い行は、名前のあとに対応表のラベルを出すべき
`req~navigation.wbs-leaf-label~1` P1
- 受け入れ: 設計の下に画面(その下に部品 done・ボタン todo)と API(doing)がある表で、対応表で done = 100 完了・doing = 50 作業中・todo = 0 未着手 → 「1.1.1 部品 · 完了」「1.1.2 ボタン · 未着手」「1.2 API · 作業中」。対応表に無い値 blocked → ラベルは出さない。

#### WBS がオンのとき、子のある行は、子孫の子の無い行の割合の平均を `%` で出すべき(対応表に無い値と値の無い行は 0% と数える)
`req~navigation.wbs-parent-percent~1` P1
- 受け入れ: 設計の下に画面(その下に部品 done・ボタン todo)と API(doing)がある表で、対応表で done = 100・doing = 50・todo = 0 → 「1 設計 · 50%」「1.1 画面 · 50%」。API の status を空にする → 「1 設計」は 33%(100+0+0 の平均)。対応表に無い値 blocked → 0% と数える。

#### WBS のオン・オフ、進み具合のキー、対応表は、ビューの設定として反映で効くべき
`req~navigation.wbs-apply~1` P1
- 受け入れ: 対応表で doing のラベルを「作業中」にして反映 → 「1.2 API · 作業中」。

#### WBS のオン・オフ、進み具合のキー、対応表は、見た目の状態と mdgrid のビュー(base-view.own-views)に残るべき
`req~navigation.wbs-save~1` P1
- 受け入れ: 対応表を作って反映し、開き直す → 対応表が残る。

#### 親子で並べていないときは WBS を出さないべき
`req~navigation.wbs-needs-tree~1` P1
- 受け入れ: 親子と WBS をオンにした表で親子をオフ → 番号と進み具合が消える。

#### ビューの帯(navigation.search-box)の、検索の欄の右に「絞り込み」のボタンを出し、押すと、ボタンのすぐ下に、今のビューの設定のフィルターの条件(navigation.settings-filter・navigation.filter-conditions)を1つ1行の表(列・比べ方・値・外す)で並べた小さな窓を開くべき
`req~navigation.filter-window-open~1` P1
- なぜ: 設定の画面(navigation.view-settings)を開いて反映する手間なしに、並べ替えの窓(navigation.sort-window)と同じ手軽さで絞るため
- 受け入れ: ビューの帯の「絞り込み」を押す → ボタンの下に窓が開き「+ 絞り込みを足す」。

#### 絞り込みの窓では、条件を足す(列を選び、その列に合う条件を選ぶ: 値の一覧のチェック・含む・含まない・比較・空である・空でない)・外す・変えることができるべき
`req~navigation.filter-window-edit~1` P1
- 受け入れ: 窓で status の done を外したあと priority の `≥ 3` を足す → 両方を満たす行だけ。窓で条件を外す → 元の行。

#### 絞り込みの窓で条件を変えるときは、条件の行の列・比べ方・値のどれかを選んで、その場で直せるべき
`req~navigation.filter-window-inline~1` P1
- 受け入れ: 条件の行の比べ方を選んで「含む」に変え、「ing」と打つ → その場で条件が変わる。

#### 絞り込みの窓で変えると、すぐ表に効かせるべき
`req~navigation.filter-window-live~1` P1
- 受け入れ: 窓で status を選び done のチェックを外す → すぐ done の行が消え、上の帯に「status: done を除く」。

#### 絞り込みの窓での変更は、設定のフィルター(navigation.view-settings)として見た目の状態(screen.state-store)に保存するべき
`req~navigation.filter-window-save~1` P1
- 受け入れ: 窓で status の done のチェックを外し、終了して開き直す → 同じ。

#### 絞り込みの条件が効いている間は、絞り込みのボタンに条件の数を見せる(例「絞り込み 2」)
`req~navigation.filter-window-count~1` P1
- 受け入れ: 窓で status を選び done のチェックを外す → ボタンに「絞り込み 1」。

#### 絞り込みの窓は、窓の外のクリックか Esc で閉じる
`req~navigation.filter-window-close~1` P1
- 受け入れ: 絞り込みの窓を開いて Esc → 窓が閉じる。窓の外をクリック → 窓が閉じる。

#### 検索の欄を出さない設定(navigation.search-box)では、絞り込みの窓を操作の一覧とコマンドのパレットから開けるべき
`req~navigation.filter-window-palette~1` P1
- 受け入れ: 検索の欄を出さない設定 → パレットから絞り込みの窓を開ける。端末が狭くてボタンが入らない → ボタンを出さず、パレットから開ける。

#### 絞り込みの窓は、読むだけの起動(cli.readonly)でも使え、ノートは書かないべき
`req~navigation.filter-window-readonly~1` P1
- 受け入れ: 読むだけの起動で窓から status の done を外す → 絞れる(ノートは書かない)。

#### 表のモードで、次の問題(cell-view.problems)のセルへ跳ぶ操作(既定 `!`。screen.key-table のキーの表で割り当て直せ、パレットと操作の一覧からも使える)があるべき
`req~navigation.problem-jump-next~1` P1
- 受け入れ: 3つの問題のある表で `!` → 1つめの問題のセルに移る。

#### 表のモードで、前の問題へ跳ぶ操作(既定 `@`。screen.key-table のキーの表で割り当て直せ、パレットと操作の一覧からも使える)があるべき
`req~navigation.problem-jump-prev~1` P1
- なぜ: `@` は `!` の隣のキーで、`g` で始まる前置きのキーを増やさない(screen.key-modes)
- 受け入れ: 3つの問題のある表で `!` を2回のあと `@` → 前へ(1つめの問題のセル)。

#### 問題へ跳ぶ操作は、表の行の順に(同じ行では列の順に)探して、端で反対の端に回るべき
`req~navigation.problem-jump-wrap~1` P1
- 受け入れ: 3つの問題のある表で、3回目の `!` のあと `!` → 1つめに戻る。

#### 問題へ跳んだら、メッセージ行にそのセルの問題の理由を出すべき
`req~navigation.problem-jump-reason~1` P1
- 受け入れ: 3つの問題のある表で `!` → メッセージ行に「due: 型が合わない(日付ではない)」。

#### 問題が無ければ、問題へ跳ぶ操作はそう知らせて動かないべき
`req~navigation.problem-jump-none~1` P1
- 受け入れ: 問題の無い表 → `!` で「問題は無い」。

#### 下の帯に、今の表(ビューの絞り込みのあとの行と、見えている列)の問題の数を、1つ以上あるときだけ出すべき(例「問題 3」)
`req~navigation.problem-jump-count~1` P1
- 受け入れ: 3つの問題のある表 → 下の帯に「問題 3」。問題の無い表 → 帯に数が出ない。

#### 問題のある行だけに絞る切り替えを、パレット・操作の一覧・下の帯の問題の数のクリックから使えるべき
`req~navigation.problem-jump-filter~1` P1
- 受け入れ: 3つの問題のある表でパレットの「問題のある行だけ」→ 問題のある行だけ。

#### 問題のある行だけに絞っている間は、それと分かるように出すべき
`req~navigation.problem-jump-filter-shown~2` P1
- 受け入れ: パレットの「問題のある行だけ」で絞る → ヘッダーに「問題のある行だけ」(英語の画面では `only rows with problems`)が出る。もう一度の切り替えか Esc で外す → ヘッダーから消える。

#### 問題のある行だけの絞り込みは、もう一度の切り替えで外せるべき
`req~navigation.problem-jump-filter-off~1` P1
- 受け入れ: パレットの「問題のある行だけ」で絞ったあと、もう一度 → 元に戻る。

#### 問題へ跳ぶ操作と問題のある行だけに絞る切り替えは、読むだけの起動(cli.readonly)でも使えるべき
`req~navigation.problem-jump-readonly~1` P1
- 受け入れ: `--readonly` で3つの問題のある表 → `!` で1つめの問題のセルに移り、パレットの「問題のある行だけ」で問題のある行だけになる。

### 並べ替えと列(旧い仕様の「承認の記録」の節にあった行)

#### ためた値でも絞り込み・並べ替え・グループを計算し直すべきで、直した行が目の前から消えないように、値を直した行は、保存するか移動の操作をするまで元の位置に留めるべき
`req~navigation.keep-edited-rows~1` P1
- 受け入れ: status で絞ったビューで done に直す → 行は残り、印が付く。保存で消える。

### 選択と詳細

#### 行に印を付ける(Space)、範囲で選ぶ(`v` と Shift+矢印)、全部を選ぶ、選択を解く、ができるべき
`req~navigation.select-rows~1` P1
- 受け入れ: Space を3行 → 下の帯に「選択 3」。

#### 選んだ行の全プロパティを縦に並べた詳細の表示(`K`)があるべき
`req~navigation.detail-view~1` P1
- 受け入れ: 行で `K` → その行の全プロパティが縦に並ぶ。

#### 詳細の表示からも編集できるべき
`req~navigation.detail-edit~1` P1
- 受け入れ: 詳細の表示で status の値を done に直して保存 → ノートの status が done になる。

#### 詳細の表示では、長い値は折り返して全文を見せるべき
`req~navigation.detail-wrap~1` P1
- 受け入れ: 詳細の表示で長い説明の値 → 折り返して全文。

#### 詳細の表示のプロパティの下には、本文(フロントマターのあと)の先頭の最大 20 行を、制御文字を cell-view.grapheme-width のとおり出さずに、読むだけで見せるべき
`req~navigation.detail-body~1` P1
- なぜ: ノートを開かずに中身を確かめられるよう
- 受け入れ: 本文のあるノートで `K` → プロパティの下に「本文」の見出しと先頭の行。本文の無いノートでは見出しを出さない。

### ビューの設定

#### 表の見せ方を変えるビューの設定の画面を持ち、そこでフィルター・並べ替え・グループ分け・表示する列を組んで、反映の操作で表に効かせるべきで、反映するまでは表を変えず、取り消せば元のままにするべき
`req~navigation.view-settings~1` P1
- 受け入れ: 設定の画面(左の一覧のフィルターの区画)で status の done を隠す → 反映の前は表に done の行が残り、反映で消える。取り消しで閉じる → 表は開く前のまま。

#### 設定のフィルターは、todo の status に限らない用途で使うため、特定の列に決め打ちせず、どの列でも、その列の値を選んで、その値の行を隠すか、その値の行だけを出すかを決められるべき
`req~navigation.settings-filter~1` P1
- 受け入れ: status の done を外す → done の行が消える。種別の列の 本 だけを残す → 本 の行だけ。キーの無いノートは「(空)」として選べる。

#### 設定で、グループ分けをする列を選ぶことと、グループ分けをしないことを選べるべきで、`.base` の groupBy より設定を優先し、`.base` は書き換えないべき
`req~navigation.settings-group~1` P1
- 受け入れ: groupBy のある `.base` のビューで「しない」を選んで反映 → 見出しの行が消える。種別の列を選ぶ → 種別ごとの見出し。`.base` のバイトは同じ。

#### 設定で効いているフィルター・並べ替え・グループ分けを、表の上の帯に並べて見せるべき
`req~navigation.settings-bar-items~1` P1
- 受け入れ: 設定の画面のフィルターの区画で status の done を外して、上の「反映」 → 表の上に「status: done を除く」の項目が出る。

#### 設定で何も効いていなければ、表の上の帯を出さないでよい
`req~navigation.settings-bar-empty~1` P1
- 受け入れ: 設定で何も効かせていない表 → 表の上に設定の帯が無い。

#### 設定は、開いた対象とビューごとの見た目の状態(screen.state-separate・screen.state-store)に覚え、次に同じものを開いたときに当て、`.base` とノートには書かないべき
`req~navigation.settings-remember~1` P1
- 受け入れ: 設定を反映して終了 → 同じ `.base` の同じビューで開き直すと同じ設定。`.base` とノートのフォルダは変わらない。

#### 設定の画面は、よくある表の道具の設定の画面に倣い、左に区画の一覧(表示する列(表示・非表示と順)・フィルター・並べ替え(複数の列と向き)・グループ分け・表示(screen.display-options))を、それぞれの今の状態(例「列 5/6」「並べ替え 2」)を添えて並べるべき
`req~navigation.settings-layout-sections~1` P1
- 受け入れ: `o` → 左に「列 6/6」「フィルター —」「並べ替え —」「グループ —」「表示」。

#### 設定の画面の右には、左で選んだ区画の中身(何をする所かの一行の説明と項目)を置くべき
`req~navigation.settings-layout-pane~1` P1
- 受け入れ: `o` → 右に列の区画の説明と項目。Tab で並べ替えの区画 → 右がその中身に変わる。低い画面では区画の中を流し、選んだ項目がいつも見える(ゴールデン)。

#### 設定の画面では、`h`・← で左の区画の一覧に移り、そこで `j`・`k`・↑↓ で区画を選ぶと右がその区画の中身に替わり、`l`・→・Enter で右の項目へ移れるべき
`req~navigation.settings-sections-keys~1` P1
- なぜ: 今は区画を Tab と Shift+Tab で順に巡るだけで、離れた区画へ移るのに何度も押すうえ、今どの区画にいるかを左の一覧で選べないため(2026-11-20 に人が挙げた)
- 受け入れ: `o` で列の区画の項目にいる → `h` → 左の「列」が選ばれ、選びの印が左に移る。`j` を2回 → 左は「並べ替え」で、右が並べ替えの中身。`l` → 右の並べ替えの1つ目の項目。Tab と Shift+Tab はこれまでどおり区画を巡る。
- 確かめ: stest

#### 設定の画面の左の区画の一覧では、数字のキー(1〜9)でその番号の区画へ移り、区画の名前の頭に番号を添えるべき
`req~navigation.settings-sections-digits~1` P1
- なぜ: よく使う区画(フィルター・並べ替え)へ1打で移るため
- 受け入れ: `o` → 左の一覧が「1 列」「2 フィルター」「3 並べ替え」…。`3` → 並べ替えの区画が選ばれ、右がその中身。区画の数より大きい数字 → 何も変わらない。値の入力の欄にいるとき(navigation.settings-filter の値の入力)の数字 → 入力の文字になる。

#### 設定の画面で、左の区画の一覧と右の項目のどちらにキーが効くかを、色だけでなく選びの印でも見せるべき
`req~navigation.settings-sections-focus~1` P1
- なぜ: ↑↓ が区画を替えるのか項目を動かすのかを、色の見えにくい端末でも迷わないようにするため(cell-view.not-color-only)
- 受け入れ: 左の一覧にいる → 左の選んだ区画の頭に選びの印、右の項目には印が無い。`l` → 印が右の項目へ移る。

#### 設定の画面の「反映」と「取り消し」は、画面の上の右にいつも見せるべき
`req~navigation.settings-layout-buttons~1` P1
- 受け入れ: `o` → 画面の上の右に「反映」と「取り消し」。上の「反映」をクリック → 表に効く。

#### 設定の画面で反映していない変更があれば、画面の上にその数を添えるべき
`req~navigation.settings-layout-pending~1` P1
- 受け入れ: 設定の画面で並べ替えを足す → 上に「未反映 1」。

#### 設定の画面のオン・オフの項目はスイッチの形で見せ、色だけに頼らず文字でも状態が分かるようにするべき(cell-view.not-color-only)
`req~navigation.settings-layout-switch~1` P1
- 受け入れ: 色なしで `o` → 区画の印とスイッチの文字で分かる。

#### 設定のフィルターの条件は、値の一覧にチェックを付け外しする形(値ごとの件数を添え、空は「(空)」、リストの値は要素ごと)を基本にする
`req~navigation.filter-conditions-checklist~1` P1
- 受け入れ: tags の値の一覧で 会議 だけにチェック → 会議 を要素に持つ行だけ。

#### 設定のフィルターの条件は、テキストの「含む・含まない」、数と日付の比較(`=`・`≠`・`<`・`≤`・`>`・`≥`)、「空である・空でない」も選べてよい
`req~navigation.filter-conditions-more~1` P1
- 受け入れ: 日付の列の `< 2026-10-01` → それより前の行だけ、`someday` のような型の合わない値は残らない。

#### 設定のフィルターの複数の条件は、全部を満たす行だけを残すべき
`req~navigation.filter-conditions-all~1` P1
- 受け入れ: priority `≥ 3` と status の done を外す を並べる → 両方を満たす行だけ。

#### 設定のフィルターとグループ分けは、`.base` の filters・groupBy の結果の上に重ねるべき
`req~navigation.settings-layering-filter~1` P1
- 受け入れ: `.base` の filters で 10 行のビューに設定で done を外す → 10 行のうち done でない行だけ。

#### 設定の並べ替えは、設定があれば `.base` の sort に代わるべき
`req~navigation.settings-layering-sort~1` P1
- 受け入れ: `.base` の sort が title のビューで、設定の並べ替えに due を足して反映 → title の順でなく due の順に並ぶ。

#### 検索(navigation.search)・同じ値の絞り込み(navigation.same-value)は、設定の上にさらに重ねるべき
`req~navigation.settings-layering-search~1` P1
- 受け入れ: `.base` の filters で 10 行のビューに設定で done を外し、その上で `\` で「会議」→ さらに絞る。

#### 見出しからの並べ替え(navigation.header-sort)と並べ替えの窓(navigation.sort-window)は、設定の並べ替えそのものを変えるべき
`req~navigation.settings-layering-sort-ops~1` P1
- 受け入れ: 設定で並べた列と別の列を `s` で並べる → 設定の並べ替えがその列に置き換わり、`s` を3回で並べ替えが外れる(採択 2026-10-10 の sort-views の文どおり)。

#### 絞り込みの窓(navigation.filter-window)は、設定のフィルターそのものを変えるべき
`req~navigation.settings-layering-filter-ops~1` P1
- 受け入れ: 絞り込みの窓で status の done を外す → 設定の画面のフィルターの区画でも done が外れている。

#### 値を直した行の留め方(navigation.keep-edited-rows)は、設定のフィルターにも効くべき
`req~navigation.settings-layering-keep-edited~1` P1
- 受け入れ: 設定で done を外した表で done に直す → 直した行は保存か移動まで残る。

#### グループ分けは、空の値のまとまりを隠すことと、まとまりの並び(昇順・降順)を選べてよい
`req~navigation.group-options~1` P1
- 受け入れ: status でグループ分けし、空を隠す → `(空)` の見出しと行が出ない。

#### 設定を `.base` のビューの既定(`.base` なしなら何もしない状態)に戻す操作を持つべき
`req~navigation.settings-reset-default~1` P1
- 受け入れ: 設定を戻す → 帯が消え、`.base` のままの表。

#### 表の上の帯の項目から、1つの条件を外す操作を持つべき
`req~navigation.settings-reset-bar-item~1` P1
- 受け入れ: 帯の「status: done を除く」を外す → done の行が戻る。

#### 読むだけで開いたとき(cli.readonly)も、設定は変えられるべき
`req~navigation.settings-reset-readonly~1` P1
- 受け入れ: `--readonly` で設定を反映 → 表は変わる。

#### 読むだけで開いたときは、設定を変えても、ノート・`.base`・見た目の状態のどのファイルも書かないべき
`req~navigation.settings-reset-readonly-no-write~1` P1
- 受け入れ: `--readonly` で設定を反映 → ノート・`.base`・状態の置き場のどれも書かれない。

#### 選んでいる列の値の頻度表(値ごとの件数。リストの値は要素ごと、空は「(空)」)を、キー(既定 `%`。screen.key-table のキーの表で割り当て直せる)・パレット・操作の一覧(screen.action-menu の列の節)から、表の上に重ねる窓で開けるべき
`req~navigation.frequency-open~1` P1
- 受け入れ: tags の列で `%` → 表の上に頻度表の窓が開く。値の無い行は「(空)」として数える。

#### 頻度表は、件数の多い順(同じ数は値の順)に、件数と全行に対する割合を添えて並べるべき
`req~navigation.frequency-order~1` P1
- 受け入れ: tags の列で `%` → 窓に「会議 3 (25%)」「家 2 (17%)」…が多い順。

#### 頻度表で ↑↓ で選んで Enter を押すと、その値の行だけに絞るべき(navigation.same-value の同じ値の行だけの絞り込みと同じ絞り込みにし、リストの値はその要素を持つ行、空は値の無い行とする)
`req~navigation.frequency-enter~1` P1
- 受け入れ: tags の列の頻度表で「会議」で Enter → tags に会議を持つ行だけになり、`,` と同じくヘッダーに絞り込みが出る。

#### 頻度表は、Esc で何もせず閉じるべき
`req~navigation.frequency-esc~1` P1
- 受け入れ: tags の列で `%` のあと Esc → 何も変わらない。

#### 頻度表で数えるのは、今のビューの絞り込みのあとの行とするべき
`req~navigation.frequency-scope~1` P1
- 受け入れ: 設定で status の done を外したビューの tags の列で `%` → done の行の tags は数えない。

### 式の絞り込み

#### 表の画面から、`.base` には書かずに、base-view.expr-eval と同じ書き方の式(名前の条件 `cond("名前")` を含む。condition.expr-call)で絞り込みを足せるべき
`req~navigation.expr-filter~2` P1
- なぜ: 列の値を選ぶだけでは書けない条件(期限が7日以内で、まだ終わっていない、など)を、`.base` や設定を書き換えずにその場で試すため
- 受け入れ: パレットの「式で絞り込む」で `status != "done" && cond("soon")` → 当たる行だけになり、設定の帯にその式が出る。`.base` のバイトは同じ。
- 確かめ: stest

#### 画面で足した式が読めないか評価できないときは、絞り込みを変えずに、理由をメッセージ行に出し、打った式を残した入力の欄に戻るべき
`req~navigation.expr-filter-error~1` P1
- なぜ: 打ち損じで今の絞り込みを失わず、どこを直せばよいかを人に伝えるため
- 受け入れ: `due <=` → 行は変わらず、メッセージ行に読めない理由、入力の欄に `due <=` が残る。`cond("nosuch")` → condition.unknown-name と同じ理由。

#### 画面で足した式の絞り込みは、設定の帯の項目を外すと解け、数字のキーの条件(condition.digit-lifetime)と同じく、ほかの表へ移るか起動し直したら解くべき
`req~navigation.expr-filter-lifetime~1` P1
- なぜ: その場で試す絞り込みを、ほかの表に持ち越して黙って行を消さないため(ビューにずっと掛けるのは condition.view-settings)
- 受け入れ: 式で絞ったあと帯の項目を外す → 元の行。式で絞ったまま引き出しでほかの表へ移る → 絞っていない。

### ワークスペースを横断して探す

#### 全体の設定 `[search]` の `across` を `true` にしたときだけ、パレットに「ワークスペースを横断して探す」を出すべきで、既定は `false` にするべき
`req~navigation.across-option~1` P1
- なぜ: 範囲の表を全部読むのは表が多いと時間がかかり、要らない人の操作の候補も増えるため(2026-11-20 に人が「オプションで。基本はオフ」と決めた)
- 受け入れ: 設定に `[search]` が無い → パレットに「ワークスペースを横断して探す」が無い。`across = true` → ある。

#### 横断して探すときは、打った語を、今の範囲(workspace.scope-order-rank)の表の全部の行から、表の中の検索(navigation.search)と同じ当て方で探し、当たった行を表ごとにまとめて(表の名前・ノートの名前・当たった列と値を添えて)並べるべき
`req~navigation.across-results~1` P1
- なぜ: どの表に書いたか覚えていない行を、表を1つずつ開かずに見つけるため
- 受け入れ: tasks に「見積もり」を含む行が2つ、projects に1つ → 「見積もり」と打って Enter → 「tasks」の下に2行、「projects」の下に1行。どこにも当たらない → 「当たる行がありません」。
- 確かめ: stest

#### 横断の検索の結果で行を選ぶと、その表を開き、同じ語で検索した状態でその行を選ぶべき
`req~navigation.across-open~1` P1
- なぜ: 見つけた行をそのまま直せるようにするため
- 受け入れ: projects の「見積もりの返事」を Enter → projects の表が開き、検索の欄に「見積もり」、選んだ行が「見積もりの返事」。ためた変更があれば workspace.drawer-unsaved と同じく確かめる。

#### 横断の検索は、表を読み終えた分から結果を出し、読み終えるまで読んでいる表の数を見せ、Esc で止めて元の画面に戻れるべき
`req~navigation.across-progress~1` P1
- なぜ: 大きな表があっても待たされている理由が分かり、いつでもやめられるようにするため
- 受け入れ: 表が 5 つの範囲で探す → 読み終えた表の結果から並び、「3 / 5 を読みました」。途中で Esc → 元の表の画面のまま。読めない表 → その表を飛ばし、結果の終わりに読めなかった表の名前。

### 次の段(まだ作っていない。P2)

#### 絞り込みを積み重ね、1段ずつ戻せてよい
`req~navigation.filter-stack~1` P2
- 受け入れ: 2段絞ったあと戻る → 1段目の結果(読んで判定)。

## 未確定事項

