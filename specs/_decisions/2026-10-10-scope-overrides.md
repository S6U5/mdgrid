---
type: Proposal
id: 01M4JMQFDWXKVQJ9ND865638ET
title: 表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)
status: accepted
approval-evidence: 会話 2026-10-10(人の発言「さいたく」)
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-10
touches: [SR-44, SR-43, WS-1, WS-7, CLI-21]
created: 2026-10-10
updated: 2026-10-10
---

# 表のプロファイルを、ワークスペース・表・ビューの範囲で上書きできるようにし、設定の画面で保存先の範囲を選べるようにする(scope-overrides)

## きっかけ

人が、テーブルごと・ワークスペースごとに見た目を変えられるか(オーバーライドのように)、もちろん既定の見た目は設定できるとよい、と尋ねた(会話 2026-10-10)。今は見た目が全体で1つしかなく、ワークスペースの印・workspaces.toml・views.toml は見た目を持てない。AI が範囲と優先の案を示し、人は4段(全体・ワークスペース・表・ビュー)と、上書きできるのはプロファイル全部を選んだ。形は config-v2 の提案の表のプロファイルを使う。前の決定(look-section)では「見た目の変更をビューごとに残す」を「テーマと組は端末と人の好みで、表ごとに変わるものではない」として却下したが、人が表とワークスペースごとに変えたいと言ったので覆す。人の依頼を AI が要件の文に起こした。

## 差分

