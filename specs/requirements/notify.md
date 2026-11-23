# 新しく当たった行を知らせる(notify) の要求

## 目的

名前の条件(condition)に新しく当たった行を、1行ずつ、選んだ出す先(端末の通知・OS のデスクトップの通知・自分のコマンド)に知らせる。ノートに書いた日時(`remind <= now()`)や期限(`due <= today()`)を条件にすれば、Markdown のノートのままで、思い出したい時刻に知らせを受けられる。mdgrid を開いている間は画面が確かめ、閉じているあいだは OS の定期実行から呼ぶ `mdgrid --notify` が1回ずつ確かめる。動き続ける見張りのプロセスは置かない。

macOS・Linux・Windows と、WSL の中の mdgrid で、同じ設定が動くようにする。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-23 に足した。

## 範囲外

- 採らない: 常駐するプロセス。開いている間は画面が、閉じているあいだは OS の定期実行が確かめる(2026-11-23 に人が「常駐はよくない」と決めた)。
- 次の段: OS の定期実行への登録を mdgrid がすること。今は説明書に OS ごとの例を載せ(notify.schedule-docs)、登録は人がする。
- 次の段: 知らせから操作すること(あとでもう一度知らせる・済みにする・押して mdgrid で開く)。
- 次の段: 鳴らす音を選ぶこと。今は OS と端末の既定の音に任せる。
- 採らない: 鳴らさない時間帯の設定。OS の集中モード(おやすみモード)に任せる。
- 条件の書き方は condition。当たる行の数を表の上に見せるのは通知の欄(notice)。

## 要求

### 項目を書く

#### 設定の `[[notify]]` の並びに項目を書いたときだけ確かめて鳴らし、書いていなければ鳴らした記録も作らないべき
`req~notify.opt-in~1` P1
- Rationale: 要らない人に知らせを出さず、状態の置き場にも何も足さないため
- Acceptance:
  - Given: `[[notify]]` の無い設定と、名前の条件に当たる行のある表
  - When: 画面で開く
  - Then: どの出す先にも何も出ず、状態の置き場に鳴らした記録のファイルが無い。
- Needs: utest

#### 項目は、条件の名前 `cond`(condition.define)と、確かめる範囲 `scope`(notice.scope と同じ値。書かなければ `"workspace"`)で書き、書いていない条件の名前と、前の項目と同じ `cond` は、警告してその項目を使わないべき
`req~notify.item~1` P1
- Rationale: 通知の欄と同じ条件と範囲の書き方で、どの行を知らせるかを決めるため。範囲を書かないときは、どの表を開いていても知らせが届くよう、範囲の表の全部にする(tbd~notify.default-scope)
- Acceptance:
  - Given: `cond = "remind"` で `scope` を書かず、tasks と projects の表のあるワークスペースで tasks を開いている
  - When: projects の1行が remind に新しく当たる
  - Then: その行を鳴らす。
- Acceptance:
  - Given: `cond = "nothing"`(`[conditions]` に無い名前)
  - When: 起動する
  - Then: cli.config-file-warning-form の形で警告し、その項目では鳴らさない。
- Acceptance:
  - Given: `cond = "remind"` の項目が2つ
  - When: 起動する
  - Then: 2つ目を警告して使わず、1つ目で鳴らす。
- Needs: utest

#### 知らせの題は `title`、文は `text` に書き、どちらも `{<列の名前>}` をその行の値(表のセルに出る文字)に置き換え、書かなければ題は条件の見せる名前(condition.label)、文は行の名前(ノートの名前。CSV・TSV の表は1列目の値)にするべき
`req~notify.text~1` P1
- Rationale: 行ごとに、何の知らせかを人が自分の言葉で決められるようにするため(2026-11-23 に人が「行ごとはもちろん」と決めた)
- Acceptance:
  - Given: `title = "思い出す"`・`text = "{title}({due})"` の項目と、title が「請求書を送る」・due が 2026-11-25 のノート
  - When: そのノートが新しく当たる
  - Then: 題が「思い出す」、文が「請求書を送る(2026-11-25)」の知らせが出る。
- Acceptance:
  - Given: `title` と `text` を書かず、条件の見せる名前が「リマインダー」の項目
  - When: ノート `請求書.md` が新しく当たる
  - Then: 題が「リマインダー」、文が「請求書」。
