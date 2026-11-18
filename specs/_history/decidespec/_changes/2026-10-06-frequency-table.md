---
type: Change
id: 01M46S9S6MHZR9SWWHKQZGPS8D
title: 列の値の頻度表(frequency-table)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# 列の値の頻度表(frequency-table)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [frequency-table](../_decisions/2026-10-06-frequency-table.md)(AI)、[sr16-freq-mode](../_decisions/2026-10-06-sr16-freq-mode.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-13)が、値ごとの件数を見る手段が無いと指摘した。NV-9 を版1に上げ、SR-16 のモードに頻度表を足した。

終わりの条件: NV-9 の例のとおり。英語と日本語で出る(SR-23)。キーの表・ヘルプ・docs/keys.md に出る(SR-4・SR-22)。関係する要件: NV-9・SR-16・SR-24(列の節に項目)・SR-4・SR-22・SR-23・NV-8・NV-16。

## 不明点と仮定

- 仮定: 値の並びの同じ数のときは値の文字の順。割合は全行(今のビューの絞り込みのあと)に対する整数の百分率。リストの値は要素ごとに数えるので、割合の合計は 100% を超えうる。
- 仮定: 選んだ値で絞るのは、NV-8 の `,`(同じ値の行だけ)が作るのと同じ設定の条件(リストの列なら要素を含む条件、空なら値の無い条件)。
- 仮定: 読むだけで開いたとき(WB-15)も使える(絞り込みは設定で、ノートを書かない)。

## 設計

- 新しい src/ui/freq.rs: 頻度表の窓(操作の一覧の窓 src/ui/menu.rs と同じ形: 表の上に重ね、窓の外の表は見えたまま、窓が入らない端末では開かず理由)。項目を作る純関数(今の列・今の行の並び → (値, 件数) の並び)、開く・動かす・決める・閉じる・描く。
- src/ui/keymap.rs: 動作 `Action::Freq`(名前 `frequency`)を表のモードの `%` に、新しいモード `Mode::Freq`(設定の名前 `freq`)と、そのモードの ↑↓・`j` `k`・Enter・Esc・`g g`・`G`。docs/keys.md・keys.ja.md に足す(SR-22 の試験が突き合わせる)。
- 決めたときは、NV-8 の同じ値で絞る道(`,`)に値を渡して同じ条件を作る(設定の帯に出る)。
- src/ui/menu.rs の列の節に「頻度表」の項目、パレットにも出る(キーの表から作られる)。
- 文言は src/i18n.rs に英語と日本語。
- 却下: 全画面の別の表(どの列の表かが見えなくなる)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 頻度表 | NV-9, SR-16, SR-24, SR-22, SR-23 | NV-9 の例のとおりで、既存の試験が通る | src/ui/freq.rs, src/ui/keymap.rs, src/ui/menu.rs, src/ui/mod.rs, src/ui/app.rs, src/ui/view.rs, src/ui/bands.rs, src/ui/nav.rs, src/ui/help.rs, src/i18n.rs, src/config_items.rs, docs/keys.md, docs/keys.ja.md, docs/config.md, docs/config.ja.md, tests/golden/, src/ui/test_freq.rs, src/ui/test_freq_more.rs, src/ui/grid.rs, README.md | test_nv_9_*(src/ui/test_freq.rs) | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。試験は実装を見ていない別の役が先に書き、実装は別の役、レビューも別の役(指摘3件を直した))。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-9 | checked | コード: src/ui/freq.rs、src/ui/nav.rs の Same の並び、src/ui/grid.rs の判定、src/ui/bands.rs のヘッダー / テスト: src/ui/test_freq.rs::test_nv_9_*(18本)、src/ui/test_freq_more.rs(6本) / 今: 通った(全体 1134 passed) / 前: 落ちた(実装の前は Mode::Freq が無くコンパイルで落ちた。試験を書く役が Mode::Freq を Mode::Table に置き換えた写しで回し、窓が開かない・frequency の動作が無いで落ちることを確かめた)。実物: 英語の見本の owner の列で `%` → `Alex 4 (40%)`・`Ken 2 (20%)`… が多い順 |
| SR-16 | checked | コード: src/ui/keymap.rs の Mode::Freq とそのモードのキー / テスト: src/ui/keymap.rs::test_sr_16_no_duplicate_keys_per_mode / 今: 通った(cargo test) / 前: 落ちた(実装の前は Mode::Freq が無く、試験の Mode::by_name("freq") がコンパイルで落ちた。要件の手前) |
| SR-24 | checked | コード: src/ui/menu.rs の列の節の頻度表の項目 / テスト: src/ui/test_freq.rs::test_nv_9_action_menu_column_section_has_item / 今: 通った(cargo test) / 前: 落ちた(写しの回しで「列の節に頻度表の項目が無い」) |
| SR-22 | checked | 読んで判定: docs/keys.md・keys.ja.md の freq の節と動作 frequency を、キーの表と突き合わせる SR-22 の試験(src/ui/test_keys_doc.rs)が通った |
| SR-23 | checked | コード: src/i18n.rs の頻度表の文言 / テスト: src/ui/test_freq.rs::test_nv_9_english_window_has_no_japanese / 今: 通った(cargo test) / 前: 落ちた(実装の前は窓が開かず、英語の窓の文字が無い) |

既存のテストの削除・skip・弱体化: なし(golden sr_5 はヘルプの行数だけ)

確かめた: 5 / 5
