---
type: Change
id: 01M46NJSM0NN7AH7RKFAET8BWY
title: .base のビューで作るノートを file.inFolder のフォルダに作る(new-note-infolder)
status: done
size: bugfix
created: 2026-10-06
updated: 2026-10-06
---

# .base のビューで作るノートを file.inFolder のフォルダに作る(new-note-infolder)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 不具合の直し(CE-25 の「作った行がそのビューに残るようにするべき」に反していた) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-4)が見つけた。英語の見本の Tasks.base(絞り込み `file.inFolder("Tasks")`)の Open のビューで `a` から「My new task」を作ると、`.base` のフォルダ(保管庫の根)に `My new task.md` ができ、「今の絞り込みでは見えない」と出てビューから消えた。`done == false` は入れたが、フォルダの条件を守っていない。

終わりの条件: 絞り込みに `file.inFolder("Tasks")` がある `.base` のビューで作ると `Tasks/<名前>.md` にでき、その行が選ばれる。設定の `new_note.folder` が Tasks の下(`Tasks/inbox`)ならそのまま。フォルダの条件が無ければ今と同じ。関係する要件: CE-25。

## 不明点と仮定

- 仮定: `file.inFolder` のパスは Obsidian と同じく保管庫の根(`.obsidian` のある最寄りの上のフォルダ)から。作る場所の根(`.base` のフォルダ)の外を指すとき(`..` が要る)は、今の決まり(外には作らない)のまま変えない。
- 仮定: `&&` でつないだ `file.inFolder` が2つ以上なら、一番深いフォルダ。`||` の中のものは使わない(1つに決まらない)。
- 仮定: 設定の `new_note.folder` がそのフォルダの中なら設定を使い、外ならフォルダの条件を使う(作った行がビューに残ることを先にする)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | file.inFolder のフォルダに作る | CE-25 | 上の終わりの条件のとおり | src/expr.rs, src/newnote.rs, src/ui/new_note.rs, src/ui/test_new_note_infolder.rs, src/ui/mod.rs | test_ce_25_infolder_*(src/ui/test_new_note_infolder.rs) | 済 |

## 実装の気づき

- 作る場所の根(`.base` のフォルダ)と保管庫の根が違うとき(`.base` が下のフォルダにある)は、`file.inFolder` のパスを保管庫の根から解いて、作る場所の根の中にあるときだけ使う。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。小さな不具合の直しのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-25 | checked | コード: src/expr.rs の Fixed::Folder、src/newnote.rs の view_folder、src/ui/new_note.rs の start_new_note と folder_under_root / テスト: src/ui/test_new_note_infolder.rs::test_ce_25_infolder_creates_in_view_folder・_keeps_configured_subfolder・_overrides_folder_outside_view / 今: 通った(3 passed、全体 1084 passed) / 前: 落ちた(実装の前に試験を置いて回し、creates_in_view_folder と overrides_folder_outside_view の2本が「Tasks/ に作る」で落ちた。keeps_configured_subfolder は設定のフォルダが元から Tasks の中なので前も通る) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
