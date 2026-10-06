# mdgrid の TODO(OSS の公開と、改善・開発のループ)

人がセッションを1つずつ割り当てて進めるための作業の一覧。1行が1セッションの単位(大きいものは → で分けた)。
洗い出し: 2026-10-02(調べる役のサブエージェント。根拠は末尾)。終わったら `[x]` にし、変更の記録(specs/_changes/)のパスを添える。

印: 優先度 P0 = 公開に必須 / P1 = 公開前にあるとよい / P2 = 公開後。大きさ S / M / L。**人** = 人の判断が要る。

## 0. 先に知っておくこと(前提を変える事実)

1. **人の承認の照合は手元の会話の記録に頼る**(decidespec の `approval-evidence: claude:…`)。GitHub Actions の上では照合できないので、「Issue → AI が実装」を全自動にすると、守られる要件の人の承認を CI で確かめられない。GitHub のコメントを証拠にするには decidespec の OM-12 を変える(OM-13 で要件 ID ごとの事前の承認が要る)→ H-7。
2. **decidespec と cellops-creator にリモートが無い**。公開した mdgrid の CI から check.py を取れない(先に公開する / 版を固定して写す / CI では回さない)。
3. **画面の文言が全部日本語**(help・USAGE・理由・下の帯。golden も依存)。README を英語が主にするのと食い違う → O-3。
4. **試験の錠が0本**(`specs/test-locks.json` が `{}`、W13 が59件)。AI に実装させる前に錠を掛けないと、守られる要件のテストを弱めても検出できない → H-1。
5. **コミットの書き方が release-plz と合わない**(`実装:` など。Conventional Commits でない)→ O-7。
6. **sessions の設計(clap の conflicts_with)と実装(自前の引数の読み取り。clap なし)が食い違う** → D-5。
7. **公開前に消す私的な情報**: `specs/_changes/` の一部にラボのパス、決定の記録に会話の ID。古い記述(write-back の範囲外の「次の段: CE-13」)。
8. **名前は空いている**(2026-10-02): crates.io の `mdgrid` なし、Homebrew core なし、GitHub は他人の小さいリポが1件。
9. **MSRV は 1.90**(saphyr の ordered-float 5.5。2026-10-06 に確かめた。`rust-version` に書いた)。

## 1. ハーネス(改善と開発のループ)の全体像

```
[入口]            [仕分け]                [仕様化]                    [人の門1]
Issue(テンプレ) → triage(AI・読むだけ) → decidespec の提案を起票 → 人が承認(守られる要件・新しい要件)
 bug/feature/      ラベルの案・重複・       (spec-change。PR「spec:」)   → specs: 採択 + approval-evidence
 harness           要件 ID の当たり          status:spec-proposed          status:ready(人だけが付ける)
                   status:triage
                                                                              │
[実装] 人が1件ずつセッションに割り当てる                                         ┘
  sdd-flow: 変更の記録 → 設計 → タスク → テストの先置き(別の役・錠)→ 実装 → レビュー(別の役)→ 照合 → PR
[機械の門] CI: fmt / clippy / test(3 OS)/ MSRV / cargo-deny / check.py / golden の成果物 / 秘密の走査
[人の門2]  PR のレビュー → マージ(AI は承認もマージもしない)
[人の門3]  リリース PR のマージ → タグ → バイナリ・Homebrew
[人の門0]  公開そのもの(履歴の無い新しいリポに書き出す。人の指示で)

[ハーネス自身の見直し]
  月1回: 指標を集める → 「ハーネスの振り返り YYYY-MM」Issue → 人と AI で振り返る
  → 変えるなら dev-loop の仕様への提案 → 人の承認 → ワークフローとプロンプトを直す → HARNESS の版を上げる
```

- **段階1(公開の直後から)**: Issue・ラベル・テンプレートを入口に、人が手元の Claude Code のセッションに1件ずつ割り当てる。人の承認は今どおり会話で照合。Actions は CI・仕分けの下書き・定期の検査だけ。
- **段階2**: decidespec が GitHub の証拠で承認を照合できるようになってから(H-7)、`status:ready` の Issue を Actions(claude-code-action / `claude -p`)が拾って PR を作る。
- **人の門**: 門0 公開(SC-10・ML-10)/ 門1 仕様の採択(check.py の E14・E15・E22。`status:ready` は書き込み権限のある人だけ)/ 門2 マージ(ルールセット・CODEOWNERS で `.github/`・`specs/`・`deny.toml`・`specs/test-locks.json` を人に固定)/ 門3 リリース / ハーネスの変更(門の位置は人が決める守られる要件)。
- **秘密と信頼**: API キーは GitHub の Secrets だけ。外の人の Issue は信頼しない入力(書き込み権限のある人だけが起動、`allowed_non_write_users` を使わない、仕分けの役は `gh issue view` とコメントだけ、`pull_request_target` で外の PR を checkout しない)。
- **記録の正本**はリポの `specs/_changes/`・`_decisions/`(ML-16)。Issue・PR は入口と議論の場。
- **置き分け**: mdgrid(公開)= ワークフロー・テンプレート・ラベル・CODEOWNERS・各段のプロンプト(`.claude/commands/`)・`specs/dev-loop/`・`HARNESS.md`・振り返り(`docs/harness/retros/`)・指標のスクリプト。ラボ(control/harness)= 横断の決まり(公開リポの AI の秘密と外部入力、ラベルの語彙)・公開リポの台帳・ML-10 の解除の記録。decidespec = GitHub の証拠での承認の照合・check.py の配り方。cellops-creator = 2本目のリポで使い回すときに昇格(今は作らない)。
- **見直しの仕組み**: `HARNESS.md` の `harness-version`(門・ラベル・プロンプトを変えたら上げる)、PR のテンプレートに「作ったハーネスの版」の欄、月1回の振り返りの Issue と記録、指標(Issue からマージまでの日数・人の直しなしでマージした AI の PR の割合・差し戻しの回数・CI の初回の通過率・check.py の警告数・承認待ちの日数・照合の gap 数・1件の費用・revert の数)。

