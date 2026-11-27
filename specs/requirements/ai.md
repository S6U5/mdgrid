# AI との接続(ai) の要求

## 目的

人のほかに、AI(端末で動くエージェントや、手元につないだチャットの道具)も台帳の値を直す。AI の直しは、台帳で許した列だけを直に書き、ほかは「案」として台帳に置き、人が画面で差分を見て受ける。共有のフォルダ(クラウドの同期・git)で何人かと AI が同じ台帳を書いても、だれかの直しを黙って消さないこと(write-back.merge-other-keys・cli.apply-stale-row)と、AI に許していない値を人の目を通さずに変えないことを芯にする。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-28 に人が選んだ4つ(書き戻しの3者比較・案を画面で受ける・MCP の口・mdgrid から AI を呼ぶ)と、AI の書き込みの既定(許した列だけ直に書く)から始めた。

## 範囲外

- 採らない: 何人かが同時に同じ画面を開いて直す共同編集と、行の錠のサーバー(ファイルで持つ道具の範囲を超える)。
- 採らない: AI の直しをだれが出したかの名前(ログイン名)を案に書くこと(共有の台帳に個人の名前を残さないため。理由は案の `note` に書く)。

## 要求

### 案

#### `mdgrid [<表>] --apply <ファイル> --propose <理由>` は、`--apply` と同じ入力を読んで同じ検査(cli.apply-reject-all・cli.apply-stale-row)を通し、今と違うセルのうち AI が直に書いてよい列(ai.write-columns)のセルを画面の保存と同じ道で書き、ほかのセルは書かずに案(ai.proposal-file)として台帳に置くべき
`req~ai.propose-command~1` P1
- Rationale: AI が `--apply --yes` を使うと、許していない列まで人の目を通さずに書く。`--yes` を禁じるだけだと、AI は差分を人に見せて待つしかなく、人がいないと進まない。許した列は進め、残りは人があとで画面で受けられるようにする
- Acceptance:
  - Given: 印に `[ai] write = ["status"]` を書いた台帳の、`status: todo`・`due: 2026-12-01` のノート
  - When: `--print --with-path` の CSV の status を done、due を 2026-12-20 に直して `--apply <その CSV> --propose "締め切りを見直す"`
  - Then: ノートの status は done になり、due は 2026-12-01 のまま。台帳の `.mdgrid/proposals/` に、due を 2026-12-20 にする案のファイルが1つ。終了コード 0 で、標準出力に書いたセルの数・案にしたセルの数・案のファイルの名前。
- Acceptance:
  - Given: `[ai]` の無い台帳
  - When: 同じく status と due を直して `--propose`
  - Then: ノートは変わらず、status と due の両方が1つの案になる。
- Acceptance:
  - Given: 書き出したあとに外で due を直したノート
  - When: 書き出した CSV の status を直して `--propose`
  - Then: 何も書かず、案も置かず、終了コード 2(cli.apply-stale-row)。
- Acceptance:
  - Given: 書き出したまま直していない CSV
  - When: `--propose`
  - Then: 何も書かず、案も置かず、終了コード 0。
- Acceptance:
  - Given: `--propose` に `--yes` を添える、または `--apply` の無い `--propose`
  - When: 起動する
  - Then: 理由1行と終了コード 2(cli.fail-fast)。
- Needs: stest

#### 案は、台帳の根(印 `.mdgrid/workspace.toml` のあるフォルダ。無ければ開いたフォルダ)の `.mdgrid/proposals/` に、1つの案を1つの JSON のファイルにして置き、ほかの案のファイルを置き換えてはならない
`req~ai.proposal-file~1` P1
- Rationale: 同期の道具は同じファイルへの同時の書き込みを競合にする。案ごとに別のファイルにすれば、何台の機械から置いても重ならない
- Acceptance:
  - Given: 台帳
  - When: 別の直しで `--propose` を2回
  - Then: `.mdgrid/proposals/` に2つの `.json`。どちらも、`note`(理由)・`created`(作った日時 `YYYY-MM-DDTHH:MM`)・`changes`(`path`(台帳の根からのパス)・`set`(列 → 値)・`was`(案を作ったときの、その列の値))を持つ。
