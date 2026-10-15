---
type: Change
id: 01M4XVVAFV8SMHJWKFVA63WMX1
title: Windows を対象に入れる(設定の置き場・既定のエディタ・クリップボード・書き戻しの確かめ)(windows-target)
status: active
size: full
created: 2026-10-15
updated: 2026-10-15
---

# Windows を対象に入れる(設定の置き場・既定のエディタ・クリップボード・書き戻しの確かめ)(windows-target)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [2026-10-15-windows-target](../_decisions/2026-10-15-windows-target.md) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | 「実装: [CLI-3][SR-8][OUT-1][WB-5][WB-8] …(specs/_changes/2026-10-15-windows-target.md)」のコミット |
| 照合 | 進行中 | この記録の「照合」 |

## 要求

人が Windows を mdgrid の対象に入れると決めた(会話 2026-10-14。プロジェクトのチャット。docs/todo.md の O-11 の人の判断)。クラウドの調べで見つかった次を直す: (1) 設定の置き場が `HOME` だけを見る(CLI-3)→ Windows では `%APPDATA%` に倒す。(2) 既定のエディタが `vi`(SR-8)で、OS のクリップボードのコマンドが pbcopy・wl-copy・xclip だけ(OUT-1)→ Windows の代わりを足す。(3) 書き戻しの安全(ハードリンクの見分け・ほかのアプリが開いているファイルへの名前の変更)は Windows で確かめていない → 安く確かめられる分を足し、残りは todo に書く。そのうえで CI の windows の試験の `continue-on-error` を外す。終わりは、手元の macOS で `cargo test` が通り、Windows の分岐が macOS でも試験で確かめられ、docs/todo.md の O-11 が「対象に入れる」で決着していること。

## 不明点と仮定

- 仮定: Windows の状態の置き場は `%LOCALAPPDATA%\mdgrid\`(端末ごとの物で、ローミングしない)。`XDG_STATE_HOME` があればそちら。外れたら: `%APPDATA%` に寄せる。
- 仮定: Windows でも `XDG_CONFIG_HOME`・`XDG_STATE_HOME`(絶対パス)を先に見る(Git Bash などで置く人と、試験のため)。
- 仮定: `~` のホームは `HOME` が無ければ Windows では `USERPROFILE`(places.toml の `~/…` のため)。
- 仮定: Windows の OS のクリップボードは `clip`(どの Windows にもある)。日本語を化けさせないため、標準入力には BOM つきの UTF-16LE で、改行を CRLF にして渡す。
- 仮定: notepad は閉じるまで戻らない(エディタを待って読み直す SR-8 の形のまま)。Windows 11 の新しいメモ帳でも、起動したプロセスは閉じるまで残る。外れたら: 設定の `editor` か `%EDITOR%` を置くよう文書に書く。
- 仮定: 手元(macOS)では Windows 向けに組めない(rustup が無い)。Windows の分岐は `cfg!(windows)` の値を引数に取る純な関数にして macOS の試験で確かめ、`#[cfg(windows)]` だけで組む部分(書き戻しの試験)は公開リポの CI で確かめる。
- ハードリンクの見分けは Windows では std の安定版に数を取る手段が無く(`number_of_links` は不安定)、Win32 を直接呼ぶ必要がある。今回は入れず todo に残す(WB-5 の Windows の抜け)。

## 設計

