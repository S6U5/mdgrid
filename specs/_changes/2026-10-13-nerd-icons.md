---
type: Change
id: 01M4S3SF04XE2CW1WFRE692CA5
title: アイコン用のフォントの記号を設定で使えるようにする
status: done
size: full
created: 2026-10-13
updated: 2026-10-13
---

# アイコン用のフォントの記号を設定で使えるようにする

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [nerd-icons](../_decisions/2026-10-13-nerd-icons.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が会話 2026-10-11 で、画面の記号に絵文字ではなくアイコン用のフォント(Nerd Font)の記号を使う形を、ほかの端末の道具と同じく設定で使うことにしようと言った。作業の並び(2026-10-12)の5番目。

終わりの条件:
- 既定(`terminal.nerd_icons = false`)は今と1字も変わらない。
- true で、設定のボタン・画面の型のボタンと窓・列の見出しの型の印・カレンダーの題に Nerd Font の記号。絵文字は使わない。どの行も画面の幅を超えない。

関係する要件: SR-45(新しい)、SR-9・CV-6(字の幅)、SR-35(見出しの型の印)。

## 不明点と仮定

- 仮定: 記号は Nerd Font の Font Awesome の字(歯車 U+F013・表 U+F0CE・関係マップ U+F0E8・数 U+F292・日付 U+F073・日時 U+F017・リスト U+F0CA・札 U+F192・チェック U+F046)。どの Nerd Font にも入っている字を選んだ。
- 仮定: 既定の画面の型の窓の項目には記号を付けない(今と同じ)。true のときだけ項目の頭に記号を付ける。
- 仮定: 設定は全体だけ(端末の字形の話なので、`terminal.nerd_font` と同じ)。

## 設計

変える場所: 新しい `src/ui/glyphs.rs`(記号の種類 `Glyph` と、設定に従って字を返す `glyph`・カレンダーの題の頭 `calendar_head`)、`src/config.rs`(`terminal.nerd_icons` を読み、`--print-config --resolved` に出す)、`src/schema.rs`(項目)、`src/ui/app.rs`(設定を当てる)、`src/ui/bands.rs`(設定のボタンと画面の型のボタン)、`src/ui/screen_menu.rs`(窓の項目)、`src/ui/view.rs` の `head_text`(見出しの型の印)、`src/ui/calendar.rs`(題)、`docs/config.md`・`docs/config.ja.md`。

判断: 記号を1か所(`glyphs.rs`)に集め、使う所はそれを通す(記号を足す・変えるときに1か所で済む)。却下: 丸い札の端の設定(`terminal.nerd_font`)にまとめる(提案に記録。既定の振る舞いが合わない)。

要件との対応: SR-45 は上の全部。

検証: `cargo test --bin mdgrid test_nerd_icons`、`cargo test --no-fail-fast`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | SR-45 | 実装の前に落ちる | `src/ui/test_nerd_icons.rs` `src/ui/mod.rs` | test_sr_45_* | 済 |
| 2 | 記号を glyphs.rs に集め、設定で切り替える | SR-45 | 1 の試験と既存の試験が通る | `src/ui/glyphs.rs` `src/ui/mod.rs` `src/config.rs` `src/schema.rs` `src/ui/app.rs` `src/ui/bands.rs` `src/ui/screen_menu.rs` `src/ui/view.rs` `src/ui/calendar.rs` `docs/config.md` `docs/config.ja.md` | test_sr_45_* | 済 |

## 実装の気づき

- 実装を書きかけのあいだに試験を書いたので、実装の前に落ちることは、実装の前のコミットを書き出した木に試験だけを足して確かめた(記号を使う3本が落ち、既定の1本は今の振る舞いを守る試験なので通った)。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-13)が 412342a を書き出した木と、実装の前の bc562ac と同じ中身の木(別の target)で確かめた。正しさの不具合は無く、試験の見ていない所は試験を足した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-45 | checked | コード: src/ui/glyphs.rs `plain`・`nerd`・`glyph`・`calendar_head`、src/config.rs(`terminal.nerd_icons`。既定 false、全体だけ)、src/ui/bands.rs `menu_button_text`・`settings_button_text`、src/ui/screen_menu.rs `entry`、src/ui/view.rs `head_text`、src/ui/calendar.rs(題)、src/schema.rs(Scope::Global) / テスト: src/ui/test_nerd_icons.rs の7本(既定の字の記号、ヘッダーと見出しの印、全部の型の印、関係マップのボタン、画面の型の窓、カレンダーの題、あいまいな字を全角とする設定での幅とクリックの当たり) / 今: 通った(`cargo test --bin mdgrid test_nerd_icons`) / 前: bc562ac で記号を使う3本が落ちた(設定の名前が知らない項目で警告され、画面の中身の確かめまで進まない。落ちる向きは正しい)。既定の1本は今の振る舞いを守る試験で、前も通った |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1

指摘と行き先:
- 関係マップ・数・日時・チェック・リスト・札の記号と、あいまいな字を全角とする設定での幅・クリックの当たりに試験が無かった。足した(test_sr_45_nerd_icons_every_type_mark・test_sr_45_nerd_icons_relmap_button・test_sr_45_nerd_icons_ambiguous_wide_click)。
- 文書と設定の説明の既定の記号の並びに、チェックの印(部品の形。既定 ✓)が無かった。足した。
- 全体の試験で e2e の SR-8 の3本が落ちたのは、ほかの cargo test と並んだ負荷による時間切れで、単独では通る(今回の変更とは関係ない)。
