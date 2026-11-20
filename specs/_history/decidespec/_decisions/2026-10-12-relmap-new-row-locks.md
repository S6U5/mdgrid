---
type: Proposal
id: 01M4QM99A4CZD4YN6R3YG7RTTY
title: 関係マップの「+ 新規」の行と窓の位置が変わる試験の錠を掛け直す(relmap-new-row-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: eeabf836ce67d999
decided-by: 人
trigger: AI の提案
trigger-link: 会話 2026-10-12
touches: [CE-25, REL-8, REL-12]
created: 2026-10-12
updated: 2026-10-12
---

# 関係マップの「+ 新規」の行と窓の位置が変わる試験の錠を掛け直す(relmap-new-row-locks)

## きっかけ

人の決定 2026-10-12-relmap-new-row(関係マップの「+ 新規」を表の画面のビューの帯と同じ行に置き、ヘッダーの下に帯の上の余白と同じ行の数を空ける)で、関係マップの窓が既定で1行下がり、「+ 新規」の行が変わった。既存の試験 5 本は、行の決め打ちと、行を探す目印だけを直した。確かめる中身は消しも緩めもしていない。関係マップの画面の型のメニュー(REL-8)とクリック(REL-12)の錠も持つので、決定の記録だけでは掛け直せない。

| ファイル | 試験 | 直し |
|---|---|---|
| `src/ui/test_header_menu.rs` | test_ce_25_new_same_place_in_relations | 関係マップの「+ 新規」の行を、ヘッダーのすぐ下から帯の真ん中の行に |
| `src/ui/test_header_menu.rs` | test_rel_8_menu_back_to_table | メニューの窓の「表」の項目を探す目印を、「関係マップ」の字が無い行から、枠の中が「表」だけの行に(窓が関係マップの窓の上の縁と同じ行に重なるようになったため) |
| `src/ui/test_relmap_click.rs` | test_rel_12_click_link_and_linked_row | 盤の窓の中の行に、帯の上の余白の行の数を足す |
| `src/ui/test_relmap_new_note.rs` | test_ce_25_relmap_new_note・test_ce_25_relmap_new_note_other_table(共有の関数) | 関係マップの「+ 新規」の行を、帯の真ん中の行に |

## 差分

- 錠: CE-25(test_header_menu::test_ce_25_new_same_place_in_relations、test_relmap_new_note::test_ce_25_relmap_new_note・test_ce_25_relmap_new_note_other_table。「+ 新規」の行が変わった)
- 錠: REL-8(test_header_menu::test_rel_8_menu_back_to_table。メニューの窓が関係マップの窓の上の縁と同じ行に重なるようになった)
- 錠: REL-12(test_relmap_click::test_rel_12_click_link_and_linked_row。関係マップの窓が帯の上の余白の分だけ下がった)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を変えずに、関係マップの窓をヘッダーのすぐ下に戻す | 人が決めた「+ 新規」を表の画面と同じ行に置く形に合わない |

## 承認の記録

2026-10-12 人の承認(要約: 関係マップの「+ 新規」の行と窓の位置で変わる試験5本の錠を掛け直す提案に「承認」と答えた)