## 2. 作業の一覧

### H. ハーネス

- [x] **H-1 試験の錠を掛ける** — specs/_changes/2026-10-03-test-locks.md(0fc933e、照合: 要件なしのタスク 1 / 1 checked)。Rust はファイルごとの錠(check.py の仕様)。ci.sh は `DECIDESPEC_CHECK` で check.py を回す
- [ ] **H-2 dev-loop の仕様を新設する** [P0/M] **人**(門の位置・AI に任せる範囲・段階)
  - → 提案: `specs/dev-loop/spec.md`(入口は Issue、`status:ready` は人だけ、マージは人だけ、AI の PR は変更の記録と照合を持つ、外の入力は指示にしない、ハーネスの版と月の振り返り)
  - → 採択と `HARNESS.md` v0.1.0
- [ ] **H-3 Issue・PR のテンプレート・ラベル・CODEOWNERS** [P0/S] 依存 H-2 — `ISSUE_TEMPLATE/{bug,feature,harness}.yml`・`pull_request_template.md`(要件 ID・変更の記録・テストの先置き・照合・ci.sh と check.py・ハーネスの版)・`labels.yml` と同期のワークフロー・CODEOWNERS。
- [ ] **H-4 ループの各段のプロンプト(段階1)** [P0/M] 依存 H-2・H-3
  - → `.claude/commands/triage-issue`(案をコメントするだけ)と `spec-from-issue`(提案の PR)
  - → `implement-issue`(sdd-flow・テストの先置きと実装の役を分ける・レビューと照合・PR)。AGENTS.md に「Issue からの始め方」3行
- [ ] **H-5 指標と月の振り返り** [P1/M] 依存 H-3・Q-1 **人**(指標の選び方)— `scripts/loop-metrics`、schedule のワークフロー、`docs/harness/retros/` の雛形。
- [ ] **H-6 仕分けの自動化(Actions・読むだけ)** [P1/S] 依存 H-4 **人**(API キーか OAuth か、費用の上限)— `issues: opened` で案をコメント。ラベルは人。
- [ ] **H-7 GitHub の証拠での承認の照合(decidespec 側)** [P1/L] **人**(OM-12・OM-13 の変更)
  - → 提案(`approval-evidence: gh:…` の形、書き込み権限の確かめ方)
  - → check.py の実装と試験(2セッション)
- [ ] **H-8 Actions での実装(段階2)** [P2/M] 依存 H-7・Q-1・H-1 **人** — `status:ready` + `agent:ok` で実装して PR。ランナーへの decidespec の入れ方、費用の上限、concurrency、`workflow_dispatch`。
- [ ] **H-9 ラボの側の決まり** [P1/S] 依存 H-2 **人** — control/harness に公開リポの AI の秘密と外部入力・ラベルの語彙の提案、ML-10 の解除の記録の答え。

### O. OSS の公開の準備

- [ ] **O-1 公開の判断の束を人に見せる** [P0/S] **人** — 「5. 人が決めること」を7件以内の束に。
- [ ] **O-2 README を英語(主)と日本語に分ける** [P0/M] 依存 D-1・O-5 — 1行の説明・GIF・特徴(1バイトも変えない書き戻し、`.base` 互換)・Install・Quick start・文書へのリンク・Obsidian と無関係である旨・ライセンス。
- [x] **O-3 画面の英語化(i18n)** [P0 か P1/L] **人**(対象の利用者) — 済: specs/_changes/2026-10-03-language.md(808d998・193420d・6100b5e・6ac3336、照合 1 / 1)。設定 language(auto/en/ja、既定 auto)
  - → 人の判断(2026-10-03 の束): 英語と日本語を切り替え、公開の条件にする(LANG と設定の language。既定は LANG に従う)
  - → 文言を1か所の表に集め、`LANG` / 設定の `language` で切り替える設計の提案
  - → help・USAGE・理由・下の帯の置き換え、golden を言語ごとに(2セッション)
- [ ] **O-4 Cargo.toml の metadata と crates.io** [P0/S] 依存 O-10 — 一部済(specs/_changes/2026-10-06-oss-community.md: readme・keywords・categories・include。残りは repository と publish、公開のあと)。元の項目: `publish`・`version 0.1.0`・`rust-version`・`repository`・`homepage`・`readme`・`keywords`・`categories`・`exclude`、`cargo publish --dry-run`。
- [ ] **O-5 入れ方** [P0/M] 依存 Q-1・O-4 **人**(tap の名前)
  - → `dist init`(GitHub Releases のバイナリ・インストーラ・binstall)
  - → Homebrew の tap。最低でも `cargo install --locked mdgrid` を P0
- [ ] **O-6 Nix flake** [P2/S] — crane か naersk、`nix run github:…` を README に。
- [x] **O-12 インストーラの検討** — specs/_changes/2026-10-06-release.md(まず GitHub Releases の圧縮ファイルとソースからの cargo install。tap・crates.io・署名は公開のあと、と AI が決めた)。元の項目: O-5 の前に、入れ方の選び方を1枚にまとめて決める。
  - → 候補の比較(対象の OS・利用者の手間・保守の手間・更新のしかた): shell と PowerShell の1行インストーラ(dist)・`cargo install` / `cargo binstall`・Homebrew の tap(のち core)・Nix(flake / nixpkgs)・Windows の winget と Scoop・Linux の .deb / .rpm(cargo-deb・cargo-generate-rpm)・AUR・MSI(dist)
  - → 安全と信頼: ダウンロードしたバイナリのチェックサムと署名(GitHub の artifact attestations / SLSA)、macOS の Gatekeeper(公証するか、Homebrew 経由を勧めるか)、1行インストーラの中身を読める形にする
  - → 更新のしかた: 各パッケージの仕組みに任せるか、`mdgrid --self-update`(axoupdater)を足すか。設定とビューの置き場(~/.config/mdgrid)を更新と削除で消さないこと、アンインストールの手順
  - → 決めたものを O-5・Q-6 の作業に落とし、README の Install の節の形を決める