- Acceptance:
  - Given: `text = "{owner}: {title}"` の項目と、owner の無いノート
  - When: そのノートが新しく当たる
  - Then: 文が「: 請求書を送る」で、エラーにしない(無い列は空にする)。
- Needs: utest

#### 出す先 `to` は、`"terminal"`(notify.terminal)・`"desktop"`(notify.desktop)・`"command"`(notify.command)から1つ以上を並べ、書かなければ `["desktop"]` にし、知らない値は警告してその値だけを使わないべき
`req~notify.to~1` P1
- Rationale: 使う端末や OS、知らせを受けたい所(電話への中継など)が人によって違うため(2026-11-23 に人が「選べるといいね」と決めた)。書かないときは、macOS・Windows・WSL で足す道具なしに出せる OS の通知にする(tbd~notify.default-to)
- Acceptance:
  - Given: `to = ["terminal", "command"]` の項目
  - When: 1行が新しく当たる
  - Then: 端末とコマンドに1回ずつ出し、OS の通知には出さない。
- Acceptance:
  - Given: `to = ["desktop", "phone"]` の項目
  - When: 起動する
  - Then: `phone` を警告し、当たった行は OS の通知に出す。
- Needs: utest

### いつ鳴らすか

#### 鳴らすのは、前の確かめで当たっていなかった行が当たったときだけにし、当たり続ける行は二度鳴らさず、外れたあとにまた当たったら鳴らすべき
`req~notify.new-match~1` P1
- Rationale: 同じことを何度も知らせないため。日時を延ばして外れた行が、また来たときは知らせる。行は、ノートの表ではノートのファイル、CSV・TSV の表では行の値の並びで見分ける(tbd~notify.csv-row-id)
- Acceptance:
  - Given: remind に当たっていなかったノート a
  - When: a が当たり、そのあと3回確かめる
  - Then: 鳴るのは当たった最初の確かめの1回だけ。
- Acceptance:
  - Given: 鳴らしたあと、remind を明日に直して外れたノート a
  - When: 明日になって、また当たる
  - Then: a をもう1回鳴らす。
- Acceptance:
  - Given: 鳴らしたあとも当たり続けているノート a
  - When: a の title を直して保存する
  - Then: a を鳴らさない(同じファイルなので同じ行)。
- Needs: utest

#### その項目でその表を初めて確かめたときは、当たっている行を鳴らさずに覚えるだけにするべき
`req~notify.first-silent~1` P1
- Rationale: 設定を書いた途端に、前から当たっていた行の知らせが一度に来ないようにするため(2026-11-23 に人が「黙って覚えるだけ」と決めた)
- Acceptance:
  - Given: remind に当たるノートが5つある表に、`[[notify]]` の項目を初めて書いた
  - When: 確かめる
  - Then: 何も鳴らない。
- Acceptance:
  - Given: その確かめのあと
  - When: 6つ目のノートが新しく当たる
  - Then: 6つ目だけを鳴らす。
- Acceptance:
  - Given: 前から確かめているワークスペースに、remind に当たる行が2つある表を足した
  - When: 確かめる
  - Then: 足した表の2行は鳴らない。
- Needs: utest

#### 1回の確かめで1つの項目に新しく当たった行が `group_over`(書かなければ 3)を超えたら、行ごとに鳴らさず1件にまとめ、題を条件の見せる名前、文を `group_text`(`{count}` を行の数に置き換える。書かなければ「新しく {count} 件」)にするべき
`req~notify.group~1` P1
- Rationale: 閉じていたあいだにたまった行や、まとめて日付を直した行で、知らせが一度にあふれないようにするため(2026-11-23 に人が「決めた数を超えたらまとめる」と決めた。Android も同じアプリの知らせが4件になると1つにまとめる)。数は項目ごとに決められる(tbd~notify.group-default)
- Acceptance:
  - Given: `group_over` を書かない項目
  - When: 1回の確かめで3行が新しく当たる
  - Then: 3件を1行ずつ鳴らす。
- Acceptance:
  - Given: `group_over` を書かない項目
  - When: 1回の確かめで4行が新しく当たる
  - Then: 題が条件の見せる名前、文が「新しく 4 件」の1件だけを鳴らし、4行とも鳴らしたと覚える。
- Acceptance:
  - Given: `group_over = 0` の項目
  - When: 1行が新しく当たる
  - Then: 文が「新しく 1 件」の1件にまとめる。
- Needs: utest

