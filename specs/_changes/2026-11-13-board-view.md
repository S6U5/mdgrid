---
type: Change
id: 01M799C8J9A9V8585NV6HD2SFE
title: 列の値ごとのレーンにノートをカードとして並べる「ボード」の画面を足す(board-view)
status: done
size: full
created: 2026-11-13
updated: 2026-11-13
---

# 列の値ごとのレーンにノートをカードとして並べる「ボード」の画面を足す(board-view)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。順番どおり(`specs/README.md` の `pace: sequential`)なら、前のフェーズが済か対象外になるまで、次を始めない。同時進行(`pace: concurrent` か無い)なら、要求を済にしたあと不明点から実装までを並べて進め、照合はそれらが済か対象外になってから。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定](../_decisions/2026-11-13-board-view.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人から、機能の改善と追加の依頼があり(会話 2026-11-13)、仕様で「次の段」に残っている候補を選択肢で見せたところ選ばれた4つのうち、「カンバンの画面」をこの記録で扱う。

- 何を: status のような列の値ごとに、縦のレーンへノートをカードとして並べる画面の型(ボード)を足す。カードを隣のレーンへ移すと、その列の値を書き換える変更としてためる。
- なぜ: グループ分け(BV-5)は縦に1列で、状態ごとの量と偏りを横に見比べられない。状態を変えるにも、セルを1つずつ開いて値を選ぶことになる。
- 終わりの条件(例。値は作ったもの):
  - status が todo(2行)・doing(1行)・done(1行)の表で、status の列にカーソルを置いて `B` → 3つのレーンに4枚のカードが出る。
  - todo のカードで `L` → doing のレーンに移り「未保存 1」、ファイルは変わらない。Ctrl+S → 保存の確認に差分が出る。
  - 数の列で `B` → 切り替わらず理由が出る。
- 関係する要件: BD-1〜BD-6(この変更で足す)・SC-9(別の画面の型)・SC-8(.base の kanban は開かないまま)・REL-8(画面の型のメニュー)・CE-36(決まりの values)・WB-9・WB-10・WB-15・CE-8・CE-9・SR-29(ノートの名前)。

## 不明点と仮定

- 仮定: まとめる列は、切り替えたときにカーソルのあった列(設定なし)。理由: 木の画面と違い、列が1つ決まれば作れるので、印を書かずに試せるほうがよい。外れたら: 印の設定(BD-7。次の段)を先に入れる。
- 仮定: まとめられるのはテキストの列だけ。理由: 移すと値を書き換えるので、数・日付・リストでは型と合わない値を作りうる。外れたら: リストの列の扱いを提案にする。
- 仮定: レーンの順は、列の決まりの `values`(CE-36)、残りは文字の順。理由: 新しい設定を足さずに、todo → doing → done の順を書ける。外れたら: 印に順を書く形(BD-7)を入れる。
- 仮定: カードを移すキーは `H`・`L`(と Shift+←・→)。理由: レーンを選ぶ `h`・`l` の大文字で、覚えやすい。表の画面の `H`・`L`(列を動かす)とはモードが違うのでぶつからない。外れたら: 割り当て直せる(SR-13)。

## 設計

