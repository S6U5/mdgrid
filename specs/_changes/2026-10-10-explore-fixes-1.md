---
type: Change
id: 01M4JA0AYWG38NRAYW7JRHNRBJ
title: 点検で見つかった不具合の直し(1): 曜日の言語、見た目の区画が config.toml の部品の形と auto を消す
status: done
size: bugfix
created: 2026-10-10
updated: 2026-10-10
---

# 点検で見つかった不具合の直し(1)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 今の要件どおりに動かない不具合(SR-23・SR-43) | — |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人の依頼で、サブエージェントにアプリを操作させて点検した(会話 2026-10-10)。見つかった不具合のうち次を直す。

1. 英語の画面でも、日付の形の `ddd` が日本語の曜日(木)で出る(SR-23: 画面の文言は英語と日本語)。
2. 見た目の区画でテーマだけを変えて反映しても、look.toml に組まで書かれ、config.toml の `[style]` の部品ごとの形が効かなくなる(SR-43: 区画で選んだものだけを残す)。
3. 丸い札の端の既定 `auto` が、区画では判定した結果の `false` と出て、反映すると look.toml に `false` と書かれる。日本語の画面でも `false` のまま出る。

## 不明点と仮定

- 仮定: 打ち込みの曜日は、どちらの言語の名前でも受け付ける(言語を切り替えても前の打ち込みが通る)。
- 仮定: look.toml には、区画で開いたときから変えた項目と、もともと look.toml にあった項目だけを書く。

## 設計

- types.rs: 曜日の短い名前を今の言語で選ぶ(ja: 日〜土、en: Sun〜Sat)。読むときは両方の名前を受ける。
- look_section.rs: 写しの始まりは、look.toml の値 > 設定の値(theme の auto・nerd_font の auto を含む)> 今の画面。反映では、開いたときから変えた項目か、look.toml にあった項目だけを書く。丸い札の端の選び手の文言を日英に訳す。
- App は設定の theme・nerd_font が auto かどうかを覚える。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 曜日を画面の言語で | SR-23 | 英語の画面で Thu | src/types.rs | test_sr_23_weekday_follows_language(src/test_weekday_lang_unit.rs) | 済 |
| 2 | 見た目の区画が変えた項目だけ残す・auto を保つ | SR-43 | テーマだけ変えると look.toml に theme だけ。auto のまま | src/ui/look_section.rs, src/ui/app.rs, src/i18n.rs | test_sr_43_only_changed_items_saved・test_sr_43_auto_kept(src/ui/test_look_keep.rs) | 済 |

## 実装の気づき

- 日本語の曜日の名前の表は、日本語のリテラルを許す表(tests/test_no_japanese_literals.rs。錠あり)が `const WEEKDAY_NAMES` の行で許しているので、名前をそのままにした。
- 丸い札の端の選び手の文言は「true(オン。Nerd Font)」のように、設定の値を先に、訳を後ろにした(錠のある試験が値の頭で選ぶため。config.toml との対応も見える)。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-23 | checked | テスト: src/test_weekday_lang_unit.rs::test_sr_23_weekday_follows_language |
| SR-43 | checked | テスト: src/ui/test_look_keep.rs::test_sr_43_only_changed_items_saved・test_sr_43_auto_kept、src/ui/test_look_section.rs(今までのまま通る) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
