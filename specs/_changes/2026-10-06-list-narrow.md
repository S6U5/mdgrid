---
type: Change
id: 01M476VPSEBYW0PVBGAKBA02DD
title: リストで打った文字で候補を絞り、操作の一覧に位置の印を出す(list-narrow)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# リストで打った文字で候補を絞り、操作の一覧に位置の印を出す(list-narrow)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-list-narrow.md |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の A-8・A-9。

- A-8: 候補のリストで打つと、リストが消える。打った文字で候補を絞って出し続け、↑↓ で選べるようにする(CE-3 の変更を採択した)。
- A-9: 操作の一覧(SR-24)が窓に入らずに流れるとき、今どこかの印が無い。リストの選択と同じく、窓の縁に `3/22` を出す。

終わりの条件: 採択した CE-3 の確かめ方のとおり。操作の一覧は、全部が入らないときだけ下の縁に「選んでいる項目の番号/項目の数」を出す。

## 不明点と仮定

- 仮定: 絞るのは部分一致(大文字小文字を問わない)。「なし」は絞った候補に出さない(打った文字で「なし」は選ばない)。候補が打った文字と同じ1つだけなら出さない(見ても何も足さない)。
- 仮定: 絞った候補では選びの印 `>` を出さない(錠のある test_ce_3_none_and_free_input が自由入力で `|>` が無いことを見ている。選んでいない形と合う)。

## 設計

- `List` に `filter: Option<String>` を足す。自由入力から ↑↓ でリストに戻るとき、打った文字に当たる候補があれば filter に打った文字を入れ、最初の候補を選ぶ(moved)。当たらなければ今までどおり全部のリスト。Ctrl+R と打ち直しで filter を消す。
- 見せる項目は `App::list_view`(リスト・見せる項目の添字・選べるか)で1か所で決め、位置・描画・クリックはそれを使う。自由入力の間は選べない形(クリックでは選んで確定)。
- 操作の一覧は `overlay` の下の縁に、`g.vis < ents.len()` のときだけ番号を入れる。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 打った文字で候補を絞る | CE-3 | 上の終わりの条件 | src/ui/list.rs, src/ui/input.rs, src/ui/entry.rs, src/i18n.rs, src/ui/new_note.rs, src/ui/mod.rs, src/ui/test_list_narrow.rs | test_ce_3_narrow_*(src/ui/test_list_narrow.rs) | 済 |
| 2 | 操作の一覧の位置の印 | SR-24 | 上の終わりの条件 | src/ui/menu.rs, src/ui/test_list_narrow.rs | test_sr_24_menu_position_*(src/ui/test_list_narrow.rs) | 済 |

## 実装の気づき

- 書込なしのレビュー役(フレッシュ文脈)が、添字の取り違え・落ちる所は無いとし、小さい3点を挙げた。どれも直した: 絞った候補を ↑↓ で選んだあとに打つと続きに足していた(CE-3 の「選び直したあとも」に合わせて置き換える)、↑ でも最初の候補を選んでいた(最後の候補から)、自由入力の案内が「↑↓ でリストに戻る」のままだった(絞った候補が出ているときは「↑↓ で当たる候補を選ぶ」)。
- A-5(狭い端末)は、錠のある 80×8 で開かない試験と境目の規則を合わせる要があり、この記録では扱わない(docs/todo.md に後回しの理由)。

## 照合

フレッシュ文脈の書込なしのレビュー役(実装のあと)と、自分の照合。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CE-3 | checked | コード: src/ui/list.rs の List::matches・shown、App::list_view・list_move・list_click、geometry・overlay、src/ui/entry.rs の案内 / テスト: src/ui/test_list_narrow.rs::test_ce_3_narrow_shows_matches_without_selection・test_ce_3_narrow_down_picks_match・test_ce_3_narrow_no_match_hides_and_revert_restores・test_ce_3_narrow_up_picks_last_and_typing_replaces、錠のある src/ui/test_input.rs::test_ce_3_none_and_free_input / 今: 通った(全体 1243 passed) / 前: 落ちた(実装の前に回し、CE-3 の3本が落ちた) |
| SR-24 | checked | コード: src/ui/menu.rs の overlay の下の縁 / テスト: src/ui/test_list_narrow.rs::test_sr_24_menu_position_shown_when_scrolled・test_sr_24_menu_position_hidden_when_all_fit / 今: 通った(全体 1243 passed) / 前: 落ちた(流れるときの試験が、実装の前に印が無く落ちた) |

既存のテストの削除・skip・弱体化: なし

確かめた: 2 / 2
