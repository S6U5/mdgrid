# 道具の芯と最初の版の範囲(scope) の要求

## 目的

mdgrid は、フロントマター付き Markdown の集まりをターミナルの表で見て、その場で値を直す単体の道具。Obsidian を開かずに使えて、Obsidian Bases の `.base` の table ビューの一部と互換の表を扱う。ここでは、道具の芯、作り方の大枠、最初の版に入れるものと入れないもの、公開と安全を決める。表の組み立ては [base-view](base-view.md)、セルの見せ方は [cell-view](cell-view.md)、セルの編集は [cell-edit](cell-edit.md)、ファイルへの書き戻しは [write-back](write-back.md)、画面と操作は [screen](screen.md)、移動・検索・選択は [navigation](navigation.md)、コピーと書き出しは [output](output.md)、起動と設定は [cli](cli.md) にある。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-17 に decidespec の仕様 [specs/_history/decidespec/scope/spec.md](../_history/decidespec/scope/spec.md) から移した。旧い ID との対応は [renames.md](renames.md)。

## 範囲外

- 次の段: Markdown と CSV・TSV(scope.csv-open〜scope.csv-features)以外の形式(JSON・YAML・SQLite など、ほかの表の道具が扱う形式)の読み込み口。どの読み込み口も、書き戻しの約束(write-back)を守る。CSV とノートの間のリレーションの持ち方は未定(人)。
- 次の段: AI エージェントの機能(起動・枠・実行ログ)。scope.no-agent-v1 で最初の版から省いてよいとされ、今後載せる。
- 次の段: タスクの画面。ガントは `gantt`(gantt.config〜gantt.screen-config)、カンバンは `kanban`(kanban.config〜kanban.docs)、グラフは `chart`(chart.config〜chart.docs)のケイパビリティに移した。カレンダー・マトリクスは、scope.screen-types と gantt.screen-config の形で別の画面の型として足す。
- 次の段: `.base` の table 以外のビュー(cards・list・map・kanban)。最初の版は table だけ(scope.table-only)で、足すなら scope.screen-types の別の画面の型として足す(表の見た目の切り替えで作るのは採らない。base-view の範囲外)。
- 次の段: ノートの中の ```` ```base ```` の埋め込み(base-view の範囲外)。
- 採らない: 既にある `.base` ファイルの編集(ビューや絞り込みを TUI で変える。base-view の base-view.base-untouched)。画面で作るビューは mdgrid のビュー(base-view.own-views)に持ち、`.base` へは新しいファイルへの書き出し(base-view.export-base)だけをする。

## 前提(旧い仕様で AI が置いた仮定)

- 仕様の量に上限は設けない(人の判断、2026-09-30。点検役が挙げた上限の案は採らない)。版1は縦切りを先に作る順で扱う。
- 作り始める時期は、cellops-creator と decidespec の実物が回ってからとしていた(設計ログの AI の判定。新しい道具が同時に増えると一人で回らない、という反証の評価役の注意による)。2026-09-30 に実装に入った(変更の記録 [v1-full-spec](../_history/decidespec/_changes/2026-09-30-v1-full-spec.md) のタスク 1〜3)。
- 公開するときは、決定記録の人の発言の原文と、非公開の設計ログへの参照(パス・コミット)を除いて書き出す(検証役の指摘。scope.publish-fresh-history の書き出しのときに行う)。
- cellops(自作の Rust の TUI)の仕様とコードは流用してよい(人の発言の要約: 自作なので流用してよい、2026-09-30。規範ではないので要件にしない)。ただし cellops のレイアウトは、タスクの画面を載せたときのものとして参考にとどめる。
- 名前は mdgrid(人の選択、2026-09-30。crates.io に同名なし、GitHub の同名のリポ2件)。cellops はこの TUI の上にタスクの画面(プラグイン)として載せてよい(scope.screen-types の画面の型)。

## 要求

### なぜ要るか

この節の要求は、1つの機能ではなく、いくつもの機能にまたがる価値を言う。機能の案と変更を測る物差しとして使い、機能の要求は `Rationale:` からここの名前を指してよい。

#### 台帳の仕事は、端末から出ずに終えられるべき
`req~scope.value-in-terminal~1` P1
- Rationale: GUI のアプリを開くのは疲れる。CLI のコーディングエージェントを使っているときは、端末のマルチプレクサの隣の区画で、同じ端末のまま台帳を開ける
- Acceptance:
  - Given: 機能の案が、その機能を使うのに端末の外の窓やアプリを開くことを求める
  - When:
  - Then: この価値を損なうので、端末の中で終えられる形に直すか、採らない(読んで判定)。

#### どの操作も、キーボードだけで手を止めずにできるべき
`req~scope.value-keyboard~1` P1
- Rationale: キーボードの操作で効率よく作業することが、この道具の目的だから
- Acceptance:
  - Given: 機能の案が、マウスでしかできない操作を持つ
  - When:
  - Then: この価値を損なうので、キーだけでできる道を足すか、採らない(読んで判定)。

#### 画面は、説明を読まなくても見て分かり、操作できる表を土台にするべき
`req~scope.value-intuitive-table~1` P1
- Rationale: キーボードで速く操作できても、利用者が直感で見て操作できる表でなければ使われないから
- Acceptance:
  - Given: 機能の案が、説明書を読まないと、何が見えていて次に何ができるかが分からない画面を足す
  - When:
  - Then: この価値を損なうので、見て分かる形に直すか、採らない(読んで判定)。

#### ノートのあいだのつながり(リレーション)は、どの画面でも見失わないようにするべき
`req~scope.value-relations-visible~1` P1
- Rationale: AI を使って作業していると、ノートのあいだのつながりをつい見失うため
- Acceptance:
  - Given: 機能の案が、ノートを指す値を持つ行を見せるのに、指す先を見ることもたどることもできない
  - When:
  - Then: この価値を損なうので、つながりを見てたどれる形に直すか、採らない(読んで判定)。

#### 仕事の場(表・ワークスペース)は、速く切り替えられるべき
`req~scope.value-fast-switch~1` P1
- Rationale: ほかの道具より速く、仕事の場を切り替えられるようにするため
- Acceptance:
  - Given: 機能の案が、別の表やワークスペースへ移るのに、mdgrid を終えて起動し直すことを求める
  - When:
  - Then: この価値を損なうので、終えずに切り替えられる形に直すか、採らない(読んで判定)。

### 芯と作り方

#### この道具の芯は、Markdown の YAML フロントマターを表として表示し、編集できることにするべき
`req~scope.core~1` P1
- Acceptance:
  - Given: 機能を足す案が、表の表示と編集のどちらにも役立たない
  - When:
  - Then: 芯から外れる(読んで判定)。

#### 単体の OSS として作るべき
`req~scope.standalone-oss~1` P1
- Acceptance:
  - Given:
  - When: 他の道具(cellops・Obsidian のプラグイン等)の一部としてしか動かない作り
  - Then: 違反(読んで判定)。

#### Obsidian を開かずに使えるようにするべき
`req~scope.without-obsidian~1` P1
- Rationale: Obsidian を開くとメモリを使うため
- Acceptance:
  - Given:
  - When:
  - Then: Obsidian を起動していない環境で、`.base` とノートのフォルダを渡すと表が出る(読んで判定)。

#### 最初から Rust で作るべき
`req~scope.rust~1` P1
- Rationale: 他で試作すると作り直しになるため
- Acceptance:
  - Given:
  - When: Rust 以外の言語・フレームワークで試作してから移す案
  - Then: 理由が無ければ違反(読んで判定)。

#### 製品名に、Obsidian の名前やロゴを使ってはならない
`req~scope.no-obsidian-brand~1` P1
- Acceptance:
  - Given:
  - When: 名前の候補に「obsidian」を含む
  - Then: 違反。README で互換を説明するのはよい(読んで判定)。

### 最初の版の範囲

#### 最初の版には、AI エージェントの機能を載せなくてよい(今後載せる)
`req~scope.no-agent-v1~1` P1
- Acceptance:
  - Given: 最初の版にエージェントの起動・枠・実行ログが無い
  - When:
  - Then: 適合(読んで判定)。

#### 最初の版では、値の書き換えとリストからの選択ができてよい
`req~scope.edit-v1~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 対象外(許可)。中身は cell-edit で決める。