- [ ] **O-7 CHANGELOG と版の付け方** [P0/M] **人**(コミットの書き方)— semver 0.x、release-plz(git-cliff・Trusted Publishing)、Conventional Commits に変えるか `cliff.toml` で写すか、最初の CHANGELOG。
  - → 人の判断(2026-10-03 の束): 今のコミットの書き方のまま、git-cliff(cliff.toml)で CHANGELOG の節に写す
  - → 済: cliff.toml と履歴の突き合わせの試験(specs/_changes/2026-10-03-cliff.md、0495d83)。残り: release-plz・最初の CHANGELOG の生成(git-cliff が要る。Q-6 で)
- [x] **O-8 CONTRIBUTING・CODE_OF_CONDUCT・SECURITY** — specs/_changes/2026-10-06-oss-community.md(報告先は Private vulnerability reporting、と AI が決めた)。元の項目: 仕様駆動の説明(specs/ が正本・提案から・`[XX-n]`)、ci.sh と check.py、Issue の流れ、Contributor Covenant 2.1、Private vulnerability reporting。
- [x] **O-9 スクリーンショットと GIF** — specs/_changes/2026-10-06-demo-gif.md(vhs を使わず、説明書の場面と ffmpeg で scripts/demo-gif.sh)。元の項目: vhs の `.tape` で showcase を操作。CI で作り直すか `docs/assets/` に置く。
- [ ] **O-10 公開の書き出し(門0)** [P0/M] 依存 ほぼ全 P0・O-1 **人** — 手元で check.py(E27・E28)、ラボのパスと会話の ID の扱い、`git archive` → `git init`、gitleaks、人の指示で `gh repo create` とルールセット。
- [ ] **O-11 Windows の確かめ** [P1/M] 依存 Q-1 **人**(対象に入れるか)— 名前の変更での置き換え・ハードリンク・fsync・`$EDITOR`・日本語のファイル名。直すか未対応と書く。

### D. 設定と文書

- [x] **D-1 設定の文書を仕上げる(oss-config)** — specs/_changes/2026-10-02-oss-config.md(3578aa0、照合 4 / 4)
  - → タスク1 editor の設定(test_SR_8)
  - → タスク2 項目の表・`--print-config`・docs/config.md と config.ja.md(test_config_docs が通るまで)
- [x] **D-2 キーの割り当ての一覧の文書** — specs/_changes/2026-10-03-keys-docs.md(e1334b8、照合 1 / 1)
- [x] **D-3 `.base` の対応範囲の文書** — specs/_changes/2026-10-03-bases-docs.md(0724f90、照合 1 / 1)
- [x] **D-4 書き戻しの安全の文書** — specs/_changes/2026-10-03-safety-docs.md(c89a2d9、照合 1 / 1)。F-3 は「採る」と決まり(承認の一文は待ち)、文書は今の振る舞いを書く。0444 は F-2 が入ったら直す
- [x] **D-5 引数の読み取りの方針と補完・man** — 人の判断(2026-10-03): clap を入れる。specs/_changes/2026-10-03-clap.md(cc4e26c、照合 1 / 1)。`--completions <シェル>`・`--man`(CLI-13)
- [ ] **D-6 設定の JSON Schema** [P2/S] 依存 D-1 — 項目の表から生成、taplo の補完、SchemaStore。views.toml・sessions.toml も。
- [ ] **D-7 英語の設計の概観** [P1/S] **人**(仕様を英語にするか)— `docs/architecture.md`。design.md は日本語の正本のまま。
- [ ] **D-8 FAQ** [P2/S] — OSC 52 と tmux、`NO_COLOR`、日本語入力、パイプで起動したとき。

### F. 機能