#### 開いている間は、開いたとき・保存したあと・1分ごとに、範囲の表をファイルから読んで確かめ、画面の操作を待たせないべき
`req~notify.while-open~1` P1
- Rationale: `now()` を使う条件は、ファイルが変わらなくても時刻が進むだけで当たるため。外のエディタで直したノートも拾えるよう、確かめはファイルから読む。1分より細かい時刻は、ノートの知らせには要らない(tbd~notify.interval)
- Acceptance:
  - Given: 9:58 に開いた画面と、`expr = "remind <= now()"` の条件の項目と、remind が 10:00 のノート
  - When: 時刻が 10:00 を過ぎる
  - Then: 10:01 までに、そのノートを鳴らす。
- Acceptance:
  - Given: 画面でためた変更で、remind を過去にした行
  - When: 保存する
  - Then: 保存する前は鳴らず、保存したあとに鳴らす。
- Acceptance:
  - Given: 読むのに時間のかかる表を含む範囲の項目
  - When: 1分ごとの確かめ
  - Then: 確かめている間も、画面はキーに応える(notice.background と同じく裏で読む)。
- Needs: utest

#### `mdgrid [<.base | フォルダ> | -w <名前>] --notify` は、画面を出さずに `[[notify]]` の項目を1回確かめて鳴らし、端末の合図(notify.terminal)のほかは標準出力に何も出さずに、終了コード 0 で終わるべき
`req~notify.cli~1` P1
- Rationale: mdgrid を閉じているあいだも、OS の定期実行(systemd のタイマー・launchd・タスク スケジューラ・cron)から呼んで知らせるため(2026-11-23 に決めた)。標準出力に何も出さないので、定期実行の記録やシェルのプロンプトを汚さない
- Acceptance:
  - Given: `to = ["command"]` の項目と、前の `--notify` のあとで新しく当たった行が1つ
  - When: `mdgrid -w home --notify`
  - Then: コマンドが1回動き、標準出力は空で、終了コード 0。
- Acceptance:
  - Given: `[[notify]]` が無いか、新しく当たった行が無い
  - When: `mdgrid -w home --notify`
  - Then: 何も鳴らさず、標準出力は空で、終了コード 0。
- Acceptance:
  - Given: `-w` に、無いワークスペースの名前
  - When: `--notify`
  - Then: cli.fail-fast と同じく、理由1行と終了コード 2。
- Needs: stest

#### 出す先に届けられなかったときは、開いている間は画面の下の帯に理由を出し、`--notify` は理由を標準エラーに1行ずつ出して終了コード 1 で終わり、どの出す先にも届かなかった行は覚えずに、次の確かめでまた鳴らすべき
`req~notify.fail~1` P1
- Rationale: cron から呼ぶと Linux の OS の通知が届かないことがあり、黙って落とすと知らせを逃すため。出す先のコマンドが止まっても、画面と定期実行を止めない(打ち切りの時間は tbd~notify.timeout)
- Acceptance:
  - Given: `to = ["command"]` の項目で、コマンドが終了コード 1 で終わる
  - When: `--notify` で1行が新しく当たる
  - Then: 標準エラーに、出す先とコマンドの終了コードを含む1行が出て、mdgrid の終了コードは 1。次の `--notify` で、その行をまた鳴らそうとする。
- Acceptance:
  - Given: `to = ["desktop", "command"]` の項目で、コマンドだけが失敗する
  - When: `--notify` で1行が新しく当たる
  - Then: 終了コードは 1 だが、その行は鳴らしたと覚え、次の `--notify` では鳴らさない。
- Acceptance:
  - Given: 開いている画面と、10 秒たっても終わらないコマンドの出す先
  - When: 1行を鳴らす
  - Then: 画面はキーに応え続け、10 秒で打ち切って下の帯に理由を出す。同じ出す先の同じ理由は、開いている間に1回だけ出す。
- Acceptance:
  - Given: 範囲の表の1つのフォルダが消えている
  - When: `--notify`
  - Then: ほかの表は確かめて鳴らし、消えた表の理由を標準エラーに出して、終了コード 1。消えた表の記録は消さない。
- Needs: utest・stest

### 記録

#### 当たった行の記録は、見た目の状態の置き場(screen.state-store-key と同じ置き場)に持ってノート・`.base`・設定には書かず、開いている画面と `--notify` で同じ記録を使い、同じ行を二度鳴らさないべき
`req~notify.state~1` P1
- Rationale: 鳴らしたかどうかをノートに書くと、ノートの差分や同期が騒がしくなるため(2026-11-23 に決めた)。画面と定期実行の両方が確かめても、知らせは1回にする
- Acceptance:
  - Given: 開いている画面が、新しく当たったノート a を鳴らした
  - When: 続けて `--notify`
  - Then: a を鳴らさない。
