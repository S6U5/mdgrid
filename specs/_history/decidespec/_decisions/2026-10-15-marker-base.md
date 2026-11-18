---
type: Proposal
id: 01M4XR07MA2V4EATXZ56ZD1D7T
title: 印の表に、.base と同じ中身(絞り込み・計算の列・表示名・ビュー)を TOML で書けるようにする(marker-base)
status: accepted
decided-by: AI
trigger: 人の発言
trigger-link: 会話 2026-10-15
touches: [WS-9]
created: 2026-10-15
updated: 2026-10-15
---

# 印の表に、.base と同じ中身(絞り込み・計算の列・表示名・ビュー)を TOML で書けるようにする(marker-base)

## きっかけ

人が、タスクの台帳の見え方を `.base` のファイルではなくフォルダの設定ファイル(印 `.mdgrid/workspace.toml`)で決めたいと言い、AI の出した2案(`.base` のまま・印にビューを書く)から印にビューを書く案を選んだ。mdgrid の設定の機能が使えるかを自分で使って確かめる目的もあり、設定は柔軟で手で書きやすいほうがよいと言った(会話 2026-10-15。プロジェクトのチャット)。台帳の表は、計算の列(担当)と列の表示名と、状態ごとのビューを使う。

AI の判断: mdgrid の自前のビュー(BV-17)には計算の列と列の表示名が無いので、新しいビューの形を作らず、`.base` と同じ中身を TOML の表で書けるようにして、`.base` の読み取り(base-view の BV-1〜)をそのまま使う。こうすると Obsidian の Bases と同じ書き方・同じ意味になり、文書も `.base` の説明を指せる。項目の名前 `base` と形は AI が足した細部。

## 差分

- 追加: WS-9(節「版1 — ワークスペース」)「印の `[[table]]` には、`path` の `.base` のファイルの代わりに、`.base` と同じ中身(`filters`・`formulas`・`properties`・`summaries`・`views`。キーの名前と式は `.base` と同じ)を TOML の表 `base` に書いてよく、そのときその表は、印の根に置いた同じ中身の `.base` を開いたのと同じに振る舞うべき(ビューのタブ・絞り込み・計算の列・表示名・グループ分け・並べ替え・`--print`・`--check`・`--view`)。ノートを探す範囲は印の根からにし(BV-2)、書いた中身は mdgrid が書き換えないべき(BV-3 と同じ理由)。`base` を書いた表には `path` は要らず、両方を書いたときは `base` を使って警告するべき(CLI-3 の形)。中身が `.base` として読めないとき(BV-1 の読めない `.base`)は、`.base` のときと同じく理由を出して開かないべき。」(確かめ方: 根に `tasks.base` を置かず、印に `open = "台帳"`、`[[table]] name = "台帳"` と `[table.base]` に `filters = { or = ['file.inFolder("todo")'] }`・`[table.base.formulas] 担当 = 'if(actor == "ai", "AI", "人")'`・`[table.base.properties."note.title"] displayName = "題名"`・`[[table.base.views]]` を2つ(`name = "未完了"` で `status != "done"`、`name = "終わった"` で `status == "done"`、どちらも `groupBy = { property = "formula.担当", direction = "ASC" }`)→ 根で `mdgrid --print` が未完了のビューの行で、見出しに「題名」。`--view 終わった` → done の行だけ。画面でタブに「未完了」「終わった」があり、担当でまとまりに分かれる。AGENTS.md の行は無い。開く前後で印のバイトが同じ。`base` と `path = "x.base"` を両方書く → `.mdgrid/workspace.toml: table[1].path:` で始まる警告で `base` を使う。`[table.base] views = "x"` のような読めない中身 → 理由1行と 0 以外の終了コード(`test_ws_9_*`))

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない(`.base` のまま) | 人は印にビューを書く案を選んだ |
| mdgrid の自前のビュー(views.toml の形)に計算の列と列の表示名を足して印に書く | 同じことを2つの書き方で持つことになり、`.base` と意味がずれるもとになる。`.base` の読み取りと試験をそのまま使えない |
| 印の中に `.base` の YAML の文を文字列で埋める | 手で書きにくく、TOML の検査も効かない |

## 承認の記録

2026-10-15 AI 自己採択。新しい要件を足すだけで、守られる要件の文は変えない。人はプロジェクトのチャットで印にビューを書く案を選んだが、その答えは会話の記録で照合できる形で残っていないため、人の決定にはしない。
