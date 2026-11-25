# ルーチンの回を作る(routine) の要求

## 目的

ルーチン(毎週のレビュー・毎月の請求のような繰り返しの仕事)のノートに書いた決まりから、その回のノート(「週次レビュー 2026-11-23」)を、ルーチンの表とは別の表に1つずつ作る。回のノートは普通のノートなので、表・カンバン・ガント・関係マップでそのまま扱え、済んだかどうかも回ごとに残る。mdgrid を開いている間は画面が確かめて作るかを聞き、閉じているあいだは OS の定期実行から呼ぶ `mdgrid --runs` が確かめて作る。動き続ける見張りのプロセスは置かない(notify と同じ)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-24 に足した。

## 範囲外

- 採らない: 抜けた回を全部作ること。しばらく開かず定期実行も走らなかったときは、一番新しい回だけを作る(2026-11-24 に人が「一番新しい回だけ」と決めた。routine.latest-only)。
- 採らない: 常駐するプロセス。開いている間は画面が、閉じているあいだは OS の定期実行が確かめる(notify と同じ)。
- 次の段: OS の定期実行への登録を mdgrid がすること。今は説明書に OS ごとの例を載せ(routine.schedule-docs)、登録は人がする。
- 次の段: 回を済ませたときに、ルーチンのノートの最後にやった日(`last_done`)と次にやる日(`next_due`)を書き換えること(tbd~routine.next-due)。
- 条件に当たった行を知らせるのは notify。回のノートを作るのはこの能力だけで、notify はノートを書かない。

## 要求

### 項目を書く

#### 印の `[[table]]` に `runs = { folder = "<フォルダ>", rule = "<列>" }` を書いた表だけを、ルーチンの表として回を作り、書いていない表では回を作らず、作った回の記録も作らないべき
`req~routine.opt-in~1` P1
- Rationale: 要らない人のフォルダにノートを足さず、状態の置き場にも何も足さないため。どの表がルーチンで、回をどこに作るかは、人が印に書いて決める
- Acceptance:
  - Given: `runs` の無い印と、`recurrence: daily` のノートがある表
  - When: 画面で開く
  - Then: 回を作るという知らせは出ず、ノートのフォルダのファイルの数とバイトは変わらず、状態の置き場に回の記録のファイルが無い。
- Acceptance:
  - Given: 表「ルーチン」に `runs = { folder = "Runs", rule = "recurrence" }` を書いた印
  - When: `mdgrid <台帳> --runs`
  - Then: `Runs/` に、決まりの読めるルーチンの回のノートを作る。
- Acceptance:
  - Given: `runs` に `folder` か `rule` の無い印
  - When: 起動する
  - Then: cli.config-file-warning-form の形で警告し、その表では回を作らない。
- Needs: utest

#### 決まりの列の値は、`daily`(毎日)・`weekly <曜日…>`(`weekly mon thu` のように並べてよい)・`monthly <日…>`(`monthly 1 15`。月の末日は `last`)・`every <N>d`(起点の列 `start` の日から N 日ごと)で読み、日本語の `毎日`・`毎週 月 木`・`毎月 1日 15日`・`毎月 末日`・`N日ごと` も同じに読むべき
`req~routine.rule~1` P1
- Rationale: 今のルーチンの「繰り返し」は自由な文で、機械では次の回が分からないため。英語の短い形と日本語の形のどちらで書いても同じ回にする(書き方は tbd~routine.rule-syntax)
- Acceptance:
  - Given: 今日が 2026-11-24(火)で、`recurrence: weekly mon` のルーチン
  - When: 回の日を決める
  - Then: 2026-11-23(月)。
- Acceptance:
  - Given: 今日が 2026-11-24 で、`recurrence: 毎月 末日` のルーチン
  - When: 回の日を決める
  - Then: 2026-10-31(11月の末日の 11-30 はまだ来ていない)。
