---
type: Proposal
id: 01M5S6YTKM3NQ9A059Q14QN56P
title: Windows でパスの区切りの違いで落ちるスターターの試験の補助を直し、錠を掛け直す(windows-starter-tests-locks)
status: accepted
decided-by: AI
trigger: 検証の指摘
trigger-link: https://github.com/S6U5/mdgrid/pull/45
touches: [ST-1, ST-2, ST-3, ST-4, ST-5]
created: 2026-10-25
updated: 2026-10-25
---

# Windows でパスの区切りの違いで落ちるスターターの試験の補助を直し、錠を掛け直す(windows-starter-tests-locks)

## きっかけ

人の決定 2026-10-24-starter で入れたスターターの試験は、手元の Mac で書いて錠を掛けた。公開リポの PR #45 の CI(windows-latest)で初めて Windows で回り、単体の試験 9 件が落ちた(`test_st_1_*` 3 件・`test_st_2_*` 2 件・`test_st_4_*` 4 件)。落ち方はどれも、試験の補助がファイルの相対パスを `to_string_lossy()` で文字にして `".mdgrid/templates/card.md"` と比べているのに、Windows では `.mdgrid\templates\card.md` になる、というパスの区切りの違い。要件の振る舞い(印と雛形を同じ相対パスで写す・見本の変数を埋める・1つでもあれば書かない)は Windows でも同じで、確かめる強さも変えない。

直す所は試験の補助だけで、試験の関数の本文と、スターターの核(`src/starter.rs`。`StarterFile.rel` は OS 生のパスのまま)は変えない。CLI・e2e・見本の同期の試験(`tests/test_starter_*.rs`)は先頭に `#![cfg(unix)]` があり Windows では組み立てないので、この提案の外。

| ファイル | 直し |
|---|---|
| `src/test_starter_unit.rs` | 補助 `rels`・`files_under`・`file_rels` が返す相対パスの `\` を `/` にそろえる |
| `src/ui/test_starter.rs` | 補助 `files_under` が返す相対パスの `\` を `/` にそろえる |
| `src/ui/test_starter_more.rs` | `test_st_5_folder_is_relative_to_the_open_folder_unless_home_or_absolute` の絶対パスの例を、`/abs/x`(Windows では絶対でない)でなく OS の一時フォルダから作る |

## 差分

- 錠: ST-1(src/test_starter_unit.rs の補助 `rels`・`files_under`・`file_rels`。区切りを `/` にそろえる。関数の本文は変えない)
- 錠: ST-2(同じ補助を使う test_st_2_* の2件)
- 錠: ST-3(同じファイルの test_st_3_* の5件。補助の変更で錠の値が変わるだけで、関数の本文は変えない)
- 錠: ST-4(同じ補助を使う test_st_4_* の4件)
- 錠: ST-5(src/ui/test_starter.rs の補助 `files_under`。src/ui/test_starter_more.rs の絶対パスの例を OS の一時フォルダから作る)

要件の文と例は変えない。

## 却下した案

| 案 | 理由 |
|---|---|
| スターターの単体と画面の試験を Windows で組み立てない(`#![cfg(unix)]`) | Windows は対象(2026-10-15-windows-target)で、スターターも Windows で使う。確かめる強さが落ちる |
| 試験の期待値を OS ごとに分ける | 補助で区切りをそろえれば1つの期待値で済み、読みやすい |
| 試験を消す | 確かめる強さが落ちる |

## 錨との照合

- 要件 ST-1〜ST-6 の文と例は変えない。変えるのは試験の補助(相対パスの文字の作り方)と、絶対パスの例の作り方だけで、macOS・ubuntu で確かめる値と強さは今までどおり。Windows では、今まで組み立てられずに確かめていなかったものが、同じ強さで確かめられるようになる。
- 前例: 2026-10-15-windows-tests-locks(AI、検証の指摘)が、人の決定 2026-10-15-windows-target(Windows を対象に入れる)の漏れを、同じ形(補助と OS ごとの既定の直し・錠の掛け直し)で埋めた。ここもその向きに沿う。
- 人の軌跡: 2026-10-24-starter(人の決定)は Windows を除いておらず、スターターの試験が Windows で通ることは、その決定とこれから出す v0.4.0(Windows の実行ファイルも配る)の向きそのもの。
- specs に principles.md は無く、照らす P 番号も gated も無い。

判定: 向きに沿う。新しい文脈の検証役(書込なし)も、作業ツリーの差分と PR #45 の windows-latest の記録(lib の試験 328 通り 9 落ち、落ち方はどれも区切りの違い)を読んで「向きに沿う。食い違う人の決定: なし」と判定した。検証役の抜けの指摘2点のうち、核 `src/starter.rs` の `rel` の持ち方を変える案は取り下げて補助だけの直しにし(表示が混ざった区切りになるのを避ける)、CLI の試験の範囲の漏れは `#![cfg(unix)]` で Windows では組み立てないので当たらない。

## 承認の記録

AI 自己採択(人の決定の要件の文は変えず、試験の補助の OS の違いを直すだけ。錨との照合あり。2026-10-25)
