---
name: decidespec-sdd-flow
description: "コードを足す・直す・消す依頼を、要求 → 不明点 → 仕様化 → 設計 → タスク → 実装 → 照合 の7つのフェーズで進める。使う場面: (1) decidespec の形で仕様を持つリポ(specs/README.md に decidespec の生成区画がある)でコードを変える依頼は、頼まれなくても必ず使う。(2) 仕様駆動のツールを何も使っていないリポで、人が「仕様駆動で」進めるよう頼んだときは、spec-init で specs/ を新設してから使う。他の仕様駆動ツールの仕様を持つリポ(.specify/・.kiro/・openspec/・.bmad-core/ など)では使わない。変更ごとに記録を作り、7つのフェーズを先に並べ、規模(tiny / bugfix / full)で省けるフェーズは「対象外: 理由」で宣言する。仕様だけを変える依頼は spec-change を使う。"
---

このスキルの本体は `.decidespec/skills/sdd-flow/SKILL.md` にあるので、そのファイルを読み、書かれた手順に従う。
