---
type: Change
id: 01M451BHK7G42D3DXJBV6C8QWF
title: その場の操作の一覧と、前置きのキーの続きの案内(action-menu)
status: done
size: full
created: 2026-10-05
updated: 2026-10-05
---

# その場の操作の一覧と、前置きのキーの続きの案内(action-menu)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [action-menu](../_decisions/2026-10-05-action-menu.md)(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

会話 2026-10-05 の依頼(lazygit のような操作感)。キーを覚えなくても使えるように、(1) 選んでいるセルで今使える操作の一覧を `x` と右クリックで開いて選んで実行する(SR-24)、(2) 前置きのキー(`g` など)を押したら下の帯に続きのキーを出す(SR-25)。

終わりの条件: SR-24・SR-25 の例のとおり。英語の画面でも出る(SR-23)。変更のあとは画面のスクショを会話に貼る。

## 不明点と仮定

- 人の選択: 開くキーは `x` と右クリック、並べる操作は推しに任せる、案内は下の帯にすぐ。
- 仮定: 一覧に並べる操作とその順は SR-24 の文の4つの節のとおり。各行のキーはキーの表の今のキー(割り当て直しのあと。無い動作はキーの欄を空にする)。
- 仮定: 一覧は表の上に重ねる小さな窓(リストの選択の見せ方と同じ部品)で、選んでいるセルの近くに出す。モードは新しく `Mode::Menu`(設定の名前 `menu`)を足し、キーの表で ↑↓・`j` `k`・Enter・Esc を割り当て直せるようにする(SR-16)。行に添えたキーで実行するのは、一覧の上ではその打鍵を表のモードのキーとして読む。
- 仮定: 文言は src/i18n.rs の表に足す(SR-23)。docs/keys.md・keys.ja.md に動作 `action_menu` と新しいモードの節を足す(SR-22 の試験が突き合わせる)。

## 設計

- src/ui/keymap.rs: `Action::ActionMenu`(名前 `action_menu`)と表のモードの `x`、`Mode::Menu`(`menu`)と、そのモードの移動・実行・閉じるのキー。
- 新しい src/ui/menu.rs: 一覧を作る純関数(今の App の状態から、使える操作の (節, Msg, Action, 今のキー) の並びを作る)と、開く・動かす・実行する・閉じる、描く。実行は `table_action`(キーで直接行ったときと同じ道)に渡す。
- 右クリック: src/ui/app.rs のマウスの処理で、表のセルの右クリックでそのセルを選んで一覧を開く。
- 前置きのキーの案内: 前置きを受けたあと(今の `is_prefix` の待ちの状態)に、下の帯を「`<前置き>` → 続きのキー 動作の名前 …」に替える(src/ui/bands.rs)。続きか Esc で戻る。
- 却下: パレット(`:`)を流用する(パレットはすべてのコマンドで、その場の文脈で絞らない。キーを添えた短い一覧の方が lazygit の手触りに近い)。
- 検証: 新しい試験(下のタスク)、`./ci.sh`、錠と golden が変わらない。終わったら画面のスクショ(scratchpad の写し取りの道具)。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | その場の操作の一覧と前置きのキーの案内 | SR-24, SR-25 | 例のとおりに動き、英語の画面でも出る | src/ui/keymap.rs, src/ui/menu.rs, src/ui/mod.rs, src/ui/app.rs, src/main.rs, tests/golden/sr_5.txt, tests/test_action_menu_e2e.rs, src/ui/bands.rs, src/ui/view.rs, src/ui/help.rs, src/i18n.rs, src/config_items.rs, docs/keys.md, docs/keys.ja.md, docs/config.md, docs/config.ja.md, src/ui/test_action_menu.rs, src/ui/test_action_menu_more.rs, docs/manual-scenarios.toml | test_sr_24_*・test_sr_25_*(src/ui/test_action_menu.rs) | 済 |
| 2 | 照合の gap を埋める試験と、端末が低くなったら一覧を閉じる | SR-23, SR-24, SR-25 | 節「ビューとファイル」・読むだけの起動で保存と新しいノートと書き出しが無い・一覧の上で項目に続かない前置きを受けない・英語の前置きの案内・開いたまま低くすると閉じて理由、の試験が通る | src/ui/app.rs, src/ui/menu.rs, src/ui/test_action_menu_gaps.rs, src/ui/mod.rs | test_sr_2*_gaps_*(src/ui/test_action_menu_gaps.rs) | 済 |

