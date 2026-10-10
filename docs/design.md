# 設計(how)

仕様(`specs/`)は何をするか(what)だけを書く。どう作るか(how)はここに書く。ここは提案を経ずに直してよいが、仕様と食い違ったら仕様が正しい。

## 最初に決めること(cellops で後から直した点を繰り返さない)

cellops(自作のタスクの TUI)は、レイアウトを5回作り直し、描画の部品(ratatui)を依存に入れたまま使わずに外し、その場の編集・外クリックでの確定・書き込みの範囲を後から足した。mdgrid では、次のことを最初のコミットで決める。

- **描画の部品**: ratatui 0.30 系(crossterm)を実際に使って決める。表は `Table`・`TableState` を使わず、幅に合わせて切り詰めた行を自前で作って `Paragraph` で描く(横スクロールと幅の計算を1か所にまとめるため。タスク 9 で決めた)。全角の既知の不具合(ratatui の CJK の Issue 群)を避けるため、セルの文字は自前で幅に合わせて切り詰めてから渡す。
- **Pane の契約**: 画面の各部分は `(矩形, 共有の文脈, 自分の状態) → (行の列, クリックの領域)` の純関数にする。状態は直接書き換えず「意図」を返し、真ん中で判定する(cellops pane-api-v1)。1つのファイルに状態・入力・書き込みを集めない(cellops の app.rs は 5830 行)。
- **キーの表**: キー・表示名・効く条件・ヘルプの節・下の帯での順位・振り分け先を1つの表に持つ(SR-4)。cellops は表示だけを表から作り、振り分けは別の match に書いていてずれうる。
- **マウス**: ボタンイベントの追跡(`?1002h`)と SGR(`?1006h`)を使い、ドラッグ中は掴んだ部分が入力を捕まえる。
- **描画**: 同期出力(mode 2026)で包み、たまったイベントを全部さばいてから1回だけ描く。最下行と右端の1桁は空ける(端末の自動折り返しを避ける)。

## 全体の形(v1-full-spec の設計)

1つの cargo のパッケージに、ライブラリ(`src/lib.rs`)と実行ファイル(`src/main.rs`)を持つ。ライブラリは画面に依存しない核で、テストは主にここに書く。

| モジュール | 役割 | 主な要件 |
|---|---|---|
| `vault` | ノートの集まりを読む。根の決め方、重なりとリンクの重複の除去、変化の検出(更新時刻と大きさのポーリング) | BV-1・BV-2・BV-9・BV-11 |
| `frontmatter` | フロントマターを読む(値と型)、第1階層のキーの値のバイトの範囲を求める、読むだけにする理由を判定する | CE-8・WB-5 |
| `types` | `.obsidian/types.json` と推定による列の型、複数の根の食い違い | CE-2・BV-12 |
| `base` | `.base` を読む(saphyr)、知らないキーを残す、ビュー | BV-3〜BV-8・BV-13 |
| `expr` | 式の字句解析 → 構文木 → 評価。未対応は構文木の段で分かる | BV-6・BV-7 |
| `changes` | ためる変更(ノートとキーごとの新しい値)、取り消しとやり直しの積み、差分の生成 | WB-9・WB-10・CE-10 |
| `writeback` | 保存: 基準の検査 → 範囲の置き換え → 一時ファイル → 読み直しの検査 → fsync → 直前の再検査 → 名前の変更 → 基準の更新 | WB-1〜WB-8・WB-12・WB-14、読むだけの起動で書かないこと(WB-15。要件は cli/spec.md にある) |
| `keymap` | 動作の一覧(1か所の表)、既定のキー(vim 風・矢印・Bases の主なキー)、設定での割り当て直し | SR-4・SR-13 |
| `ui`(実行ファイル側) | Pane ごとの純関数の描画、入力 → 意図 → 状態の更新、ヘルプ・パレット・差分の画面・詳細の表示 | screen・navigation |
| `clipboard` | OSC 52 と OS のクリップボード(失敗しても止めない) | OUT-1 |
| `config` | TOML の設定、見た目の状態(XDG の state) | CLI-3・SR-11・SR-12 |
| `print`(`src/print.rs`) | 画面を出さずにビューの表を組み(`.base` は `Base::build`・`Base::cell`、mdgrid のビューは `views::to_base` から作った `.base` に `settings::apply`、既定の表は `default_grid`)、csv(RFC 4180)・json・md に手で書く(依存を足さない)。印を付けない素の値、評価できない列は空か null と列ごとに1行の警告。json の鍵は見出しで、displayName が前の列と重なれば `見出し (列の id)`(それでも重なれば `#列の番号` を添える)。csv の1列だけの表の空の値は `""`(空行にしない)。md は `\` を `\\` にしてから縦棒を `\|` にする。画面と共有する純関数(`today_now`・`val_text_with`・`val_value`・`kind_in`・`synth`、`--pick <列>` の `plain`・`one_line`)もここに置き、`src/ui` はこれを呼ぶ | CLI-5・SR-10・BV-7 |

**依存(候補。入れる前に配布元・保守・既知の脆弱性を確かめ、`cargo deny` を CI に置く)**: ratatui と crossterm(描画と入力)、saphyr(`.base` と値の読み取り)、serde と toml(設定)、clap(引数。derive で定義を1つにし、同じ定義から clap_complete で補完、clap_mangen で man を作る。CLI-13。2026-10-03 に入れた: clap 4.6・clap_complete 4.6・clap_mangen 0.3、いずれも clap-rs の配布で MIT OR Apache-2.0)、unicode-width と unicode-segmentation(幅)、sha2(ハッシュ)、similar(差分の表示)、arboard(OS のクリップボード。任意の機能にする)、dirs(置き場所)。フロントマターの範囲の特定は自前にし、YAML の編集の部品(yamlpatch・yaml-edit)は使わない(第1階層のスカラーとフローのリストだけなら自前で範囲を決められ、依存と書式の揺れを減らせる。対応しない形は読むだけにする)。

**ためる変更と保存の流れ**: 確定した値は `changes` に (ノートのパス, キー) → 新しい値 として入り、取り消しの積みにも1手として入る(一括の設定は1手に複数の変更)。表は元の値の代わりにためる値を色と `*` の印で見せる(SR-15)。保存(Ctrl+S・`:w`)で、ファイルごとに「今のファイルに範囲の置き換えを当てた結果」と今のファイルの差分(similar)を画面に出し、確定で `writeback` に渡す。ファイルごとに成功・失敗を出し、失敗したノートの変更はためたまま残す(WB-14)。

**読み直しとためる変更がぶつかるとき**: 外でノートが変わったとき(BV-9)、そのノートにためる変更が無ければ読み直す。ためる変更があれば、表の値は読み直すが、ためる変更は残して行に「外で変更」の印を付ける。保存では WB-4 の検査で止まり、利用者が差分を見て、もう一度保存するか捨てるかを選ぶ。

**`$EDITOR` で開いたノート**: 戻ったら読み直す(SR-8)。そのノートにためる変更があれば、上と同じ「外で変更」の扱いにする。

## 読み込み口(形式を差し替える形)

将来、フロントマター付きの Markdown 以外(CSV・JSON・YAML・SQLite など、ほかの表の道具が扱う形式)も同じ画面で編集できるようにする。形式ごとに違うのは「何を開くか」と「どう読み込み・どう書き戻すか」だけで、表・編集・ためる変更・差分を見て保存・画面は共通の核にする。そのため、核は読み込み口(`Source`)を通してだけ形式に触れる。

```rust
// src/source.rs(核が形式に触れる唯一の口)
pub struct RowId(pub String);            // 行の識別(Markdown ならノートのパス)
pub trait Source {
    fn rows(&self) -> Vec<RowId>;
    fn columns(&self) -> Vec<String>;
    fn get(&self, row: &RowId, col: &str) -> Cell;          // 値・型・書けるか(書けないなら理由)
    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError>; // 保存前の差分用(前, 後)
    fn save(&mut self, row: &RowId, edits: &[Edit]) -> Result<(), SaveError>;
    fn changed(&mut self) -> Vec<RowId>;                    // 外での変化(ポーリング)
}
```

最初の読み込み口は `markdown`(フロントマター付き Markdown のフォルダ。下の frontmatter と writeback を使う)だけ。ほかの形式は次の段で、1つの形式 = 1つの読み込み口として足す。書き戻しの約束(直した値の範囲だけを変え、他は1バイトも変えない。外の変更で止める)は、どの読み込み口も守る。

## 核の公開のインターフェース(縦切り: タスク 2・3)

テストを書く役は、実装を見ずにこの形だけを使う。

```rust
// src/frontmatter.rs
pub enum Value { Null, Bool(bool), Int(i64), Float(f64), Str(String), List(Vec<Value>), Other }
pub enum ReadOnly {            // 読むだけにする理由(CE-8・WB-5)
    NoFrontmatter, EmptyFrontmatter, Bom, NotUtf8, MixedNewlines, DuplicateKey(String), Unclosed, InvalidYaml,
    HardLink,                  // タスク 15。parse は返さず、save がハードリンクのあるノートで返す(WB-5)
}
pub enum Shape { Plain, SingleQuoted, DoubleQuoted, FlowList, BlockList, BlockScalar, Nested, Anchor }
pub struct Entry { pub key: String, pub value: Value, pub shape: Shape,
                   pub span: Option<std::ops::Range<usize>> }   // 値のバイトの範囲。書き換えられない形は None
pub struct Frontmatter { pub entries: Vec<Entry>, pub end: usize } // end = 閉じの区切りの行の先頭のバイト位置
pub fn parse(bytes: &[u8]) -> Result<Frontmatter, ReadOnly>;

// src/writeback.rs
pub enum NewValue { Null, Str(String), Bool(bool), Int(i64), Float(f64) }
pub struct Edit { pub key: String, pub value: NewValue }
pub enum EditError { ReadOnly(ReadOnly), NotEditable(String), Newline, Verify(String) }
/// 元のバイト列に編集を当てた結果を返す(ファイルには触らない)。WB-1・WB-3・WB-6・WB-7。
pub fn apply(original: &[u8], edits: &[Edit]) -> Result<Vec<u8>, EditError>;
pub struct Baseline { pub mtime: std::time::SystemTime, pub len: u64, pub hash: [u8; 32] }
pub fn baseline(path: &std::path::Path) -> std::io::Result<Baseline>;
pub enum SaveError { Changed, Edit(EditError), Io(std::io::Error) }
/// 基準の検査 → apply → 一時ファイル → 読み直しの検査 → fsync → 直前の再検査 → 名前の変更。新しい基準を返す。WB-4・WB-8・WB-12。
pub fn save(path: &std::path::Path, base: &Baseline, edits: &[Edit]) -> Result<Baseline, SaveError>;
```

## 核の公開のインターフェース(縦切りの続き: タスク 4・18・15・7)

テストを書く役は、実装を見ずにこの形だけを使う。上の `Source` の素案を、ためる変更と保存の基準(WB-4・WB-16)が扱える形にしたもの。

```rust
// src/frontmatter.rs(タスク 15 で足す)
pub enum ReadOnly { /* 既存に加えて */ HardLink }       // WB-5。`save` は SaveError::Edit(EditError::ReadOnly(ReadOnly::HardLink)) で返す
// writeback::apply は NewValue::Null を `key:`(コロンの後ろに何も書かない)で書く(CE-9)。キーは消さない。

