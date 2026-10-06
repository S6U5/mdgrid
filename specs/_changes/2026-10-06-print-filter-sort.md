---
type: Change
id: 01M47B27G2V8PEEF1RRZHH8529
title: --print に --filter と --sort を足す(print-filter-sort)
status: done
size: full
created: 2026-10-06
updated: 2026-10-06
---

# --print に --filter と --sort を足す(print-filter-sort)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | specs/_decisions/2026-10-06-print-filter-sort.md |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の B-8。採択した CLI-16 のとおり。

## 不明点と仮定

- 仮定: フォルダの列は既定の表(BV-1。ノートのキー。file.name は無い)のまま。旗の有る無しで列が変わらない。
- 仮定: 列の名前は `.base` の書き方と同じ(`due`・`note.due`・`file.name`・`formula.x`)。`:asc`・`:desc` 以外の後ろの部分は理由1行と終了コード 2。

## 設計

- `base::Base::narrow(view, filters, sorts)`: ビューの絞り込みを「元の絞り込み AND 渡した式」にし、並べ替えがあれば置き換える。式は `.base` の式と同じ読み取り(読めない式は Leaf の Err のまま、評価のときに BV-7 の理由)。
- main.rs の print_view: `.base` ならそのビューに narrow、フォルダなら既定の列を order にした `.base` を作って narrow、mdgrid のビューなら理由1行。
- 引数(clap)と、ヘルプの文(i18n の Usage)・README・docs に足す。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | --filter・--sort | CLI-16 | 採択した確かめ方 | src/base.rs, src/main.rs, src/i18n.rs, tests/test_print_filter_sort.rs, README.md, README.ja.md | test_cli_16_*(tests/test_print_filter_sort.rs) | 済 |

## 実装の気づき

- フォルダの既定の \`--print\` には file.name の列が無い(BV-1 の既定の表の列)。初めの案の「設定の無い table のビュー」は file.name を足すので、旗の有る無しで列が変わる。採択の文を「既定の表と同じ列のまま」に直してから入れた(採択のコミットを直した。まだ公開していない)。
- 錠のある src/test_main.rs が \`Options\` を全部の項目で組み立てるので、\`Options\` には足さず、\`Command::Print\` に式と並べ替えを足した。
- 並べ替えの \`列:向き\` の読み取りは main.rs で先にし、知らない向きは理由1行。読めない式は Leaf の Err のまま、評価できない絞り込みのビュー(BV-7)として print が理由1行を返す。

## 照合

自分で照合(フレッシュ文脈でない。独立したレビューではない。式の評価と並べ替えは今のものを使い、引数をつなぐだけのため)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| CLI-16 | checked | コード: src/base.rs の Base::plain・narrow、src/main.rs の Cli・print_view / テスト: tests/test_print_filter_sort.rs の6本(フォルダの絞り込みと並べ替え、降順と2つの鍵、2つの式、.base の絞り込みと両方・並べ替えの代わり、読めない式・向き・--print なしで終了コード 2、旗の有る無しで列が同じ) / 今: 通った(全体 1262 passed) / 前: 落ちた(実装の前は旗が無く、5本が落ちた) |

既存のテストの削除・skip・弱体化: なし

確かめた: 1 / 1
