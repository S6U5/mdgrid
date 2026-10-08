# mdgrid の機能紹介

機能ごとに、今のターミナルの中で mdgrid を動かして見せるスクリプト。いちばん下の行に字幕が出る。
要るのは python3 と cargo だけ(tmux は要らない。macOS と Linux)。見本(`examples/`)の写しと専用の設定で動くので、
本物の設定とノートには触らない。終わったあとはそのまま触れて、`exit`(か Ctrl+D)で終わると写しを消す。
字幕は今は日本語だけ。リポの根で動かす:

```sh
python3 demos/all.py                  # 全部をひと続きで(約8分。TOUR_SPEED=2 で半分)
python3 demos/all.py places themes    # 選んだものだけ(python3 demos/all.py --list で名前)

python3 demos/places.py      # 登録した表を切り替えて開く(2026-10-08)
python3 demos/new_note.py    # 新しいノートをフォームで作る(2026-10-07)
python3 demos/export.py      # 表をファイルに書き出す・--print(2026-10-07)
python3 demos/values.py      # 型ごとに値を入れて保存する(リスト・日付・数・チェック・タグ・日時)
python3 demos/themes.py      # 色のテーマを切り替える(7つ)
```

- 速さ: `TOUR_SPEED=2 python3 demos/places.py`(倍の速さ)
- 英語の画面: `TOUR_LANG=en_US.UTF-8 python3 demos/export.py`(字幕は日本語のまま)

新しい紹介は `tour.py` の `Tour` を使って `TITLE`・`setup`・`steps` を書き(書き方は tour.py の先頭)、all.py の `SECTIONS` に名前を足す。steps は保管庫(examples/vault)で始まる前提で、mdgrid は `mdgrid .` のようにフォルダを渡して起動する(登録した表があると、引数なしでは一覧が出るため)。ファイル名は Python の標準のモジュール名(types・json など)と重ねない(取り込みが壊れる)。
確かめは、tmux を端末の代わりにして流し、画面を読む:
`tmux new -d -s t -x 110 -y 30 'TOUR_SPEED=3 python3 demos/places.py'` → `tmux capture-pane -p -t t`。
