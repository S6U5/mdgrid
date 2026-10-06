---
type: Change
id: 01M495PEKDB89D3K2VYHEPBKBV
title: タグで crates.io にも出す(Trusted Publishing)と、README の入れ方(trusted-publishing)
status: done
size: tiny
created: 2026-10-07
updated: 2026-10-07
---

# タグで crates.io にも出す(Trusted Publishing)と、README の入れ方(trusted-publishing)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 対象外: tiny | なし |
| 仕様化 | 対象外: 要件を変えない | なし |
| 設計 | 対象外: tiny | なし |
| タスク | 対象外: tiny | なし |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

人が 2026-10-07 に mdgrid 0.1.0 を手で crates.io に出した。人は長く使えるトークンのローテーションを避けたいと述べた(会話 2026-10-07)ので、以後は release.yml がタグ `v*` で crates.io の Trusted Publishing(短い期限のトークン)を使って出す。その版が crates.io に既にあれば飛ばす。README の入れ方に `cargo install mdgrid` を足す。

## 照合

要件に触らない(公開の手順と文書)。release.yml の crates の job は、crates.io で mdgrid の Trusted Publishing(リポ S6U5/mdgrid・workflow release.yml・environment crates-io)を人が設定してから動く。次のタグで確かめる。

確かめた: 0 / 0
