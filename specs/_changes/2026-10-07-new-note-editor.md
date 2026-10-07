---
type: Change
id: 01M4B46PK9VB976JQC7X4AKB6X
title: 新しいノートをフォームかエディタかで作れるようにする(new-note-editor)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 新しいノートをフォームかエディタかで作れるようにする(new-note-editor)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-new-note-editor.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CE-33 のとおり。

## 不明点と仮定

- 仮定: editor では、必須の欄(CE-27)は確かめない(窓に欄が無いので、エディタで入れる)。

## 設計

- NewNote に mode(`form`・`editor`。知らない値は警告にして form)。
- start_new_note: editor なら欄を空にする(名前だけ。名前の Enter で作る)。
- Flow に open_editor。editor の mode か Ctrl+E(新しい動作 `create_note_edit`)で立て、作ったあと editor_request に作った行を入れる(`e` と同じ道)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | mode とエディタで開く作り方 | CE-33 | 採択した確かめ方 | src/newnote.rs, src/config.rs, src/config_items.rs, src/i18n.rs, src/ui/new_note.rs, src/ui/keymap.rs, src/ui/mod.rs, src/ui/test_note_editor.rs, tests/golden/, docs/config.md, docs/config.ja.md, docs/keys.md, docs/keys.ja.md, specs/test-locks.json | test_ce_33_editor_mode_creates_and_opens(src/ui/test_note_editor.rs) | 済 |

## 実装の気づき

- 下の帯が狭いと「Esc やめる」が押し出されるので、Ctrl+E は最後に短く(「エディタで」)出した。
- ヘルプのゴールデン(sr_5)は行数が増えた分だけ作り直した。

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-33 | checked | コード: src/ui/new_note.rs の note_create_and_edit・finish_note、src/newnote.rs の NewNote::editor、src/config.rs の mode / テスト: src/ui/test_note_editor.rs::test_ce_33_editor_mode_creates_and_opens、src/ui/test_note_editor.rs::test_ce_33_form_ctrl_e_creates_and_opens、src/ui/test_note_editor.rs::test_ce_33_bad_mode_warns_and_uses_form / 今: 通った(全体 1353 passed) / 前: 落ちた(mode と Ctrl+E が無かった) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