- 変える場所:
  - `src/board.rs`(新しいファイル。核。画面に頼らない): `pub struct Lane { pub value: Option<String>, pub rows: Vec<RowId> }`(`value` が None は「(なし)」)と、`pub fn lanes(rows: &[(RowId, Option<String>)], order: &[String]) -> Vec<Lane>`(渡した行の並びを保ってレーンに分ける。`order` の値を先に(行が無くても出す)、残りは文字の順、None のレーンは行があるときだけ最後)。`src/lib.rs` に `pub mod board;`。
  - `src/ui/board.rs`(新しいファイル。`impl App` の続きと描画): 状態 `Board { col, lanes, lane, card, … }` を `App.board: Option<Board>` に持つ。`open_board()`(列を確かめて組み、`Mode::Board` へ)、`rebuild_board()`(ためる変更・外の変化・取り消しのあとに、選んだカードを保って組み直す)、`close_board()`、`board_action(action)`、`board_click(x, y)`、`render(app, w, h)`。レーンの値は、ためた値があればその値(`Changes::pending`)、無ければ読んだ値の文字。
  - カードを移す: `Changes::set`(1手)に、移した先のレーンの値(`NewValue::Str`)か、「(なし)」なら `NewValue::Null` を渡す。読むだけの理由は `set` の `Skip` をそのまま出す。
  - `src/ui/keymap.rs`: `Mode::Board`(名前 `board`)、`Action::BoardView`(名前 `board_view`。表のモードの既定のキー `B`、ボードの中でも同じキーで戻る)、`Action::CardLeft`・`Action::CardRight`(名前 `card_left`・`card_right`。ボードの中の既定 `H`・`Shift+Left` と `L`・`Shift+Right`)。ボードの中のキー: `h`・`Left`・`l`・`Right`(レーン)、`j`・`Down`・`k`・`Up`(カード)、`Enter`(表のその行へ)、`u`・`Ctrl+z`(取り消し)、`Ctrl+r`・`U`(やり直し)、`Ctrl+s`(保存: ボードを閉じて保存の確認)、`Esc`・`q`・`B`(表に戻る)、`?`(ヘルプ)、`:`(パレット)。
  - `src/ui/screen_menu.rs`: 画面の型に「ボード」を足す(いつも出す。選んだ列がまとめられなければ理由を出して表のまま)。`src/ui/app.rs`(`apply`・クリック・ホイールの分岐)、`src/ui/view.rs`(描画の分岐)、`src/ui/bands.rs`(`on_board`)、`src/ui/help.rs`(パレットを閉じたら戻るモード)。
  - `locales/en.toml`・`locales/ja.toml`、`docs/keys.md`・`docs/keys.ja.md`、説明書(`views.md` に画面1枚と1項、`features.md`・`shots.md`)と場面の台本、`docs/design.md`。
