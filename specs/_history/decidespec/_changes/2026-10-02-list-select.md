---
type: Change
id: 01M3X1J59JMMMJ8FRDE6T56B3D
title: リストの値(tags など)を候補から選んで付け外しする(list-select)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# リストの値(tags など)を候補から選んで付け外しする(list-select)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [list-select](../_decisions/2026-10-02-list-select.md)(人)、[list-select-details](../_decisions/2026-10-02-list-select-details.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): tags を画面でうまく選べるようにしたい。今あるタグの中から、打って検索して候補を出す形で。

今は、リストの値は版1では読むだけ(CE-6)で、要素の足し引きは次の段(CE-13)。これを版1に入れる。リレーションの「1つのプロパティに複数のリンク」(変更の記録 relations)も同じ仕組みに乗る。

終わりの条件: リストの列のセルで、保管庫の中に今ある要素から打って絞り込んだ候補を選んで付け外しでき、候補に無い要素も足せ、選んだ行に一括で足す・外すができ、書き戻しは元の書き方を保つ。関係する要件: CE-3・CE-6・CE-8・CE-10・CE-13・WB-1・WB-7・WB-17・SR-16。

## 不明点と仮定

- 問い → 答え(要約): 選べる列の範囲 → リストの列の全部(tags に限らない)。
- 問い → 答え(要約): 一括 → 選んだ行に足す・外すを一括で(ほかの要素は残し、一部の行だけが持つ要素は「一部」の印)。
- 問い → 答え(要約): キーが無い・空のノートに初めて書く形 → Obsidian と同じ縦の形。既にあるリストは元の形を保つ。
- 依頼の続き(要約): 今あるタグの中から、打って検索して候補を出す。
- 仮定: CE-8 の読むだけの形(複数行にまたがるフローのリスト、アンカー、入れ子の要素など)は、これまでどおり読むだけ。

## 設計

設計の本文は docs/design.md の「リストの値の付け外し(CE-16〜CE-19)」。要点: 核は `NewValue::List` を足し、writeback が元の書き方(フロー・ブロック)と字下げを保って要素を書き、キーが無い・空のときは Obsidian と同じ縦の形で書く。frontmatter の BlockList に、書ける形のときだけ要素の行の範囲を持たせる。Source に要素と件数の候補(list_candidates)を足し、リストの lock を外す(CE-8 の形は読むだけのまま)。画面は新しいモード「リストの選択」(検索・候補・付け外し・新規・一括の一部の印)。

選ばなかった案: リストを丸ごと1行の形で書き直す(元の書き方が変わる)、YAML の編集の部品を入れる(依存と書式の揺れ。設計の「最初に決めること」)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | リストの書き戻しと候補の核 | CE-16, CE-18, CE-19 | フロー・ブロック・キーなし・Null・空にする の書き方、書けない形は NotEditable、候補と件数、リストの lock が外れる、WB-17 の比べ方 | src/writeback.rs, src/frontmatter.rs, src/source.rs, src/source/markdown.rs, src/changes.rs, src/test_writeback_unit.rs, src/source/test_markdown_unit.rs, src/test_changes_unit.rs, src/ui, tests/test_list_write.rs, tests/test_source.rs, tests/test_review_fixes.rs, tests/test_examples.rs, tests/test_frontmatter.rs, tests/test_changes.rs, tests/golden | test_CE_16, test_CE_18, test_CE_19 | 済 |
| 2 | リストの選択の画面と一括 | CE-16, CE-17, CE-19 | 検索で絞る・付け外し・新規・確定と取り消し・一括の一部の印・保存のゴールデン | src/ui, tests/golden | test_CE_16, test_CE_17, test_CE_19 | 済 |

順: 1 → 2。1 の受け入れの試験(tests/test_list_write.rs)は実装を見ていない役が先に書き、[CE-6] を指していた試験(リストは読むだけと確かめる試験)も、同じ役が新しい要件の振る舞いに書き直す。2 は作ってからゴールデンで固める。

## 実装の気づき

