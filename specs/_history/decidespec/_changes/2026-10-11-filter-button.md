---
type: Change
id: 01M4MZACTT5N43YT7ACVW5Q999
title: 絞り込みを並べ替えと同じボタンと窓にし、上の欄の検索と呼び分ける
status: done
size: full
created: 2026-10-11
updated: 2026-10-11
---

# 絞り込みを並べ替えと同じボタンと窓にし、上の欄の検索と呼び分ける

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [filter-button](../_decisions/2026-10-11-filter-button.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「実装: [NV-29]…」のコミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が、絞り込みは並べ替えと同じく足せる形であってよく、検索はそれとは別だと言い、その形の提案を採択した(会話 2026-10-11)。終わりの条件: 検索の欄の行の右の端に「絞り込み」と「並べ替え」のボタンが並び、「絞り込み」の窓で条件を足す・外す・変えると、すぐ表に効いてビューに覚える(NV-29)。上の欄は画面でも「検索」、`/` は「探す」と呼ぶ(NV-1・NV-2・NV-23)。関係する要件: NV-29・NV-23・NV-24・NV-2・NV-1・NV-20。OUT-2・SR-18・CLI-7 は呼び名の置き換えだけで振る舞いは変わらない。

## 不明点と仮定

- 仮定: 窓の中の条件の足し方は、設定の画面(NV-19)と同じ段取り(列 → 種類 → 値の一覧か値の入力。比較は記号を選んでから値)にする。値の一覧は、残すか隠すかを先頭の行で切り替え、Space でチェックを付け外し、Enter で決める。外れたら: 段取りだけ直す。
- 仮定: 絞り込みの窓の既定のキーは `|`(表のモードで空いていて、検索の `\` の隣。動作の名前で割り当て直せるように、キーの表に載せる)。パレットと操作の一覧からも開ける。
- 仮定: 画面の呼び名は、日本語は上の欄のモードを「検索」、`/` のモードとキーを「探す」、英語は上の欄を Search、`/` を Find にする。設定の書き方の名前(モードの `search`・`filter`、動作の `search`・`quick_filter`)は、書いた設定が読めなくならないように変えない。
- 仮定: 値を打つ段(含む・含まない・比較)は、窓の中の1行の入力にし、文字を受けるモードを分ける(設定の画面の SettingsText と同じ作り)。

## 設計

変える場所:

- 新しい `src/ui/filters.rs`: 絞り込みの窓 `FilterWin`(選び・段 `Step`)。段は 条件の並び → 列 → 種類 → 比較の記号 → 値の一覧 / 値の入力。ボタンの文字(条件が無ければ「絞り込み」、あれば「絞り込み N」)、置き場所(検索の欄の行の下、右の端にそろえる。並べ替えの窓と同じ `popup::place`)、動作・クリック・重ねて描く `overlay`。条件を変えたら `set_filters`(`set_sorts` と同じく、留めを外して並べ直し、見た目の状態に残す)。
- 設定の画面と共通にする部品: 種類と比較の記号の並び(`KINDS`・`CMPS`)、値ごとの件数(`open_values` の数え方を `value_counts_of` に切り出す)、比べる値の確かめ(`check_operand` を pub(crate))、値の行の文字(`PickValueRow`)。設定の画面の下書き(draft)の選び手は、並べ替え・グループ・WBS の選びと同じ所にあり、まとめて動かすと影響が大きいので、今回は段の進め方だけを窓の側に持ち、部品を共通にする。
- `src/ui/keymap.rs`: `Mode::FilterWin`(設定の名前 `filter_window`)・`Mode::FilterText`(`filter_input`。文字を受ける)、`Action::FilterMenu`(`filter_menu`。既定のキー `|`、パレットのコマンド)。窓の中は 並べ替えの窓と同じキー(Enter・Esc・q・j/k・d/Delete/Backspace)と Space(値のチェック)。入力は Enter・Esc・Backspace。
- `src/ui/bands.rs`: 検索の欄の右の端に「絞り込み」と「並べ替え」を並べる(両方が入るときだけ出す)。`filter_button_at`。
- `src/ui/app.rs`・`src/ui/view.rs`・`src/ui/input.rs`(カーソル)・`src/ui/menu.rs`(操作の一覧): 振り分けと描画。
- 呼び名: `src/i18n.rs` の ModeSearch・ModeFilter・KeySearch・KeyQuickFilter・BarLead など。`docs/keys*.md`・`docs/config*.md`・`src/schema.rs`。

変えない場所: 設定の画面(NV-13・NV-18)の作りと保存。条件の判定(核の mdgrid::settings)。

選んだ案: 並べ替えの窓と同じ、モードを持つ重ねる窓。却下した案: 窓から設定の画面のフィルターの区画を開く → 反映の手間が残り、NV-29 の「すぐ効く」に合わない。却下した案: 設定の画面の選び手を draft から切り離して両方で使う → 並べ替え・グループ・WBS・見た目の選びも同じ型で動いていて、変更の範囲が広くなりすぎる。

要件ごとの対応: NV-29 → filters.rs と bands.rs のボタン、keymap のパレットと操作の一覧。NV-23 → bands.rs の欄の頭の字(検索)とボタンの並び。NV-24 → ボタンの位置(絞り込みの右)。NV-2・NV-1 → 呼び名(i18n・文書)。NV-20 → `set_filters` が設定のフィルターそのものを変える。

検証のコマンド: `cargo test --bin mdgrid test_nv_29`、`cargo test --bin mdgrid test_nv_2`、全体は `cargo test`、仕様は `python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 受け入れの試験を置く(新しいファイル) | NV-29 | 絞り込みの窓と呼び名の試験が実装の前に落ちる | `src/ui/test_filter_window.rs` `src/ui/mod.rs` | test_nv_29_* | 済 |
| 2 | 絞り込みの窓とボタンを作る | NV-29, NV-23, NV-24, NV-20 | test_nv_29_* が通る | `src/ui/filters.rs` `src/ui/keymap.rs` `src/ui/app.rs` `src/ui/view.rs` `src/ui/input.rs` `src/ui/bands.rs` `src/ui/menu.rs` `src/ui/settings.rs` `src/ui/settings_pick.rs` `src/ui/settings_view.rs` `src/i18n.rs` `src/schema.rs` `docs/keys.md` `docs/keys.ja.md` `docs/config.md` `docs/config.ja.md` `tests/golden/` `src/ui/test_filter_window_more.rs` `README.md` `README.ja.md` `docs/design.md` `docs/obsidian-bases.md` `docs/obsidian-bases.ja.md` | test_nv_29_* | 済 |
| 3 | 呼び名をそろえる(上の欄は検索、`/` は探す) | NV-1, NV-2, NV-23 | ヘルプと下の帯で `/` が「探す」、上の欄のモードが「検索」、英語の欄の頭が Search | `src/i18n.rs` `src/ui/keymap.rs` `src/ui/test_search_bar.rs` `src/ui/test_filter_window.rs` `docs/keys.md` `docs/keys.ja.md` `tests/golden/` | test_nv_29_* | 済 |
| 4 | 機能の紹介の VHS の台本を足す | NV-29 | `vhs demos/filter.tape` で絞り込みの窓で条件を足して外す録画ができる | `demos/filter.tape` `demos/README.md` | 読んで判定 | 済 |

## 実装の気づき

- `Action::by_name` はキーの表にある動作しか引かない(キーの無いパレットだけのコマンドは割り当て直せない決まり)。試験と割り当て直しのために、絞り込みの窓に既定のキー `|` を付けた(仮定を直した)。Markdown の表の中では `\|` と書く(SR-22 の文書の読み手が受ける)。
- 人の決定(会話 2026-10-11)で直した錠のある試験: test_nv_29_names_find_and_search は、ヘルプの最初の画面に `/` の行がある前提だったので、ヘルプを下へ流して `/` の行を探す形にした(確かめる中身は同じ)。test_nv_23_config_off_hides_bar_and_uses_bottom_line は、欄が無いときのヘッダーの語の字を「絞り込み「会議」」から「検索「会議」」にした(NV-2 で上の欄の働きを「検索」と呼ぶため)。どちらもこの決定記録で錠を掛け直す。
- レビューの指摘で直した: 段が進んで低い端末で窓が入らなくなったら閉じて理由を出す。呼び直す前のモードの表示名(`検索`・`Search`・`絞り込み`・`Filter`)を書いた設定は前のモードに読む。残っていた古い呼び名(ヘッダー・メッセージ・ヘルプの区画・Esc のキーの名前・文書・README)。
- 残した: NV-29 の例の「done のチェックを外す」は、窓(と設定の画面)の操作では「隠す方式で done にチェックを付ける」になる。結果(done の行が消える)は同じ。値の一覧はマウスだけでは決められない(クリックはチェック、決めるのは Enter。文書どおり)。

## 照合

フレッシュ文脈の検証役(書込なし)が照合した。前のコードには `Mode::FilterWin` などが無くビルドで落ちるので、検証役が実装の前(795a6d1)の作業ツリーに、ビルドを通すためだけの足し(列挙の値・match の腕・空の欄。振る舞いは足さない)をし、HEAD の試験を写して走らせた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| NV-29 | checked | コード: src/ui/filters.rs:67・:193・:212・:452・:484、src/ui/bands.rs:339-360・:378、src/ui/app.rs:812・:966、src/ui/keymap.rs:611・:690、src/ui/menu.rs:143 / テスト: src/ui/test_filter_window.rs::test_nv_29_*(12件)、src/ui/test_filter_window_more.rs::test_nv_29_low_terminal_step_closes_with_reason / 今: 通った(cargo test --bin mdgrid test_nv_29) / 前: 落ちた(ボタンが無い・filter_menu が無い・窓が開かない) |
| NV-23 | checked | コード: src/ui/bands.rs:339-360、src/i18n.rs:938 / テスト: test_filter_window.rs::test_nv_29_buttons_filter_then_sort_at_right_end・test_nv_29_english_bar_lead_is_search、test_search_bar.rs::test_nv_23_*(6件) / 今: 通った / 前: 落ちた(絞り込みのボタンが無い、英語の欄の頭が Filter:) |
| NV-24 | checked | コード: src/ui/bands.rs:356-358・:363 / テスト: test_filter_window.rs::test_nv_29_buttons_filter_then_sort_at_right_end、既存の test_nv_24_*(6件) / 今: 通った / 前: 落ちた(並べ替えの左に絞り込みが無い) |
| NV-2 | checked | コード: src/i18n.rs:672・:783、src/ui/keymap.rs:133-145 / テスト: test_filter_window.rs::test_nv_29_names_find_and_search、test_search_bar.rs::test_nv_23_config_off_hides_bar_and_uses_bottom_line、test_filter_window_more.rs::test_nv_29_old_mode_names_keep_meaning、既存の test_nv_2_* / 今: 通った / 前: 落ちた(上の欄のモード名が「絞り込み」、ヘッダーが「絞り込み「会議」」) |
| NV-1 | checked | コード: src/i18n.rs:671・:780 / テスト: test_filter_window.rs::test_nv_29_names_find_and_search、既存の test_nv_1_*(4件) / 今: 通った / 前: 落ちた(`/` のモード名が「検索」) |
| NV-20 | checked | コード: src/ui/settings.rs:732、src/ui/filters.rs:452 / テスト: test_filter_window.rs::test_nv_29_click_opens_window_and_hide_done・test_nv_29_compare_adds_and_remove_restores、既存の test_nv_20_*(3件) / 今: 通った / 前: 落ちた(窓が無く、設定のフィルターが変わらない) |
| OUT-2 | checked | 読んで判定: 要件の文の「簡易の絞り込み」を「検索」に置き換えただけで、src/ui/export.rs は差分に無い。既存の test_export.rs::test_out_2_export_shown_table が通る |
| SR-18 | checked | 読んで判定: 「簡易の絞り込みを解く」を「検索を解く」に置き換えただけで、nav.rs のキーの処理は差分に無い。既存の test_nav.rs::test_sr_18_esc_clears_selection_then_filter ほか4件が通る |
| CLI-7 | checked | 読んで判定: 「簡易の絞り込み NV-2」を「検索 NV-2」に置き換えただけ。セッションはこの変更の前から未実装(specs/_changes/2026-10-01-sessions.md のタスクが未)で、試験も無い(W12)。この変更はセッションに触れていない |

既存のテストの削除・skip・弱体化: なし(人の決定で直した test_nv_29_names_find_and_search はヘルプを下へ流して `/` の行を探す形にし、見る行の条件はむしろ締まった。test_nv_23_config_off_hides_bar_and_uses_bottom_line は期待する字を NV-2 の呼び名に合わせただけ。どちらも錠をこの決定記録で掛け直した。golden の差分は「絞り込み」のボタンとヘルプの `|` の行だけ)

確かめた: 9 / 9