// src/vault.rs(タスク 4)
/// 渡したフォルダから保管庫の根を決める(BV-2): `.obsidian/` を持つ最寄りの上のフォルダ、無ければ渡したフォルダ。
/// 実体のパス(canonicalize)にし、同じ根・ほかの根の中に入る根は1つにまとめる。読めないパスは Err。
pub fn roots(folders: &[PathBuf]) -> io::Result<Vec<PathBuf>>;
pub struct Stamp { pub mtime: SystemTime, pub len: u64 }
pub struct Note {
    pub path: PathBuf,        // 実体のパス(canonicalize 済み。BV-11 の重複の除去の鍵)
    pub rel: String,          // 表示用: 根からの相対パス('/' 区切り)
    pub bytes: Vec<u8>,       // 最後に読んだ中身
    pub stamp: Stamp,         // 最後に読んだときの更新時刻と大きさ
    pub conflict: bool,       // 同期の道具の競合ファイル(名前に `.sync-conflict-` か `conflicted copy`)。WB-13
    pub found: Vec<PathBuf>,  // 見つけたときの道筋のパス(リンクを辿る前。根の下)。行の絞り込み(BV-1)はこれで見る
    pub links: u64,           // ハードリンクの数(nlink)。2以上なら読むだけ(WB-5)
}
pub struct Progress { pub loaded: usize, pub total: Option<usize>, pub done: bool }
pub struct Poll { pub changed: Vec<PathBuf>, pub added: Vec<PathBuf>, pub removed: Vec<PathBuf> }
pub struct Vault { /* 根と、読んだノート(path の順) */ }
impl Vault {
    /// まだ何も読まない(BV-16: 画面を先に出す)。`.md` を再帰で探す。`.obsidian/`・`.git/`・`.trash/` の下は探さない。
    /// シンボリックリンクは辿り、実体のパスで1つにする(BV-11)。リンクの輪は無視する。
    pub fn open(roots: Vec<PathBuf>) -> Vault;
    /// 最大 `budget` 件を読む。読むたびに `notes()` が増える(BV-16)。
    pub fn load(&mut self, budget: usize) -> Progress;
    /// 読み込みを止める。読んだ分は残り、以後 `load` は何もしない(BV-16)。
    pub fn cancel(&mut self);
    pub fn notes(&self) -> &[Note];               // path の昇順
    pub fn note(&self, path: &Path) -> Option<&Note>;
    /// 更新時刻と大きさを見て、変わったノートだけ読み直す(BV-9)。止めている間(`pause(true)`)は何もせず空を返す。
    pub fn poll(&mut self) -> Poll;
    pub fn pause(&mut self, paused: bool);
    /// 1つのノートを今すぐ読み直す(SR-8・WB-16 の「外の変更の上に書く」)。
    pub fn reload(&mut self, path: &Path) -> io::Result<()>;
    /// 自分で書いたあと、書いたバイトと基準でノートを置き換える(読み直さない。WB-12。その後の外の変更は次の poll で見つかる)。
    pub fn replace(&mut self, path: &Path, bytes: Vec<u8>, stamp: Stamp);
}
/// 読み直しのあとの選択(BV-10): 同じ行が残ればその位置、消えていれば元の位置に最も近い行(同じ添字、はみ出せば末尾)。行が0なら None。
pub fn reselect(old: &[PathBuf], selected: usize, new: &[PathBuf]) -> Option<usize>;

// src/source.rs(タスク 18。核が形式に触れる唯一の口)
pub use crate::frontmatter::Value;
pub use crate::writeback::{Edit, EditError, NewValue, SaveError};
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct RowId(pub String);
pub struct Cell { pub value: Option<Value>, pub lock: Option<String> } // value: None = キーが無い。lock: 読むだけの理由(表示用の短い日本語)
pub struct Stamp { pub mtime: SystemTime, pub len: u64, pub hash: [u8; 32] }   // 保存の基準(WB-4)
/// Stamp.hash と同じ方式(SHA-256)のハッシュ。画面が「見せた差分の元の内容」を覚えるのに使う。
pub fn content_hash(bytes: &[u8]) -> [u8; 32];
pub trait Source {
    fn name(&self) -> String;                                // ヘッダーに出す名前(SR-1)
    fn load(&mut self, budget: usize) -> vault::Progress;
    fn cancel(&mut self);
    fn rows(&self) -> Vec<RowId>;                            // 読んだ行(Markdown では実体のパスの文字列、昇順)
    fn label(&self, row: &RowId) -> String;                  // 行の表示名(Markdown では根からの相対パス)
    fn mark(&self, row: &RowId) -> Option<String>;          // 行の印と案内(WB-13 の競合ファイルなど)。無ければ None
    fn columns(&self) -> Vec<String>;                        // 読んだ行のキーの和(最初に現れた順)
    fn get(&self, row: &RowId, col: &str) -> Cell;
    fn stamp(&self, row: &RowId) -> Option<Stamp>;           // 最後に読んだ内容の基準
    fn reload(&mut self, row: &RowId) -> std::io::Result<()>;
    /// 保存の前の差分用: (今のディスクのバイト, それに edits を当てたバイト)。ファイルも読んだ内容(vault)も変えない(変えると changed が外の変更を見逃す)。
    fn preview(&self, row: &RowId, edits: &[Edit]) -> Result<(Vec<u8>, Vec<u8>), EditError>;
    /// base から変わっていれば SaveError::Changed。書けたら読み直して新しい基準を返す(WB-12)。
    fn save(&mut self, row: &RowId, base: &Stamp, edits: &[Edit]) -> Result<Stamp, SaveError>;
    fn changed(&mut self) -> Vec<RowId>;                     // 外での変化(BV-9 のポーリング)。読み直しも済ませる
    fn pause(&mut self, paused: bool);
}
// src/source/markdown.rs
pub struct Markdown;  impl Markdown { pub fn open(folders: &[PathBuf]) -> io::Result<Markdown>; }  impl Source for Markdown {}
// 行は BV-1 の既定の表: 渡したフォルダの下のノートだけ(根が渡したフォルダより上でも、行は渡したフォルダの下に限る)。
// Cell.lock: ReadOnly の各理由(NoFrontmatter と EmptyFrontmatter を除く)とハードリンク(WB-5)・span の無い値(CE-8)・リストの値(CE-6)・`file.*` と `formula.*` の列(CE-8)。
// フロントマターのあるノートでキーが無いセルは lock なし(WB-3 でキーを足せる)。フロントマターの無いノートと空のフロントマターのノートのセルも lock なし(値は無し。WB-3 で先頭にフロントマターを足す・区切りの間にキーの行を足す)。
// ただし set_add_frontmatter(false)(設定 add_frontmatter = false)では、この2つのセルを lock にする(理由は2つで区別)。save も基準の検査(Changed)のあとで同じ判断をして断る。
// RowId は実体のパスの文字列。UTF-8 でないパスは行に入れない(読むだけでも扱えないため。数は進捗に出さない)。