- Needs: stest

#### 設定 `ai.write`(列の名前の並び。既定は空)に書いた列だけを、AI の書き込み(ai.propose-command と MCP の口)で直に書くべき
`req~ai.write-columns~1` P1
- Rationale: 状態や進み具合のように、AI に任せてよい列は台帳ごとに違う。既定では何も直に書かず、台帳の持ち主が選んだ列だけを任せる
- Acceptance:
  - Given: 全体の設定に `[ai] write = ["status"]`、台帳の印の表に `[table.ai] write = ["status", "tags"]`
  - When: その表で `--print-config --resolved`
  - Then: `ai.write` が `["status", "tags"]` で、出どころが表の範囲。
- Acceptance:
  - Given: `[ai] write = ["status"]`
  - When: ビューに無い列(`priority`)を直した CSV で `--propose`
  - Then: priority は案になる(書かない)。
- Needs: stest

#### 画面は、開いた表の行を指す案があれば、下の帯に案の数を出し、押すかコマンドのパレットの「AI の案を見る」で案を並べるべき
`req~ai.proposal-notice~1` P1
- Acceptance:
  - Given: 開いた表の2つのノートを指す1つの案と、ほかの表のノートだけを指す案
  - When: 表を開く
  - Then: 下の帯に「AI の案 1件」。「AI の案を見る」で、その案の理由・作った日時・セルの数が1行に並ぶ。
- Acceptance:
  - Given: 表を開いているあいだに、同期で案のファイルが届いた
  - When: 次の見回り(`poll_ms`)
  - Then: 下の帯の案の数が増える。
- Needs: utest

#### 並べた案を選んで「入れる」と、案の値をためる変更にし(保存の確認で差分を見て書く)、案のファイルを消すべき。案を作ったときの値(`was`)と今の値が違うセルと、もう表に無い行のセルは入れず、その数を知らせるべき
`req~ai.proposal-take~1` P1
- Rationale: AI の案も人の直しと同じ流れ(ためる → 差分 → 保存)を通し、取り消しもできるようにする。案を作ったあとにほかの人が直したセルを案で戻さない
- Acceptance:
  - Given: due を 2026-12-20 にする案(`was` は 2026-12-01)
  - When: 案を選んで「入れる」
  - Then: due に 2026-12-20 のためる変更(未保存 1)。ノートのファイルはまだ変わらず、案のファイルは消える。
- Acceptance:
  - Given: 案を作ったあとに、外で due を 2026-12-05 に直した
  - When: 「入れる」
  - Then: due はためず、知らせに入れなかったセルの数 1。案のファイルは消える。
- Needs: utest

#### 並べた案を選んで「捨てる」と、案のファイルを消し、表とノートは変えないべき
`req~ai.proposal-discard~1` P1
- Acceptance:
  - Given: 1つの案
  - When: 選んで「捨てる」
  - Then: 案のファイルが消え、未保存は 0 のまま、下の帯の案の数が消える。
- Needs: utest

### スキル

#### mdgrid の CLI の手引きのスキルは、AI が値を直すときは `--apply <ファイル> --propose <理由>` を使い、直に書けなかったセルは人が画面で受けることを書き、`--yes` は人が頼んだときだけにする
`req~ai.skill-propose~1` P1
- Rationale: AI が `--yes` を自分で足さずに、許した列は進め、残りを人に渡せるようにするため
- Acceptance:
  - Given: SKILL.md
  - When: 読む
  - Then: `--propose` を使う手順と、案は人が画面の「AI の案を見る」で受けること、`--yes` は人が頼んだときだけであることがある。
- Needs: stest

## 未確定事項