- [x] **F-1 進行中の実装を閉じる** — click-name-editor(specs/_changes/2026-10-02-click-name-editor.md、30b99cb)、empty-frontmatter-config(355c6ec、照合 3 / 3)
- [x] **F-10 新しいノートを作る(CE-25〜CE-27)** — specs/_changes/2026-10-02-new-note.md(照合 3 / 3)
- [x] **F-11 表の見せ方の切り替え(行番号・一行おきの色・列の区切り線・上の帯。SR-20・SR-21)** — specs/_changes/2026-10-02-display-options.md(照合 2 / 2)
- [ ] **U. 初めて使う人の目の点検(2026-10-06、サブエージェント)で見つかったこと** — 高い順。済んだら行に記録を添える。
  - [x] U-2 今日の日付が UTC(specs/_changes/2026-10-06-local-today.md)
  - [x] U-1(specs/_changes/2026-10-06-list-type-replaces.md。Ctrl+U は見送り)リストの候補で文字を打つと、今の値の後ろに足される(`doing` → `doingblocked`)。打った文字で置き換えて自由入力を始め、Ctrl+U で行を消す(CE-3 の提案から)
  - [x] U-4(specs/_changes/2026-10-06-new-note-infolder.md)`.base` のビューで新しいノートを作ると、`file.inFolder("Tasks")` を守らずビューから消える(CE-25 の「作った行がそのビューに残る」に反する不具合)
  - [x] U-3(specs/_changes/2026-10-06-print-with-path.md。`--with-path`。`--columns` とグループの列は残り)`--print` にどのノートの行かが無い。`path` の列・`--columns`・グループの列(CLI-5 の提案から)
  - [x] U-5(specs/_changes/2026-10-06-bulk-outside-note.md。知らせを出す。CE-10 は守られる要件なので振る舞いは変えない)印を付けた行の外で Enter → 今の行だけ直り、印の行は黙って外れる(案内を出すか、選択全部に当てる)
  - [x] U-6(specs/_changes/2026-10-06-zero-duration-empty-label.md。日で丸めるのは残り)期間が `0s`・`-7d`。日付どうしの差は日で丸め、0 は `0d`。候補の「none」は値の文字と紛れる(`(empty)` などに)
  - [x] U-7(specs/_changes/2026-10-06-mixed-datetime.md。日付と日時が混ざる列は日時。日時の列の型とカレンダーの時刻は前から在った)日時の値が型の不一致になる(日時の列の型と、時刻つきのカレンダー)[大]
  - [x] U-8(specs/_changes/2026-10-06-help-markers.md)セルの印(`#` `!` `*` `~` `∅` `+`)の凡例がヘルプに無い。`!` が「型の不一致」と「外で変わった」の2つの意味
  - [ ] U-9(理由に行と誤りの文: specs/_changes/2026-10-06-yaml-error-detail.md。残り: --print で標準エラーに知らせる)壊れた YAML の理由が曖昧(何行目のどんな誤りか)。`--print` で黙って空の行
  - [x] U-10(specs/_changes/2026-10-06-english-wording.md)上の欄の名前が「Search」なのに中身は絞り込み。`/` で一致が無いときの知らせが無い
  - [ ] U-11(`1 files` の文法は english-wording で済。残り: 狭い端末の帯の順・--pick の案内・衝突の無いときの o)下の帯: 狭い端末で `? Help`・`q Quit` が切れる、`--pick` で Space と Enter の案内が無い、`1 files` の文法、衝突が無いのに `o` を出す
  - [x] U-12(specs/_changes/2026-10-06-open-md-file.md)`.md` のファイルを1つ渡すと「フォルダでない」で終わる(親のフォルダで開いてその行を選ぶ)
  - [x] U-13(値ごとの件数: specs/_changes/2026-10-06-frequency-table.md。組み込みの集計: specs/_changes/2026-10-06-summaries.md。残り: 式の集計とまとまりごとの集計)列の集計(数・合計・平均・最早・最遅)や値ごとの件数が無い(Bases の summaries)[大]
  - [ ] U-14(本文のちょい見は済: specs/_changes/2026-10-06-detail-body.md。残り: リンクをたどる)本文のちょい見(読むだけ)とリンクをたどる [大]
  - [ ] U-15 README: 公開したら `cargo install --git …` の1行、最初の版 v0.1.0、印の凡例、フロントマターの無いノートに足すこと、table のビューだけ、他の道具との比べ
- [ ] **O. Obsidian を使う人の目の点検(2026-10-06、サブエージェント)で見つかったこと** — 高い順。
  - [x] O-1 file.mtime・ctime が UTC(specs/_changes/2026-10-06-kind-cache-file-times.md)
  - [x] O-8 フォルダの表が保管庫の大きさの2乗で遅い(同上。20,000 ノートで 20.5 秒 → 2.1 秒)
  - [x] O-2(specs/_changes/2026-10-06-number-fn.md。錠のある試験が「期間 ÷ 数は期間」を確かめるので、number() を足し、1秒未満を ms で見せる)`(due - today()) / 86400000` が期間のまま(期間 ÷ 数は数に。`.days`・`number()` も)
  - [x] O-3(specs/_changes/2026-10-06-scalar-list-expr.md)types.json でリストの型(tags・aliases・multitext)の列の、1つの値で書かれたものを式では1つの要素のリストに(`tags.contains("proj")` が部分の一致になる)
  - [ ] O-4 `--print` で評価できない列が空のセル(印か `--strict`)
  - [ ] O-5(file.links と .base を開いたときの this は済: specs/_changes/2026-10-06-links-this.md。**人の判断待ち**: file.backlinks・file.hasLink() は索引の計算まで実装済みで、src/expr.rs の FILE_FIELDS に名前を戻せば使えるが、錠のある試験(BV-3・BV-7・CE-22・SR-23・WB-3・WB-5・CE-16 の錠)と見本の タスク.base が未対応の例に使っているので、掛け直しに人の決定が要る。link() は未対応)file.links・file.backlinks・this・link() [大]
  - [x] O-6(specs/_changes/2026-10-06-common-methods.md。relative() は未対応のまま)よく使うメソッド: .format()・.year/.month/.day・.relative()・.days・number()・.join()・.trim()・.upper()・.round()
  - [ ] O-7 .md でないファイル(添付)を表に出さない理由の知らせ
  - [x] O-9(specs/_changes/2026-10-06-poll-round-robin.md。2,000 を超える保管庫は1回に 2,000 ずつ順に見る)1秒ごとの見回りの CPU が保管庫の大きさで増える(FSEvents・inotify)
  - [x] O-10(specs/_changes/2026-10-06-checkbox-two-state.md。人の承認) チェックボックスの Enter が false → 空(CE-4 は人の決定。Obsidian の手触りと違う)
  - [x] O-11(specs/_changes/2026-10-07-scalar-list-edit.md。人の承認)1つの値で書かれたリストを、直すときにリストに書き換える(今は読むだけ)
  - [x] O-12(specs/_changes/2026-10-07-default-headings.md。人の承認)列の見出しが `formula.x`・`file.name` のまま(Obsidian の既定の見出しに)
  - [x] O-13(入力を消す Ctrl+U は済: specs/_changes/2026-10-06-clear-input.md。秒は落ちていなかった: specs/_changes/2026-10-06-calendar-seconds.md。--print の groupBy の列は CLI-5 のとおり列はビューの order で、まとまりの列は order に足せば出る。--with-path はフォルダと .base で同じ形)入力の行を一度に消すキー、--print の groupBy の列、--with-path の形のそろえ、日時のカレンダーが秒を落とす