- Acceptance:
  - Given: 今日が 2026-11-24 で、`recurrence: monthly 31` のルーチン
  - When: 回の日を決める
  - Then: 2026-10-31(31 日の無い月は、その月には回が無い)。
- Acceptance:
  - Given: 今日が 2026-11-24 で、`start: 2026-11-20`・`recurrence: 3日ごと` のルーチン
  - When: 回の日を決める
  - Then: 2026-11-23(11-20・11-23・11-26 の並び)。
- Acceptance:
  - Given: 今日が 2026-11-24 で、`recurrence: daily` のルーチン
  - When: 回の日を決める
  - Then: 2026-11-24。
- Needs: utest

#### 決まりの読めないルーチン(知らない形・`every` で起点の無いもの・空の値)は回を作らず、ノートの名前と値を添えた理由を1行出し、ほかのルーチンの回は作るべき
`req~routine.rule-unreadable~1` P1
- Rationale: 1つの書き誤りで、ほかのルーチンの回まで止めないため。黙って飛ばすと、人は回が来ないことに気づけない
- Acceptance:
  - Given: `recurrence: 毎週かも` の「掃除」と、`recurrence: daily` の「日記」
  - When: `mdgrid <台帳> --runs`
  - Then: 標準エラーに「掃除」と `毎週かも` を含む1行が出て、「日記」の回は作る。
- Acceptance:
  - Given: 決まりの列が空のルーチン
  - When: `--runs`
  - Then: そのルーチンでは回を作らず、理由は出さない(決まりを書いていないルーチンは、回を作らないルーチンとみなす)。
- Needs: utest

### 回のノート

#### 回のノートは `runs` の `folder` に、名前 `name`(書かなければ `{routine} {date:YYYY-MM-DD}`)で作り、フロントマターに、ルーチンへのリンク `routine`(relations.link-forms の `[[…]]` の形)・回の日 `date`・`set` に書いた値・`copy` に並べた列のルーチンの値を書き、本文は `body` の雛形(書かなければ見出し1行)で始めるべき
`req~routine.run-note~1` P1
- Rationale: 回のノートが、どのルーチンのどの日の回かをノートだけで言え、表・関係マップでルーチンとつながるようにするため。名前と本文の雛形の変数は新しいノート(cell-edit.new-note-config-name)と同じにする
- Acceptance:
  - Given: `runs = { folder = "Runs", rule = "recurrence", set = { status = "todo" }, copy = ["actor"] }` と、`recurrence: weekly mon`・`actor: both` の「週次レビュー」、今日が 2026-11-24
  - When: 回を作る
  - Then: `Runs/週次レビュー 2026-11-23.md` ができ、フロントマターは `routine: "[[週次レビュー]]"`・`date: 2026-11-23`・`status: todo`・`actor: both`、本文は `# 週次レビュー 2026-11-23`。
- Acceptance:
  - Given: `name = "{date:YYYYMMDD}_{routine}"` の `runs`
  - When: 同じ回を作る
  - Then: 名前は `20261123_週次レビュー.md`。
- Acceptance:
  - Given: `folder` のフォルダがまだ無い
  - When: 回を作る
  - Then: フォルダを作ってから、その中に回のノートを作る。
- Needs: utest

#### 前に作った回のあとに来るはずだった回がいくつあっても、今日までの一番新しい回だけを作り、今日より後の回は作らないべき
`req~routine.latest-only~1` P1
- Rationale: しばらく開かず、定期実行も走らなかったあとに、済ませようのない古い回のノートが一度にたまらないようにするため(2026-11-24 に人が「一番新しい回だけ」と決めた)
- Acceptance:
  - Given: `recurrence: daily` のルーチンで、最後に作った回が 2026-11-20、今日が 2026-11-24
  - When: 回を作る
  - Then: 2026-11-24 の回だけができ、11-21・11-22・11-23 の回は無い。
- Acceptance:
  - Given: 回を作ったことの無いルーチン(`runs` を初めて書いた)と、`recurrence: weekly mon`、今日が 2026-11-24
  - When: 回を作る
  - Then: 2026-11-23 の回を1つ作る(tbd~routine.first-run)。
