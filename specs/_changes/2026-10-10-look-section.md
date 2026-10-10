---
type: Change
id: 01M4HT5RA4TEEFQX68ET5MMC8K
title: 設定の画面の見た目の区画とテンプレート(SR-43)
status: done
size: full
created: 2026-10-10
updated: 2026-10-10
---

# 設定の画面の見た目の区画とテンプレート(SR-43)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-10-look-section.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

設定の画面の「見た目」の区画で、テーマ・組・丸い札の端を選び、反映で効かせて look.toml に残す。組み合わせを名前でテンプレートとして保存・選ぶ・消す。config.toml には書かない(SR-43)。

## 不明点と仮定

- 仮定: 項目は「テーマ」「組」「丸い札の端」の3行で、Enter で選び手(一覧)を開いて選ぶ。テンプレートは項目の下に並べ、Enter で写しに当て、削除の操作(d)で消す。末尾に「名前を付けて保存」と「config.toml に戻す」。
- 仮定: `theme = "auto"` を選んだときは、反映の時点で端末の地の明るさを問い合わせずに、起動のときに決めたテーマのまま(次の起動で auto として決める)。

## 設計

- 核 `mdgrid::look`: `Look { theme: Option<String>, preset: Option<Preset>, nerd: Option<Nerd> }` と `templates: Vec<(String, Look)>`。`look.toml` を読み(壊れていれば警告して空)、一時ファイル → 名前の変更で書く。`apply(&mut Config, &LookFile)` で config に重ねる(main の nerd_font・theme の auto の決定の前)。
- 画面: Draft に見た目の写し(`look`)を持ち、`pending` に数える。反映で App の theme・style・nerd_font を当て直し、look.toml に書く。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | look.toml の読み書きと config への重ね | SR-43 | 試験が通る | src/look.rs, src/main.rs | test_sr_43_look_*(src/test_look_unit.rs) | 済 |
| 2 | 見た目の区画とテンプレート | SR-43 | 試験が通る | src/ui/settings*.rs, src/ui/look_section.rs, src/i18n.rs | test_sr_43_*(src/ui/test_look_section.rs) | 済 |
| 3 | 説明書・設定の文書・録画の台本 | SR-43 | 試験が通る | docs/, demos/ | — | 済 |

## 実装の気づき

- 見た目の区画の読み書きは毎回 look.toml を読む(小さいファイル。ほかの mdgrid で足したテンプレートも出る)。
- テーマの auto を選んだときは、反映では今のテーマのまま(起動のときに端末の地で決める)。
- 録画の台本 demos/look.tape を足して手元で撮った。VHS の撮り方で一度だけ画面が空に写ったが、同じ台本の撮り直しでは正しく写り、mdgrid の標準エラーは空・終了コードは 0 だった(録画の側の揺れ)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-43 | checked | テスト: src/test_look_unit.rs(読み書き・テンプレート・壊れたファイルと使えない値の警告・config への重ね)、src/ui/test_look_section.rs::test_sr_43_choose_apply_and_save・test_sr_43_templates_save_use_delete_and_reset・test_sr_43_readonly_applies_without_writing、tests/test_look_e2e.rs(本物の実行ファイルで look.toml の組が起動で効く・壊れた look.toml は警告)、golden nv_18 |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
