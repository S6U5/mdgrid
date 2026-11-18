---
type: Change
id: 01M4RWAXF5SVFTK13YGGBJMNQ6
title: 列の型を設定と見出しの型の印から選び直せるようにする
status: done
size: full
created: 2026-10-13
updated: 2026-10-13
---

# 列の型を設定と見出しの型の印から選び直せるようにする

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [column-types](../_decisions/2026-10-13-column-types.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | |
| 照合 | 済 | この記録の「照合」 |

## 要求

作業の並び(2026-10-12)の4番目「列の追加と型の変更」。人が 2026-10-12 に作業の並びを任せた。列を足す(CE-28)と見出しの型の印(SR-35)はあるので、列の型を選び直す形を足す。

終わりの条件:
- `[types] due = "number"` で、推定より設定が勝つ。`"auto"` で推定に戻る。知らない型は警告。
- パレット・操作の一覧・見出しの型の印のクリックで型の窓が開き、選んだ型が views.toml の表の範囲に書かれ、すぐ表に効く。ノートは書き換えない。
- `file.*`・`formula.*` の列では出さない。

関係する要件: CE-35(新しい)、CE-2(優先の順)。

## 不明点と仮定

- 仮定: 窓の項目は「自動(推定の型)」・テキスト・数・チェック・日付・日時・リスト。今の型(設定が無ければ自動)に印。
- 仮定: 読むだけの起動でも型は選べる(ノートは書かず、views.toml に書くだけ)。外れたら: 読むだけでは出さない。
- 仮定: 見出しの型の印は、印の字とその後ろの空白の2桁をクリックの当たりにする。名前のほかの所は今までどおり並べ替え。

## 設計

変える場所: `src/profile.rs`(Profile に `types` の層。`read_key`・`write`・`resolve` で列ごとに狭い範囲が勝つ。`"auto"` は推定に戻す)、`src/schema.rs`(項目 `types`)、`src/ui/profile.rs`(重ねた `types` を App に当てる)、`src/ui/grid.rs` の `column_kind`(設定の型が先)、新しい `src/ui/type_menu.rs`(窓。screen_menu.rs と同じ作り)、`src/ui/keymap.rs`(Action::ColumnType・Mode::ColumnType・パレットの命令)、`src/ui/menu.rs`(操作の一覧の列の節)、`src/ui/app.rs`(クリックの振り分け)、`src/ui/view.rs`(見出しの型の印の当たり)、`src/i18n.rs` と `locales/`(文言)、`docs/config.md`・`docs/config.ja.md`・`docs/keys*.md`。

判断: 型は mdgrid の設定の層に置き、ほかの道具の型の設定は読むだけ(提案の却下した案)。表の範囲への書き込みは、見た目の区画と同じ `views::save_table_profile`(views.toml の `[[table]]`)。却下: ビューの設定にだけ持つ(提案)。

要件との対応: CE-35 は上の全部。CE-2 は `column_kind` の優先の順。

検証: `cargo test --bin mdgrid test_column_types`、`cargo test --test test_column_types_config`、`cargo test --no-fail-fast`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | CE-35, CE-2 | 実装の前に落ちる | `src/ui/test_column_types.rs` `src/ui/mod.rs` `tests/test_column_types_config.rs` | test_ce_35_* | 済 |
| 2 | 設定の層 `[types]` を読み、重ね、列の型に効かせる | CE-35, CE-2 | 設定の試験が通る | `src/types.rs` `src/profile.rs` `src/schema.rs` `src/config.rs` `src/ui/profile.rs` `src/ui/grid.rs` `src/ui/app.rs` `locales/` `docs/config.md` `docs/config.ja.md` | test_ce_35_types_* | 済 |
| 3 | 型の窓と、パレット・操作の一覧・見出しの印からの入口 | CE-35 | 画面の試験が通る | `src/ui/type_menu.rs` `src/ui/keymap.rs` `src/ui/menu.rs` `src/ui/app.rs` `src/ui/view.rs` `src/ui/grid.rs` `src/ui/mod.rs` `src/ui/test_column_types.rs` `src/schema.rs` `locales/` `docs/keys.md` `docs/keys.ja.md` `docs/config.md` `docs/config.ja.md` `tests/golden/sr_5.txt` | test_ce_35_* | 済 |
| 4 | 既存の試験を通す | CE-35 | `cargo test` が通る(前からの test_cliff を除く) | `tests/` `src/` | 既存の試験 | 済 |
| 5 | 照合の指摘: セルの入力・新しいノート・リストの選択も設定の型、「自動」は表より広い範囲の型で決める、読むだけでもパレット、計算の列の印は並べ替え、置き場が無ければ書かないと知らせる、札の判定 | CE-35 | 指摘の試験が通る | `src/ui/grid.rs` `src/ui/input.rs` `src/ui/new_note.rs` `src/ui/listpick.rs` `src/ui/type_menu.rs` `src/ui/keymap.rs` `src/ui/view.rs` `src/ui/mod.rs` `src/ui/test_column_types_more.rs` `locales/` | test_ce_35_input_follows_set_type / test_ce_35_auto_undoes_wider_type / test_ce_35_palette_in_readonly / test_ce_35_set_type_not_chips | 済 |

## 実装の気づき

- 受け入れの試験の3か所は試験の側の誤りだったので直した(錠の無い新しい試験): 窓の選んでいる行の書き方(`>   数`)を拾えなかった・「自動」へ上に戻らなかった、見出しの型の印は `[look.style] icons = true` で出る、見出しのクリックの並べ替えは `settings.sorts` に入る。
- 既定の無い表の項目は、錠のある試験(test_cli_12_table_defaults_are_real_defaults)が項目ごとの確かめ方を持つ。試験を変えないよう、`types` の既定を空の表 `{}` にし、`--print-config` は表の項目なら既定があっても書き方の例をコメントで出す形にした。
- 型の窓のモード(`column_type`)のキーが増え、ヘルプの行の数が 238 から 243 になった。ヘルプの写し(tests/golden/sr_5.txt)は、その1か所だけ作り直した(試験は変えていない)。

## 照合

検証役(フレッシュ文脈のサブエージェント、2026-10-13)が 32f4223 を書き出した木と、試験のコミット b1682e1 の木(別の target)で確かめた。gap の2つと小さな指摘は直した(タスク 5)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-35 | checked | コード: src/profile.rs `read_types`・`Resolved::apply`(列ごとに狭い範囲が勝ち、`auto` で外す)、src/types.rs `Kind::NAMES`・`parse`、src/schema.rs(項目 `types`)、src/ui/grid.rs `column_kind`・`col_type`・`guessed_kind`、src/ui/type_menu.rs(窓・`set_column_type`)、src/ui/view.rs `type_mark_at`、src/ui/menu.rs(操作の一覧)、src/ui/keymap.rs(`column_type`。読むだけの命令)、src/ui/input.rs・src/ui/new_note.rs・src/ui/listpick.rs(入力が設定の型に従う) / テスト: src/ui/test_column_types.rs の6本、src/ui/test_column_types_more.rs の4本、tests/test_column_types_config.rs の2本 / 今: 通った / 前: b1682e1 で8本中7本が落ちた(表でない値の警告の1本は、前も知らない項目として警告が出るので見分けられない) |
| CE-2 | checked | コード: src/ui/grid.rs `column_kind`(設定の型 → ルートフォルダの型の設定 → 推定) / テスト: src/ui/test_column_types.rs::test_ce_35_types_setting_wins_over_inference、既存の test_CE_2・test_ce_2_mixed_date_datetime_* / 今: 通った / 前: b1682e1 で前者が落ちた |

既存のテストの削除・skip・弱体化: なし(tests/golden/sr_5.txt は行の数の1か所だけ。新しい試験は実装のコミットで3か所直した。理由は「実装の気づき」)

確かめた: 2 / 2

指摘と行き先:
- セルの入力・新しいノート・リストの選択が設定の型を見ていなかった(gap)。直した: `col_type`(設定の型が先)に揃えた(test_ce_35_input_follows_set_type)。
- 「自動」の判定が表の範囲の値で決まり、広い範囲の型に戻ることがあった(gap)。直した: 表より広い範囲の層に型があれば `"auto"` を書く(test_ce_35_auto_undoes_wider_type)。
- 読むだけの起動でパレットに出なかった。直した: 読むだけの命令に移した(test_ce_35_palette_in_readonly)。
- 計算の列の見出しの印のクリックが並べ替えを奪っていた。直した: 計算の列は印の当たりにしない。
- views.toml の置き場が無いときも「書いた」と知らせていた。直した: この起動の間だけと知らせる。
- 設定で文字でない型にした列が札になりえた。直した: 札の列の判定で設定の型を見る(test_ce_35_set_type_not_chips)。

任意の補足: 書き戻しの側(src/changes.rs のリストの書き方の判定・src/source.rs)は今も `Source::kind` を見る。設定でリストにした列への書き方は、値がリストなら同じ形になる。
