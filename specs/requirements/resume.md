# 開き直し(resume) の要求

## 目的

端末の道具(Herdr・tmux など)が、止まったあとに窓を立て直すとき、mdgrid を同じ表とビューで開き直せるようにする。Herdr には、mdgrid が今の表とビューで開き直す命令をそのつど伝える。命令を受け取れない道具(tmux の復元のプラグインなど)のために、起動したフォルダごとに最後の画面を覚え、`mdgrid --continue` でそこへ戻れるようにする。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-20 に足した。

## 範囲外

- 採らない: 選んでいた行・画面の型(ガント・カンバンなど)・スクロールの位置を戻すこと(2026-11-20 に人が「表とビューまで」と決めた)。
- 採らない: 保存していない変更を戻すこと(mdgrid が止まると、ためた変更も消える)。
- 名前を付けて表示を残して開くのは、セッション(cli.session-save。次の段)。ここは名前の無い「最後の画面」だけを持つ。

## 要求

### Herdr に開き直す命令を伝える

#### Herdr の窓(環境変数 `HERDR_ENV` が `1` で、`HERDR_PANE_ID` と `HERDR_BIN_PATH` がある)で画面を出したときは、今の表とビューで開き直す命令を、Herdr の道具(`HERDR_BIN_PATH`)の `pane report-agent` で伝えるべき
`req~resume.herdr-report~1` P1
- なぜ: Herdr のサーバーが止まって立て直したとき、窓に同じ表とビューの mdgrid を開き直してもらうため
- 受け入れ: `HERDR_ENV=1`・`HERDR_PANE_ID=w1:p1` の窓で `mdgrid notes/tasks.base --view 進行中` → Herdr の道具が `pane report-agent w1:p1 --source mdgrid --agent mdgrid --state idle --seq <数> -- mdgrid <tasks.base の絶対パス> --view 進行中` の引数で呼ばれる。`HERDR_ENV` の無い窓 → Herdr の道具は呼ばれない。

#### 開き直す命令は、`mdgrid` に、開いている表の絶対パス(フォルダ1つなら続けて `--raw`)、今のビューの名前の `--view`、起動で渡した `-w`・`--readonly`・`--no-color`・`--config`(絶対パス)をこの順に並べたものにするべき
`req~resume.reopen-args~1` P1
- なぜ: 命令を走らせる窓のフォルダが変わっても、またフォルダの印の `open`(workspace.marker)に置き換わらずに、同じ表とビューが開くようにするため
- 受け入れ: `-w tasks --readonly` で、ビューの無いフォルダ notes を開く → 命令が `mdgrid <notes の絶対パス> --raw -w tasks --readonly`。`.base` のビュー「進行中」を開いている → `mdgrid <.base の絶対パス> --view 進行中`。

#### 表かビューを移ったら、伝えた命令を、移った先の表とビューのものに替えるべき
`req~resume.herdr-follow~1` P1
- なぜ: 立て直したときに、最後に見ていた表とビューへ戻すため
- 受け入れ: ビューのタブを「全部」へ切り替える → 2 秒以内に `-- mdgrid <パス> --view 全部` を伝える。登録した表の一覧(cli.places-list)で別の表へ移る → その表の絶対パスの命令を伝える。同じ表とビューのままの操作(行の移動)→ 新しくは伝えない。

#### 画面を終えるときは、`pane release-agent` で窓を Herdr に返してから終わるべき
`req~resume.herdr-release~1` P1
- なぜ: 人が自分で mdgrid を終えた窓を、Herdr が立て直しのときに開き直さないため
- 受け入れ: `q` で終える → Herdr の道具が `pane release-agent w1:p1 --source mdgrid --agent mdgrid --seq <数>` の引数で呼ばれ、そのあと mdgrid が終わる。

#### `--seq` の数は、伝えるたびに、前に伝えた数(mdgrid を起動し直す前の回も含む)より大きくするべき
`req~resume.herdr-seq~1` P1
- なぜ: Herdr は前より大きくない数の報告を捨てるので、起動し直したあとの命令も受け付けてもらうため
- 受け入れ: 起動してビューを切り替えて終える → 伝えた3つの `--seq` の数が、この順に大きくなる。続けて起動し直す → 最初の `--seq` の数が、前の回の最後の数より大きい。

#### 命令に Herdr が受け付けない引数(アポストロフィ `'` か制御文字を含む、65 個以上、合わせて 8 KiB を超える)があるときは、命令を伝えず、代わりに `pane release-agent` で前に伝えた命令を取り下げるべき
`req~resume.herdr-unsendable~1` P1
- なぜ: Herdr は決まりに合わない命令の報告を丸ごと捨てるので、前の表の命令が残って別の表を開き直さないようにするため
- 受け入れ: フォルダ `it's` を開く → `pane report-agent` は呼ばれず、`pane release-agent` が呼ばれる。

