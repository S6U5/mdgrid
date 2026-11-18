# trace-gate への移行で詰まった所

2026-11-17 に mdgrid の仕様駆動を decidespec から trace-gate 0.10.2 へ移したときの記録。道具(trace-gate)を直す材料にするため、使いにくさ・誤検出・足りない機能・文書の分かりにくさを1件1行で書く。上ほど移行への響きが大きい。数は移行の時点のもの。

移行の規模: 要求 271 件(能力 16)、テスト 2,377 本(うち印を付けた 2,227 本・印を付けられなかった 111 本)、試験のファイル 361(src の中 250・tests 111。テストは src の中 1,556 本・tests 821 本)、旧い印の付け替え 2,130 か所・印の行の足し 214 か所。

## 上位 10 件

1. [誤検出] Rust の断言の読みがマクロ(`assert!`・`assert_eq!` など)だけで、`assert_same(…)` のような `assert_` で始まるヘルパーの関数を数えない。README の限界の節は「ヘルパーの名前が assert・expect で始まれば数える」と書くが、Rust の正規表現は `!` を要る。断言をヘルパーに任せた 111 本(`assert_same`・`assert_gone`・`assert_only_value_changed`・`report`・`t`・`f` など)に印を付けると test-empty で止まるので、印を付けられなかった(`tbd~rust-helper-asserts`)。
2. [足りない機能] 断言のヘルパーの名前を設定で足す口が無い(trace-gate.json の鍵は固定)。リポごとの `assert_helpers` のような鍵があれば、1 の 111 本はテストを変えずに印を付けられた。
3. [足りない機能] ほかの仕様駆動の道具からの移し方(import)が無い。旧い印 `[SR-38]`・テストの名前の ID `test_wb_20_*`・ファイルの頭の `//! [CLI-17]`(ファイルの全部のテストに効く旧い決まり)を新しい印に写すのに、検査器の字句の読み(scan_brace_lang・bind)を借りた使い捨てのスクリプトを約 300 行書いた。対応表(renames.md)を渡すと印を付け替える `import --map` か、少なくとも手順の文書が欲しい。
4. [足りない機能] Rust の既定の試験の置き場が `tests/**/*.rs` だけで、Rust の流儀の src の中の単体の試験(`src/**/test_*.rs`・`#[cfg(test)] mod tests`)を拾わない。mdgrid では試験のファイルの 7 割(250 / 361)、テストの 65%(1,556 / 2,377)が src の中にあり、設定を直すまで半分以上が読まれない。
5. [使いにくさ] init が設定に関わらず `specs/requirements.md` の雛形を置き、終わりの案内も「次: specs/requirements.md に要求を書き」で固定。operations.md は 5 万字を超えたら `specs/requirements/<段>.md` に割れと言うが、init に割った置き場を選ぶ口が無い(移行では最初から 21 万字)。雛形は rm が使えない環境では消しにくく、`specs/requirements/README.md` へ動かして地図にした。
6. [使いにくさ] init は `.claude/settings.json` が既にあると Stop のフックを置かず、「人が手で足し、init --relock」と返す。移行するリポにはほぼ必ず前の道具の settings.json があるので、先に前のフックごとファイルを消してから init する順にしないと、フックが入らない。設定のほかの鍵を残したまま hooks.Stop に足す口が欲しい。
7. [文書の分かりにくさ] templates/AGENTS-snippet.md が正本を `specs/requirements.md` と決め打ちし、「合否は CI が決める」と書くが、既定の部品は ci が false(CI の雛形を置かない)。既定どおり入れたリポで、写した指示が実態と食い違う。AGENTS.md には写したうえで「このリポは CI に載せていない」を足した。
8. [文書の分かりにくさ] 1本のテストが指す要求の数の決まりがあいまい。format.md は「1つにし、結合のテストだけ itest で並べる」と言うが、検査器は utest で並べても止めず、数(tests_multi_target)を出すだけ。移したテストの 517 本が 2〜7 の要求を指していて、itest に替えるか、utest のまま並べるか、1つに絞るか(絞ると P0 の確かめが減る)を決める材料が文書に無い。今は種類はそのまま、要求を並べた。
9. [足りない機能] P0 は「1行に受け入れの基準を1つ」だが、検査器は見ない。移した P0 の 23 件は旧い要件の1行のまま、受け入れに例を複数持つ(write-back.byte-range は 5 つ)。行を基準ごとに割るには、旧い印のテスト(write-back.add-key-and-frontmatter は 58 本)を基準ごとに読み分けて付け直す要り、移行の手順に入れられなかった(`tbd~p0-one-criterion`)。
10. [誤検出] weaken が、init で初めて写した検査器 `tools/trace_gate.py` の中の `# noqa: E731` を suppress-added として人に回した。道具そのものの写しの足しは、写しの錠に載る物として外すのがよい。

