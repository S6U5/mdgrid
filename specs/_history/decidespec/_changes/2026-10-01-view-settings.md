---
type: Change
id: 01M3VM9G5X1EDZJ010D7A9AVFC
title: ビューの設定の画面と、表の上のフィルターの帯(view-settings)
status: done
size: full
created: 2026-10-01
updated: 2026-10-01
---

# ビューの設定の画面と、表の上のフィルターの帯(view-settings)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [view-settings](../_decisions/2026-10-01-view-settings.md)(人)、[view-settings-details](../_decisions/2026-10-01-view-settings-details.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-01): todo 以外の用途にも使うので、画面の中の設定から表の見せ方を変えたい。設定の画面で変えて、反映の操作で効かせる。例えば done の行を表示するかしないか、グループ分けをするかしないか。項目は Notion や Excel のような表の道具を参考にし、好きに組めるようにする。効いているフィルターは表の上に並べて見せる。

終わりの条件: 設定の画面で、フィルター(どの列のどの値でも)・並べ替え・グループ分け・表示する列を組み、反映で表に効き、取り消せば元のまま。効いている条件が表の上の帯に並ぶ。設定はビューごとに覚え、`.base` とノートには書かない。関係する要件: NV-2・NV-3・NV-4・NV-8・NV-12(重なり方)、BV-3・BV-5(`.base` を変えない・`.base` の並びとグループの上に重ねる)、SR-1(画面の帯)・SR-4・SR-11・SR-12・SR-16(キーと見た目の状態)。NV-10・NV-11(次の段)は今回の範囲に入れない。

## 不明点と仮定

- 問い → 答え(要約): done の表示の切り替えをどこまで一般にするか → 列と値を選んで隠す形(どの列でも)。
- 問い → 答え(要約): 設定をどこに覚えるか → ビューごとの見た目の状態(SR-11・SR-12 と同じ置き場)。
- 問い → 答え(要約): 設定の画面の項目 → Notion や Excel などの表の道具を参考に、好きに組める形。表の上のフィルターも。
- 仮定: 項目は Notion のビューの設定(プロパティ・フィルター・並べ替え・グループ)と Excel のオートフィルター(値の一覧にチェック)に倣う。外れたら: 項目を足し引きする提案を出す。
- 仮定: リレーション(ノートのプロパティから別のノートへのリンクを TUI で張る)は同じ会話で頼まれたが、リストの値の書き込み(CE-13、次の段)に関わるので別の変更にする。外れたら: この変更に入れる(止める)。

## 設計

設計の本文は docs/design.md の「ビューの設定(NV-13〜NV-22)」。要点: 条件の判定と絞る・並べる・まとめる処理は核の `src/settings.rs`(画面に依存しない)、設定は `config::ViewState.settings` に覚える(古い状態のファイルも読める)。重ね順は `.base` → 設定 → 簡易の絞り込み・同じ値 → 一時的な並べ替え → 直した行の留め。画面は `src/ui/settings.rs` の新しいモードと、`bands.rs` の上の帯。

選ばなかった案: 設定を `.base` に書く(BV-3・SR-11 に反する)、条件を式で持つ(人の答えは列と値を選ぶ形。式は NV-10)、絞り込みを画面の側だけで書く(核で試験を先に書けない)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 設定の核(条件の判定・絞る・並べる・まとめる・覚える) | NV-14, NV-15, NV-17, NV-19, NV-21 | 条件の種類ごとの判定、列の値でまとめ直し、空を隠す、ViewState に設定を往復できて古い状態も読める | src/settings.rs, src/test_settings_unit.rs, src/config.rs, src/lib.rs, src/ui/startup.rs, tests/test_config.rs, docs/design.md | test_NV_14, test_NV_15, test_NV_17, test_NV_19, test_NV_21 | 済 |
| 2 | 設定の画面・反映と取り消し・上の帯・重ね順・既定に戻す | NV-13, NV-16, NV-17, NV-18, NV-20, NV-22 | 設定の画面のゴールデン、反映の前は表が変わらない、帯の項目、重ね順、読むだけでも書かない | src/ui, src/main.rs, tests/golden | test_NV_13, test_NV_16, test_NV_18, test_NV_20, test_NV_22 | 済 |

順: 1 → 2。1 の受け入れの試験は、実装を見ていない役が先に書く(tests/test_settings.rs)。2 は作ってからゴールデンで固める。

