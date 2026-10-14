---
type: Spec
id: 01M3SBGVVWAZDTNPE7Z48W55P9
title: 移動・検索・選択・一時的な並べ替え(navigation)
description: 表の中を動く・探す・絞る・並べ替える・列を扱う・行を選ぶ操作。どれも .base は変えず、画面の中だけで効く。
status: active
load_when: 移動・検索・簡易の絞り込み・一時的な並べ替え・列の表示・行の選択・詳細の表示・ビューの設定を作る・変えるとき
created: 2026-09-30
updated: 2026-10-14
---

# 移動・検索・選択・一時的な並べ替え(navigation)

表の中の移動と、探す・絞る・並べ替える・列を隠す・行を選ぶ操作。`.base` の定義(base-view)は変えず、画面の中だけで効く(見た目の状態として残すものは screen の SR-11・SR-12、ビューとして残すものは base-view の mdgrid のビュー BV-17・BV-20)。csvlens・tabiew・VisiData・Obsidian Bases の操作から、表のノートに要るものを採った。

強さの読み方: 「しなければならない」= 例外なし、「するべき」= 理由があれば外してよい、「してよい」= 任意。

## 要件: 版1 — 移動と検索

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| NV-7 | 行番号での移動(`:<番号>`)、先頭・末尾(`gg` `G`)、列の先頭・末尾(`0` `$` と Home / End)、画面の半分・1画面の移動(Ctrl+D / Ctrl+U、PageDown / PageUp)があるべき。 | `:120` → 120行目が選ばれる(`test_NV_7`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-1 | `/` で表の文字を探し(画面の呼び名は「探す」。行を減らさないので、行を絞る検索(NV-2)とは呼び分ける)、一致をすべて強調して `n` / `N` で前後に移るべきで、探す語に大文字が無ければ大文字小文字を区別しない(smartcase)べき。 | `/Todo` は大文字小文字を区別、`/todo` は区別しない(`test_NV_1`) | [2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-2 | 打つたびに行を絞り込む検索があり、その入力は NV-23 の表の上の検索の欄(`\` かクリックで入る)があればそこで、欄を出さない設定なら最下行で行い、Esc で解除するべき。検索は `.base` の絞り込みに重ねて効くべき。画面では、行を減らさない `/`(NV-1。「探す」)と、列の条件で絞る絞り込み(NV-29)と呼び分け、「検索」と呼ぶべき。 | `\` で上の欄に入り「会議」と打つ → 「会議」を含む行だけ。Esc で元に戻る(`test_NV_2`) | [2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-10-02](../_decisions/2026-10-02-search-bar.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-23 | ビューの帯(SR-1)の、ビューのタブの右に、検索(NV-2)の欄を常に出し、欄の頭にキーと「検索」を示し(例「\ 検索」)、効いている語と、合う行の数(例「12/80行」)を見せるべき。欄の右には、ボタン「絞り込み」(NV-29)・「並べ替え」(NV-24)・「分類」・「列」をこの順に並べ、帯の右の端には、作れるときは「+ 新規」(CE-25)を置き、キーのあるボタンにはそのキーを添えるべき(例「S 並べ替え」。マウスで使う人もキーを覚えられるように)。ボタンの頭のキーの字は、名前と見分けられる見た目(太字で、名前と違う色。色を使わない表示では太字)にするべき(キーの字を名前の一部や数字と取り違えないように。ヘッダーのボタン(SR-42)も同じ)。「分類」と「列」は、ビューの設定の画面(NV-18)をその区画で開くべき。端末の幅が足りないときは、列・分類のボタン、今のでないビューのタブの順に省き、今のビューのタブと検索の欄と、絞り込み・並べ替え・「+ 新規」のボタンをなるべく残すべき。ビューの帯は、既定で上下に1行ずつ余白を取った3行の高さにし、タブとボタンを真ん中の行に並べるべき(表の上が詰まって見えないように)。上下の余白は設定 `display.bar_padding`(0〜2 を 0.5 刻み。既定 1。端数は端末とテーマによって半行にも1行にもなり、環境で見え方がぶれるので既定にしない)で変えられ、0 では余白を取らず1行にするべき。0.5 の端数は、帯の外側の1行を半行の余白に見せるべき(端末は行の単位でしか空けられないため)。その行は、帯の行と同じくセルの地の色で帯の色を塗り、帯の外側の半分を、画面の地の色の半分の塗りの字(上の余白は `▀`、下の余白は `▄`)で消して描くべき(字の色と地の色を端末が少し違えて描いても、帯の中に継ぎ目が出ないように)。画面の地の色は、テーマが画面の地を塗るときはその色、塗らないときは端末への問い合わせ(SR-39 と同じ OSC 11。短い待ち時間で、答えが無ければ諦める)で知った端末の地の色とするべき。端数があるときは帯の行に地の色を敷き、色を使わない表示・幅があいまいな字を全角として数える設定(半分の塗りの字が2桁になりうる)・画面の地の色が分からないときは、端数を1行に切り上げて空き行にするべき。囲み方は設定 `display.buttons` で選べ、`"plain"`(線で囲まず、欄とボタンを地の色で示す。既定)か `"box"`(検索の欄と、絞り込み・並べ替えのボタンを、それぞれ上下の余白の行に線を引いて四角く囲み、線の中は地の色で塗らない。余白が 0 なら囲まない)にできるべき。窓(NV-24・NV-29)は帯のすぐ下(`"box"` では囲みの下の線のすぐ下)に開くべき。端末の高さが16行に満たないときは、余白を 0 として描くべき(表の行を残すため)。欄は設定で出さないこともでき、出さないときは帯を1行にしてよい。 | 起動(80×24)→ ビューの帯は既定の余白 1 で、上下に1行ずつ空き、真ん中の行にタブ「既定の表」、その右に空の検索の欄「\ 検索…」と、キーを添えた絞り込み・並べ替えのボタン(例「S 並べ替え」)、「分類」「列」、右の端に「a + 新規」。設定の ⚙ は無い。「会議」で絞る → 欄に「会議」と「3/16行」。「分類」を押す → ビューの設定の画面の分類の区画。幅 60 → 列・分類が無く、絞り込みと並べ替えと「+ 新規」は出る。ボタンの頭のキーの字は太字で、名前と見た目が違う。「\| 絞り込み」を押す → 窓は帯のすぐ下に開く。設定 `display.buttons = "box"` → 欄と絞り込み・並べ替えが線で四角く囲まれ、線の中は地の色が無い。`display.bar_padding = 0` → ビューの帯は1行。`1` → 上下に1行ずつ空く。`2` → 上下に2行ずつ空く。`0.5` → 帯の上下の行が半分だけ帯の地の色で塗られ、帯は地の色の付いた帯に見える。`--no-color` で `0.5` → 上下に1行ずつ空く。高さ 14 → ビューの帯は1行。設定で欄を消す → 欄が無く、`\` を押せば最下行で打てる(`test_NV_23`) | [2026-10-12](../_decisions/2026-10-12-padding-default-one.md)、[2026-10-12](../_decisions/2026-10-12-half-row-inverse.md)、[2026-10-12](../_decisions/2026-10-12-bar-padding-half.md)、[2026-10-12](../_decisions/2026-10-12-bar-padding.md)、[2026-10-12](../_decisions/2026-10-12-view-bar-plain.md)、[2026-10-12](../_decisions/2026-10-12-search-box.md)、[2026-10-12](../_decisions/2026-10-12-header-menu.md)、[2026-10-11](../_decisions/2026-10-11-view-bar.md)、[2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) [2026-10-02](../_decisions/2026-10-02-search-bar.md) |
| NV-8 | 選んだセルと同じ値の行を強調する操作と、同じ値の行だけに絞る操作があるべき。 | status が done のセルで絞る → done の行だけ(`test_NV_8`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 版1 — 並べ替えと列

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| NV-3 | 列の見出しから並べ替え(昇順 → 降順 → 解除)ができるべきで、その並べ替えは今のビューの設定の並べ替え(NV-13)として当て、見た目の状態(SR-12)に保存して、次に開いても同じ並びにするべき(よくある表の道具と同じく、選んだ並びを覚えるため)。`.base` の並べ替えとファイルは変えるべきでない。 | 見出しを2回 → 降順、3回 → `.base` の並びに戻る。`.base` のバイトは変わらない(`test_NV_3`) | [2026-10-10](../_decisions/2026-10-10-sort-views.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-4 | 列を隠す・戻す、並びを変える、幅を変える(ドラッグと `<` `>`)、左の列(既定はファイル名)を固定する操作があるべき。 | 列を隠して終了し再起動 → 隠したまま(見た目の状態、`test_NV_4`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-24 | ビューの帯(NV-23)の「絞り込み」のボタンの右に「並べ替え」のボタンを出し、押すと、ボタンのすぐ下に、今のビューの並べ替えの決まり(列と向き。複数)を並べた小さな窓を開くべき。窓では、決まりを足す(列を選ぶ)・外す・向きを変える・順を入れ替えることができ、変えるとすぐ表に効かせ、設定の並べ替え(NV-13)として見た目の状態(SR-12)に保存するべき。並べ替えが効いている間は、ボタンに先頭の決まりを見せる(例「↑ due」、2つ以上なら「↑ due +1」)。窓の外のクリックか Esc で閉じる。検索の欄を出さない設定(NV-23)では、同じ窓を操作の一覧とコマンドのパレットから開けるべき。 | 検索の欄の右の「並べ替え」を押す → 窓が開き「+ 並べ替えを足す」。due を選ぶ → すぐ due の昇順になり、ボタンに「↑ due」。due の行で Enter → 降順「↓ due」。終了して開き直す → 同じ並び。窓で決まりを外す → 元の並び。読むだけの起動でも並べ替えられる(ノートは書かない)(`test_nv_24_*`) | [2026-10-11](../_decisions/2026-10-11-view-bar.md)、[2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-10-10](../_decisions/2026-10-10-sort-views.md) |
| NV-25 | ビューのタブのうち1つを「既定のビュー」に選べるべきで、操作の一覧とコマンドのパレットの「既定のビューにする」で今のビューを選び、mdgrid の設定の置き場の `views.toml` の、開いた対象の項目に名前で残すべき(ノートのフォルダと `.base` には書かない)。起動時は、ビューを名前で指定していなければ(`--view`・登録した表のビュー)、既定のビューを開くべき。既定のビューの名前が無くなっていたら、警告せずに先頭のビューを開くべき。読むだけの起動(`--readonly`)では選べないべき。 | ビューが3つの対象で「進行中」のタブを選び「既定のビューにする」→ views.toml の対象に default_view = "進行中"。終了して開き直す → 「進行中」が選ばれている。`--view 既定の表` で開く → 既定の表。「進行中」を消す → 次の起動は先頭。`--readonly` → 「既定のビューにする」が出ない(`test_nv_25_*`) | [2026-10-10](../_decisions/2026-10-10-sort-views.md) |
| NV-26 | ビューの設定の画面(NV-18)の左の一覧に「ビュー」の区画を持ち、開いた対象のビューのタブ(`.base` のビューと mdgrid のビュー)を今のタブの順に並べ、そこでタブの順・タブに出すか隠すか・既定のビュー(NV-25)・mdgrid のビューの名前の変更と削除(BV-18)・タブの行の切り替えの案内(「[ ] で切り替え」)を出すかを変えられるべき。この区画の変更はビューの一覧の管理で、今のビューの見せ方ではないので、反映を待たずにすぐ `views.toml` の開いた対象の項目に残すべき(ノートのフォルダと `.base` には書かない。BV-3)。隠したタブはタブの行にも `[` `]` の切り替えにも出さず、`--view` とこの区画からは開けるべき。今開いているビューは隠せず、すべてを隠すことはできないべき。`.base` のビューは名前の変更と削除をできないべき(`.base` を書き換えないため)。読むだけの起動(`--readonly`)では区画を見せるが変えられないべき(WB-15)。区画の印はタブに出している数と全部の数(例「3/4」)にするべき。 | ビューが3つ(既定の表・進行中・未完了)の対象で `o` → 左に「ビュー 3/3」。ビューの区画で「未完了」を隠す → すぐ表の上のタブが2つになり、`]` で未完了に移らない。「進行中」を K で上へ → タブが「進行中 既定の表」の順。「進行中」を既定にする → 「既定」の印が付き、views.toml に default_view = "進行中"。「進行中」の名前を「作業中」に変える → タブも変わる。切り替えの案内をオフ → タブの行に「[ ] で切り替え」が無い。終了して開き直す → 同じ順・同じ隠し方。今のビューを隠そうとする → 隠さず理由が出る。`.base` のビューで名前の変更 → 出来ない理由が出る。`--readonly` → 区画は見えて、変える操作で読むだけの理由。`test_nv_26_*` | [2026-10-10](../_decisions/2026-10-10-view-tabs.md) |
| NV-27 | ビューの設定の画面(NV-18)の左の一覧に「親子」の区画を持ち、そこでオンにしたビューでは、ノートのキー(既定 `parent`。区画で選べる)に書いた親のノートへのリンク(REL-1 と同じ書き方と解き方。リストなら先頭のリンク)で、子の行を親の行のすぐ下に並べ、深さ1つごとに2桁字下げしてノートの欄に見せるべき。兄弟の順は今の並べ替え(NV-3・NV-24・`.base` の sort)のままにするべき。子を持つ行の名前の前に ▾(開いている)か ▸(畳んでいる)を付け、`Z`(Shift+z。とその印のクリック)で子孫を畳む・開くべき。親が今の表に無い子(絞り込みで外れた・別のまとまり・解けないリンク)と、親子が輪になった行は、一番上の段に並べるべき(行を落とさないため)。グループ分け(NV-15)があるときは、まとまりの中で親子に並べるべき。オン・オフとキーはビューの設定として反映で効き、見た目の状態と mdgrid のビュー(BV-17)に残るべき。 | `parent: "[[設計]]"` の「画面」と「API」、`parent: "[[画面]]"` の「部品」がある表で親子をオン → 「設計」の下に「画面」(字下げ2桁)・その下に「部品」(4桁)・「API」(2桁)。title の降順で並べる → 兄弟の「画面」と「API」だけが入れ替わる。「設計」で `Z` → 子孫が隠れて「▸ 設計」。もう一度 `Z` → 戻る。「設計」を絞り込みで外す → 「画面」と「API」が一番上の段。A の親が B・B の親が A → 両方とも一番上の段。親子をオフ → 今までの並び。`test_nv_27_*` | [2026-10-10](../_decisions/2026-10-10-tree-key.md) [2026-10-10](../_decisions/2026-10-10-tree.md) |
| NV-28 | ビューの設定の画面の「親子」の区画で、親子で並べているビュー(NV-27)に WBS をオンにできるべき。オンのとき、行の名前の前に WBS の番号(まとまりの中で一番上の段を 1 から、子は親の番号に `.` と兄弟の中の順をつなぐ。例 `1.2.1`)を付けるべき。区画では進み具合のキー(既定 `status`)と、その値ごとの割合(0〜100)とラベルの対応表を作れるべきで、表の行に出てくる値を件数つきで並べ、値ごとに割合とラベルを打って決める。子の無い行は、名前のあとに対応表のラベルを、子のある行は、子孫の子の無い行の割合の平均を `%` で出すべき(対応表に無い値と値の無い行は 0% と数える)。WBS のオン・オフ、進み具合のキー、対応表はビューの設定として反映で効き、見た目の状態と mdgrid のビュー(BV-17)に残るべき。親子で並べていないときは WBS を出さないべき。 | 設計の下に画面(その下に部品 done・ボタン todo)と API(doing)がある表で、親子と WBS をオンにし、対応表で done = 100 完了・doing = 50 作業中・todo = 0 未着手 → 「1 設計 · 50%」「1.1 画面 · 50%」「1.1.1 部品 · 完了」「1.1.2 ボタン · 未着手」「1.2 API · 作業中」。API の status を空にする → 「1 設計」は 33%(100+0+0 の平均)。対応表に無い値 blocked → 0% と数え、ラベルは出さない。親子をオフ → 番号と進み具合が消える。開き直す → 対応表が残る。`test_nv_28_*` | [2026-10-10](../_decisions/2026-10-10-wbs.md) |
| NV-29 | ビューの帯(NV-23)の、検索の欄の右に「絞り込み」のボタンを出し、押すと、ボタンのすぐ下に、今のビューの設定のフィルターの条件(NV-14・NV-19)を1つ1行の表(列・比べ方・値・外す)で並べた小さな窓を開くべき(設定の画面(NV-13)を開いて反映する手間なしに、並べ替えの窓(NV-24)と同じ手軽さで絞るため)。窓では、条件を足す(列を選び、その列に合う条件を選ぶ: 値の一覧のチェック・含む・含まない・比較・空である・空でない)・外す・変えることができ、変えるときは条件の行の列・比べ方・値のどれかを選んで、その場で直せるべき。窓では、変えるとすぐ表に効かせ、設定のフィルター(NV-13)として見た目の状態(SR-12)に保存するべき。条件が効いている間は、ボタンに条件の数を見せる(例「絞り込み 2」)。窓の外のクリックか Esc で閉じる。検索の欄を出さない設定(NV-23)では、同じ窓を操作の一覧とコマンドのパレットから開けるべき。読むだけの起動(WB-15)でも使え、ノートは書かないべき。 | ビューの帯の「絞り込み」を押す → ボタンの下に窓が開き「+ 絞り込みを足す」。status を選び done のチェックを外す → すぐ done の行が消え、ボタンに「絞り込み 1」、上の帯に「status: done を除く」。条件の行の比べ方を選んで「含む」に変え、「ing」と打つ → その場で条件が変わる。終了して開き直す → 同じ。priority の `≥ 3` を足す → 両方を満たす行だけ。窓で条件を外す → 元の行。端末が狭くてボタンが入らない → ボタンを出さず、パレットから開ける。読むだけの起動でも絞れる(ノートは書かない)(`test_nv_29_*`) | [2026-10-11](../_decisions/2026-10-11-view-bar.md)、[2026-10-11](../_decisions/2026-10-11-filter-button.md) |
| NV-30 | 表のモードで、次の問題(CV-9)のセルへ跳ぶ操作(既定 `!`)と前の問題へ跳ぶ操作(既定 `@`。`!` の隣のキーで、`g` で始まる前置きのキーを増やさない(SR-16))があり(どちらも SR-4 のキーの表で割り当て直せ、パレットと操作の一覧からも使える)、表の行の順に(同じ行では列の順に)探して端で反対の端に回り、跳んだらメッセージ行にそのセルの問題の理由を出すべき。問題が無ければそう知らせて動かないべき。下の帯に、今の表(ビューの絞り込みのあとの行と、見えている列)の問題の数を、1つ以上あるときだけ出すべき(例「問題 3」)。問題のある行だけに絞る切り替えを、パレット・操作の一覧・下の帯の問題の数のクリックから使え、絞っている間はそれと分かるように出し、もう一度の切り替えで外せるべき。読むだけの起動(WB-15)でも使えるべき。 | 3つの問題のある表で `!` → 1つめの問題のセルに移り、メッセージ行に「due: 型が合わない(日付ではない)」。3回目の `!` のあと `!` → 1つめに戻る。`@` → 前へ。下の帯に「問題 3」。パレットの「問題のある行だけ」→ 問題のある行だけ、もう一度 → 元に戻る。問題の無い表 → 帯に数が出ず、`!` で「問題は無い」(`test_nv_30_*`) | [2026-10-14](../_decisions/2026-10-14-prev-problem-key.md)、[2026-10-14](../_decisions/2026-10-14-type-check.md) |

## 却下した案

| 案 | 理由 |
|---|---|
| タブの順と隠すことを `.base` の views の順に書く | `.base` を書き換えない(BV-3) |
| 区画の変更も「反映」でまとめて効かせる | タブの管理は今のビューの見せ方の下書きと別で、名前の変更や削除は今も下のボタンですぐ効く |

## 承認の記録

2026-10-10 人の承認(要約: タブの行も設定で変えられるべきという依頼。人の依頼を AI が要件の文に起こした | [2026-10-10](../_decisions/2026-10-10-view-tabs.md) |
| NV-12 | ためた値でも絞り込み・並べ替え・グループを計算し直すべきで、直した行が目の前から消えないように、値を直した行は、保存するか移動の操作をするまで元の位置に留めるべき。 | status で絞ったビューで done に直す → 行は残り、印が付く。保存で消える(`test_NV_12`) | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) |

## 要件: 版1 — 選択と詳細

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| NV-5 | 行に印を付ける(Space)、範囲で選ぶ(`v` と Shift+矢印)、全部を選ぶ、選択を解く、ができるべき。 | Space を3行 → 下の帯に「選択 3」(`test_NV_5`) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-6 | 選んだ行の全プロパティを縦に並べた詳細の表示(`K`)があり、そこからも編集できるべき。長い値は折り返して全文を見せるべき。プロパティの下には、ノートを開かずに中身を確かめられるよう、本文(フロントマターのあと)の先頭の最大 20 行を、制御文字を SR-9 のとおり出さずに、読むだけで見せるべき。 | 詳細の表示で長い説明の値 → 折り返して全文。本文のあるノートで `K` → プロパティの下に「本文」の見出しと先頭の行。本文の無いノートでは見出しを出さない(`test_NV_6`、`test_nv_6_body_*`) | [2026-10-06](../_decisions/2026-10-06-detail-body.md)、[2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 版1 — ビューの設定

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| NV-13 | 表の見せ方を変えるビューの設定の画面を持ち、そこでフィルター・並べ替え・グループ分け・表示する列を組んで、反映の操作で表に効かせるべきで、反映するまでは表を変えず、取り消せば元のままにするべき。 | 設定の画面(左の一覧のフィルターの区画)で status の done を隠す → 反映の前は表に done の行が残り、反映で消える。取り消しで閉じる → 表は開く前のまま(`test_NV_13`) | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) [2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-14 | 設定のフィルターは、todo の status に限らない用途で使うため、特定の列に決め打ちせず、どの列でも、その列の値を選んで、その値の行を隠すか、その値の行だけを出すかを決められるべき。 | status の done を外す → done の行が消える。種別の列の 本 だけを残す → 本 の行だけ。キーの無いノートは「(空)」として選べる(`test_NV_14`) | [2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-15 | 設定で、グループ分けをする列を選ぶことと、グループ分けをしないことを選べるべきで、`.base` の groupBy より設定を優先し、`.base` は書き換えないべき。 | groupBy のある `.base` のビューで「しない」を選んで反映 → 見出しの行が消える。種別の列を選ぶ → 種別ごとの見出し。`.base` のバイトは同じ(`test_NV_15`) | [2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-16 | 設定で効いているフィルター・並べ替え・グループ分けを、表の上の帯に並べて見せるべき。何も効いていなければ帯を出さないでよい。 | 設定の画面のフィルターの区画で status の done を外して、上の「反映」 → 表の上に「status: done を除く」の項目が出る(`test_NV_16`) | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) [2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-17 | 設定は、開いた対象とビューごとの見た目の状態(SR-11・SR-12)に覚え、次に同じものを開いたときに当て、`.base` とノートには書かないべき。 | 設定を反映して終了 → 同じ `.base` の同じビューで開き直すと同じ設定。`.base` とノートのフォルダは変わらない(`test_NV_17`) | [2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-18 | 設定の画面は、よくある表の道具の設定の画面に倣い、左に区画の一覧(表示する列(表示・非表示と順)・フィルター・並べ替え(複数の列と向き)・グループ分け・表示(SR-20))を、それぞれの今の状態(例「列 5/6」「並べ替え 2」)を添えて並べ、右に選んだ区画の中身(何をする所かの一行の説明と項目)を置くべき。「反映」と「取り消し」は画面の上の右にいつも見せ、反映していない変更があればその数を添えるべき。オン・オフの項目はスイッチの形で見せ、色だけに頼らず文字でも状態が分かるようにするべき(SR-15)。 | `o` → 左に「列 6/6」「フィルター —」「並べ替え —」「グループ —」「表示」、右に列の区画の説明と項目。Tab で並べ替えの区画 → 右がその中身に変わる。並べ替えを足す → 上に「未反映 1」。上の「反映」をクリック → 表に効く。色なしでも区画の印とスイッチの文字で分かる。低い画面では区画の中を流し、選んだ項目がいつも見える(ゴールデン `test_NV_18`) | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) [2026-10-10](../_decisions/2026-10-10-settings-layout.md)、[2026-10-01](../_decisions/2026-10-01-view-settings.md) |
| NV-19 | 設定のフィルターの条件は、値の一覧にチェックを付け外しする形(値ごとの件数を添え、空は「(空)」、リストの値は要素ごと)を基本にし、テキストの「含む・含まない」、数と日付の比較(`=`・`≠`・`<`・`≤`・`>`・`≥`)、「空である・空でない」も選べてよい。複数の条件は全部を満たす行だけを残すべき。 | tags の値の一覧で 会議 だけにチェック → 会議 を要素に持つ行だけ。priority `≥ 3` と status の done を外す を並べる → 両方を満たす行だけ。日付の列の `< 2026-10-01` → それより前の行だけ、`someday` のような型の合わない値は残らない(`test_NV_19`) | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) |
| NV-20 | 設定のフィルターとグループ分けは `.base` の filters・groupBy の結果の上に重ね、並べ替えは設定があれば `.base` の sort に代わり、検索(NV-2)・同じ値の絞り込み(NV-8)は設定の上にさらに重ねるべき。見出しからの並べ替え(NV-3)と並べ替えの窓(NV-24)は、設定の並べ替えそのものを変え、絞り込みの窓(NV-29)は、設定のフィルターそのものを変えるべき。値を直した行の留め方(NV-12)は設定のフィルターにも効くべき。 | `.base` の filters で 10 行のビューに設定で done を外す → 10 行のうち done でない行だけ。その上で `\` で「会議」→ さらに絞る。設定で並べた列と別の列を `s` で並べる → 設定の並べ替えがその列に置き換わり、`s` を3回で並べ替えが外れる(採択 2026-10-10 の sort-views の文どおり)。done を外した表で done に直した行は保存か移動まで残る(`test_NV_20`) | [2026-10-11](../_decisions/2026-10-11-filter-button.md)、[2026-10-10](../_decisions/2026-10-10-sort-views.md)、[2026-10-01](../_decisions/2026-10-01-view-settings-details.md) |
| NV-21 | グループ分けは、空の値のまとまりを隠すことと、まとまりの並び(昇順・降順)を選べてよい。 | status でグループ分けし、空を隠す → `(空)` の見出しと行が出ない(`test_NV_21`) | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) |
| NV-22 | 設定を `.base` のビューの既定(`.base` なしなら何もしない状態)に戻す操作と、表の上の帯の項目から1つの条件を外す操作を持つべき。読むだけで開いたとき(WB-15)も設定は変えられるべきで、ノート・`.base`・見た目の状態のどのファイルも書かないべき。 | 設定を戻す → 帯が消え、`.base` のままの表。帯の「status: done を除く」を外す → done の行が戻る。`--readonly` で設定を反映 → 表は変わり、ノート・`.base`・状態の置き場のどれも書かれない(`test_NV_22`) | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) |
| NV-9 | 選んでいる列の値の頻度表(値ごとの件数。リストの値は要素ごと、空は「(空)」)を、キー(既定 `%`。SR-4 のキーの表で割り当て直せる)・パレット・操作の一覧(SR-24 の列の節)から、表の上に重ねる窓で開けるべき。件数の多い順(同じ数は値の順)に、件数と全行に対する割合を添えて並べ、↑↓ で選んで Enter でその値の行だけに絞り(NV-8 の同じ値の行だけの絞り込みと同じ絞り込みにし、リストの値はその要素を持つ行、空は値の無い行とし)、Esc で何もせず閉じるべき。数えるのは今のビューの絞り込みのあとの行とするべき。 | tags の列で `%` → 窓に「会議 3 (25%)」「家 2 (17%)」…が多い順。「会議」で Enter → tags に会議を持つ行だけになり、`,` と同じくヘッダーに絞り込みが出る。Esc → 何も変わらない。値の無い行は「(空)」として数える(`test_NV_9`) | [2026-10-06](../_decisions/2026-10-06-frequency-filter-place.md)、[2026-10-06](../_decisions/2026-10-06-frequency-table.md)、[2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 要件: 次の段

| ID | 要件 | 例(確かめ方) | 決定 |
|---|---|---|---|
| NV-10 | `.base` には書かずに、`.base` の式と同じ書き方の絞り込みを、画面から足してよい。 | `status != "done"` を足す → 条件に合う行だけ(読んで判定) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |
| NV-11 | 絞り込みを積み重ね、1段ずつ戻せてよい。 | 2段絞ったあと戻る → 1段目の結果(読んで判定) | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) |

## 範囲外

- 採らない: ピボット・縦持ち化・転置・グラフ(VisiData にあるが、フロントマターの編集という目的から外れる)。
- 採らない: 表のシートを積むスタック(VisiData の型)。版1は `.base` と mdgrid のビュー(BV-17)の切り替えと詳細の表示で足りる。

## 前提(AI の仮定)

- キーの既定は、vim 風と Bases の主なキーの両方(screen の SR-13)。ここに書いたキーは既定で、設定で変えられる。
- 簡易の絞り込みの既定のキー `\` は VisiData の「選択を外す」とぶつかるが、mdgrid は選択を Space と v で持つので、ぶつからない(AI の判断)。

## 未決の問い(人の判断待ち。最大3)

- なし

<!-- decidespec:history:begin -->
<!-- この区画は check.py --write が書く。手で直さない。 -->

## この機能の決定

| 日付 | 記録 | 結果 | 決定者 | きっかけ | 触った要件 |
|---|---|---|---|---|---|
| 2026-10-14 | [型と決まりに合わない値を「問題」として探し、画面と CLI で確かめられるようにする(type-check)](../_decisions/2026-10-14-type-check.md) | accepted | 人 | 人の発言 | CE-36, CV-9, NV-30, CLI-22 |
| 2026-10-14 | [前の問題へ跳ぶ既定のキーを g ! から @ にする(prev-problem-key)](../_decisions/2026-10-14-prev-problem-key.md) | accepted | AI | 検証の指摘 | NV-30 |
| 2026-10-13 | [表示の項目が11になって変わる試験の錠を掛け直す(row-lines-locks)](../_decisions/2026-10-13-row-lines-locks.md) | accepted | 人 | AI の提案 | SR-20, SR-21, CLI-11 ほか 4 |
| 2026-10-12 | [ビューの帯は既定で線なし・上下に余白の3行にし、線で囲む形と1行の形は設定で選ぶ(view-bar-plain)](../_decisions/2026-10-12-view-bar-plain.md) | accepted | 人 | 人の発言 | SR-1, NV-23 |
| 2026-10-12 | [検索の欄と絞り込み・並べ替えのボタンを線で四角く囲み、ビューの帯を3行にする(search-box)](../_decisions/2026-10-12-search-box.md) | accepted | 人 | 人の発言 | SR-1, NV-23 |
| 2026-10-12 | [上の余白とビューの帯の余白の既定を 0.5 から 1 にする(padding-default-one)](../_decisions/2026-10-12-padding-default-one.md) | accepted | 人 | 人の発言 | NV-23, SR-1 |
| 2026-10-12 | [余白の既定を 1 にして変わる試験の錠を掛け直す(padding-default-one-locks)](../_decisions/2026-10-12-padding-default-one-locks.md) | accepted | 人 | AI の提案 | NV-23, SR-1, SR-26, CLI-11, SR-21 |
| 2026-10-12 | [設定を右上の別のボタンに、画面の型をメニューに、「+ 新規」をビューの帯に移し、ボタンのキーの字を目立たせる(header-menu)](../_decisions/2026-10-12-header-menu.md) | accepted | 人 | 人の発言 | SR-1, SR-42, NV-23, REL-8, CE-25 |
| 2026-10-12 | [右上のメニュー・3行のビューの帯で行の位置が変わる試験の錠を、まとめて掛け直す(header-menu-locks)](../_decisions/2026-10-12-header-menu-locks.md) | accepted | 人 | AI の提案 | BV-3, BV-7, BV-13 ほか 26 |
| 2026-10-12 | [余白の 0.5 刻み・半行の描き方で行の位置が変わる試験の錠を、まとめて掛け直す(half-row-locks)](../_decisions/2026-10-12-half-row-locks.md) | accepted | 人 | AI の提案 | BV-7, BV-13, BV-17 ほか 28 |
| 2026-10-12 | [半行の余白を、帯の色は地の色で塗り外側を画面の地の色の字で消す描き方にする(half-row-inverse)](../_decisions/2026-10-12-half-row-inverse.md) | accepted | 人 | 人の発言 | NV-23 |
| 2026-10-12 | [パンくずのボタンでヘッダーの字が変わる試験の錠を掛け直す(crumb-button-locks)](../_decisions/2026-10-12-crumb-button-locks.md) | accepted | 人 | AI の提案 | SR-1, BV-13, NV-15, SR-42, WS-6 |
| 2026-10-12 | [ビューの帯の余白を行の数の設定 bar_padding にし、囲み方の設定から1行の形を外す(bar-padding)](../_decisions/2026-10-12-bar-padding.md) | accepted | 人 | 人の発言 | NV-23 |
| 2026-10-12 | [ビューの帯とヘッダーの上の余白を 0.5 刻みにし、既定を 0.5 にして、端数は半分の塗りの字で半行に見せる(bar-padding-half)](../_decisions/2026-10-12-bar-padding-half.md) | accepted | 人 | 人の発言 | NV-23, SR-1 |
| 2026-10-11 | [表の上を、パンくずのヘッダーと、ビューの操作を1列に集めたビューの帯の2行にする(view-bar)](../_decisions/2026-10-11-view-bar.md) | accepted | 人 | 人の発言 | SR-1, SR-42, NV-23 ほか 6 |
| 2026-10-11 | [ビューの帯で表の行の位置と検索の欄の字が変わる試験の錠を掛け直す(view-bar-locks)](../_decisions/2026-10-11-view-bar-locks.md) | accepted | 人 | 人の発言 | BV-7, CE-16, NV-16 ほか 5 |
| 2026-10-11 | [2つの決定にまたがる要件を持つ試験の錠を、まとめて掛け直す(view-bar-locks-2)](../_decisions/2026-10-11-view-bar-locks-2.md) | accepted | 人 | 人の発言 | SR-20, SR-21, NV-2 ほか 6 |
| 2026-10-11 | [絞り込みを並べ替えと同じボタンと窓にし、上の欄の検索と呼び分ける(filter-button)](../_decisions/2026-10-11-filter-button.md) | accepted | 人 | 人の発言 | NV-29, NV-23, NV-24 ほか 6 |
| 2026-10-10 | [親子で並べた表に WBS の番号と進み具合を出し、値の対応表を設定で作って保存する(wbs)](../_decisions/2026-10-10-wbs.md) | accepted | 人 | 人の発言 | NV-28 |
| 2026-10-10 | [設定の画面に「ビュー」の区画を足し、タブの並び・出す隠す・既定・名前・削除をそこで変える(view-tabs)](../_decisions/2026-10-10-view-tabs.md) | accepted | 人 | 人の発言 | NV-26 |
| 2026-10-10 | [ビューの削除の確かめに合わせて、削除を通る試験の錠を掛け直す(view-delete-undo-locks)](../_decisions/2026-10-10-view-delete-undo-locks.md) | accepted | 人 | 人の発言 | BV-3, BV-13, BV-17 ほか 6 |
| 2026-10-10 | [親子のノートを字下げして並べ、畳めるようにする(tree)](../_decisions/2026-10-10-tree.md) | accepted | 人 | 人の発言 | NV-27 |
| 2026-10-10 | [親子の畳むキーを z から Z にする(tree-key)](../_decisions/2026-10-10-tree-key.md) | accepted | 人 | 検証の指摘 | NV-27 |
| 2026-10-10 | [並べ替えを選んで覚える(並べ替えの窓・見出しの並べ替えの保存・既定のビュー)(sort-views)](../_decisions/2026-10-10-sort-views.md) | accepted | 人 | 人の発言 | NV-3, NV-20, SR-12 ほか 4 |
| 2026-10-10 | [ビューの設定の画面を、左に区画の一覧・右に中身・上に反映の形にする(settings-layout)](../_decisions/2026-10-10-settings-layout.md) | accepted | 人 | 人の発言 | NV-18 |
| 2026-10-10 | [設定の画面の作り直しに合わせて、関わる要件の例(確かめ方)の操作を直す(settings-layout-examples)](../_decisions/2026-10-10-settings-layout-examples.md) | accepted | 人 | 人の発言 | NV-13, NV-16, NV-23 ほか 4 |
| 2026-10-10 | [公開リポで入った設定の形の作り直し(#30)が書き換えた試験の錠を掛け直す(config-v2-relock)](../_decisions/2026-10-10-config-v2-relock.md) | accepted | 人 | 人の発言 | BV-17, BV-19, BV-20 ほか 42 |
| 2026-10-10 | [設定の形の作り直しに合わせて、項目の名前と Config の形に触れる試験の錠を掛け直す(config-v2-locks)](../_decisions/2026-10-10-config-v2-locks.md) | accepted | 人 | 人の発言 | BV-17, BV-20, CE-16 ほか 34 |
| 2026-10-09 | [モダンな見た目を既定にしたのに合わせて、色の試験の比べる元を今までの見た目にして錠を掛け直す(modern-look-locks)](../_decisions/2026-10-09-modern-look-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-26, BV-19, NV-18 |
| 2026-10-07 | [左のノートの欄の名前に合わせて、画面の名前を見る試験の錠を掛け直す(note-column-locks)](../_decisions/2026-10-07-note-column-locks.md) | accepted | 人 | 人の発言 | SR-21, BV-17, CE-16 ほか 9 |
| 2026-10-07 | [group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)](../_decisions/2026-10-07-group-gap-locks.md) | accepted | 人 | 人の発言 | SR-20, SR-21, NV-16 ほか 4 |
| 2026-10-06 | [列の値の頻度表を版1に入れる(frequency-table)](../_decisions/2026-10-06-frequency-table.md) | accepted | AI | 検証の指摘 | NV-9 |
| 2026-10-06 | [NV-9 の頻度表で絞った条件の出る場所を NV-8 と同じにそろえる(frequency-filter-place)](../_decisions/2026-10-06-frequency-filter-place.md) | accepted | AI | 検証の指摘 | NV-9 |
| 2026-10-06 | [詳細の表示で本文の先頭を読むだけで見せる(detail-body)](../_decisions/2026-10-06-detail-body.md) | accepted | AI | 検証の指摘 | NV-6 |
| 2026-10-02 | [表の上に常に出る検索の欄(search-bar)](../_decisions/2026-10-02-search-bar.md) | accepted | 人 | 人の発言 | NV-2, NV-23 |
| 2026-10-01 | [ビューの設定の画面と、表の上のフィルターの帯(view-settings)](../_decisions/2026-10-01-view-settings.md) | accepted | 人 | 人の発言 | NV-13, NV-14, NV-15, NV-16, NV-17, NV-18 |
| 2026-10-01 | [ビューの設定の細部(フィルターの条件・重なり方・戻し方)(view-settings-details)](../_decisions/2026-10-01-view-settings-details.md) | accepted | AI | AI の提案 | NV-19, NV-20, NV-21, NV-22 |
| 2026-09-30 | [仕様の点検の指摘を直す(v1-review-fixes)](../_decisions/2026-09-30-v1-review-fixes.md) | accepted | AI | 検証の指摘 | SR-13, SR-10, SR-12 ほか 18 |
| 2026-09-30 | [最初の版の仕様を、表の TUI の調査に合わせて書き切る(v1-full-spec)](../_decisions/2026-09-30-v1-full-spec.md) | accepted | AI | 人の発言 | BV-4, BV-9, CE-1 ほか 52 |

## この機能のつながり

| 要件 | 最新の決定 | テスト | 最後の照合 |
|---|---|---|---|
| NV-7 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_help.rs::test_nv_7_goto_line, src/ui/test_keymap.rs::test_sr_13_default_keys, src/ui/test_nav.rs::test_nv_12_goto_line_after_release ほか 1 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| NV-1 | [2026-10-11](../_decisions/2026-10-11-filter-button.md) | src/ui/test_csv_usable.rs::test_sc_17_filter_and_sort, src/ui/test_filter_window.rs::test_nv_29_names_find_and_search, src/ui/test_filter_window_more.rs::test_nv_29_old_mode_names_keep_meaning ほか 5 | [2026-10-11-filter-button](../_changes/2026-10-11-filter-button.md) 2026-10-11 |
| NV-2 | [2026-10-11](../_decisions/2026-10-11-filter-button.md) | src/ui/test_filter_window.rs::test_nv_29_english_bar_lead_is_search, src/ui/test_filter_window.rs::test_nv_29_names_find_and_search, src/ui/test_filter_window_more.rs::test_nv_29_old_mode_names_keep_meaning ほか 5 | [2026-10-11-filter-button](../_changes/2026-10-11-filter-button.md) 2026-10-11 |
| NV-23 | [2026-10-12](../_decisions/2026-10-12-padding-default-one.md) | src/test_config_unit.rs::test_nv_23_config_search_bar, src/ui/test_bar_padding.rs::test_nv_23_bar_padding_two_rows_each, src/ui/test_bar_padding.rs::test_nv_23_bar_padding_zero_is_one_row ほか 54 | [2026-10-12-search-box](../_changes/2026-10-12-search-box.md) 2026-10-12 |
| NV-8 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_freq.rs::test_nv_9_status_enter_same_as_comma, src/ui/test_freq_more.rs::test_nv_8_comma_on_empty_cell_shows_empty_label, src/ui/test_freq_more.rs::test_nv_9_count_matches_rows_after_comma_on_other_column ほか 2 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| NV-3 | [2026-10-10](../_decisions/2026-10-10-sort-views.md) | src/ui/test_csv_usable.rs::test_sc_17_filter_and_sort, src/ui/test_nav.rs::test_nv_3_temporary_sort_keeps_base, src/ui/test_startup.rs::test_sr_12_temporary_sort_not_kept | [2026-10-10-sort-views](../_changes/2026-10-10-sort-views.md) 2026-10-10 |
| NV-4 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_native_state.rs::test_bv_18_save_as_keeps_moved_columns, src/ui/test_nav.rs::test_nv_4_freeze_refused_on_narrow_terminal, src/ui/test_nav.rs::test_nv_4_frozen_columns_keep_selected_visible ほか 4 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| NV-24 | [2026-10-11](../_decisions/2026-10-11-view-bar.md) | src/ui/test_filter_window.rs::test_nv_29_buttons_filter_then_sort_at_right_end, src/ui/test_search_box.rs::test_nv_23_box_window_below_bottom_line, src/ui/test_sort_more.rs::test_nv_24_hidden_bar_and_narrow ほか 7 | [2026-10-11-view-bar](../_changes/2026-10-11-view-bar.md) 2026-10-12 |
| NV-25 | [2026-10-10](../_decisions/2026-10-10-sort-views.md) | src/ui/test_default_view.rs::test_nv_25_default_view, src/ui/test_default_view.rs::test_nv_25_missing_and_readonly | [2026-10-10-sort-views](../_changes/2026-10-10-sort-views.md) 2026-10-10 |
| NV-26 | [2026-10-10](../_decisions/2026-10-10-view-tabs.md) | src/test_tab_prefs_unit.rs::test_nv_26_tab_prefs_broken_file_not_written, src/test_tab_prefs_unit.rs::test_nv_26_tab_prefs_default_removes_keys, src/test_tab_prefs_unit.rs::test_nv_26_tab_prefs_round_trip_keeps_views ほか 5 | [2026-10-10-view-tabs](../_changes/2026-10-10-view-tabs.md) 2026-10-10 |
| NV-27 | [2026-10-10](../_decisions/2026-10-10-tree-key.md) | src/test_tree_unit.rs::test_nv_27_order_empty_and_flat, src/test_tree_unit.rs::test_nv_27_order_missing_parent_and_cycles_go_top, src/test_tree_unit.rs::test_nv_27_order_preorder_keeps_sibling_order ほか 7 | [2026-10-10-tree](../_changes/2026-10-10-tree.md) 2026-10-10 |
| NV-28 | [2026-10-10](../_decisions/2026-10-10-wbs.md) | src/test_wbs_unit.rs::test_nv_28_numbers_and_progress, src/ui/test_wbs.rs::test_nv_28_numbers_progress_and_labels, src/ui/test_wbs.rs::test_nv_28_saved_with_native_view ほか 2 | [2026-10-10-wbs](../_changes/2026-10-10-wbs.md) 2026-10-10 |
| NV-29 | [2026-10-11](../_decisions/2026-10-11-view-bar.md) | src/ui/test_filter_window.rs::test_nv_29_bad_operand_is_refused_with_reason, src/ui/test_filter_window.rs::test_nv_29_buttons_filter_then_sort_at_right_end, src/ui/test_filter_window.rs::test_nv_29_click_opens_window_and_hide_done ほか 13 | [2026-10-11-view-bar](../_changes/2026-10-11-view-bar.md) 2026-10-12 |
| NV-30 | [2026-10-14](../_decisions/2026-10-14-prev-problem-key.md) | src/ui/test_problems.rs::test_nv_30_click_count_toggles, src/ui/test_problems.rs::test_nv_30_folded_group_not_counted_or_jumped, src/ui/test_problems.rs::test_nv_30_next_problem_wraps_with_reason ほか 6 | [2026-10-14-type-check](../_changes/2026-10-14-type-check.md) 2026-10-15 |
| NV-12 | [2026-09-30](../_decisions/2026-09-30-v1-review-fixes.md) | src/ui/test_csv_note_only.rs::test_sc_17_add_row_shown_with_pending_edits, src/ui/test_grid.rs::test_ce_11_tab_after_row_leaves_view_does_not_write_other_row, src/ui/test_nav.rs::test_nv_12_edited_row_stays_until_save ほか 5 | [2026-10-10-explore-fixes-2](../_changes/2026-10-10-explore-fixes-2.md) 2026-10-10 |
| NV-5 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | src/ui/test_action_menu_more.rs::test_sr_25_more_esc_after_prefix_keeps_marks, src/ui/test_keymap.rs::test_sr_13_default_keys, src/ui/test_nav.rs::test_nv_5_marks_range_all_and_esc ほか 1 | [2026-09-30-v1-full-spec](../_changes/2026-09-30-v1-full-spec.md) 2026-10-01 |
| NV-6 | [2026-10-06](../_decisions/2026-10-06-detail-body.md) | src/ui/test_checkbox.rs::test_ce_4_detail_view_cycles, src/ui/test_detail.rs::test_ce_1_click_inside_detail_input_keeps_editing, src/ui/test_detail.rs::test_nv_6_detail_wraps_long_value ほか 9 | [2026-10-06-detail-body](../_changes/2026-10-06-detail-body.md) 2026-10-06 |
| NV-13 | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) | src/ui/test_display_options.rs::test_sr_20_settings_screen_has_display_section, src/ui/test_display_options.rs::test_sr_20_toggle_in_settings_applies_to_that_view_only, src/ui/test_native_views.rs::test_bv_18_save_as_adds_tab ほか 3 | [2026-10-10-settings-layout](../_changes/2026-10-10-settings-layout.md) 2026-10-10 |
| NV-14 | [2026-10-01](../_decisions/2026-10-01-view-settings.md) | src/test_views_unit.rs::test_bv_19_keep_drop_match_settings, src/ui/test_settings_screen.rs::test_nv_14_filter_by_key_not_in_view, src/ui/test_settings_screen.rs::test_nv_14_keep_only_book_from_screen ほか 6 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-15 | [2026-10-01](../_decisions/2026-10-01-view-settings.md) | src/test_settings_unit.rs::test_nv_15_inherit_drops_emptied_groups, src/test_settings_unit.rs::test_nv_15_sort_list_value_in_non_list_column_skips_empty_elements, src/ui/test_csv_usable.rs::test_sc_17_saved_view_with_group ほか 11 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-16 | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) | src/ui/test_display_options.rs::test_sr_20_chips_hidden_but_f_works, src/ui/test_display_options.rs::test_sr_20_chips_hidden_from_settings_screen, src/ui/test_freq.rs::test_nv_9_down_enter_filters_to_second_value ほか 4 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-17 | [2026-10-01](../_decisions/2026-10-01-view-settings.md) | src/test_settings_unit.rs::test_nv_17_broken_settings_keep_other_state, src/test_settings_unit.rs::test_nv_17_keys_toml_form, src/ui/test_native_state.rs::test_bv_20_applied_settings_and_columns_survive_tabs_and_restart ほか 7 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-18 | [2026-10-10](../_decisions/2026-10-10-settings-layout-examples.md) | src/ui/test_display_more.rs::test_sr_20_display_section_scrolls_on_low_screen, src/ui/test_native_state.rs::test_nv_18_left_right_follow_button_rows, src/ui/test_row_lines.rs::test_cv_8_settings_cycles_zero_to_three ほか 5 | [2026-10-12-row-lines](../_changes/2026-10-12-row-lines.md) 2026-10-13 |
| NV-19 | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) | src/test_settings_unit.rs::test_nv_19_bad_operand_keeps_nothing, src/test_settings_unit.rs::test_nv_19_datetime_cmp_reads_time, src/test_settings_unit.rs::test_nv_19_ne_on_list_needs_every_element_unequal ほか 16 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-20 | [2026-10-11](../_decisions/2026-10-11-filter-button.md) | src/ui/test_settings_screen.rs::test_nv_20_edited_row_released_on_save, src/ui/test_settings_screen.rs::test_nv_20_edited_row_stays_until_move, src/ui/test_settings_screen.rs::test_nv_20_layers | [2026-10-11-filter-button](../_changes/2026-10-11-filter-button.md) 2026-10-11 |
| NV-21 | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) | src/ui/test_settings_screen.rs::test_nv_15_group_off_and_by_column, src/ui/test_settings_screen.rs::test_nv_21_hide_empty_group_from_screen, tests/test_settings.rs::test_nv_21_group_dir_desc_reverses_groups ほか 1 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-22 | [2026-10-01](../_decisions/2026-10-01-view-settings-details.md) | src/ui/test_display_more.rs::test_sr_20_hidden_chips_f_shows_selected_condition, src/ui/test_display_options.rs::test_sr_20_chips_hidden_but_f_works, src/ui/test_settings_screen.rs::test_nv_22_readonly_applies_but_writes_nothing ほか 2 | [2026-10-01-view-settings](../_changes/2026-10-01-view-settings.md) 2026-10-01 |
| NV-9 | [2026-10-06](../_decisions/2026-10-06-frequency-filter-place.md) | src/ui/test_audit4.rs::test_nv_9_small_percent, src/ui/test_audit4.rs::test_sr_3_load_keeps_top, src/ui/test_audit4.rs::test_wb_20_toml_frontmatter_is_read_only ほか 27 | [2026-10-06-review6-fixes](../_changes/2026-10-06-review6-fixes.md) 2026-10-06 |
| NV-10 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | なし | なし |
| NV-11 | [2026-09-30](../_decisions/2026-09-30-v1-full-spec.md) | なし | なし |
<!-- decidespec:history:end -->
