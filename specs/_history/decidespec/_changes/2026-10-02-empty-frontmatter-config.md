---
type: Change
id: 01M3XZ3YAYN6E8PQTA2G6AJJVP
title: 空のフロントマターにも書き、フロントマターを足すかを設定で選べる(empty-frontmatter-config)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# 空のフロントマターにも書き、フロントマターを足すかを設定で選べる(empty-frontmatter-config)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [empty-frontmatter-config](../_decisions/2026-10-02-empty-frontmatter-config.md)(人)、[bulk-skip-mismatch](../_decisions/2026-10-02-bulk-skip-mismatch.md)(人。実装は checkbox-three-state で済、試験 test_ce_4_ce_10_bulk_skips_mismatched_rows) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): 中身の無いフロントマターなどの書かない条件を決め、設定で変えられるようにしたい。設定で変えられるのはフロントマターの無いノートと空のフロントマターのノートだけ。空のフロントマターは既定で書ける。

終わりの条件: `---\n---` のノートのセルに書けて、区切りの間にキーの行が足される。設定 `add_frontmatter = false` では、フロントマターの無いノートと空のフロントマターのノートが読むだけで理由が出る。ほかの書かない条件(WB-5)は設定で変わらない。見本の設定と README に項目を書く。関係する要件: WB-3・WB-5・CLI-3・CE-10。

## 不明点と仮定

- 仮定: 設定の項目は最上位の `add_frontmatter`(真偽、既定 true)。外れたら: `[write]` の表に移す。
- 仮定: 「同じキーが2回」の見本のノート(examples/vault/メモ/週の振り返り.md)は、本文に読むだけになる理由が書いてあるので、異常系の見本として残す。

## 設計

核の `writeback::apply` は、読み方が `EmptyFrontmatter` のとき、閉じの `---` の行の前にキーの行を足す(足す行の改行は開きの行に合わせる。書き方は今のキーを足すときと同じ)。読み直しの検査は、足した行の外が元のバイトと同じで、書いた値だけを持つことを確かめる。`add_frontmatter` は Config に持ち、Source(Markdown)を開くときに渡す。false なら NoFrontmatter・EmptyFrontmatter のセルを lock(理由: フロントマターが無い・空のフロントマター(設定で書かない))にし、apply は呼ばれない(画面が lock を見る)。核の apply は設定を知らない(書く判断は Source と画面の lock)。ただし Markdown の preview と save でも同じ判断を通し(外の変更の判定(WB-4・WB-16)のあと)、設定で書かないノートは書かない(レビューで直した)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 空のフロントマターに書く・add_frontmatter の設定 | WB-3, WB-5 | 空のフロントマターに足す(LF・CRLF・本文が変わらない)、設定 false で2つの形が読むだけと理由、ほかの WB-5 は変わらない、設定の読み取りと警告、見本と README | src/writeback.rs, src/test_writeback_unit.rs, src/frontmatter.rs, src/source/markdown.rs, src/source/test_markdown_unit.rs, src/config.rs, src/main.rs, src/ui, tests, examples, README.md, docs/design.md | test_WB_3, test_WB_5 | 済 |

受け入れの試験と、今の振る舞い(空のフロントマターは読むだけ)を前提にした既存の試験の書き直しは、実装を見ていない役が先にする。

## 実装の気づき

- タスク1: lock だけでは Source の preview・save を直に呼ぶ道が守れないので、Markdown の save も設定 false の2つの形を NotEditable で断るようにした(基準の検査で Changed を先に返し、そのあとで判断する。preview は書かないので見ない。Changes を通る道は set の Skip で止まる)。
- タスク1(再レビューの残り、低): 設定 false で外の変更の上に書いたあと、preview は書けるように見えるが save は NotEditable で断る(データは守られる。preview でも基準が同じときだけ設定を見る形にできる)。ハードリンクかつフロントマターの無いノートを直に save すると理由が設定の方になる(画面の lock はハードリンクが先)。基準の読み取りが重なる(正しさには影響なし)。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-3 | checked | コード: src/writeback.rs:133 / テスト: tests/test_empty_frontmatter.rs::test_wb_3_empty_frontmatter_gets_key_between_delimiters / 今: 通った(cargo test) / 前: 落ちた(7f3afba の写しでは空のフロントマターが Err(ReadOnly(EmptyFrontmatter))。変異で空を断る・設定を無視する・設定の読み取りを無視する → それぞれ落ちた) |
| WB-5 | checked | コード: src/source/markdown.rs:622 / テスト: tests/test_empty_frontmatter.rs::test_wb_5_other_forms_unaffected_by_setting / 今: 通った(cargo test) / 前: 落ちた(7f3afba の写しで test_unwritable の9件が落ちた。変異で設定 false のときほかの形の理由を変える → 落ちた) |
| CE-10 | checked | コード: src/ui/input.rs:586 / テスト: src/ui/test_checkbox.rs::test_ce_4_ce_10_bulk_skips_mismatched_rows / 今: 通った(cargo test) / 前: 落ちた(4cdfe13 の親では `done: yes` の行が false で上書きされた) |

既存のテストの削除・skip・弱体化: なし(空のフロントマターを読むだけの表から書ける側の対照へ移し、バイトの完全一致で確かめる。残る7つの形は全段のまま)

残した気づき: main.rs の `set_add_frontmatter(config.add_frontmatter)` の受け渡しを消す変異は自動の試験をすり抜ける(検証役が疑似端末で、--config の false で読むだけと理由が出ることを確かめた)。受け渡しの試験は oss-config の --print-config の試験の形で後で足せる。

確かめた: 3 / 3