- Acceptance:
  - Given: 新しく当たったノート a
  - When: `--notify` を2つ同時に走らせる
  - Then: a は合わせて1回だけ鳴る。
- Acceptance:
  - Given: 新しく当たったノート a
  - When: `--notify` で鳴らす
  - Then: ノートのフォルダのファイルの数とバイトは変わらない。
- Needs: stest

#### 記録が読めないときは、初めて確かめたとき(notify.first-silent)と同じく黙って覚え直し、警告を1回出すべき
`req~notify.state-broken~1` P1
- Rationale: 壊れた記録のせいで、前から当たっている行の知らせが一度に来ないようにするため
- Acceptance:
  - Given: 記録のファイルの中身が壊れていて、remind に当たる行が3つ
  - When: `--notify`
  - Then: 何も鳴らさず、標準エラーに記録を作り直した旨の1行が出て、終了コード 0。
- Acceptance:
  - Given: その `--notify` のあと
  - When: 4つ目の行が新しく当たって `--notify`
  - Then: 4つ目だけを鳴らす。
- Needs: stest

### 出す先

#### `"terminal"` は、mdgrid を動かしている端末に、その端末が通知として見せる合図を1つだけ送り、tmux の中では外の端末に届く形で送り、見分けられない端末には BEL を1つ送るべき
`req~notify.terminal~1` P1
- Rationale: 端末ごとに通知の合図が違い(iTerm2 は OSC 9、kitty は OSC 99、Ghostty・WezTerm・foot は OSC 777。Windows Terminal は 1.26 から、設定で OSC 777 を受ける)、2つ以上送ると両方を読む端末で二重に鳴るため。端末を人が選び直せるようにするかは tbd~notify.terminal-pick
- Acceptance:
  - Given: kitty の中(`TERM=xterm-kitty`)
  - When: 1行を鳴らす
  - Then: 端末への出力に OSC 99 の合図があり、OSC 9 と OSC 777 は無い。
- Acceptance:
  - Given: tmux の中(`TMUX` がある)で、外の端末が Ghostty
  - When: 1行を鳴らす
  - Then: OSC 777 の合図を、tmux を通り抜ける DCS(`tmux;`)で包んで送る。
- Acceptance:
  - Given: 見分けられない端末
  - When: 1行を鳴らす
  - Then: BEL を1つだけ送り、OSC の合図は送らない。
- Acceptance:
  - Given: `--notify` で、標準出力がパイプ
  - When: 1行を鳴らす
  - Then: 端末の合図を送らず、届けられなかったとは数えない。
- Needs: utest

#### `"desktop"` は、OS のデスクトップの通知に題と文を出し、macOS・Linux・Windows で動き、WSL の中では Windows の通知に出すべき
`req~notify.desktop~1` P1
- Rationale: 端末が通知の合図を読まなくても、知らせを画面の端に出すため。WSL の中の Linux には通知を受ける所が無いことが多いので、Windows の側に出す(2026-11-23 に人が「WSL 上や Windows ネイティブ、macOS など様々な OS を考慮して」と決めた)。macOS・Windows・WSL では、OS に元からある仕組みだけを使う
- Acceptance:
  - Given: macOS
  - When: 1行を鳴らす
  - Then: macOS の通知の仕組みに、題と文が1回渡る。
- Acceptance:
  - Given: WSL の中(`WSL_DISTRO_NAME` があるか、カーネルの版の名前に `microsoft` を含む)
  - When: 1行を鳴らす
  - Then: Linux の通知の仕組みを使わず、Windows の通知の仕組みに題と文が1回渡る。
- Acceptance:
  - Given: Windows
  - When: 1行を鳴らす
  - Then: Windows の通知の仕組みに、題と文が1回渡る。
- Acceptance:
  - Given: Linux で、デスクトップの通知を受ける所に届かない
  - When: 1行を鳴らす
  - Then: notify.fail のとおり、届けられなかったとして扱う。
- Needs: utest

