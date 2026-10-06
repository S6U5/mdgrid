---
type: Change
id: 01M488DKBH0CQBEXG9NS94Z2FT
title: チェックボックスの Enter を真と偽の切り替えにする(checkbox-two-state)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# チェックボックスの Enter を真と偽の切り替えにする(checkbox-two-state)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-checkbox-two-state.md・2026-10-06-checkbox-two-state-examples.md(人の承認) |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CE-4 のとおり(人の判断待ちの表の3。人がおすすめどおりと承認)。

## 不明点と仮定

- 仮定: 一括(CE-10)も、今の行の次の状態(真なら偽、ほかは真)を選んだ行にそろえる(今の作りのまま、次の状態の決め方だけ変える)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 次の状態を真と偽に | CE-4, CE-10, WB-17 | 採択した確かめ方 | src/ui/input.rs, src/ui/test_checkbox.rs, src/ui/test_input.rs, src/ui/test_checkbox_two_state.rs, src/ui/mod.rs, docs/manual-scenarios.toml, specs/test-locks.json | test_ce_4_two_state_*(src/ui/test_checkbox_two_state.rs)と、書き直した src/ui/test_checkbox.rs・src/ui/test_input.rs の CE-4 の試験 | 済 |

## 実装の気づき

- 錠のある src/ui/test_checkbox.rs(12本)と src/ui/test_input.rs の3本が、古い3つの状態(空 → 真 → 偽 → 空)を確かめていた。人の決定で要件を変えたので、新しい2つの状態に書き直した(観点は保つ: 元の値に戻れば外れる、一括、空の文字列、型の合わない行、詳細の表示、取り消しの1手)。空にする場面は BS で確かめる。
- 錠はファイルごとで、test_input.rs には関係のない WB-3・WB-5 の試験も入っていて掛け直せなかった。人の選択で、decidespec(0.11.2)に Rust の試験の関数ごとの錠(TL-8)を入れ、錠を関数ごとに移した(c54bfc8 を直したコミット。中身の変わらないファイルだけ)。
- 関数ごとでも、書き直した試験の多くは元の錠が CE-10・WB-17 も名乗るので、同じ承認の範囲で CE-4・CE-10・WB-17 の確かめ方の例に2つの状態の場面を書く決定(checkbox-two-state-examples)を足し、その決定で掛け直した(11本)。CE-4 だけの4本は checkbox-two-state で掛け直した。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。次の状態の決め方を1か所変える直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-4 | checked | コード: src/ui/input.rs の次の状態 / テスト: src/ui/test_checkbox_two_state.rs::test_ce_4_two_state_false_and_true・test_ce_4_two_state_keyless_and_empty、src/ui/test_checkbox.rs の12本 / 今: 通った(全体 1307 passed) / 前: 落ちた(実装の前に2本が落ちた) |
| CE-10 | checked | コード: src/ui/input.rs の toggle_checkbox(一括) / テスト: src/ui/test_checkbox.rs::test_ce_4_ce_10_bulk_toggle_follows_current_row ほか / 今: 通った / 前: 落ちた(3つの状態の期待) |
| WB-17 | checked | コード: 同上 / テスト: src/ui/test_input.rs::test_wb_17_ce_4_empty_checkbox_toggles_back・test_wb_17_ce_4_ce_10_bulk_toggle_per_row / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし(錠のある試験は、人の決定で変えた要件に合わせて書き直し、その決定で掛け直した)

確かめた: 3 / 3
