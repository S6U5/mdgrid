---
type: Change
id: 01M4P3P4159C2NTNW5AKYX09EP
title: 設定を右上の別のボタンに、画面の型をメニューに、「+ 新規」をビューの帯に移し、ボタンのキーの字を目立たせる
status: done
size: full
created: 2026-10-12
updated: 2026-10-12
---

# 設定を右上の別のボタンに、画面の型をメニューに、「+ 新規」をビューの帯に移し、ボタンのキーの字を目立たせる

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [header-menu](../_decisions/2026-10-12-header-menu.md)、[relmap-new-row](../_decisions/2026-10-12-relmap-new-row.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、ビューの帯の「o ⚙」を設定だと気づかず、頭の o を 0 と読んだ。設定は右上の別のボタン、画面の型(表・関係)はメニュー、「+ 新規」はビューの帯の右の端、ボタンの頭のキーの字は名前と見分けられる見た目にすると決めた(会話 2026-10-12)。ブランチ view-bar の続き。終わりの条件: 決定記録の5つの要件のとおりに動き、画面を撮り直して前と並べられる。関係する要件: SR-1・SR-42・NV-23・REL-8・CE-25。

## 不明点と仮定

- 仮定: 画面の型のメニューのボタンは、表の画面で「▦ 表 ▾」、関係マップで「◇ 関係マップ ▾」。つながる表が無いときも出す(窓には「表」だけ)。
- 仮定: 窓は並べ替えの窓(NV-24)と同じ作り(ボタンのすぐ下、↑↓・Enter・Esc・クリック)で、新しいモード screen_menu を足す(キーの表で割り当て直せる)。
- 仮定: 設定のボタンの字は「⚙ 設定」で、頭にキー(既定 o)。幅が足りなければ先に省く。
- 仮定: キーの字の見た目は、モダンな見た目ではテーマの強調の色と太字、色なしでは太字。

## 設計

変える場所: `src/ui/bands.rs`(ヘッダーの右の端をメニューのボタンと設定のボタンに。ビューの帯から Settings を外し、右の端に「+ 新規」。ボタンの頭のキーの字を別の span で描く)、`src/ui/new_note.rs`(「+ 新規」の当たりをビューの帯へ)、`src/ui/relmap.rs`(2行目の右の端に「+ 新規」)、新しい `src/ui/screen_menu.rs`(窓)、`src/ui/app.rs`(クリックの振り分け)、`src/ui/keymap.rs`(Mode::ScreenMenu)、`src/i18n.rs`、`docs/keys*.md`、`tests/golden/`。変えない場所: パンくず、引き出し、関係マップの中身、R のキー。検証: `cargo test --bin mdgrid`、`cargo test`、`sh demos/record.sh layout layout-narrow`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | SR-1, SR-42, NV-23, REL-8, CE-25 | 実装の前に落ちる | `src/ui/test_header_menu.rs` `src/ui/mod.rs` | test_sr_42_settings_* / test_rel_8_* / test_ce_25_new_in_view_bar_* / test_nv_23_key_* | 済 |
| 2 | 右上のメニューと設定のボタン、ビューの帯の「+ 新規」、キーの字の見た目を作る | SR-1, SR-42, NV-23, REL-8, CE-25 | 1 の試験が通る | `src/ui/bands.rs` `src/ui/new_note.rs` `src/ui/relmap.rs` `src/ui/screen_menu.rs` `src/ui/app.rs` `src/ui/keymap.rs` `src/ui/input.rs` `src/ui/view.rs` `src/ui/mod.rs` `src/i18n.rs` `docs/keys.md` `docs/keys.ja.md` `docs/config.md` `docs/config.ja.md` `src/schema.rs` `src/config.rs` `docs/manual/` `docs/manual-scenarios.toml` | 1 の試験 | 済 |
| 3 | 変わる既存の試験と画面の写しを直し、錠を掛け直す | SR-1, SR-42, NV-23, REL-8, CE-25 | `cargo test` が通る(前からの test_cliff を除く) | `src/ui/test_*.rs` `tests/` `tests/golden/` `specs/test-locks.json` | 既存の試験 | 済 |
| 4 | 後の画面を撮り直す | SR-1 | `demos/out/layout-compare/` が新しい | `demos/layout.tape` | 読んで判定 | 済 |
| 5 | 関係マップの「+ 新規」を、表の画面のビューの帯と同じ行に置く([relmap-new-row](../_decisions/2026-10-12-relmap-new-row.md)) | CE-25 | 新しい受け入れの試験が通る | `src/ui/test_relmap_new_row.rs` `src/ui/mod.rs` `src/ui/relmap.rs` `src/ui/bands.rs` `src/ui/test_*.rs` `tests/` `tests/golden/` | test_ce_25_relmap_new_* | 済 |

## 実装の気づき

- 設定のボタンは端末の幅が80以上のときだけ出す(`SETTINGS_MIN_W`)。ビューの帯の省く順は 列・分類 → 今のでないタブ → 検索の欄を中身の幅まで詰める → 並べ替え・絞り込み → 「+ 新規」。
- `App::new` で作る場所の既定を読み込み口のフォルダにした(`start()` を呼ばない試験の起動でも「+ 新規」が出る)。
- 既存の試験は、1行目の「[表] 関係」「+ 新規」と2行目の ⚙ を前提にした所を位置合わせした(アサーションは消さず緩めない)。タブを数える試験は、「+ 新規」で80桁では今のでないタブが省かれるので、描く幅を広げた。ほかの要件の錠を持つものは [header-menu-locks](../_decisions/2026-10-12-header-menu-locks.md) で掛け直す。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-12)が HEAD 33a3b90 と、試験のコミットの worktree(試験と実装が同じコミットのものは実装の直前に試験を重ねた)で確かめた。check.py --evidence は、今の試験が実装前のコードでコンパイルできず全部 gap になったので使わなかった。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-1 | checked | コード: src/ui/bands.rs:195 `header_buttons`・:661 `view_bar` / テスト: src/ui/test_header_menu.rs::test_sr_42_settings_top_right_after_menu・test_ce_25_new_in_view_bar_right_end / 今: 通った(`cargo test --no-fail-fast`) / 前: 11892aa で落ちた(「▦ 表 ▾」が無く、帯の右の端に「a + 新規」が無い) |
| SR-42 | checked | コード: src/ui/bands.rs:171 `SETTINGS_MIN_W`・:183 `settings_button_text`・:214 `header_button_at`、src/ui/app.rs:1116 `press_header_button` / テスト: src/ui/test_header_menu.rs::test_sr_42_settings_top_right_after_menu・test_sr_42_settings_button_opens_settings・test_sr_42_settings_dropped_first_when_narrow / 今: 通った / 前: 11892aa で落ちた(1行目に「設定」が無い) |
| NV-23 | checked | コード: src/ui/bands.rs:122 `key_style`・:661 / テスト: src/ui/test_header_menu.rs::test_nv_23_no_settings_in_view_bar・test_nv_23_key_char_looks_different / 今: 通った / 前: 11892aa で落ちた(帯に「o ⚙」があり、キーの字が名前と同じ見た目) |
| REL-8 | checked | コード: src/ui/screen_menu.rs:31 `screens`・:106 `open_screen_menu`、src/ui/bands.rs:174 `menu_button_text`・:227 `menu_anchor` / テスト: src/ui/test_header_menu.rs::test_rel_8_menu_lists_types_and_switches ほか 3 本 / 今: 通った / 前: 11892aa で落ちた(ヘッダーに「表 ▾」が無い) |
| CE-25 | checked | コード: src/ui/relmap.rs:704(ヘッダーの下に帯の上の余白と同じ行の数を空ける)・:781(窓の上の縁の右の端に「+ 新規」)・:140(クリックの行)、src/ui/bands.rs:976 `new_button_at` / テスト: src/ui/test_relmap_new_row.rs::test_ce_25_relmap_new_same_row_as_view_bar / 今: 通った(`cargo test --bin mdgrid test_relmap_new_row`・`relmap`・`test_header_menu`) / 前: a1d2ec6 で落ちた(関係マップの「+ 新規」が帯の真ん中の行より1行上) |

既存のテストの削除・skip・弱体化: なし

確かめた: 5 / 5

指摘: CE-25 の「関係マップでも表の画面と同じ位置(2行目の右の端)」と、3行のビューの帯が食い違う。関係マップの「+ 新規」はヘッダーのすぐ下の行(既定で3行目)、表の画面では帯の真ん中の行(既定で4行目)で、桁は同じ。行き先: 人に聞き、人が同じ行に揃える案を選んだ([relmap-new-row](../_decisions/2026-10-12-relmap-new-row.md))。直した(タスク 5)。

再照合(検証役、2026-10-12)の結果は、上の表の CE-25 の行に写した。

1298062 で直した既存の試験は、行の期待と行を探す条件だけを変え、アサーションは減っていない。検証役は端末の大きさ・余白・地の色・検索の欄・囲み方の 720 通りで、両方の画面の「+ 新規」が同じ行・同じ桁になることを手元の試しで確かめた。
任意の補足: ビューの帯を全部隠す設定(タブも検索の欄も出さない)では、表の画面に「+ 新規」が出ず、関係マップには出る(この変更の前からの振る舞い。CE-25 の文には反しない)。
