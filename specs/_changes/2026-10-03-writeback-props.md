---
type: Change
id: 01M3ZX06TE3875QYKRM8GQMWF2
title: 書き戻しの性質の試験(writeback-props)
status: done
size: bugfix
created: 2026-10-03
updated: 2026-10-03
---

# 書き戻しの性質の試験(writeback-props)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 既存の要件(WB-1・WB-5・WB-6・WB-7)を確かめる試験を足すだけ | なし |
| 設計 | 対象外: 条件に当たらない(試験の追加。見つかった不具合は bugfix として直す) | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 4080306(試験の追加だけ。コードの変更なし。見つけた食い違いは人の判断待ち) |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の Q-8。書き戻しの安全(WB-1: 対象のキーの値の外を1バイトも変えない、WB-6・WB-7: 書いた値を読み直すと意図した値と型になる、WB-5: 読めないノートは読むだけ)を、例の試験だけでなく、ランダムに作ったノートと値で確かめる(性質の試験)。読み取りがどんなバイト列でも panic しないことも確かめる。見つかった不具合は直す。

終わりの条件: 性質の試験が `cargo test` で回り(既定の回数)、通る。試験は意図して壊した書き戻し(変異)で落ちる。関係する要件: WB-1・WB-5・WB-6・WB-7。

## 不明点と仮定

- 仮定: cargo-fuzz は nightly が要り、この環境に rustup と nightly が無い。読み取りが panic しないことは proptest の任意のバイト列で確かめ、cargo-fuzz の的は置かない。外れたら: fuzz/ を足す(CI の外)。
- 仮定: proptest を dev-dependencies に足す(cargo deny のライセンスの許可に収まるかを確かめる)。試験の時間を抑えるため、ケースの数は既定(256)以下にする。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 書き戻しと読み取りの性質の試験(と、見つかった不具合の直し) | WB-1, WB-5, WB-6, WB-7 | 性質の試験が通り、変異で落ちる | tests/test_writeback_props.rs, Cargo.toml, Cargo.lock, src/writeback.rs, src/frontmatter.rs | test_wb_1_props_*, test_wb_5_*, test_wb_7_* | 済 |

## 実装の気づき

- 性質の試験が食い違いを1件見つけた: Null を書くと値の前の区切りの空白も消える(`k0: a` → `k0:`、`k:  # c` に Null → `k: # c`)。WB-1(人)の「値の範囲の外を変えない」と CE-9 の `key:` がぶつかる。人の判断待ち(判断の束の8件目)。それまで試験は Null のときだけ区切りの空白を許す(tests/test_writeback_props.rs の注)。
- cargo-fuzz は nightly が要るので置かない。任意のバイト列で panic しないことは proptest で確かめる。

## 照合

書込なしのフレッシュ文脈の照合役で照合した(2026-10-03、コミット済みの clone で)。既にある振る舞いを縛る試験なので、「前に落ちる」は変異で確かめた(9つの変異がすべて落ちた)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-1 | checked | コード: src/writeback.rs の apply_edits(値の範囲だけを splice)・new_key_lines / テスト: tests/test_writeback_props.rs::test_wb_1_props_existing_scalar_key・test_wb_1_props_added_key_is_one_line / 今: 通った(3回) / 前: 変異で落ちた(行末のコメントの前の空白を詰める・足す行を常に LF にする) |
| WB-5 | checked | コード: src/frontmatter.rs の parse_full・parse_entries、src/writeback.rs の apply_inner / テスト: test_wb_5_props_broken_notes_are_read_only・arbitrary_bytes_do_not_panic・mutated_notes_do_not_panic / 今: 通った(3回) / 前: 変異で落ちた(BOM を読むだけにしない・同じキーを読むだけにしない・特定の入力で panic) |
| WB-6 | checked | コード: src/writeback.rs の verify・verify_entries・apply_edits / テスト: test_wb_6_props_written_value_reads_back / 今: 通った(3回) / 前: 変異で落ちた(2つ目の編集を無視する・数を文字列で書く) |
| WB-7 | checked | コード: src/writeback.rs の render_value・plain_safe・double_quote・single_quote / テスト: test_wb_7_props_text_reads_back_as_same_string / 今: 通った(3回) / 前: 変異で落ちた(`no` を囲まない・`'` を二重にしない) |

既存のテストの削除・skip・弱体化: なし(4080306 は新しい試験のファイルと dev-dependencies・錠・記録だけ。Null を書くときだけ区切りの空白を許す所は、人の判断待ちの注つきで、ほかの値の検査は緩めていない)

残した気づき: Null を書くと値の前の区切りの空白が消える(WB-1 と CE-9 のぶつかり。人の判断待ち)。cargo-fuzz は nightly が無く置いていない。

確かめた: 4 / 4
