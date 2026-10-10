# Feature tours / 機能紹介の録画

Scripts for [VHS](https://github.com/charmbracelet/vhs) that record mdgrid's features as GIFs. Each `.tape` runs mdgrid on a copy of `examples/` in a temporary folder, with its own home, config and state, so nothing outside that folder is touched. The recordings go to `demos/out/`, which is not committed. Each release records them in CI (`.github/workflows/demos.yml`) and attaches them to the release: see the [latest release](https://github.com/S6U5/mdgrid/releases/latest).

[VHS](https://github.com/charmbracelet/vhs) の台本で、mdgrid の機能を GIF に録る。どの台本も、一時フォルダに写した `examples/` と、その中のホーム・設定・状態で mdgrid を動かすので、そのフォルダの外には触らない。録画は `demos/out/` に出て、コミットしない。リリースのたびに CI で撮り(`.github/workflows/demos.yml`)、リリースに添付する(最新のリリースのページで見られる)。

Try it by hand (a copy of `examples/` with its own config, removed when you quit) / 手で試す(写した見本と専用の設定で開き、終わると消す):

```sh
sh demos/try.sh               # relation map sample (press R) / 関係マップの見本(R を押す)
sh demos/try.sh workspace     # workspaces sample (-w Work) / ワークスペースの見本
sh demos/try.sh demo          # English sample table / 英語の見本の表
sh demos/try.sh ai-human      # human and AI tasks (.base) / 人と AI のタスク
sh demos/try.sh showcase      # most features on / 多くの機能を入れた見本
sh demos/try.sh vault         # Japanese sample vault / 日本語の見本の保管庫
```

```sh
brew install vhs              # VHS(ttyd と ffmpeg も入る)
sh demos/record.sh            # 全部を撮る
sh demos/record.sh places     # 1本だけ
```

| Tape / 台本 → GIF | Shows / 見せること |
|---|---|
| `values.tape` → [demo-values.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-values.gif) | 型ごとの入力(リスト・日付・数・チェック・タグ)と保存の確認 |
| `new-note.tape` → [demo-new-note.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-new-note.gif) | 新しいノートのフォーム(雛形の値・必須の欄・隠して入れる作成日) |
| `export.tape` → [demo-export.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-export.gif) | 絞った表を CSV に書き出す・画面なしの `--print` |
| `places.tape` → [demo-places.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-places.gif) | 登録した表を一覧から切り替えて開く・この表を登録する |
| `relations.tape` → [demo-relations.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-relations.gif) | リレーション: リンクを名前で見せる・行き先のノートから選ぶ・行き先を開く・つながった行 |
| `relmap.tape` → [demo-relmap.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-relmap.gif) | 関係マップ(ER 図): 表の箱とつながりの矢印、タブとキーで切り替え |
| `workspace.tape` → [demo-workspace.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-workspace.gif) | ワークスペース: 表をまとめ(`--add-to`)、一覧(`--workspaces`)、`-w` で開き、リンクと関係マップはその表で |
| `view-tabs.tape` → [demo-view-tabs.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-view-tabs.gif) | 設定の画面の「ビュー」の区画でタブを整える(隠す・順・既定・切り替えの案内) |
| `look.tape` → [demo-look.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-look.gif) | 設定の画面の「見た目」の区画でテーマと組を選び、テンプレートとして保存する |
| `tree.tape` → [demo-tree.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-tree.gif) | 親子のノートを字下げして並べ、`Z` で畳む |
| `wbs.tape` → [demo-wbs.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-wbs.gif) | WBS: 番号と進み具合、値の対応表を設定の画面で作る |
| `themes.tape` → [demo-themes.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-themes.gif) | 色のテーマ(examples/themes/ の6つ) |

To add a tour, copy a tape, keep `Source demos/setup.tape` after the settings, and start mdgrid with a folder (`mdgrid .`). The screens are in English on the English sample (`examples/demo`). The font is BIZ UDGothic, a monospace font that also has Japanese glyphs.

足すときは台本を写し、設定の後に `Source demos/setup.tape` を置き、mdgrid はフォルダを渡して起動する(`mdgrid .`)。画面は英語で、見本は英語の `examples/demo`。字の形は日本語も含む等幅の BIZ UDGothic。
