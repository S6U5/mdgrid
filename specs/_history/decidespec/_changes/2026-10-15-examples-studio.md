---
type: Change
id: 01M4XQ3M21N7F78YPKYREGXZJ9
title: 見本を架空の制作会社 Northwind Studio の英語のデータに作り直す(examples-studio)
status: done
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 見本を架空の制作会社 Northwind Studio の英語のデータに作り直す(examples-studio)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | 錠の掛け直しの決定 [2026-10-15-examples-studio-locks](../_decisions/2026-10-15-examples-studio-locks.md)(要件の文は変えない) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-15、要約): mdgrid の見本を、しっかりしたリレーションがあり、見て納得できる題材に作り直す。relations だけでなく全部の見本を作り直す。題材は「小さな制作会社」、言語は英語。古い見本は消してよく、見本の試験の錠の掛け直しも承認した。

終わりの条件: 見本はどれも同じ架空の制作会社のもので、リンクでつながった表(多対一と多対多、表の中のつながり)が見られる。見本を確かめる試験が新しい見本で通り、どのリンクにも行き先のノートがある。README・説明書・画面の写し・録画の台本・try.sh が新しい見本を指す。

## 不明点と仮定

- 仮定: 見本の根に README.md を置くとノートの行に数えられるので、見本の説明は examples/README.md に1つにまとめる(ai-human は .base が Tasks/ だけを選ぶので、今までどおり README.md を置く)。
- 仮定: 案件の状態のキーは `stage` にし、タスクの `status` と分ける。同じルートフォルダで同じキーを使うと、タスクの status の候補に案件の値(active・planning)が混ざるため。
- 仮定: 画面の写しの今日は 2026-10-06 に固定する(MDGRID_TODAY)。

## 設計

- `examples/studio/`: Tasks 20・Projects 4・Clients 3・People 5・Meetings 7。つながりは Tasks.project・owner(多対一)、Tasks.depends_on(タスクどうし、多対多)、Projects.client・lead(多対一)、Projects.team(多対多)、Clients.account_manager、Meetings.project(多対一)・attendees(多対多)。`.mdgrid/workspace.toml` に5つの表(Tasks は Tasks.base)、`.obsidian/types.json`、`Tasks.base`(Open・This week・By status・By project・By owner・Dependencies・Workload)、`equipment.csv`。
- `examples/quirks/`: 旧 vault の扱いにくいノートを、同じ会社の雑用で作り直す(Tasks 12・Notes 5、Tasks.base の5つのビュー)。
- `examples/configs/studio.toml`: 旧 showcase の設定の後継(日付の形・週の始まり・表示・新しいノート・決まり・値の色・キー)。
- `examples/ai-human/`: 英語だけにし、`Tasks.base` と `Tasks/` を見本の根に置く。
- 消す: examples/vault・showcase・demo・relations・schedule。
- 試験: tests/test_examples.rs を新しい見本で書き直す(錠を掛け直す)。新しいファイル tests/test_example_studio.rs で、印の表・つながり・全部のリンクの行き先・つながった行を確かめる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 見本のデータと試験 | REL-1, REL-5, REL-6, WS-7(見本), CE-16, WB-3, WB-5, BV-3, BV-7, CE-22, CLI-3(錠) | 上の終わりの条件のうち、見本と試験 | examples/, tests/test_examples.rs, tests/test_example_studio.rs, tests/test_example_ai_human.rs, tests/test_apply_review.rs | test_examples_*(tests/test_examples.rs)、test_example_studio_*、test_example_ai_human_*、test_cli_17_round_trip_sample_vaults_writes_nothing | 済 |
| 2 | 文書・画面の写し・録画の台本 | なし(文書) | README・説明書(英・日)・画面の写しの台本と一覧・README の画像と GIF・録画の台本・try.sh・Pages の入口が新しい見本を指す | README.md, README.ja.md, docs/manual/, docs/manual-scenarios.toml, docs/assets/, demos/, scripts/pages.py, scripts/readme-hero.sh, scripts/demo-gif.sh | 読んで判定(撮った画面と録画のコマを見る) | 済 |

## 実装の気づき