- [ ] **A. 3回目の点検(2026-10-06、サブエージェント。書き戻しは1バイトも変わらないことを確かめ直した)で見つかったこと** — 高い順。
  - [x] A-3・A-4(specs/_changes/2026-10-06-type-replaces-date-number.md)日付と数の入力でも、打った最初の文字で今の値を置き換える(`+3` が `2026-10-05+3`、`5` が `15` になる)
  - [x] A-1(specs/_changes/2026-10-06-trust-notices.md)64ビットに入らない整数が `{…}` になり、--print で null(元の文字のまま見せて `!`)
  - [x] A-2(specs/_changes/2026-10-06-trust-notices.md)未対応の集計の知らせが画面でほぼ見えない(開いたときに一度出す)
  - [ ] A-5 40×12 で操作の一覧と頻度表が開かない(表の上に重ねる)。知らせの文が切れる(後回し: 40×12 はまれで、錠のある 80×8 で開かない試験と境目の規則を合わせる要がある。SR-24 の「入らない」の決め方を変える提案から)
  - [x] A-6(specs/_changes/2026-10-06-theme-meaning-colors.md)明るいテーマで差分の緑が読みにくい(テーマごとの意味の色)
  - [x] A-7(specs/_changes/2026-10-06-trust-notices.md).obsidian の無い .base が黙って0行(保管庫の根を知らせ、文書に書く)
  - [x] A-8(specs/_changes/2026-10-06-list-narrow.md)リストで打つと候補が消える(打った文字で候補を絞る)
  - [x] A-9(specs/_changes/2026-10-06-list-narrow.md)操作の一覧に流れの印が無い(`1/22`)
  - [x] A-10(specs/_changes/2026-10-06-docs-refresh.md)フォルダの --print に行の名前が無い(README に --with-path)
  - [x] A-11(specs/_changes/2026-10-06-docs-refresh.md)英語の `1 rows`(単数と複数)
  - [x] A-12(specs/_changes/2026-10-06-docs-refresh.md)--print-config に theme の名前の一覧が無い
  - [x] A-13(見本の日付は README に MDGRID_TODAY の一文。ほかは specs/_changes/2026-10-06-docs-refresh.md)文書の古さ: obsidian-bases.md の英語(「(空)」「画面は日本語」)、new_note のフォルダ、K の本文、集計を README に、--help の使い方に .md、「Seven themes」、examples/themes の英語、見本の日付が古くなる
- [ ] **B. 4回目の点検(2026-10-06、サブエージェント。初めて見た開発者の目。書き戻しはバイト単位で同じ、20,000 ノートの --print 約1秒を確かめた)で見つかったこと** — 高い順。
  - [x] B-1(specs/_changes/2026-10-06-audit4-first.md) 大きなフォルダで、起動のカーソルが表の途中(20,000 ノートで 5201 行目)。読み込みの途中の組み直しで選んだノートを追うため
  - [x] B-2(specs/_changes/2026-10-06-audit4-first.md) Hugo・Zola の TOML のフロントマター(`+++`)を「無い」とみなし、保存すると前に `---` の区画を足してページを壊す(BOM と同じく読むだけに)
  - [x] B-3(specs/_changes/2026-10-06-skip-hidden-dirs.md) README の `--pick path | xargs -o vi` が空白を含むパスで壊れる(見本の名前は全部空白入り)
  - [x] B-4(specs/_changes/2026-10-06-bom-values.md。アンカー・タグつきは CE-8 のまま) BOM つきのノートの行が全部空に見える(YAML は読める)。アンカー・タグつきの値が `{…}`
  - [x] B-5(specs/_changes/2026-10-06-skip-hidden-dirs.md。隠しフォルダと node_modules。.gitignore は読まない(却下の理由は決定の記録)) フォルダを開くと .gitignore・node_modules を見ない。隠しフォルダの扱いがばらばら。`.markdown` を拾わない
  - [x] B-6(specs/_changes/2026-10-06-print-md-rows.md。人の承認。もとは見送り: CLI-15 が「.md を渡した --print はフォルダの表と同じ出力」と決め、錠のある試験が確かめる。変えるには人の決定が要る) `mdgrid note.md --print` がフォルダ全体を出す(渡したノートだけに)
  - [ ] B-7(列を足すのは済: specs/_changes/2026-10-06-add-column.md。名前の変更・削除・置き換えは WB-1(人の決定)を変えるので人の判断待ち) 機能: プロパティの名前の変更・削除・足す(全ノートで)、列の置き換え
  - [x] B-8(specs/_changes/2026-10-06-print-filter-sort.md) 機能: --print に --filter・--sort
  - [x] B-9(specs/_changes/2026-10-06-apply.md) 機能: CSV・JSON で直して戻す(--apply)
  - [x] B-10(gif-01〜11 を nord で撮り直した) README の GIF が色なしで地味(テーマの色で、広い端末で撮り直す)
  - [x] B-11(直さない: CLI-5・OUT-3 の決まり。要素を保つなら --format json) CSV のリストを `, ` でつなぐので分けられない(`["a,b", c]` → `a,b, c`)
  - [x] B-12(specs/_changes/2026-10-06-audit4-first.md) 頻度表が 20,000 行で `81 (0%)`(1% 未満は小数1桁)
  - [x] B-13(直さない: BV-10 と同じく選んだノートを追う。VisiData なども同じ。先頭へは g g・Home) 大きな表で `s` の並べ替えのあと、カーソルが元のノートのまま(先頭が見えない)
  - [x] B-14(specs/_changes/2026-10-06-help-wording.md) 文書: cards のビューの理由に「version 1」(バイナリは 0.0.1)、--help の折り返し
  - [x] B-15(直さない: BV-11 のとおり実体のパスで1行にしている) シンボリックリンクのノート・フォルダを黙って飛ばす