#### 最初の版のビューは、table だけにするべき
`req~scope.table-only~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `.base` の cards・list・map・kanban のビューは開かず、未対応と表示する。

#### タスクの画面(ガント・エージェントの枠など)は、表のテーマの切り替えで作ってはならず、同じ核の上の別の画面の型として足すべき
`req~scope.screen-types~1` P1
- Acceptance:
  - Given:
  - When: テーマの設定に「ガントの列」を足す案
  - Then: 違反(読んで判定)。

#### 画面・編集・ためる変更・保存の流れは形式によらない共通の核にし、何を開くか・どう読み込み・どう書き戻すかを、形式ごとの読み込み口として差し替えられる形にするべき
`req~scope.source-adapters-core~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: 核のコードが Markdown の読み書きに直接触れず、読み込み口の口(`Source`)だけを通す(読んで判定)。

#### 最初の読み込み口は、フロントマター付き Markdown のフォルダだけでよい
`req~scope.source-adapters-markdown-first~1` P1
- Acceptance:
  - Given:
  - When: 最初の版の読み込み口がフロントマター付き Markdown のフォルダの1つだけ
  - Then: 適合(読んで判定)。

### 公開と安全

#### 人の明示の指示があるまで、このリポの remote を作ってはならず、公開してはならない
`req~scope.no-remote-without-ask~1` P1
- Acceptance:
  - Given:
  - When: `git remote -v` が空。人の指示の無いまま `gh repo create` を走らせる
  - Then: 違反(読んで判定)。