- タスク1: 旧 CE-6(リストは読むだけ)を前提にした試験5つ(tests/test_frontmatter.rs の ce_8_block_list_has_no_span、tests/test_changes.rs の test_ce_10_set_on_read_only_cell_is_skip、src/ui/test_screen.rs の test_sr_18_read_only_cell_reason・test_ce_11_tab_skips_read_only・test_sr_1_five_bands のゴールデンの `#[本, 秋]`)が書き直されておらず落ちる。核の実装は受け入れの試験を全部通す。要素が文字列でないリスト(`[2024]` など)は型を保てないので読むだけにした。
- タスク2: モード ListPick(`list_select`)を src/ui/listpick.rs に足した。開くのは「値がリスト、か列の型がリストで値が空」のセルだけで、空でないスカラー(`tags: foo`)は従来の1行の入力のまま。キーは Enter=確定(新規を選んでいれば足す)・Tab=確定・Space=付け外し・Esc・Ctrl+R・↑↓/Ctrl+P/Ctrl+N・PageUp/Down・BS で、設計の「Enter(か Space)で開く」の Space は表では行の印(NV-5)なので Enter だけにした。新規の大文字小文字は区別して比べる(`meeting` と `Meeting` は別の要素)。保管庫に無い要素(ためた値だけにある)は件数なしで候補に出す。listpick.rs は 631 行で 600 行をやや超え、ヘルプの行が増えて sr_5 のゴールデンを更新した。
- タスク2の直し(照合・レビュー): 上の行の決まりを改めた。リストのセル(列の型がリストか値がリスト)はどの道(Tab の移動・一括で今の行が開けない)でも1行の入力を開かず、Tab の移動は書けるリストならリストの選択、書けなければ飛ばす。核の Changes::set もリストのセルへのリストでない値を Skip にした。キーは検索が空なら Space=付け外し・Enter=確定、空でなければ Space=空白・Enter=選んだ候補の付け外し(新規は足す)で検索を空に戻す、Tab=いつでも確定。tags の列だけ新規の比べ方を大文字小文字を区別しない形にした。確定の前に選んだセルが見えているかを確かめ直す。listpick.rs は 697 行で 600 行を超えている(分けるなら描画を listpick_view.rs へ)。
- 2026-10-02(main): 検索中の Enter は、再照合の指摘で「付け外し」から「付けるだけ(既に付いていれば何もしない、新規は足す)」に変えた。外すのは検索が空のときの Space だけ。tags の大文字小文字違いは、この行が持つ綴り → 同じ綴り → 件数の多いもの の順に選び、行が既に持つものは足さない。listpick.rs は約 720 行で 600 行の目安を超える(描画を分けるのが次の候補)。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02、3回)。1回目(90827de)は CE-16・CE-17・CE-18・WB-17 が checked、CE-19 が gap(画面で書けない形の理由を確かめる試験が無い)で、レビューの指摘7件と合わせて 774bcb2 で直した。2回目は CE-19 が gap(検索中の Enter が付いている要素を外す、tags の大文字小文字違いで要素が重なる)で、6f2cd69 で直し、3回目で3つとも checked。「前」は、実装の前のコミットではコンパイルで落ちるだけの要件は変異で、直しの前のコミットでは今の試験を入れて確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-16 | checked | コード: src/ui/listpick.rs:231 / テスト: src/ui/test_listpick.rs::test_ce_16_tab_never_opens_line_input_on_list / 今: 通った(cargo test) / 前: 落ちた(90827de では Tab でリストの列に1行の入力が開いて落ちた) |
| CE-17 | checked | コード: src/ui/listpick.rs:413 / テスト: src/ui/test_listpick.rs::test_ce_17_bulk_enter_in_search_only_attaches / 今: 通った(cargo test) / 前: 落ちた(774bcb2 では検索中の Enter で全部の行が持つ要素が外れて落ちた) |
| CE-18 | checked | コード: src/writeback.rs:147 / テスト: tests/test_list_write.rs::test_ce_18_block_list_indent_2_add_remove_reorder / 今: 通った(cargo test) / 前: 落ちた(変異: ブロックの新しい行の字下げを捨てる → 6本落ちた) |
| CE-19 | checked | コード: src/ui/listpick.rs:101 / テスト: src/ui/test_listpick.rs::test_ce_19_tags_case_prefers_held_spelling / 今: 通った(cargo test) / 前: 落ちた(774bcb2 では行が持つ meeting ではなく Meeting を選んで重なり落ちた) |
| WB-17 | checked | コード: src/changes.rs:544 / テスト: tests/test_list_write.rs::test_wb_17_list_set_back_to_original_is_not_pending / 今: 通った(cargo test) / 前: 落ちた(変異: List どうしを常に違うとする → 今と同じ並びをためて落ちた) |

既存のテストの削除・skip・弱体化: なし(削除した CE-6 を指していた試験と、リストを読むだけと前提にしていた5本は、確かめる中身を保って材料を読むだけの形に移し、書けることの確かめを足した。test_sr_18 の `#[a, b]` の完全一致は「先頭が `#`」に緩めたが、書けるリストに `#` が無いことの確かめを足した。test_ce_19_hash_and_no_duplicates は一度 Tab に変えたのを Enter に戻した)

確かめた: 5 / 5
