# 通知の欄(notice) の要求

## 目的

設定した人だけ、表の上に小さな欄を出し、名前の条件(condition)に当たる行の数(「期限切れ 3 件」など)と、自分で書いた連絡(決めたノートの本文の頭)を見せる。欄から当の行へすぐ移れるようにし、同じ中身を `mdgrid --notices` で mdgrid の外(tmux の帯・シェル・Herdr のサイドバー)にも出す。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-21 に足した。

## 範囲外

- 採らない: OS の通知(デスクトップの通知・音)。mdgrid を開いていないときに知らせるのは、`--notices` を端末の道具の帯に出して行う。
- 採らない: 通知の欄の中で値を直すこと。直すのは、欄から移った先の表で行う。
- 条件そのものの書き方は condition。

## 要求

### 項目を書く

#### 設定の `[[notices]]` の並びに項目を書いたときだけ、通知の欄を出すべき
`req~notice.opt-in~2` P1
- Rationale: 要らない人の表の行を減らさないため(2026-11-20 に人が「設定したい人は」と決めた)
- Acceptance:
  - Given: `[[notices]]` の無い設定
  - When:
  - Then: 画面に通知の欄が無く、表の見える行の数は今と同じ。
- Acceptance:
  - Given: 件数の項目を1つ書き、当たる行がある
  - When:
  - Then: ヘッダーの上に通知の欄が出る。
- Needs: stest

#### 件数の項目は、条件の名前 `cond`(condition.define)と、件数を差し込む文 `text`(`{count}` を件数に置き換える)で書き、`text` が無ければ条件の見せる名前(condition.label)と件数を出すべき
`req~notice.count-item~2` P1
- Rationale: 何を何件と知らせるかを、人が自分の言葉で決められるようにするため(neomutt のサイドバーが、保存した検索ごとに件数を並べるのと同じ形)
- Acceptance:
  - Given: `cond = "overdue"`・`text = "期限切れ {count} 件"` で、overdue に当たる行が3つ
  - When:
  - Then: 欄に「期限切れ 3 件」。
- Acceptance:
  - Given: `text` を書かない
  - When:
  - Then: 「overdue 3」。
- Acceptance:
  - Given:
  - When: `cond` に書いていない名前
  - Then: cli.config-file-warning-form の形で警告し、その項目を出さない。

#### 件数が 0 の項目は出さず、`show_zero = true` を書いた項目だけ 0 でも出すべき
`req~notice.zero~2` P1
- Rationale: 知らせることが無いときは欄を空け、表の行を押し出さないため
- Acceptance:
  - Given: overdue が 0 件
  - When:
  - Then: 項目が出ず、ほかに出す項目も無ければ欄ごと出ない。
- Acceptance:
  - Given: `show_zero = true`
  - When:
  - Then: 「期限切れ 0 件」。

#### 件数の項目は、数える範囲 `scope` を、今の表(`"table"`)・今の範囲の表の全部(`"workspace"`。workspace.scope-order-rank で決まる範囲)・表の名前の並び(ワークスペースの表の `name`)から選べ、書かなければ `"table"` にするべき
`req~notice.scope~1` P1
- Rationale: 今の表のことだけ知りたい人と、どの表を開いていても全体の期限切れを知りたい人がいるため(2026-11-21 に人が「跨いでもいい。使う人による」と決めた)
- Acceptance:
  - Given:
  - When: tasks に2行・projects に1行の overdue があるワークスペースで tasks を開く
  - Then: `scope = "table"` は「期限切れ 2 件」、`"workspace"` は「期限切れ 3 件」、`["projects"]` は「期限切れ 1 件」。
- Needs: stest

