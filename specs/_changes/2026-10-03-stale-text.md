---
type: Change
id: 01M3ZWVGZHMJW5N3CF9PF64K0P
title: 仕様の古い記述を直す(stale-text)
status: done
size: tiny
created: 2026-10-03
updated: 2026-10-03
---

# 仕様の古い記述を直す(stale-text)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [stale-text](../_decisions/2026-10-03-stale-text.md)(AI)。守られる要件の分は [list-lines](../_decisions/2026-10-03-list-lines.md)(人の承認待ち) |
| 設計 | 対象外: 仕様の文だけの変更 | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 3632f5e・acdc643(仕様の文の直し。コードは変えない) |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の F-9。あとの決定で古くなった仕様の記述を、今の決定と実装に合わせる。読むだけの調べ役が挙げた食い違いのうち、守られる要件に触らないものをこの変更で直し、守られる要件(WB-1・WB-3・CE-10)に触るものは人の承認待ちの提案に分けた。人が決める論点(NV-9 と NV-19 の重なり、CLI-7 とビューの設定)は変えずに気づきに残す。

終わりの条件: 直した7要件の文が、決定の記録と実装(src/)と食い違わない。

## 不明点と仮定

- 仮定: 実装に合わせて文を直すのは、決定の記録に根拠があるものだけ(実装だけにある振る舞いを仕様に上げない)。

## 設計

対象外: 仕様の文だけの変更

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 古い記述を直す | SR-1, SR-16, CLI-2, CLI-3, CE-12, WB-16, WB-2 | 直した文が決定と実装と食い違わない | specs/*/spec.md | 読んで判定 | 済 |

## 実装の気づき

- CLI-2 に並べた `-s` / `--session`・`--sessions`(CLI-6・CLI-9)は版1の要件だが未実装(F-4)。

## 照合

書込なしのフレッシュ文脈の照合役で照合した(2026-10-03)。仕様の文だけの変更なので、どれも読んで判定(決定の記録と src/ と golden に照らす)。1回目で SR-1・WB-2 が gap(帯と「+ 新規」の出る条件の書き落とし、保存の一時ファイルの除外の書き落とし)→ [stale-text-fix](../_decisions/2026-10-03-stale-text-fix.md) で直し、別の照合役で再照合した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-1 | checked | 読んで判定: specs/screen/spec.md:22 の今の文を、view-settings.md:29(NV-16)・new-note.md:28(CE-25)と、src/ui/view.rs:503-516 の描く順・bands.rs:224-226 の chip_rows・new_note.rs:547-554 の button_shown・tests/golden/sr_1.txt に照らした(再照合) |
| SR-16 | checked | 読んで判定: 13のモードが src/ui/keymap.rs:10-52 の Mode の label と1対1 |
| CLI-2 | gap | 読んで判定: 決定の記録(sessions.md・sessions-details.md)とは合うが、src/main.rs は `-s`・`--session`・`--sessions` を受け付けない。欠けているもの: 実装。行き先: F-4(sessions。人が割り当てる)で実装する(仕様は CLI-6・CLI-9 で既に求めている旗を並べただけ) |
| CLI-3 | checked | 読んで判定: 旧の5項目が src/config_items.rs:26-160 の ITEMS にあり、ほかは「など(CLI-12 の文書)」で受ける |
| CE-12 | checked | 読んで判定: editor-setting.md(SR-8)と src/config.rs:139-142 の resolve_editor、src/ui/input.rs:223・379-382 |
| WB-16 | checked | 読んで判定: src/ui/external.rs:512・src/vault.rs:522・src/changes.rs:299-349(置き換えだけで振る舞いは不変) |
| WB-2 | checked | 読んで判定: specs/write-back/spec.md:34 の今の文と、src の書く所の grep(writeback.rs:838-857・789 の一時ファイルと rename=WB-8、newnote.rs:476-491=CE-25、native_io.rs:338-357=BV-19、config.rs:404-451=SR-12、views.rs:407-419=BV-20)(再照合) |

既存のテストの削除・skip・弱体化: なし(3632f5e・acdc643 は specs/ だけを変えた)

残した気づき: NV-9(頻度表、次の段)と NV-19(値の一覧と件数、版1)の中身が重なる。CLI-7(セッションが持つもの)がビューの設定(NV-13〜NV-22)と mdgrid のビュー(BV-17)を書いていない。どちらも人が決める(F-4 のとき)。

確かめた: 6 / 7
