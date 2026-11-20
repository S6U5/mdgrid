---
type: Change
id: 01M46MP348XNKY7PB4MFAHBDFP
title: Actions の外部のアクションを SHA で固定し、書き込みの権限の job から外す(actions-pin)
status: done
size: tiny
created: 2026-10-06
updated: 2026-10-06
---

# Actions の外部のアクションを SHA で固定し、書き込みの権限の job から外す(actions-pin)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 対象外: 製品の振る舞いを変えない(CI とリリースの設定) | なし |
| 設計 | 対象外: 条件に当たらない | なし |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

release(specs/_changes/2026-10-06-release.md)のあとの自動の安全の点検で、書き込みの権限(contents: write)を持つ release の job が外部のアクション(orhun/git-cliff-action)をタグで使い、ci.yml・release.yml の外部のアクション(dtolnay/rust-toolchain・Swatinem/rust-cache・EmbarkStudios/cargo-deny-action)もタグで使っていると指摘された。タグは差し替えられうるので、外部のアクションを40桁の SHA で固定し(版をコメントに)、変更点を作る job を読むだけの権限に分けて、書き込みの job は GitHub のアクションと gh だけにする。

終わりの条件: 外部のアクションがすべて SHA で固定され、contents: write の job に外部のアクションが無い。YAML として読める。関係する要件: 該当なし。

## 不明点と仮定

- 仮定: GitHub 自身のアクション(actions/*)はタグのまま。Dependabot(github-actions)が SHA とコメントを上げる。
- 仮定: MSRV の job は rust-toolchain の固定した SHA に `toolchain: 1.90` を渡す(ブランチの名前で版を選ばない)。

## 設計

対象外: 条件に当たらない

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | アクションの固定と job の分け方 | なし(CI とリリースの設定) | 外部のアクションが SHA で、書き込みの job に外部のアクションが無い | .github/workflows/ci.yml, .github/workflows/release.yml | 読んで判定 | 済 |

## 実装の気づき

なし

## 照合

自分で照合(フレッシュ文脈でない。要件の無い CI とリリースの設定のため)。

| 要件 | 結果 | 証拠 |
|---|---|---|

要件の無いタスク1の照合: checked。読んで判定: 2つのワークフローを ruby の YAML で読めた。外部のアクション4つ(rust-toolchain v1・rust-cache v2.9.2・cargo-deny-action v2.1.1・git-cliff-action v4.9.1)を `git ls-remote` で引いた40桁の SHA で固定し、版をコメントにした。git-cliff は読むだけの権限の notes の job に移し、contents: write の release の job は actions/download-artifact と gh だけになった。

既存のテストの削除・skip・弱体化: なし(試験に触れていない)

確かめた: 0 / 0(要件の無い変更。タスク1は上の文のとおり checked)
