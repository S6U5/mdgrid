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

#### Ctrl+C と `y` は、選んだセルの値だけを、行の選択(navigation.select-rows)があれば今の列の選んだ行の値を1行に1つずつ、クリップボードにコピーし、`Y` は行(選択があれば選んだ行)を見出しつきのタブ区切りでコピーするべき
`req~output.copy-tsv~2` P1
- Rationale: コピーしたセルを、ほかのセルやほかの道具にそのまま貼れるようにするため(2026-11-24 に人が「セルの列を普通にコピペしたい」と決めた。前は、セル1つでも `列の名前<TAB>値` の形で、行を選んだ `y` も行の全部だった)。列の名前や行の全部が要るときは `Y`
- Acceptance:
  - Given: status が「進行中」のセルを選んでいる(行の選択なし)
  - When: `y`
  - Then: クリップボードに `進行中` だけが出る(列の名前は付かない)。Ctrl+C でも同じ。
- Acceptance:
  - Given: title が「会議のメモ」と「買い物」の2行を選び、title の列にいる
  - When: `y`
  - Then: クリップボードに `会議のメモ` と `買い物` の2行(見出しの行もノートの名前も付かない)。
- Acceptance:
  - Given: 同じ2行を選んでいる
  - When: `Y`
  - Then: 見出しの行と2行の、タブ区切りの3行が出る。
- Needs: utest

#### 表の画面で端末の貼り付け(Ctrl+V など。端末が送る貼り付けの文字)を受けたら、改行で行、タブで列に分けた値を、選んだセルを左上にして下の行と右の列へ、ためる変更(write-back.pending)として1手で貼るべき
`req~output.paste~2` P1
- Rationale: コピーしたセルの列(output.copy-tsv)や、表計算の道具から写した値を、1つずつ打ち直さずに入れるため(2026-11-24 に人が決めた)。ためる変更なので、保存の前に差分で確かめられ、1回で取り消せる
- Acceptance:
  - Given: 3行の表で、1行目の status のセルを選んでいる
  - When: `done`・`doing`・`todo` の3行を貼る
  - Then: 上から3行の status が done・doing・todo のためる変更になり、取り消し1回で3つとも戻る。
- Acceptance:
  - Given: 列が title・status の順の表で、1行目の title のセルを選んでいる
  - When: `A<TAB>x` と `B<TAB>y` の2行を貼る
  - Then: 1行目は title = A・status = x、2行目は title = B・status = y。
- Acceptance:
  - Given: 2行の表で、2行目の status のセルを選んでいる
  - When: 3行の値を貼る
  - Then: 2行目だけに1行目の値が入り、表に入らなかった値が2つあると知らせる。
- Acceptance:
  - Given: priority が数の列の表で、1行目の priority のセルを選んでいる
  - When: `5` と `abc` の2行を貼る
  - Then: 1行目は 5 になり、2行目は変わらず、数として読めない理由が出る。
- Acceptance:
  - Given: まとまりの見出しの行をはさんで行が並ぶ、グループ分けした表
  - When: 見出しの上の行から2行の値を貼る
  - Then: 見出しの行は飛ばし、見出しの下の最初の行に2行目の値が入る。
- Acceptance:
  - Given: `--readonly` で開いた表
  - When: 値を貼る
  - Then: 何もためず、読むだけの理由が出る。
- Acceptance:
  - Given: セルの入力の欄を開いている
  - When: `abc` と `def` の2行を貼る
  - Then: 入力の欄に `abc` が入り(1行目だけ)、ほかのセルは変わらない。
- Needs: utest・stest

#### クリップボードへのコピーは、OSC 52 と OS のクリップボードの両方に送るべき
`req~output.copy-both-targets~1` P1
- Acceptance:
  - Given:
  - When: セルを1つ選んで `y`
  - Then: 同じ文字が OSC 52 と OS のクリップボードの両方に送られる。

#### クリップボードへのコピーは、OSC 52 と OS のクリップボードの片方が失敗しても、止まらずに知らせるべき
`req~output.copy-one-side-fails~1` P1
- Acceptance:
  - Given:
  - When: OS のクリップボードへの送りが失敗する環境で `y`
  - Then: 止まらずに OSC 52 は出し、失敗を知らせる。

#### 表のモードで、今画面に出している表(ビューの設定・検索・一時的な並べ替え・隠した列・列の並びを当てたあとの行と列)を、ファイルに書き出せるべき
`req~output.export-shown~1` P1
- Acceptance:
  - Given:
  - When: status で絞った3行の表で、パレットの「表を書き出す」から `out.csv`
  - Then: 見出しの行と3行の CSV。隠した列は出ない。

#### 画面の表の書き出しの形は、ファイル名の拡張子(`.csv`・`.tsv`・`.json`・`.md`)で選ぶべき
`req~output.export-format~2` P1
- Acceptance:
  - Given:
  - When: `out.tsv`
  - Then: タブ区切り。