- [ ] **C. 5回目の点検(2026-10-06、サブエージェント。本物に近い雑多な保管庫。--yes の書き込みはバイト単位で意図どおり、--print → --apply の往復は「変わる値が無い」を確かめた)で見つかったこと** — 高い順。
  - [x] C-1(specs/_changes/2026-10-06-datetime-offsets.md) `Z`・時差・空白の区切りの日時(`2026-10-05T21:44:59Z`)が型の合わない値になり、カレンダーで日を選ぶと時刻を捨てる。--apply も「YYYY-MM-DD で書く」と断る
  - [x] C-2(specs/_changes/2026-10-06-datetime-offsets.md) 同じ日時が式で誤る: `date("…Z")` が Z を落として地域の時刻に、`+09:00` の値は null、並べ替えで後ろに
  - [ ] C-3(見送り: Obsidian が日付の型のプロパティにある日時の値をどう見せ、どう並べるかを確かめてから。人に確かめてもらう問い) 日付の型の列の日時の値(`2026-10-05T07:00`)が型の合わない値として後ろに並ぶ
  - [x] C-4(specs/_changes/2026-10-06-flow-frontmatter.md) JSON・フローの形のフロントマター(`{"tags": [...]}`)の行が空で、--print は黙って空
  - [x] C-5(specs/_changes/2026-10-06-tag-keys.md。` #inbox` は YAML のコメントなので Obsidian も拾わない。候補の区切りは CE-3 の扱いで残す) `Tags`・`tag` のキーと、空白で区切った文字(`project/eps #inbox`)を file.tags・hasTag が拾わない。タグの候補が `daily, journal` を1つに
  - [x] C-6(specs/_changes/2026-10-06-types-case.md。Status と status の列をまとめるかは BV-1 の仕様の問いで残す) types.json の型を大文字小文字を区別して引く(`Rating` に `rating` の型が効かない)
  - [x] C-7(specs/_changes/2026-10-06-moment-tokens.md) `.format()` が Moment の `Do`・`hh`・`A`・`WW`・`gggg`・`Q` などを文字のまま出す
  - [x] C-8(specs/_changes/2026-10-06-cli-polish.md) --apply が file.*・formula.* の列の直しを黙って捨てる(何件捨てたかを知らせる)
  - [x] C-9(specs/_changes/2026-10-06-cli-polish.md) --sort の知らない列を黙って受ける(--apply と同じく綴りを確かめて終了コード 2)
  - [x] C-10(specs/_changes/2026-10-06-cli-polish.md) --apply の文言: `1 files would change`、重なりの行の番号が最初の行でない
- [x] **F-13 左のノートの欄の見せ方**(specs/_changes/2026-10-07-note-column.md。人の承認) [P1/M] — 今はフォルダと `.md` を付けたパス(`Tasks/Fix login redirect bug.md`)で、幅の上限で切れる。`.base` の order に `file.name` があると同じ名前が2回並ぶ。案: 全行に共通のフォルダと `.md` を除く、名前の列があるときは欄を隠す。golden 22本と錠のある試験に響くので、仕様の提案から(specs/_changes/2026-10-06-demo-gif.md の不明点)。
- [ ] **F-12 新しいノートの残り** [P2/S] — どのノートにも無い列を聞いて日付を答えると囲んで書く(型が分からない)、画面のビューごとの決まりの試験、テキストの列の候補、フォルダを2つ以上開いたときの作る場所
  - → 済: 画面のビューごとの決まりの試験とテキストの列の候補(specs/_changes/2026-10-03-new-note-gaps.md、3e206d9、照合 1 / 1)
  - → **人**(承認の一文): 守られる要件を変えるので、人の承認なしには採択できない。型の決まらない列の日付を囲まない(提案 untyped-date、WB-18)、フォルダが2つ以上のとき作る場所を選ぶ欄(提案 new-note-folders、CE-25)。記録 specs/_changes/2026-10-03-new-note-rest.md
- [x] **F-2 書けないファイル(0444)の扱い** — 人の承認(2026-10-03)で採択した readonly-permission を実装: specs/_changes/2026-10-03-readonly-permission.md(500f91e、照合 1 / 1)
  - → 人の判断(2026-10-03 の束): 読むだけにして理由を出す。提案 specs/_proposals/2026-10-03-readonly-permission.md(WB-5)。承認の一文を待つ
- [x] **F-3 strict-write-safety の提案の決着** — 人の承認(2026-10-03)で採択: specs/_decisions/2026-10-03-strict-write-safety.md(9要件を「しなければならない」に。実装は既に満たす。守られる要件になった試験に錠)
  - → 人の判断(2026-10-03 の束): 採る(人の依頼として出し直した)。specs/_decisions/2026-10-03-strict-write-safety.md。承認の一文を待つ。採択の順は list-lines → readonly-permission → strict-write-safety
- [ ] **F-4 sessions(CLI-6〜CLI-10)** [P1/L] 依存 D-5 — S1 読み書き → S2 起動の旗 → S3 表に当てる → S4 パレットから保存。
- [x] **F-5 `--print` と `--pick`(CLI-5・OUT-3)** — specs/_changes/2026-10-03-print-pick.md(b775fc4・ca74d1e、照合 3 / 3)
- [ ] **F-6 リレーション** [P2/L] **人** — 不明点の聞き取り(`[[ノート]]`・相対パス・根からのパス、保管庫の外の警告の設定、候補の出し方)→ 提案 → 設計 → 実装(2セッション)。
- [ ] **F-7 親タスクの階層表示** [P2/L] 依存 F-6 **人**(どの画面で見せるか)— 表の中で字下げして畳むか、別の画面の型か(グループ分けとぶつかる)。
- [ ] **F-8 次の段の残りを Issue にする** [P2] — 式の絞り込み・絞り込みの積み重ね・頻度表・正規表現の置き換え・summaries・書き出し・貼り付け・テーマ・前置きのキーの案内・保存後の取り消し・`this`・table 以外のビュー・CSV と JSON の読み込み・行ごとに外部のコマンドを動かす列。
- [x] **F-9 仕様の古い記述を直す** — specs/_changes/2026-10-03-stale-text.md(3632f5e・acdc643、照合 6 / 7。CLI-2 の gap は sessions の未実装で F-4 へ)。守られる要件(WB-1・WB-3・CE-10)の直しは提案 list-lines(人の承認待ち)

