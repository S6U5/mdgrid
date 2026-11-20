---
type: Change
id: 01M4B32P06XED74YQ2RB5Z5J88
title: 新しいノートをフォームで作り、雛形の変数と本文の雛形を足す(new-note-form)
status: done
size: full
created: 2026-10-07
updated: 2026-10-07
---

# 新しいノートをフォームで作り、雛形の変数と本文の雛形を足す(new-note-form)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-07-new-note-form.md・2026-10-07-note-templates.md(人の承認) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

採択した CE-26・CE-27・CE-32 のとおり。

## 不明点と仮定

- 仮定: 今の打ち方(名前を打って Enter で次の欄、最後の欄の Enter で作る、Shift+Tab で前、Esc でやめる)は保つ。ask の無い設定では欄が見えている列になるので、名前の Enter で作られていたところは、Ctrl+S で作る(試験はそう直す)。
- 仮定: 欄が画面に入らないときは、今の欄の周りだけを出す。
- 仮定: `{date:形}` の形は CE-22 の日付の形(`date_format` と同じ書き方)。`{weekday}` は今の言語の曜日の名前。`{now}` は秒なしの `YYYY-MM-DDTHH:MM`(地域の時刻)。

## 設計

- src/newnote.rs: NewNote に required・hidden・body。雛形の変数を埋める `expand(s, &Vars)`。`build_with`(本文の雛形と変数つき)。
- src/config.rs・views(serde の Raw)・config_items.rs・docs/config: 新しい項目。
- src/ui/new_note.rs: Flow に欄の並び(fields。ask か見えている列から hidden を除く)と、欄ごとの答え(置き換え)と、欄の理由。窓は表の上に重ね、今の欄の行に入力ボックスを置く(カレンダーと候補の窓はその行から出る)。Ctrl+S(Edit と ListPick の `create_note`)で、今の欄を確かめてから、名前と必須の欄を確かめて作る。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 設定と雛形の変数・本文の雛形 | CE-27, CE-32 | 採択した確かめ方 | src/newnote.rs, src/config.rs, src/config_items.rs, src/views.rs, src/i18n.rs, src/test_note_templates_unit.rs, src/lib.rs, docs/config.md, docs/config.ja.md | test_ce_32_template_variables(src/test_note_templates_unit.rs) | 済 |
| 2 | フォームの窓 | CE-26 | 採択した確かめ方 | src/ui/new_note.rs, src/ui/keymap.rs, src/ui/app.rs, src/ui/view.rs, src/ui/calendar.rs, src/ui/mod.rs, src/ui/test_language.rs, src/ui/test_note_form.rs, src/ui/test_new_note*.rs, tests/, tests/golden/, docs/keys*.md, docs/manual/, specs/test-locks.json, specs/_decisions/ | test_ce_26_form_shows_fields_and_creates(src/ui/test_note_form.rs) | 済 |

## 実装の気づき

- 窓は表の見出しの行から重ね、今の欄の行に入力ボックスを置いた(カレンダーと候補の窓はそこから出る)。入らない高さでは今の欄の周りだけを出す。
- 雛形を埋めた値が日付・日時の形なら、型の決まらない列(WB-18)と日付・日時の列では囲まずに書く(as_date)。
- 錠のある 40 の試験は、作るキー(Enter → Ctrl+S)と画面の確かめ(窓の今の欄)を直し、new-note-form と new-note-form-locks(人の承認)で掛け直した。
- 新しいキーの名前が「新しいノート」を含むと、読むだけでコマンドを隠す試験に当たるので「新規の窓で作る」にした。
- フォームとエディタを選ぶ案は、人の承認を受けて別の変更(new-note-editor)にする。

## 照合

自分で照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-26 | checked | コード: src/ui/new_note.rs の finish_note・note_create_now・overlay / テスト: src/ui/test_note_form.rs::test_ce_26_form_shows_fields_and_creates、src/ui/test_note_form.rs::test_ce_26_form_defaults_to_visible_columns_and_back、src/ui/test_note_form.rs::test_ce_26_form_name_problem_shown_on_field / 今: 通った(全体 1350 passed) / 前: 落ちた(窓が無かった) |
| CE-27 | checked | コード: src/newnote.rs の NewNote・from_toml、src/config.rs の read_new_note / テスト: src/ui/test_note_form.rs::test_ce_27_required_and_date_template、src/test_note_templates_unit.rs::test_ce_27_required_hidden_body_round_trip / 今: 通った / 前: 落ちた |
| CE-32 | checked | コード: src/newnote.rs の expand・build_with / テスト: src/test_note_templates_unit.rs::test_ce_32_template_variables、src/ui/test_note_form.rs::test_ce_32_hidden_created_and_body_template / 今: 通った / 前: 落ちた |

既存のテストの削除・skip・弱体化: なし(錠のある試験は、作るキーと画面の確かめを窓に合わせ、人の決定で掛け直した)

確かめた: 3 / 3
