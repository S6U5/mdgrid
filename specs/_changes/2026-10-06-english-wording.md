---
type: Change
id: 01M46QYPKPZQ5C8DVKCSZERMAV
title: 英語の画面の文言の直しと、検索の「見つからない」(english-wording)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# 英語の画面の文言の直しと、検索の「見つからない」(english-wording)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 要件の意味を変えない(英語の文言だけ) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-10・U-11)が指摘した英語の文言を直す。表の上の欄は中身が絞り込み(NV-2・NV-23)なのに「Search:」と出て、本当の検索(`/`)と紛れる。保存の確認の見出しが `1 files`、終了の確認が `1 unsaved changes` と文法が崩れる。

終わりの条件: `/` で一致の無い語を Enter で確定すると、メッセージ行に「見つからない: 語」が残る。英語の画面で、上の欄が `Filter: (\ to filter)`、保存の確認が `1 file(s)`、終了の確認が `1 unsaved change(s)`。日本語は変えない。関係する要件: SR-23(意味は変えない)・NV-1。

## 不明点と仮定

- 仮定: 下の帯の並び(狭い端末で `? Help` が切れる)と、衝突が無いのに `o` を出すことは、キーの表の順位と golden に響くので、この変更に入れない(docs/todo.md に残す)。
- 確かめた: `/` で一致の無い語を打つと、打っている間は「見つからない」が出るが、Enter で確定するとモードが表に戻るときにメッセージが消える(点検の指摘のとおり)。確定のあとも「見つからない: 語」を出す(NV-1 の不具合の直し)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 英語の文言と、確定のあとの見つからない | SR-23, NV-1 | 上の終わりの条件のとおり | src/i18n.rs, src/ui/nav.rs, src/ui/test_search_not_found.rs, src/ui/mod.rs, docs/assets/, docs/todo.md | test_nv_1_not_found_stays_after_enter(src/ui/test_search_not_found.rs)と、読んで判定(英語の画面を撮る) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。文言と小さな直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-1 | checked | コード: src/ui/nav.rs の prompt_action の Commit / テスト: src/ui/test_search_not_found.rs::test_nv_1_not_found_stays_after_enter・test_nv_1_found_has_no_not_found / 今: 通った(全体 1102 passed) / 前: 落ちた(実装の前に試験を置き、not_found_stays_after_enter が「確定のあとも残る: \"\"」で落ちた) |
| SR-23 | checked | 読んで判定: 英語の画面を撮り、上の欄が `Filter: (\ to filter)`、保存の確認の見出しが `1 file(s)`。終了の確認の `{0} unsaved change(s)` は i18n の表で確かめた。日本語は変えていない |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2

README の画像(docs/assets/demo-*.svg)と demo.gif を撮り直した(上の欄の文字が変わったため)。
