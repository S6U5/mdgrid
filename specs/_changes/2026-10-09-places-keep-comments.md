---
type: Change
id: 01M4EC58XDR3Q1JM9JG0PRX6ZX
title: 登録の保存で、places.toml の手書きのコメントと読めない行を消さない(CLI-18)
status: done
size: bugfix
created: 2026-10-09
updated: 2026-10-09
---

# 登録の保存で、places.toml の手書きのコメントと読めない行を消さない(CLI-18)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: bugfix | — |
| 仕様化 | 対象外: bugfix(CLI-18 の「places.toml は手で書いてもよく」「読めない行は警告にして飛ばす」に合わせる) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 対象外: bugfix(1か所) | — |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

「この表を登録」で保存すると、places.toml を読み直して全部を書き直すため、手で書いたコメント・空行・`~` のパス・読めなかった行が消えていた(前の作業で気づいた残り)。足すときは末尾に付け、置き換えるときはその区画だけを書き換え、ほかは文字のまま残す。

## 設計

- src/places.rs: save はファイルの文字を `[[place]]` の見出しの行で区画に分け、各区画を TOML として読んで name を比べる。同じ name の区画は、区画の末尾のコメントと空行を残して中身だけを置き換える。無ければ末尾に足す。ほかの区画・見出しの前のコメント・読めない区画は文字のまま。
- toml_edit などの依存は足さない(区画ごとの文字の置き換えで足りる)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-18 | checked | コード: src/places.rs(save・upsert) / テスト: src/test_places_keep_unit.rs::test_cli_18_add_keeps_hand_written_text・test_cli_18_replace_only_that_block・test_cli_18_broken_block_is_kept、既存の src/test_places_unit.rs と src/ui/test_places.rs |

既存のテストの削除・skip・弱体化: なし

1か所の bugfix のため、メインが試験で確かめた。workspaces.toml(WS-1)は入れ子の区画があるので、この形にしていない(読めない行があれば書かずに理由を出すまで)。

確かめた: 1 / 1
