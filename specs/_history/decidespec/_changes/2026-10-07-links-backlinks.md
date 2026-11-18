---
type: Change
id: 01M48X55FWPM14BTT6W2TE246N
title: file.backlinks と file.hasLink() を評価する(links-backlinks)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# file.backlinks と file.hasLink() を評価する(links-backlinks)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-links-backlinks.md・2026-10-07-links-backlinks-sample.md(人の承認) |
| 設計 | 対象外: 評価のコードは links-this で入っている(名前を一覧に足すだけ) | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した BV-22 のとおり(人の判断待ちの表の1)。

## 不明点と仮定

- 仮定: 未対応の例は、まだ未対応の `link()`(関数)と `file.embeds`(file の項目)に差し替える。

## 設計

対象外: 評価のコードは links-this で入っている

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 名前を足し、例を差し替える | BV-22, BV-7 | 採択した確かめ方 | src/expr.rs, src/test_base_unit.rs, tests/test_base.rs, tests/test_language_lib.rs, tests/test_expr.rs, tests/test_links.rs, tests/test_examples.rs, examples/vault/タスク.base, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, specs/test-locks.json | test_bv_22_backlinks_and_has_link(tests/test_links.rs) | 済 |

## 実装の気づき

- 未対応の例を差し替えた錠のある試験(BV-7・SR-23・WB-3・WB-5・CE-22)は、決定の「錠:」の行(decidespec 0.11.3 の TL-9)で掛け直した。見本の .base の試験(BV-3・BV-7)は、見本の未対応の式の例を file.embeds に替える補いの決定で掛け直した。
- 錠の無い tests/test_links.rs の「未対応のまま」の試験は、対応したことを確かめる試験に書き直した。

## 照合

自分で照合(名前を一覧に足すだけ。評価のコードは links-this で試験済み)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-22 | checked | コード: src/expr.rs の FILE_FIELDS / テスト: tests/test_links.rs::test_bv_22_backlinks_and_has_link、src/test_links_unit.rs::test_bv_22_backlinks_and_has_link / 今: 通った(全体 1317 passed) / 前: 落ちた(実装の前は未対応で落ちた) |
| BV-7 | checked | コード: 同上 / テスト: tests/test_base.rs::test_bv_7_unsupported_function_in_filters_is_err_with_name ほか(例を link()・file.embeds に) / 今: 通った / 前: 通った(例の差し替え) |

既存のテストの削除・skip・弱体化: なし(錠のある試験は、未対応の例を別の未対応の名前に差し替え、人の決定で掛け直した)

確かめた: 2 / 2
