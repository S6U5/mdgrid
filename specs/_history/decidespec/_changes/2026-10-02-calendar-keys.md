---
type: Change
id: 01M3XXMTX5XZVTYKZEFVVXY345
title: カレンダーで月と年を飛ばすキー(calendar-keys)
status: done
size: full
created: 2026-10-02
updated: 2026-10-02
---

# カレンダーで月と年を飛ばすキー(calendar-keys)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [calendar-month-keys](../_decisions/2026-10-02-calendar-month-keys.md)(人)、[calendar-year-keys](../_decisions/2026-10-02-calendar-year-keys.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

依頼の要約(会話 2026-10-02): 日付の入力で月を飛ばせた方がよい。今は矢印でしか動かせない。

終わりの条件: カレンダーで Shift+←→ で1か月、Shift+↑↓ で1年動き、下の縁にそのキーが出る。PageUp・PageDown もこれまでどおり効く。関係する要件: CE-20・CE-21・CE-23・CE-24・SR-4。

## 不明点と仮定

- 仮定: PageUp・PageDown は既にあるが、Mac のノートでは Fn+↑↓ で、縁にも出ていないので気づけなかった。矢印に Shift を足す形なら、どのキーボードでも押せる。外れたら: 別のキーを割り当て直しで選べる(SR-4)。
- 仮定: カレンダーが出ていないとき、Shift+←→ は ←→ と同じ(文字のカーソル)、Shift+↑↓ は ↑↓ と同じ(候補)に読み替える。ほかのモード(表・設定の画面・検索の欄など)でも、Shift+←→ は ←→ と同じに動く(実装とレビューで決めた)。

## 設計

キーの表(src/ui/keymap.rs)の編集のモードに、Shift+Left・Shift+Right(前の月・次の月)と Shift+Up・Shift+Down(前の年・次の年)の動作を足す(新しい Action)。カレンダー(src/ui/calendar.rs)は、カレンダーが出ているときだけ、月は今の types::add_months(±1)、年は add_months(±12)(無い日は月末)で選んでいる日を動かし、入力ボックスの文字も合わせる(←→ と同じ流れ)。下の縁を2行にし(格子の幅は変えない)、1行目に月と年のキー、2行目に今日・空のキーを、キーの表から引いて出す(実装で決めた。1行では収まらないため)。ヘルプにも出る。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | カレンダーの月と年のキーと縁の表示 | CE-23, CE-24 | Shift+←→ で月、Shift+↑↓ で年、月末の補い、縁に出る、カレンダーが無いときは効かない、ヘルプに出る | src/ui/calendar.rs, src/ui/keymap.rs, src/ui/help.rs, src/ui/app.rs, src/ui/test_calendar.rs, src/ui/test_startup.rs, src/ui/test_settings_screen.rs, src/ui/test_shift_arrows.rs, src/ui/mod.rs, tests/golden | test_CE_23, test_CE_24 | 済 |

受け入れの試験は実装を見ていない役が先に書く。

## 実装の気づき

- タスク1(実装役、2026-10-02): test_ce_23_shift_left_right_moves_month は Shift+→ で 2026-11-30 にした後に Shift+← 2回で 2026-08-30 を期待するが、1か月ずつなら 2026-09-30。試験の数え違いで、実装役は止めて返した → main が試験を Shift+← 3回に直した(仕様の例は 10-30 から2回で 08-30 で正しい。起点を明記する字句の直しをした)。下の縁は2行(1行目に月・年、2行目に今日・空)にし、幅 30 は保った。
- レビューの直し(実装役、2026-10-02): 縁の行は数え方に合わせて増やす(ambiguous_wide では3行、高さは `height`)。Shift 側の表示名は ←→・↑↓ の形(「左・前の月」「前の候補・前の年」)にして PageUp の「日付: 前の月」と分け、矢印の見せ方は keymap::display に寄せた(表の Shift+↑↓ もヘルプで `Shift+↑` と出る)。

## 照合

書込なしのフレッシュ文脈の検証役で照合した(2026-10-02)。レビューの差し戻しの直しの再レビューも兼ねた(新しい不具合は無し)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-23 | checked | コード: src/ui/calendar.rs:456 / テスト: src/ui/test_calendar.rs::test_ce_23_shift_left_right_moves_month / 今: 通った(cargo test ce_23) / 前: 落ちた(9ede70d では Shift+→ が1日送りで 11-30 が 10-31。縁に月のキーが無い。カレンダーが無いときの守りの試験は calendar_fallback を消す変異で落ちた) |
| CE-24 | checked | コード: src/ui/calendar.rs:458 / テスト: src/ui/test_calendar.rs::test_ce_24_shift_up_down_moves_year / 今: 通った(cargo test ce_24) / 前: 落ちた(9ede70d では Shift+↓ で年が変わらない。縁に年のキーが無い) |

既存のテストの削除・skip・弱体化: なし(受け入れの試験の Shift+← 2回→3回は数え違いの直し、test_sr_13 の例の入れ替え、test_ce_21 の title+10、ゴールデンの縁とヘルプの行数は、どれも理由と合う)

残した気づき: test_shift_arrows の検索の欄・パレット・リストの選択の場面は、そこに ←→ の割り当てが無いので読み替えが無くても通る(守りとしては弱い)。前置きのあとの Shift+矢印は前置きを捨てて引き直す(既定の表には影響なし)。

確かめた: 2 / 2
