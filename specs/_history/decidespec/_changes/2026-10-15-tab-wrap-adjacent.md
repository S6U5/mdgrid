---
type: Change
id: 01M4ZERBQ02G30SZ3ZBPYJQQBH
title: 折り返したタブの行を、帯の上の余白の下(真ん中の行のすぐ上)に置く(tab-wrap-adjacent)
status: done
size: full
created: 2026-10-15
updated: 2026-10-15
---

# 折り返したタブの行を、帯の上の余白の下(真ん中の行のすぐ上)に置く(tab-wrap-adjacent)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [提案](../_decisions/2026-10-15-tab-wrap-adjacent.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「実装: [NV-31] 折り返したタブの行を…」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、タブを何行かに分けてから使い勝手が悪くなったようなので直してほしい、と言った(中継 2026-10-15)。本物の台帳(ops-base/tasks。タブ11個)を幅 100 で開くと、折り返して足した行(NV-31)がヘッダーのすぐ下に詰まり、残りのタブのある真ん中の行との間に帯の上の余白の空き行が挟まって、タブの並びが2つに割れて見える。足した行を真ん中の行のすぐ上(上の余白の下)に置き、タブを1つの塊に見せたら終わり。関係する要件: NV-31(NV-23 は変えない)。

## 不明点と仮定

- 仮定: 「フィルターを複数乗」は、期限のタブを足してタブが2行に折り返すようになった件(まとめ役の既定)。外れたら: 人の言う「フィルター」が絞り込みの窓(NV-29)の条件のことなら、別の変更にする。
- 仮定: `buttons = "box"` は今のまま(足した行は余白のさらに上)。囲みの上の線が真ん中の行に接する余白の行にあるため。

## 設計

- `src/ui/bands.rs` の `view_bar_lines`: plain(囲まない)では、上の余白の行(全部の行と、端数の `▀` の行)の後に足した行、真ん中の行、下の余白の順に並べる。端数で帯の行に地の色を敷くときは、足した行にも `lay_bg` で同じ地の色を敷く。box と余白 0 は今のまま。
- 足した行の画面の行を返す `wrap_y`(plain では `bar_y + bar_padding`、box では `bar_y`)を足し、`tab_at` のクリックの判定をそれに合わせる。`bar_mid`(= `bar_y + 足した行 + 余白`)と帯の高さは変わらない。
- 却下: タブを全部専用の行に移す案(提案の「却下した案」)。
- 検証: `cargo test --lib test_tab_wrap`、新しい試験 `src/ui/test_tab_wrap_adjacent.rs`、全部の試験、本物の台帳を幅 100 で撮る。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | plain で足した行を余白の下に置き、クリックの行を合わせる | NV-31 | 幅 60・余白 1 で、足した行が検索の欄の行のすぐ上、ヘッダーとの間に空き行。足した行のタブのクリックでそのビュー | src/ui/bands.rs src/ui/test_tab_wrap_adjacent.rs src/ui/mod.rs | test_nv_31_plain_wrapped_rows_touch_bar・test_nv_31_plain_wrapped_click_after_padding | 済 |
| 2 | box は今のまま | NV-31 | box では足した行が囲みの上の線より上 | src/ui/test_tab_wrap_adjacent.rs | test_nv_31_box_wrapped_above_box | 済 |

## 実装の気づき

なし

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-31 | checked | コード: src/ui/bands.rs:547(wrap_y)・src/ui/bands.rs:1014(足した行を余白の下に) / テスト: src/ui/test_tab_wrap_adjacent.rs::test_nv_31_plain_wrapped_rows_touch_bar / テスト: src/ui/test_tab_wrap_adjacent.rs::test_nv_31_plain_zero_padding_same_place / テスト: src/ui/test_tab_wrap_adjacent.rs::test_nv_31_plain_wrapped_click_after_padding / テスト: src/ui/test_tab_wrap_adjacent.rs::test_nv_31_box_wrapped_above_box / テスト: src/ui/test_tab_wrap.rs::test_nv_31_click_wrapped_tab_selects_view / 今: 本物の台帳(タブ11個)を幅 100 で撮り、足した行が検索の欄の行のすぐ上、ヘッダーとの間に空き行1つ / 前: 同じ台帳で、足した行がヘッダーのすぐ下にあり、検索の欄の行との間に空き行が挟まっていた |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
