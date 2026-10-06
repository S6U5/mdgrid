---
type: Change
id: 01M471YH379Q8Z2KKMY9Z0HDQD
title: 入力の行を一度に消す Ctrl+U(clear-input)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# 入力の行を一度に消す Ctrl+U(clear-input)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 要件の意味を変えない(キーの表に既定の割り当てを1つ足すだけ。SR-4・SR-13 の範囲) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、Obsidian を使う人の目の点検(docs/todo.md の O-13)と初めて使う人の目の点検(U-1)が、入力の行を一度に消すキーが無いと指摘した(日時を打ち直すのに BS を 19 回)。編集のモードの Ctrl+U(端末の行の編集と同じ)で入力の文字を全部消す。

終わりの条件: 編集のモードで Ctrl+U → 入力が空になり、カーソルが先頭。リストが出ていれば自由入力に切り替わる。動作の名前 `clear_input` で割り当て直せ、docs/keys.md・keys.ja.md・ヘルプに出る。関係する要件: SR-4・SR-13・SR-22・SR-23・CE-1。

## 不明点と仮定

- 仮定: 消したあとに Ctrl+R(編集前に戻す)で戻せる(今の Revert のまま)。一括(CE-10)では「触った」に数える(空を入れるつもりの操作)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | Ctrl+U で入力を消す | SR-4, SR-22, CE-1 | 上の終わりの条件のとおり | src/ui/keymap.rs, src/ui/input.rs, src/i18n.rs, docs/keys.md, docs/keys.ja.md, tests/golden/sr_5.txt, src/ui/test_clear_input.rs, src/ui/mod.rs | test_ce_1_clear_input_*(src/ui/test_clear_input.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。キーを1つ足すだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-1 | checked | コード: src/ui/input.rs の Action::ClearInput / テスト: src/ui/test_clear_input.rs::test_ce_1_clear_input_empties_the_box / 今: 通った / 前: 落ちた(動作が無く、組み立てで落ちた。要件の手前) |
| SR-4 | checked | 読んで判定: キーの表(src/ui/keymap.rs)に1行足しただけで、振り分け・下の帯・ヘルプ・パレットは表から作られる(test_clear_input の by_name と、SR-4 の既存の試験が通った) |
| SR-22 | checked | 読んで判定: docs/keys.md・keys.ja.md の edit の節と動作の一覧に clear_input を足し、キーの表と突き合わせる試験(src/ui/test_keys_doc.rs)が通った |

既存のテストの削除・skip・弱体化: なし(ヘルプの golden は変わらない)

確かめた: 3 / 3