## 実装の気づき

- タスク1: ヘルプ(SR-5)に `x`(`action_menu`)と `menu` のモードのキー(節「操作の一覧の中」)を出す(メインの判断で SR-5 を守る。一時的に入れていた src/ui/help.rs の除外 `menu_key` は外した)。ヘルプが 146 行から 153 行に増えたので(差し戻しの直しで一覧の `g g`・`G` を足した分を含む)、tests/golden/sr_5.txt だけを `UPDATE_GOLDEN` を sr_5 の試験に付けて更新した(件数の表示と、`x` を足して2列の並びが1つずつずれた行。ほかの golden は変わらない)。
- タスク1: `x` は下の帯の順位 0(今の下の帯を変えない)。動作の列挙子は clippy の enum_variant_names のため `Action::Menu`(名前は `action_menu`)。
- タスク1: 一覧の窓は表の上に重ねるだけで、窓の外の表は消さない(差し戻し1回目で直した。提案 action-menu-fixes)。窓の上の縁にモードの名前「操作の一覧」。一覧のモードの下の帯の表示名は既存の「次の候補」「前の候補」を使った。
- タスク1: 「選んだ行にまとめて入れる」は、選んだ行(印か範囲)のうち今の列を直せる行が1つでもあれば出す。今の行が選択の外なら選択の最初の行へ移ってから Enter(`edit`)と同じ道で開く。
- タスク1(差し戻し1回目): 前置きのあとの Esc は前置きを取り消すだけ(src/ui/app.rs の `key`)。一覧の上では、続きに一覧の項目の動作がある表の前置きだけを受け、帯の続きもそれに絞る。窓が入らない(窓の中に3行見せられない)端末では開かず、メッセージ行に「端末が狭くて操作の一覧を出せない」(`Msg::MenuNoRoom`)。項目の左クリックで実行して閉じ、窓の外の左クリックで閉じるだけ(見出しと縁は何もしない)。
- タスク1(差し戻し1回目): `test_sr_25_more_esc_after_prefix_in_menu_keeps_menu` は「一覧で `g` のあと Esc でも一覧が開いたまま」を求める。表の `g`(続きは `top` だけで一覧の項目に無い)を一覧で受けないと、`g` が何もせず Esc で一覧が閉じるので、一覧のモードに詳細の表示・ヘルプと同じ `g g`(先頭の項目)・`G`(末尾の項目)を足した(帯の順位 0、docs/keys.md・keys.ja.md の `menu` の節にも足した)。
- タスク1(要判断・試験の側): 同じ試験の最後の確かめ(55行目)は、一覧が見えることを画面の行全部の中身が「セル」だけの行で見ている。窓の外の表を消さない直し(直す点1)と合わず、80×24 で行0から開くと見出し「セル」の行が c.md の表の行に重なって落ちる(一覧は開いていて、見出しは窓の中にある)。窓の内側で読む `menu_lines` に直すのは試験を書く役。
- タスク1(会話 2026-10-06、人の選択 B: 提案 action-menu-fixes の承認を待たずに撮る): 説明書の場面の定義 docs/manual-scenarios.toml に `action-menu`(`x`)と `prefix-hint`(`g`)を足し、app-manual で英日を撮った。直しが入って見た目が変わったら `--only` で撮り直す。

## 照合