- Acceptance:
  - Given: `recurrence: weekly fri` のルーチンで、2026-11-20 の回は作ってある、今日が 2026-11-24
  - When: 回を作る
  - Then: 何も作らない(次の回の 11-27 はまだ来ていない)。
- Needs: utest

#### 同じルーチンの同じ日の回は1つだけ作り、同じ名前のノートが既にあれば作らずに上書きもせず、作った記録のある回は、そのノートが消えていても作り直さないべき
`req~routine.once~1` P1
- Rationale: 画面と定期実行の両方が確かめても、回のノートを二重に作らず、人のノートも上書きしないため。要らない回のノートを消したら、それで済んだことにする(tbd~routine.deleted-run)
- Acceptance:
  - Given: 2026-11-23 の「週次レビュー」の回を作ったあと
  - When: もう一度回を作る
  - Then: 何も作らず、`Runs/` のファイルの数とバイトは変わらない。
- Acceptance:
  - Given: 人が先に手で作った `Runs/週次レビュー 2026-11-23.md`
  - When: その回を作る
  - Then: そのノートのバイトは変わらず、作った回と記録して、次から作らない。
- Acceptance:
  - Given: 作った回のノートを人が消した
  - When: 同じ日のうちに、もう一度回を作る
  - Then: 作り直さない。
- Acceptance:
  - Given: 作る回のある台帳
  - When: `--runs` を2つ同時に走らせる
  - Then: 回のノートは1つだけできる。
- Needs: utest・stest

#### 作った回の記録は、見た目の状態の置き場(screen.state-store-key と同じ置き場)に持ってノート・`.base`・設定には書かず、開いている画面と `--runs` で同じ記録を使うべき
`req~routine.state~1` P1
- Rationale: 作ったかどうかをルーチンのノートに書くと、ノートの差分や同期が騒がしくなるため(notify.state と同じ考え)
- Acceptance:
  - Given: 開いている画面で 2026-11-23 の回を作った
  - When: 続けて `--runs`
  - Then: その回を作らない。
- Acceptance:
  - Given: 作る回の無い台帳
  - When: `--runs`
  - Then: ルーチンのノートのバイトは変わらない。
- Acceptance:
  - Given: 記録のファイルの中身が壊れている
  - When: `--runs`
  - Then: 記録を作り直した旨の1行を標準エラーに出し、同じ名前のノートがある回は作らず(routine.once)、ほかの回は作る。
- Needs: utest

### 開いている間

#### 開いたときと、開いている間に日付が変わったときに、作る回があるかを確かめ、あれば画面の下の帯に回の数と作る操作を出し、人が作る操作をするまではノートを書かないべき
`req~routine.screen-offer~1` P1
- Rationale: 開いただけでノートが増えると、書く前に差分で確かめる決まり(write-back)を破るため。開いている間に日付が変わったら、その日の回が来たことを知らせる(2026-11-24 に人が「開いたとき」と「OS の定期実行」の両方で作ると決めた)
- Acceptance:
  - Given: 作る回が2つある台帳
  - When: 画面で開く
  - Then: 下の帯に「ルーチンの回が 2 件来ています」と作る操作が出て、`Runs/` のファイルの数は変わらない。
- Acceptance:
  - Given: 2026-11-24 の 23:59 に開いた画面と、`recurrence: daily` のルーチン(11-24 の回は作ってある)
  - When: 日付が 11-25 に変わる
  - Then: 1分以内に、下の帯に「ルーチンの回が 1 件来ています」が出る。
- Acceptance:
  - Given: 作る回の無い台帳
  - When: 画面で開く
  - Then: 下の帯に回の知らせは出ない。
- Needs: utest

