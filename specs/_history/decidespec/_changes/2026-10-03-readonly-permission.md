---
type: Change
id: 01M408QH0RQPSHPASST59EQBTX
title: 書き込みの権限が無いノートを読むだけにする(readonly-permission)
status: done
size: full
created: 2026-10-03
updated: 2026-10-03
---

# 書き込みの権限が無いノートを読むだけにする(readonly-permission)

## フェーズ

状態は 未 / 進行中 / 済 / 対象外: 理由。要求・実装・照合は対象外にしない。前のフェーズが済か対象外になるまで、次を始めない。

| フェーズ | 状態 | 成果物 |
|---|---|---|
| 要求 | 済 | この記録の「要求」 |
| 不明点 | 済 | この記録の「不明点と仮定」 |
| 仕様化 | 済 | [readonly-permission](../_decisions/2026-10-03-readonly-permission.md)(人) |
| 設計 | 済 | この記録の「設計」 |
| タスク | 済 | この記録の「タスク」 |
| 実装 | 済 | コミット |
| 照合 | 済 | この記録の「照合」 |

## 要求

docs/todo.md の F-2。書き戻しは一時ファイルからの名前の変更で置き換えるので、利用者が書き込みを禁じたノート(`chmod 444` など)も書き換えてしまう。人の判断(2026-10-03)で、読むだけにして理由を出す(WB-5)。ためたあとに権限が無くなったら、保存で止めて知らせ、ためた変更を残す(WB-14)。

終わりの条件: `chmod 444` のノートの status のセルが読むだけで「書き込めない権限」と理由が出る。status をためたあとに外で `chmod 444` → 保存で止まり、ファイルの中身と権限は変わらず、未保存の数は減らない。docs/safety.md・ja.md の 0444 の記述が新しい振る舞いになる。関係する要件: WB-5・WB-14・WB-8・CE-10・WB-19。

## 不明点と仮定

- 「書き込みの権限が無い」の判定: 利用者がそのファイルに書けるか(unix では access(2) の W_OK に当たる判定。root でも 0444 は書かない、とはしない)。仮定: Rust の std で判定できる範囲(`metadata.permissions().readonly()` は「誰も書けない」だけを見るので、持ち主の書き込みの bit で判定する)。外れたら: libc の access を使う。
- 設計の制約: 読み取りの層の読むだけの理由(src/frontmatter.rs の ReadOnly)に種類を足すと、錠のある tests/test_safety_docs.rs(錠は WB-3・WB-5)の網羅の match を書き換えることになり、この決定(WB-5 だけ)では錠を掛け直せない。権限はハードリンクと同じくファイルの性質なので、読む側(Source)の読むだけの理由として持つ。安全の文書には「Read-only notes」の節の外(書き方の節)に書く。
- 仮定: ディレクトリに書けないが、ファイルには書ける場合は今どおり(保存で一時ファイルが作れず止まる)。

## 設計

- 読み込み(src/source/markdown.rs など、ノートごとの読むだけの判定の所)で、書き込みの権限が無いノートの行を読むだけにし、理由「書き込めない権限」を持たせる(セルを選ぶと下の帯に出る。一括の設定(CE-10)では飛ばして数に入る)。
- 保存(src/writeback.rs の save_with)で、書く直前に権限を確かめ、無ければ書かずに止める(新しい SaveError の種類か、既存の止め方)。止まった行のためた変更は残す(WB-14)。読み直し(WB-16)で権限の変化を拾う。
- docs/safety.md・ja.md の 0444 の段を新しい振る舞いに書き直す(「Read-only notes」の節の外)。
- 却下: ReadOnly に NoPermission を足す(上の錠の制約)。
- 検証: 新しい試験 tests/test_readonly_permission.rs、`./ci.sh`。

## タスク

| # | タスク | 要件 | 完了の条件 | 範囲 | 受け入れ(テスト名) | 状態 |
|---|---|---|---|---|---|---|
| 1 | 書き込みの権限が無いノートを読むだけにし、保存で止め、文書を直す | WB-5 | 例のとおりに動き、試験が通る | src/source/markdown.rs, src/source.rs, src/writeback.rs, src/changes.rs, src/vault.rs, src/ui/review.rs, docs/safety.md, docs/safety.ja.md, tests/test_readonly_permission.rs | test_wb_5_permission_*(tests/test_readonly_permission.rs) | 済 |

## 実装の気づき

- 判定(仮定の更新): std には getuid が無く「持ち主が自分か」を mode の bit だけでは決められないので、`writeback::can_write` で「書き込みに開けるか」(`File::options().write(true).open`。書かずに閉じるので中身も更新時刻も変わらない)を見る。断られた理由が PermissionDenied・ReadOnlyFilesystem のときだけ書けないとする。持ち主・グループ・ACL・読むだけのファイルシステムを OS がまとめて判定し、root は 0444 でも書ける(仮定のとおり)。libc は足していない。
- 読み込み: `vault::Note` に `writable` と権限の印 `perm`(mode・uid・gid)を足し、読むたびに判定する。`Markdown::get` はハードリンクの次に行全体の lock「書き込めない権限」を出す(`writeback::NO_PERMISSION`)。Changes::set・set_many は今の lock の仕組みでそのまま飛ばす(src/changes.rs は変えていない)。
- 保存: `writeback::save_with` が最初の検査(ハードリンクの次)と名前を変える直前の再検査で権限を確かめ、無ければ新しい `SaveError::NoPermission` で止める。Changes::save は Saved 以外の行のためた変更を残すので WB-14 はそのまま満たす。保存の確認の画面(src/ui/review.rs)は「読むだけ(書き込めない権限)」と出す。
- 読み直し(WB-16): chmod・chown は更新時刻も大きさも変えないので、今の見回り(stamp の比較)では拾わなかった。`Vault::poll` で stamp が同じでも `perm` が変われば `writable` を見直すようにした。中身は変わらないので changed には入れない(入れるとためた変更のある行が「外で変更」になる)。セルの lock は毎回 `writable` から作るので次の描画で出る。ACL だけの変化は印に出ないので拾えないが、保存で止まる。

## 照合

書込なしのフレッシュ文脈の照合役で照合した(2026-10-03、コミット済みの clone で)。

| 要件 | 結果 | 証拠 |
|---|---|---|
| WB-5 | checked | コード: src/writeback.rs の can_write・NO_PERMISSION・save_with の refuse_no_permission(最初と置き換える直前の2回)・SaveError::NoPermission、src/source/markdown.rs の行の lock、src/vault.rs の writable と poll / テスト: tests/test_readonly_permission.rs::test_wb_5_permission_*(4本) / 今: 通った(3回とも 4 passed) / 前: 落ちた(500f91e^ で4本が「読むだけにならない」「書けてしまう」で落ちた)。変異: lock を外す → 2本、両方の検査を外す → 2本が落ちた |

既存のテストの削除・skip・弱体化: なし(試験の差分は新しいファイルの追加だけ)

残した気づき: 保存の2回の権限の検査は片方ずつ外しても試験は通る(最初の検査と置き換えの間に権限が無くなる場合の試験が無い)。Vault::poll で権限の変化を拾う所は試験なし(コードを読んで確かめた)。tests/test_readonly_permission.rs は一時フォルダの名前が並列でぶつかりうる(錠は CE-10・WB-5 で、直すには両方に触る人の決定が要る)。ACL・chflags の変化は見回りで拾わない(保存で止まる)。

確かめた: 1 / 1
