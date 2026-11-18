---
type: Change
id: 01M3ZW8S038SRBHBGR6112Y9WZ
title: 新しいノートの残り(new-note-rest)
status: done
size: full
created: 2026-10-03
updated: 2026-10-03
---

# 新しいノートの残り(new-note-rest)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [new-note-folders](../_decisions/2026-10-03-new-note-folders.md)(人、2026-10-03 採択)、[untyped-date](../_decisions/2026-10-03-untyped-date.md)(人、2026-10-03 採択) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の F-12(new-note の照合で残した気づき)を片付ける。このうち 2・3(仕様を変えないもの)は [new-note-gaps](2026-10-03-new-note-gaps.md) に分けた。この記録は 1・4(人の承認待ちの提案)を扱う。会話 2026-10-03 の目標と判断の束による。

1. どのノートにも無い列を新しいノートで聞いて日付を答えると、型が分からず囲んで書く。→ 日付の形なら囲まずに書く(人の選択)。
2. 画面の側で、ビューごとの決まり(mdgrid のビューの `new_note`)を使うことを確かめる試験が無い(無視する変異が落ちない)。
3. 聞く項目がテキストの列のとき、セルの編集と同じ候補のリスト(CE-3)が出ない。
4. フォルダを2つ以上開くと「+ 新規」が出ず、仕様に無い。→ 名前の前に作る場所を選ぶ欄を出す(人の選択)。

終わりの条件:
- どのノートにも無い列 deadline を聞いて `2026-10-05` → `deadline: 2026-10-05`。`2026-10-05 ごろ` → 囲む。
- mdgrid のビューに `new_note` があるとき、設定の `[new_note]` でなくそちらの ask・set が使われる(画面の試験)。
- status の値が todo・done だけのとき、聞く項目 status でリストが出る。
- フォルダ A・B を開いて「+ 新規」→ 一覧で B を選び「メモ」→ `B/メモ.md`。

関係する要件: CE-25・CE-26・CE-3・CE-2・WB-18・WB-7・BV-17。

## 不明点と仮定

- 作る場所と型の決まらない日付は、会話 2026-10-03 の判断の束で人が選んだ(提案の2件)。
- 仮定: 「型の決まらない列」は CE-2 の推定で値が1つも無い列(Kind が決まらない)。新しいノートの聞く列に限らず、表のセルの編集にも同じに効く。外れたら: 新しいノートの聞く列だけに絞る。
- 仮定: テキストの列の候補は、セルの編集と同じ条件(CE-3 の異なる値が既定 20 以下)でリストにし、自由入力にも切り替えられる。外れたら: 1行の入力のまま。
- 仮定: 作る場所の一覧は、開いた引数の順(`.base` はそのフォルダ)で、同じフォルダは1つにまとめる。フォルダが1つなら今どおり欄を出さない。

## 設計

- 型の決まらない列(WB-18): 開く側(Source)に「その列の型が決まっているか」(型の設定 CE-2 があるか、読み込んだどれかのノートに空でない値があるか)を返す口を足す(既定の実装つきの trait のメソッドにして、ほかの実装を壊さない。公開の構造体 ColumnKind の欄は足さない: 既存の試験が構造体の式で作るため)。値をためる所(セルの編集の確定と、新しいノートの聞く項目の確定・前もって入れる値)で、型の決まらない列に日付(`YYYY-MM-DD`)か日時の形だけの文字が来たら、日付の値として扱い、WB-18 のとおり囲まずに書く。それ以外の文字は今どおり文字列(WB-7)。
- 作る場所を選ぶ欄(CE-25): 開いたフォルダの一覧(起動の引数の順。`.base` はそのフォルダ。同じフォルダは1つ)を App が持つ。2つ以上なら「+ 新規」を出し、始めたら名前の欄の前に一覧から選ぶ欄を出す(既定は最初。Enter で決める、Esc で取りやめ)。選んだフォルダを作る場所の根にし、そのあとは今の流れ(名前・聞く項目)。1つなら今どおり選ぶ欄を出さない。部品は今あるリストの選択の見せ方を使う。
- 却下: ColumnKind に欄を足す(既存の試験を書き換えることになる)。
- 検証: 新しい試験(下のタスク)、`./ci.sh`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 型の決まらない列の日付を囲まずに書く | WB-18 | どのノートにも無い列に 2026-10-05 を書くと `key: 2026-10-05`、`2026-10-05 ごろ` は囲む | src/source.rs, src/source/markdown.rs, src/changes.rs, src/ui/input.rs, src/ui/new_note.rs, src/newnote.rs, src/types.rs, src/ui/test_untyped_date.rs, src/ui/mod.rs | test_wb_18_untyped_*(src/ui/test_untyped_date.rs) | 済 |
| 2 | フォルダが2つ以上のとき作る場所を選ぶ欄 | CE-25 | A・B を開いて「+ 新規」→ 一覧で B を選び「メモ」→ `B/メモ.md` | src/ui/new_note.rs, src/ui/app.rs, src/ui/startup.rs, src/source.rs, src/source/markdown.rs, src/ui/bands.rs, src/main.rs, src/ui/test_new_note_folders.rs, src/ui/mod.rs | test_ce_25_folders_*(src/ui/test_new_note_folders.rs) | 済 |
| 3 | 錠のある古い試験を新しい CE-25 に直し、CE-26 の順を試験にする | CE-25, CE-26 | 2つ開いて `a` → 作る場所を選ぶ欄、場所 → 名前 → 聞く項目の順 | src/ui/test_new_note_more.rs, src/ui/test_new_note_order.rs, src/ui/mod.rs | test_ce_25_two_folders_have_no_button(錠の掛け直し)・test_ce_26_place_then_name_then_asked_items | 済 |

