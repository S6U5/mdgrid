---
type: Proposal
id: 01M4E3N7702K8RF9XDDBXH8XTA
title: モダンな見た目を既定にしたのに合わせて、色の試験の比べる元を今までの見た目にして錠を掛け直す(modern-look-locks)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: 0b85c0848e1be5c4
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-09
touches: [SR-20, SR-26, BV-19, NV-18]
created: 2026-10-09
updated: 2026-10-09
---

# モダンな見た目を既定にしたのに合わせて、色の試験の比べる元を今までの見た目にして錠を掛け直す(modern-look-locks)

## きっかけ

会話 2026-10-09。人が lazygit や OpenTUI くらいモダンにしたいと頼み、AI が示した案のうち「全体を lazygit 風に(新しい見た目を既定にし、今の見た目は設定で残す。仕様の変更と、錠の掛かった試験の掛け直しが多めに要る)」を選んだ。これでモダンな見た目(SR-33。[modern-look](2026-10-09-modern-look.md))が既定になった。

錠のある色の試験(テーマの塗り替え SR-26、表の見せ方 SR-20)は、色のある表示で「テーマなしの画面は反転」を比べる元にしていた。比べる元と試験の App を、設定 `look = "classic"`(今までの見た目)で作るように変えた。確かめる中身(テーマの色・反転の外れ方・一行おきの色・見出しの色)は変えていない(TL-9)。モダンな見た目そのものは、新しい試験(src/ui/test_look.rs)で確かめる。

## 差分

- 錠: SR-26(src/ui/test_themes.rs・src/ui/test_themes_more.rs の、比べる元と試験の App を classic で作る)
- 錠: SR-20(src/ui/test_display_more.rs・src/ui/test_themes_more.rs の同じ変更)
- 錠: BV-19(src/ui/test_display_more.rs の同じ試験の関数が BV-19 も確かめる)
- 錠: NV-18(src/ui/test_display_more.rs の同じ試験の関数が NV-18 も確かめる)

## 却下した案

| 案 | 理由 |
|---|---|
| 色の試験をモダンな見た目で書き直す | テーマの塗り替え(SR-26)と見せ方(SR-20)が確かめたいのは今までの見た目の上の塗り替えで、モダンな見た目は SR-33 の試験で別に確かめる |
| 既定を classic のままにする | 人がモダンを既定にする案を選んだ |

## 承認の記録

2026-10-09 人の承認(要約: lazygit や OpenTUI くらいモダンにしたいと頼み、錠の掛け直しが多めに要ると書いた「全体を lazygit 風に」の案を選んだ)