- Acceptance:
  - Given:
  - When: `out.txt`
  - Then: 書かずに理由。

#### 画面の表の書き出しの値の書き方は、`--print` の同じ形(cli.print)と同じにするべき
`req~output.export-print-values~1` P1
- Acceptance:
  - Given:
  - When: 既定の表を `out.json` に書き出す
  - Then: 各列の値の書き方が `--print --format json` の出力と同じ。

#### 画面の表の書き出しでは、ノートを書き換えないべき
`req~output.export-no-note-write~1` P1
- Acceptance:
  - Given:
  - When: パレットの「表を書き出す」から `out.csv`
  - Then: 表のノートのファイルは変わらない。

#### 画面の表の書き出し(output.export)は、先頭の列に各行のノートのパス(`path`。`--print --with-path` と同じ)を入れるべき
`req~output.export-path-column~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 書き出した CSV の1列目が `path`。

#### 画面の表の書き出し(output.export)で書いたファイルは、`--apply` で戻せるようにするべき
`req~output.export-path-apply~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 書き出した CSV を `--apply` に渡すと差分なし。

#### 画面の表の書き出しのファイル名は、起動したフォルダからの相対か絶対のパスとし、先頭の `~` はホームのフォルダにするべき
`req~output.export-path-name~2` P1
- Acceptance:
  - Given: `~/out.csv`
  - When:
  - Then: ホームの下に書く。
- Acceptance:
  - Given:
  - When: `out.csv`
  - Then: 起動したフォルダの下に書く。

#### 画面の表の書き出しで、同じ名前のファイルが既にあれば、`y` で確かめてから上書きするべき
`req~output.export-path-overwrite~2` P0
- Rationale: 既にあるファイル(人や自分が作ったもの)を、書き出しで黙って置き換えて壊さないため
- Acceptance:
  - Given: 既にある `out.csv`
  - When:
  - Then: 確かめの問い、`n` で書かない。

#### 画面の表を書き出したら、書いた行の数とパスを知らせるべき
`req~output.export-path-report~1` P1
- Acceptance:
  - Given:
  - When: 3行の表を `out.csv` に書き出す
  - Then: 3行と `out.csv` のパスを知らせる。

#### 読むだけの起動でも、画面の表を書き出せてよい(ノートは書かない)
`req~output.export-path-readonly~1` P1
- Acceptance:
  - Given:
  - When: `--readonly` の起動で、パレットの「表を書き出す」から `out.csv`
  - Then: 書き出せれば適合で、ノートのファイルは変わらない。

### 標準出力

#### `mdgrid [<.base | フォルダ> …] --pick path|<列の名前>` は、画面を端末(`/dev/tty`)に出し、読むだけ(cli.readonly と同じ)で開くべき
`req~output.pick-open~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --pick path` で開き、セルを空にして Ctrl+S
  - Then: ノートは書かれない。

#### `--pick path` は、Enter で、印を付けた行(印が無ければ選んでいる行)ごとにノートのパスを1行ずつ標準出力に出し、終了コード 0 で終わるべき(パスは起動の引数のフォルダにノートの相対のパスをつないだもの)
`req~output.pick-path~2` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --pick path` で2行に印を付けて Enter
  - Then: `notes/a.md` と `notes/b.md` の2行が出て終わる。
- Acceptance:
  - Given:
  - When: 印なしで Enter
  - Then: 選んでいる行の1行。

#### `--pick <列の名前>` は、Enter で、印を付けた行(印が無ければ選んでいる行)ごとにその列の値の素の文字を1行ずつ標準出力に出し、終了コード 0 で終わるべき(画面の印を付けず、null と空は空、リストは `, ` でつなぎ、日付は `YYYY-MM-DD`、改行は空白。cli.print の csv と同じ決まり)
`req~output.pick-column~1` P1
- Acceptance:
  - Given: `--pick status`
  - When:
  - Then: 値が1行ずつ。

#### `--pick` は、`q` と、解くもの(印・範囲・絞り込み・検索)が無いときの Esc で取りやめたら、何も出さずに終了コード 1 で終わるべき
`req~output.pick-cancel~1` P1
- Acceptance:
  - Given:
  - When: `q`
  - Then: 何も出ず終了コード 1。

#### `--pick` は、標準出力がパイプでも動くべき(cell-view.no-color)
`req~output.pick-pipe~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid notes --pick path | cat` で2行に印を付けて Enter
  - Then: 標準出力がファイルのときと同じ2行が出る。

#### `--pick` に表に無い列の名前を渡したら、cli.fail-fast と同じく理由1行と終了コード 2 にするべき
`req~output.pick-unknown-column~1` P1
- Acceptance:
  - Given: `--pick 無い列`
  - When:
  - Then: 理由1行と終了コード 2。