### Q. 品質とメンテナンスの自動化

- [x] **Q-1 GitHub Actions の CI** — specs/_changes/2026-10-06-oss-ci.md(check.py は CI で回さない、と AI が決めた)。元の項目: fmt・clippy・test(ubuntu・macos・windows)・MSRV・cargo-deny・rust-cache・concurrency・権限は `contents: read`・check.py の job。
- [ ] **Q-2 画面の golden を成果物に** [P1/S] 依存 Q-1 — 落ちたときの画面のテキストを upload-artifact。vhs の GIF も。
- [x] **Q-3 MSRV を決めて守る** — specs/_changes/2026-10-06-oss-ci.md(1.90。saphyr の ordered-float 5.5)。元の項目: `cargo msrv find`、`rust-version`・CI の job・README、方針を CONTRIBUTING に。
- [x] **Q-4 依存の更新** — specs/_changes/2026-10-06-oss-ci.md。元の項目: Dependabot(cargo と github-actions、週1回)。
- [ ] **Q-5 定期の検査** [P1/S] 依存 Q-1 — 週1回 cargo-deny の advisories と check.py の W6・W18、見つけたら Issue。
- [ ] **Q-6 リリースの自動化** [P0/M] — 一部済(specs/_changes/2026-10-06-release.md: タグで5つのターゲットのバイナリと git-cliff の変更点を GitHub Release に。残り: release-plz・crates.io・dist のインストーラ)。元の項目: release-plz と dist の release.yml。
- [ ] **Q-7 秘密の走査とブランチの保護** [P0/S] 依存 リポの作成 — secret scanning・push protection・gitleaks の pre-commit、ルールセット。
- [x] **Q-8 書き戻しの性質の試験** — specs/_changes/2026-10-03-writeback-props.md(4080306、照合 4 / 4)。proptest で7つの性質。空にするときの区切りの空白(WB-1 と CE-9)は人の判断待ち。cargo-fuzz は nightly が要るので置いていない
- [ ] **Q-9 カバレッジ** [P2/S] — cargo-llvm-cov を成果物に(門にはしない)。
- [x] **Q-10 本物のバイナリを動かす e2e の試験** — specs/_changes/2026-10-03-e2e.md(c54df44、照合 4 / 4)。portable-pty と vt100 で cargo test の中で回す(unix だけ)。道具は tests/pty/mod.rs

## 3. 公開までのセッションの順(案)

1. ~~F-1 click-name-editor~~ 2. ~~F-1 empty-frontmatter-config~~ 3. ~~D-1 前半(editor)~~ 4. ~~D-1 後半(--print-config と文書)~~
5. O-1 判断の束(F-2・F-3・O-3・D-5 も) 6. F-2 0444 7. H-1 錠 8. D-2 キーの一覧 9. D-3 `.base` の範囲 10. D-4 安全の文書
11. Q-1 CI と Q-3 MSRV 12. H-2 dev-loop の仕様 13. H-3 テンプレートとラベル + O-8 14. H-4 各段のプロンプト
15. O-4 Cargo.toml + O-7 CHANGELOG 16. O-12 インストーラの検討 → O-5 dist と tap + Q-6 17. O-2 README + O-9 GIF 18. O-3 英語化(3セッション)
19. O-10 書き出しと公開(門0、人の指示で)

公開後: Q-4・Q-5・H-5・H-6・F-4・F-5 → H-7 が済んだら H-8。

## 4. リスク

- 全自動のループで人の承認を照合できない(H-7 まで段階1)。
- 錠の無いまま AI に実装させるとテストを弱められる(H-1 を先に)。
- 履歴の書き出しは一方通行(公開リポを正本にし、今のリポは凍結か控え)。
- 決定の記録の会話の ID とラボのパス(E27・E28 は手元でしか効かない)。
- 日本語の仕様と外の貢献者(英語の要約か、貢献者は Issue だけで仕様化は持ち主の側の AI)。
- Actions の費用と悪用(起動は書き込み権限のある人だけ、費用の上限、同時実行の制限)。
- Windows の書き戻し(名前の変更・ハードリンク・fsync)は未確認。

## 5. 人が決めること(公開の前)

- リポの名前と置く所(`S6U5/mdgrid` か)・公開の範囲・今の履歴の扱い
- 著作権者の名前(`S6U5` のままか)と MIT OR Apache-2.0 の確定
- crates.io に出すか・Homebrew の tap の名前・対象の OS(Windows を含めるか)・公式にするインストーラ(O-12)・macOS の公証をするか
- 画面の言語(英語化を公開の条件にするか)
- 仕様と決定の記録を公開するか(しないとテストの `[XX-n]` が宙に浮く)・会話の ID とラボのパスの消し方
- check.py を CI でどこから取るか
- コミットの書き方(Conventional Commits か git-cliff で写すか)
- clap を入れるか
- AI の認証(API キーか OAuth か)・月の費用の上限・段階2に進む条件
- 承認待ちの提案(strict-write-safety)と未決の問い(0444、外部のコマンドを動かす列)

## 6. 2026-10-03 の判断の束と承認待ち

判断の束(7件)の答え: F-3 採る / F-2 読むだけにして理由 / D-5 clap を入れる(済) / F-12 型の決まらない列の日付は囲まない / F-12 フォルダが2つ以上なら作る場所を選ぶ欄 / O-3 英日の切り替えを公開の条件に / O-7 今の書き方のまま git-cliff で写す。どれもおすすめの案。