- 変えない場所: `.base` の読み取り(SC-8。kanban のビューは今までどおり未対応)、グループ分け(BV-5)、`Changes`・書き戻し。
- 選んだ案: 木の画面(`src/ui/folder_tree.rs`)と同じ組み立て(核の純関数 + `App` の状態 + モード + 描画)。レーンに分ける所を核の純関数にして、画面なしで並びの決まり(BD-2)を確かめる。
- 却下した案: グループ分けの見た目を横に並べ替える設定にする(SC-9 に反する。表の描画に別の画面を混ぜる)。印の設定を必須にする(試しにくい。次の段の BD-7)。
- 名前と文言(試験が探す語): 画面の型の名前は、日本語「ボード」・英語「Board」。レーンの見出しは「値 数」(例 `todo 2`)。空のレーンは、日本語「(なし)」・英語「(none)」。まとめられない列の理由は、日本語「この列ではボードにできない」・英語「this column cannot make a board」で始める。左右に隠れたレーンの印は、見出しの行の左端の `<` と右端の `>`。上下に隠れたカードの印は、そのレーンの見出しの数のあとの `^`・`v`。選んでいるカードは、1行目の頭に `>`。カードを移したメッセージは、日本語「… へ移した」・英語「moved to …」。下の帯の未保存の数の出し方は表の画面と同じ。
- 要件の対応: BD-1 → `ui/board.rs` の `open_board`・`keymap.rs`・`screen_menu.rs`。BD-2 → `board::lanes` と `ui/board.rs` の組み立て(`values` は表のプロファイルの決まり)。BD-3 → `ui/board.rs` の描画と Enter。BD-4 → `board_action` の `CardLeft`・`CardRight`・`Undo`・`Redo`・`Save`。BD-5 → `ui/board.rs` の段組み。BD-6 → `board_click` とホイール。
- 関わる束: `cargo test --lib test_bd_`・`cargo test --bin mdgrid test_bd_`・`cargo test --bin mdgrid test_keys_doc`・`cargo test --bin mdgrid test_header_menu` と `cargo test --test test_board --test test_board_e2e`
- 全体: `cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`cargo test`・`python3 .decidespec/scripts/check.py .`

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | レーンに分ける核 | BD-2 | 値と順を渡すと、決まりの順・文字の順・最後に「(なし)」のレーンが返る | src/board.rs, src/lib.rs | tests/test_board.rs の `test_bd_2_*` | 済 |
| 2 | ボードの画面(開く・描く・動く・表へ) | BD-1, BD-2, BD-3, BD-5 | `B` でレーンとカードが出て、Enter で表のその行、Esc で戻る | src/ui/board.rs, src/ui/keymap.rs, src/ui/app.rs, src/ui/view.rs, src/ui/bands.rs, src/ui/help.rs, src/ui/screen_menu.rs, src/ui/mod.rs, locales/en.toml, locales/ja.toml, docs/keys.md, docs/keys.ja.md | src/ui/test_board.rs・tests/test_board_e2e.rs の `test_bd_1_*`・`test_bd_3_*`・`test_bd_5_*` | 済 |
| 3 | カードを移す・取り消し・保存 | BD-4 | `L` で隣のレーンへ移って未保存 1、`u` で戻る、Ctrl+S で保存の確認 | src/ui/board.rs | src/ui/test_board.rs・tests/test_board_e2e.rs の `test_bd_4_*` | 済 |
| 4 | マウス | BD-6 | カードのクリックで選び、もう一度で表のその行 | src/ui/board.rs, src/ui/app.rs | src/ui/test_board.rs の `test_bd_6_*` | 済 |
| 5 | 文書: 見せ方の一覧・できることの一覧・場面・設計 | BD-1 | 説明書の「見せ方の一覧」に画面1枚と1項 | docs/manual, docs/manual-scenarios.toml, docs/design.md, README.md, README.ja.md | 読んで判定 | 済 |

## 実装の気づき

- レビュー1回目の版: 2d5ca81fe214c0c3332ff6d9f605045ed29cd2bc(作業ツリーには、別の記録 save-undo・regex-replace の実装も入っている。この記録の範囲はタスクの表のファイル。`src/board.rs`・`src/ui/board.rs` は追跡していない新しいファイル)。
- テスト役のテスト 54 本のうち 2 本(表の行を見出しと読み違える検査・同じセルの未保存の数)はテストの側の誤りだった。人に聞いて(TL-7)テストを直し、決定 2026-11-13-board-tests-locks で錠を掛け直した。
- 隠れたレーンの印 `<`・`>` は、設計の「見出しの行」をテスト役がレーンの題の行と読んだ。レーンの題の行(窓の上の縁)の両端に置く形にした。
- 確かめる回(別の新しい文脈): 指摘 1・2 は本当。直し: 題は値の側を切って数と印を残す(1)。カードの名前は、ノートの欄を隠す判定を通さない名前(共通のフォルダと `.md` を除き、名前の正規表現を当てる)にし、表の行の印を付けない(2)。読んだ値のレーンへ戻すときは、ためた値を外す(3。2行以上の値のレーンへ別のカードを移すと1行目だけが入る点は、差分に出るので見送り)。クリックの当たりを見えるカードまでに(4)。5 は木の画面と同じ形で、この変更では触らない。直しの試験は `src/ui/test_board_fixes.rs`(実装した本人が書いた。独立したテスト役ではない)。
- 直しの試験で分かったこと: カードを移して空になったレーンが消えると、元のレーンへ戻せない。開いている間はレーンを消さず、前の並びを保つ(`rebuild_board`)。
- モードを足すと、設定の項目 keys の説明(`src/schema.rs`)・`docs/config*.md`・`docs/keys*.md` のモードの節・ヘルプのゴールデン(`tests/golden/sr_5.txt`)が変わる。

## レビューの指摘

| # | 出どころ | 区分 | 箇所 | 指摘 | 行き先 |
|---|---|---|---|---|---|
| 1 | レビュー | 重要 | src/ui/board.rs:454 | レーンの題に「値 数 ^ v」を詰めて窓の題に渡すので、題が長いと末尾から切れ、カードの数と上下に隠れたカードの印が消える(幅 20 のレーンで値 `in-progress`・12 枚・上下に隠れあり → 印が出ない。幅 18 では数も消える)。値の側を切って、数と印を残す形が要る(BD-2・BD-5)。 | 直した |
| 2 | レビュー | 重要 | src/ui/board.rs:395 | カードの名前に表の左のノートの欄の文字(`view::label`)を使うので、ノートの欄を出さない表(ビューに `file.name` の列がある・`note_path = off`・CSV の表)では、カードの1行目が `>` か `*` だけになり、どのノートのカードか分からない(BD-3)。親子の印・WBS の番号・留めた行の `~` も名前に混ざる。 | 直した |
| 3 | レビュー | 細部 | src/ui/board.rs:238 | 隣へ移して元のレーンへ戻しても、元の値がレーンの値の文字と違う行(テキストの列の中の数・真偽、空の文字列、2行以上の値)は、未保存が残る。2行以上の値のレーンへ別のカードを移すと、1行目だけの値が入る。 | 直した |
| 4 | レビュー | 細部 | src/ui/board.rs:345 | クリックの当たりに「見えるカードの数より下は無い」が入っていず、最後の空きの行のクリックが、描いていない次のカードを選ぶ。内側が1行のとき、描いていないカードのクリックで表へ移る。 | 直した |
| 5 | レビュー | 細部 | src/ui/help.rs:565 | ボードの中の `:<番号>` は、見えない表の行を動かすだけで、ボードの選びは変わらない(木の画面も同じ形)。 | 見送り |
| 6 | セキュリティ | 細部 | src/ui/board.rs:244 | カードを移すとき、改行を含む値を断らない。レーンの値は1行目だけなので、`"a\nb"` のような値のカードを移すと、値の全体が移した先の1行の文字で置き換わる(差分には出る)。 | 直した |

## 照合

検証役(実装を見ていない新しい文脈。書込なし)が、実装の前(acf43ac に今の e2e のテストを重ねた worktree)と後(b92e7b0)で確かめた。証拠のスクリプト(`check.py --evidence`)は「前: 要件の手前で落ちた」(単体試験が、実装前には無い名前を使うのでコンパイルで落ちる)として gap を出したので、前で落ちることは e2e で確かめた。

| 要件 | 結果 | 証拠 |
|---|---|---|
| BD-1 | checked | コード: src/ui/board.rs:156 / テスト: tests/test_board_e2e.rs::test_bd_1_board_shows_lanes_and_back_e2e / 今: 通った(`cargo test --test test_board --test test_board_e2e`・`cargo test --bin mdgrid test_bd_`) / 前: 落ちた(`B` を押しても表のままで、ボードの画面を待って時間切れ) |
| BD-2 | checked | コード: src/board.rs:15 / テスト: src/ui/test_board_lanes.rs::test_bd_2_emptied_none_lane_stays_until_reopened / 今: 通った(`cargo test --test test_board`・`cargo test --bin mdgrid test_bd_2`) / 前: 落ちた(ボードの画面が無い。BD-1 の e2e で確かめた) |
| BD-3 | checked | コード: src/ui/board.rs:424 / テスト: src/ui/test_board.rs::test_bd_3_card_name_and_first_two_columns / 今: 通った(`cargo test --bin mdgrid test_bd_`) / 前: 落ちた(ボードの画面が無い。BD-1 の e2e で確かめた) |
| BD-4 | checked | コード: src/ui/board.rs:244 / テスト: tests/test_board_e2e.rs::test_bd_4_move_card_then_save_writes_file_e2e / 今: 通った(`cargo test --test test_board_e2e`・`cargo test --bin mdgrid test_bd_`) / 前: 落ちた(ボードの画面を待って時間切れ) |
| BD-5 | checked | コード: src/ui/board.rs:83 / テスト: src/ui/test_board.rs::test_bd_5_lines_fit_any_width_and_height / 今: 通った(`cargo test --bin mdgrid test_bd_`) / 前: 落ちた(ボードの画面が無い。BD-1 の e2e で確かめた) |
| BD-6 | checked | コード: src/ui/board.rs:364 / テスト: src/ui/test_board.rs::test_bd_6_click_card_selects_then_goes_to_table_row / 今: 通った(`cargo test --bin mdgrid test_bd_`) / 前: 落ちた(ボードの画面が無い。BD-1 の e2e で確かめた) |

既存のテストの削除・skip・弱体化: なし(既存のテストのファイルは変えていない。ゴールデン `tests/golden/sr_5.txt`・`sr_14.txt` は、足したキーとコマンドの分と、パレットの表示名の欄を広げた分が変わった)。受け入れのテスト 2 本の直し(決定 2026-11-13-board-tests-locks)は、検証役が弱体化でないと判定した

gap と行き先: 検証役は BD-2 を gap にした(開いている間は空になったレーンを消さない実装が、BD-2 の文に無い)。行き先は「提案にした」: 決定 2026-11-13-board-lanes-stay(AI 自己採択。錨との照合あり)で BD-2 に書き足し、`src/ui/test_board_lanes.rs` の試験を足して、今のコードで通ることを確かめた(再照合 1 回)。検証役のほかの指摘: 改行を含む値のカードを移せる点は、セキュリティの回の指摘と同じで直した。テキストの列の中の数の値から出来たレーンへ移すと文字でためる点は、差分に出る端の場合として残す

セキュリティの回: 指摘 1 件(細部。重要なし)

全体の走り: 約 4 分(`cargo test --no-fail-fast`。最後の「実装:」の直前。前から落ちる `test_cliff_history_prefixes_all_covered` の1本を除いて通った。この1本は、この変更を外しても落ちる)。`cargo fmt --check`・`cargo clippy --all-targets -- -D warnings`・`check.py` は通った

確かめた: 6 / 6
