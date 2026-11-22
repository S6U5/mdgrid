# 縦に並べる(stack) の要求

## 目的

別の表を1つの画面に上下に並べ、見比べながら、どの表でもふだんどおり探し・絞り・直せるようにする。並べ方はその場で作れ、気に入ったら名前を付けて「ページ」としてワークスペースに残し、次からは引き出しや起動の引数ですぐ開けるようにする(表の道具で、1ページに複数のデータベースのビューを上下に置くのと同じ考え)。

要求を足す・変える・消すときは、PR で人がレビューして承認する(AI だけで要求を変えない)。要求の文の「しなければならない・するべき・してよい」の読み方は [README.md](README.md)。2026-11-21 に足した。

## 範囲外

- 採らない: 同じ表の別のビューを並べること(2026-11-21 に人が「別の表」と決めた。同じ表のビューの見比べは、ビューのタブの切り替えで行う)。
- 採らない: 左右に並べること。端末の横幅は列に使う。
- 採らない(次の段): 上の表で選んだ行に合わせて下の表を絞ること(リレーションでつなぐ親と子の表示)。
- 並べるのは表の画面だけ。ガント・カンバン・グラフ・木・関係マップに替えるときは、選んでいる表だけを1つの画面で見せる。

## 要求

### その場で並べる

#### 表の画面で、パレットの「下に表を並べる」から今の範囲(workspace.scope-order-rank)の表を選ぶと、今の表の下にその表を並べるべき
`req~stack.add~1` P1
- Rationale: 別の表(タスクとプロジェクトなど)を、行き来せずに見比べるため(2026-11-20 に人が挙げ、2026-11-21 に「別の表」と決めた)
- Acceptance:
  - Given:
  - When: ワークスペース work の tasks を開き、パレットの「下に表を並べる」で projects を選ぶ
  - Then: 画面の上半分に tasks、下半分に projects の行が出る。
- Needs: stest

#### 並べた表は、それぞれ表の名前・ビューの名前・行の数を見出しの1行に出し、今選んでいる表の見出しを、ほかの表と色だけでなく印でも分けるべき
`req~stack.title~1` P1
- Rationale: どの表のどのビューを見ていて、キーがどの表に効くかを迷わないため
- Acceptance:
  - Given:
  - When: tasks の下に projects を並べ、projects を選ぶ
  - Then: projects の見出しの頭に選びの印、tasks の見出しには無い。見出しは「projects · 進行中 · 4」の形。

#### 並べた表の間は、`Ctrl+w` のあとの `j`・`k`(下・上の表へ)か、表の見出しや行のクリックで移るべき
`req~stack.focus~2` P1
- Rationale: vim の窓の移り方に合わせ、表の中の動き(`j`・`k`・Tab)とぶつけずに表の間を移るため
- Acceptance:
  - Given:
  - When: 上の tasks を選んで `Ctrl+w` `j`
  - Then: 下の projects が選ばれ、`j` が projects の中の行を動かす。
- Acceptance:
  - Given:
  - When: いちばん下で `Ctrl+w` `j`
  - Then: 動かない。
- Acceptance:
  - Given:
  - When: 下の表の行をクリック
  - Then: その表が選ばれ、その行が選ばれる。

