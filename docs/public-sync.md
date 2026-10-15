# 手元のリポと公開リポの行き来(public-sync)

mdgrid は、手元のリポ(作業の正本。履歴を公開しない)と、公開リポ S6U5/mdgrid の2つにある。2つの履歴はつながっていない。手元の履歴の承認の証拠(`approval-evidence`)は会話の記録の場所を指すので、履歴ごと公開しない(decidespec の OM-17)。公開リポには、手元の中身を伏せ字にして書き出す。公開リポだけで入った変更(クラウドのセッションの PR、Dependabot など)は、手元へ取り込む。

この行き来は `scripts/public_sync.py` でする。手元のリポで動かす。公開リポの側(クラウドのセッション)では、ふだんどおりブランチと PR で作業すればよく、この道具は要らない。

## 決まり

- 公開リポは、手元では remote `github` として取ってくるだけにする。手元のブランチ(main を含む)を `github` へ push しない。pre-push のフックが、手元の履歴を含む push と伏せ忘れを止める。
- 公開リポの main の変更は、手元の main に取り込むまで「未反映」。書き出しの前に取り込む(道具は、未反映があると書き出さない。書き出すと消えるため)。
- 書き出しは、手元の main の木を伏せ字にしたものを、公開リポの main の上の1つのコミットにし、`export/<名前>` のブランチへ push する。PR のマージは人が公開リポでする(squash でよい)。force push と履歴の書き換えはしない。
- 取り込みは `git cherry-pick -x`。`-x` が付ける `(cherry picked from commit <sha>)` の行が、取り込み済みの印になる。取り込みのコミットの文は、cherry-pick の文のままでよい。
- 伏せ字にする所は、公開しない新しいものを公開リポに足すときに、手元で `〔伏せ字: 理由〕` に置き換えてから書き出す(人の言い回しなど)。道具が機械で伏せるのは下の「伏せ字」のものだけ。

## 伏せ字

道具は、書き出す木の全部のテキストのファイルで、次を置き換える。

- 承認の証拠の場所(`claude:<ID>/<ID>`、`codex:` も)→ `〔伏せ字: 会話の記録の場所〕`
- git の設定 `publicsync.redactPath` に書いた手元のパスで始まる語 → `〔伏せ字: 手元のパス〕`
- git の設定 `publicsync.redactWord` に書いた手元の名前 → `〔伏せ字: 手元の名前〕`

書き出す前と push の前に、置き換えた後の木に、承認の証拠の ID・`redactPath`・`redactWord`・`publicsync.forbid` が残っていないかを確かめ、残っていれば止める。手元のパスや名前は公開しないので、この道具とこの文書には書かず、手元の `.git/config` にだけ置く。

## 最初の1回(済んでいれば要らない)

```sh
git remote add github https://github.com/S6U5/mdgrid.git
git config --add publicsync.redactPath '<手元のラボの根のパス>'   # いくつでも
git config --add publicsync.redactWord '<手元のラボの名前>'
git config --add publicsync.forbid '<出したくない語>'
git fetch github
python3 scripts/public_sync.py init --local <公開リポの main と中身が揃う手元のコミット> --github github/main
python3 scripts/public_sync.py install-hook
```

印は2つ。`refs/public-sync/github` は、公開リポの main のうち手元に反映済みと数えた所まで。`refs/public-sync/local` は、最後に書き出してマージされた手元のコミット(未書き出しの一覧の起点)。どちらも手元の `.git/` の中だけにある。

## ふだんの手順

1. `python3 scripts/public_sync.py status`: 公開リポを取ってきて、印を進め、まだ取り込んでいない公開リポの変更、まだ書き出していない手元のコミット、今書き出すと変わる量を出す。
2. 取り込みがあれば、手元の main で `python3 scripts/public_sync.py import`。ぶつかったら直して `git cherry-pick --continue` し、もう一度 `import`。`specs/` の生成の区画と `specs/test-locks.json` は手元の側を取り、`check.py --write` で作り直す(公開リポの側は手元の履歴が無いので、生成の区画が手元と違う)。誤って入った物(`__pycache__/` など)は取り込みのコミットから外す。作業ツリーに別の作業の未コミットの変更があるときは、それに触らないように進める(git は、上書きになる cherry-pick を自分で止める)。
3. 書き出すときは `python3 scripts/public_sync.py export <名前> --dry-run` で変わるファイルを見てから、`python3 scripts/public_sync.py export <名前> -m '<PR の題>' -m '<本文>' --pr`。`--pr` は PR が無ければ作る。直しを足すときは、手元の main を直して同じ名前でもう一度 `export` する(同じブランチに次のコミットとして積む)。
4. 人が公開リポで PR をマージしたら、次の `status` で印が進む。

まだ手元の main に入れていないブランチを書き出すときは `--from <ブランチ>`。そのブランチは手元の main を含んでいること(取り込みを消さないため)。

## 伏せ字のままの所と、公開リポにだけある所

- 公開リポの過去の履歴(マージ済みのコミット)には、クラウドのセッションが書いた承認の証拠の ID が残ることがある。履歴は書き換えない。次の書き出しで、今の木からは消える。
- 公開リポの `specs/` の生成の区画は、書き出しのたびに手元の版に揃う。公開リポの側で `check.py --write` を回すと、また手元と違う中身になる(取り込みのときに手元の側を取る)。
