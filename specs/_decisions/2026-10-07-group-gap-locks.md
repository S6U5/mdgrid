---
type: Proposal
id: 01M4B09927NMFE49D6B1GPVTWQ
title: group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 911e8c95d491ed9b
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-07
touches: [SR-20, SR-21, NV-16, NV-23, NV-13, BV-17, CLI-11]
created: 2026-10-07
updated: 2026-10-07
---

# group_gap に合わせて、表示の設定の試験の錠を掛け直す(group-gap-locks)

## きっかけ

会話 2026-10-07。別のセッションが SR-30(まとまりの見出しの上を空ける group_gap。[group-gap](2026-10-07-group-gap.md))を入れたとき、錠のある2つのファイル(src/ui/test_display_options.rs・tests/test_display_options.rs)に、新しい空きの行に番号を付けない確かめと、[display] の項目が1つ増えた確かめを足したが、錠を掛け直していなかった(E19 が15件)。AI がこの掛け直しを人に示し、人がよいと答えた。要件の文は変えず、確かめる中身を弱めていない試験の錠だけを掛け直す(TL-9)。

## 差分

- 錠: SR-20(src/ui/test_display_options.rs の共有の手助け assert_numbered に、空きの行(Slot::Gap)に番号を付けない確かめを足した)
- 錠: SR-21(tests/test_display_options.rs の [display] の項目に group_gap が増えた。同じファイルの共有の手助けも変わる)
- 錠: NV-16(src/ui/test_display_options.rs の共有の手助けが変わる)
- 錠: NV-23(src/ui/test_display_options.rs の共有の手助けが変わる)
- 錠: NV-13(src/ui/test_display_options.rs の共有の手助けが変わる)
- 錠: BV-17(src/ui/test_display_options.rs の共有の手助けが変わる)
- 錠: CLI-11(tests/test_display_options.rs の print-config の [display] の項目の数が増えた)

## 却下した案

| 案 | 理由 |
|---|---|
| 試験を元に戻す | Slot::Gap の分岐が無いと試験の match が通らず、group_gap の既定値も確かめられない |

## 承認の記録

2026-10-07 人の承認(要約: 示した錠の掛け直しと、公開の順番(PR を先にマージ)でよいと答えた)
