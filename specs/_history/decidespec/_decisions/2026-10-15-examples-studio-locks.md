---
type: Proposal
id: 01M4XNF61F52GJD6E08FJRHH47
title: 見本を架空の制作会社の英語のデータに作り直して変わる、見本の試験の錠を掛け直す(examples-studio-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: afb084fae6397389
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-15
touches: [CE-16, WB-3, WB-5, BV-3, BV-7, CE-22, CLI-3]
created: 2026-10-15
updated: 2026-10-15
---

# 見本を架空の制作会社の英語のデータに作り直して変わる、見本の試験の錠を掛け直す(examples-studio-locks)

## きっかけ

人が、見本(examples/)を、つながり(リレーション)がしっかりあって見て納得できる題材に全部作り直すよう頼み、題材を「小さな制作会社」、言語を英語に決めた(会話 2026-10-15)。見本はどれも架空の制作会社 Northwind Studio のものになった。

- `examples/studio`: タスク 20・案件 4・顧客 3・人 5・会議 7 の5つのフォルダを、フロントマターのリンクでつないだルートフォルダ。`Tasks.base`(7つのビュー)・ワークスペースの印・型の宣言・備品の CSV。旧 demo・relations・schedule の後継。
- `examples/quirks`: 扱いにくいノート(フロントマター無し・空・重複キー・同期の競合・型の合わない値・ネストした値)を集めた見本。旧 vault の後継。
- `examples/configs/studio.toml`: 多くの機能を有効にした設定。旧 showcase の後継。
- `examples/ai-human`: 英語の制作会社のタスクに作り直し、`Tasks.base` を見本の根に置いた。

見本を開く試験 `tests/test_examples.rs` は旧 vault と旧 showcase の中身(ノートの名前・数・列・ビューの名前・候補の件数)を確かめていたので、新しい見本の同じ形の中身を確かめるように書き直した。確かめる要件と強さは変えていない。

| 試験 | 前 | 後 |
|---|---|---|
| test_examples_open_folder | examples/vault をフォルダで開く(タスク 12・メモ 4)。null・空・キーなし、ネストした値・ブロックスカラー、縦と1行のリスト、重複キー、競合ファイル、フロントマター無し、tags の候補の件数 | examples/quirks をフォルダで開く(タスク 12・メモ 5)。同じ項目を同じ形のノートで確かめ、空のフロントマターのノート(Blank)も書けることを足した |
| test_examples_base_views | examples/vault/タスク.base の5つのビュー(進行中・状態ごと・期限・未対応の式・カード) | examples/quirks/Tasks.base の5つのビュー(Doing・By status・Due・Unsupported・Cards)。同じ確かめ方 |
| test_examples_showcase | examples/showcase の設定と プロジェクト.base | examples/configs/studio.toml と examples/studio(5つのフォルダと Tasks.base の7つのビュー)。設定の読み取り・リストの列の編集と候補・全部のビューの build・今日を固定したビューの行・式の値を確かめる |

## 差分

- 錠: CE-16(test_examples_open_folder・test_examples_showcase。見本のノートと列の名前が変わった)
- 錠: WB-3(test_examples_open_folder。同じ理由)
- 錠: WB-5(test_examples_open_folder。同じ理由)
- 錠: BV-3(test_examples_base_views。見本の .base とビューの名前が変わった)
- 錠: BV-7(test_examples_base_views。同じ理由)
- 錠: CE-22(test_examples_showcase。設定の見本の場所と見本のノートが変わった)
- 錠: CLI-3(test_examples_showcase。同じ理由)

## 却下した案

| 案 | 理由 |
|---|---|
| 旧 vault と showcase を残し、新しい見本の試験を新しいファイルに足す | 人は見本を全部作り直すよう頼んだ。古い見本を残すと、題材のそろわない見本が並び続ける |
| 見本の試験を消す | 見本が仕様や実装の変更で開けなくなったことに気づけなくなる |

## 承認の記録

2026-10-15 人の承認(要約: 見本を作り直して変わる見本の試験の錠の掛け直しを、スレッドで承認した)
