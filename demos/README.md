# Feature tours / 機能紹介の録画

Scripts for [VHS](https://github.com/charmbracelet/vhs) that record mdgrid's features as GIFs. Each `.tape` runs mdgrid on a copy of `examples/` in a temporary folder, with its own home, config and state, so nothing outside that folder is touched. The recordings go to `demos/out/`, which is not committed: upload a GIF where you need it (a pull request, an issue or a release) instead of adding it to the repository.

[VHS](https://github.com/charmbracelet/vhs) の台本で、mdgrid の機能を GIF に録る。どの台本も、一時フォルダに写した `examples/` と、その中のホーム・設定・状態で mdgrid を動かすので、そのフォルダの外には触らない。録画は `demos/out/` に出て、コミットしない(要るところ、PR・Issue・リリースに貼る)。

```sh
brew install vhs              # VHS(ttyd と ffmpeg も入る)
sh demos/record.sh            # 全部を撮る
sh demos/record.sh places     # 1本だけ
```

| Tape / 台本 | Shows / 見せること |
|---|---|
| `values.tape` | 型ごとの入力(リスト・日付・数・チェック・タグ)と保存の確認 |
| `new-note.tape` | 新しいノートのフォーム(雛形の値・必須の欄・隠して入れる作成日) |
| `export.tape` | 絞った表を CSV に書き出す・画面なしの `--print` |
| `places.tape` | 登録した表を一覧から切り替えて開く・この表を登録する |
| `themes.tape` | 色のテーマ(examples/themes/ の6つ) |

To add a tour, copy a tape, keep `Source demos/setup.tape` after the settings, and start mdgrid with a folder (`mdgrid .`). The screen is Japanese (`Env LANG`); set `Env LANG "en_US.UTF-8"` for English. The font is BIZ UDGothic, a monospace font with Japanese glyphs.

足すときは台本を写し、設定の後に `Source demos/setup.tape` を置き、mdgrid はフォルダを渡して起動する(`mdgrid .`)。画面は日本語(`Env LANG`)。字の形は日本語を含む等幅の BIZ UDGothic。