## ほか

- [誤検出] weaken の instructions-changed は `.claude/` と AGENTS.md・CLAUDE.md だけを見て、Codex の置き場(`.agents/skills/`・`.codex/hooks.json`)の消しを人に回さなかった。前の道具の門番(Codex の SessionStart・Stop のフック)を外したことが見えない。
- [誤検出] weaken は `ci.sh` から前の道具の検査の行(`python3 "$DECIDESPEC_CHECK" .`)を消したことを回さない。gate-loosened は CI のワークフローだけを見て、共有の設定の ci.sh はテストの語のある行しか見ないため。検査を走らせる行の消しは、ワークフローでなくても回したい。
- [文書の分かりにくさ] 「AI は錠を書かない」と「init が錠を書く」の関係が分かりにくい。初めての init を AI が走らせると写しの錠ができる。移行では依頼に従って AI が走らせたので、錠の中身(tools/trace_gate.py と .claude/settings.json のハッシュ)は人が差分で確かめる。
- [文書の分かりにくさ] テストの種類(utest・itest・stest)と Rust の言葉がぶつかる。Rust の tests/ の試験は「結合の試験(integration tests)」と呼ばれるが、trace-gate の itest(要求を並べる結合)とは別。本物の実行ファイルを動かすファイル(`CARGO_BIN_EXE_mdgrid` か `mod pty;`)を stest、ほかを utest にしたが、手引きが無い。
- [誤検出] `扱い: 廃止(理由)` の理由に括弧を入れると、「扱い: は … 受け入れ基準の終わりに1つだけ書く」で止まる。誤りの文が括弧の入れ子のせいだと分からない。
- [足りない機能] f2p(実装前に落ちるか)は、移した要求を「base に無い P0」と数える。移行の 23 件の P0 は振る舞いが既にあるので実装前に落ちようがなく、外すには人の例外が要る(fail-to-pass.md の 34 行目)。`扱い: 踏襲` のように「既にある振る舞いを要求にした」印で外せるとよい。
- [足りない機能] Rust の `cargo test` は JUnit XML を出さないので、`check --junit`・`f2p`・`results` を使うには cargo-nextest を入れる要る(依存が増える)。rust.json の test の役は tool を nextest にしているが command が tbd のまま。
- [使いにくさ] 設定の requirements を `specs/requirements/*.md` にすると、同じ置き場の地図(README.md)と対応表(renames.md)も要求の正本として読まれ、台帳の inputs に並ぶ。operations.md は地図を同じ置き場に置けと言うので、requirements に exclude が欲しい。
- [使いにくさ] 未確定事項の `期限:` は日付しか読まず、「期限: main に取り込む前」のような文を黙って期限なしと扱う。読めない期限は警告してほしい。
- [足りない機能] 要求の名前を変える命令が無い。名前を変えると、テストの印・ほかの要求の文の中の参照・対応表を手で直す。移行で 271 の名前を付けたあと、人の見直しで名前が変わると同じ手間がまた要る(移行の途中で workspace.detect を workspace.detect-roots に改めたときも、正本・対応表・印を手で直した)。
- [足りない機能] 文の中の旧い ID(括弧の無い `SR-23` などの参照)を名前に替える道具も無い。印は付け替えたが、src・tests のコメントと文字列に約 4,300 か所、docs などに約 570 か所が残り、renames.md で引くしかない。
- [使いにくさ] 名前の形(英小文字とドット)が、このリポの設定の鍵の道筋(`workspace.detect` など)と同じ形で、文の中で見分けにくい。WS-5 を workspace.detect と名付けたら、設定の鍵 `workspace.detect` と同じ字になった。名前の段を設定の鍵と分ける手引きか、文の中で要求を指す書き方の決まりが欲しい。
- [文書の分かりにくさ] 「段の名前は人が決める(AI が付ける名前は崩れやすい)」とあるが、移行では段(能力)だけでなく 271 の後ろの名前も AI が付けるしかない。段は decidespec の能力のフォルダの名前をそのまま使い、後ろの名前は AI が付けた(人の見直しが要る)。名前を人が見直す手順が無い。
- [使いにくさ] 1行1件の要求は、旧い要件を写すと1行が長い(平均 約 330 字、長いものは 1,700 字)。21 万字の正本の差分は行ごとにしか読めず、人のレビューが重い。長い旧い要件を割るか、そのまま写すかの手引きが欲しい。
- [足りない機能] intents の Rust の意図の行は最初の `///` だけを読む。このリポは本体の中の `// [utest -> …] 意図の文` に意図を書く流儀で、印のコメントは意図から外されるので、intents に意図が出ない。印と同じ行の後ろの文を意図として読むとよい。
- [足りない機能] Codex 向けのフックの雛形が無い。decidespec は `.codex/hooks.json`(SessionStart・Stop)を置いていたが、trace-gate は Claude Code の Stop だけなので、Codex のセッションでは終わりの check が走らない。
- [文書の分かりにくさ] 移行(ほかの仕様駆動の形から移す)の手引きが無い。references/replace.md はシステムのリプレース(現新比較)の話で、仕様の形の移し替え(旧い ID の対応表・凍結・門番の外し・旧い印の扱い)には当たらない。
- [使いにくさ] 手元の `.git/hooks/pre-commit` は前の道具のもの(decidespec の flow_guard)のまま残る。trace-gate の pre-commit は手で写す決まりで、init は触らない。枝で移すと、main に取り込むまで main の守りを残したいので、ここで入れ替えなかった(取り込んだあとに人が入れ替える)。