- 置き場(CLI-3・SR-12): `config.rs` の `base_dir` を、環境変数を読む関数と Windows かを引数に取る純な `base_dir_in` にする。順は `XDG_*`(絶対パス)→ Windows は `APPDATA`・`LOCALAPPDATA`、ほかは `HOME` の下。`home_dir()`(`HOME`、無ければ Windows は `USERPROFILE`)を足し、`places.rs`・`workspace.rs`・`ui/places.rs` の `HOME` の読み取りをこれにそろえる。依存の crate(dirs)は足さない(環境変数を1つ読むだけで足りるため)。
- エディタ(SR-8): `config::DEFAULT_EDITOR`(`default_editor_for(cfg!(windows))`。Windows は `notepad`、ほかは `vi`)を `resolve_editor` と `editor_argv` の既定にする。錠のある3つの試験は `"vi"` の決め打ちを `DEFAULT_EDITOR` に変える(提案の `- 錠:`)。
- クリップボード(OUT-1): `ui/external.rs` に OS ごとのコマンドの表 `clip_commands(os)` と渡す形 `ClipInput`(UTF-8 / Windows の文字)を置き、`os_copy` はそれを順に試す。`pipe_to` はバイト列を渡す `pipe_bytes_to` の包みにする(呼ぶ側と試験はそのまま)。
- 書き戻し(WB-5・WB-8・WB-14): コードは変えない。`#[cfg(windows)]` の試験を新しいファイル `tests/test_windows_writeback.rs` に置く: ほかのハンドルが削除の共有なしで開いているノートの保存は誤りで止まり、元のファイルが変わらず一時ファイルが残らない。読み取り専用の属性のノートは `NoPermission`。日本語の名前のノートは書ける。
- CI: windows の試験の `continue-on-error` を外す。
- 文書: docs/design.md の置き場とクリップボード、docs/config.md・docs/config.ja.md の置き場とエディタ、src/schema.rs のエディタの説明、説明書の concepts.md の置き場の表の下に Windows の置き場を足す。docs/todo.md の O-11 を決着させ、残り(ハードリンク・疑似端末の試験・notepad の待ち)を書く。
- 検証: `cargo test`、`cargo clippy --all-targets -- -D warnings`、`python3 .decidespec/scripts/check.py .`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 設定と状態の置き場とホームを Windows に倒す | CLI-3 | Windows の環境で `APPDATA`・`LOCALAPPDATA`・`USERPROFILE` を使う | `src/config.rs` `src/places.rs` `src/workspace.rs` `src/ui/places.rs` `src/test_windows_unit.rs` | test_cli_3_windows_* | 済 |
| 2 | 既定のエディタを Windows で notepad にする | SR-8, SR-19 | どれも無いとき Windows は notepad | `src/config.rs` `src/ui/external.rs` `src/test_config_unit.rs` `src/ui/test_external.rs` `src/ui/test_click_name.rs` `src/test_windows_unit.rs` | test_sr_8_default_editor_* | 済 |
| 3 | Windows の OS のクリップボードに clip を使う | OUT-1 | Windows の表は clip で、BOM つき UTF-16LE と CRLF | `src/ui/external.rs` `src/ui/test_windows_clip.rs` | test_out_1_windows_* | 済 |
| 4 | Windows の書き戻しの試験を足す | WB-5, WB-8, WB-14 | 開いているノートの保存が止まり元が変わらない | `tests/test_windows_writeback.rs` | test_wb_8_windows_* | 済 |
| 5 | CI と文書と todo を直す | CLI-3, SR-8 | windows の試験が止める側になり、文書に Windows の置き場 | `.github/workflows/ci.yml` `docs/` `src/schema.rs` | 読んで判定 | 済 |

## 実装の気づき

- 試験は実装と同じ文脈で書いた(独立したテスト役ではない)。Windows の分岐は新しい関数がないと組めないので、試験を先にコミットすると組めない木になる。試験と実装を1つのコミットにした。
- 他のスレッドが同じ作業ツリーで未コミットの変更を持つ(docs/config.md・docs/config.ja.md・src/schema.rs・specs/cli/spec.md)。それらのファイルは HEAD に自分の置き換えだけを入れた版を index に置いてコミットし、錠も自分の試験の分だけを書いた。
- 手元に Windows の組み立ての道具が無い。`#[cfg(windows)]` の tests/test_windows_writeback.rs は公開リポの CI で初めて組まれる。名前の変更が開いているノートを置き換えられるかは std の版で違いうるので、試験は「置き換えたら新しい中身、だめなら元のまま」の両方を安全とみなす。
- zsh では `path` が PATH と結びついた変数なので、`while read … path` と書くと PATH が壊れる。

