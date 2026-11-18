---
type: Change
id: 01M3ZXEY1BN6WMYMZXR0MB878K
title: --print と --pick(print-pick)
status: done
size: full
created: 2026-10-03
updated: 2026-10-03
---

# --print と --pick(print-pick)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [print-pick](../_decisions/2026-10-03-print-pick.md)(AI) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | b775fc4(--print)・この後のコミット(--pick) |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の F-5。画面を出さずにビューの表を標準出力に出す `--print [--format csv|json|md]`(CLI-5)と、画面で選んだ行のパスか列の値を標準出力に出して終わる `--pick path|<列>`(OUT-3)を入れる。引数は clap(D-5)の上に足す。

終わりの条件: specs の CLI-5・OUT-3 の例のとおりに動く。関係する要件: CLI-5・OUT-3・SR-10・CLI-2・CLI-4・BV-7・WB-15。

## 不明点と仮定

- 仮定: JSON と CSV は依存を足さず手で書く(文字列の逃がしと RFC 4180 の括りだけ。serde_json を入れるほどではない)。
- 仮定: `--view` は、`--print` のときは `.base` のビューを先に、無ければ mdgrid のビュー(views.toml の今の対象の分)を名前で探す。画面の起動の `--view` は今のまま(`.base` のビューだけ)。外れたら: 画面の起動も mdgrid のビューを探す(別の変更)。
- 仮定: csv・md・`--pick <列>` の文字は、画面の印(`*`・`#`・`!`・`?`・`∅`)を付けない素の文字(null と空は空、リストは `, ` でつなぐ、日付は設定の date_format でなく YYYY-MM-DD。改行は md と `--pick <列>` では空白、csv では RFC 4180 の括りの中に残す)。json は型を保つ。
- 仮定: `--pick` の Esc は、印か範囲があれば今どおり解き、無ければ取りやめ(終了コード 1)。q も取りやめ。Enter は見出しの行では今どおり開閉。
- 仮定: `--pick path` は、ノートの実体のパスが起動の引数のフォルダ(`.base` ならそのフォルダ)を正規化した場所の下なら「引数の文字 + 相対」、外なら絶対パス。

## 設計

- 表の組み立て(lib): 新しい `src/print.rs`(lib)に「ビューの表を作る」関数を置く。`.base` のビューは `Base::build` と `Base::cell`、mdgrid のビューは src/ui/native_views.rs の `synth_of` と同じ組み立て(`views::to_base` → `base::parse` → `build(0)`)に `settings::apply` を重ね、列は `nv.order` から `nv.hidden` を除く(空なら `src.columns()`)。既定の表は `base::default_grid`。今 src/ui にある純粋な関数(`today_now`・`val_value`・`kind_in`・`formula_kind`・`val_text`)のうち要るものは lib に移し、src/ui からはそれを呼ぶ(重ねて持たない)。
- 書き出し(lib の同じ所): csv・json・md の3つの形の関数。BV-7 の列は空・null と、列ごとの1行の警告を返す。
- 起動(src/main.rs): `Cli` に `--print`・`--format`・`--pick` を足し、`Command` の新しい腕(`Print`・`Pick`)か別の構造体で返す(錠の試験の `Options` は4つの欄のまま)。`--print` は check_tty の前に分け、ノートを全部読んで(load を終わりまで)出す。`--print` と `--pick` は同時に受けない(clap の conflicts_with)。知らない形は clap の値の検査で理由1行・終了コード 2。
- `--pick`(src/ui と main): 画面の入出力を `/dev/tty` に向ける(`run`・`event_loop`・`enter`・`restore` が書き先を受ける)。App に選びの状態(名前は `pick` 以外。`pick` はリストの選択で使用中)を持たせ、表の `Action::Edit` の手前で、選びのときは結果を作って終わる。読むだけ(WB-15)で開く。結果は終了の後に標準出力へ。終了コードは main が App の結果で決める。
- 却下: App を画面なしで作って `--print` に使う(overlay・stay・隠す列の都合が混ざり、App は bin の側にある)。
- 検証: `cargo test`、`./ci.sh`。`--pick` の受け入れは疑似端末が要るので、Q-10(e2e)の道具の上で書く。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | `--print`(lib の表の組み立てと3つの形、起動) | CLI-5, SR-10 | CLI-5 の例のとおりに出る | src/print.rs, src/lib.rs, src/main.rs, tests/test_print.rs, src/ui/grid.rs, src/ui/cell.rs, src/ui/native_views.rs, src/ui/app.rs, README.md, docs/design.md | test_cli_5_*(tests/test_print.rs) | 済 |
| 2 | `--pick`(/dev/tty の画面と選び) | OUT-3, SR-10 | OUT-3 の例のとおりに出て終わる | src/main.rs, src/ui/app.rs, src/ui/nav.rs, src/ui/startup.rs, src/ui/external.rs, src/print.rs, docs/keys.md, docs/keys.ja.md, tests/test_pick.rs, tests/pty/mod.rs, README.md, docs/design.md | test_out_3_*(疑似端末。Q-10 の道具の上) | 済 |

