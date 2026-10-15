---
type: Proposal
id: 01M4XVVAEWWEQEM6P0Z9ZCBMH7
title: Windows を対象に入れ、設定の置き場とエディタの既定を Windows に合わせる(windows-target)
status: accepted
decided-by: AI
trigger: 人の発言
trigger-link: 会話 2026-10-14
touches: [CLI-3, SR-8, SR-19]
created: 2026-10-15
updated: 2026-10-15
---

# Windows を対象に入れ、設定の置き場とエディタの既定を Windows に合わせる(windows-target)

## きっかけ

AI がプロジェクトのチャットで Windows を対象に入れるかを尋ね、人が入れると答えた(会話 2026-10-14。プロジェクトのチャット)。docs/todo.md の O-11 は「対象に入れるか」を人の判断として残していた。クラウドの調べで、設定の置き場が `HOME` だけを見る(Windows ではふつう無い)、エディタの既定が `vi`(Windows にはふつう無い)、OS のクリップボードのコマンドが Unix のものだけ、と分かった。置き場に `%APPDATA%` を使うこと、既定のエディタを `notepad` にすることは AI が足した細部。

## 差分

- 変更: CLI-3 旧「設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`)に1つ置くべき。項目は、アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、端末の性質の `[terminal]`(`color`・`ambiguous_wide`・`nerd_font`)、`[workspace]`、`[keys]`)と、表の見え方と振る舞いを決める表のプロファイル(`[look]`(テーマ・組・セルの部品の切り替え・部品の形 `[look.style]`・列ごとの見せ方 `[look.columns]`・色 `[look.colors]`)・`[display]`・`[dates]`・`[edit]`(候補の数(CE-3)・フロントマターを足すか(WB-3))・`[new_note]` と、土台にするテンプレートの名前 `use`)に分け、名前を付けたプロファイルの断片(テンプレート)を `[templates.<名前>]` に書けるべき(全項目は CLI-12 の文書)。画面で選んだ設定(SR-43)は、同じ置き場の `ui.toml` に config.toml と同じ形で置いて config.toml の上に重ね、config.toml は書き換えないべき。設定が無くても既定で動くべきで、知らない項目・型の違う値・その場所に書けない項目は警告にとどめるべき。警告は、どの設定のファイル(config.toml・ui.toml・views.toml・workspaces.toml・ワークスペースの印・places.toml・sessions.toml)でも「ファイルの名前: 項目の道筋(`look.style.status` など): 理由」の形にそろえるべき。」→ 新「設定ファイルは TOML で、OS の設定の置き場(`XDG_CONFIG_HOME` か、macOS でも `~/.config/mdgrid/config.toml`。Windows では `XDG_CONFIG_HOME` が無ければ `%APPDATA%\mdgrid\config.toml`)に1つ置くべき。項目は、アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、端末の性質の `[terminal]`(`color`・`ambiguous_wide`・`nerd_font`)、`[workspace]`、`[keys]`)と、表の見え方と振る舞いを決める表のプロファイル(`[look]`(テーマ・組・セルの部品の切り替え・部品の形 `[look.style]`・列ごとの見せ方 `[look.columns]`・色 `[look.colors]`)・`[display]`・`[dates]`・`[edit]`(候補の数(CE-3)・フロントマターを足すか(WB-3))・`[new_note]` と、土台にするテンプレートの名前 `use`)に分け、名前を付けたプロファイルの断片(テンプレート)を `[templates.<名前>]` に書けるべき(全項目は CLI-12 の文書)。画面で選んだ設定(SR-43)は、同じ置き場の `ui.toml` に config.toml と同じ形で置いて config.toml の上に重ね、config.toml は書き換えないべき。設定が無くても既定で動くべきで、知らない項目・型の違う値・その場所に書けない項目は警告にとどめるべき。警告は、どの設定のファイル(config.toml・ui.toml・views.toml・workspaces.toml・ワークスペースの印・places.toml・sessions.toml)でも「ファイルの名前: 項目の道筋(`look.style.status` など): 理由」の形にそろえるべき。」(確かめ方: 今の確かめ方に加えて、Windows で `XDG_CONFIG_HOME` が無く `APPDATA` が `C:\Users\a\AppData\Roaming` → `C:\Users\a\AppData\Roaming\mdgrid\config.toml`。`XDG_CONFIG_HOME` が絶対パスならそちら、相対パスは無視して `APPDATA`。どちらも無い・空 → 置き場なし(既定で動く)(`test_cli_3_windows_*`))
- 変更: SR-8 旧「選んだ行のノートをエディタで開けるべきで、戻ったらそのノートを読み直すべき。エディタは、設定(CLI-3 の設定ファイル)の `editor`、環境変数 `$VISUAL`、`$EDITOR` の順で空でない最初のものとし、どれも無ければ `vi` とするべき。値が引数つき(`code -w` など)でも動き、シェルを通さずに起動し、ノートのパスは1つの引数として渡すべき。」→ 新「選んだ行のノートをエディタで開けるべきで、戻ったらそのノートを読み直すべき。エディタは、設定(CLI-3 の設定ファイル)の `editor`、環境変数 `$VISUAL`、`$EDITOR` の順で空でない最初のものとし、どれも無ければ `vi`(Windows では `notepad`。vi が入っていないのがふつうのため)とするべき。値が引数つき(`code -w` など)でも動き、シェルを通さずに起動し、ノートのパスは1つの引数として渡すべき。」(確かめ方: `EDITOR="code -w"` で開く → 戻ると行が更新。設定に `editor = "nvim"` と `EDITOR=vim` → nvim で開く。設定が無く `VISUAL=hx`・`EDITOR=vim` → hx。どれも無い → Unix では vi、Windows では notepad(OS ごとの既定は文字の値 `"vi"`・`"notepad"` で比べる)。設定の `editor = ""` は無いのと同じ(`test_SR_8`・`test_sr_8_default_editor_per_os`))
- 錠: SR-8(src/test_config_unit.rs::test_sr_8_blank_is_same_as_empty と src/ui/test_external.rs::test_sr_8_editor_argv_split。どれも無いときの `vi` を、OS ごとの既定の名前と比べる形にする。OS ごとの既定が `"vi"`・`"notepad"` であることは、新しい試験 `test_sr_8_default_editor_per_os` が文字の値で確かめるので、確かめは弱まらない)
- 錠: SR-19(src/ui/test_click_name.rs::test_sr_19_empty_editor_click_same_as_e。同じ理由)

要件の文を変えない関わり: OUT-1 は「OS のクリップボード」と OS を限らずに書いてあるので、Windows の `clip` を足しても文は変えない(試験 `test_out_1_windows_*` は OUT-1 を指す)。見た目の状態の置き場(Windows は `%LOCALAPPDATA%\mdgrid\`)は要件の文に無く(screen の前提に「OS の状態用のフォルダを想定」とある)、`~` のホームを Windows で `USERPROFILE` から取るのも、どの要件の文も OS を限っていないので、docs/design.md に書く。notepad が閉じるまで待たない場合(Windows 11 のタブのメモ帳がすでに開いているとき)は、待つエディタを `editor` に置くよう docs/config.md に書き、docs/todo.md に確かめを残す。

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(Windows は対象外のまま) | 人が対象に入れると決めた |
| Windows でも `HOME` か `~/.config` を見る | Windows には `HOME` がふつう無く、設定の置き場は `%APPDATA%` が決まり。`XDG_CONFIG_HOME` を置けば今までどおりそちらを使える |
| 依存の crate(dirs)で置き場を決める | 環境変数を1つ読むだけで足り、依存と cargo deny の確かめを増やさない |
| Windows でも既定のエディタを `vi` のまま | Windows には vi がふつう無く、`e` が「エディタが無い」で止まる。notepad はどの Windows にもある |

## 錨との照合

新しい文脈の検証役(書込なし、2026-10-15)の照合の要約。差分の旧は CLI-3・SR-8 の今の文と一字ずつ一致した。`specs/principles.md` は無い。

- 沿う: 人が Windows を対象に入れると決めたこと(会話 2026-10-14、docs/todo.md の O-11)に沿う。2026-10-09-windows-test-locks(人)の、Windows で落ちる試験を残さず確かめる中身は変えない向きにも沿う。2026-10-02-editor-setting(人)の順「設定 > $VISUAL > $EDITOR > 既定」は残り、最後の既定だけが変わる。2026-10-10-config-v2(人)の CLI-3 の項目の形には触れない。
- ぶつかり: 無い。`%APPDATA%` と `notepad` は AI が足した細部(きっかけに書いた)。v1 は macOS でも `~/.config` に寄せたが、Windows でも `XDG_CONFIG_HOME` を先に見るので食い違わない。
- 抜け(指摘を受けて直した): 錠の掛け直しで `"vi"` を定数と比べると確かめが弱まる → 新しい試験が `"vi"`・`"notepad"` を文字の値で確かめると差分の錠の行と確かめ方に書いた。SR-8 の確かめ方の「どれも無い → vi」を OS ごとに書き直した。notepad が待たない場合の扱いを書いた。OUT-1・状態の置き場・`~` の扱いを文を変えない関わりとして書いた。相対の `XDG_CONFIG_HOME` を確かめ方に書いた。
- 判定: 向きに沿う

## 承認の記録

2026-10-15 AI 自己採択(錨との照合あり)。人の言葉はプロジェクトのチャットで、会話の記録で照合できる形で残っていないため、人の決定にはしない。
