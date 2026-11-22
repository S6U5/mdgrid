# コピーと書き出し(output) の要求

## 目的

表の値を外へ持ち出す操作。版1はクリップボードへのコピーと、画面で選んだ行を標準出力に出す `--pick`(output.pick)で、ファイルへの書き出しと貼り付けは次の段(画面を出さない表の出力は cli の cli.print)。クリップボードは、ssh 越しや Wayland でも届くように、端末の仕組み(OSC 52)と OS のクリップボードの両方に送る(xleak・harlequin の教訓)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/output/spec.md](../_history/decidespec/output/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 採らない: 表計算の形式(xlsx など)への書き出し。

## 前提(旧い仕様で AI が置いた仮定)

- コピーの既定のキー(Ctrl+C と `y`)は Bases と vim の両方の流儀に合わせた(screen の screen.keys-vim-and-arrows)。

## 要求

### コピー

#### 選んだセル・行・選んだ複数の行を、タブ区切りでクリップボードにコピーできるべき(Ctrl+C と `y`)
`req~output.copy-tsv~1` P1
- 受け入れ: 2行を選んで `y` → 見出しつきのタブ区切り2行が OSC 52 で出る。

#### クリップボードへのコピーは、OSC 52 と OS のクリップボードの両方に送るべき
`req~output.copy-both-targets~1` P1
- 受け入れ: セルを1つ選んで `y` → 同じ文字が OSC 52 と OS のクリップボードの両方に送られる。

#### クリップボードへのコピーは、OSC 52 と OS のクリップボードの片方が失敗しても、止まらずに知らせるべき
`req~output.copy-one-side-fails~1` P1
- 受け入れ: OS のクリップボードへの送りが失敗する環境で `y` → 止まらずに OSC 52 は出し、失敗を知らせる。

#### 表のモードで、今画面に出している表(ビューの設定・検索・一時的な並べ替え・隠した列・列の並びを当てたあとの行と列)を、ファイルに書き出せるべき
`req~output.export-shown~1` P1
- 受け入れ: status で絞った3行の表で、パレットの「表を書き出す」から `out.csv` → 見出しの行と3行の CSV。隠した列は出ない。

#### 画面の表の書き出しの形は、ファイル名の拡張子(`.csv`・`.tsv`・`.json`・`.md`)で選ぶべき
`req~output.export-format~1` P1
- 受け入れ: `out.tsv` → タブ区切り。`out.txt` → 書かずに理由。

#### 画面の表の書き出しの値の書き方は、`--print` の同じ形(cli.print)と同じにするべき
`req~output.export-print-values~1` P1
- 受け入れ: 既定の表を `out.json` に書き出す → 各列の値の書き方が `--print --format json` の出力と同じ。

#### 画面の表の書き出しでは、ノートを書き換えないべき
`req~output.export-no-note-write~1` P1
- 受け入れ: パレットの「表を書き出す」から `out.csv` → 表のノートのファイルは変わらない。

#### 画面の表の書き出し(output.export)は、先頭の列に各行のノートのパス(`path`。`--print --with-path` と同じ)を入れるべき
`req~output.export-path-column~1` P1
- 受け入れ: 書き出した CSV の1列目が `path`。

#### 画面の表の書き出し(output.export)で書いたファイルは、`--apply` で戻せるようにするべき
`req~output.export-path-apply~1` P1
- 受け入れ: 書き出した CSV を `--apply` に渡すと差分なし。

#### 画面の表の書き出しのファイル名は、起動したフォルダからの相対か絶対のパスとし、先頭の `~` はホームのフォルダにするべき
`req~output.export-path-name~1` P1
- 受け入れ: `~/out.csv` → ホームの下に書く。`out.csv` → 起動したフォルダの下に書く。

#### 画面の表の書き出しで、同じ名前のファイルが既にあれば、`y` で確かめてから上書きするべき
`req~output.export-path-overwrite~1` P0
- 受け入れ: 既にある `out.csv` → 確かめの問い、`n` で書かない。

#### 画面の表を書き出したら、書いた行の数とパスを知らせるべき
`req~output.export-path-report~1` P1
- 受け入れ: 3行の表を `out.csv` に書き出す → 3行と `out.csv` のパスを知らせる。

#### 読むだけの起動でも、画面の表を書き出せてよい(ノートは書かない)
`req~output.export-path-readonly~1` P1
- 受け入れ: `--readonly` の起動で、パレットの「表を書き出す」から `out.csv` → 書き出せれば適合で、ノートのファイルは変わらない。

### 標準出力

#### `mdgrid [<.base | フォルダ> …] --pick path|<列の名前>` は、画面を端末(`/dev/tty`)に出し、読むだけ(cli.readonly と同じ)で開くべき
`req~output.pick-open~1` P1
- 受け入れ: `mdgrid notes --pick path` で開き、セルを空にして Ctrl+S → ノートは書かれない。

#### `--pick path` は、Enter で、印を付けた行(印が無ければ選んでいる行)ごとにノートのパスを1行ずつ標準出力に出し、終了コード 0 で終わるべき(パスは起動の引数のフォルダにノートの相対のパスをつないだもの)
`req~output.pick-path~1` P1
- 受け入れ: `mdgrid notes --pick path` で2行に印を付けて Enter → `notes/a.md` と `notes/b.md` の2行が出て終わる。印なしで Enter → 選んでいる行の1行。

#### `--pick <列の名前>` は、Enter で、印を付けた行(印が無ければ選んでいる行)ごとにその列の値の素の文字を1行ずつ標準出力に出し、終了コード 0 で終わるべき(画面の印を付けず、null と空は空、リストは `, ` でつなぎ、日付は `YYYY-MM-DD`、改行は空白。cli.print の csv と同じ決まり)
`req~output.pick-column~1` P1
- 受け入れ: `--pick status` → 値が1行ずつ。

#### `--pick` は、`q` と、解くもの(印・範囲・絞り込み・検索)が無いときの Esc で取りやめたら、何も出さずに終了コード 1 で終わるべき
`req~output.pick-cancel~1` P1
- 受け入れ: `q` → 何も出ず終了コード 1。

#### `--pick` は、標準出力がパイプでも動くべき(cell-view.no-color)
`req~output.pick-pipe~1` P1
- 受け入れ: `mdgrid notes --pick path | cat` で2行に印を付けて Enter → 標準出力がファイルのときと同じ2行が出る。

#### `--pick` に表に無い列の名前を渡したら、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~output.pick-unknown-column~1` P1
- 受け入れ: `--pick 無い列` → 理由1行と終了コード 2。

### 次の段(まだ作っていない。P2)

#### タブと改行で区切られた値を、ためる変更として、選んだセルから下へ複数の行に貼れてよい
`req~output.paste~1` P2
- 受け入れ: 3行の値を貼る → 3つのためる変更(読んで判定)。
