---
type: Change
id: 01M47W9ER251281JW2JDMG7K4G
title: このセッションのコードのレビューの直し(型の引き方の速さ・隠しフォルダ・カレンダー・--apply・割合・書式・--sort)(review6-fixes)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# このセッションのコードのレビューの直し(review6-fixes)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: どれも今の要件(CE-2・BV-23・CE-21・CLI-17・NV-9・BV-6・CLI-16)のとおりに動かす直し | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

フレッシュ文脈の書込なしのレビュー役が、62de54d..HEAD の src の差分(約 2.4k 行)を読んで見つけたもの。書き込みの安全の穴は無かった。

1. 高: types-case の `declared_of` が、同じ名前の無い列でセルごとに types.json の全部をなめる(5,000 ノート・800 項目の types.json で --print が 0.32 秒 → 6.5 秒)。大文字小文字だけ違う宣言の勝ちが HashMap の順で決まる。
2. 中: BV-23 の戻り。`.obsidian` のある保管庫の中の隠しフォルダを直接渡すと、探すのは保管庫の根からなので、そのフォルダを飛ばして0行。
3. 中: 空白の区切りの日時(datetime-offsets で読めるようにした)のセルでカレンダーで日を選ぶと、`2026-10-07 10:00` を書こうとして書き戻しが断る。
4. 低〜中: --apply で同じ列に当たる2つの見出し(`status` と `note.status`)を黙って後の方にする。
5. 低: 頻度表の 1% のすぐ下が `(0.10%)`。
6. 低: `.format()` が年 0 と 9999 の端の週で、週の記号を使わなくても null。
7. 低: --apply の JSON の `""` をリストの列に当てると `[""]`、i64 を超える整数が丸めて書かれる。
8. 低: --sort の列の名前に `:` があると(`time:start`)向きの誤りとして止まる。

## 不明点と仮定

- 仮定(1): 大文字小文字を外した名前の表を開くときに1回作る。同じ名前の無い列で、大文字小文字だけ違う宣言の型が食い違えば、根の食い違い(BV-12)と同じ扱い(テキスト・読むだけ)。
- 仮定(2): 飛ばさないのは、渡したフォルダとその上(根から渡したフォルダまでの道)のフォルダ。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 8件の直し | CE-2, BV-23, CE-21, CLI-17, NV-9, BV-6, CLI-16 | 上の要求のとおり | src/source/markdown.rs, src/vault.rs, src/ui/calendar.rs, src/apply.rs, src/ui/freq.rs, src/expr.rs, src/main.rs, src/i18n.rs, src/ui/mod.rs, src/ui/test_review6.rs, src/test_review6_unit.rs, src/source/test_review6_unit.rs, tests/test_review6.rs | tests/test_review6.rs・src/test_review6_unit.rs・src/source/test_review6_unit.rs・src/ui/test_review6.rs の各試験 | 済 |

## 実装の気づき

- 1: 大文字小文字を外した名前の表(declared_lower)を開くときに1回作り、同じ名前の無い列は O(1) で引く。5,000 ノート・800 項目の types.json の --print --sort が 6.5 秒(レビュー役の計測)→ 0.77 秒(release、ノートを探す時間を含む)。
- 2: Vault に keep(渡したフォルダの実体のパス)を持たせ、隠しフォルダでも keep のどれかがその下にあれば探す。poll の探し直しも同じ。
- 7: --apply でリストの列を空にする値は Null でなく空のリストにした(書き戻しは空のリストを \`key:\` と書き、Null はフローのリストに書けない)。CSV の空の欄も同じ道(前は空の欄でリストを空にしようとすると書き戻しが断っていた)。キーの無いノートに空のリストは足さない。
- 8: --sort の後ろが asc・desc でなければ、全部を列の名前として読む(\`due:sideways\` は列の綴りの確かめで終了コード 2 のまま。錠のある tests/test_print_filter_sort.rs も通る)。

## 照合

フレッシュ文脈の書込なしのレビュー役(見つけた役)と、自分の照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-2 | checked | コード: src/source/markdown.rs の lower_declared・declared_of / テスト: src/source/test_review6_unit.rs::test_ce_2_types_case_variants_conflict / 今: 通った(全体 1305 passed) / 前: 落ちた |
| BV-23 | checked | コード: src/vault.rs の open_keeping・visit / テスト: tests/test_review6.rs::test_bv_23_hidden_folder_inside_vault / 今: 通った / 前: 落ちた(0行) |
| CE-21 | checked | コード: src/ui/calendar.rs の time_of / テスト: src/ui/test_review6.rs::test_ce_21_space_datetime_day_pick_writes_t / 今: 通った / 前: 落ちた |
| CLI-17 | checked | コード: src/apply.rs の header_ids・to_value・plan / テスト: tests/test_review6.rs::test_cli_17_two_headers_same_column・test_cli_17_json_empty_list_and_big_int / 今: 通った / 前: 落ちた |
| NV-9 | checked | コード: src/ui/freq.rs の pct_text / テスト: src/ui/test_review6.rs::test_nv_9_percent_just_under_one / 今: 通った / 前: 落ちた(0.10%) |
| BV-6 | checked | コード: src/expr.rs の format_when / テスト: src/test_review6_unit.rs::test_bv_6_format_year_edges / 今: 通った / 前: 落ちた(null) |
| CLI-16 | checked | コード: src/main.rs の print_view の --sort の読み取り / テスト: tests/test_review6.rs::test_cli_16_sort_column_with_colon / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし

確かめた: 7 / 7