#### Herdr の道具が返らないか失敗するときも、画面の操作と終了を待たせないべき
`req~resume.herdr-no-wait~1` P1
- なぜ: Herdr の側の不具合で、mdgrid が使えなくならないため
- 受け入れ: 呼ばれると 30 秒眠る Herdr の道具で起動する → 表が出てキーが効き、`q` から 5 秒以内に mdgrid が終わる。終了コード 1 で終わる道具 → 知らせの行に何も出さずに動き続ける。

#### 設定 `terminal.herdr` が `false` のときは、Herdr の窓でも Herdr の道具を呼ばないべき
`req~resume.herdr-config-off~1` P1
- なぜ: Herdr の立て直しで mdgrid を開き直したくない人が、mdgrid の側だけで止められるようにするため
- 受け入れ: config.toml に `[terminal]` の `herdr = false` を書き、Herdr の窓で起動して終える → Herdr の道具は1回も呼ばれない。書かない → 呼ばれる(既定は `true`)。

#### `--pick`(output.pick)の画面と、画面を出さない命令(`--print`・`--check` など)では、Herdr の道具を呼ばないべき
`req~resume.herdr-skip-pick~1` P1
- なぜ: 選んで終わる一度きりの画面を、立て直しで開き直さないため
- 受け入れ: Herdr の窓で `--pick path` を開いて Esc で取りやめる → Herdr の道具は呼ばれない。

### 最後の画面を覚えて `--continue` で開き直す

#### 画面で表かビューを開くたびに、起動したフォルダごとに、開き直す命令(resume.reopen-args)と同じ中身を「最後の画面」として見た目の状態の置き場(screen.state-store-key と同じ置き場)に書き、ノートのフォルダには書かないべき
`req~resume.last-screen-save~1` P1
- なぜ: tmux の復元のプラグインのような、mdgrid から命令を受け取れない道具でも、立て直したあとに今の表とビューへ戻れるようにするため
- 受け入れ: notes で起動して tasks.base のビュー「進行中」に切り替える → `$XDG_STATE_HOME/mdgrid/` の下に、notes の最後の画面として tasks.base と「進行中」が書かれ、notes の中に新しいファイルが無い。

#### `mdgrid --continue` は、今のフォルダで起動した最後の画面の表とビュー(と `-w`・`--readonly`・`--no-color`・`--config`)で開くべき
`req~resume.continue~1` P1
- なぜ: 立て直しの道具が窓のフォルダで `mdgrid --continue` を走らせるだけで、前の表とビューに戻れるようにするため
- 受け入れ: notes で tasks.base のビュー「進行中」に切り替えて終える → notes で `mdgrid --continue` → tasks.base の「進行中」のタブで開く。別のフォルダ other で最後に開いた表 → notes の `--continue` には使わない。

#### 今のフォルダの最後の画面が無いか壊れているか、その表がもう無いときは、`--continue` を引数なしの起動と同じに開くべきで、覚えたビューがもう無いときは、表を開いてビューが無い理由を知らせの行に出すべき
`req~resume.continue-fallback~1` P1
- なぜ: 立て直しの道具が走らせる `mdgrid --continue` が、エラーで止まって窓を空にしないため
- 受け入れ: 最後の画面の無いフォルダで `mdgrid --continue` → `mdgrid` と同じ表が開く。最後の画面のファイルを壊す → 同じ。覚えたビュー「進行中」を `.base` から消す → tasks.base が開き、「進行中」が無い理由が知らせの行に出る。

#### `--continue` を、開くものを決める引数(パス・`--view`・`-w`・`--raw`)か画面を出さない命令(`--print`・`--pick` など)と一緒に渡されたときは、cli.fail-fast と同じく理由を1行で出して終了コード 2 で終わるべき
`req~resume.continue-conflict~1` P1
- なぜ: 覚えた画面と引数のどちらを開くかが曖昧になるため(cli.session-errors の `-s` とパスと同じ扱い)
- 受け入れ: `mdgrid --continue notes` → 理由1行と終了コード 2。`mdgrid --continue --print` → 同じ。`mdgrid --continue --readonly` → 最後の画面を読むだけで開く。

## 未確定事項

### Herdr に伝えるのを止める設定の名前と置き場
`tbd~resume.herdr-config-name`
- 決める人: 人
- 仮: `[terminal]` の `herdr`(真偽。既定は `true`)

### Herdr のサイドバーに出す mdgrid の状態
`tbd~resume.herdr-state`
- 決める人: 人
- 仮: いつも `idle`。Herdr は、命令を伝えた窓の mdgrid をエージェントの一覧に名前 `mdgrid` で並べる。保存していない変更があるときに `blocked` にして知らせるのは採らない

### `--readonly` で開いた画面でも最後の画面を書くか
`tbd~resume.readonly-save`
- 決める人: 人
- 仮: 書く(書くのは見た目の状態の置き場だけで、ノートと設定には書かない。`--continue` も読むだけで開く)

### 最後の画面をどの単位で分けるか
`tbd~resume.last-screen-key`
- 決める人: 人
- 仮: 起動したときの今のフォルダ(実体のパス)ごとに1つ。同じフォルダで2つの窓に mdgrid を開いていたら、あとで表かビューを移った方が残る
