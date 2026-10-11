---
type: skill
id: 01M4JJBQX8S151FWKBETPD4DDD
title: 公開(spec-change のモードの手順)
created: 2026-10-10
updated: 2026-10-10
---

# 公開するとき

呼ばれたときだけ読む。パス・守られる要件・人と AI・書き方は [SKILL.md](../SKILL.md) に従う(modes/ からは道具の根は `../../../`)。

decidespec を使うリポを公開するときは、公開範囲の切り替えや今の履歴の push をしない。今の中身だけを、履歴の無い新しいリポに書き出す(`git archive HEAD` を展開して `git init`)。書き出す前に、会話の記録がある環境で `check.py` を通す(E27・E28 が人の発言の引用・一致を拾う)(OM-17)。