順: 1 → 2(どちらも src/ui/new_note.rs を触るので順に)。

## 実装の気づき

- タスク1: Source に `typed(col)`(既定の実装は true)、`source::date_if_untyped`、`types::date_or_datetime` を足した。Changes::plan(WB-17 の判定の前)と新しいノートの答え・前もって入れる値で当てる。読み込みの途中(BV-16)は、まだ読んでいないノートにある値を知らないので「決まらない」と判定しうる(WB-18 の文「読み込んだどのノートにも」のとおり)。核の Changes::plan と、前もって入れる値の日付の単体の試験は無い(UI の試験でだけ確かめている)。
- タスク2: 範囲に src/source.rs と src/source/markdown.rs を足す。作る場所の一覧を state_target の改行つなぎから分けて作ると、改行を含むフォルダ名で壊れ、開いていないフォルダに作れてしまうので、Source に「開いたフォルダを引数の順で返す」口(既定の実装つきの trait のメソッド)を足し、そこから作るため。

## 照合

タスク2の実装は済み。錠のある src/ui/test_new_note_more.rs::test_ce_25_two_folders_have_no_button(前の振る舞いを確かめていた)は、2026-10-06 に決定 new-note-folders-order(人が判断を AI に委ねた承認)で新しい振る舞いに直し、`check.py --lock --relock` で錠を掛け直した(タスク3)。

タスク1・2とも書込なしのフレッシュ文脈の照合役で照合した(2026-10-03、コミット済みの clone で。実装のコミットは c020900・2080a12)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-18 | checked | コード: src/source.rs の date_if_untyped と Source::typed、src/source/markdown.rs の typed、src/types.rs の date_or_datetime、src/changes.rs の Changes::plan、src/ui/new_note.rs の answer と前もって入れる値 / テスト: src/ui/test_untyped_date.rs::test_wb_18_untyped_*(8本) / 今: 通った(8 passed) / 前: 落ちた(c020900^ で4本が「囲んで書かれる」で落ちた)。変異: plan で当てない → 2本、answer で当てない → 2本、typed を常に false → 3本が落ちた |
| CE-25 | checked | コード: src/ui/new_note.rs の places・start_new_note の選ぶ欄・place_commit・note_choosing、src/source.rs と src/source/markdown.rs の folders、src/ui/startup.rs / テスト: src/ui/test_new_note_folders.rs::test_ce_25_folders_*(6本) / 今: 通った(6 passed。古い錠の試験を除く全体も 0 failed) / 前: 落ちた(2080a12^ で4本が「+ 新規が無い」「一覧が出ない」で落ちた)。変異: 常に最初のフォルダ → 4本、名前の順に並べ替え → 1本、選んだフォルダを根にしない → 1本が落ちた。選ぶ欄で自由入力にならないことと、選んだ根でも `..` を断ることは読んで判定 |

| CE-26 | checked | コード: src/ui/new_note.rs(作る場所 → 名前 → 聞く項目の順) / テスト: src/ui/test_new_note_order.rs::test_ce_26_place_then_name_then_asked_items / 今: 通った(cargo test --bin mdgrid test_ce_26_place) / 前: 読んで判定(順はタスク2の実装で既に満たしていた。決定 new-note-folders-order は文に順を書き足しただけで、振る舞いは変えない。自分で照合(フレッシュ文脈でない)) |

既存のテストの削除・skip・弱体化: なし(タスク1の範囲)。タスク3で錠のある test_ce_25_two_folders_have_no_button を、人の委ねた決定で新しい振る舞いの確かめに替えた(弱めていない: 「+ 新規」が出ることと、`a` で作る場所を選ぶ欄が出ることを確かめる)

確かめた: 3 / 3

残した気づき: 前もって入れる値(set と絞り込み)への当て方は試験なし(コードを読んで確かめた)。