// src/changes.rs(タスク 7。Source だけを通す)
pub struct Skip { pub row: RowId, pub reason: String }
pub struct Preview { pub row: RowId, pub before: Vec<u8>, pub after: Vec<u8>, pub external: bool }
pub enum Outcome { Saved, Failed(SaveError), Changed }       // Changed: 外で変わって止めた(WB-4・WB-16)
pub struct Changes { /* (行, 列) → 新しい値、行ごとの基準、取り消しとやり直しの積み、外で変更の印 */ }
impl Changes {
    pub fn new() -> Changes;
    /// 1手としてためる。読むだけのセル(Cell.lock)は Err(Skip)。その行に初めてためるとき、src.stamp を基準に取る(WB-4)。
    /// NewValue::Null で、キーの無いセルなら何もしない(SR-18)。
    pub fn set(&mut self, src: &dyn Source, row: &RowId, col: &str, v: NewValue) -> Result<(), Skip>;
    /// 複数の行に同じ値を1手でためる。直せない行は飛ばして返す(CE-10)。
    pub fn set_many(&mut self, src: &dyn Source, rows: &[RowId], col: &str, v: NewValue) -> Vec<Skip>;
    pub fn undo(&mut self) -> bool;                          // WB-10
    pub fn redo(&mut self) -> bool;
    pub fn pending(&self, row: &RowId, col: &str) -> Option<&NewValue>;
    pub fn count(&self) -> usize;                            // 未保存の数 = ためた (行, 列) の数
    pub fn rows(&self) -> Vec<RowId>;
    /// 外で変わった行を受け取り、ためた変更のある行に「外で変更」の印を付ける(WB-16)。
    pub fn note_external(&mut self, rows: &[RowId]);
    /// 読み直した行のためたセルを今の読んだ値と型つきで比べ、同じセルを1手として外す(WB-17)。行のためる変更が無くなれば基準と印も外れる。
    /// note_external は読んだ値を見ないので、画面は外の変化を受けたら(poll・$EDITOR から戻ったあと)続けてこれを呼ぶ。
    /// 外の変化で積む手は、やり直しを消さない。
    pub fn drop_same(&mut self, src: &dyn Source, rows: &[RowId]);
    /// 行ごとの値を1手で当てる。Some(v) は set と同じ決まりでためる、None はためた値を外す(CE-4 の一括の切り替え・CE-10・WB-17)。
    /// set_many はこれで書く。直せない行は飛ばして返す。
    pub fn set_each(&mut self, src: &dyn Source, col: &str, items: &[(RowId, Option<NewValue>)]) -> Vec<Skip>;
    pub fn external(&self, row: &RowId) -> bool;
    /// previews・save・save_rows は、読んだ値と同じセルを書く変更から除く(WB-17 の守り。undo で戻ったセルなど)。
    /// 除いて何も残らない行は差分に出さず、保存では書かずにためる変更から外す(結果には出さない)。
    pub fn previews(&self, src: &dyn Source) -> Vec<Result<Preview, (RowId, EditError)>>;
    /// 行ごとに保存する。Saved の行の変更は消え、基準は新しくなる。それ以外は残す(WB-14)。
    pub fn save(&mut self, src: &mut dyn Source) -> Vec<(RowId, Outcome)>;
    /// WB-16 の「外の変更の上に書く」: 今のファイルを読み直して基準にし、印を消す。
    pub fn overwrite(&mut self, src: &mut dyn Source, row: &RowId) -> std::io::Result<()>;
    /// 画面からの「外の変更の上に書く」: 読み直した内容のハッシュが `seen`(利用者に見せた差分の元の内容)と同じときだけ
    /// 基準を進めて印を消し、Ok(true)。違えば基準も印も変えず Ok(false)(見せていない内容の上に書かない。WB-9・WB-16)。
    /// overwrite・overwrite_seen は読み直したあと drop_same で同じ値のセルを外す(WB-17)。全部外れたら書くものは無く Ok(true)。
    pub fn overwrite_seen(&mut self, src: &mut dyn Source, row: &RowId, seen: [u8; 32]) -> std::io::Result<bool>;
    /// 指定の行だけを保存する(保存の確認の画面で1ファイルを選んで書くとき)。結果の形は save と同じ。
    pub fn save_rows(&mut self, src: &mut dyn Source, rows: &[RowId]) -> Vec<(RowId, Outcome)>;
    /// WB-16 の「ためた変更を捨てる」。その行の変更を1手として消す。
    pub fn discard(&mut self, row: &RowId);
}
```

## 核の公開のインターフェース(残り: タスク 5・6・8)

テストを書く役は、実装を見ずにこの形だけを使う。日付は 1970-01-01 からの日数(`i64`)、日時は地域の時計の秒(`i64`。UNIX 秒に地域の時差を足した物差しで、フロントマターの時刻の付いた値も地域の時刻として同じ物差しで読む)。今日の日付は画面が `MDGRID_TODAY`(`YYYY-MM-DD`)か、時計と起動の最初に決めた地域の時差(`print::set_local_offset`。`time` の `UtcOffset::current_local_offset` は糸が1本のうちにしか答えないので main の先頭で呼ぶ)から決めて渡す。

```rust
// src/types.rs(タスク 5)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind { Text, Number, Checkbox, Date, DateTime, List }
/// `.obsidian/types.json`(`{"types": {"due": "date", ...}}`)を読む。text・number・checkbox・date・datetime・multitext・tags・aliases を
/// Kind に写す(multitext・tags・aliases は List)。知らない型の名前は Text。読めない・壊れているなら空(起動は止めない)。
pub fn read_types_json(bytes: &[u8]) -> HashMap<String, Kind>;
/// Obsidian の推定(CE-2): 最初に見つかった空でない値(Null・空の文字列・空のリストは飛ばす)で決める。
/// `YYYY-MM-DD` の文字列 → Date、`YYYY-MM-DDTHH:MM`(秒は任意)→ DateTime、Int・Float → Number、Bool → Checkbox、List → List、ほか → Text。何も無ければ Text。
pub fn infer<'a>(values: impl Iterator<Item = &'a Value>) -> Kind;
/// 値が列の型に合うか(CV-2)。Null と空の文字列はどの型にも合う。Date は実在する日付だけ。
pub fn fits(kind: Kind, v: &Value) -> bool;
/// 複数の根の types.json を合わせる。同じキーで型が食い違えば Conflict(BV-12)。
pub enum Declared { One(Kind), Conflict }
pub fn merge(per_root: &[HashMap<String, Kind>]) -> HashMap<String, Declared>;
/// 日付の文字列の読み書き(CE-5 と式で使う)。
pub fn parse_date(s: &str) -> Option<i64>;   // 実在しない日付は None
pub fn format_date(days: i64) -> String;

// src/source.rs(タスク 5・6 で足す)
pub struct ColumnKind { pub kind: types::Kind, pub lock: Option<String> }   // BV-12 の食い違いは Text と lock
pub struct FileInfo {
    pub name: String, pub basename: String, pub ext: String,
    pub path: String,     // 根からの相対('/' 区切り)
    pub folder: String,   // path の親(根の直下は "")
    pub size: u64, pub mtime: i64, pub ctime: i64,   // UNIX 秒
    pub tags: Vec<String>,  // フロントマターの tags と本文の `#tag`(先頭の # を除く。入れ子 `a/b` はそのまま)
                            // フロントマターが読めない(ReadOnly)ノートは tags を読まず(値を推測しない)本文だけ。本文の始まりは BOM を除き、`---` で閉じていればその後ろ。コード(フェンス・字下げ4つ・インライン)の中は見ない
}
trait Source {   // 既存に加えて
    fn kind(&self, col: &str) -> ColumnKind;          // types.json があればそれ、無ければ読んだ全行の値から infer(CE-2)
    fn file(&self, row: &RowId) -> Option<FileInfo>;
    fn candidates(&self, col: &str, max: usize) -> Option<Vec<Value>>;  // CE-3: 読んだ全行の異なる値(Null と空は除く)。max を超えたら None
}
impl Markdown { pub fn open_vault(base_file: &Path) -> io::Result<Markdown>; }  // `.base` のとき: 根は BV-2 で決め、行は根の下の全ノート

// src/expr.rs(タスク 6)
#[derive(Clone, Debug, PartialEq)]
pub enum Val { Null, Bool(bool), Num(f64), Str(String), Date(i64), DateTime(i64), Duration(i64 /* ミリ秒 */), List(Vec<Val>) }
pub enum ExprError { Syntax(String), Unsupported(String /* 未対応の関数・メソッド・演算子の名前 */) }
pub struct Expr { /* 構文木 */ }
/// 字句解析 → 構文木。未対応の関数・メソッドはここで Unsupported(BV-7)。
/// 対応(BV-6): 比較 == != < <= > >=、論理 && || !、算術 + - * / %、括弧、文字列('…' と "…")・数・true・false・null・リスト […]、
/// 参照 `status`・`note.status`・`note["a b"]`・`file.name` ほか FileInfo の項目・`formula.x`、
/// 関数 if(c, a, b?)・date(s)・now()・today()・duration(s)・file.hasTag(t…)・file.inFolder(f)・file.hasProperty(p)、
/// メソッド .contains(x)・.containsAll(x…)・.containsAny(x…)・.isEmpty()・.length(プロパティ)・.toString()。
/// 日付 ± 期間の文字列("1d"・"2 weeks"・"3h" など。単位 y M w d h m s とその英語の名前)、日付 − 日付 = 期間。
pub fn parse(src: &str) -> Result<Expr, ExprError>;
pub struct Env<'a> {
    pub prop: &'a dyn Fn(&str) -> Option<Value>,   // ノートのキー(ためた値を重ねたもの。NV-12)
    pub file: &'a FileInfo,
    pub formula: &'a dyn Fn(&str) -> Option<Val>,  // formula.x の値(循環は Null)
    pub today: i64, pub now: i64,
}
/// 評価。型の合わない演算は Null。filters は Val::Bool(true) のときだけ行を残す。
pub fn eval(e: &Expr, env: &Env) -> Val;
pub fn to_val(v: &Value) -> Val;                  // フロントマターの値 → Val(`YYYY-MM-DD` の文字列は Date)

// src/base.rs(タスク 6)
pub fn parse(text: &str) -> Result<Base, String>;      // YAML として読めない → Err(理由)。知らないキーは無視(BV-8)
pub enum Dir { Asc, Desc }
pub struct View { pub kind: String, pub name: String, pub order: Vec<String>, pub sort: Vec<(String, Dir)>, pub group_by: Option<(String, Dir)>, pub limit: Option<usize> }
pub struct Base { pub views: Vec<View>, /* 全体の filters・formulas・properties.displayName */ }
pub struct Column { pub id: String, pub title: String }  // id: ノートのキーは素の名前(`note.status` も `status`)、ほかは `file.name`・`formula.x`。title は displayName か既定の見出し(base::default_title。式の名前・`file name` など。BV-24)
pub struct Grid {
    pub columns: Vec<Column>,
    pub rows: Vec<RowId>,
    pub groups: Vec<(String, std::ops::Range<usize>)>,   // groupBy の見出しと rows の範囲(groupBy なしなら空)
    pub notes: Vec<String>,                               // 未対応の列の説明(BV-7。下の帯に出す)
}
pub enum Shown { Prop(Cell), Computed(Val), Unsupported(String) }
impl Base {
    /// ビューを組み立てる(BV-4・BV-5・BV-13)。table 以外(SC-8)・評価できない filters(BV-7)は Err(理由。関数名を含む)。
    /// 並べ替え: sort の順、型の合わない値は型の合う値の後ろ(CV-2)。groupBy はまとまりで並べ、まとまりの中は sort。limit は最後に。
    /// filters の葉は真偽の規則で判定(if と同じ。Null・false・0・空の文字列は偽)。
    pub fn build(&self, view: usize, src: &dyn Source, prop: &dyn Fn(&RowId, &str) -> Option<Value>, today: i64, now: i64) -> Result<Grid, String>;
    pub fn cell(&self, src: &dyn Source, prop: &dyn Fn(&RowId, &str) -> Option<Value>, row: &RowId, col: &str, today: i64, now: i64) -> Shown;
}
/// `.base` なしの既定の表(BV-1): 列は src.columns()、行は src.rows()、グループなし。
pub fn default_grid(src: &dyn Source) -> Grid;