#### 作る操作では、その時に来ている回を数え直して、作るノートの名前を並べて見せ、確かめてから作り、やめたら何も作らず、その日のうちは同じ回の下の帯の知らせを出し直さない(人が自分で選んだパレットの作る操作では並べる)べき
`req~routine.screen-create~2` P1
- Rationale: 何ができるかを人が見てから書くため。やめた回を下の帯で何度も知らせて邪魔をしない。ただし、人が自分でパレットから選んだときまで「来ている回は無い」と返すと、確かめの欄で Enter だけを押してやめた人が、その日は画面から作れなくなる。開いたときに数えた結果を使い回すと、足したばかりのルーチンの回も出ない(2026-11-25 の実地の操作で見つかった)。作る操作はパレットからも選べる(キーは tbd~routine.offer-key)
- Acceptance:
  - Given: 下の帯に「ルーチンの回が 2 件来ています」が出ている
  - When: 作る操作をして、確かめる
  - Then: 並べた2つの名前のノートができ、表を読み直して、回の表にその2行が出る。
- Acceptance:
  - Given: 作るノートの名前の一覧が出ている
  - When: やめる
  - Then: `Runs/` のファイルの数は変わらず、その日のうちは下の帯の知らせが出ない。次の日に来た回は、また知らせる。
- Acceptance:
  - Given: その日にやめたあと
  - When: パレットの「ルーチンの回を作る」を選ぶ
  - Then: やめた回の名前が並び、確かめると作る。
- Acceptance:
  - Given: 開いたあとに、ルーチンの表で `recurrence: daily` の「散歩」を足して保存した
  - When: パレットの「ルーチンの回を作る」を選ぶ
  - Then: 「散歩」の今日の回の名前が並ぶ。
- Needs: utest

#### 読むだけの起動(cli.readonly)では、回の知らせを出さず、回を作らず、`--runs` は理由1行と終了コード 2 で何も書かないべき
`req~routine.readonly~1` P1
- Rationale: 読むだけの起動は何も書かない約束のため
- Acceptance:
  - Given: 作る回のある台帳
  - When: `--readonly` で開く
  - Then: 下の帯に回の知らせは出ず、パレットに作る操作が無い。
- Acceptance:
  - Given: 作る回のある台帳
  - When: `mdgrid <台帳> --runs --readonly`
  - Then: 理由1行と終了コード 2 で、ノートのフォルダのファイルの数は変わらない。
- Needs: utest・stest

#### 式で、決まりの文字の値に `nextRun()`(起点の日を渡してよい: `recurrence.nextRun(start)`)を付けると、今日以後の一番近い回の日(今日が回の日なら今日)を日付で返し、読めない決まり・空の値・`every` で起点が無いときは空を返すべき
`req~routine.next-run~1` P1
- Rationale: ルーチンの表で、次の回の日を、人が `next_due` を書き直さなくても見せるため(繰り返しの仕事を扱う道具は、次の予定日を出すのが当たり前)。計算の列なので、ノートには書かない(tbd~routine.next-due)
- Acceptance:
  - Given: 今日が 2026-11-24(火)で、`recurrence: weekly mon` のルーチン
  - When: 式の列 `recurrence.nextRun()` を見る
  - Then: 2026-11-30。
- Acceptance:
  - Given: 今日が 2026-11-24(火)で、`recurrence: 毎週 火` のルーチン
  - When: 同じ式の列を見る
  - Then: 2026-11-24(今日)。
- Acceptance:
  - Given: 今日が 2026-11-24 で、`start: 2026-11-20`・`recurrence: 3日ごと` のルーチン
  - When: 式の列 `recurrence.nextRun(start)` を見る
  - Then: 2026-11-26。
- Acceptance:
  - Given: `recurrence: 毎週かも` のルーチン
  - When: 式の列 `recurrence.nextRun()` を見る
  - Then: セルは空で、`--print` の値も空。
- Needs: utest

### 閉じているあいだ