- 追加: SR-44（節「要件: 版1 — 設定と見た目の状態」）「表のプロファイル(CLI-3 の `[look]`・`[display]`・`[dates]`・`[edit]`・`[new_note]` と `use`)は、全体・ワークスペース・表・ビューの4つの範囲に書けるべき。全体は config.toml と ui.toml、ワークスペースはその印 `.mdgrid/workspace.toml`(WS-7)か workspaces.toml の `[[workspace]]`(WS-1)、表はワークスペースの表の項目(印の `[[table]]`・`[[workspace.table]]`)と views.toml の `[[table]]`(BV-20)、ビューはビューの設定(NV-17。mdgrid のビューは保存で定義にも入る。BV-18)に書く。ワークスペースの範囲は WS-6 で決めた範囲のものとし、検知(WS-5)と登録した表(CLI-18)から来た範囲にはワークスペースの範囲は無い。効く値は項目ごとに、狭い範囲を広い範囲より先にし(ビュー → 表 → ワークスペース → 全体 → 既定)、同じ範囲に2つあるときは画面が書いたものを手で書いたものより先にする(全体は ui.toml → config.toml、表は views.toml → ワークスペースの表の項目)べき。ただし、組(`look.preset`)を書いた範囲より広い範囲の部品ごとの形(`[look.style]`)と、テーマ(`look.theme`)を書いた範囲より広い範囲の役割の色(`[look.colors]` の役割。値の色 `[look.colors.values]` は除く)は使わないべき。`use = "<名前>"` は、テンプレート(`[templates.<名前>]`。同じ名前は ui.toml を config.toml より先にする)を、その範囲で書いた値の下に敷くべきで、無い名前は警告して無視するべき。アプリ全体の項目(最上位の `language`・`editor`・`poll_ms`、`[terminal]`・`[workspace]`・`[keys]`)は全体にだけ書け、ほかの範囲に書けば警告して無視するべき。表を切り替えたら(WS-2・CLI-19・関係マップから開く)、その表で効く値で画面を描き直すべき。関係マップのように1つの画面に複数の表が出るときは、今開いている表の値を使うべき。」（確かめ方: config.toml に `[look] theme = "sumi"`、notes/.mdgrid/workspace.toml に `[look] theme = "nord"`、views.toml の notes/tasks の `[table.look] theme = "dracula"` → notes/tasks の地が dracula、notes/projects は nord、notes の外の表は sumi。ワークスペースに `[look] preset = "saas"`、config.toml に `[look.style] status = "chip"` → ワークスペースの表の status は saas の形。ワークスペースに `[look] theme = "nord"`、config.toml に `[look.colors] accent = "orange"` → accent は nord の色。表に `use = "夜"`(`[templates.夜.look] theme = "dracula"`)と `[look] preset = "grid"` → dracula と grid。`use = "無い"` → 警告。ワークスペースに `[terminal] color = false` → 警告して無視。表に `[dates] format = "YYYY/MM/DD"` → その表だけ日付が `/` 区切り。関係マップから notes/projects を開く → nord で描き直す(`test_sr_44_*`)）
- 変更: SR-43 旧「ビューの設定の画面(NV-18)の左の一覧に「見た目」の区画を持ち、テーマ(SR-26・SR-37。`auto` を含む)・組(SR-36 の `preset`)・丸い札の端(`nerd_font`: `auto`・`true`・`false`)を選べるべき。選んだ見た目は「反映」で画面にすぐ効かせ、設定の置き場の `look.toml` に残し、次の起動では config.toml の `theme`・`[style]` の `preset`・`nerd_font` に代えて使うべき(config.toml は人が書いた注釈を持つファイルなので書き換えない)。組を変えたときは、config.toml の `[style]` の部品ごとの形は重ねず、その組の形にするべき。今の見た目の組み合わせ(テーマ・組・丸い札の端)は、名前を付けてテンプレートとして `look.toml` に保存でき、区画のテンプレートの一覧から選んで写しに当てられ、消せるべき。区画の「config.toml に戻す」で `look.toml` の見た目を外し、config.toml のとおりに戻せるべき。読むだけの起動(`--readonly`)では、反映で画面には効かせるが `look.toml` には書かないべき(WB-15)。区画の印は今の組の名前にするべき。」→ 新「ビューの設定の画面(NV-18)の左の一覧に「見た目」の区画を持ち、テーマ(SR-26・SR-37。`auto` を含む)・組(SR-36 の `look.preset`)・丸い札の端(`terminal.nerd_font`: `auto`・`true`・`false`)を選べるべき。区画の上で保存先の範囲(全体・このワークスペース・この表・このビュー。SR-44)を選べ、既定は全体にするべき。テーマと組の横には、今効いている値がどこから来たか(例「dracula(この表)」「sumi(config.toml)」)を出すべき。選んだ見た目は「反映」で画面にすぐ効かせ、選んだ範囲の画面が書く場所(全体は ui.toml、ワークスペースは workspaces.toml のそのワークスペース、表は views.toml の `[[table]]`、ビューはビューの設定)に残すべき(config.toml とワークスペースの印は人が書くファイルなので書き換えない)。丸い札の端は端末の性質なので、どの範囲を選んでいても ui.toml に残すべき。組を変えたときは、選んだ範囲より広い範囲の部品ごとの形は重ねず、その組の形にするべき(SR-44)。今のテーマと組の組み合わせは、名前を付けてテンプレートとして ui.toml の `[templates.<名前>]` に保存でき、区画のテンプレートの一覧(config.toml と ui.toml のテンプレート)から選んで写しに当てられ、ui.toml のものは消せるべき。区画の「この範囲の上書きを外す」で、選んだ範囲に画面が書いた見た目を外し、それより広い範囲のとおりに戻せるべき。ワークスペースの範囲が無いとき(SR-44)は保存先にワークスペースを出さず、ワークスペースが印から来たときにそれを選んで反映したら、印には書かないので印に手で書くことを理由として出し、書かないべき。読むだけの起動(`--readonly`)では、反映で画面には効かせるがどのファイルにも書かないべき(WB-15)。区画の印は今の組の名前にするべき。」（確かめ方: `o` → 左に「見た目 sumi」、保存先は「全体」。テーマ dracula・組 dozy-pink を選ぶ → 上に「未反映 1」。反映 → 画面の地が dracula の色、札が dozy-pink の形。ui.toml の `[look]` に theme = "dracula"・preset = "dozy-pink"。終了して開き直す → 同じ見た目。保存先「この表」でテーマ nord を反映 → views.toml の対象の `[table.look]` に theme = "nord"、テーマの横に「(この表)」、ほかの表は dracula のまま。「この範囲の上書きを外す」→ その表も dracula。「名前を付けて保存」で「夜」→ ui.toml に `[templates.夜.look]`、テンプレートに「夜」。組を sumi に戻して反映 → テンプレートの「夜」を選んで反映 → dracula と dozy-pink。印のワークスペースで保存先「このワークスペース」を反映 → 理由が出て、印は変わらない。`--readonly` → 反映で効くがファイルは無い。壊れた ui.toml → 警告して config.toml のとおりに起動。`test_sr_43_*`）
- 変更: WS-1 旧「ワークスペース(名前の付いた表の集まり)を、設定のフォルダの `workspaces.toml`(`[[workspace]]` の並び。`name` と、表の並び `table`(`name`・`path`(フォルダか `.base`)・`view`))に持つべき。ノートのフォルダには何も書かない。`path` の先頭の `~` はホームのフォルダ。読めない行は警告にして飛ばし、起動を止めてはならない。」→ 新「ワークスペース(名前の付いた表の集まり)を、設定のフォルダの `workspaces.toml`(`[[workspace]]` の並び。`name` と、表の並び `table`(`name`・`path`(フォルダか `.base`)・`view`)と、ワークスペースと表それぞれの表のプロファイル(SR-44。`[workspace.look]`・`[workspace.table.look]` など))に持つべき。ノートのフォルダには何も書かない。`path` の先頭の `~` はホームのフォルダ。読めない行は警告にして飛ばし、起動を止めてはならない。」（確かめ方: `[[workspace]]` に name = "Product"、表を2つ → 2つの表のワークスペース。path の無い表 → 警告して飛ばす。`[workspace.look] theme = "nord"` → そのワークスペースの表が nord(`test_ws_1_*`)）
- 変更: WS-7 旧「フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対)で、表を書かなければ根の直下の、ノートのあるフォルダを表にする(`.base` は WS-5 と同じ理由で入れない)。`mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。」→ 新「フォルダの `.mdgrid/workspace.toml` を、そのフォルダを根とするワークスペースの印として受けるべき。形はアプリの側の1つのワークスペースと同じ(`name`(無ければフォルダの名前)と表の並び `table`。`path` は根からの相対。ワークスペースの表のプロファイルは最上位に、表のものは `[table.look]` などに書く。SR-44)で、表を書かなければ根の直下の、ノートのあるフォルダを表にする(`.base` は WS-5 と同じ理由で入れない)。`mdgrid [<フォルダ>] --init-workspace` で、そのフォルダ(無ければ今のフォルダ)に表を書かない印を作り、既にあれば書かずに理由を出すべき。」（確かめ方: notes/.mdgrid/workspace.toml(表なし)→ notes の直下の tasks・projects がワークスペースの表。table を1つ書く → その表だけ。印に `[look] theme = "nord"` → その下の表が nord。`mdgrid notes --init-workspace` → notes/.mdgrid/workspace.toml ができる(`test_ws_7_*`)）
- 追加: CLI-21（節「要件: 版1 — 起動と設定」）「`mdgrid [<表>] --print-config --resolved` は、その表(引数が無ければ今のフォルダ。`--view` があればそのビュー)を開いたときに効く表のプロファイルとアプリ全体の項目の全部を、CLI-11 と同じ区画の TOML で標準出力に出し、各項目の上に、その値がどこから来たか(既定・config.toml・ui.toml・ワークスペースの印か workspaces.toml とその名前・views.toml の表・ビューの名前・テンプレートの名前)をコメントで添えて、終了コード 0 で終わるべき。出したものは設定ファイルとして警告なしに読めるべき。画面を開かず、ファイルは書かないべき。」（確かめ方: SR-44 の例で `mdgrid notes/tasks --print-config --resolved` → `theme = "dracula"` の上に `# views.toml (notes/tasks)`、`preset` の上に `# default`。出した文を `--config` で読む → 警告なし(`test_cli_21_*`)）

