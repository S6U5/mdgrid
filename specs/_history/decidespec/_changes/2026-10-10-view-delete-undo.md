---
type: Change
id: 01M4JD98W3XMEATWRV9V2QSGZ1
title: ビューの削除を確かめ、直後の u で戻す(BV-18)
status: done
size: full
created: 2026-10-10
updated: 2026-10-10
---

# ビューの削除を確かめ、直後の u で戻す(BV-18)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-10-view-delete-undo.md(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

ビューの削除(設定の画面の下のボタンと、ビューの区画の選び手)を、名前を示して確かめ(`y`)、消した直後の表での `u` で元に戻す(BV-18)。

## 不明点と仮定

- 仮定: 戻せるのは最後に消した1つで、消したあとにセルの変更をためたら、`u` はまずセルの変更を戻す(ためた変更の数が消したときと同じときだけビューを戻す)。

## 設計

- 確かめは設定の画面の値の入力(`TextKind::DeleteView`)で `y` を打つ。ほかの打ち込みと Esc はやめる。
- 消す前に、定義・並びの位置・タブの好み・既定のビューの名前・見た目の状態を App.view_trash に取っておく。表の `u` で、ためた変更の数が同じなら views.toml に戻す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 確かめと取り消し | BV-18 | y で消え、Esc ではそのまま、u で戻る | src/ui/native_views.rs, src/ui/view_tabs.rs, src/ui/settings*.rs, src/ui/app.rs, src/i18n.rs, src/ui/test_native_state.rs, src/ui/test_native_views.rs, src/ui/test_view_tabs.rs | test_bv_18_delete_*(src/ui/test_view_delete_undo.rs) | 済 |

## 実装の気づき

- 削除を通る錠のある試験に、確かめに y を打つ手順を足し、specs/_decisions/2026-10-10-view-delete-undo-locks.md(人)で錠を掛け直した。test_native_views.rs は共通の手伝いの関数を直したので、同じファイルの全部の試験の錠を掛け直した。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| BV-18 | checked | テスト: src/ui/test_view_delete_undo.rs::test_bv_18_delete_confirm_and_undo・test_bv_18_undo_restores_tab_prefs_and_default・test_bv_18_cell_change_after_delete_undone_first、src/ui/test_native_views.rs::test_bv_18_delete_removes_tab |

既存のテストの削除・skip・弱体化: なし(削除を通る試験に確かめの手順を足した)

確かめた: 1 / 1