#### このリポに秘密情報を置いてはならない
`req~scope.no-secrets~1` P1
- Acceptance:
  - Given: API キーの形の文字列を含むファイルがある
  - When:
  - Then: 違反(読んで判定)。

#### 公開するときは、履歴の無い新しいリポに書き出さなければならない
`req~scope.publish-fresh-history~1` P1
- Rationale: 非公開のリポを公開に切り替えると、非公開の間のコミットが見えるため
- Acceptance:
  - Given: 公開の手順が「このリポの公開範囲を切り替える」になっている
  - When:
  - Then: 違反(読んで判定)。

#### 起動の引数に `.csv`・`.tsv` のファイルを渡したら、1行目を列の名前、2行目からを行とする表として開くべき
`req~scope.csv-open-table~1` P1
- Acceptance:
  - Given: `id,name,amount` と3行の `台帳.csv`
  - When:
  - Then: 3行3列の表。

#### CSV・TSV の区切りは、`.csv` なら `,`、`.tsv` ならタブとして読むべき
`req~scope.csv-open-delimiter~1` P1
- Acceptance:
  - Given: `id,name,amount` と3行の `台帳.csv` をタブ区切りにした `.tsv`
  - When:
  - Then: `台帳.csv` と同じ3行3列の表。

#### UTF-8 の CSV・TSV の先頭の BOM は、読み飛ばすべき
`req~scope.csv-open-bom~1` P1
- Acceptance:
  - Given: 先頭に BOM のある `id,name,amount` の `台帳.csv`
  - When:
  - Then: 1列目の名前は BOM を含まない `id`。

#### CSV・TSV の引用符の中の区切り・改行・二重の引用符(`""`)は、値として読むべき
`req~scope.csv-open-quotes~1` P1
- Acceptance:
  - Given: `台帳.csv` の `"東京, 本社"`
  - When:
  - Then: 1つの値。

#### UTF-8 でないか1行目が無い CSV・TSV は、理由1行で開かないべき(文字は UTF-8 だけを受ける)
`req~scope.csv-open-unreadable~2` P1
- Acceptance:
  - Given:
  - When: Shift_JIS のファイル
  - Then: 理由1行と 0 以外の終了コード。
- Acceptance:
  - Given: 1行目の無い空の `.csv`
  - When:
  - Then: 理由1行で開かない。

#### 列の数が行で食い違う CSV・TSV は、理由1行で開かないか、食い違う行を読むだけにして理由を出すべき
`req~scope.csv-open-ragged-rows~1` P1
- Acceptance:
  - Given: 列の数が足りない行
  - When:
  - Then: その行は読むだけで理由が出る。

