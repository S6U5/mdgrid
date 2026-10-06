---
type: Change
id: 01M474XHC4GARVW7PDA960GJ15
title: 文書の古さと英語の文言の直しの束(docs-refresh)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# 文書の古さと英語の文言の直しの束(docs-refresh)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 要件の意味を変えない(文書と英語の文言だけ) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、3回目の点検(docs/todo.md の A-10・A-11・A-12・A-13)が見つけた文書の古さと英語の文言を直す。

- A-11: 英語の `1 rows`・`(1 rows)` などの単数と複数(1 のとき単数)。
- A-12: `--print-config` の theme の説明に7つの名前を並べる(src/config_items.rs と docs/config.md・ja.md)。
- A-10: README の「Use it from scripts」で、フォルダの `--print` は行の名前が無いので `--with-path` を勧める。
- A-13: docs/obsidian-bases.md の英語に残る「(空)」「画面の文言は日本語」などの古い記述、config.md の new_note のフォルダ(.base の file.inFolder のフォルダに作る)、keys.md の `K`(本文の先頭も見せる)、README に集計の行と `%`・テーマの見本の置き場、`--help` の使い方に `.md` のファイル、README の「Seven color themes」の言い方、examples/themes/*.toml の注釈を英語に。

終わりの条件: 上のどれもが今の振る舞いと合う。文書を突き合わせる試験(config・keys・bases の文書の試験)が通る。関係する要件: SR-23・SR-22・CLI-12・BV-21。

## 不明点と仮定

- 仮定: 単数と複数は英語の文言の表に単数の形を足し、数が 1 のとき選ぶ。日本語は変えない。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 文書と英語の文言 | SR-23, SR-22, CLI-12 | 上の終わりの条件のとおり | README.md, README.ja.md, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, docs/config.md, docs/config.ja.md, docs/keys.md, docs/keys.ja.md, src/i18n.rs, src/config_items.rs, src/main.rs, src/ui/bands.rs, src/ui/view.rs, examples/themes/, src/ui/test_plural_en.rs, src/ui/mod.rs, docs/design.md, docs/assets/ | test_sr_23_plural_*(src/ui/test_plural_en.rs)と、読んで判定 | 済 |

## 実装の気づき

- 単数と複数は使う側(多くが範囲の外のファイル)でなく `Msg::fill` の中で選ぶ(`Msg::singular` が 24 の英語の文言の単数の形と数の位置を持つ)。保存の文言は `file(s)` をやめ `file`・`files` にした。docs/design.md の言語の節に書いた。
- 見本の日付が古くなる(A-13 の一部)は、この記録の要求に入れていないので残した。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。文書と文言の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-23 | checked | コード: src/i18n.rs の Msg::singular と fill / テスト: src/ui/test_plural_en.rs::test_sr_23_plural_*(3本) / 今: 通った(全体が通った) / 前: 落ちた(英語のヘッダーが `1 rows` で落ちた)。実物: 保存の確認の見出しが `1 file` |
| SR-22 | checked | 読んで判定: docs/keys.md・keys.ja.md の detail の説明に本文の先頭を足し、キーの表と突き合わせる試験が通った |
| CLI-12 | checked | 読んで判定: src/config_items.rs と docs/config.md・ja.md の theme の説明に7つの名前、new_note の file.inFolder のフォルダの説明。設定の文書の試験が通った |

既存のテストの削除・skip・弱体化: なし

確かめた: 3 / 3

README の画像 demo-save.svg と demo.gif を撮り直した(`1 file`)。