## 進行中だった変更の記録の整理

凍結の時点で status が active だった変更の記録 12 件を、記録は書き換えずに次のように分けた。「終わった」は、実装とテストがあり、照合の記録だけが残っていたもの(trace-gate では、要求の印のテストと check がその確かめに当たる)。

| 記録(specs/_history/decidespec/_changes/) | 分け | 行き先 |
|---|---|---|
| 2026-10-01-relations | やめた | 要求だけで止まった。ノートのあいだのリンクは、別の記録で relations の要求(relations.link-forms・relations.link-input など。テストあり)として入った。外へ張るときの警告は要求に入らなかった |
| 2026-10-01-sessions | 新しい要求に移す | cli.session-save・cli.session-content・cli.session-store・cli.session-screen・cli.session-errors を P1 のまま残し、作るか外すかを `tbd~cli.sessions` に置いた(実装もテストも無い) |
| 2026-10-09-modern-borders | 終わった | screen.frames(テスト 5 本) |
| 2026-10-09-modern-look | 終わった | screen.modern-look(テスト 10 本) |
| 2026-10-09-relation-map | 終わった | relations.map・relations.screen-menu・relations.layout(テスト 13・9・3 本) |
| 2026-10-09-workspaces | 終わった | workspace.app-side から workspace.marker まで、relations.backlinks・relations.table-links(どれもテストあり) |
| 2026-10-15-marker-base | 終わった | workspace.marker-base・workspace.marker-where(テスト 10・10 本) |
| 2026-10-15-marker-open | 終わった | cli.launch・workspace.marker(テスト 13・10 本) |
| 2026-10-15-note-path | 終わった | screen.display-config・screen.note-column・screen.note-path・screen.note-name(どれもテストあり) |
| 2026-10-15-tab-wrap | 終わった | navigation.tab-wrap(テスト 11 本)。実装は済で、照合だけが未だった |
| 2026-10-15-views-skill | 終わった | cli.views-skill(テスト 3 本)。実装は済で、照合だけが未だった |
| 2026-10-15-windows-target | 新しい要求に移す | 照合 4 / 7 で止まっていた残り(Windows のハードリンクと、Windows の CI での write-back.atomic-replace・write-back.keep-on-stop)を `tbd~write-back.windows` に置いた |