- `.base` の order に file.* の列を入れると、左のノートの欄と重なる。Tasks.base には入れない。
- リンクの列でまとめた見出しと `%` の件数の表は、リンクを名前でなく `[[…]]` で出す。README の画像は By status のまとまりで撮った(本体の不具合は別の作業に出した)。
- `.base` の表からリンクの行き先を開くと「この表にあるが絞り込みで隠れている」と出る(.base の表をルートフォルダとみなすため)。`-w` で一部のフォルダのワークスペースを開くと、関係マップに余分な表が出る。どちらも本体の不具合として別の作業に出し、録画の places はフォルダの表で撮る形にした。
- タスク 1(見本と試験・錠)とタスク 2(文書と画像)は、コミットを分けた。
- 書き直した試験が rustfmt の形になっていなかった(cargo fmt --check が落ちる)。整形して、錠を [2026-10-15-examples-fmt-locks](../_decisions/2026-10-15-examples-fmt-locks.md)(AI 自己採択、錨との照合あり)で掛け直した。
- scripts/readme-hero.sh・demo-gif.sh の設定が古い名前(`nerd_font`・`theme`・`[style]`)で、画像の下に警告が写っていたので、新しい名前にした。

## 照合

自分で照合(見本と文書の作り直し。要件の文は変えない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| REL-1 | checked | テスト: tests/test_example_studio.rs::test_example_studio_every_link_resolves(studio の全部のリンクを parse して resolve) / 今: 通った / 前: 見本が無い / 変異: owner を `[[Sam Ortis]]` にすると落ちる |
| REL-5 | checked | テスト: tests/test_example_studio.rs::test_example_studio_linked_rows_of_a_project(Clinic Booking App を指す Tasks 7・Meetings 2、列は project) / 今: 通った / 前: 見本が無い |
| REL-6 | checked | テスト: tests/test_example_studio.rs::test_example_studio_edges(9つのつながりと多対多の印) / 今: 通った / 前: 見本が無い |
| WS-7 | checked | テスト: tests/test_example_studio.rs::test_example_studio_marker_tables(印の5つの表、Tasks は .base) / 今: 通った / 前: 見本が無い |
| CE-16 | checked | テスト: tests/test_examples.rs::test_examples_open_folder・test_examples_showcase(1行と縦のリストが編集でき、候補が件数の順) / 今: 通った / 前: 旧 vault・showcase で同じ確かめ方 |
| WB-3 | checked | テスト: tests/test_examples.rs::test_examples_open_folder(Notes/Snack list・Notes/Blank が lock なし) / 今: 通った / 前: 旧 vault の 買い物メモ で同じ確かめ方 |
| WB-5 | checked | テスト: tests/test_examples.rs::test_examples_open_folder(Notes/Weekly review が読むだけ) / 今: 通った / 前: 旧 vault の 週の振り返り で同じ確かめ方 |
| BV-3 | checked | テスト: tests/test_examples.rs::test_examples_base_views(操作の前後で quirks/Tasks.base のバイトが同じ) / 今: 通った / 前: 旧 vault で同じ確かめ方 |
| BV-7 | checked | テスト: tests/test_examples.rs::test_examples_base_views(Unsupported の列が全行 Unsupported、notes に理由) / 今: 通った / 前: 旧 vault で同じ確かめ方 |
| CE-22 | checked | テスト: tests/test_examples.rs::test_examples_showcase(configs/studio.toml の日付の形・週の始まり) / 今: 通った / 前: 旧 showcase の設定で同じ確かめ方 |
| CLI-3 | checked | テスト: tests/test_examples.rs::test_examples_showcase(configs/studio.toml が警告なく読める) / 今: 通った / 前: 旧 showcase の設定で同じ確かめ方 |

要件の無い変更(見本)の照合: checked。見本: examples/studio・quirks・configs・ai-human・places・themes / テスト: tests/test_examples.rs の3本(錠を掛け直した)、tests/test_example_studio.rs の4本、tests/test_example_ai_human.rs の3本、tests/test_apply_review.rs の見本の往復 / 今: 全部 passed / 前: 新しい見本は無い / 変異: studio のタスクの owner を `[[Sam Ortis]]`(無い人)にすると test_example_studio_every_link_resolves が落ちる。タスク2: docs/manual-scenarios.toml の95場面を app-manual の tui_shot で英・日に撮り、失敗なし。テキストの写しを見て、キーの動き(日付・チェック・数・リストの列、quirks の場面)が狙いの場所に当たることを確かめた。録画の台本 20本を vhs で撮り、コマを並べて見た。README の画像・見出しの画像・GIF を作り直して見た

既存のテストの削除・skip・弱体化: なし(tests/test_examples.rs は同じ要件を同じ強さで新しい見本に当てて書き直した。決定記録に前後の表)

確かめた: 11 / 11
