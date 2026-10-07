---
type: Change
id: 01M4A6QG6HYQQNR28516B6S1P1
title: 履歴の書き出しの試験からマージコミットを外す(cliff-no-merges)
status: done
size: tiny
created: 2026-10-07
updated: 2026-10-07
---

# 履歴の書き出しの試験からマージコミットを外す(cliff-no-merges)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny(1ファイル、要件の意味を変えない) | なし |
| 仕様化 | 対象外: 要件の無い開発の道具の試験(specs/_changes/2026-10-03-cliff.md) | なし |
| 設計 | 対象外: tiny | なし |
| タスク | 対象外: tiny | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

公開のリポの初めての PR(S6U5/mdgrid#6、文書だけ)で、3つの OS の test と msrv が落ちた。actions/checkout は PR の仮のマージコミット(`Merge <head> into <base>`)を取り出し、`tests/test_cliff.rs::test_cliff_history_prefixes_all_covered` がその件名を commit_parsers のどれにも当たらないとして落ちる。main への直接の push では出ないので、今まで見えなかった。

終わりの条件: 履歴の突き合わせは、マージコミット(親が2つ以上)を数えない。マージコミットのある履歴でもこの試験が通り、書き出しの無いふつうのコミットは前と同じく落とす。

## 不明点と仮定

- 仮定: 公開のリポの main はスカッシュでマージするので、本物の履歴にマージコミットは残らない。CHANGELOG(git-cliff)の側は変えない。

## 設計

対象外: tiny

## タスク

対象外: tiny

## 実装の気づき

なし

## 照合

自分で照合(1行の直し)。マージコミットのある履歴を一時の複製で作り、試験を比べた。

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無い変更(開発の道具)の照合: checked。コード: tests/test_cliff.rs の git_subjects(`git log --no-merges`) / テスト: tests/test_cliff.rs::test_cliff_history_prefixes_all_covered / 今: `Merge abc into def` のある履歴で通った(6 passed)。このリポの履歴でも通った(別のビルド先で 6 passed) / 前: 同じ履歴で落ちた(「当たらない 1 件: Merge abc into def」) / 変異: 書き出しの無いふつうのコミット `noprefix` を足すと、直した後も落ちる

既存のテストの削除・skip・弱体化: なし(マージコミットだけを外した。ふつうのコミットの突き合わせは前と同じ)

確かめた: 0 / 0(要件の無い変更。上の文のとおり checked)