#### `"command"` は、項目の `command`(プログラムと引数の並び)を、シェルを通さずに動かし、引数の中の `{title}`・`{text}`・`{path}` を、知らせの題・文・行のファイルのパスに置き換えるべき
`req~notify.command~1` P1
- Rationale: 電話への中継や自分の道具など、mdgrid の知らない出す先へ渡せるようにするため。シェルを通さないので、行の値に `$(…)` や `;` があっても命令として動かない
- Acceptance:
  - Given: `command = ["<受けた引数をファイルに書くプログラム>", "{title}", "{text}"]` の項目
  - When: 題が「リマインダー」、文が「請求書を送る」の行を鳴らす
  - Then: プログラムが、「リマインダー」と「請求書を送る」の2つの引数で1回動く。
- Acceptance:
  - Given: `text = "{title}"`・`command = ["<受けた引数をファイルに書くプログラム>", "{text}"]` の項目と、title が `$(touch made)` のノート
  - When: そのノートを鳴らす
  - Then: 引数は `$(touch made)` の文字のままで、made のファイルはできない。
- Acceptance:
  - Given: `to` に `"command"` があり、`command` が無いか空の項目
  - When: 起動する
  - Then: 警告して、その出す先を使わない。
- Needs: utest

#### 知らせの題と文は、行の値の制御文字(改行・ESC・BEL など)を空白に替えてから出すべき
`req~notify.sanitize~1` P1
- Rationale: ノートの値に書かれたエスケープの並びが端末の合図に混ざって、端末を操ったり(クリップボードを書く OSC 52 など)、知らせの文を崩したりしないようにするため。人から受け取ったノートでも、安心して鳴らせるようにする
- Acceptance:
  - Given: title に ESC・`]52;c;aGk=`・BEL を続けて含むノート
  - When: `"terminal"` で鳴らす
  - Then: 端末への出力の ESC は通知の合図の枠の分だけで、OSC 52 の合図は無い。
- Acceptance:
  - Given: `text = "{title}"` の項目と、title が「請求書」・改行・「を送る」のノート
  - When: そのノートを鳴らす
  - Then: 文は「請求書 を送る」。
- Needs: utest

### 説明書

#### 説明書に、`--notify` を OS の定期実行から呼ぶ例を、Linux(systemd のタイマー)・macOS(launchd)・Windows(タスク スケジューラ)・WSL(Windows のタスク スケジューラから `wsl.exe`)ごとに載せ、cron から呼ぶと OS の通知が届かないことがある理由と直し方を書くべき
`req~notify.schedule-docs~1` P1
- Rationale: 閉じているあいだの知らせは、人が OS の定期実行に登録して初めて動くため(登録を mdgrid がするのは次の段)。WSL の中の cron は WSL が動いているあいだしか動かないので、Windows の側から呼ぶ
- Acceptance:
  - Given: 説明書の日本語と英語の版
  - When:
  - Then: 4つの OS の登録の例が写して動く形であり、WSL の例は窓を出さずに動き、cron の理由(デスクトップの通知の宛先の環境変数が無いこと)と直し方がある(読んで判定)。

## 未確定事項

### 設定の `[[notify]]` と旗 `--notify` の名前
`tbd~notify.names`
- Decider: 人
- Provisional: `[[notify]]` と `--notify`(通知の欄の `[[notices]]`・`--notices` と1字違いなので、取り違えるなら `[[alerts]]`・`--alert` などに替える)

### `scope` を書かないときの範囲
`tbd~notify.default-scope`
- Decider: 人
- Provisional: `"workspace"`(通知の欄の既定 `"table"` と違う。どの表を開いていても知らせを届けるため)

### `to` を書かないときの出す先
`tbd~notify.default-to`
- Decider: 人
- Provisional: `["desktop"]`

### まとめる行の数の既定
`tbd~notify.group-default`
- Decider: 人
- Provisional: 3(4行から1件にまとめる)

### 開いている間に確かめる間隔
`tbd~notify.interval`
- Decider: 人
- Provisional: 1分

### 出す先を打ち切る時間
`tbd~notify.timeout`
- Decider: 人
- Provisional: 10 秒

### CSV・TSV の表の行の見分け方
`tbd~notify.csv-row-id`
- Decider: 人
- Provisional: 行の値の並び(値を直すと別の行になり、まだ当たっていればもう1回鳴る)。行の番号で見分けると、上の行を消したときに下の行を取り違える

### 端末の合図を人が選び直せるようにするか
`tbd~notify.terminal-pick`
- Decider: 人
- Provisional: 選べない(見分けるだけ)。見分けを誤る端末が出たら、項目に合図を選ぶ鍵を足す
