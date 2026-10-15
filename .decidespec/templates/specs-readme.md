---
type: Spec Index
id: {ULID}
title: {リポ名} の仕様
description: {リポ名} の仕様の正本の入口。
status: active
pace: concurrent
created: {YYYY-MM-DD}
updated: {YYYY-MM-DD}
---

# {リポ名} の仕様

このリポの仕様の正本。ケイパビリティ(何ができるか)ごとに `<capability>/spec.md` を置く。変えるときは `_proposals/` に提案を書き、採択したら `_decisions/` へ移す。誤字・表現だけの直しは提案なしで直し、コミットを `specs: 字句` で始める。`pace: concurrent` の間は、要求のあと仕様と実装を並べて進める。リリースしたら(人に配った・公開した)、`pace: sequential` にしてフェーズを順に進める(`sequential` から外すときは決定の記録が要る)。下の区画は `check.py --write` が書く。
