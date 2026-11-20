---
type: Proposal
id: 01M46PBFZ8C0DP231QF4BDKSWM
title: --print に各行のノートのパスの列を足す --with-path(print-with-path)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: docs/todo.md
touches: [CLI-14]
created: 2026-10-06
updated: 2026-10-06
---

# --print に各行のノートのパスの列を足す --with-path(print-with-path)

## きっかけ

会話 2026-10-06 の改善の繰り返しで、初めて使う人の目の点検(docs/todo.md の U-3)が、`--print` の行がどのノートのものか分からない(フォルダを出すとフロントマターの無いノートは `,,,,` の空の行になり、JSON は null だけのオブジェクト)と指摘した。スクリプトで表を使う人が最初に要るのは、行とノートの結び付き。既定の出力を変えると今の使い方と試験が壊れるので、旗で足す。

## 差分

cli の spec.md の CLI-13 の次に足す。

- 追加: CLI-14「`--print` に `--with-path` を付けたときは、表の先頭に見出し `path` の列を足し、各行にそのノートのパス(OUT-3 の `--pick path` と同じ形: 起動の引数のフォルダにノートの相対のパスをつないだもの)を出すべき。json では各オブジェクトの最初の鍵 `path` にし、ビューに同じ見出しの列があれば CLI-5 の重なりの決まりで見分けるべき。`--print` の無い `--with-path` は CLI-4 と同じく理由1行と終了コード 2 にするべき。」(確かめ方: `mdgrid examples/demo/Tasks.base --print --with-path` → 見出しの行が `path,status,…`、各行の先頭が `examples/demo/Tasks/Fix login redirect bug.md` のようなパス。`--format json` → 各オブジェクトの最初が `"path"`。フロントマターの無いノートの行もパスで見分けられる。`--with-path` だけ → 理由1行と終了コード 2(`test_CLI_14`))

## 却下した案

| 案 | 理由 |
|---|---|
| 何もしない | 行とノートを結べず、`--print` の結果をスクリプトで直すことができない |
| 既定で path の列を出す | 今の出力を読むスクリプトと試験が壊れる(CLI-5 の「列はビューの列の並び」) |
| `--columns` で列を選ぶ | 便利だが大きい。path が無いことが先に困る |

## 承認の記録

AI 自己採択(守られる要件に触らない。CLI-5 と CLI-2 の決定はどれも AI。人は 2026-10-06 に判断を AI に委ねた)