順: 1 → (Q-10) → 2。

## 実装の気づき

- タスク1の試験(試験を書く役): 列の名前は displayName の無い列の id(ノートのキーは素の名前、式の列は `formula.名前`)、csv・md の真偽は `true` / `false`、キーの無いセルは json で null、と仮定した。書き換えの1回を Edit でなくスクリプトで行った(tests/test_print.rs だけ。本人の報告)。
- タスク1の実装(→ タスク2): `--pick <列>` の素の文字は `print::PrintCell` と同じ規則(印なし・リストは `, `・日付は YYYY-MM-DD)なので、`src/print.rs` の `plain` を pub にして App の `cell`(ためた値は `ui::grid::value_of` で重ねる)から使うと二重に持たずに済む。`Cli::rest()` は命令の旗を `parse_args` に渡し直すので、`--pick` もそこに足す(`--print` と `--format` は足した)。
- タスク2の実装(→ 照合): crossterm の入力と raw は標準入力が端末ならそれ、でなければ `/dev/tty` で読む(確かめた)が、`cursor::position` は標準出力に `ESC[6n` を書くので、ratatui の `Terminal::clear` は `/dev/tty` の画面では使えない(問い合わせない `Terminal::resize` で消す。src/main.rs の `clear_screen`)。`Startup` は試験が欄を並べて作るので欄を足さず、選びは `App::start_choosing` で当てた。

## 照合

タスク1(`--print`)は書込なしのフレッシュ文脈の照合役で照合した(2026-10-03、コミット済みの clone で。実装のコミットは b775fc4)。タスク2(`--pick`)も別の照合役で照合した(実装のコミットは ca74d1e)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-5 | checked | コード: src/print.rs の table・render・csv_field・md_cell、src/main.rs の Command::Print・print_view / テスト: tests/test_print.rs::test_cli_5_*(11本) / 今: 通った(11 passed、cargo test 全体 906 passed) / 前: 落ちた(b775fc4^ で11本が「知らないオプション: --print」で落ちた)。変異: csv の括りをやめる・json の数を文字列にする・md の縦棒を逃がさない・base の notes の警告を消す → それぞれ落ちた |
| SR-10 | checked | コード: src/main.rs の main で Command::Print を check_tty より前に分ける、emit の BrokenPipe / テスト: tests/test_print.rs::test_cli_5_*(Command::output で標準出力はパイプ) / 今: 通った / 前: 落ちた(b775fc4^ で11本)。変異: --print の前に check_tty を呼ぶ → 10本が落ちた。--pick の分: コード: src/main.rs で --pick のとき check_tty の代わりに check_dev_tty だけを見る / テスト: tests/test_pick.rs::test_out_3_pick_works_when_stdout_is_pipe(ほかの12本も標準出力はファイル) / 今: 通った(2回) / 前: 落ちた(ca74d1e^ で「知らないオプション: --pick」)。変異: --pick でも標準出力が端末でないと止める → 13本が落ちた |
| OUT-3 | checked | コード: src/main.rs の Command::Pick・pick_path・Screen::Tty、src/ui/nav.rs の choose_action・start_choosing・find_column・pick_text、src/ui/external.rs の TtyClipboard / テスト: tests/test_pick.rs::test_out_3_*(13本) / 今: 通った(2回とも 13 passed) / 前: 落ちた(ca74d1e^ で13本が「知らないオプション: --pick」)。変異: 印を付けた順に出す・path を常に絶対で出す・q で終了コード 0・読むだけにしない・列の値に画面の印を付ける → それぞれ落ちた |

既存のテストの削除・skip・弱体化: なし(e579658^..b775fc4 の試験の変更は新しいファイルだけ)

残した気づき: 選んでいる間のエディタで開くの止めと OSC 52 を /dev/tty に書く道は、コードを読んで判定した(試験なし)。選んでいる間に別のビューへ移り、指定の列が無くなると空の行になる。評価できない列の警告に列の名前が2回出る(`formula.謎: formula.謎: …`。src/base.rs の notes の作り方)。1行は守られている。

確かめた: 3 / 3