## 却下した案

| 案 | 理由 |
|---|---|
| 表ごとの見た目を `.base` に書く | `.base` は Obsidian と共有し、書き戻さない(BV-3) |
| 印(`.mdgrid/workspace.toml`)に画面から書く | ノートのフォルダに書かない(WS-1・SR-11)。印は人が書いて git で分け合うファイル |
| 後に読んだ範囲が勝つ(範囲に優先を付けない) | 狭い範囲が勝つほうが、どの値が効くかを予想しやすい |
| 組やテーマを変えても、広い範囲の部品の形と役割の色を重ねる | 組やテーマを選んだのに見た目が混ざる。今の SR-43(組を変えたら config.toml の部品の形は重ねない)をどの範囲にも広げるほうがそろう |
| 見た目だけを範囲で上書きできる | 人はプロファイル全部を選んだ(日付の形・新しいノート・表示も表ごとに違ってよい) |

## 錠

この提案で触る要件の試験のうち錠のあるもの(SR-43 の look.toml を読む試験など)は、採択のあと、錠の掛け直しの記録(config-v2-locks)で掛け直す(人の決定 Q2)。

## 承認の記録

2026-10-10 人の承認(要約: 設定の形の作り直しと範囲ごとの上書きの2つの提案を示し、人が「さいたく」と採択した。先に、旧い書き方は読んで移す・錠は掛け直してよい・上書きは4段・上書きできるのはプロファイル全部を選んでいた)