#### `mdgrid [<フォルダ> | -w <名前>] --runs` は、画面を出さずに、作る回を全部確かめずに作り、作ったノートのパスを標準出力に1行ずつ出して、終了コード 0 で終わるべき
`req~routine.cli~1` P1
- Rationale: mdgrid を閉じているあいだも、OS の定期実行(systemd のタイマー・launchd・タスク スケジューラ・cron)から呼んで回を作るため。定期実行に登録したことを、書いてよいという人の同意とみなす(2026-11-24 に決めた)
- Acceptance:
  - Given: 作る回が1つある台帳
  - When: `mdgrid <台帳> --runs`
  - Then: 回のノートができ、標準出力はそのパスの1行で、終了コード 0。
- Acceptance:
  - Given: 作る回の無い台帳か、`runs` の無い印
  - When: `--runs`
  - Then: 何も書かず、標準出力は空で、終了コード 0。
- Acceptance:
  - Given: `-w` に、無いワークスペースの名前
  - When: `--runs`
  - Then: cli.fail-fast と同じく、理由1行と終了コード 2。
- Needs: stest

#### 作れなかった回(フォルダに書けない・名前に使えない字が入るなど)は、`--runs` は理由を標準エラーに1行ずつ出して終了コード 1 で終わり、開いている間は下の帯に理由を出し、どちらも作れなかった回は記録せずに次の確かめでまた作ろうとし、ほかの回は作るべき
`req~routine.fail~1` P1
- Rationale: 黙って落とすと回を逃すため。1つの回が作れなくても、ほかの回は止めない
- Acceptance:
  - Given: 書けない `Runs/` のフォルダと、作る回が1つ
  - When: `--runs`
  - Then: 標準エラーにフォルダを含む1行が出て、終了コード 1。書けるようにしてから `--runs` すると、その回を作る。
- Acceptance:
  - Given: 2つのルーチンの表のうち1つの `folder` だけが書けない
  - When: `--runs`
  - Then: 書ける方の回は作り、終了コード 1。
- Needs: utest・stest

### 説明書

#### 説明書に、`--runs` を OS の定期実行から呼ぶ例を、notify.schedule-docs と同じ4つの OS ごとに載せ、`--notify` と1つの登録で続けて呼ぶ例と、決まりの書き方の一覧を載せるべき
`req~routine.schedule-docs~1` P1
- Rationale: 閉じているあいだの回は、人が OS の定期実行に登録して初めてできるため(登録を mdgrid がするのは次の段)
- Acceptance:
  - Given: 説明書の日本語と英語の版
  - When:
  - Then: 4つの OS の登録の例が写して動く形であり、`--runs` と `--notify` を続けて呼ぶ例と、routine.rule の書き方の表がある(読んで判定)。

## 未確定事項

### 設定の `runs` と旗 `--runs` の名前
`tbd~routine.names`
- Decider: 人
- Provisional: `runs` と `--runs`、式の `nextRun()`(回を英語で run と呼ぶ)

### 決まりの書き方(英語の短い形と日本語の形の範囲)
`tbd~routine.rule-syntax`
- Decider: 人
- Provisional: routine.rule の形だけ。隔週(`every 2w`)・第2火曜(`monthly 2nd tue`)は次の段

### 回を作ったことの無いルーチンで、今の回を作るか
`tbd~routine.first-run`
- Decider: 人
- Provisional: 作る(notify.first-silent と違い、回はまだ済ませていない仕事なので、黙って覚えるだけにしない)

### 人が消した回を作り直すか
`tbd~routine.deleted-run`
- Decider: 人
- Provisional: 作り直さない(消したことを、要らない回だと人が決めたとみなす)

### 画面で回を作る操作のキー
`tbd~routine.offer-key`
- Decider: 人
- Provisional: 下の帯の知らせのクリックと、パレットの「ルーチンの回を作る」。キーは割り当てない

### 回が済んだら、ルーチンの `last_done` と `next_due` を書き換えるか
`tbd~routine.next-due`
- Decider: 人
- Provisional: 書き換えない(次の段)。次の回の日は、式の `nextRun()`(routine.next-run)の計算の列で見せる