## 2026-11-18: 最新の道具(trace-gate 0.32.0・forge-gate 0.6)での詰まりの照らし

上の詰まり 10 件と、あとで見つけた 11〜14 を、写しを 0.32.0 に替えた木で照らし直した。「実測」は手元で走らせて確かめたもの、「読み」は道具のソースと手引きを読んだだけのもの。

| # | 結果 | 確かめたこと |
|---|---|---|
| 1 Rust のヘルパーの断言 | 直った | 実測。assert・expect で始まる関数の呼び出しを断言と数える。名前だけだった 58 本に印を付けて check が通った |
| 2 ヘルパーの名前の設定 | 直った | 実測。trace-gate.json の `assert_helpers`(Rust)に check・report・input_err など 11 の名前を足し、43 本に印を付けて通った。値の断言が無い 3 本は印を付けず残した(`tbd~rust-helper-asserts`) |
| 3 import | 未 | 読み。`--help` の命令に import が無い。この移行は済んでいるので響かない |
| 4 src の中の単体の試験 | 未 | 読み。既定の tests は `**/tests/**/*.rs` だけ。設定で `src/**/test_*.rs` を足して回避している |
| 5 init の `specs/requirements.md` | 直った | 読み。init が置く種は trace-gate.json だけ(`init --force` は人だけが走らせるので未実行) |
| 6 既にある settings.json | 直った | 読み。既にあれば雛形との差の行を出し、更新の指定があるときだけ trace-gate のフックの項目を替える(未実行) |
| 7 AGENTS-snippet | 直った | 0.24.0 の時に読んで確かめた |
| 8 1本が指す要求の数 | 直った | 実測。check の要約に「複数の要求を指すテスト 1,156 本(それだけで確かめた P0 25)」と出る |
| 9 P0 の基準は1つ | 未 | 実測。受け入れに基準が 4 つの P0(write-back.conflict-stop)が check を通った。分ける前の P0 は人が見て 1 基準にそろえた |
| 10 写した検査器の抑止 | 直った | 実測。`weaken --base 4c79a0c --head 0e64220` は、写しを錠に載った物として行の点検から外し、suppress-added を出さず、test-config-changed で人に回した |
| 11 `check --junit` の Rust の名前 | 一部 | 実測。nextest の JUnit(2,374 本が通過)で照らすと、印のあるテスト 2,328 本のうち 322 本が test-not-run。tests/ の統合テストは照らせるが、src の `test_*.rs` は JUnit の名前が `source::markdown::tests::名前` のようにファイル名と別の道になる(`#[path]` で取り込んでいるため)ものが合わない |
| 12 条件つきのテスト | 直った | 実測。Windows だけのテスト 7 本は、エラーでなく「この OS では走らない」の知らせになる |
| 13 test の仕事の道具の段 | 直った | 読み。forge-gate.json の `test.setup` に段を書ける。nextest を入れる段と JUnit の設定(`.config/nextest.toml`)を置いた。GitHub 上での実行は未 |
| 14 最初の PR の f2p-run | 直った | 実測。`f2p-run --base main` は、基点に設定も要求の正本も無いと「入れる最初の PR なので数えない」と出て落ちない |

集計: 直った 10(1・2・5・6・7・8・10・12・13・14。うち 5・6・13 は読みだけ)、一部 1(11)、未 3(3・4・9)。

ここから CI を GitHub で走らせると、11 の 322 本が test の仕事を赤にするので、CI を有効にした PR の中で名前の照らし方を直すか、src のテストの置き方を揃える必要がある。
