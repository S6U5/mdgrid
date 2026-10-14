---
type: Change
id: 01M4P63MBTH1BR4YGTFVGTFDSA
title: 検索の欄と絞り込み・並べ替えのボタンを線で四角く囲み、ビューの帯を3行にする
status: done
size: full
created: 2026-10-12
updated: 2026-10-12
---

# 検索の欄と絞り込み・並べ替えのボタンを線で四角く囲み、ビューの帯を3行にする

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [search-box](../_decisions/2026-10-12-search-box.md)、[padding-default-one](../_decisions/2026-10-12-padding-default-one.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、検索の欄を線で囲んで作らないかと言い、欄と絞り込み・並べ替えのボタンを四角く囲む3行の形を選んだ。一旦やってみる形で戻すかもしれず、設定でボタンの形を選べるようにする案も出し、仕様は任せると言った(会話 2026-10-12)。ブランチ view-bar の続き。終わりの条件: 決定記録のとおりに描き、`display.buttons = "flat"` と低い端末で1行に戻り、画面を撮り直して見られる。関係する要件: SR-1・NV-23。

## 不明点と仮定

- 仮定: 囲みの線は今の窓の線(`popup::frame`。丸い角か、設定の罫線)と同じ字。
- 仮定: 囲む3つ(検索の欄・絞り込み・並べ替え)は、それぞれ別の四角。タブ・分類・列・「+ 新規」は真ん中の行に並べる。
- 仮定: クリックの当たりは、囲みの3行のどこを押してもそのボタン(欄)。
- 仮定: `display.buttons` は全体・ワークスペース・表・ビューの各段で書ける表示の項目(SR-20 のほかの表示の項目と同じ扱い)。

## 設計

変える場所: `src/ui/bands.rs`(帯の行の数 `tab_rows` を囲むとき3に、`view_bar_line` を3行に、当たりと `button_anchor` を囲みに合わせる)、`src/ui/view.rs`(帯の3行を並べる)、`src/display.rs` か設定の表示の項目(`display.buttons`)、`src/schema.rs`、`src/config.rs`、`docs/config.md`・`docs/config.ja.md`、`tests/golden/`。変えない場所: ヘッダー、窓の中身、関係マップ。検証: `cargo test --bin mdgrid`、`cargo test`、`sh demos/record.sh layout layout-narrow`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | SR-1, NV-23 | 実装の前に落ちる | `src/ui/test_search_box.rs` `src/ui/mod.rs` | test_nv_23_box_* | 済 |
| 2 | 囲みの3行の帯と `display.buttons` を作る | SR-1, NV-23 | 1 の試験が通る | `src/ui/bands.rs` `src/ui/view.rs` `src/ui/app.rs` `src/ui/sorts.rs` `src/ui/filters.rs` `src/ui/relmap.rs` `src/ui/screen_menu.rs` `src/ui/drawer.rs` `src/ui/popup.rs` `src/display.rs` `src/profile.rs` `src/settings.rs` `src/schema.rs` `src/config.rs` `src/i18n.rs` `src/ui/settings.rs` `src/ui/settings_view.rs` `docs/config.md` `docs/config.ja.md` `docs/manual/` | 1 の試験 | 済 |
| 3 | 変わる既存の試験と画面の写しを直す | SR-1, NV-23 | `cargo test` が通る(前からの test_cliff を除く) | `src/ui/test_*.rs` `src/test_*.rs` `tests/` `tests/golden/` | 既存の試験 | 済 |
| 4 | 後の画面を撮り直す | SR-1 | `demos/out/layout-compare/` が新しい | `demos/layout.tape` | 読んで判定 | 済 |
| 5 | 既定を線なし・上下に余白の3行にし、box は中を塗らず、1行は compact にする([view-bar-plain](../_decisions/2026-10-12-view-bar-plain.md)、余白は [bar-padding](../_decisions/2026-10-12-bar-padding.md) で行の数の設定に) | SR-1, NV-23 | 新しい受け入れの試験と、box を設定で選ぶように直した test_search_box が通る | `src/ui/test_view_bar_plain.rs` `src/ui/test_search_box.rs` `src/ui/test_bar_padding.rs` `src/ui/mod.rs` `src/ui/bands.rs` `src/ui/display.rs` `src/ui/settings_view.rs` `src/display.rs` `src/profile.rs` `src/schema.rs` `src/i18n.rs` `docs/config.md` `docs/config.ja.md` `tests/golden/` | test_nv_23_plain_* | 済 |
| 6 | 余白を 0.5 刻みにし既定を 0.5、ヘッダーの上の余白 `display.top_padding` を足し、端数は半分の塗りの字で描く([bar-padding-half](../_decisions/2026-10-12-bar-padding-half.md)) | SR-1, NV-23 | 新しい受け入れの試験が通る | `src/ui/test_bar_padding_half.rs` `src/ui/test_bar_padding_theme.rs` `src/ui/mod.rs` `src/ui/bands.rs` `src/ui/view.rs` `src/ui/look.rs` `src/display.rs` `src/profile.rs` `src/schema.rs` `src/i18n.rs` `src/ui/display.rs` `src/ui/settings_view.rs` `src/ui/relmap.rs` `src/ui/drawer.rs` `src/ui/screen_menu.rs` `src/ui/sorts.rs` `src/ui/filters.rs` `src/ui/theme.rs` `src/ui/help.rs` `src/ui/native_io.rs` `tests/test_display_options.rs` `docs/config.md` `docs/config.ja.md` `src/ui/test_*.rs` `tests/golden/` | test_sr_1_half_* / test_nv_23_half_* | 済 |
| 7 | 半行の余白を、帯の色は地の色で塗り外側を画面の地の色の字で消す描き方にする([half-row-inverse](../_decisions/2026-10-12-half-row-inverse.md)) | NV-23 | 新しい受け入れの試験が通る | `src/ui/test_half_row_inverse.rs` `src/ui/test_half_row_inverse_termbg.rs` `src/ui/mod.rs` `src/ui/bands.rs` `src/ui/view.rs` `src/ui/look.rs` `src/ui/termbg.rs` `src/ui/test_bar_padding_half.rs` `src/ui/test_bar_padding_theme.rs` `src/ui/test_themes.rs` `src/schema.rs` `docs/config.md` `docs/config.ja.md` `src/ui/app.rs` `src/ui/startup.rs` `src/main.rs` `src/ui/mod.rs` `src/ui/relmap.rs` `src/ui/test_*.rs` `tests/golden/` | test_nv_23_half_inverse_* | 済 |
| 8 | 上の余白とビューの帯の余白の既定を 1 にする([padding-default-one](../_decisions/2026-10-12-padding-default-one.md)) | SR-1, NV-23 | 新しい受け入れの試験が通る | `src/ui/test_padding_default_one.rs` `src/ui/mod.rs` `src/display.rs` `src/profile.rs` `src/schema.rs` `src/ui/display.rs` `src/ui/settings_view.rs` `docs/config.md` `docs/config.ja.md` `src/ui/test_*.rs` `tests/` `tests/golden/` | test_sr_1_default_one_* / test_nv_23_default_one_* | 済 |
| 9 | 手引きとカタログの見本を新しい配置(パンくずのボタン・右上のメニューと設定・余白のあるビューの帯・帯の右の端の「+ 新規」)に合わせる | SR-1, NV-23 | 手引きに古い配置の言い方が無く、カタログの見本のヘッダーと帯が本体と同じ並び。`cargo test --test test_catalog` が通る | `docs/manual/` `docs/manual-scenarios.toml` `docs/catalog/index.html` | 読んで判定 / test_catalog | 済 |

## 実装の気づき

- 帯の行の数は 1 + 2×余白、真ん中の行は 1 + 余白(`bands::bar_padding`・`three_rows`)。余白は、検索の欄を出さないときと高さ16行未満で 0。
- 既存の試験は、帯の行を真ん中の行に、表の上の行の数を帯の行の数に合わせた(アサーションは消さず緩めない)。ほかの要件の錠を持つものは [header-menu-locks](../_decisions/2026-10-12-header-menu-locks.md) で掛け直す。
- 余白の既定を 1 にした(`Pad::DEFAULT`)。半行の描き方の試験は設定で 0.5 を明示し、錠は [padding-default-one-locks](../_decisions/2026-10-12-padding-default-one-locks.md) で掛け直した。`--print-config` は既定を整数の `1` で書く。
- 「表示」の区画の切り替えのため `src/ui/display.rs`・`src/ui/settings_view.rs` も直した(範囲に足した)。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-12)が HEAD 33a3b90 と、試験のコミットの worktree(試験と実装が同じコミットのものは実装の直前に試験を重ねた)で確かめた。check.py --evidence は、今の試験が実装前のコードでコンパイルできず全部 gap になったので使わなかった。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-1 | checked | コード: src/display.rs:117 `Pad::DEFAULT`・:152 `Pad::rows`、src/ui/bands.rs:392 `top_pad`・:456 `top_lines`・:291 `head_bg` / テスト: src/ui/test_padding_default_one.rs::test_sr_1_default_one_with_known_terminal_background、src/ui/test_bar_padding_half.rs::test_sr_1_half_*、src/ui/test_half_explicit_rounds_up.rs::test_sr_1_half_explicit_no_color_rounds_up / 今: 通った(`cargo test --no-fail-fast`) / 前: 6320593 で既定 1 の試験が「半行の字は無い」で落ちた。7e4265e で test_sr_1_half_* が上の余白の assert で落ちた |
| NV-23 | checked | コード: src/ui/bands.rs:342 `tab_rows`・:350 `BOX_MIN_H`・:356 `half_ok`・:375 `bar_pad`・:444 `half_line`・:472 `boxed`、src/ui/look.rs:63 `screen_bg`、src/ui/termbg.rs:31 `rgb_from_osc11`、src/main.rs:1506 / テスト: src/ui/test_search_box.rs、src/ui/test_view_bar_plain.rs、src/ui/test_bar_padding.rs、src/ui/test_bar_padding_half.rs::test_nv_23_*、src/ui/test_half_row_inverse.rs、src/ui/test_bar_padding_theme.rs、src/ui/test_padding_default_one.rs::test_nv_23_*、src/ui/test_half_explicit_rounds_up.rs::test_nv_23_* / 今: 通った / 前: 685d176 の囲み・余白の試験を 952c863 に重ねて 9 本が要件の assert で落ちた。7e4265e で test_bar_padding_half の 4/5 が落ちた。半行の描き方の試験を 8a9cdd1 に重ねて 5/6 が落ちた。test_half_row_inverse_termbg は前のコードに `App::term_bg` が無くコンパイルできず、前は確かめていない(同じ振る舞いを test_bar_padding_theme が確かめる) |

既存のテストの削除・skip・弱体化: アサーションの削除・#[ignore] は無い。既定を 1 にした 33a3b90 で、0.5 の切り上げを見ていた試験 4 本(設定なしで起動)が既定 1 の配置を見るようになり、色なしとあいまいな字の設定で 0.5 を切り上げる確かめが抜けた(検証役の指摘)。行き先: 直した。0.5 を明示した試験を新しいファイル src/ui/test_half_explicit_rounds_up.rs に足した(端末の地の色が分かっていても切り上げることと、対照の半行)。

確かめた: 2 / 2
