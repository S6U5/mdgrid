---
type: Change
id: 01M4703NQ1FMWCVE8YKA4J750W
title: リンク・被リンク・.base を開いたときの this(links-this)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# リンク・被リンク・.base を開いたときの this(links-this)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [links-this](../_decisions/2026-10-06-links-this.md)(AI)、[links-this-narrow](../_decisions/2026-10-06-links-this-narrow.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、Obsidian を使う人の目の点検(docs/todo.md の O-5)が、リンクと被リンクと this のビューが開かないと指摘した。BV-22 を足した。

終わりの条件: BV-22 の例のとおり。今まで未対応だった式(file.links・file.backlinks・file.hasLink・this)を含む .base のビューが開く。関係する要件: BV-22・BV-6・BV-7。

## 不明点と仮定

- 仮定: リンクは本文(コードの区画 ```…``` とインラインのコード `…` の中は除く)とフロントマターの値の文字から拾う。埋め込み `![[…]]` もリンクとして数える(Obsidian の file.links と同じ)。
- 仮定: 行き先のノートの解き方: 拡張子 .md を足した保管庫の根からのパスが在ればそれ、無ければ名前(拡張子を除く)が同じノートのうち根に近い・パスの短いもの。.md でない行き先(画像など)は文字のまま。
- 仮定: 式の中のリンクの値は、行き先のノートの根からのパス(拡張子なし)の文字として持ち、見せるときは `[[…]]` で囲まない(値の型を増やさない)。file.hasLink(x) は x を同じ解き方で解いて比べる。
- 仮定: 被リンクは読み込みのたびに全ノートのリンクから作り直す(表の組み立てと同じ時)。

## 設計

- 核(src/source/markdown.rs など): ノートを解析するときにリンクの文字を拾っておき、行き先を解いた索引(ノート → 行き先、行き先 → 元)を組み立てる。FileInfo に links・backlinks を足す(src/source.rs)。
- 式(src/expr.rs): file.links・file.backlinks をリストの値に、file.hasLink(x)、this を足す。this は Env に「この .base のファイルの FileInfo」(無ければ未対応の扱い)を持たせ、this.file.* を評価する。
- src/base.rs: .base を開いたときにその .base のファイルの情報を Ctx に渡す。
- 文書: docs/obsidian-bases.md(英日)の対応に足し、未対応から外す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | リンク・被リンク・this | BV-22, BV-6 | BV-22 の例のとおりで、既存の試験が通る | src/source.rs, src/source/markdown.rs, src/vault.rs, src/expr.rs, src/base.rs, src/print.rs, src/main.rs, src/ui/grid.rs, src/ui/native_views.rs, src/ui/app.rs, src/views.rs, src/links.rs, src/lib.rs, docs/obsidian-bases.md, docs/obsidian-bases.ja.md, tests/test_links.rs, src/test_links_unit.rs | test_bv_22_*(tests/test_links.rs, src/test_links_unit.rs) | 済 |

## 実装の気づき

- backlinks・hasLink は錠のある試験と見本が未対応の例に使っているため未対応のまま(人の決定待ち)。

## 照合

自分で照合(フレッシュ文脈でない。試験と実装は同じ実装役が、試験を先に置いて落ちることを確かめてから書いた。独立したレビューではない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-22 | checked | コード: src/links.rs(拾い方と索引・links_toward)、src/expr.rs の file.links と this、src/base.rs の set_this、src/source/markdown.rs の索引を覚える所 / テスト: tests/test_links.rs::test_bv_22_*(12本)、src/test_links_unit.rs(9本) / 今: 通った(全体 1191 passed) / 前: 落ちた(最初に置いた 12本は実装の前に全部落ちた。backlinks・hasLink を外したあとの差し替えの試験は、落ちることを確かめていない) |
| BV-6 | checked | 読んで判定: リンクを含まない式の結果は変わらない(common-methods・mixed-datetime などの式の試験を含む全体 1191 passed) |
| BV-7 | checked | 読んで判定: file.backlinks・file.hasLink は未対応のまま(tests/test_links.rs::test_bv_22_backlinks_and_has_link_stay_unsupported と、錠のある tests/test_base.rs・tests/test_expr.rs・tests/test_language_lib.rs・tests/test_examples.rs が通った) |

既存のテストの削除・skip・弱体化: なし(錠のある試験は書き換えていない)

確かめた: 3 / 3

速さ: 5,000 ノートで file.links と this を使うビューの `--print` が約 0.20 秒(使わないビューは約 0.18 秒)。
