# ai-human: 人のタスクと AI のタスクを分けて見る見本

AI エージェントに仕事を任せていると、タスクは2種類に分かれます。AI に任せきれるものと、最後に人の手が要るもの(払う・署名する・提出する・送る、PC の外での手続き)です。この見本は、その2つをまとまりに分けて、人のタスクを上に並べる表を、`.base` 1つと設定 1つで作ります。

`vault/タスク/` に 11 個のタスク(終わっていないもの 9・終わったもの 2)。日付は 2026年10月に寄せてあります。

## 開く

リポの根で、一時フォルダに写してから開きます(写した先なら編集して保存してよい):

```sh
cp -R examples/ai-human /tmp/mdgrid-ai-human
mdgrid --config /tmp/mdgrid-ai-human/config.toml /tmp/mdgrid-ai-human/vault/タスク.base
```

最初のタブ「人とAI」で、`👤 人間のタスク` と `🤖 ロボットのタスク` の見出しの下にタスクが並びます。見出しの上で `Enter` を押すと畳めます。`[` `]` でほかのビューに切り替えます。

| ビュー | 中身 |
|---|---|
| 人とAI | 終わっていないタスクを、人 → AI のまとまりに分ける。まとまりの中は優先度 → 期限の順 |
| 人だけ | 人の手が要るタスクだけを期限の順に |
| AIだけ | AI に任せられるタスクだけを優先度の順に(どの Agent がどこで動くか) |
| 終わった | 終わった・取りやめたタスクと、その担当 |

表として出すこともできます:

```sh
mdgrid /tmp/mdgrid-ai-human/vault/タスク.base --print --format md --with-path --view 人だけ
```

## 人か AI かの決め方

`vault/タスク.base` の式 `担当` で決めます。作者の別の道具 cellops で使っていた決め方と同じです。

```yaml
formulas:
  担当: if(actor == "human" || if(human_gate, human_gate != "none", false), "👤 人間のタスク", "🤖 ロボットのタスク")
```

- `actor: human` … PC の外の仕事(窓口・移動など)。人。
- `human_gate` が `submit`・`pay`・`sign`・`send` … 準備は AI、最後のボタンは人。人。
- それ以外 … AI。`actor` を書いていないノートも AI。

ビュー「人とAI」は `groupBy` にこの式を使います。`👤` は `🤖` より文字の順が前なので、昇順(`ASC`)で人のまとまりが上に来ます。

## フロントマター

| キー | 列の名前 | 値の例 |
|---|---|---|
| `status` | 状態 | `todo`・`doing`・`done`・`dropped` |
| `priority` | P | `p1`〜`p5` |
| `due` | 期限 | `2026-10-15` |
| `actor` | (担当の式で使う) | `ai`・`human` |
| `human_gate` | 人の関門 | `none`・`submit`・`pay`・`sign`・`send` |
| `runner` | Agent | `Claude`・`Codex` |
| `workdir` | 起動場所 | `~/src/shop/api` |
| `done_when` | (表には出さない) | 終わりの条件の1文 |

## 設定(config.toml)

- `[new_note]`: 新しいタスク(`a`)を作るとき、`actor`・`human_gate`・`due` を順に聞き、`status = "todo"`・`priority = "p3"` を入れる。
- `[display]`: 列の区切り線と一行おきの色。

## 自分のコマンドにする

自分のタスクのフォルダに `タスク.base` を置き、alias を書けば、ひとことで開けます:

```sh
alias tasks='mdgrid --config ~/notes/mdgrid-tasks.toml ~/notes/タスク.base'
alias mine='mdgrid ~/notes/タスク.base --view 人だけ'
```
