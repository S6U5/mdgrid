---
type: Change
id: 01M799C8J9A9V8585NV6HD2SFE
title: 列の値ごとのレーンにノートをカードとして並べる「ボード」の画面を足す(board-view)
status: active
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
| 不明点 | 進行中 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [決定](../_decisions/2026-11-13-board-view.md) |
| 設計 | 進行中 | この記録の「設計」 |
| タスク | 進行中 | この記録の「タスク」 |
| 実装 | 進行中 | コミット |
| 照合 | 未 | この記録の「照合」 |

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
| 1 | レーンに分ける核 | BD-2 | 値と順を渡すと、決まりの順・文字の順・最後に「(なし)」のレーンが返る | src/board.rs, src/lib.rs | tests/test_board.rs の `test_bd_2_*` | 未 |
| 2 | ボードの画面(開く・描く・動く・表へ) | BD-1, BD-2, BD-3, BD-5 | `B` でレーンとカードが出て、Enter で表のその行、Esc で戻る | src/ui/board.rs, src/ui/keymap.rs, src/ui/app.rs, src/ui/view.rs, src/ui/bands.rs, src/ui/help.rs, src/ui/screen_menu.rs, src/ui/mod.rs, locales/en.toml, locales/ja.toml, docs/keys.md, docs/keys.ja.md | src/ui/test_board.rs・tests/test_board_e2e.rs の `test_bd_1_*`・`test_bd_3_*`・`test_bd_5_*` | 未 |
| 3 | カードを移す・取り消し・保存 | BD-4 | `L` で隣のレーンへ移って未保存 1、`u` で戻る、Ctrl+S で保存の確認 | src/ui/board.rs | src/ui/test_board.rs・tests/test_board_e2e.rs の `test_bd_4_*` | 未 |
| 4 | マウス | BD-6 | カードのクリックで選び、もう一度で表のその行 | src/ui/board.rs, src/ui/app.rs | src/ui/test_board.rs の `test_bd_6_*` | 未 |
| 5 | 文書: 見せ方の一覧・できることの一覧・場面・設計 | BD-1 | 説明書の「見せ方の一覧」に画面1枚と1項 | docs/manual, docs/manual-scenarios.toml, docs/design.md, README.md, README.ja.md | 読んで判定 | 未 |

## 実装の気づき

- レビュー1回目の版: 2d5ca81fe214c0c3332ff6d9f605045ed29cd2bc(作業ツリーには、別の記録 save-undo・regex-replace の実装も入っている。この記録の範囲はタスクの表のファイル。`src/board.rs`・`src/ui/board.rs` は追跡していない新しいファイル)。
- テスト役のテスト 54 本のうち 2 本(表の行を見出しと読み違える検査・同じセルの未保存の数)はテストの側の誤りだった。人に聞いて(TL-7)テストを直し、決定 2026-11-13-board-tests-locks で錠を掛け直した。
- 隠れたレーンの印 `<`・`>` は、設計の「見出しの行」をテスト役がレーンの題の行と読んだ。レーンの題の行(窓の上の縁)の両端に置く形にした。
- モードを足すと、設定の項目 keys の説明(`src/schema.rs`)・`docs/config*.md`・`docs/keys*.md` のモードの節・ヘルプのゴールデン(`tests/golden/sr_5.txt`)が変わる。

## レビューの指摘

なし

## 照合

| 要件 | 結果 | 証拠 |
|---|---|---|

確かめた: 0 / 0