## 照合

新しい文脈の検証役(書込なし、2026-10-15)の照合。実装の前 0da68b4、実装 567166e。手元は macOS で Windows 向けに組めないので、「前」は前のコードを読んで判定した。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-3 | checked | コード: src/config.rs の config_path_in・state_dir_in・home_dir_in・base_dir_in / テスト: src/test_windows_unit.rs::test_cli_3_windows_config_in_appdata ほか 3 / 今: 通った(cargo test --locked --lib tests_windows) / 前: 落ちる(読んで判定: 0da68b4 の base_dir は XDG のあと HOME だけを見た) |
| SR-8 | checked | コード: src/config.rs の DEFAULT_EDITOR・default_editor_for・resolve_editor、src/ui/external.rs の editor_argv / テスト: src/test_windows_unit.rs::test_sr_8_default_editor_per_os / 今: 通った(cargo test --locked --lib test_sr_8、--bin mdgrid test_sr_8) / 前: 落ちる(読んで判定: 前は vi の決め打ちで OS で分けなかった) |
| SR-19 | checked | コード: src/ui/external.rs の editor_argv / テスト: src/ui/test_click_name.rs::test_sr_19_empty_editor_click_same_as_e / 今: 通った(cargo test --locked --bin mdgrid test_sr_19_empty_editor) / 前: 落ちる(読んで判定: 前は Windows でもクリックが vi を起動した。触ったのは錠の掛け直しだけ) |
| OUT-1 | checked | コード: src/ui/external.rs の clip_commands・ClipInput::encode・os_copy・pipe_bytes_to / テスト: src/ui/test_windows_clip.rs::test_out_1_windows_uses_clip ほか 1 / 今: 通った(cargo test --locked --bin mdgrid tests_windows) / 前: 落ちる(読んで判定: 前は unix でない OS のコマンドの表が空) |
| WB-5 | gap | 足りない: Windows ではハードリンクを見分けず書いてしまう(refuse_hard_link が cfg(not(unix)) で何もしない)。読み取り専用の属性の試験 tests/test_windows_writeback.rs::test_wb_5_windows_read_only_attribute_is_not_written は手元で組めず、書き戻しのコードは変えていないので前でも通る。行き先: docs/todo.md の O-11 に残した(依頼が「安く確かめられる分だけ」で、ハードリンクは Win32 を直接呼ぶ必要がある) |
| WB-8 | unassessed | 理由: tests/test_windows_writeback.rs の2つの試験は Windows 専用で、手元で組めず、公開リポの CI にまだ出ていない。読んで判定では std の API の使い方は正しく、組める。書き出しのあとの Windows の CI で確かめる(docs/todo.md の O-11) |
| WB-14 | unassessed | 理由: WB-8 と同じ試験で、書き戻しの層の「止まって元のまま、閉じたあとのやり直しで書ける」だけを確かめる。画面の層の未保存の数は Windows で確かめていない。書き出しのあとの Windows の CI で確かめる |

既存のテストの削除・skip・弱体化: 削除と skip は無い。3つの試験の `"vi"` の決め打ちを `DEFAULT_EDITOR` に変えた(決定 2026-10-15-windows-target の錠の掛け直し)。`"vi"`・`"notepad"` の文字の値は新しい test_sr_8_default_editor_per_os が確かめるので、確かめは弱まらない。

検証役の指摘のうち要件に関わるもの: Windows のエディタの値の `\` が引用符の外で打ち消しになる(SR-8)。docs/config.md に `/` か引用符で書くよう案内し、直すかを docs/todo.md の O-11 に残した。Git Bash 流の `XDG_CONFIG_HOME=/c/Users/…` は Windows では絶対パスでないので無視して APPDATA に倒れる(仕様のとおり)。

unassessed が残るので、この記録は active のままにする(Windows の CI の結果で WB-8・WB-14 を埋める)。

確かめた: 4 / 7