#### CSV・TSV の書き戻しは、直したセルの値だけを書き換え、ほかの行・列の文字・引用符の付け方・改行の形(LF・CRLF)・最後の改行の有無を1バイトも変えないべき
`req~scope.csv-write-back-bytes~1` P0
- Rationale: CSV・TSV の書き戻しも、Markdown のノートと同じ約束を守るため
- Acceptance:
  - Given:
  - When: 2行目の amount を 300 に直して保存
  - Then: その値の文字だけが変わり、ほかのバイトと CRLF はそのまま。

#### CSV・TSV の書き戻しは、書く前にファイルの差分を見せるべき(write-back.pending-and-diff)
`req~scope.csv-write-back-diff~1` P1
- Rationale: CSV・TSV の書き戻しも、Markdown のノートと同じ約束を守るため
- Acceptance:
  - Given:
  - When: 2行目の amount を 300 に直して保存
  - Then: 書く前に `台帳.csv` の差分が出て、確定で書かれる。

#### CSV・TSV の書き戻しは、開いたあとに外で変わったファイルには書かないべき(write-back.conflict-stop)
`req~scope.csv-write-back-conflict~1` P0
- Rationale: CSV・TSV の書き戻しも、Markdown のノートと同じ約束を守るため
- Acceptance:
  - Given:
  - When: 保存の前に外で書き換える
  - Then: 書かずに理由。

#### CSV・TSV に書き戻す値は、区切り・引用符・改行が入るときだけ引用符で囲むべき
`req~scope.csv-write-back-quote~2` P0
- Rationale: 区切り・引用符・改行の入った値で CSV・TSV の行が壊れるのを防ぐため
- Acceptance:
  - Given:
  - When: `a,b` を入れる
  - Then: `"a,b"` と書く。

#### CSV・TSV の表では、値の直し(型ごとの入力・まとめての直し・取り消し)を使えるべき
`req~scope.csv-features-edit~1` P1
- Acceptance:
  - Given:
  - When: `台帳.csv` で status を done に直して保存
  - Then: `台帳.csv` のその行の status が done になる。

#### CSV・TSV の表では、絞り込み・並べ替え・グループ分け・ビューの設定と mdgrid のビュー(base-view.own-views)を使えるべき
`req~scope.csv-features-views~1` P1
- Acceptance:
  - Given:
  - When: `台帳.csv` で `o` で絞り込みと保存
  - Then: 絞り込んだ見せ方が mdgrid のビューとして残る。

#### CSV・TSV の表では、書き出し(`--print`)を使えるべき
`req~scope.csv-features-print~1` P1
- Acceptance:
  - Given:
  - When: `mdgrid 台帳.csv --print`
  - Then: 見出しの行と `台帳.csv` の行が CSV で標準出力に出る。

#### CSV・TSV の表では、行を足す操作を使えるべき(新しいノートの代わりにファイルの末尾に1行)
`req~scope.csv-features-add-row~1` P1
- Acceptance:
  - Given:
  - When: `台帳.csv` で `a` で行を足す
  - Then: `台帳.csv` の末尾に1行。

#### CSV・TSV の表では、ノートだけの機能を出さないか、理由を出して断るべき(エディタで開く・ノートの名前の変更・リンクとリレーション・関係マップ・親子の並べ方・キーの名前の変更と削除・フロントマターを足す)
`req~scope.csv-features-note-only~1` P1
- Acceptance:
  - Given:
  - When:
  - Then: `台帳.csv` で `e`・`R`・「ノートの名前を変える」→ CSV では使えない理由。

## 未確定事項

### 使うメモリの上限をいくつにするか
`tbd~scope.value-memory-budget`
- Decider: 人
- Provisional: 数は決めていない。限られたスペックの PC で、Electron などを使った GUI より少ないメモリで動くことが目的(人の発言、2026-11-27)。ノートの数と上限の数(MB)が決まったら、要求にする

### 表やワークスペースの切り替えにかけてよい時間
`tbd~scope.value-switch-time`
- Decider: 人
- Provisional: 数は決めていない。scope.value-fast-switch は「速く」を読んで判定する。何秒以内かが決まったら、測って確かめる受け入れ基準を足す