フレッシュ文脈の検証役(書込なし)が 2026-10-06 に照合した(実装前 a8b2290 では錠のある試験がコンパイルでき、実行で落ちた。新しい試験は写しの上の変異で示した)。gap の4つと正しさの指摘1つはタスク2で直し、実装役が写しの変異で落ちることを確かめた(gap の再照合は自分で。フレッシュ文脈でない)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| SR-24 | checked | コード: src/ui/menu.rs(項目の組み立て・窓・クリック・入るかの判定・close_menu_if_no_room)、src/ui/keymap.rs の action_menu と x、src/ui/app.rs の resize とクリック、src/main.rs の右クリック / テスト: src/ui/test_action_menu.rs::test_sr_24_*、src/ui/test_action_menu_more.rs::test_sr_24_more_*、src/ui/test_action_menu_gaps.rs::test_sr_24_gaps_*、tests/test_action_menu_e2e.rs::test_sr_24_e2e_right_click_selects_cell_and_opens_menu / 今: 通った(cargo test) / 前: 落ちた(a8b2290 で「動作 action_menu が無い」「節『セル』を待って時間切れ」。変異: 一覧を開かない → 16本、窓の外を空白 → 1本、読むだけで編集を出す → 1本、外のクリックで実行 → 1本、入るかの判定を外す → 1本、ビューとファイルの節を消す → 3本、読むだけの起動でも保存を出す → 1本、resize で閉じない → 1本) |
| SR-25 | checked | コード: src/ui/bands.rs の prefix_hints、src/ui/app.rs の前置きと Esc、src/ui/menu.rs の menu_key の絞り / テスト: src/ui/test_action_menu.rs::test_sr_25_*、src/ui/test_action_menu_more.rs::test_sr_25_more_*、src/ui/test_action_menu_gaps.rs::test_sr_25_gaps_menu_ignores_prefix_without_menu_item / 今: 通った / 前: 落ちた(a8b2290 で「g のあとの帯に先頭の行が無い」。変異: 案内を出さない → 3本、Esc で選択も解く → 2本、案内を g にだけ出す → 1本、一覧で項目に無い前置きも受ける → 1本) |
| SR-23 | checked | コード: src/i18n.rs の一覧と案内の文言 / テスト: src/ui/test_action_menu.rs::test_sr_24_english_menu_has_no_japanese、src/ui/test_action_menu_gaps.rs::test_sr_25_gaps_english_prefix_hint、tests/test_i18n.rs::test_sr_23_english_has_no_japanese / 今: 通った / 前: 落ちた(変異: 英語の節の名前を日本語に → 2本、First row を日本語に → 1本) |
| WB-5 | checked | コード: src/ui/menu.rs の読むだけのセルの判定 / テスト: src/ui/test_action_menu.rs::test_sr_24_readonly_cell_hides_edit_and_clear / 今: 通った / 前: 落ちた(変異: 読むだけのセルで編集を出す) |
| SR-16 | checked | コード: src/ui/keymap.rs の Mode::Menu と menu のモードのキー / テスト: src/ui/keymap.rs::test_sr_16_no_duplicate_keys_per_mode / 今: 通った / 前: 落ちた(変異: 一覧のモードの G を重ねる → 4本) |

既存のテストの削除・skip・弱体化: なし。錠のある src/ui/test_action_menu.rs は決定 action-menu-fixes(人が判断を AI に委ねた承認)で掛け直した(道具 menu_lines を窓の枠の内側で読むように変え、試験を1本足した。assert は減っていない)。tests/golden/sr_5.txt はヘルプに x が足された分だけ。

確かめた: 5 / 5

残した気づき: 並べ替えの項目の文言はキーの表の表示名「一時的な並べ替え」(SR-24 の例の「この列で並べ替え」と違うが、要件の文は「動作の名前」で食い違いは無い)。e2e の一覧の見分け方は画面の配置に頼る(錠のある tests/test_action_menu_e2e.rs。今は通る)。