人の承認の一文を待つ提案(守られる要件に触る。check.py --approve で照合してから採択する):

| 提案 | 触る要件 | 頼む一文の例 |
|---|---|---|
| specs/_decisions/2026-10-03-strict-write-safety.md(F-3) | WB-1・2・4〜8、BV-3・7 | F-3 の strict-write-safety を人の依頼として承認します |
| specs/_proposals/2026-10-03-readonly-permission.md(F-2) | WB-5 | F-2 で書き込めない権限のノートを読むだけにする WB-5 の変更を承認します |
| specs/_proposals/2026-10-03-untyped-date.md(F-12) | WB-18 | F-12 で型の分からない列の日付を囲まずに書く WB-18 の変更を承認します |
| specs/_proposals/2026-10-03-new-note-folders.md(F-12) | CE-25 | F-12 でフォルダが2つ以上のとき作る場所を選ぶ欄を出す CE-25 の変更を承認します |
| specs/_proposals/2026-10-03-list-lines.md(F-9 の残り) | WB-1・WB-3・CE-10 | F-9 でリストを足す行の書き方と CE-6 の参照を直す WB-1・WB-3・CE-10 の変更を承認します |

済(2026-10-03、人の承認で採択: list-lines・readonly-permission・untyped-date・new-note-folders・strict-write-safety・null-separator)。以下はその前の記録。

新しい問い(Q-8 の性質の試験が見つけた。→ 案 A で採択: specs/_decisions/2026-10-03-null-separator.md): 値を空にすると `status: done` が `status:` になり、値の前の空白も消える。CE-9(`key:` と書く)と WB-1(値の範囲の外を変えない)がぶつかる。案 A(推し)WB-1 に「空にするときは区切りの空白も値の範囲」と一文足す / 案 B 実装を `status: ` に / 案 C 決めない。試験は空のときだけ許す形で置いてある(tests/test_writeback_props.rs の注)。

残した気づき(この回の照合から): CLI-2 に並べた sessions の旗は未実装(F-4)。NV-9 と NV-19 の中身が重なる。CLI-7 がビューの設定と mdgrid のビューを書いていない。Rust の試験の錠はファイルごと(関数ごとは decidespec の変更が要る)。cargo-fuzz は nightly が要る。--pick の選ぶ間のエディタの止めと OSC 52 の書き先は試験なし。評価できない列の警告に列の名前が2回出る。新しいノートの候補のリストはクリックで選べない。

## 7. 2026-10-06 の人の判断待ち(改善の繰り返しで、AI が決めずに残したもの)

どれも、人の決定の要件(守られる要件)か、錠のある試験に触る。決めたら、その決定の記録で錠を掛け直して直す。

| # | 項目 | 今 | 案(おすすめ) | 触るもの |
|---|---|---|---|---|
| 1 | O-5 `file.backlinks`・`file.hasLink()`(済: 人の承認) | 未対応の理由を出す(索引のコードは入っている) | 対応する(FILE_FIELDS に足すだけ) | 7つの要件の試験の錠 |
| 2 | O-12 列の見出し(済: 人の承認 2026-10-06) | displayName が無いと `formula.x`・`file.name` のまま | Obsidian の既定の見出し(`name` など)にする | 錠のある tests/test_base.rs・tests/test_print.rs |
| 3 | O-10 チェックボックスの Enter(済: 人の承認 2026-10-06) | true → false → 空 と回る | Obsidian と同じく true と false だけを切り替える | CE-4(人の決定) |
| 4 | O-11 1つの値で書かれたリスト(`tags: project`)(済: 人の承認 2026-10-06) | 読むだけ | 直すときにリスト(`[project, x]`)に書き換える | CE-19 と錠のある試験 |
| 5 | B-6 `.md` を渡した `--print`(済: 人の承認 2026-10-06) | フォルダの表を出す | 渡したノートの行だけを出す | CLI-15 と錠のある試験 |
| 6 | B-7 キーの名前の変更・削除・値の置き換え(全ノート)(名前の変更・削除は済: 人の承認 2026-10-06) | できない(列の追加は済) | 差分の確認つきの別の操作として足す | WB-1・WB-2(人の決定) |
| 7 | 公開(人の決定 2026-10-07: S6U5/mdgrid で公開・著作権者 S6U5・今は GitHub だけ・specs は会話の場所とパスを伏せて含める。準備済み: HEAD を履歴なしで書き出し、check.py 0 errors(W21 が4件。同じ日の記録の前後は履歴のあるこのリポで確かめ済み)・gitleaks 漏れなし。リポの作成と push は人の指示待ち) | 手元だけ | 5節のとおり(名前・著作権者・crates.io・Homebrew・公開の範囲) | O-10(門0) |
| 8 | F-13 左のノートの欄(済: 人の承認 2026-10-06) | `Tasks/Fix login redirect bug.md`(フォルダと `.md` つき) | 全行に共通のフォルダと `.md` を除き、名前の列があるときは欄を隠す | 錠のある画面の試験とゴールデン |

## 根拠

mdgrid の README・AGENTS.md・Cargo.toml・ci.sh・deny.toml・specs/(spec.md の次の段・範囲外・未決の問い、_changes の status、test-locks.json)・docs/design.md・src/main.rs・`check.py .` の結果(0 errors, 59 warnings)。ラボの control/harness(README・AGENTS.md・my-lab の ML-1〜ML-16・cross-repo)、cellops-creator の framework-split・agents-block、decidespec の README・check.py・hooks・spec-change の「公開するとき」・OM-12〜OM-17。外: crates.io の API、GitHub の検索、Homebrew の formula の API、anthropics/claude-code-action(README と docs/security.md)、release-plz の文書、axodotdev/cargo-dist。