#### 件数は、表の行(`.base` の最上位の絞り込みと condition.hide-always を当てたあと)のうち条件に当たるものを数え、ビューの絞り込み・検索・数字のキーの条件に左右されないべき
`req~notice.count-rows~2` P1
- Rationale: 表の中で絞り込みを変えるたびに、知らせの数が変わらないようにするため
- Acceptance:
  - Given:
  - When: overdue が3行の表で、`\` の検索で1行に絞る
  - Then: 欄は「期限切れ 3 件」のまま。
- Acceptance:
  - Given:
  - When: overdue の行を done に絞ったビューへ移る
  - Then: 「期限切れ 3 件」のまま。

#### 連絡の項目は、ノートのパス `note`(先頭の `~` はホーム、相対なら設定のファイルのあるフォルダから)を書くと、そのノートの本文(フロントマターを除く)の空でない行を頭から `lines` 行(既定は 1)出すべき
`req~notice.note-item~2` P1
- Rationale: 自分で書いた連絡事項を、表を開くたびに目に入れるため
- Acceptance:
  - Given: `note = "~/notes/inbox.md"`・`lines = 2` で、inbox.md の本文が空の行・「- 金曜に請求書」・「- 鍵を返す」・「- 歯医者」の順
  - When:
  - Then: 欄に「- 金曜に請求書」と「- 鍵を返す」。
- Acceptance:
  - Given: 本文が空
  - When:
  - Then: 項目を出さない。
- Acceptance:
  - Given: ファイルが無い
  - When:
  - Then: 項目に「inbox.md がありません」と出し、起動は止めない。
- Needs: stest

### 見せ方

#### 通知の欄は、ヘッダーの上に、書いた順に項目を並べ、件数の項目は1行に `·` で区切ってつなぎ、連絡の項目は1行ずつ置き、3行を超える分は3行目を「ほか N 行」にまとめるべき
`req~notice.layout~2` P1
- Rationale: 知らせを一目で読める形にしつつ、表の行を押し出しすぎないため
- Acceptance:
  - Given: 件数の項目2つと連絡の項目1つ(`lines = 1`)
  - When:
  - Then: 欄は2行で、1行目が「期限切れ 3 件 · 今日まで 2 件」、2行目が連絡。
- Acceptance:
  - Given: 連絡を `lines = 5` で出す
  - When:
  - Then: 欄は3行で、3行目が「ほか 3 行」。
- Needs: stest

#### 通知の欄の件数と連絡は、開いたとき・保存したあと・表を読み直したとき(screen.editor-reload など)・日付が変わったときに、数え直して読み直すべき
`req~notice.refresh~2` P1
- Rationale: 直した値や今日の日付と、知らせの中身がずれないようにするため
- Acceptance:
  - Given:
  - When: 期限切れ 3 件の表で1行の due を来月に直して保存
  - Then: 「期限切れ 2 件」。
- Acceptance:
  - Given: 開いたまま日付が 2026-11-21 から 2026-11-22 に変わる
  - When:
  - Then: due が 2026-11-21 の行も期限切れに数える。

#### ほかの表を数える項目は、最初の画面を待たせずに数え、数え終わるまで件数の所に「…」を出すべき
`req~notice.background~1` P1
- Rationale: ワークスペースの表が多くても、開いた表をすぐに使えるようにするため
- Acceptance:
  - Given: 数えるのに時間のかかる表を含む `scope = "workspace"`
  - When:
  - Then: 表は先に出て操作でき、欄は「期限切れ … 件」のあと数に替わる。

#### 件数の項目の範囲に読めない表(フォルダが無い・`.base` が読めない)があるときは、その表を飛ばして数え、件数のあとに「?」を添えるべき
`req~notice.unreadable~1` P1
- Rationale: 1つの表が読めないせいで知らせ全体が消えたり、数が足りないのに正しく見えたりしないようにするため
- Acceptance:
  - Given: `scope = "workspace"` の表の1つのフォルダが消えている
  - When:
  - Then: 「期限切れ 2 件?」で、起動は止めない。

### 操作

#### 通知の欄へは、キー(screen.keys-vim-and-arrows-remap で割り当て直せる)かクリックで移り、←→ か ↑↓ で項目を選び、Esc で表に戻るべき
`req~notice.focus~2` P1
- Rationale: マウスを使わない人も、知らせから当の行へ移れるようにするため
- Acceptance:
  - Given:
  - When: 通知の欄のある画面で欄へ移るキー
  - Then: 1つ目の項目が選ばれ、下の帯に欄のキーが出る。
- Acceptance:
  - Given:
  - When: Esc
  - Then: 表の同じセルに戻る。
- Acceptance:
  - Given:
  - When: 通知の欄の無い画面でそのキー
  - Then: 何も変わらない。

#### 件数の項目で Enter を押すと、範囲が今の表だけなら今の表をその条件で絞り、ほかの表を含むなら、当たる行のある表を件数つきで並べて、選んだ表をその条件で絞って開くべき
`req~notice.jump~2` P1
- Rationale: 知らせを見てすぐに当の行を直せるようにするため
- Acceptance:
  - Given:
  - When: `scope = "table"` の「期限切れ 3 件」で Enter
  - Then: 今の表が overdue の3行になり、設定の帯に overdue が出る(数字のキーで入れたときと同じく、ほかの表へ移ると解ける)。
- Acceptance:
  - Given:
  - When: `scope = "workspace"` で Enter → 「tasks 2 · projects 1」の並び → projects を選ぶ
  - Then: projects の表が overdue の1行で開く。ためた変更があれば workspace.drawer-unsaved と同じく保存か取りやめを確かめる。
- Needs: stest

#### 連絡の項目で Enter を押すと、そのノートを外のエディタ(screen.editor-open)で開き、戻ったら欄を読み直すべき
`req~notice.open-note~2` P1
- Rationale: 連絡を書き足す・消すのを、表から離れずにできるようにするため
- Acceptance:
  - Given:
  - When: 連絡の項目で Enter
  - Then: エディタで inbox.md が開く。
- Acceptance:
  - Given:
  - When: 1行目を消して閉じる
  - Then: 欄に2行目だった行が出る。

### 外に出す

#### `mdgrid [<.base | フォルダ> | -w <名前>] --notices` は、画面を出さずに、通知の欄と同じ項目を1項目1行の文で出し、終了コード 0 で終わるべき
`req~notice.cli~2` P1
- Rationale: mdgrid を開いていないときにも、tmux の帯・シェル・Herdr のサイドバーに同じ知らせを出すため(1回書いた条件が画面の外にも出る)
- Acceptance:
  - Given: overdue が3行と連絡が1行
  - When:
  - Then: 標準出力に「期限切れ 3 件」と「- 金曜に請求書」の2行と終了コード 0。
- Acceptance:
  - Given: `[[notices]]` が無いか、出す項目が無い
  - When:
  - Then: 何も出さず終了コード 0。標準出力がパイプでも同じ。
- Needs: stest

#### `--notices` は、件数の項目を数え終わってから出し、読めない表は通知の欄と同じく「?」を添え、項目の文に色や飾りを付けないべき
`req~notice.cli-plain~2` P1
- Rationale: 端末の道具の帯にそのまま差し込めるようにするため
- Acceptance:
  - Given: 表の1つのフォルダが消えている
  - When:
  - Then: 「期限切れ 2 件?」の1行。
- Acceptance:
  - Given:
  - When: `mdgrid --notices | od -c`
  - Then: エスケープの文字が無い。

## 未確定事項

### 通知の欄へ移るキー
`tbd~notice.focus-key`
- Decider: 人
- Provisional: `m`(表の画面で空いているキー。vim モードで `m` を印に使うと決めたら替える)

### 通知の欄の置き場
`tbd~notice.position`
- Decider: 人
- Provisional: ヘッダーの上(画面のいちばん上)。ヘッダーとビューの帯のあいだに置く形も選べる
