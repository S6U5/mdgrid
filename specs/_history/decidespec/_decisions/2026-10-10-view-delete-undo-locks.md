---
type: Proposal
id: 01M4JDH0EQYJQFNK5VB82ZA7X8
title: ビューの削除の確かめに合わせて、削除を通る試験の錠を掛け直す(view-delete-undo-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 494386da2e398ed2
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [BV-3, BV-13, BV-17, BV-18, BV-19, BV-20, NV-13, NV-26, SR-12]
created: 2026-10-10
updated: 2026-10-10
---

# ビューの削除の確かめに合わせて、削除を通る試験の錠を掛け直す(view-delete-undo-locks)

## きっかけ

会話 2026-10-10。人が、ビューの削除に取り消しを付けると決めた([view-delete-undo](2026-10-10-view-delete-undo.md))。削除の前に名前を示して確かめる(`y`)ので、削除のボタンを押してすぐ消えることを前提にしていた錠のある試験が、確かめに答える手順を足す必要がある。

試験の確かめる中身(消えたビュー・残るビュー・状態・タブ)は変えていない。変えたのは、削除のあとに `y` を打って Enter を押す手順だけ。src/ui/test_native_views.rs は共通の手伝いの関数(confirm_if_asked)に足したので、同じファイルの試験の全部の錠を掛け直す。

## 差分

- 錠: BV-18(src/ui/test_native_state.rs::test_bv_18_two_apps_keep_each_others_views、src/ui/test_native_views.rs の BV-18 の試験。削除の確かめに y)
- 錠: BV-20(src/ui/test_native_state.rs::test_bv_20_rename_moves_and_delete_removes_state。削除の確かめに y。src/ui/test_native_views.rs の BV-20 の試験は共通の部分の変更)
- 錠: SR-12(test_bv_20_rename_moves_and_delete_removes_state が SR-12 も確かめる)
- 錠: NV-26(src/ui/test_view_tabs.rs::test_nv_26_rename_delete_follow_prefs。区画の削除の確かめに y)
- 錠: BV-17(src/ui/test_native_views.rs。共通の部分の変更だけ)
- 錠: BV-19(src/ui/test_native_views.rs。共通の部分の変更だけ)
- 錠: BV-3(同じ試験の関数が BV-3 も確かめる)
- 錠: BV-13(同じ試験の関数が BV-13 も確かめる)
- 錠: NV-13(同じ試験の関数が NV-13 も確かめる)

## 却下した案

| 案 | 理由 |
|---|---|
| 確かめの入力に y を最初から入れておく(Enter だけで消える) | 確かめが Enter の打ち間違いで通ってしまう |

## 承認の記録

2026-10-10 人の承認(要約: ビューの削除に取り消しを付ける。それに合わせた試験の手順の追加)