#### 並べた表のどれでも、検索・絞り込み・並べ替え・ビューの切り替え・セルの編集・新しいノートは、今選んでいる表にだけ効くべき
`req~stack.each-table~2` P1
- Rationale: 並べても、1つの表を開いたときと同じ操作で使えるようにするため
- Acceptance:
  - Given:
  - When: 下の projects を選んで `\` で「会議」と打つ
  - Then: projects だけが絞られ、上の tasks の行は変わらない。
- Acceptance:
  - Given:
  - When: projects でビューのタブを移る
  - Then: projects だけのビューが替わる。

#### 並べた表から、選んでいる表を `Ctrl+w` のあとの `c` かパレットの「この表を閉じる」で外し、残りの表で画面を分け直すべき
`req~stack.close~2` P1
- Rationale: 見比べ終わった表を外して、もとの1つの表に戻すため
- Acceptance:
  - Given:
  - When: tasks と projects を並べて projects を選び `Ctrl+w` `c`
  - Then: tasks だけの画面。
- Acceptance:
  - Given:
  - When: 表が1つだけのときの `Ctrl+w` `c`
  - Then: 何もせず、メッセージ行に閉じられない理由。

#### 並べる表は 4 つまでとし、それぞれの表に見出しのほか 3 行が入らない高さの端末では、それ以上足さずに理由を出すべき
`req~stack.limit~2` P1
- Rationale: 1つ1つの表が読めないほど細くならないようにするため
- Acceptance:
  - Given:
  - When: 表を4つ並べて「下に表を並べる」
  - Then: 足さずに理由。
- Acceptance:
  - Given:
  - When: 高さ 12 行の端末で2つ目を足す
  - Then: 入るなら足し、入らなければ理由。
- Acceptance:
  - Given: 端末を低くして入らなくなる
  - When:
  - Then: 下の表から順に畳んで見出しだけにする。

#### 並べた表の高さは等しく分け、選んだ表の高さを `Ctrl+w` のあとの `+`・`-` で1行ずつ変えられるべき
`req~stack.resize~1` P1
- Rationale: 行の多い表に高さを多く回すため
- Acceptance:
  - Given:
  - When: 2つ並べて上を選び `Ctrl+w` `+` を2回
  - Then: 上が2行高く、下が2行低い。下が見出しと3行になったら、それ以上は上を高くしない。

### 変更と保存

#### 並べた表のどれかで直した値は、同じノートを出しているほかの並べた表にも、ためた変更として同じに見せるべき
`req~stack.shared-pending~1` P1
- Rationale: 同じノートを2つの表で別々に直しても、どちらの変更も1回の保存で書けるようにするため(別々にためると、片方を書いたあともう片方の保存が外の変更として止まる(write-back.conflict-stop)。値が消えないことは、その P0 の要求が守る)
- Acceptance:
  - Given:
  - When: 同じフォルダを指す表を2つ並べ、上の表で a.md の status を done に、下の表で a.md の title を「新」に直して保存
  - Then: a.md の status が done で title が「新」(どちらの変更も消えない)。
- Needs: stest

#### 並べた表の保存(`Ctrl+s`・`:w`)は、並べた表の全部のためた変更を、表ごとに分けた1つの差分の確認(write-back)に出してから書くべき
`req~stack.save-all~1` P1
- Rationale: 表ごとに保存し忘れて、閉じるときに変更を落とさないため
- Acceptance:
  - Given:
  - When: 上で1つ・下で2つ直して `Ctrl+s`
  - Then: 差分の確認に tasks の1件と projects の2件が見出しつきで並び、確定で3件とも書く。

#### ためた変更があるときに並べた表を閉じる(stack.close)なら、その表だけの変更を、保存するか捨てるかを確かめるべき
`req~stack.close-pending~2` P1
- Rationale: 閉じた表の変更を黙って捨てないため
- Acceptance:
  - Given:
  - When: 下の projects に1件ためて `Ctrl+w` `c`
  - Then: 保存か取りやめの確認。
- Acceptance:
  - Given:
  - When: 取りやめ
  - Then: projects は並んだまま。

### 名前を付けて残す(ページ)

#### 並べた形(表と、それぞれのビュー・高さの比)を、パレットの「並べ方に名前を付けて残す」で、今のワークスペースのページとして残せるべき
`req~stack.page-save~2` P1
- Rationale: よく見比べる組を、毎回並べ直さずに開けるようにするため(2026-11-21 に人が「両方」(その場で並べ、名前を付けて残す)と決めた)
- Acceptance:
  - Given:
  - When: tasks(今日)と projects(進行中)を並べ、名前「朝」で残す
  - Then: `workspaces.toml` の work に `[[workspace.page]]`・`name = "朝"`・2つの表とビュー。
- Acceptance:
  - Given: 範囲がワークスペースでないとき
  - When:
  - Then: 残さずに理由。
- Needs: stest

#### ページは、ワークスペース(`workspaces.toml` のワークスペースと印 workspace.marker)に `[[page]]`(`name` と、表の名前 `table`・`view`・高さの比 `height` の並び)で手で書いてもよく、読めない行は cli.config-file-warning-form の形で警告して飛ばすべき
`req~stack.page-store~1` P1
- Rationale: ページを設定として手で直したり、ほかのマシンへ写したりできるようにするため
- Acceptance:
  - Given: 印に `[[page]]` `name = "朝"` と、無い表の名前を1つ含む表の並び
  - When:
  - Then: 起動して警告が出て、ページは残りの表で開く。

#### 印のワークスペースで画面からページを残すときは、印のファイルに、ほかの行・並び・コメントを変えずに足すべき
`req~stack.page-save-marker~1` P1
- Rationale: 人が手で書いた印の中身を、ページを足すついでに崩さないため
- Acceptance:
  - Given:
  - When: コメントと空行を含む印でページ「朝」を残す
  - Then: 印の差分は足した `[[page]]` の区切りだけ。

#### ページは、引き出し(workspace.drawer)の今の範囲の表の上に並べ、選ぶと並べた形で開くべき
`req~stack.page-open~1` P1
- Rationale: 表と同じ所から、ページをすぐ開けるようにするため
- Acceptance:
  - Given:
  - When: ページ「朝」のある work で引き出しを開く → 表の上に「朝」→ 選ぶ
  - Then: tasks(今日)と projects(進行中)が並ぶ。

#### `mdgrid -w <ワークスペース> --page <名前>` で、そのページを並べた形で開けるべき
`req~stack.page-cli~2` P1
- Rationale: よく使う並べ方を、端末の道具の窓の命令やシェルの別名から1回で開くため
- Acceptance:
  - Given:
  - When: `mdgrid -w work --page 朝`
  - Then: tasks と projects が並ぶ。
- Acceptance:
  - Given: 無い名前
  - When:
  - Then: 「ページ「夜」はありません(ある名前: 朝)」の1行と終了コード 2。

#### ページを開いているときは、開き直し(resume)で伝える命令と最後の画面を、`-w <ワークスペース> --page <名前>` にするべき
`req~stack.page-resume~2` P1
- Rationale: 端末の道具が窓を立て直したときに、ページの並べ方のまま戻すため(その場で並べた形は名前が無いので、選んでいる表とビューだけを戻す)
- Acceptance:
  - Given:
  - When: Herdr の窓で `-w work --page 朝` を開く
  - Then: 伝える命令が `mdgrid -w work --page 朝`。
- Acceptance:
  - Given: その場で並べた形
  - When:
  - Then: 命令は選んでいる表とビュー(resume.reopen-args)。

## 未確定事項

### 並べた表の間を移るキー
`tbd~stack.window-keys`
- Decider: 人
- Provisional: vim の窓の操作に合わせた `Ctrl+w` の前置き(`j`・`k`・`c`・`+`・`-`)

### ページを置く所の名前
`tbd~stack.page-name`
- Decider: 人
- Provisional: `page`(印は `[[page]]`、`workspaces.toml` は `[[workspace.page]]`、引数は `--page`)
