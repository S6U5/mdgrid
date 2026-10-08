# Feature tours / 機能紹介の録画

Scripts for [VHS](https://github.com/charmbracelet/vhs) that record mdgrid's features as GIFs. Each `.tape` runs mdgrid on a copy of `examples/` in a temporary folder, with its own home, config and state, so nothing outside that folder is touched. The recordings go to `demos/out/`, which is not committed. Each release records them in CI (`.github/workflows/demos.yml`) and attaches them to the release: see the [latest release](https://github.com/S6U5/mdgrid/releases/latest).

[VHS](https://github.com/charmbracelet/vhs) の台本で、mdgrid の機能を GIF に録る。どの台本も、一時フォルダに写した `examples/` と、その中のホーム・設定・状態で mdgrid を動かすので、そのフォルダの外には触らない。録画は `demos/out/` に出て、コミットしない。リリースのたびに CI で撮り(`.github/workflows/demos.yml`)、リリースに添付する(最新のリリースのページで見られる)。

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
| `themes.tape` → [demo-themes.gif](https://github.com/S6U5/mdgrid/releases/latest/download/demo-themes.gif) | 色のテーマ(examples/themes/ の6つ) |

To add a tour, copy a tape, keep `Source demos/setup.tape` after the settings, and start mdgrid with a folder (`mdgrid .`). The screens are in English on the English sample (`examples/demo`). The font is BIZ UDGothic, a monospace font that also has Japanese glyphs.

足すときは台本を写し、設定の後に `Source demos/setup.tape` を置き、mdgrid はフォルダを渡して起動する(`mdgrid .`)。画面は英語で、見本は英語の `examples/demo`。字の形は日本語も含む等幅の BIZ UDGothic。