## 実装の気づき

- タスク 1: tests/test_settings.rs は (a) `test_nv_17_old_state_file_without_settings_loads` の `!old.contains("settings")` が、一時フォルダの名前 `mdgrid-settings-…` を含む target のパスにも当たって必ず落ち、(b) `cargo fmt --check` を通らない(書式だけの差 9 か所)ので、試験を書いた役の直しが要る。(→ main が 3acc6ad で試験を直した)
- タスク 2: 列の選び手をビューの列だけにすると groupBy の列(examples の「状態ごと」の status は order に無い)で絞れないので、ビューに無いノートのキー(`Source::columns`)も選べるようにした(その列の見出しは displayName でなく id)。キーは 設定 `o`・帯 `f`、画面は settings.rs・settings_pick.rs・settings_view.rs に分け、ヘルプのゴールデン sr_5 は行数だけ変わった。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-01)。「前」は、実装の前のコミットではモジュールが無くコンパイルで落ちるだけのため、変異で確かめた。1回目は NV-20 の「直した行は保存で外れる」側の試験が無く gap だったので、試験を足し(main)、保存のあとに留めを外す処理を消す変異で落ちることを確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-13 | checked | コード: src/ui/settings.rs:449 / テスト: src/ui/test_settings_screen.rs::test_nv_13_apply_and_cancel / 今: 通った(cargo test) / 前: 落ちた(変異: 取り消しで写しを当てる → 写しを捨てずに落ちた) |
| NV-14 | checked | コード: src/settings.rs:298 / テスト: tests/test_settings.rs::test_nv_14_drop_done_hides_done_rows / 今: 通った(cargo test) / 前: 落ちた(変異: Keep を常に真 → 本 だけにならず落ちた) |
| NV-15 | checked | コード: src/settings.rs:426 / テスト: tests/test_settings.rs::test_nv_15_off_removes_groups / 今: 通った(cargo test) / 前: 落ちた(変異: Off で .base のまとまりを残す → 見出しが残り落ちた) |
| NV-16 | checked | コード: src/ui/bands.rs:136 / テスト: src/ui/test_settings_screen.rs::test_nv_16_band_lists_settings / 今: 通った(cargo test) / 前: 落ちた(変異: 帯の行数を 0 → 項目が出ず落ちた) |
| NV-17 | checked | コード: src/config.rs:203 / テスト: src/ui/test_settings_screen.rs::test_nv_17_settings_survive_restart_per_view / 今: 通った(cargo test) / 前: 落ちた(変異: 読むときに settings を捨てる → 既定に戻り落ちた) |
| NV-18 | checked | コード: src/ui/settings_view.rs:393 / テスト: src/ui/test_settings_screen.rs::test_nv_18_settings_screen_golden / 今: 通った(cargo test) / 前: 落ちた(変異: 並べ替えの区画の見出しを消す → ゴールデンと違い落ちた) |
| NV-19 | checked | コード: src/settings.rs:298 / テスト: tests/test_settings.rs::test_nv_19_conditions_are_and / 今: 通った(cargo test) / 前: 落ちた(変異: AND を OR に → 落ちた) |
| NV-20 | checked | コード: src/ui/grid.rs:276 / テスト: src/ui/test_settings_screen.rs::test_nv_20_layers / 今: 通った(cargo test) / 前: 落ちた(変異: 設定を overlay の後に → s の並びが勝たず落ちた。保存で留めを外す処理を消す → test_nv_20_edited_row_released_on_save が落ちた) |
| NV-21 | checked | コード: src/settings.rs:503 / テスト: tests/test_settings.rs::test_nv_21_hide_empty_group / 今: 通った(cargo test) / 前: 落ちた(変異: hide_empty を無視 → (空) の見出しが残り落ちた) |
| NV-22 | checked | コード: src/ui/settings.rs:434 / テスト: src/ui/test_settings_screen.rs::test_nv_22_readonly_applies_but_writes_nothing / 今: 通った(cargo test) / 前: 落ちた(変異: readonly でも状態を書く → 状態のファイルができ落ちた) |

既存のテストの削除・skip・弱体化: なし(3acc6ad は一時フォルダのパスに依る誤りの直しで、確かめる中身は同じ。tests/test_config.rs は ViewState の欄が増えた分の1行。sr_5 のゴールデンはヘルプの行数だけ)

確かめた: 10 / 10
