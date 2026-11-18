---
type: Proposal
id: 01M3XZNQDN7P4X3JBPWSACTEZD
title: ノートを開くエディタを設定で選べる(editor-setting)
status: accepted
approval-evidence: 〔伏せ字: 会話の記録の場所〕
approval-hash: a7cebc4705b0e838
decided-by: 人
trigger: 人の発言
trigger-link: 会話 2026-10-02
touches: [SR-8]
created: 2026-10-02
updated: 2026-10-02
---

# ノートを開くエディタを設定で選べる(editor-setting)

## きっかけ

会話 2026-10-02 で、人が編集のときのエディタを設定で変えられるかを尋ねた。今は環境変数 `$EDITOR` だけで決まり、空なら `vi`。設定の `editor`、`$VISUAL`、`$EDITOR`、`vi` の順で決める案を示し、人が始める作業に選んだ。人の依頼を AI が要件の文に起こした。順(git などと同じ)は AI が示し、人が選んだ。

## 差分

- 変更: SR-8
  - 旧: 「選んだ行のノートを `$EDITOR` で開けるべきで、戻ったらそのノートを読み直すべき。`$EDITOR` の値が引数つき(`code -w` など)でも動き、シェルを通さずに起動し、ノートのパスは1つの引数として渡すべき。」(確かめ方: `EDITOR="code -w"` で開く → 戻ると行が更新(`test_SR_8`))
  - 新: 「選んだ行のノートをエディタで開けるべきで、戻ったらそのノートを読み直すべき。エディタは、設定(CLI-3 の設定ファイル)の `editor`、環境変数 `$VISUAL`、`$EDITOR` の順で空でない最初のものとし、どれも無ければ `vi` とするべき。値が引数つき(`code -w` など)でも動き、シェルを通さずに起動し、ノートのパスは1つの引数として渡すべき。」(確かめ方: `EDITOR="code -w"` で開く → 戻ると行が更新。設定に `editor = "nvim"` と `EDITOR=vim` → nvim で開く。設定が無く `VISUAL=hx`・`EDITOR=vim` → hx。どれも無い → vi。設定の `editor = ""` は無いのと同じ(`test_SR_8`))

## 却下した案

| 案 | 理由 |
|---|---|
| `$EDITOR` だけ(今のまま) | mdgrid だけ別のエディタにできず、設定を dotfiles で配れない |
| `$VISUAL` を見ない | 端末の道具の多く(git など)が `$VISUAL` を先に見る。合わせた方が驚きが少ない |

## 承認の記録

2026-10-02 人の承認(要約: 編集のときのエディタを設定で変えられるかを尋ね、設定 > $VISUAL > $EDITOR > vi の順の案を始める作業に選んだ)
