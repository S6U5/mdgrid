# セルと文字の見せ方(cell-view) の要求

## 目的

表のセルの値をどう見せるか。表の TUI への不満の中心の1つは「日本語や端末の違いで崩れる」なので、幅と色の決まりをここに集める。値の種類(空・null・型と合わない値・改行)を見分けられることは、書き戻しの前に利用者が値を確かめるための芯になる。画面の構成は [screen](screen.md)、編集は [cell-edit](cell-edit.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/cell-view/spec.md](../_history/decidespec/cell-view/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- テーマ(色の組み合わせ)は screen の screen.themes・screen.theme-names。端末のダーク / ライトの自動判定は screen の範囲外を参照。
- 採らない: 表の中での画像の表示。
- 旧い CV-7(新しい名前なし)は 2026-10-11 に削除([記録](../_history/decidespec/_decisions/2026-10-11-emoji-vs16-revert.md))

## 前提(旧い仕様で AI が置いた仮定)

- 空・null・キーなしの見せ方は harlequin(`∅ null`)、型と合わない値の扱いは Obsidian(不正と表示して隠さない)と bases.nvim(型が混ざる列の並べ替え)から採った(2026-09-30 の調査)。
- 列の幅の決め方は csvlens(見えている行で測り、1列の上限は画面の幅の3割)から採った。

## 要求

### 値の見せ方

#### null は薄い `∅`、空の文字列は `""`、キーが無いときは空欄にして、null・空の文字列・キーが無い、の3つを見分けられる表示にするべき
`req~cell-view.null-empty-missing~1` P1
- 受け入れ: 3つのノートの表 → 3通りの見え方(ゴールデン)。

#### 列の型と合わない値(date の列の `someday` など)は、値をそのまま見せて `!` の印を付け、並べ替えでは型の合う値の後ろに置き、編集では列の型の入力を出すべきで、合わない値のまま新しく書かせるべきでない
`req~cell-view.type-mismatch~1` P1
- 受け入れ: date の列の someday → `!` つきで表示、昇順でも降順でも末尾。

#### 改行を含む値は、1行目の後ろに `…⏎` を付けて表に出し(行を伸ばす設定(cell-view.row-lines)で伸ばせる行があるときは、改行で行を分けて見せ、入らない分だけ最後の行の後ろに `…` を付ける)、全文は詳細の表示で見せるべき
`req~cell-view.multiline~1` P1
- 受け入れ: 2行の説明 → 1行目と `…⏎`(ゴールデン)。

#### 数の列は右寄せ、チェックボックスの列は中央寄せで見せるべき
`req~cell-view.align~1` P1
- 受け入れ: 数の列のゴールデンで右寄せ。

### 幅と色

#### 文字の幅は書記素のまとまりで数え、どの行も画面の幅を超えるべきでない
`req~cell-view.grapheme-width-fit~1` P1
- 受け入れ: 全角・絵文字・結合文字・エスケープ文字を含む値の表 → 全行が画面の幅ちょうど以下(不変条件のテスト)。

#### 値の中の制御文字は、そのまま端末に出すべきでない
`req~cell-view.grapheme-width-control-chars~1` P1
- 受け入れ: 値に ESC を含むセルの表 → 端末への出力に、その値の ESC がそのまま出ない。

#### `NO_COLOR` があるときは、色を出すべきでない
`req~cell-view.no-color-env~1` P1
- 受け入れ: `NO_COLOR=1` で起動 → エスケープの色指定が無い。

#### `TERM=dumb` では、色と絵文字を出すべきでない
`req~cell-view.no-color-term-dumb~1` P1
- 受け入れ: `TERM=dumb` で起動 → エスケープの色指定が無く、値の中の絵文字も出ない。

#### 標準の入出力が端末でないとき(パイプ)は、画面を出さずに理由を1行出して終わるべき
`req~cell-view.no-color-not-tty~1` P1
- 受け入れ: 標準出力をパイプにして起動 → 画面を出さず理由1行で終わる。

#### 画面を出さない起動(`--print`、cli.print)と、画面は端末(`/dev/tty`)に出して結果だけを標準出力に出す起動(`--pick`、output.pick)は、標準出力がパイプでも動くべき
`req~cell-view.no-color-pipe-print-pick~1` P1
- 受け入れ: 標準出力をパイプにして `--print` で起動 → 理由で止まらずに表を出す。標準出力をパイプにして `--pick path` で起動 → 画面を `/dev/tty` に出して開く。

#### トゥルーカラーに対応しない端末では 256 色に落とすべきで、ためる変更・読むだけ・選択は、色のほかに印か文字でも示し、色だけで意味を伝えるべきでない
`req~cell-view.not-color-only~1` P1
- 受け入れ: `COLORTERM` の無い端末 → 256 色で描き、ためる変更のセルに `*` の印。

#### 列の幅の既定は、見えている行の値の幅から決め、1列の上限を画面の幅の3割にするべき
`req~cell-view.column-width~1` P1
- 受け入れ: 長い値の列 → 幅は画面の3割で `…` で切れる。

#### East Asian Ambiguous の文字(○● など)の幅は既定で1とし、設定で2にできるべき
`req~cell-view.ambiguous-width~1` P1
- 受け入れ: 設定で2にすると ○ の列がずれない。

#### 表の行は、設定 `display.row_extra_lines`(0〜9 の整数。既定 0)で、長い値と改行のある値を折り返し、データの1行を最大で「1 + その数」の行の高さまで伸ばせるべき
`req~cell-view.row-lines-extend~1` P1
- なぜ: 列の幅を広げなくても、長い説明を詳細の表示を開かずに表で読めるようにするため
- 受け入れ: `row_extra_lines = 2`、列の幅 20 に 50 字の説明 → その行は3行の高さで、説明が全部見える。

#### 伸ばす行の高さは、その行のセルのうち一番多くの行が要るものに合わせ、行ごとに変わってよい(短い値だけの行は1行のまま)
`req~cell-view.row-lines-height~1` P1
- 受け入れ: `row_extra_lines = 2`、列の幅 20 に 50 字の説明と短い status のある行 → 説明に合わせて3行の高さ。短い値だけの行 → 1行。

#### 伸ばす行の折り返しは、列の幅で、字の幅(cell-view.ambiguous-width)に従って切るべき
`req~cell-view.row-lines-wrap-width~1` P1
- 受け入れ: `row_extra_lines = 2`、列の幅 20 に全角 15 字の説明 → 各行の字の幅が 20 以下で折り返し、2行の高さ。

#### 行を伸ばすとき、改行のある値は改行で行を分けるべき
`req~cell-view.row-lines-wrap-newline~1` P1
- 受け入れ: `row_extra_lines = 2` で2行の説明(改行あり) → 2行の高さで1行目と2行目。

#### 伸ばした行に入りきらない値は、最後の行の後ろに `…` を付けるべき
`req~cell-view.row-lines-ellipsis~1` P1
- 受け入れ: `row_extra_lines = 2`、列の幅 20 に 61 字の説明 → 3行目の終わりが `…`。

#### 行を伸ばしても、リストの値の札・まとまりの見出しの行・まとまりの間の空き行は1行のままにしてよい
`req~cell-view.row-lines-single-parts~1` P1
- 受け入れ: `row_extra_lines = 2` で、列の幅より長いリストの値の札だけがある行と、まとまりの見出しの行 → 1行のままでも適合。

#### 伸ばした行の2行目より下の行には、行の印(`>` など)と行番号を出さないべき
`req~cell-view.row-lines-continuation~1` P1
- 受け入れ: `row_extra_lines = 2`、行番号を出す設定で、選んでいる3行の高さの行 → 2行目と3行目に `>` と行番号が無い。

#### 伸ばした行では、列の区切りと、一行おきの色・選んでいる行の色を、行の高さ全部に付けるべき
`req~cell-view.row-lines-full-height~1` P1
- 受け入れ: `row_extra_lines = 2` で3行の高さの行 → 列の区切り線と一行おきの色が3行全部に付き、その行を選ぶと選んでいる行の色も3行全部に付く。

#### 行を伸ばしているとき、縦のスクロール・ページ送り・マウスのクリックは、伸ばした行の高さで数えるべき
`req~cell-view.row-lines-count-height~1` P1
- 受け入れ: 伸ばした行の2行目をクリック → その行を選ぶ。

#### 行を伸ばしているとき、選んでいる行は高さ全部が見えるように表を流すべき(画面より高い行は上から入る分だけ見せる)
`req~cell-view.row-lines-keep-visible~1` P1
- 受け入れ: 一番下の行を選ぶ → その行の高さ全部が見えるまで表が流れる。

#### 伸ばした行の値の編集の欄と候補の窓は、その行の1行目に出すべき
`req~cell-view.row-lines-edit-first-line~1` P1
- 受け入れ: `row_extra_lines = 2` で3行の高さの行のセルを編集する → 入力の欄と候補の窓がその行の1行目に出る。

#### `row_extra_lines` が 0 では、今の1行の見え方と1字も変わらないべき
`req~cell-view.row-lines-zero~1` P1
- 受け入れ: `row_extra_lines = 0` → 今と同じ(ゴールデン `test_CV_3` が変わらない)。

#### ビューの設定の「表示」の区画(navigation.settings-layout)で、`row_extra_lines` を 0・1・2・3 と順に切り替えられるべき
`req~cell-view.row-lines-settings-toggle~1` P1
- 受け入れ: ビューの設定の「表示」の区画で行を伸ばす項を 0 から切り替える → 1・2・3 の順に変わる。

#### `row_extra_lines` は、ビューごとに上書きできるべき(screen.display-options と同じ)
`req~cell-view.row-lines-view-override~1` P1
- 受け入れ: 設定の `row_extra_lines` が 0 で、あるビューの上書きが 2 → そのビューの行は伸びる。

#### `row_extra_lines` の範囲の外の値は、警告して既定にするべき
`req~cell-view.row-lines-out-of-range~1` P1
- 受け入れ: `row_extra_lines = 12` → 警告して 0。

#### 決まり(cell-edit.rules)に合わないセルにも、型と合わない値(cell-view.type-mismatch)と同じ `!` の印を付けるべき(必須のキーが無いノートの空のセルにも付ける。印は色だけに頼らない。cell-view.not-color-only)
`req~cell-view.problems-mark~1` P1
- 受け入れ: `[rules.status] values = ["todo", "done"]` → `wip` のセルに `!`。`required = true` で status の無いノートの空欄に `!`。date の列の someday は今までどおり `!`。

#### 型と合わない値と決まりに合わない値を、画面と CLI でまとめて「問題」と呼び、理由を「型が合わない(日付ではない など)」「必須のキーが無い」「選べる値ではない」の3つで言い分けるべき
`req~cell-view.problems-reasons~1` P1
- 受け入れ: date の列の someday → 問題の理由は「型が合わない(日付ではない)」。`required = true` で status の無いノート → 「必須のキーが無い」。`[rules.status] values = ["todo", "done"]` の `wip` → 「選べる値ではない」。
