---
type: Change
id: 01M4B51YFEB12WMXTWYGSP4NVA
title: views.toml の new_note の知っている項目に mode・required・hidden・body を入れる(views-new-note-keys)
status: done
size: bugfix
created: 2026-10-07
updated: 2026-10-07
---

# views.toml の new_note の知っている項目に mode・required・hidden・body を入れる(views-new-note-keys)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | なし |
| 仕様化 | 対象外: bugfix(CE-27・CE-33 の「ビューごとにも決められる」のとおりに直す) | なし |
| 設計 | 対象外: bugfix | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

保守性の調べ(リファクタリングの前の調べ)で見つけた。views.rs の `NEW_NOTE_KEYS` が folder・name・ask・set だけで、new-note-form・new-note-editor で足した mode・required・hidden・body を知らない。そのため views.toml を読むと、その4つが「知らない項目」と警告され、ビューを保存し直すと前のビューの値が「知らない項目」として新しい表に持ち越される(消した値が戻る)。知っている項目の並びを NewNote の側に1つだけ置き、views.rs はそれを使う。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 項目の並びを1つにする | CE-27 | 4つの項目で警告が出ず、消した値が戻らない | src/newnote.rs, src/views.rs, tests/test_views_new_note_keys.rs | test_ce_27_views_new_note_new_keys(tests/test_views_new_note_keys.rs) | 済 |

## 実装の気づき

- 知っている項目の並びを NewNote::KEYS に1つだけ置き、views.rs の NEW_NOTE_KEYS はそれを指す。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-27 | checked | コード: src/newnote.rs の NewNote::KEYS、src/views.rs の NEW_NOTE_KEYS / テスト: tests/test_views_new_note_keys.rs::test_ce_27_views_new_note_new_keys / 今: 通った(全体 1354 passed) / 前: 落ちた(4つの項目が「知らない項目」と警告された) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
