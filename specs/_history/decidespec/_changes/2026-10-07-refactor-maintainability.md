---
type: Change
id: 01M4B55XVS96X5Z24VF63DG5Z7
title: 保守性のためのリファクタリング(振る舞いを変えない)(refactor-maintainability)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 保守性のためのリファクタリング(振る舞いを変えない)(refactor-maintainability)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 振る舞いを変えない(要件は変えない。設計の判断は docs/design.md に書く) | なし |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット(タスクごとに1つ以上) |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼(会話 2026-10-07): 保守性を高めるために、設計原則と良いやり方を調べたうえでリファクタリングする。ブランチ refactor/maintainability で進め、進め方は任された。

## 不明点と仮定

- 仮定: 振る舞い(画面・書き戻し・出力)は変えない。試験(1354 件・ゴールデン 25)が全部通ることと、錠のある試験を書き換えないことを、各段の条件にする。
- 仮定: 錠のある試験が読む App の項目と関数の名前は変えない(変えるなら人の決定が要る)。

## 設計

調べ(読むだけの2つの調べ役: コードの込み合い、Rust と ratatui の設計の決まり)からの方針:

- ratatui の作り(TEA・部品・Flux)に共通するのは一方向(入力 → 動作 → 状態の更新 → 描画)。描画は状態を変えない。
- 重ねる窓(カレンダー・候補・メニュー・件数・リストの選択・新しいノートの窓)は、置き場・枠・当たりの決め方が同じ形で6か所に写されている。共通の部品(src/ui/popup.rs)にまとめる。
- 同じ働きの関数の写し(値を文字にする関数・`one_line` の同名異義・原子的な書き込み・TOML の誤りの行)を1つにする。
- ライブラリの輪(types・expr ↔ print の時計、config・views ↔ newnote の rule_for、links ↔ source)を断つ。公の道は `pub use` で残す(試験を変えない)。
- `--apply`(src/apply.rs)を画面の部品(ui::entry・ui::diff・ui::review)から離す。
- App(81 の項目)は、試験が読まない項目から部分の構造体にまとめる。
- 各段は小さく、振る舞いを変えず、段ごとに全部の試験と clippy を通してコミットする(古い道と新しい道を並べ、移し終えたら古い道を消す)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 値を文字にする関数の写しを1つにし、同名の one_line を名前で分ける | SR-1 | 全部の試験が通る | src/print.rs, src/ui/external.rs, src/ui/input.rs, src/ui/nav.rs, src/ui/new_note.rs, src/ui/cell.rs, src/config.rs, src/views.rs, src/writeback.rs, src/source.rs, src/ui/*.rs | 既存の全部 | 済 |
| 2 | 原子的な書き込みと TOML の誤りの行を1つの関数に | SR-11 | 全部の試験が通る | src/config.rs, src/views.rs, src/fsutil.rs, src/lib.rs | 既存の全部 | 済 |
| 3 | 重ねる窓の置き場・枠・当たりを src/ui/popup.rs に | SR-9 | 全部の試験とゴールデンが通る | src/ui/popup.rs, src/ui/freq.rs, src/ui/menu.rs, src/ui/listpick.rs, src/ui/list.rs, src/ui/calendar.rs, src/ui/mod.rs | 既存の全部 | 済 |
| 4 | リストの選択の上下と、動作の振り分けの写しを1つに | SR-4 | 全部の試験が通る | src/ui/*.rs | 既存の全部 | 済 |
| 5 | ライブラリの輪を断つ(時計・links・YAML の見張り) | BV-1 | 全部の試験が通る | src/clock.rs, src/mdtext.rs, src/yaml_guard.rs, src/base.rs, src/frontmatter.rs, src/print.rs, src/types.rs, src/expr.rs, src/newnote.rs, src/views.rs, src/config.rs, src/links.rs, src/source.rs, src/source/markdown.rs, src/lib.rs, src/main.rs, src/ui/*.rs | 既存の全部 | 済 |
| 6 | --apply を画面の部品から離す | CLI-17 | 全部の試験が通る | src/apply.rs, src/ui/entry.rs, src/ui/diff.rs, src/diff.rs, src/ui/review.rs, src/ui/input.rs, src/ui/mod.rs, src/edit.rs, src/lib.rs, src/main.rs | 既存の全部 | 済 |
| 7 | App の試験が読まない項目を部分の構造体に | SR-1 | 全部の試験が通る | src/ui/*.rs | 既存の全部 | 済 |
| 8 | 設計の判断を docs/design.md に | SR-1 | 書いた | docs/design.md | 読んで判定 | 済 |

## 実装の気づき

- 調べの途中で見つけた不具合は、別の記録で直した: views.toml の new_note の知らない項目(views-new-note-keys)、雛形の時刻の時差(template-time-offset)。
- 輪のうち newnote::rule_for は、試験がその道を使うので残した(docs/design.md)。
- 頻度表と操作の一覧は w < 8 の判定を先にしたが、どちらも None を返すので振る舞いは同じ。リストの選択(listpick)は少なくとも見せる行を 0 にして、今までの判定(room < FRAME)と同じにした。
- 一覧の選びの上下(step_sel)は、同じ動作の組(↑↓・先頭・末尾・1画面)を持つ頻度表・操作の一覧・詳しく見る画面だけに使った。↑↓ だけのパレットとヘルプは変えていない(先頭・末尾の動作が届くと振る舞いが変わるため)。

## 照合

振る舞いを変えない変更なので、要件ごとの新しい試験は足さず、既存の全部(1355 件・ゴールデン)が各段で通ることで確かめた。錠のある試験の本文と共有部分は変えていない(check.py の E19 が無い)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-1 | checked | コード: src/ui/popup.rs、src/ui/grid.rs の Built / テスト: src/ui/test_screen.rs::test_sr_1_five_bands / 今: 通った(全体 1355 passed) / 前: 通った(振る舞いを変えない) |
| SR-11 | checked | コード: src/config.rs の write_atomic / テスト: tests/test_config.rs::test_sr_11_save_then_load_round_trip、src/test_config_unit.rs::test_sr_11_save_state_removes_stale_tmp_files / 今: 通った / 前: 通った |
| SR-9 | checked | コード: src/ui/popup.rs の place_with / テスト: src/ui/test_listpick.rs::test_sr_9_window_fits_small_terminal / 今: 通った / 前: 通った |
| SR-4 | checked | コード: src/ui/popup.rs の step_sel / テスト: src/ui/test_freq.rs::test_nv_9_window_keeps_table_outside / 今: 通った / 前: 通った |
| BV-1 | checked | コード: src/clock.rs、src/mdtext.rs、src/yaml_guard.rs / テスト: tests/test_base.rs::test_bv_1_default_grid_uses_source_columns_and_rows / 今: 通った / 前: 通った |
| CLI-17 | checked | コード: src/edit.rs、src/diff.rs、src/apply.rs / テスト: tests/test_apply.rs::test_cli_17_csv_round_trip_from_stdin / 今: 通った / 前: 通った |

既存のテストの削除・skip・弱体化: なし

確かめた: 6 / 6
