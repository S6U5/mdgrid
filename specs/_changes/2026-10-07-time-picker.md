---
type: Change
id: 01M497YFZ7DDAP0CMAVZ9NRMGR
title: 日時の列のカレンダーに時刻の欄を足す(time-picker)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 日時の列のカレンダーに時刻の欄を足す(time-picker)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-time-picker.md(人の承認)・2026-10-07-time-picker-details.md(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CE-30・CE-31 のとおり。

## 不明点と仮定

- 仮定: 時刻の欄は、カレンダーが画面に出ているときだけ操作できる(出ていなければ今までどおり打ち込みだけ)。
- 仮定: 時刻を選んでいるとき ←→ は日を動かさない(時刻の欄では使わない)。PageUp・PageDown と月・年のキーは日の操作のまま。

## 設計

- src/ui/calendar.rs の Cal に `on_time`(時刻の欄を選んでいるか)を持つ。窓の6週の下に時刻の行(`| 時刻 09:00 |`、空は `--:--`。選んでいれば反転と `>`)を足し、下の縁にキー(`Ctrl+O 時刻`)を出す。日時の列だけ。
- 新しい動作 `Action::TimeFocus`(名前 `time_focus`、Edit の既定 Ctrl+O)。時刻を選んでいるとき、ListUp/ListDown を ±15分(刻みにそろえる)、PrevYear/NextYear(Shift+↑↓)を ±1時間にする。
- 時刻の部分 `T09:00`(後ろの秒・タイムゾーンは保つ)の時と分だけを書き換え、`time_typed` を立てて入力の文字を作り直す(確定は今の choice の道)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 時刻の欄の状態・操作・描画 | CE-30, CE-31 | 採択した確かめ方 | src/ui/calendar.rs, src/ui/keymap.rs, src/ui/input.rs, src/ui/mod.rs, src/i18n.rs, src/ui/test_time_picker.rs, tests/golden/, docs/keys.md, docs/keys.ja.md, docs/manual/, README.md, README.ja.md | test_ce_30_time_picker_moves_and_commits(src/ui/test_time_picker.rs) | 済 |

## 実装の気づき

- 下の縁のキーは ^O の形で出る(ほかの Ctrl のキーと同じ)。
- 時刻の欄をクリックしても時刻の欄を選ぶ。
- ヘルプのゴールデン(sr_5)は行数が1つ増えただけで作り直した。

## 照合

自分で照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-30 | checked | コード: src/ui/calendar.rs の calendar_action・time_step・rows / テスト: src/ui/test_time_picker.rs::test_ce_30_time_picker_moves_and_commits / 今: 通った(全体 1328 passed) / 前: 落ちた(実装の前は時刻の欄が無く、Ctrl+O で動かなかった) |
| CE-31 | checked | コード: src/ui/calendar.rs の step_time・hm・edges、src/ui/keymap.rs の time_focus / テスト: src/ui/test_time_picker.rs::test_ce_31_time_picker_details、src/ui/test_time_picker.rs::test_ce_31_bulk_time_goes_to_every_row / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