// src/config.rs(タスク 8)
pub struct Config {
    pub keys: Vec<(String /* モード */, String /* キーの表記 "j"・"ctrl+s"・"shift+tab" */, String /* 動作の名前か "none" */)>,
    pub color: bool,            // false で色なし(NO_COLOR と同じ)
    pub candidates: usize,      // CE-3 の上限(既定 20)
    pub poll_ms: u64,           // BV-9(既定 1000)
    pub ambiguous_wide: bool,   // CV-6(既定 false)
    pub search_bar: bool,       // NV-23(既定 true)
    pub date_format: DateFormat, // CE-22(既定 YYYY-MM-DD)
    pub week_start: WeekStart,  // CE-21(既定 Sun。設定の値は "sun"・"mon")
    pub add_frontmatter: bool,  // WB-3(既定 true)。false ならフロントマターの無いノートと空のフロントマターのノートを読むだけ
    pub editor: Option<String>, // SR-8(既定 None。"" も None として読む。文字列でなければ警告して None)
}
/// TOML を読む。知らない項目は警告の文にして返し、止めない(CLI-3)。壊れた TOML は Err(理由1行)。
/// 知る項目かどうかは ITEMS で決める。
pub fn parse(text: &str) -> Result<(Config, Vec<String>), String>;
/// SR-8: 設定の editor > $VISUAL > $EDITOR の空でない最初、無ければ "vi"。値はそのまま(単語分けは ui の editor_argv)。
/// 環境変数は main が読んで渡し、決めた文字列をイベントのループの open_editor に渡す。
pub fn resolve_editor(config: Option<&str>, visual: Option<&str>, editor: Option<&str>) -> String;
/// 設定の項目の表(CLI-11・CLI-12): 1か所に名前・型・既定値の TOML の表記(None = 既定で書かない editor・keys)・
/// 例・英語の説明・日本語の説明を持つ。知らない項目の判定・KEYS・--print-config がこれを使い、
/// 試験が docs/config.md・docs/config.ja.md の見出し(### `name`)と突き合わせる。
/// 文書の各項目の `- Type: `…``・`- Default: `…``(日本語は `- 型:`・`- 既定:`。既定の無い項目は none・なし)と ```toml の例も、
/// 表と同じかを tests/test_config_docs_values.rs で確かめる。editor の空白だけの値は空と同じ。
pub struct Item { pub name: &'static str, pub ty: &'static str, pub ty_ja: &'static str, pub default: Option<&'static str>, pub example: &'static str, pub en: &'static str, pub ja: &'static str }
pub const ITEMS: &[Item];
pub const KEYS: &[&str];                            // ITEMS の名前(const の中で写す)
/// `--print-config` の出力: 英語の説明のコメント付きの TOML。既定の無い項目は例をコメントで出す(読み直すと警告なしで既定と同じ)。
pub fn default_toml() -> String;
pub fn config_path() -> Option<PathBuf>;            // $XDG_CONFIG_HOME/mdgrid/config.toml、無ければ ~/.config/mdgrid/config.toml
/// 見た目の状態(SR-11・SR-12): $XDG_STATE_HOME/mdgrid/(無ければ ~/.local/state/mdgrid/)に、開いた対象のパスとビューの名前ごとの TOML。
pub struct ViewState { pub order: Vec<String>, pub hidden: Vec<String>, pub widths: Vec<(String, u16)>, pub folded: Vec<String>, pub view: Option<String> }
pub fn state_dir() -> Option<PathBuf>;              // $XDG_STATE_HOME/mdgrid、無ければ ~/.local/state/mdgrid
pub fn load_state(dir: &Path, target: &Path, view: &str) -> ViewState;   // 無い・壊れている → 既定(空)
pub fn save_state(dir: &Path, target: &Path, view: &str, s: &ViewState) -> io::Result<()>;  // 一時ファイル → 名前の変更
/// SR-12: 状態の並びと隠す列は今ある列にだけ当て、状態に無い列は右に足す。(並び, 隠す列)を返す。
pub fn apply_state(columns: &[String], s: &ViewState) -> (Vec<String>, Vec<String>);

// src/writeback.rs(タスク 16。試験のための失敗を差し込む口。#[doc(hidden)] で公開)
#[derive(Default)]
pub struct Faults {
    pub corrupt: bool,      // 一時ファイルに書くバイトの、対象の値の1バイトを変える(WB-6 の読み直しの検査が落ちるはず)
    pub fail_write: bool,   // 一時ファイルへの書き込みの途中で io::Error
    pub fail_fsync: bool,   // fsync で io::Error
    pub misquote: bool,     // 文字列の値をクオートせず素のまま書く(文字列の `true` → 真偽値)。編集の結果を作る段で効き、WB-6 の読み直しの検査が落ちるはず
    pub before_rename: Option<Vec<u8>>,  // fsync のあと名前を変える前に、ノートをこの内容で外から書き換える(WB-12 の直前の再検査で止まるはず)
}
pub fn save_with(path: &Path, base: &Baseline, edits: &[Edit], faults: &Faults) -> Result<Baseline, SaveError>;  // save は save_with(.., &Faults::default())
```

## 画面の形(縦切り: タスク 8 の一部・9・11・12)

画面は実行ファイルの側(`src/main.rs` と `src/ui/`)に置き、ライブラリ(核)は ratatui に依存しない。画面のテスト(ゴールデン・キー列)は `src/ui/` の中の `#[cfg(test)]` に書き、ゴールデンの文字列は `tests/golden/*.txt` に置く(`UPDATE_GOLDEN=1` で更新)。描画は ratatui の `TestBackend` に描いて、バッファを行ごとの文字列にして比べる。

| ファイル | 役割 |
|---|---|
| `src/main.rs` | 引数(CLI-1・CLI-2・CLI-4・CLI-11・CLI-13。clap の derive の `Cli` 1つで読み、補完 `--completions <シェル>` と man `--man` も同じ定義から出す。`--help` は clap の英語の案内でなく日本語の `USAGE` を出し、clap の誤りは日本語の理由1行に直す。`--` の後ろはパス。命令の旗が重なったら help > version > print-config > completions > man の順で先のものを取る)、`--print [--format csv|json|md]`(CLI-5。端末の検査の前に分け、ノートを全部読んでから核の `print` で組んで標準出力に出す。`--view` は `.base` のビューを先に、無ければ今の対象の views.toml の mdgrid のビューを探す。`--format` は `--print` が要る)、`--pick path|<列>`(OUT-3。`--print` とは同時に受けない。画面の書き先を `Screen`(標準出力か `/dev/tty`)で分け、`--pick` では `run`・`event_loop`・`enter`・`restore` とパニックのフック・Restore のガードが開いた `/dev/tty` に書く。キーの読み取りと raw は crossterm が標準入力(端末でなければ `/dev/tty`)で行う。`Terminal::clear` は crossterm の `cursor::position` で標準出力にカーソルの位置を問うので、`/dev/tty` では問わない `Terminal::resize` で消す。SR-10 の検査は標準出力を見ず `/dev/tty` が開けるかだけ。読むだけ(WB-15)で開き、列の名前はノートを読み終えてから表の列(見せている列と隠した列の id か見出し)で確かめ、無ければ画面を出さずに理由1行と終了コード 2。結果は画面を戻したあとに標準出力へ出し、BrokenPipe は黙って 0。path は、ノートの実体のパスが引数のフォルダ(`.base` ならそのフォルダ)を正規化した場所の下なら「引数の文字 + 相対」、外なら絶対。列の値は `print::plain` と同じ素の文字で改行は空白(`print::one_line`))、端末の準備と後始末(raw・代替画面・マウス・同期出力、パニックでも戻す)、イベントのループ(たまったイベントを全部さばいてから1回描く。1秒ごとに `Source::changed`) |
| `src/ui/app.rs` | 状態(`App`: Source・Changes・モード・選んだ行と列・列の順と幅・横のスクロール・メッセージ)と、意図(`Intent`)を受けて状態を変える `App::apply`。描画も入力も直接は状態を書き換えない |
| `src/ui/keymap.rs` | キーの表(SR-4): `(モード, キー) → 動作` と、表示名・ヘルプの節・下の帯での順位を1つの `const` の表に持つ。振り分けと下の帯はこの表から作る。全角の英数字と `、`・`・` を半角に直してから引く(SR-17)。割り当て直し(SR-13 の設定)は後のタスク |
| `src/ui/view.rs` | 描画の純関数: `(矩形, &App) → 行の列`。ヘッダー・ビューのタブ・表・下の帯・メッセージ行(SR-1)、セルの見せ方(CV-1・CV-3・CV-4・CV-5・SR-15)、入力ボックス(CE-1)、保存の確認の画面(WB-9・WB-16)と終了の確認(WB-11) |
| `src/ui/width.rs` | 書記素のまとまりで幅を数え、幅に合わせて切り詰める(`…`)。制御文字は `␛` などの見える文字に置き換える(SR-9)。ratatui には切り詰め済みの文字列だけを渡す |

モード: `Table`・`Edit`(1行の入力ボックス)・`Confirm`(保存の差分と確定)・`Quit`(保存する・捨てる・戻る)。縦切りで入れる動作: 移動(`h` `j` `k` `l`・矢印・Home / End・PageUp / PageDown・`gg` `G`・Tab / Shift+Tab)、編集(Enter で入力ボックス。値が読むだけなら理由をメッセージ行に出す。改行を含む値は開かない(CE-12))、Backspace / Delete で空にする(SR-18)、取り消し `u` / Ctrl+Z とやり直し Ctrl+R / `U`(WB-10)、保存 Ctrl+S(WB-9)、終了 `q`(WB-11)、読み込みの中止 Ctrl+G(BV-16)。編集のモードの中: Enter で確定、Esc で取り消し、Ctrl+R で編集前の値、Tab / Shift+Tab で確定して隣へ(CE-11)。型ごとの入り方(CE-3〜CE-7)は後のタスクで、縦切りでは全部テキストとして入れ、数・真偽・null に見える入力も文字列として書く(WB-7 がクオートする)。

列の順は表の側(`App`)が持つ。`Source::columns` の並びが変わっても、既に見せている列の順は変えず、新しい列は右に足す(タスク 18 の気づき)。列の幅は見えている行の値の幅から決め、上限は画面の幅の3割(CV-5)。色: `NO_COLOR` があれば色なし、`COLORTERM` が truecolor / 24bit でなければ 256 色(SR-10・SR-15)。ためる変更のセルは色に加えて `*`、読むだけのセルは薄い表示に加えて選んだときにメッセージ行に理由(SR-15)。

## 画面の形(残り: タスク 8・9・10・11・13・14)

`App` に機能を足し続けると cellops の app.rs(5830 行)と同じになるので、状態と意図の処理を機能ごとのファイルに分ける。`App::apply` は振り分けだけにし、各ファイルは `impl App` の続き(同じ構造体へのメソッド)として書く。1ファイルはおおむね 600 行まで。

| ファイル | 中身 | 要件 |
|---|---|---|
| `src/ui/grid.rs` | 表の行と列の組み立て: `.base` があれば `Base::build`、無ければ `default_grid`。その上に簡易の絞り込み(NV-2)・同じ値の絞り込み(NV-8)を重ね(見出しの並べ替え NV-3 と並べ替えの窓 NV-24 は設定の並べ替えを変える。settings.rs の set_sorts)、ためた値で計算し直す(NV-12: 値を直した行は、保存か移動の操作まで元の位置に留める)。列の順・隠す列・幅(NV-4・SR-12)、グループの開閉(SR-2) | BV-4・BV-5・BV-13・NV-2・NV-3・NV-4・NV-8・NV-12・SR-2 |
| `src/ui/nav.rs` | 移動(NV-7: `:<番号>`・`0` `$`・Ctrl+D / Ctrl+U)、検索(NV-1: `/`・`n` `N`・smartcase・一致の強調)、選択(NV-5: Space・`v`・Shift+矢印・Ctrl+A・Esc)、`*` の強調・`,` の絞り込み(SR-18・NV-8)、`--pick` の選び(OUT-3: `App.choosing`。表の Enter はノートの行なら印の行(無ければ選んでいる行)を `App.chosen` に決めて終わり、見出しの行は開閉のまま。Esc は SR-18 の順に解き、解くものが何も無いときだけ取りやめ。q はふだんの終了で、`chosen` が無いまま終われば取りやめ。エディタ(`e`・名前のクリック)は開かず理由を出す。コピーの OSC 52 は `TtyClipboard` で `/dev/tty` へ) | NV-1・NV-5・NV-7・NV-8・SR-18・OUT-3 |
| `src/ui/detail.rs` | 詳細の表示(NV-6: `K`。全プロパティを縦に、長い値は折り返す。Enter で選んだプロパティの編集、Esc と `K` で閉じる) | NV-6・SR-16 |
| `src/ui/input.rs` | 型ごとの入り方(CE-2〜CE-7): テキストは候補のリスト(CE-3。`Source::candidates`、今の値に印、「なし」、自由入力に切り替え。開いた直後(↑↓ のあとも)に打つと今の値を置き換える: Input の `fresh`)、チェックは切り替え(CE-4)、日付は `YYYY-MM-DD`・`+3`・空(CE-5)、数は数だけ(CE-7)。不正なら閉じずに理由。一括の設定(CE-10: 選んだ行に `Changes::set_many`、飛ばした数と理由) | CE-2〜CE-7・CE-10 |
| `src/ui/help.rs` | ヘルプ(SR-5: `?`。先頭に今のモードで押せるキー、その下に全部)とパレット(SR-14: `:` と Ctrl+P。動作の名前のあいまい検索、割り当てキーを横に、`:120`・`w`・`q`)。どちらもキーの表から作る | SR-4・SR-5・SR-14・SR-16 |
| `src/ui/external.rs` | `$EDITOR`(SR-8: 引数つきを空白で分け、シェルを通さず、パスは1つの引数。端末を戻してから起動し、戻ったら読み直す)、コピー(OUT-1: `y` と Ctrl+C。セル・行・選んだ行を見出しつきのタブ区切りで、OSC 52 と OS のクリップボード(macOS は `pbcopy`、Linux は `wl-copy` か `xclip`。依存の crate は足さない)。片方が失敗しても止めずに知らせる) | SR-8・OUT-1 |

起動(タスク 8): `mdgrid [<.base | フォルダ> …] [--view <名前>] [--readonly] [--no-color] [--config <パス>] [--print-config] [--completions <シェル>] [--man] [--help] [--version]`(CLI-1・CLI-2。`--print-config` は CLI-11 で、`config::default_toml()` を出して終了コード 0。`--completions`・`--man` は CLI-13 で、`clap_complete`・`clap_mangen` が clap の定義から作ったものを出して終了コード 0。どちらも画面を出さないので、標準出力がパイプでも動く)。`.base` を渡したら `Markdown::open_vault`。設定(CLI-3)を読み、警告はメッセージ行に出す。キーの割り当て直し(SR-13)は、設定の `(モード, キー, 動作)` で表を上書きした写しを作る(`"none"` で外す)。起動で開くビューは `.base` の先頭のビュー(`--view` があればそれ。BV-13)で、前に選んだビューは起動に使わない。見た目の状態(SR-11・SR-12)は、ビューごとに、そのビューを開いたときに読んで列の順・隠す列・幅・畳んだまとまりに当て、終了とビューの切り替えで書く。`--readonly`(WB-15)では編集・空にする・保存の動作を表から外し、保存のキーで「読むだけで開いている」と出す。在る `.md` のファイル(拡張子の大文字小文字を問わない)を渡したら(CLI-15)、`main::md_to_folder` が `check_paths` の前にそのフォルダ(親が空なら `.`)に置き換え、`--print`・`--pick` もその置き換えたフォルダを起動の引数とみなす(`--pick path` の形も)。画面では `App::select_after_load` に覚え、読み込みが終わったとき(取りやめを含む)に一度だけその行を選ぶ(途中で選ぶと並べ替えで行が動き画面が跳ねるので、途中では選ばない)。無い `.md` は置き換えず、CLI-4 の理由1行にする。

見せ方(タスク 9 の残り): ビューのタブ(BV-13。`[` `]` か Tab の帯のクリックで切り替え)、未対応の列の薄い `?` と、選んだときの下の帯の説明(BV-7)、型の合わない値の `!`(CV-2)、East Asian Ambiguous の幅(CV-6。設定 `ambiguous_wide`。`width.rs` で ○● などを2と数える)、列の幅の `<` `>` とドラッグ(SR-3・NV-4)。

## ビューの設定(NV-13〜NV-22)

Notion のビューの設定(プロパティ・フィルター・並べ替え・グループ)と Excel のオートフィルター(値の一覧にチェック)に倣う。条件の判定と、行を絞る・並べる・まとめる処理は核の `src/settings.rs` に置き(画面に依存しない。試験は核で書く)、設定は見た目の状態(`config::ViewState.settings`)に覚える。

```rust
// src/settings.rs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CmpOp { Eq, Ne, Lt, Le, Gt, Ge }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Op {
    Keep(Vec<Option<String>>),   // 値の一覧のチェック: 残す値(None は「(空)」)。リストの値は要素のどれかが入れば残す
    Drop(Vec<Option<String>>),   // 隠す値(「done を隠す」)。リストの値は要素のどれかが入れば隠す
    Contains(String), NotContains(String),   // テキスト(大文字小文字を区別しない)
    Cmp(CmpOp, String),          // 数と日付の比較。列の型(Kind)で読む。型の合わない値・空は残らない
    Empty, NotEmpty,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cond { pub col: String, pub op: Op }
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Dir { Asc, Desc }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum Group { #[default] Inherit, Off, By { col: String, dir: Dir, hide_empty: bool } }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Settings { pub filters: Vec<Cond>, pub sorts: Vec<(String, Dir)>, pub group: Group }
impl Settings {
    pub fn is_default(&self) -> bool;
    /// 表の上の帯の項目(NV-16)。1条件 = 1項目。例「status: done を除く」「並べ替え: due ↑」「グループ: 種別」「グループ: なし」
    pub fn chips(&self) -> Vec<String>;
}
/// 値の鍵(表示の文字列)。空(Null・空の文字列・空のリスト・キーなし)は [None]、リストは要素ごと
pub fn value_keys(v: Option<&Value>) -> Vec<Option<String>>;
/// 値の一覧(件数つき。空は None。件数の多い順、同数は文字の順)。リストは要素ごとに数える(NV-19)
pub fn value_counts<'a>(values: impl Iterator<Item = Option<&'a Value>>) -> Vec<(Option<String>, usize)>;
pub fn matches(cond: &Cond, v: Option<&Value>, kind: Kind) -> bool;
/// 行を絞り(全部の条件を満たす行だけ。NV-14・NV-19)、並べ(sorts があれば並べ直す。安定。型の合わない値と空は後ろ)、
/// まとめる(Inherit は渡された groups のまま、Off はまとめない、By は列の値でまとめ直す。NV-15・NV-21)。
pub fn apply(rows: Vec<RowId>, groups: Vec<(String, Range<usize>)>, s: &Settings,
             get: &dyn Fn(&RowId, &str) -> Option<Value>, kind: &dyn Fn(&str) -> Kind)
             -> (Vec<RowId>, Vec<(String, Range<usize>)>);
// src/config.rs: ViewState に `#[serde(default)] pub settings: Settings` を足す(古い状態のファイルも読める)
```

保存の形(NV-17): 状態のファイルの最上位の `settings` の表に書き(既定なら書かない。無ければ既定)、Keep・Drop の値は TOML が None を書けないので `{ value = "..." }` と「(空)」の `{ empty = true }` の表の配列にする。

重ね順(NV-20): `.base` の filters・sort・groupBy(`Base::build`)→ 設定(`settings::apply`)→ 簡易の絞り込み(NV-2)・同じ値(NV-8)→ 直した行の留め(NV-12)。見出しの並べ替え(NV-3)と並べ替えの窓(NV-24)は設定の並べ替えそのものを変え、見た目の状態に残す。計算の列(`file.*`・`formula.*`)は画面の値(`Shown::Computed`)を `Value` に写して渡す。

画面(`src/ui/settings.rs`): モード「ビューの設定」。開くと今の設定の写しを編集し、「反映」で `App` に当てて組み立て直し、見た目の状態に書く(WB-15 では書かない)。「取り消し」・Esc は写しを捨てる。4つの区画(列・フィルター・並べ替え・グループ)を Tab で移り、フィルターは列を選ぶ → 値の一覧(件数つき)にチェック(Space)か、条件の種類を選んで値を打つ。区画の下に「反映」「取り消し」「既定に戻す」のボタン(クリックでも押せる)。キーはすべて keymap の表(SR-4・SR-16)。表の上の帯(`bands.rs`)は、ビューのタブの下に `settings.chips()` を並べ、選んで Backspace(か帯の項目のクリック)でその条件だけを外す(NV-22)。開くキーは表のモードの空いたキー(パレットの動作 `view_settings` からも)。

## リストの値の付け外し(CE-16〜CE-19)

核(画面に依存しない。受け入れの試験を先に置く):

```rust
// src/writeback.rs
pub enum NewValue { /* 既存に加えて */ List(Vec<String>) }   // 要素の文字列の並び。空の Vec は CE-19 のとおり `key:`
// apply が List を書く形(CE-18):
//  - 元が1行のフローのリスト `key: [a, b]` → 値の範囲を `[a, b, c]` に置き換える(区切りは ", "。残る要素は元の書き方のまま、
//    新しい要素は WB-7 の規則でクオートし、フローの中では `,[]{}` と引用符 `'` `"` を含む値も二重引用符で書く)
//  - 元が複数行のブロックのリスト(要素が1行の素のスカラーか引用符つきの値だけ)→ 要素の行の範囲の中で、残る要素は元の行全体
//    (字下げ・`-` のあとの空白・中身・末尾の空白・改行)をそのまま使い、消す要素は行ごと消し、並べ替えは元の行を並べ替える。
//    新しい要素の行だけを、元の最初の要素の行の字下げと `- ` で作る(改行コードは元に合わせる。WB-1)
//  - 元が `key:`(Null)か空の文字列か、キーが無い → `key:` の行の次から `  - a` の行を並べる(Obsidian と同じ形。字下げは2つの空白)
//  - 空にする → `key:`(要素の行を消す)。元がフローなら値の範囲を消して `key:` にする
//  - CE-8 の形(複数行のフロー・アンカー・入れ子の要素・要素の中のコメント)は NotEditable
//  - 読み直しの検査(WB-6)は List を要素の文字列の並びとして比べる
// src/frontmatter.rs: BlockList の Entry に、要素の行の範囲(span)を持たせる(書ける形のときだけ。書けない形は None のまま)
// src/source.rs
fn list_candidates(&self, col: &str) -> Vec<(String, usize)>;   // 読んだ全ノート(BV-2)のその列の要素と件数。件数の多い順、同じ件数は文字の順(CE-19)。tags は先頭の # を除く
//   1つの値で書かれたもの(`tags: x`)も1つの要素として数える
// Cell.lock: リストの値は書ける形なら lock なし(CE-6 の削除)。書けない形は形ごとの理由(frontmatter の ListIssue:
//   要素の間の空行・要素の間のコメント・要素の行末のコメント・字下げの違う要素・`-` のあとのタブ・複数行にまたがる要素・
//   入れ子の要素・文字列でない要素)。リストの列で文字列でない1つの値のセル(`tags: 2024`)は「文字列でない要素を含むリスト」。
//   1つの文字列(`tags: x`)は lock なしで1つの要素のリストとして開き、writeback の write_list が値の範囲を `[..]` に置き換える(CE-19)
// src/changes.rs: same_value は List と Value::List を要素の文字列の並びで比べる(WB-17)。空の List は Null・空の文字列・
//   キーなしと同じ(書かない)
```

画面(`src/ui/listpick.rs`。モード「リストの選択」): リストの列のセルで Enter(か Space)で開く。上に検索の入力(打つたびに候補を大文字小文字を区別しない部分一致で絞る。CE-16・CE-19)、下に候補(`[x] 会議  5件` / 一括のとき一部の行だけが持つ要素は `[-]`)。↑↓ で選び、Space で付け外し(一部 → 付ける → 外す の順)、打った文字が候補に無ければ先頭に「+ 新規: …」が出て Enter で足す。Enter(候補が選ばれていないとき)か Tab で確定し、Esc で取り消し。確定は1行なら `Changes::set`、選んだ行(CE-17)なら行ごとに「元の要素に、付けた要素を足し、外した要素を除いた」並びを `Changes::set_each` で1手。要素の並びは元の並びを保ち、足した要素は末尾。

## 日付のカレンダーと日付の形(CE-20〜CE-22)

核(画面に依存しない。受け入れの試験を先に置く):

```rust
// src/types.rs に足す
pub struct DateFormat { /* 部品の並び */ }
impl DateFormat {
    /// 設定の形を読む。使える部品は YYYY・YY・MM・M・DD・D・ddd(曜日の短い名前「月」など)と、区切りの文字(`-` `/` `.` 空白 `年` `月` `日` など)。
    /// 読めない形は Err(理由1行)。既定は "YYYY-MM-DD"。
    pub fn parse(pattern: &str) -> Result<DateFormat, String>;
    pub fn iso() -> DateFormat;
    /// 日数(1970-01-01 から)を設定の形の文字列にする(表の見せ方。CE-22)
    pub fn format(&self, days: i64) -> String;
}
/// 打ち込みを読む(CE-5・CE-22): `YYYY-MM-DD`、設定の形(年が無い形なら今年)、`+N`・`-N`(今日から)。空は Ok(None)(null にする)。
/// 実在しない日付・読めない形は Err(理由)。
pub fn parse_date_input(s: &str, fmt: &DateFormat, today: i64) -> Result<Option<i64>, String>;
pub enum WeekStart { Sun, Mon }
/// カレンダーの1か月の格子(CE-21): その月の日を週ごとに並べ、前後の月の日は None。週の始まりは WeekStart。
pub fn month_grid(year: i32, month: u32, start: WeekStart) -> Vec<[Option<u32>; 7]>;
/// 月を足し引きした日(同じ日が無ければ月末。CE-21 の PageUp・PageDown)
pub fn add_months(days: i64, n: i32) -> i64;
// src/config.rs: Config に date_format(既定 "YYYY-MM-DD"。読めなければ警告して既定)と week_start(既定 Sun)
```

ノートに書くのはいつも `YYYY-MM-DD`(日時は元の形)。表の日付の列のセルの見せ方だけを `DateFormat::format` にする(型の合わない値は今までどおりそのまま `!`)。

画面(`src/ui/calendar.rs`): 日付の列のセルの編集で、入力ボックスの下(入らなければ上)に月の格子を重ねる。入力ボックスは今のまま(打つたびに読めればカレンダーをその日へ動かす)。←→↑↓・PageUp/PageDown で日を動かすと入力ボックスの文字もその日(設定の形)に変わる。今日に戻す・空にする のキー。Enter で確定(入力ボックスの文字を CE-5・CE-22 で読む)。今日は `[ ]` ではなく下線、元の値は `*`、選んでいる日は反転と `>`(色だけに頼らない。SR-15)。日時の列は日だけを変えて時刻を保つ。画面に収まらなければ格子を出さない(SR-9)。

## mdgrid のビュー(BV-17〜BV-20)

核(`src/views.rs`。画面に依存しない。受け入れの試験を先に置く):

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct NativeView {
    pub name: String,
    pub order: Vec<String>,            // 列の並び(空なら既定の並び)
    pub hidden: Vec<String>,
    pub filters_expr: Vec<String>,     // `.base` と同じ書き方の式の絞り込み(全部を満たす行だけ。`.base` から取り込んだ filters もここ)
    pub settings: settings::Settings,  // NV-13〜NV-22 のフィルター・並べ替え・グループ
}
/// views.toml を読む。対象(実体のパス)の分だけ返す。知らない項目・壊れたファイルは警告の文にして返し、止めない(BV-20)
pub fn load_views(dir: &Path, target: &Path) -> (Vec<NativeView>, Vec<String>);
/// 対象のビューを全部書き直す(ほかの対象のビューは保つ)。一時ファイル → 名前の変更
pub fn save_views(dir: &Path, target: &Path, views: &[NativeView]) -> io::Result<()>;
/// BV-19: `.base` の YAML の文字列にする。表せない部分は落として、その説明を返す
pub fn to_base(view: &NativeView) -> (String, Vec<String>);
/// BV-19: `.base` のビュー(全体の filters と合わせる)を取り込む。解釈できない部分は落として説明を返す
pub fn from_base(base: &base::Base, view: usize) -> (NativeView, Vec<String>);
```

`views.toml` の形:

```toml
[[target]]
path = "/Users/.../notes"           # 開いた対象の実体のパス
  [[target.view]]
  name = "未完了"
  order = ["title", "status", "due"]
  hidden = []
  filters_expr = ['status != "done"']
  [target.view.settings]            # settings::Settings と同じ形
```

画面: ビューのタブに「既定の表」(`.base` で開いたときは `.base` のビュー)、続けて mdgrid のビューを並べる。mdgrid のビューを選ぶと、その order・hidden・filters_expr・settings で表を組む(filters_expr は expr で評価し、`.base` の filters と同じく評価できない式のビューは開かずに理由)。ビューの設定の画面(NV-13)のボタンに「名前を付けて保存」「上書き」「名前の変更」「削除」を足す(名前は入力の欄で打つ)。パレットに「.base に書き出す」(新しい名前のファイルだけに書く。BV-3)と「.base のビューを取り込む」を足し、落とした部分をメッセージで知らせる。`--readonly` では保存・書き出しを出さない。

## 書き戻し(write-back)

- 日付の列(WB-18): `NewValue::Date(String)` は、元の値が引用符で囲んであれば元の引用符、そうでなければ囲まずに `YYYY-MM-DD`(日時は元の形)で書く。画面は列の型が date・datetime のセルにだけ Date を使い、ほかの列の日付に見える文字は WB-7 で囲む。

- 行単位の置換はしない。フロントマターを構文木で読み、対象のキーの値のバイトの範囲(開始と終わり)を特定し、その範囲だけを新しい値の文字列で置き換える。
- 部品の候補だった yamlpatch/yamlpath(tree-sitter-yaml の上、best-effort)と yaml-edit(若い)は使わず、第1階層のキーの行と、その値(1行のスカラー、フローのリスト)の範囲を自前で特定する(上の「全体の形」)。ブロックのリスト・ブロックスカラー・ネストした map・アンカーの値は範囲を求めず、読むだけにする。
- 値を空にする(CE-9)ときは、値の範囲と、コロンから値までの空白と、値の後ろから行末までの空白を消して `key:` にする。行末のコメントは残し、その前の空白は1つ以上残す(`key:#` にしない)。この空白を消すのは WB-1 の「値の範囲」の読み方の内とする(YAML の意味は変わらない)。
- フロントマターの無いノート(parse が `NoFrontmatter`)に書く(WB-3)ときは、先頭に `---`・編集のキーの行(edits の順で、画面から保存すると列の名前の順。書き方はキーを足すときと同じ)・`---` を足し、元のバイトはその後ろにそのまま置く。改行は本文の最初の改行に合わせ、無ければ LF。最初の編集でフロントマターを作り、残りはキーを足す・書き換える道で当てる。読み直しの検査は、足したフロントマターが読めて書いた値だけを持ち、閉じの `---` の行の直後が元のバイトと同じことを確かめる。BOM・UTF-8 でない・閉じないは足さない(WB-5)。
- 空のフロントマター(parse が `EmptyFrontmatter`。`---` が2行だけ)に書く(WB-3)ときは、閉じの `---` の行の前(開きの行の直後)に編集のキーの行を足す。改行は開きの行に合わせ(閉じの行の後ろに改行が無い `---\n---` でも)、書き方はキーを足すときと同じ(`new_key_lines`)。最初の編集の行を足し、残りはキーを足す・書き換える道で当てるので、複数の編集は書いた順に並ぶ。読み直しの検査は、結果が読め、開きの行と、閉じの行から後ろが元のバイトと同じ(足した行の外は変わらない)で、書いた値だけを持つことを確かめる。
- 設定 `add_frontmatter`(CLI-3。既定 true)は核の `apply` には渡さない。`main` が `Config` の値を `Markdown::set_add_frontmatter` で読み込み口に渡し(load の前)、false なら読み込み口がフロントマターの無いノートと空のフロントマターのノートのセルを lock にする(理由は「フロントマターが無い(設定で書かない)」「空のフロントマター(設定で書かない)」で区別)。画面と Changes は lock を見て書かない(set は Skip)。読み込み口の save も、基準の検査(外で変わっていれば Changed。WB-4・WB-16)のあとで、基準の内容に同じ判断をして断る。preview は書かないので設定を見ない(外で変わった行の差分を外の変更として見せる)。ほかの WB-5 の形は設定で変わらない。
- 書く前に一時ファイルへ書き、読み直して WB-6 を確かめてから名前を変えて置き換える。
- 差分テストの材料(最初から揃える): ブロックリスト、フローのリスト、ネストした map、ブロックスカラー(`|`・`>`)、行末コメント、CRLF、BOM、空のフロントマター(`---\n---`)、日本語のキー、キーの重複、`----` の行、引用符つきの値(`"`・`'`)、`: ` や ` #` を含む値、数字や `no` に見える文字列。
- 更新時刻だけの比較は、時刻の粒度が粗いファイルシステムで同じ秒の変更を見逃すので、大きさと中身のハッシュも比べる(WB-4)。

## 表の組み立て(base-view)

- `.base` は本物の YAML パーサで読む(saphyr / serde-saphyr)。知らないキーは残す(BV-8)。
- 式は、字句解析 → 構文木 → 評価の3段で作る。未対応の関数・演算子は構文木の段で分かるので、評価の前に「このビューは開けない / この列は読むだけ」を決められる(BV-7)。
- 型の推定は Obsidian の規則に合わせる(最初に見つかった空でない値。`YYYY-MM-DD` → date、`YYYY-MM-DDTHH:MM` → datetime、数値 → number、真偽 → checkbox、配列 → list)。
- 変化の検出は、毎秒すべてのノートの更新時刻と大きさを見て、変わったものだけを読み直す。ファイルの変更通知(notify)は補助にとどめる(イベントの取りこぼしとエディタの原子的な保存で監視が外れるため。cellops ADR-0007)。

## 画面と起動の文言の言語(SR-23)

形と、選ばなかった案は変更の記録 specs/_changes/2026-10-03-language.md にある。

- 文言の表: 画面と起動の文言は lib の `src/i18n.rs` の `msgs!` の表1か所に、英語と日本語の組で置く(`Key = "english", "日本語";`)。マクロが `enum Msg`・`Msg::ALL`・`const fn en`・`const fn ja`・`text`(今の言語)・`fill`(`{0}`・`{1}` の番号の差し込み。ほかの `{` と差し込んだ値の中は見ない)を作る。片方が欠けた行はコンパイルで落ちる。`Msg` は `Display` と `From<Msg> for String` を持ち、`Some(MSG.into())` のまま今の言語の文になる。依存は足さない(fluent などは const で持てず、欠けを型で閉じられない)。
- 日本語が同じでも意味の違う文(入力の「取り消し」とビューの設定の「取り消し」のボタンなど)は別の行にする。日本語の文は表に移す前の文とバイトまで同じにし、golden と錠のある試験をそのまま通す。
- src の文字列のリテラルに日本語を残さない(tests/test_no_japanese_literals.rs が試験のファイルと `src/i18n.rs` を除いて調べる)。日本語の名前を値として持つ公開の定数(`writeback::NO_PERMISSION`・`display::ITEMS` の名前・`keymap` の `label`・`native_views::DEFAULT_TAB` など)は `Msg::X.ja()` を指し、表示には今の言語の文(`text`・`label()`)を出す。`DEFAULT_TAB`(既定の表)は日本語の値。mdgrid のビューの名前の検査は英日の両方の名前(`Msg::DefaultTabName.ja()` と `.en()`)と比べて断る(どちらの言語でも既定の表と同じ名前のタブが並ばず、`--view` も取り違えない)。タブ・メッセージと、既定の表から書き出す `.base` のビューの名前は今の言語の名前(`Msg::DefaultTabName.text()`)。ビューの状態に書く名前(views.toml・見た目の状態の鍵)は言語で変わらない(既定の表の名前を状態の鍵に使わない)。`Y` のコピーの見出しの「ノート」(`view::LABEL_HEADER`)のように、出す文字の中の見出しや名前も今の言語で変わる。
- 言語の決め方: 設定の `language`(`auto`・`en`・`ja`。既定 `auto`)が `en`・`ja` ならそれ。`auto` は `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値が `ja` で始まれば日本語、ほか(どれも無いときも)は英語(`i18n::resolve`)。今の言語は、スレッドの上書き(`scoped`。試験用)> 大域の既定(`set_default`。main だけが呼ぶ)> 環境 の順。main は最初に環境で決め、`--config` を読んだあと `config::peek_language` で設定の `language` だけを読んで上書きする。言語は引数でも App・Source でも持ち回らない(公開の trait の形と、錠のある試験が構造体の式で作る `Startup` を変えないため)。
- clap の引数の誤りは設定を読む前に出るので、環境の言語で出す(SR-23 の小さな例外)。
- 試験と `cargo run` の言語: `.cargo/config.toml` の `[env]` が `LC_ALL`・`LANG` を `ja_JP.UTF-8` に force するので、`cargo test` と、そこから立てる実行ファイルと、`cargo run` は、利用者の `LANG` と設定の `auto` に関係なく日本語になる。英語の試験は `scoped(Lang::En)` か、子の環境に `LANG=en_US.UTF-8` を明示し `LC_ALL`・`LC_MESSAGES` を外して確かめる。開発中に英語で見るには、設定に `language = "en"` と書くか、`target/debug/mdgrid`(`target/release/mdgrid`)を直接起動する。
- 訳さないもの: 曜日の名前(`types::WEEKDAY_NAMES`。日付の形の `ddd` の読み書きに使うノートの値の形。英語の曜日の食い違いの文にも日本語の曜日が入る)、`config_items.rs` の `ja`・`ty_ja`(docs/config.ja.md と突き合わせる正本)、man と補完の説明(clap の `about`・`long_about`・`///`。日本語のまま)、`--print-config` のコメント(英語のまま)。試験の許す表(tests/test_no_japanese_literals.rs の `ALLOWED`)はこの一覧と合わせる。
- `scoped` はスレッドごとなので、試験で別のスレッドで作る文は大域の既定(試験では環境の日本語)の言語になる。


英語の単数と複数(docs-refresh): 数を含む英語の文言は `Msg::singular` で単数の文言と数の差し込みの位置を持ち、`Msg::fill` が英語でその値が `1` のときに単数を選ぶ(使う側は今までどおり `fill` を呼ぶだけ)。日本語は数で形が変わらないので同じ文。
## 画面のテーマ(SR-26・SR-27)

形と色の表は変更の記録 specs/_changes/2026-10-06-themes.md にある。

- 表: lib の `src/theme.rs` に `Theme`(設定の名前 `default`・`nord`・`solarized-light`・`dracula`・`gruvbox`・`pink-monster`・`dozy-pink`。`Theme::ALL` はこの順)と、役割ごとの RGB の `Palette`(`default` は None)。`to_indexed` は xterm の 256 色の色の立方体(16〜231)と灰色(232〜255)から最も近い番号(0〜15 は端末ごとに違うので使わない)。設定は最上位の `theme`。読めない値は型の警告(`Msg::WantTheme`)で既定のまま。
- 塗り方: `ui::draw` の最後に、描き終えたバッファを `ui/theme.rs` の `paint` が役割で塗り替える(後段の1か所)。反転のセルは下の帯の行(描く範囲の下から2行目)なら帯の色、ほかは選択の色にして反転を外す。反転でないセルは、前景が無ければ 1行目 → header・太字+下線 → colhead・太字 → strong・左端の `>` → mark・ほか → fg、背景が無ければ bg。一行おきの色(`ui/display.rs` の 237 / (58,58,58))は zebra の色に替える。セルが自分で持つ色(ためた値・一致・差分)と、自分の色を持つ反転のセルはそのまま。色なし(`ColorMode::None`)とパレットの無いテーマでは何もしないので、`default` の画面は1セルも変わらない。表の画面の列の見出しの行(`view::data_y` の1つ上)の太字+下線だけを colhead にし、ほかの太字+下線(入力ボックス・一致した名前)は strong。一行おきの色は「色なし」とみなして役割で塗り(反転なら選択・帯の色、fg の役のときだけ zebra_fg、地は zebra_bg)、選んでいるセルや `>` も zebra の行で役割の色になる。
- 右下の隅の1セルは塗らない: 描くのと同じく、端末の右下の隅を書くと自動折り返しでずれる端末があるため(ほかの最下行と右端は地の色で塗る)。
- 選ばなかった案: 各部品(表・帯・窓・ヘルプ・差分・カレンダー)にテーマを渡して色を付ける(部品が多く、取りこぼすと色の混ざった画面になり、`default` の不変を試験で守りにくい)。窓の枠だけの色(枠は行の文字と1つの文字列で、枠だけを見分けられない)。

## 部品の形と端末に従うテーマ(SR-36・SR-39)

- 設定: lib の `src/style.rs` に部品ごとの列挙(`named!` で名前と値の表を1か所に)と組(`Style::of`)。`[style]` は `preset` を先に当て、ほかの項目で上書き。カタログ(SR-38)と名前を突き合わせる試験の元もこの表。
- 値の部品: `ui/chips.rs` の `parts` が値を「前置き・要素ごとの切れ・区切り」の並び(`Parts`)にし、セルの文字(幅を数える)と描く所(`view::chip_spans`)の両方がこれを使う(幅と描きのずれを作らない)。切れの見た目は行の見た目に重ねる(`patch`)ので、一行おきの色や十字の選びの地を点や区切りが消さない。色は状態の英語の語(done・doing・blocked・todo など)なら意味の色、ほかは値の文字の指紋から。
- 選び・線・タブ・帯は `ui/look.rs` の `select`・`rules`・`Look::tab`・`Look::band_key`。モダンな見た目(色あり・look = modern)のときだけ効き、classic と色なしは今まで(SR-33)。どの形でも画面の文字は変えない(見出しの下の線は下線、選びの `bar` も印は `>` のまま色で)。窓の枠は `popup::frame` が `frames` から(線なしも幅は同じ)。
- 端末に従うテーマ: `ui/termbg.rs`。`COLORFGBG` の地の番号(7・9〜15 を明るい)を先に見て、無ければ `/dev/tty` を raw にして OSC 11 と DA1 を続けて送り、`poll` で最大 200ms 待つ。DA1 の答えが先に来たら答えない端末とみなしてすぐやめる(遅れた答えを表のキーと読まないため)。依存を足さず、std がリンクしている libc の `poll` を直接呼ぶ(`ui/external.rs` の `signal` と同じ形)。Windows は問い合わせない。解決は起動のとき1回(`main::open_app`)。
- 選ばなかった案: 選びの `bar` で印を `▌` にする(SR-33 の文字を変えない決まりと、`>` を確かめる多くの試験とぶつかる)。状態の日本語の語の表(SR-23 でコードに日本語のリテラルを置かない)。

## 設定の形と範囲ごとの上書き(CLI-3・CLI-11・CLI-12・CLI-20・CLI-21・SR-43・SR-44)

要求と調べた不整合は変更の記録 specs/_changes/2026-10-10-config-v2.md。

- **項目の表を1つに**: lib の `src/schema.rs` に、項目ごとの `Item { path: "look.style.status", ty, ty_ja, default, example, scope: Scope::Global | Scope::Profile, en, ja }` を並べる(`config_items.rs` を置き換える)。読み取りの知らない項目の判定・`--print-config`(区画ごと)・`--print-config --resolved`・docs/config*.md と突き合わせる試験・カタログの試験が、この表を見る。新しい項目は表に1行足し、読み取りの `match` に1つ足す(足し忘れは試験で落ちる)。
- **旧い名前の表**: 同じファイルの `LEGACY: &[(old, new)]`。読み取りは、最上位で旧い名前を見つけたら、表を新しい形に写して(`legacy::lift`)から読み、「旧い名前は新しい道筋に移った」と警告する。新しい道筋にも書いてあれば新しいほう。`--migrate-config` は同じ写しをして `toml::to_string` で出す(注釈は移らない)。docs の旧い名前の表もこの表と突き合わせる。
- **プロファイル**: `src/profile.rs` の `Profile`(どの欄も Option の部分の値): `look: LookLayer`(theme・preset・mode・cells・style の部品ごと・columns・colors の役割ごと・values)、`display: DisplayOverride`(今の型。tabs を `Tabs3` にする)、`dates: DatesLayer`、`edit: EditLayer`、`new_note: Option<NewNote>`、`use_: Option<String>`。`Profile::read(table, file, prefix, warns)` が config.toml・ui.toml・ワークスペースの印・workspaces.toml・views.toml のどれでも同じ読み方をし、`Profile::write(&mut toml::Table)` が画面の書くファイルに書く。
- **決まった値**: `profile::resolve(layers: &[Layer], templates) -> Resolved` が、既定から狭い範囲へ順に重ねる。`Layer { origin: Origin, profile: &Profile }`、並びは 既定 → config.toml → ui.toml → ワークスペース → 表(手) → 表(views.toml) → ビュー。各層は先に `use` のテンプレートを敷いてから自分の値を重ねる(テンプレートの中の `use` は警告して読まない)。組を持つ層で `style` をその組の形に戻し(それより前の層の部品の形を捨てる)、テーマを持つ層で役割の色を空に戻す(値の色は残す)。`Resolved` は今の画面が使う型(`Style`・`Colors`・`Cells`・`Display`・`DateFormat`・`WeekStart`・`NewNote` など)と、項目の道筋ごとの `Origin`(SR-43 の「(この表)」と CLI-21 のコメント)を持つ。
- **Config**: アプリ全体の項目(`language`・`editor`・`poll_ms`・`terminal`・`workspace`・`keys`)と、config.toml のプロファイルとテンプレートを持つ。ui.toml は `UiFile { profile, templates, nerd_font }`。App は全体・ワークスペース・表の層を持ち、ビューを選ぶたびにビューの層を足して `resolve` し直す(`App::apply_profile`)。表を切り替えるときは main が開き直すので、範囲(WS-6)が決まったあとの `App::start` で層を集める。`add_frontmatter` は表で決まるので、読み込みの前(`start` の中)に源へ渡す。
- **警告の形**: `Msg::SettingWarn = "{0}: {1}: {2}"`(ファイル・道筋・理由)。理由は `Msg::Want*`・`Msg::Moved` など。どのファイルの読み取りも `warn(file, path, reason)` を通す。
- **画面の書く場所**: 全体 → ui.toml(`config::write_atomic`)、ワークスペース → workspaces.toml のそのワークスペースの区画(`workspace::save_profile`。文字の区画を置き換える今の書き方で、手のコメントを残す)、表 → views.toml の `[[table]]` の `look` など、ビュー → ビューの設定(`Settings` に `look`・`dates`・`edit`・`use_` を足す)。
- 選ばなかった案: 範囲ごとに丸ごとの Config を作って上書きする(どの値がどこから来たかが分からず、組とテーマの決まりが書けない)。`serde` の derive で全部を読む(知らない項目・型の違いを「ファイル: 道筋: 理由」で出せず、旧い名前を写せない)。

## 文字の幅

- unicode-width と書記素のまとまり(unicode-segmentation)で数える。East Asian Ambiguous(○● など)は既定で1とし、設定で2にできるようにする。
- 絵文字は幅が端末ごとにぶれるので、表の飾りには使わない。

## モジュールの分け方と、写しを作らない決まり(refactor-maintainability。2026-10-07)

調べ(ratatui の作りの型(TEA・部品・Flux)、gitui・helix の重ねる窓の扱い、Rust の型と誤りの設計、安全な
リファクタリング)と、コードの込み合いの調べから決めた。振る舞いは変えていない。

- **ライブラリに輪を作らない**: 下の層(types・expr・frontmatter・links)は上の層(print・base・source)に頼らない。
  - 時計(地域の時差・今日と今)は `clock.rs`。`print::today_now` などの道は `pub use` で残す。
  - 本文の行の読み方(字下げ・コードのフェンス)は `mdtext.rs`(source::markdown と links が使う)。
  - YAML の見張り(別名の展開の数・入れ子の深さ)は `yaml_guard.rs`(base と frontmatter が使う)。
  - 残した輪: `newnote::rule_for`(config・views に頼る)。試験が `newnote::rule_for` の道を使うので動かしていない。
- **`--apply` は画面に頼らない**: 入力の読み方(`Entry`・`parse`)と書けない理由の文は `src/edit.rs`、差分は
  `src/diff.rs`(どちらもバイナリの中。画面の `ui::entry`・`ui::review`・`ui::diff` の道は再輸出で残す)。
- **重ねる窓の共通の部品は `ui/popup.rs`**: 画面の大きさ(`screen`)、セルの下か上への置き方(`place`・
  `place_with`)、上の縁(`top_edge`)、当たり(`hit`)、表の行への重ね方(`blit`)、一覧の選びの上下(`step_sel`)。
  新しい窓はこれを使い、置き方の計算を写さない。
- **同じ働きの関数は1つ**: ノートの値の素の文字は `print::value_plain`、ためた値の素の文字は
  `ui::input::new_value_text`。`one_line` は「改行を空白に」(print)だけで、空白をまとめるのは
  `config::squash_ws`、セルの1行目と `…⏎` は `cell::first_line_marked`。
- **設定の書き込みは1つ**: `config::write_atomic`(一時ファイル・fsync・名前の変更)と `config::toml_error`(壊れた
  TOML の行と理由)を、状態(state)と views.toml の両方が使う。
- **新しいノートの決まりの項目は `NewNote::KEYS` だけ**: 設定と views.toml の読み書きは、この並びを見る。
- **App を部分の構造体に分けていく**: 試験が読まない項目から、意味のまとまりで分ける(`grid::Built`・
  `nav::PickOut`)。試験が直接読む項目(mode・row・input など)の名前は変えない(錠のある試験を変えないため)。
- 次の段の候補(まだしていない): 重ねる窓を `enum Overlay` の重なり(上の窓が先にキーを受け、使わなければ
  モードのキーへ)にする、窓を ratatui の Widget として自分の Rect に描く、`Result<_, String>` を誤りの型に、
  NewNote.mode などの文字の値を enum に。

## テスト

- 画面は文字列のゴールデン(`UPDATE_GOLDEN=1` で更新)。設定なし・今日の日付は環境変数で注入(`MDGRID_TODAY`)・材料の更新時刻は固定。
- キー列を入れて状態を確かめるテスト。
- 全行が画面の幅以下になる不変条件のテスト(SR-9)。
- テストの関数名は Rust の決まりどおり小文字(`test_wb_1_keeps_other_bytes` など)にし、確かめる要件の ID は本文のコメントに `[WB-1]` と書いて、要件とつなぐ(decidespec は本文の `[WB-1]` を数える。`test_WB_1` の形は `non_snake_case` の警告になり、CI の `clippy -D warnings` で落ちる)。仕様の確かめ方に書いたテスト名は、この形に読み替える。
- src の中の単体のテストは test_*.rs のファイルに置く(check.py が数える)。統合のテストは `tests/test_*.rs`。子のモジュール(`#[cfg(test)] #[path = "test_<名前>.rs"] mod tests;`、ui/mod.rs は `mod test_screen;`)にして、`use super::*;` で private な関数も使う。
- 回帰の試験表: 全角・絵文字・結合文字・East Asian Ambiguous・制御文字を含む材料の画面のゴールデンを持ち、ratatui と crossterm の版を上げるときに必ず走らせる(csvlens の日本語の行が重なる不具合は ratatui の版で起きた)。手で確かめる端末の表(Terminal.app・iTerm2・Ghostty・WezTerm・tmux の中)を docs に置く。
- 端末で区別できないキー(Ctrl+Shift+Z・Shift+Space・Ctrl+Space など。kitty のキーボードの手順が無い端末)は、同じ動作に区別できるキー(やり直しは Ctrl+R と `U`)も割り当てる。
- CI: fmt・`clippy -D warnings`・test。
- CHANGELOG: コミットの書き方(`実装:`・`修正:` などの書き出し)は今のままにし、リポの根の `cliff.toml`(git-cliff)で書き出しを CHANGELOG の節に写す(`実装`・`追加` → Features、`修正` → Fixes、`文書` → Documentation、`specs` → Specification、ほかは出さない)。CHANGELOG.md はリリースの作業(Q-6)で生成する。新しい書き出しを使うときは cliff.toml に足す(tests/test_cliff.rs が履歴の書き出しの漏れを見る)。
- TUI 本体(配る実行ファイルの言語・フレームワーク)は Rust だけにし、cellops のような Node との二重持ちはしない。試験の道具はこの限りでなく、通しの試験(本物の疑似端末で起動 → 操作 → 画面とファイルを確かめる)には Node の Microsoft tui-test を `e2e/` に分けて使ってよい(人の判断、2026-10-01)。単体の試験とゴールデンは `cargo test` に置く。

## セッション(CLI-6〜CLI-10)

表示に名前を付けたもの。起動の引数と同じ「起動の指定」(`Launch`)に名前を付けて、設定の置き場の `sessions.toml` に置く(ノートのフォルダ・`.base` には書かない)。核(base・changes・ui)はセッションを知らず、`Launch` だけを受ける。絞り込みは画面で掛けた種類のまま持つ(`text` = NV-2、`value` = NV-8、のちに `expr` = NV-10)。画面からの保存はパレットのコマンドで、既定のキーは割り当てない。書くときは一時ファイル → 名前の変更。形式と、選ばなかった案(`.base` として書き出す・`config.toml` に混ぜる)は変更の記録 specs/_changes/2026-10-01-sessions.md にある。

