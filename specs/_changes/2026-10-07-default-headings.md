---
type: Change
id: 01M48ZE08CQ4VC2W7QBV9SRHPM
title: displayName の無い列の見出しを Obsidian の既定にする(default-headings)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# displayName の無い列の見出しを Obsidian の既定にする(default-headings)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-default-headings.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した BV-24 のとおり(人の判断待ちの表の2)。

## 不明点と仮定

- 仮定: 見出しの文字は英語の Obsidian の語で、日本語の画面でも同じにする(Obsidian の日本語の画面の語は文書に無く、確かめられない)。

## 設計

- src/base.rs に `default_title(id)` を置き、Base::title の既定と、mdgrid のビュー・--print の見出しの既定(id のまま)をこれに替える。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 既定の見出しを入れ、試験を合わせる | BV-24 | 採択した確かめ方 | src/base.rs, src/print.rs, src/ui/grid.rs, src/ui/native_views.rs, tests/, src/ui/test_*.rs, tests/golden/, docs/, README.md, README.ja.md, specs/test-locks.json, specs/_decisions/ | test_bv_24_default_headings(tests/test_default_headings.rs) | 済 |

## 実装の気づき

- \`--print --format json\` の鍵も見出しなので変わる(BV-24 の確かめ方のとおり)。json の鍵を見ていた錠のある試験(BV-22・BV-7)は、補いの決定 2026-10-07-default-headings-locks の「錠:」の行で掛け直した。
- 画面のゴールデン bv_7 と、式の列の寄せの試験(CV-4。錠なし)の見出しの幅を、式の名前に合わせて直した。

## 照合

自分で照合(見出しの既定を1か所の関数にした)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-24 | checked | コード: src/base.rs の default_title / テスト: tests/test_default_headings.rs::test_bv_24_default_headings、tests/test_default_headings.rs::test_bv_24_display_name_wins / 今: 通った(全体 1322 passed) / 前: 落ちた(見出しが file.name のままで落ちた) |

既存のテストの削除・skip・弱体化: なし(見出しの文字を新しい既定に替えた。錠のある試験は人の決定で掛け直した)

確かめた: 1 / 1
