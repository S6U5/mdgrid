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

**書く単位(SC-15・SC-16)。** 行と、行が書かれるファイルは1対1とは限らない(CSV は全部の行が1つのファイル)。そこで読み込み口は `unit(row)`(行が書かれる先)を返し、ためる変更(`Changes`)は差分(`previews`)・保存(`save`・`save_rows`)・外の変更の印・捨てるを、この単位ごとにまとめる。単位ごとに `preview_unit`・`save_unit` を1回呼び、全部の行の直しを1つのバイトに当てて1回で書く。保存の基準(WB-4)は単位の中の行で1つでなければ止める。Markdown は単位が行そのもの(既定の実装。今までの振る舞いと同じ)で、`Preview.row` と保存の結果の鍵は単位(Markdown では行)。こうすると、行ごとに同じファイルを書き直したり、読み込み口が自分で書いた基準を覚えて受けたりしなくてよい。CSV の行の鍵は行の位置でなく、開いたときに配った変わらない番号(`ファイル#番号`)で、外で変わったときは1文字も変わっていない行だけが番号を引き継ぐ(外で行が足されても、直しを別の行に書かない)。開く対象は形式によらず `Box<dyn Source>` で持ち(`main` の `Target`)、`Box<S>` にも `Source` を実装して転送する。

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
/// 渡したフォルダからルートフォルダを決める(BV-2): `.obsidian/` を持つ最寄りの上のフォルダ、無ければ渡したフォルダ。
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
pub fn config_path() -> Option<PathBuf>;            // $XDG_CONFIG_HOME/mdgrid/config.toml、無ければ ~/.config/mdgrid/config.toml(Windows は %APPDATA%\mdgrid\config.toml)
/// 見た目の状態(SR-11・SR-12): $XDG_STATE_HOME/mdgrid/(無ければ ~/.local/state/mdgrid/)に、開いた対象のパスとビューの名前ごとの TOML。
pub struct ViewState { pub order: Vec<String>, pub hidden: Vec<String>, pub widths: Vec<(String, u16)>, pub folded: Vec<String>, pub view: Option<String> }
pub fn state_dir() -> Option<PathBuf>;              // $XDG_STATE_HOME/mdgrid、無ければ ~/.local/state/mdgrid(Windows は %LOCALAPPDATA%\mdgrid。端末ごとでローミングしない)
pub fn home_dir() -> Option<PathBuf>;               // $HOME、無ければ Windows は %USERPROFILE%(先頭の `~`)。置き場の3つは環境変数と Windows かを引数に取る *_in で、どの OS の試験でも Windows の分岐を確かめる
pub const DEFAULT_EDITOR: &str;                     // SR-8 の既定: vi、Windows は notepad(vi がふつう無い)
pub fn load_state(dir: &Path, target: &Path, view: &str) -> ViewState;   // 無い・壊れている → 既定(空)
pub fn save_state(dir: &Path, target: &Path, view: &str, s: &ViewState) -> io::Result<()>;  // 一時ファイル → 名前の変更
pub fn load_state_with_base(dir: &Path, target: &Path, view: &str) -> (ViewState, Option<Vec<String>>);  // 並びを覚えたときの印の order の控えも
pub fn save_state_with_base(dir: &Path, target: &Path, view: &str, s: &ViewState, base: Option<&[String]>) -> io::Result<()>;
/// screen.state-store-follow-order: 覚えた並びを今の印の order に合わせる(人の動かした列のうち食い違わないものを残す)。
pub fn follow_order(before: &[String], saved: &[String], now: &[String]) -> Vec<String>;
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
| `src/ui/grid.rs` | 表の行と列の組み立て: `.base` があれば `Base::build`、無ければ `default_grid`。その上に検索(NV-2)・同じ値の絞り込み(NV-8)を重ね(見出しの並べ替え NV-3 と並べ替えの窓 NV-24 は設定の並べ替えを変える。settings.rs の set_sorts。絞り込みの窓 NV-29 は設定のフィルターを変える。set_filters)、ためた値で計算し直す(NV-12: 値を直した行は、保存か移動の操作まで元の位置に留める)。列の順・隠す列・幅(NV-4・SR-12)、グループの開閉(SR-2) | BV-4・BV-5・BV-13・NV-2・NV-3・NV-4・NV-8・NV-12・SR-2 |
| `src/ui/nav.rs` | 移動(NV-7: `:<番号>`・`0` `$`・Ctrl+D / Ctrl+U)、検索(NV-1: `/`・`n` `N`・smartcase・一致の強調)、選択(NV-5: Space・`v`・Shift+矢印・Ctrl+A・Esc)、`*` の強調・`,` の絞り込み(SR-18・NV-8)、`--pick` の選び(OUT-3: `App.choosing`。表の Enter はノートの行なら印の行(無ければ選んでいる行)を `App.chosen` に決めて終わり、見出しの行は開閉のまま。Esc は SR-18 の順に解き、解くものが何も無いときだけ取りやめ。q はふだんの終了で、`chosen` が無いまま終われば取りやめ。エディタ(`e`・名前のクリック)は開かず理由を出す。コピーの OSC 52 は `TtyClipboard` で `/dev/tty` へ) | NV-1・NV-5・NV-7・NV-8・SR-18・OUT-3 |
| `src/ui/detail.rs` | 詳細の表示(NV-6: `K`。全プロパティを縦に、長い値は折り返す。Enter で選んだプロパティの編集、Esc と `K` で閉じる) | NV-6・SR-16 |
| `src/ui/input.rs` | 型ごとの入り方(CE-2〜CE-7): テキストは候補のリスト(CE-3。`Source::candidates`、今の値に印、「なし」、自由入力に切り替え。開いた直後(↑↓ のあとも)に打つと今の値を置き換える: Input の `fresh`)、チェックは切り替え(CE-4)、日付は `YYYY-MM-DD`・`+3`・空(CE-5)、数は数だけ(CE-7)。不正なら閉じずに理由。一括の設定(CE-10: 選んだ行に `Changes::set_many`、飛ばした数と理由) | CE-2〜CE-7・CE-10 |
| `src/ui/help.rs` | ヘルプ(SR-5: `?`。先頭に今のモードで押せるキー、その下に全部。screen.help-run: 今押せるキーを選んで Enter でする)とパレット(SR-14: `:` と Ctrl+P。動作の名前のあいまい検索、割り当てキーを横に、`:120`・`w`・`q`・`wq`・`x`)。どちらもキーの表から作る | SR-4・SR-5・SR-14・SR-16 |
| `src/ui/external.rs` | `$EDITOR`(SR-8: 引数つきを空白で分け、シェルを通さず、パスは1つの引数。端末を戻してから起動し、戻ったら読み直す)、コピー(OUT-1: `y` と Ctrl+C。セル・行・選んだ行を見出しつきのタブ区切りで、OSC 52 と OS のクリップボード(macOS は `pbcopy`、Linux は `wl-copy` か `xclip`、Windows は `clip` に BOM つきの UTF-16LE と CRLF で渡す(UTF-8 のままだと日本語が化ける)。表は `clip_commands`。依存の crate は足さない)。片方が失敗しても止めずに知らせる) | SR-8・OUT-1 |

起動(タスク 8): `mdgrid [<.base | フォルダ> …] [--view <名前>] [--readonly] [--no-color] [--config <パス>] [--print-config] [--completions <シェル>] [--man] [--help] [--version]`(CLI-1・CLI-2。`--print-config` は CLI-11 で、`config::default_toml()` を出して終了コード 0。`--completions`・`--man` は CLI-13 で、`clap_complete`・`clap_mangen` が clap の定義から作ったものを出して終了コード 0。どちらも画面を出さないので、標準出力がパイプでも動く)。`.base` を渡したら `Markdown::open_vault`。設定(CLI-3)を読み、警告はメッセージ行に出す。キーの割り当て直し(SR-13)は、設定の `(モード, キー, 動作)` で表を上書きした写しを作る(`"none"` で外す)。起動で開くビューは `.base` の先頭のビュー(`--view` があればそれ。BV-13)で、前に選んだビューは起動に使わない。見た目の状態(SR-11・SR-12)は、ビューごとに、そのビューを開いたときに読んで列の順・隠す列・幅・畳んだまとまりに当て、終了とビューの切り替えで書く。`--readonly`(WB-15)では編集・空にする・保存の動作を表から外し、保存のキーで「読むだけで開いている」と出す。在る `.md` のファイル(拡張子の大文字小文字を問わない)を渡したら(CLI-15)、`main::md_to_folder` が `check_paths` の前にそのフォルダ(親が空なら `.`)に置き換え、`--print`・`--pick` もその置き換えたフォルダを起動の引数とみなす(`--pick path` の形も)。画面では `App::select_after_load` に覚え、読み込みが終わったとき(取りやめを含む)に一度だけその行を選ぶ(途中で選ぶと並べ替えで行が動き画面が跳ねるので、途中では選ばない)。無い `.md` は置き換えず、CLI-4 の理由1行にする。

見せ方(タスク 9 の残り): ビューのタブ(BV-13。`[` `]` か Tab の帯のクリックで切り替え)、未対応の列の薄い `?` と、選んだときの下の帯の説明(BV-7)、型の合わない値の `!`(CV-2)、East Asian Ambiguous の幅(CV-6。設定 `ambiguous_wide`。`width.rs` で ○● などを2と数える)、列の幅の `<` `>` とドラッグ(SR-3・NV-4)。

ヘルプから選んでする(screen.help-run。`src/ui/help.rs`): 本文の行(`Row`)に選べる項目(`Cell`: 始まりの桁・幅・Enter でする動作)を持たせ、`HelpState::sel` は本文の項目を上から数えた番号にする。2列の行では項目はその片方で、↑↓ は読む順(左 → 右 → 次の行)に1つずつ動く。←→ で列を移す形では ↑↓ だけで右の列に届かず、1列に並べ直すと先頭の節が倍の長さになって1画面に見える今押せるキーが半分になるので、2列のまま読む順にした。動作を持つのは先頭の「今押せるキー」の節の項目だけで、Enter はヘルプを閉じて開いた元のモードに戻してから `App::apply` に渡す(キーを押したときの `dispatch` と、パレットの実行と同じ道)。数字のキーを1行にまとめた項目(`1-9`)は先頭のキーの動作(`GoToSection(1)`・`Choose(1)`)にする。下の節の項目も選べるが動作を持たず、同じ動作の行でも Enter は何もしない。選んだ項目は反転と、項目の2桁左の空白に置く印 `>` で見せ(色だけで伝えない)、モダンな見た目では `ui::paint` の `swap_reverse` が反転を選びの地に替える。反転はキーの欄の見た目に重ねるので、テーマに選びの文字の色が無いとき(default)はキーのアクセントの色が残る。開いたときは先頭の項目を選ぶ。↑↓ で選びが上へ出たら、節の先頭の項目なら見出しと空行も見えるように流す。ホイールは選びでなく本文を1行流し(screen.mouse-shift-wheel-other の「ヘルプが下へ流れる」を保つ)、選んだ項目が見えなくなったら見えている一番近い項目へ移す。クリックは項目(左の印の桁を含む)を選び、選んでいる項目のクリックは Enter と同じ(表のセルの2段階のクリックと同じ)。

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

重ね順(NV-20): `.base` の filters・sort・groupBy(`Base::build`)→ 設定(`settings::apply`)→ 検索(NV-2)・同じ値(NV-8)→ 直した行の留め(NV-12)。見出しの並べ替え(NV-3)と並べ替えの窓(NV-24)は設定の並べ替えそのものを、絞り込みの窓(NV-29)は設定のフィルターそのものを変え、見た目の状態に残す。計算の列(`file.*`・`formula.*`)は画面の値(`Shown::Computed`)を `Value` に写して渡す。

画面(`src/ui/settings.rs`): モード「ビューの設定」。開くと今の設定の写しを編集し、「反映」で `App` に当てて組み立て直し、見た目の状態に書く(WB-15 では書かない)。「取り消し」・Esc は写しを捨てる。4つの区画(列・フィルター・並べ替え・グループ)を Tab で移り、フィルターは列を選ぶ → 値の一覧(件数つき)にチェック(Space)か、条件の種類を選んで値を打つ。区画の下に「反映」「取り消し」「既定に戻す」のボタン(クリックでも押せる)。キーはすべて keymap の表(SR-4・SR-16)。表の上の帯(`bands.rs`)は、ビューのタブの下に `settings.chips()` を並べ、選んで Backspace(か帯の項目のクリック)でその条件だけを外す(NV-22)。開くキーは表のモードの空いたキー(パレットの動作 `view_settings` からも)。

左の区画の一覧のキー(navigation.settings-sections-keys・-digits・-focus。`src/ui/settings_nav.rs`): キーが効く側を写しの `Draft::nav`(左の一覧か右の項目か)に持ち、選び手が開いている間は右とみなす。左にいる間の ↑↓ は区画を替え、項目に効く動作(Space・K / J・消す)はしない。Tab・Shift+Tab と、項目を選ぶ操作(`Draft::select`・右のクリック)は右へ戻し、左の一覧のクリックは左にする。選びの印 `>` は効く側にだけ付け、右の選んだ項目は左にいる間も覚えて流す位置に使う。数字のキーは番号を持つ動作 `Action::GoToSection(n)` を 1〜9 の9行で表に置く(名前は `go_to_section_1` のように番号ごとに分け、パレットの探す鍵(動作ごとに違う名前)と割り当て直し(SR-13)を崩さない。ヘルプでは1行にまとめ、続いた数字を `1-9` と縮める)。番号は左の一覧の並び(ボタンを除いた Tab の順。CSV の表では親子を除いて詰める)で、選び手の上では Tab と同じく移らない。値の入力のモード(`settings_input`)は文字を受けるので、数字は値になる。

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

画面(`src/ui/listpick.rs`。モード「リストの選択」): リストの列のセルで Enter(か Space)で開く。上に検索の入力(打つたびに候補を大文字小文字を区別しない部分一致で絞る。CE-16・CE-19)、下に候補(`[x] 会議  5件` / 一括のとき一部の行だけが持つ要素は `[-]`)。↑↓ で選び、Space で付け外し(一部 → 付ける → 外す の順)、打った文字が候補に無ければ先頭に「+ 新規: …」が出て Enter で足す。検索が空の Enter も付け外しで、窓の一番下の「[ 決める ]」の行の Enter かクリックと、どの行でも Tab で確定し、Esc で取り消し(screen.checklist-keys。下の「一覧の窓のキー」)。確定は1行なら `Changes::set`、選んだ行(CE-17)なら行ごとに「元の要素に、付けた要素を足し、外した要素を除いた」並びを `Changes::set_each` で1手。要素の並びは元の並びを保ち、足した要素は末尾。

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

## 日付の言葉(cell-edit.date-words。2026-11-21)

言葉の読みは核の `src/datewords.rs` に置き、`types::parse_date_input` が `YYYY-MM-DD`・`+3`・設定の形のあとに試す(前からの形が先に読む。言葉と形がぶつかっても前の読みを変えない)。セルの編集・一括・新しいノートの欄・カンバンのレーン・カレンダーの追いかけ・絞り込みの比べる値は、どれもこの1つの口を通る。`--apply` は前からの決まりどおり `YYYY-MM-DD` だけを受け(当てる日で値が変わるため)、言葉も読まない。

- 週の始まりは読み方の引数にする。`parse_date_input` と `edit::parse` は形の代わりに `DateInput { fmt, week }` を受け、`&DateFormat` を渡せば週は既定(日曜)になる(`impl Into<DateInput>`)。前からの呼び出しと試験の形を変えずに、画面だけが設定の `dates.week_start` を渡す(`App::date_input`)。
- 語の表は訳さない。日本語と英語の語を、画面の言語によらず両方読む(screen.md の採らない: 日付の言葉は日本語と英語だけ)。台帳に置くと、台帳を足すたびに語の訳が要り、その言語でしか読めなくなるため。リテラルの検査(tests/test_no_japanese_literals.rs)は `datewords.rs` を許す表に入れた。
- 読みの見せ方(`src/ui/date_words.rs`)は、字(かな・漢字・英字)を含む打ち込みだけに出す。`2026-11-0` のように数字を打っている途中で「読めない」と出さないためで、日時の時刻つきの形(`T`)も言葉とみなさない。
- 範囲を出る言葉(`+99999y`)は、範囲の端で止めずに「範囲の外」と知らせる(`types::add_months` は端で止めるので、言葉の側に月の足し算を持つ)。
- 絞り込みの比べる値に打った言葉は、打ったときの今日で `YYYY-MM-DD` に直して条件に残す。条件は見た目の状態に残り、言葉のまま残すと開く日によって絞る行が変わって、条件の欄の見た目と結果が合わなくなるため。

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

形と、選ばなかった案は変更の記録 specs/_history/decidespec/_changes/2026-10-03-language.md にある。

- 文言の台帳: 画面と起動の文言は言語ごとの台帳 `locales/<言語の符号>.toml` に置く(1行に1つ `Key = "文"`。`##` の行はその文言の説明、`#` の行は区切り。英語の `en.toml` が名前と並びの正本)。`build.rs` が台帳を `src/ledger.rs`(依存は toml だけの純関数。lib の `mdgrid::ledger` としても試験できる)で読んで確かめ、欠け・余り・空・差し込みの番号(`{0}` など)の食い違いがあればビルドを落とす。誤りが無ければ `OUT_DIR/locales.rs` に `Lang`(台帳のファイルの名前から。英語が先頭で既定)・`Language`(`Auto` と各言語)・`Msg`・`Msg::ALL`・言語ごとの `const fn en`・`ja`・`get` を書き、`src/i18n.rs` が `include!` する。`text`(今の言語)・`fill`(`{0}`・`{1}` の番号の差し込み。ほかの `{` と差し込んだ値の中は見ない)・`singular` は `src/i18n.rs` にある。`Msg` は `Display` と `From<Msg> for String` を持ち、`Some(MSG.into())` のまま今の言語の文になる。言語を足すときは台帳を足す(`auto` は環境変数の値の頭が英語のほかの言語の符号なら、その言語)。実行時に読む形や fluent などは使わない(配るファイルが増え、欠けが動かすまで分からず、const で持てない)。
- 日本語が同じでも意味の違う文(入力の「取り消し」とビューの設定の「取り消し」のボタンなど)は別の行にする。日本語の文は表に移す前の文とバイトまで同じにし、golden と錠のある試験をそのまま通す。
- src の文字列のリテラルに日本語を残さない(tests/test_no_japanese_literals.rs が試験のファイルと `src/i18n.rs` を除いて調べる。文言の文は台帳にある)。日本語の名前を値として持つ公開の定数(`writeback::NO_PERMISSION`・`display::ITEMS` の名前・`keymap` の `label`・`native_views::DEFAULT_TAB` など)は `Msg::X.ja()` を指し、表示には今の言語の文(`text`・`label()`)を出す。`DEFAULT_TAB`(既定の表)は日本語の値。mdgrid のビューの名前の検査は英日の両方の名前(`Msg::DefaultTabName.ja()` と `.en()`)と比べて断る(どちらの言語でも既定の表と同じ名前のタブが並ばず、`--view` も取り違えない)。タブ・メッセージと、既定の表から書き出す `.base` のビューの名前は今の言語の名前(`Msg::DefaultTabName.text()`)。ビューの状態に書く名前(views.toml・見た目の状態の鍵)は言語で変わらない(既定の表の名前を状態の鍵に使わない)。`Y` のコピーの見出しの「ノート」(`view::LABEL_HEADER`)のように、出す文字の中の見出しや名前も今の言語で変わる。
- 言語の決め方: 設定の `language`(`auto`・`en`・`ja`。既定 `auto`)が `en`・`ja` ならそれ。`auto` は `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値が `ja` で始まれば日本語、ほか(どれも無いときも)は英語(`i18n::resolve`)。今の言語は、スレッドの上書き(`scoped`。試験用)> 大域の既定(`set_default`。main だけが呼ぶ)> 環境 の順。main は最初に環境で決め、`--config` を読んだあと `config::peek_language` で設定の `language` だけを読んで上書きする。言語は引数でも App・Source でも持ち回らない(公開の trait の形と、錠のある試験が構造体の式で作る `Startup` を変えないため)。
- clap の引数の誤りは設定を読む前に出るので、環境の言語で出す(SR-23 の小さな例外)。
- 試験と `cargo run` の言語: `.cargo/config.toml` の `[env]` が `LC_ALL`・`LANG` を `ja_JP.UTF-8` に force するので、`cargo test` と、そこから立てる実行ファイルと、`cargo run` は、利用者の `LANG` と設定の `auto` に関係なく日本語になる。英語の試験は `scoped(Lang::En)` か、子の環境に `LANG=en_US.UTF-8` を明示し `LC_ALL`・`LC_MESSAGES` を外して確かめる。開発中に英語で見るには、設定に `language = "en"` と書くか、`target/debug/mdgrid`(`target/release/mdgrid`)を直接起動する。
- 訳さないもの: 曜日の名前(`types::WEEKDAY_NAMES`。日付の形の `ddd` の読み書きに使うノートの値の形。英語の曜日の食い違いの文にも日本語の曜日が入る)、`config_items.rs` の `ja`・`ty_ja`(docs/config.ja.md と突き合わせる正本)、man と補完の説明(clap の `about`・`long_about`・`///`。日本語のまま)、`--print-config` のコメント(英語のまま)。試験の許す表(tests/test_no_japanese_literals.rs の `ALLOWED`)はこの一覧と合わせる。
- `scoped` はスレッドごとなので、試験で別のスレッドで作る文は大域の既定(試験では環境の日本語)の言語になる。


単数と複数(docs-refresh): 数を含む文言は `Msg::singular` で単数の文言と数の差し込みの位置を持ち、`Msg::fill` がその値が `1` のときに、今の言語の単数の文を選ぶ(使う側は今までどおり `fill` を呼ぶだけ)。英語に限らず、どの言語の台帳でも効く。日本語のように数で形が変わらない言語は、同じ文を書いておく。数の形が3つ以上ある言語(ロシア語など)を足すときは、選ぶ所がこの `fill` の1か所なので、そこに言語ごとの決まりを足す。

言語を足しても訳せる文の持ち方: 文を部品から組み立てない。`relative()`(base-view.relative)の「あと14日」「5日前」は、向き(先・前)と単位の組ごとに1文(`RelInDays`・`RelAgoDays` など)を台帳に持つ。「あと {0}」と「{0} 日」を別々に持ってつなぐ形は、向きで語順や格が変わる言語で訳せないので採らない。
## 画面のテーマ(SR-26・SR-27)

形と色の表は変更の記録 specs/_history/decidespec/_changes/2026-10-06-themes.md にある。

- 表: lib の `src/theme.rs` に `Theme`(設定の名前 `default`・`nord`・`solarized-light`・`dracula`・`gruvbox`・`pink-monster`・`dozy-pink`。`Theme::ALL` はこの順)と、役割ごとの RGB の `Palette`(`default` は None)。`to_indexed` は xterm の 256 色の色の立方体(16〜231)と灰色(232〜255)から最も近い番号(0〜15 は端末ごとに違うので使わない)。設定は最上位の `theme`。読めない値は型の警告(`Msg::WantTheme`)で既定のまま。
- 塗り方: `ui::draw` の最後に、描き終えたバッファを `ui/theme.rs` の `paint` が役割で塗り替える(後段の1か所)。反転のセルは下の帯の行(描く範囲の下から2行目)なら帯の色、ほかは選択の色にして反転を外す。反転でないセルは、前景が無ければ 1行目 → header・太字+下線 → colhead・太字 → strong・左端の `>` → mark・ほか → fg、背景が無ければ bg。一行おきの色(`ui/display.rs` の 237 / (58,58,58))は zebra の色に替える。セルが自分で持つ色(ためた値・一致・差分)と、自分の色を持つ反転のセルはそのまま。色なし(`ColorMode::None`)とパレットの無いテーマでは何もしないので、`default` の画面は1セルも変わらない。表の画面の列の見出しの行(`view::data_y` の1つ上)の太字+下線だけを colhead にし、ほかの太字+下線(入力ボックス・一致した名前)は strong。一行おきの色は「色なし」とみなして役割で塗り(反転なら選択・帯の色、fg の役のときだけ zebra_fg、地は zebra_bg)、選んでいるセルや `>` も zebra の行で役割の色になる。
- 右下の隅の1セルは塗らない: 描くのと同じく、端末の右下の隅を書くと自動折り返しでずれる端末があるため(ほかの最下行と右端は地の色で塗る)。
- 選ばなかった案: 各部品(表・帯・窓・ヘルプ・差分・カレンダー)にテーマを渡して色を付ける(部品が多く、取りこぼすと色の混ざった画面になり、`default` の不変を試験で守りにくい)。窓の枠だけの色(枠は行の文字と1つの文字列で、枠だけを見分けられない)。

## 部品の形と端末に従うテーマ(SR-36・SR-39)

- 設定: lib の `src/style.rs` に部品ごとの列挙(`named!` で名前と値の表を1か所に)と組(`Style::of`)。`[style]` は `preset` を先に当て、ほかの項目で上書き。カタログ(SR-38)と名前を突き合わせる試験の元もこの表。
- 値の部品: `ui/chips.rs` の `parts_named` が (値, 見せる文字) の並びを「前置き・要素ごとの切れ・区切り」の並び(`Parts`)にし、セルの文字(幅を数える)と描く所(`view::chip_spans`)の両方がこれを使う(幅と描きのずれを作らない)。切れの見た目は行の見た目に重ねる(`patch`)ので、一行おきの色や十字の選びの地を点や区切りが消さない。色は状態の英語の語(done・doing・blocked・todo など)なら意味の色、ほかは値の文字の指紋から(見せる文字が名前でも、色は値で決める。navigation.value-labels-cells)。
- セルの印と部品(cell-view.marks-on-parts。2026-11-25): `cell::shown` は値の前の印(`!`、読むだけの `#`)を `CellPart::Chips` の `mark` にも入れ、`chip_spans` は部品の前に印を描いてから残りの幅に部品を並べる。前は部品を値の並びから描き直していたので、文字の頭の `!` が札では消えていた。ためた値(`*`)・評価できない式(`?`)・読むだけのセルは部品にしないので、前から文字の印のまま。
- 札にするか(screen.rich-cells-table-wide。2026-11-25): `looks_like_select` に渡す行は、今のビューの行でなく `App::table_rows`(`.base` があれば `Base::table_rows` で全体の `filters` を通った行、無ければ読んだ行の全部)。ビューの行で決めると、行の少ないタブで値がくり返さず、同じ列がタブごとに札と文字に変わった。全体の絞り込みを見るのは、`.base` の表は全体の絞り込みが表の中身を決める(同じフォルダのほかの表の行を数えない)ため。判定の使い回し(`SelectsKey`)は前のまま。
- 窓の縁の外の札の切れ端(screen.window-chip-remnants。2026-11-25): 窓は表の行に字の単位で重ねる(`list::splice`)ので、窓が札の体を隠すと、縁の外に丸い端の字(U+E0B6・U+E0B4)だけが残った。窓ごとに直さず、描く直前(`ui::paint`)に `chips::drop_lone_caps` が、隣に体(端の字の色を地にした字)の無い丸い端の字を空白にする。どの窓(新しいノート・候補・操作の一覧・絞り込み・カレンダーなど)と、列の幅で切れた札にも効く。Nerd Font の無いときの端(`▐`・`▌`)は、グラフの棒の字と同じ字なので見分けられず、扱わない。
- 式の列の `#`(cell-view.formula-mark-no-color。2026-11-25): 式の列(`formula.*`)は列ごと読むだけで、見出しに `ƒ` がある(screen.formula-mark-prefix)。色のある表示では `cell::shown` が `#` を付けず、薄い表示(`locked` は残す)と見出しで見分ける。色を使わない表示では薄い表示が出ないので `#` を付ける。読むだけのノートと `file.*` の列は前のまま。
- 選び・線・タブ・帯は `ui/look.rs` の `select`・`rules`・`Look::tab`・`Look::band_key`。モダンな見た目(色あり・look = modern)のときだけ効き、classic と色なしは今まで(SR-33)。どの形でも画面の文字は変えない(見出しの下の線は下線、選びの `bar` も印は `>` のまま色で)。窓の枠は `popup::frame` が `frames` から(線なしも幅は同じ)。
- 端末に従うテーマ: `ui/termbg.rs`。`COLORFGBG` の地の番号(7・9〜15 を明るい)を先に見て、無ければ `/dev/tty` を raw にして OSC 11 と DA1 を続けて送り、`poll` で最大 200ms 待つ。DA1 の答えが先に来たら答えない端末とみなしてすぐやめる(遅れた答えを表のキーと読まないため)。依存を足さず、std がリンクしている libc の `poll` を直接呼ぶ(`ui/external.rs` の `signal` と同じ形)。Windows は問い合わせない。解決は起動のとき1回(`main::open_app`)。
- 選ばなかった案: 選びの `bar` で印を `▌` にする(SR-33 の文字を変えない決まりと、`>` を確かめる多くの試験とぶつかる)。状態の日本語の語の表(SR-23 でコードに日本語のリテラルを置かない)。

## 設定の形と範囲ごとの上書き(CLI-3・CLI-11・CLI-12・CLI-20・CLI-21・SR-43・SR-44)

要求と調べた不整合は変更の記録 specs/_history/decidespec/_changes/2026-10-10-config-v2.md。

- **項目の表を1つに**: lib の `src/schema.rs` に、項目ごとの `Item { path: "look.style.status", ty, ty_ja, default, example, scope: Scope::Global | Scope::Profile, en, ja }` を並べる(`config_items.rs` を置き換える)。読み取りの知らない項目の判定・`--print-config`(区画ごと)・`--print-config --resolved`・docs/config*.md と突き合わせる試験・カタログの試験が、この表を見る。新しい項目は表に1行足し、読み取りの `match` に1つ足す(足し忘れは試験で落ちる)。
- **旧い名前の表**: 同じファイルの `LEGACY: &[(old, new)]`。読み取りは、最上位で旧い名前を見つけたら、表を新しい形に写して(`legacy::lift`)から読み、「旧い名前は新しい道筋に移った」と警告する。新しい道筋にも書いてあれば新しいほう。`--migrate-config` は同じ写しをして `toml::to_string` で出す(注釈は移らない)。docs の旧い名前の表もこの表と突き合わせる。
- **プロファイル**: `src/profile.rs` の `Profile`(どの欄も Option の部分の値): `look: LookLayer`(theme・preset・mode・cells・style の部品ごと・columns・colors の役割ごと・values)、`display: DisplayOverride`(今の型。tabs を `Tabs3` にする)、`dates: DatesLayer`、`edit: EditLayer`、`new_note: Option<NewNote>`、`use_: Option<String>`。`Profile::read(table, file, prefix, warns)` が config.toml・ui.toml・ワークスペースの印・workspaces.toml・views.toml のどれでも同じ読み方をし、`Profile::write(&mut toml::Table)` が画面の書くファイルに書く。
- **決まった値**: `profile::resolve(layers: &[Layer], templates) -> Resolved` が、既定から狭い範囲へ順に重ねる。`Layer { origin: Origin, profile: &Profile }`、並びは 既定 → config.toml → ui.toml → ワークスペース → 表(手) → 表(views.toml) → ビュー。ただし `[look]` だけは、印から来た層(`Origin::by_author`。`Scope::layer`・`table_layer` が印の範囲で立てる)を先に、人の層をそのあとに重ねる(screen.profile-scopes-marker-look。`resolve` は `[look]` を `apply_look` で、ほかを `apply_rest` で2回に分けて重ねる)。印を `Place` の新しい値にしないのは、`[display]` などは今までどおりワークスペースの範囲で、出どころの範囲(「(この表)」・`--print-config --resolved` のコメント)も変えないため。却下: 印の層を並びの先頭に動かす(`[look]` のほかの区画まで負けてしまう)。各層は先に `use` のテンプレートを敷いてから自分の値を重ねる(テンプレートの中の `use` は警告して読まない)。組を持つ層で `style` をその組の形に戻し(それより前の層の部品の形を捨てる)、テーマを持つ層で役割の色を空に戻す(値の色は残す)。`Resolved` は今の画面が使う型(`Style`・`Colors`・`Cells`・`Display`・`DateFormat`・`WeekStart`・`NewNote` など)と、項目の道筋ごとの `Origin`(SR-43 の「(この表)」と CLI-21 のコメント)を持つ。
- **Config**: アプリ全体の項目(`language`・`editor`・`poll_ms`・`terminal`・`workspace`・`keys`)と、config.toml のプロファイルとテンプレートを持つ。ui.toml は `UiFile { profile, templates, nerd_font }`。App は全体・ワークスペース・表の層を持ち、ビューを選ぶたびにビューの層を足して `resolve` し直す(`App::apply_profile`)。表を切り替えるときは main が開き直すので、範囲(WS-6)が決まったあとの `App::start` で層を集める。`add_frontmatter` は表で決まるので、読み込みの前(`start` の中)に源へ渡す。
- **警告の形**: `Msg::SettingWarn = "{0}: {1}: {2}"`(ファイル・道筋・理由)。理由は `Msg::Want*`・`Msg::Moved` など。どのファイルの読み取りも `warn(file, path, reason)` を通す。
- **画面の書く場所**: 全体 → ui.toml(`config::write_atomic`)、ワークスペース → workspaces.toml のそのワークスペースの区画(`workspace::save_profile`。文字の区画を置き換える今の書き方で、手のコメントを残す)、表 → views.toml の `[[table]]` の `look` など、ビュー → ビューの設定(`Settings` に `look`・`dates`・`edit`・`use_` を足す)。
- **見た目の区画のカスタム(screen.look-section-custom)**: 写しの `LookPick` に `custom` と役割の色 `colors` を持ち、カスタムなら区画に元のテーマと16の役割の行を足す(項目の番号は 4 が元のテーマ、5 からが役割で、テンプレートの行はそのあと)。反映でテーマか役割の色を変えていれば、選んだ範囲の層に元のテーマと写しの役割の色を丸ごと書く。テーマを書いた層は広い範囲の役割の色を使わないので、広い範囲から来ていた色も同じ層にそろえて持たせる。名前のテーマを選ぶと写しの色を空にし、反映でその層の役割の色も外す。値の色(`[look.colors.values]`)は触らない。却下: `theme = "custom"` のような新しいテーマの名前にする(設定の形が増え、元のテーマが残らない)。
- 選ばなかった案: 範囲ごとに丸ごとの Config を作って上書きする(どの値がどこから来たかが分からず、組とテーマの決まりが書けない)。`serde` の derive で全部を読む(知らない項目・型の違いを「ファイル: 道筋: 理由」で出せず、旧い名前を写せない)。

## 型と決まりの問題(CE-36・CV-9・NV-30・CLI-23。type-check。2026-10-14)

要求は変更の記録 specs/_history/decidespec/_changes/2026-10-14-type-check.md。

- **判定は1か所**: lib の `src/rules.rs` の `problem(kind, rule, value)` が、型(`types::fits`)→ 必須 → 選べる値の順で最初の理由を返す。画面(`App::problem`。ためた値を重ねた値)と `--check`(`rules::check`。ノートの全部のキーと決まりのある列)が同じ関数を使う。却下: 画面の cell.rs の判定を CLI に写す(食い違いが生まれる)。
- **一緒に入れる値(`then`)**: `[rules.<列>.then]`(`Resolved.then: 列 → 値 → 列 → 入れる値(`rules::Fill`: 文字と if_empty)`。`Rule` の形は変えない)。画面(`ui/then.rs` の `App::follow`)が変数を埋めて列の型で読み、`Changes::set_each_then` が元の直しと同じ1手(`Step`)に入れる。だから取り消し・やり直し・差分は今までの道のまま。前のためた値で入れた列は、別の値に直したときに外す。却下: 保存のときに入れる(差分に出る前に書かれ、取り消せない)。 空のときだけ入れる値(cell-edit.then-if-empty)は、`Changes` が入れる前に、入れる先の列の今の値(ためた値があればその値、無ければ読んだ値)が空かを見る。画面でなく `Changes` で見るのは、セル・一括・カンバン・子を一緒に完了にする道が全部 `set_each_then_with` を通るので、1か所で全部に効くため。別の値に直したときに外すのは、一緒に入れた値と同じ値がたまっているときだけ(先に手で入れた値を、入れなかったのに外さない)。書き方は表の形 `{ value, if_empty = true }` にした。却下: 入れる文字の印(`"{date}?"`。`?` で終わる文字と区別できず、書き落としても気づけない)と、別の表 `then_if_empty`(同じ値に2つの表が効くと、どちらが勝つかを覚える要がある)。書き誤った表は「いつも入れる」と読まずに、その列を入れない(残したい値を上書きしないため)。
- **値の字と名前(`icons`・`labels`。navigation.group-values-icon。2026-11-24)**: `RuleLayer` の `icons`・`labels`(値 → 文字)を `Resolved.icons`・`Resolved.labels` に重ねる(`then` と同じく、書いた範囲のものが丸ごと代わる。`Rule` の形は変えない)。見せる文字は `Resolved::value_shown` の1か所で作り、まとまりの見出し(`App::group_shown`)とカンバンのレーンの題と、セルの見せ方(下の「値の名前をセルにも」)が呼ぶ。書く値と書き出しは値のまま。同じ `[rules.<列>]` に `values` があれば、そこに無い値の項目は読むときに `ファイル: 道筋: 理由` の警告にして捨てる。`values` が広い範囲にあるときは、読むときには確かめず、`value_shown` が重ねた `values` に無い値を出さない。却下: 重ねたあとに警告する(起動の途中の、範囲の層を集める前の重ねで、狭い範囲の `values` を知らずに誤って警告し、表で `values` を狭めただけの正しい設定にも警告が出る)。
- **まとまりの順(navigation.group-values-order)**: まとめる列に `values` があれば、候補の順(降順は逆)→ 候補に無い値(今までの値の順)→ 値の無い行。ビューの設定でまとめ直すときは `settings::apply_with`(`apply` は候補を見ない形のまま残す。前からの試験と呼ぶ側を変えないため)、`.base` の groupBy のまとまりのままのときは `settings::order_groups` が `Base::build` の結果のまとまりを並べ直す(`Base` は設定を知らないので、画面の側で重ねる)。`--print` は設定を重ねないので、今までの値の順のまま。
- **決まりの置き場**: 表のプロファイルの `[rules.<列>]`(`Profile.rules: 列 → RuleLayer`)。`[types]` と同じく範囲を重ね、列ごと・項目ごと(`required`・`values`)に狭い範囲が勝つ(`Resolved.rules`)。決まりは書き戻しを止めない(型の CV-2 と違い、合わない値でも書き戻しの安全に関わらないため)。
- **数は組み立てのときに**: 下の帯の問題の数は、組み立て直し(`rebuild`)の終わりに今の行と見えている列で数えて `Built.problems` に持つ。描くたびに数えない(1万行で重い)。問題のある行だけの絞り込みは `overlay` の `passes` に重ねる(簡易の絞り込み・同じ値と同じ段)。そのため列の型(`kinds`)は `overlay` の前に決める。
- **CLI の列の型**: 画面と同じく `[types]` → `Source::kind`(ルートフォルダの型の設定か、読んだ全部のノートからの推定)。行の番号は `frontmatter::key_lines` のキーの文字の位置から。

## 木の画面(TR-1〜TR-7。tree-view。2026-10-22)

要求は変更の記録 specs/_history/decidespec/_changes/2026-10-22-tree-view.md。

- **切り替えのキーは `T`**: 表のモードで空いていた大文字(関係マップの `R` と並べて、窓を開かずに切り替える)。木の中でも `T`・`Esc`・`q` で表に戻る。木の中のキーは別のモード `tree`(`[keys.tree]` で割り当て直せる)。`o` で URL を開き、`x` で行の操作の窓、`Tab` で欄を切り替える。
- **木を組むのはライブラリ**: `src/folder_tree.rs`(画面に依存しない。値を区切りで切る・途中の階層を作る・下までの行の数・名前の順・「(なし)」は根の最後・行の並び・開いてよい URL)。NV-27 の `src/tree.rs`(行どうしの親子)とは別。
- **設定は印を開いた表のパスで引き直す**: `workspace::marker_tree(開いたもの)` が印の `[[table]]` の `tree` を読む(`inline_base` と同じ形)。`WsTable`・`Scope` に欄を足さない(錠のある試験がその形を直に作るため)。範囲の読み(`read_tables`)は、印のときだけ `tree` を知らない項目にしない。階層の列の有無は読み終えてから(`load_step` の終わり)確かめて警告する。
- **木は今のビューを通った行(`App::rows`)で組む**: ビューの絞り込み・where・計算の列と、簡易の絞り込みが効いた行。外で変わったら(`poll`)組み直し、開閉と選びは鍵(根からの道)で保つ。
- **段組み**: 描ける幅 76 以上(80 桁の端末を含む)は左右に並べ、未満は1つずつにして上のタブの行と `Tab` で切り替える。窓は関係マップの `panel`・`side_by_side` を使う。
- **URL を開く口は差し替えられる**: `external::Opener`(本物は macOS `open`・Windows `rundll32 url.dll,FileProtocolHandler`・ほか `xdg-open` に1つの引数で渡し、シェルを通さない。終わりを待つ糸で子を残さない)。試験のビルドでは渡された値を覚えるだけ。開くのは `folder_tree::web_url` を通った `http://`・`https://` だけで、空白と制御文字を含む値も開かない。
- **コピーと貼り付け**(`ui/external.rs`。output.copy-tsv・output.paste): `y`・Ctrl+C は値だけ(行を選んでいれば今の列の値を改行でつなぐ)、`Y` は行を見出しつきのタブ区切り。貼り付けは端末の bracketed paste(`ESC [ ? 2004 h` を `main::enter` で入れ、`restore` で切る)で、文字のまとまり `Event::Paste` として受ける。入れなければ、貼った文字が1字ずつキーとして届き、表の画面では思わぬ操作になる。表の画面では `paste_cells` が改行とタブで分け、選んだ行から下のノートの行(見出しと空きの行は飛ばす)と、選んだ列から右の列へ、`Changes::set_cells` で1手でためる。値は打ち込みと同じ読み方(`paste_text_value`)。入力の欄のある画面では1行目を打ったのと同じに入れ、ほかの画面では捨てる。
- **伸ばした行の高さは全部の列で決める**(`ui/rows.rs`。cell-view.row-lines-height): 高さは `height_cols`(ビューの全部の列と、その列の幅)で数える。見えている列だけで数えると、横に送って長い値の列が画面の外に出たとたんに行が縮み、行の位置が動く。右端で切って見せる列は、切った幅で折り返すと高さに入りきらないことがあるので、その行の高さ(`RowCtx.max`)までで切って `…` を付ける。数えるセルが増える(見えている行 × 全部の列)ので、遅いと分かれば行ごとの高さを覚える。
- **ガントの操作の帯・期間・棒を動かす**(`ui/gantt.rs`): 操作の帯は窓の下の縁の行に重ねる(`controls_line`。行と時間軸の位置を変えないため。項目の桁は描画とクリックで同じ `control_spots`)。期間は新しい状態を持たず、左端の日(`origin`)と単位(`unit`)を決めるだけ(`gantt_set_range`: 期間の最初の日から最後の日までが幅に収まる最も細かい単位にして、左端を単位の境に切り下げる)。`Gantt.range` は `r` の次の期間を決めるためだけに覚える。棒を動かす・伸ばす(`gantt_edit`)は、書く列と値を先に全部決め、どの列も直せると確かめてから `Changes::set_cells` で1手でためる(片方だけ書かず、取り消しも1手)。日時の値は頭の10字(日)だけを替える。`days` の設定では、動かすときは始まりだけ、伸ばすときは日数だけを書く。取り消しとやり直しはカンバンと同じ道(`kanban_undo`)。期間と単位をビューごとに覚えるかは未確定(`tbd~gantt.range-remember`。今は覚えない)。
- **新しいノートの窓の頭とクリック**(`ui/new_note.rs`): `[new_note] name_head = "hide"` なら、名前の雛形を埋めた文字を `Flow.head` に持って名前の欄には出さず、確定のとき `full_name` で最後の名前の部分に付ける(打った文字は `Flow.typed`、作る名前は `Flow.name`)。欄の行のクリックは、移る先を `Flow.jump` に置いてからふだんの確定を呼ぶ(確かめと答えの記録を1つの道にするため。読めない値なら移らない)。キーの行のクリックは、描くのと同じ並び(`key_actions`)から桁を割り出して、その動作を `apply` に渡す。リストの選択の窓の上のクリックは、下の欄の行に通さない(`listpick::covers`)。
- **切って見せた URL の行き先(cell-view.url-full-target)**: 端末は画面の文字から URL を見つけて開くので、`…` で切った文字のままだと途中までの URL が開く。描く側(表の `view::data_row`・木の右の欄・詳細)が「この桁のこの文字は、この URL」を `App.url_spots`(`ui/hyperlink.rs`。描く関数は `&App` なので RefCell)に覚えさせ、`main` が描いたあとに、その桁の字を OSC 8 で包んで書き直す(ratatui のバッファに制御の列を入れると字の幅の数えが狂うので、描画には混ぜない)。窓や区画の中で覚えた桁は `shift` で画面の桁に直す。描き終えたバッファの字が覚えた文字と違う所(窓が重なった所)は包まない。包み直しは毎回の描画でする(ratatui が字を書き直すと行き先が外れるため。同期の更新の中なのでちらつかない)。行き先にするのは `web_url` を通り `url_refusal` に当たらない値だけで、`TERM=dumb` では書かない。
- **パレットは木の上に重ねる**: 木から開いたパレットは閉じると木に戻り、木に無い動作を選んだら表に戻ってから行う。
- **新しいノートの窓の欄はセルの編集と同じ入れ方**(cell-edit.new-note-form-same-input。2026-11-25): 欄の候補は `open_step` が `input.rs` の `open_at_inner` と同じ順(リンクの列 → 決まりの values → ノートの値。チェックボックスは true・false・なし)で作り、開いた直後に打つと置き換える `fresh` も同じ条件で立てる。リンクの書く形と答えたリンクの見せ方は、まだ無い作るノートのパス(`note_row`)を行の鍵にして `link_list`・`link_cands`・`link_text_new` に渡す(相対のパスの形でも、作る場所から書く)。ノートの値の候補が0件のテキストの列だけは、セルと違って「なし」だけのリストを出さない(cell-edit.new-note-form-rule-values)。
- **候補の窓の Esc**(cell-edit.new-note-form-esc-popup): 値の候補のリストとカレンダーは、Esc で入力の `list`・`cal` を外すだけ(入力は打った文字のまま開いたまま。入り直すと作り直す)。リストの欄の付け外しの窓は、`Revert` で開いたときの付け方に戻して `Flow.rest` を立て、描かない(`listpick::geometry` が None。文字は捨てる)。`rest` のあいだの Enter・Space で窓を戻し、Tab はその値で次の欄へ、Esc は窓を閉じる。窓を閉じる Esc は確かめない(セルの入力の Esc と同じ。cell-edit.new-note-form-esc)。
- **欄の候補の窓の置き場所**(cell-edit.new-note-form-popup-place・navigation.detail-popup-place): 値の候補のリスト・カレンダー・付け外しの窓は、新しいノートの窓では `popup::beside` で窓の四角(`new_note::form_anchor`)に重ねない所を選ぶ: 右(右の空きの1桁のあと。幅は 24 桁まで縮めてよい)→ 下 → 上。どこにも入らなければ今の欄の行の下(上)に、値の桁より右に重ねる(`popup::right_of`)。右を先にするのは今の欄の行のすぐ横で選べるため。前はセルの候補と同じく入力の欄の下に置き、窓が広いと画面の右の端に合わせて左へずらしていたので、欄の名前と窓の下の部分を隠した。詳細の表示は画面の全部が欄なので、置き場所は今までどおり(値の欄の下か上)で、左の縁だけを値の桁より左に出さない(`right_of`)。表のセルの候補は今までどおり(左へずれても隠すのはほかのセルで、直しているセルと列の見出しは隠さない)。
- **理由の印**(cell-edit.new-note-form-error-clear): `Flow.error` は作るときの理由の欄と文で、その欄の値を決めたら(必須の欄がまだ空なら残して)消す。名前の欄で打っている間の理由はメッセージ行だけに出す(作れない名前は確定で止めるので、欄の印にはならない)。
- **頭を隠すときの `{name}`**(cell-edit.template-vars-name-head): 変数の名前は、頭を隠すとき(`Flow.head` が空でない)は打った名前 `Flow.typed`、ほかは作る名前 `Flow.name` の最後の部分。名前の欄の見せ方も同じ。`--new-note` も、頭を付ける前の渡した名前で埋める。
- **`--new-note` の表のフォルダ**(cli.new-note-table-folder): 渡したフォルダの上に印があり、自分は印の根でないとき、印の表のうち `[new_note] folder` か `.base` の絞り込みの `file.inFolder`(ルートフォルダから)がそのフォルダを指す最初の表を選び、作る場所の根を印の根(渡したパスからフォルダの段だけ上がったもの)にして、その表のパスで設定を重ねる。出すパスは渡したフォルダにつなぐ。画面と画面を出さない起動で表のフォルダを直に開いたときも、`marker_paths` が同じ `table_of_folder` でその表(`.base` の表だけ。フォルダの表は、そのフォルダを開けばもうその表)に置き換える(cli.launch-table-folder)。本文の雛形がフォルダなら、画面で選ばずに作ったときと同じ最初の雛形(`newnote::default_body`。`.md` の無いフォルダは本文なし。画面の窓も同じ関数を使う)。
- **`--table` で表を名前で選ぶ**(cli.table-open・cli.table-commands): 印は開くパス(ファイルならそのフォルダ。無ければ今のフォルダ)から上へたどる(`open` と違い、根でなくてよい)。見つけた印の根は、開くパスから段を数えて上がった書き方(相対なら相対。今のフォルダからなら `..`)で言い直す(`marker_root_of`)。印の `open` が根を開くパスにつないだ表のパスと同じ形になり、`--with-path` と `--new-note` の出すパスが、開いたフォルダから見た道になる。リンクで実体が食い違うときだけ実体のパス。`-w` があれば `workspaces.toml` のその表から選ぶ。命令ごとの開くパスは `headless_paths` の1か所で決め(`--table` → `-w` → 印の `open` と表のフォルダ)、画面は `--table` を印の `open` と同じ道に置く(`.md` を渡せば、その表でその行を選ぶ)。旗の値は `Options` に足さずに外に持つ(main の試験が `Options` を直に組み立てるので、項目を足すと main の試験が組み立てられなくなる)。補完に表の名前は出さない(`tbd~cli.table-completion`。補完の定義は引数の定義から前もって作る)。
- **`--check` の全部の行**(cli.check-problems~2): `--view` が無ければ `ViewOut::CheckAll` にし、`.base` の最初のビューの絞り込みと件数の上限を外して(`Base::open_first_view`。全体の絞り込みは残す)表の全部の行を組む。印の表の `view` も当てない。いつも隠す条件は今までどおり当てる(`tbd~cli.check-hidden-rows`)。

## 台帳のスターター(ST-1〜ST-6。starter。2026-10-24)

要求は変更の記録 specs/_history/decidespec/_changes/2026-10-24-starter.md。

- **呼び名**: 「型」は列の型・画面の型に、「テンプレート」は `[templates]`(CLI-3)と新しいノートの雛形(CE-32)に使っているので、「スターター」(`starter`)。
- **組み込みはファイルの木のまま埋め込む**: リポの `starters/<名前>/` を build.rs が歩き、`OUT_DIR/starters.rs` に `include_str!` の表(`BUILTIN`)を書く(文言の台帳と同じ道)。build.rs が、`starter.toml` の `[name]` に台帳の全言語があること、`ledger/.mdgrid/workspace.toml` があること、`<部分>.<符号>/` の符号が台帳の言語であることを確かめ、違えばビルドを落とす。却下: Rust の定数に手で書く(中身は印の TOML と Markdown で、ファイルのままの方が人が読め、ユーザーのスターターと同じ形になる)。却下: 実行ファイルの隣に置く(1つのバイナリで配る形に合わない)。
- **組み込みもユーザーのものも同じ `Starter`**: `src/starter.rs` が `BUILTIN` と設定の置き場の `starters/<名前>/` を同じ型に読む(組み込みだけの特例を作らない)。名前と説明は `starter.toml` に持つ(文言の台帳に持つと、ユーザーのスターターが同じ仕組みで説明を持てない)。一覧はユーザー → 組み込みで、同じ名前はユーザーが隠す。読めないものは CLI-3 の形で警告して飛ばす。
- **言語の変種は丸ごと差し替え**: `ledger.ja/` があれば `ledger/` を使わない(重ねると見本のノートの名前が英語と日本語で混ざる)。英語(接尾なし)が正本。
- **作るのは核**: `starter::create` が、書く先を全部並べて1つでもあれば何も書かず(WS-7 と同じ `WsInitExists`)、`ledger/` は素のまま、`samples/` は `newnote::expand` で変数を埋めて `newnote::create` で書く。CLI(`ws_cli`)と画面(`src/ui/starter.rs`)はこれを呼ぶだけ。`workspace::init` は広げない(印の読み書きの核で、マニフェスト・一覧・写し・変数は関心が違う)。
- **雛形は根の `.mdgrid/templates/`**: CE-32 の `body` は開いたフォルダの中のファイルだけなので、どのスターターも根から開く表を `[table.base]` にし、雛形を根の `.mdgrid/templates/` に置く。people の `People` はフォルダの表なので雛形を持たない(持つなら `People/.mdgrid/` になり、見本のノートと見分けにくい)。
- **people の表の形**: 根の `[table.base]` は1つ(Meetings)にし、People はフォルダの表にする。リレーションは `.base` を根の表とみなすので、根に2つあると見分けられない(studio の見本と同じ形)。
- **組み込みの印に `name` は書かない**: 作った先のフォルダの名前がワークスペースの名前になる(`read_marker`)。
- **見本 `examples/bookmarks` はスターターから作る**: `tests/test_starter_examples.rs` が一致を縛る(見本の日付は固定なので、いつ作っても同じ)。
- **画面の流れ**: `Ask::StarterPick` → `Ask::StarterDir`(欄にスターターの名前を入れて始める)→ `Ask::StarterSamples`(y/n)→ 作って、`marker_open` の表を `switch_to` で開き直す(登録した表を開くのと同じ道。ためた変更の確かめもそこで効く)。

## 文字の幅

- unicode-width と書記素のまとまり(unicode-segmentation)で数える。East Asian Ambiguous(○● など)は既定で1とし、設定で2にできるようにする。
- 絵文字は幅が端末ごとにぶれるので、表の飾りには使わない。

## 表の行の高さ(CV-8。row-lines。2026-10-12)

- 行の高さと画面の位置の計算は `src/ui/rows.rs` に集める(描く・クリック・入力の欄と窓・スクロール・ページ送り)。`data_y + row - top` のように行を画面の1行と数える式を、ほかの所に書かない。
- `row_extra_lines = 0` では、rows.rs のどの関数も今までの算術(1 slot = 画面の1行)をそのまま返す。0 で画面が1字も変わらないことを、計算の道を分けて守る。
- `app.top` は slot の添字のまま(行の途中から描かない)。画面より高い行は上から入る分だけ見せる。
- 列の幅は先に決め、行の高さはその幅で数える。幅を測る行は「1 slot を1行と数えたときに見える範囲」にする(実際に見える行で測り直すと、高さと幅が循環する)。
- 行の高さは、見えている列のセルで数える。改行を残した文字は、`cell.rs` のスレッドローカルの切り替えで `shown` から作る(`shown` に引数を足さない。改行を残すのは表の行を描くときだけ)。

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
  `place_with`)、上の縁(`top_edge`)、当たり(`hit`)、表の行への重ね方(`blit`)、一覧の選びの上下(`step_sel`)、
  下の縁のキー(`key_hints`・`hints_width`・`widen_for_hints`・`bottom_hints`・`hint_at`・`on_bottom`)と
  「[ 決める ]」の行の字(`decide_label`)。新しい窓はこれを使い、置き方の計算を写さない。値の候補の窓(`ui/list.rs`)も上と下の縁(`FRAME`)で閉じ、
  当たりは `hit`(2026-11-25 までは入力の欄の側の縁が無く、窓が欄より広いと、入力の欄の行の右の列の札や文字が
  窓の中に続いて見えた)。
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
- CHANGELOG: コミットの書き方(`実装:`・`修正:` などの書き出し)は今のままにし、リポの根の `cliff.toml`(git-cliff)で書き出しを CHANGELOG の節に写す(`実装`・`追加` → Features、`修正` → Fixes、`文書`・公開リポの PR の題の `docs` → Documentation、`specs` → Specification、ほかは出さない)。CHANGELOG.md はリリースの作業(Q-6)で生成する。新しい書き出しを使うときは cliff.toml に足す(tests/test_cliff.rs が履歴の書き出しの漏れを見る)。
- TUI 本体(配る実行ファイルの言語・フレームワーク)は Rust だけにし、cellops のような Node との二重持ちはしない。試験の道具はこの限りでなく、通しの試験(本物の疑似端末で起動 → 操作 → 画面とファイルを確かめる)には Node の Microsoft tui-test を `e2e/` に分けて使ってよい(人の判断、2026-10-01)。単体の試験とゴールデンは `cargo test` に置く。

## 配布(入れ方)

入れ方はどれも、タグを push すると走る `.github/workflows/release.yml` の成果物から出る。アーカイブの名前(`mdgrid-<タグ>-<target>.tar.gz`、Windows は `.zip`)と中の並び(同じ名前のフォルダの下に `mdgrid`・`mdgrid.1`・`completions/`)を変えるときは、下の binstall の指定も同じ変更で直す。

- GitHub Releases のアーカイブと crates.io(`cargo install`): release.yml の build・release・crates のジョブ。
- cargo binstall: `Cargo.toml` の `[package.metadata.binstall]` に、アーカイブの URL と中の実行ファイルの場所を書く。binstall は crates.io に出た版の Cargo.toml を読むので、指定はその版を出したときから効く。
- Nix: リポの根の `flake.nix`(nixpkgs の `rustPlatform.buildRustPackage`。依存は `Cargo.lock` から取るのでハッシュの書き換えは要らない)。ソースは crate と同じファイルだけを渡し、試験は回さない(試験は `cargo test` の CI が見る)。`flake.lock` は nixpkgs の版を留めるので、`rust-version` を上げて nixpkgs の Rust が足りなくなったら `nix flake update` で進める。ci.yml の nix のジョブが `nix flake check` でビルドを見る。タグを付けない `github:S6U5/mdgrid` は main の先頭(まだ出していない変更も入る)なので、README と説明書は `github:S6U5/mdgrid/v<版>` とタグを付けて書き、ほかの入れ方と同じくリリースした版が入るようにする。版を上げたら文書の番号も直す(`tests/test_install_docs.rs` が `Cargo.toml` の版との食い違いを見る)。
- まだ無いもの: Homebrew(tap を持つと、formula を書き換えるトークンと守るリポが1つ増えるので見送った。人の判断、2026-11-18)、Windows の Scoop・winget、Linux の .deb・.rpm・AUR、1行のインストーラ。

## セッション(CLI-6〜CLI-10)

表示に名前を付けたもの(cli.session-*。2026-11-21 に作った)。設定の置き場の `sessions.toml` に `[sessions.<名前>]` で置く(ノートのフォルダ・`.base` には書かない)。形式と、選ばなかった案(`.base` として書き出す・`config.toml` に混ぜる)は変更の記録 specs/_history/decidespec/_changes/2026-10-01-sessions.md にある。

- **置き場所**: ファイルの読み書きは核の `src/session.rs`(`Session`・`Filter`。画面に依存しない)、画面に当てる・保存するのは `src/ui/sessions.rs`、`-s` と `--sessions` は main。核(base・changes・source)はセッションを知らない。記録の案の `Launch` は作らず、`-s` は main でセッションの開くパス(`.base` があればそれ、無ければ `folders`)を起動の引数に足し、`--readonly` を重ねて、ふだんの起動の道(`open_app`)を通す。`Options` の項目は増やさない(試験が構造体の字面で組むため)。
- **形**: `folders`・`base`・`view`・`columns`(見せる列の並び)・`hidden`・`readonly` と、`[[sessions.<名前>.filters]]`(`kind` は `text` = navigation.search の `\`・`value` = navigation.same-value と頻度表(`by_keys`・`list` も)・`expr` = navigation.expr-filter・`cond` = condition.digit-keys・`show` = condition.hide-toggle で止めたいつも隠す条件)と `[[sessions.<名前>.sort]]`(`key`・`dir`)。絞り込みは画面で掛けた種類のまま持つ(打った文字の絞り込みは `.base` の式1つに写せないため。sessions-filters)。ビューの設定(絞り込みの窓 NV-29・まとまり)は持たない(見た目の状態にある)。パスはホームの下なら `~/…` で書く(dotfiles で持ち運べるように)。ASCII でない名前は TOML の決まりで引用符が付く(`[sessions."進行中"]`)。
- **当て方**: main は開いてから `select_view_named` でビューを選び(無ければ先頭のビューと理由。`--view` と違って止めない)、`App::apply_session` が、列の並びと隠す列を見た目の状態の写し(`saved`)に入れて列を組み直し(列が現れるたびに `add_column` が引くので、見た目の状態より先に効く)、並びをビューの設定の並べ替え(`settings.sorts`。見出しからの並べ替えと同じ置き場)に、絞り込みをそれぞれの置き場(`filter`・`same`・`cond`)に入れる。今の表に無い列は、読み終えてから(`select_pending`)まとめて警告する。読めない式・設定に無い名前の条件は、その絞り込みだけ使わずに警告する。
- **見た目の状態との関係**: セッションで開いた画面も、ふだんどおり終えるときに見た目の状態を書く。なので、セッションの列と並べ替えは、そのフォルダを素で開いたときにも残る(画面で変えたことと同じ扱い。セッションで開いたときだけ書かない、という特例は作っていない)。
- **画面から保存**: パレットのコマンド `save_session`(`READ_COMMANDS`。ノートを書かないので読むだけの起動でも出す)。名前 → 同じ名前があれば置き換えの確かめ(`Ask::SessionOverwrite`。y のほかは元のまま)→ 書く。名前の欄には開いたセッションの名前が入って始まる。空の名前と制御の文字の名前は断る(`--sessions` の1行を壊さない)。書くときは、ファイルを見出しで区画に分け、そのセッションの区画(`[sessions.<名前>]` と、その `[[...filters]]`・`[[...sort]]`)だけを置き換える(手で書いたコメント・ほかのセッションは文字のまま残る)。書いた結果を読み直して狙った中身と違えば、中身から書き直す。今のファイルが壊れていれば書かない。書くのは一時ファイル → 名前の変更(`config::write_atomic`)。
- **起動の失敗**: 無い名前(ある名前を並べる)・壊れた `sessions.toml`(行の番号)・開くフォルダが無い・`-s` とパスの引数を一緒に渡した、は理由1行と終了コード 2。`-s` は `--view`・`--raw`・`--continue`・画面を出さない命令と一緒に受けない(clap の conflicts)。知らない項目と型の違う値は警告にとどめ、`-s` の起動では画面の警告に、`--sessions` では標準エラーに出す。
- **開き直す命令**: セッションで開いた表も、Herdr への命令と `--continue` の最後の画面は、ふだんどおり開いている表とビュー(`Reopen`)で作る(`-s <名前>` にはしない)。立て直しで戻るのは表とビューまでで、セッションの絞り込みは戻らない。

## 説明書とサイトの並びと文(SR-49・SR-50。views-page / docs-reading-order)

説明書の章は読む人の順(どう見えるか → 見せ方の一覧 → はじめかた → 手引き → 考え方 → 見た目 → 困ったとき → 参照)で、`scripts/pages.py` の `PAGES` と `docs/manual/<言語>/index.md` の目次が同じ順を持つ。見せ方の一覧は説明書の1ページ(`views.md`)にし、入口と同じ形の別の HTML にしなかった(説明書なら mdBook の目次と検索に入り、リポでも読める)。画像は場面の台本の id で指すので、場面が無い見せ方(親子の字下げ)は画像なしで録画へのリンクにした。画面の型や見せ方を足す変更は、この一覧に画面1枚と1項を足す。機能の一覧は別のページ `features.md`(SR-51。見せ方の一覧の次の章)にし、先頭に「場面で見る」(4つのスターター)を置いて、画面の型の一覧とは読む目的を分けた。機能を足す変更は、こちらに画面1枚と1項を足す。文の決まりは `docs/writing.md`(世間のナレッジ: Wikipedia の「Signs of AI writing」、日本語の AI 文の癖、Diátaxis)に置き、AGENTS.md から指す。

サイトの入口(`scripts/pages.py` の `landing()`。specs/_history/decidespec/_changes/2026-11-13-site-landing.md)は、`docs/writing.md` の読む人の順をそのまま段にする: 見出しと入れるコマンドと本物の画面3枚(Markdown のフォルダ・CSV のファイル・つながった表)→ データベースとして(絞る・並べる・まとめる・ビュー・値の分布・1枚を見る)→ リレーション → 分割した画面で AI の隣の pane に置く(左はエージェントの図、右は見本 ai-human の本物の画面)→ ターミナルで完結する(端末との相性)→ 書くのは直した値だけ(仕組みの図)→ 1分で試す → 自分の場面(4つのスターター)→ 見た目(テーマ)→ 次に読むもの → もう一度入れるコマンド。前に出す軸は、人が決めた mdgrid の一番の良さ(md や CSV をデータベースとして扱える・リレーション・1枚を見る画面・CLI で軽くターミナルと相性がいい)。段の見出しの文は人が書く。主役は撮った場面の SVG で、ブラウザの中で触れる図(ノートのファイルと表)は仕組みの説明に限り、図だと断って置く。図のノートは見本 `examples/studio/Tasks` の本物のファイルを作るときに読む(見本が変われば図も変わる)。見た目(`scripts/site/site.css`)と入口の動き(`scripts/site/landing.js`)は別のファイルに書いて各ページに埋め、外のフォントや部品は読まない。色は本物の画面の色(地・青緑・黄)から取った。JS が無くても画面・文・コマンドは全部読める形にし、動きは `prefers-reduced-motion` で止める。入口の部品はサイトの検査(`scripts/check_site.py`)が公開の前に触って確かめる。

## ガントの画面(GA-1〜GA-7。gantt)

木(TR)と同じ仕組みで足した: 印の `[[table]]` の設定(`gantt = { start, end, days }`)→ 画面の型のメニュー(REL-8)→ 画面のモジュール(`src/ui/gantt.rs`)。核(`src/gantt.rs`)は画面に依存せず、列の名前からの推定(GA-1)、単位(日・週・月)とマスの往復、棒の範囲と親の棒(子孫の範囲)を持つ。印の読みは `workspace::marker_setting` に共通化し、`marker_tree` と `marker_gantt` が同じ形で呼ぶ(GA-7。`WsTable`・`Scope` に欄は足さない)。行は表の `rows`(ビューを通り、親子で並べ替えた行)のうち畳まれていないもので、深さ・番号・進み具合は表と同じ `tree`・`wbs_num`・`wbs_pct` を読む(ガントで独自の並びを持たない。NV-27・NV-28)。読むだけの画面で、日付は表で直す(GA-5。棒のドラッグは次の段)。キーは `C`(`G` は表の最後の行)。画面の型の登録の仕組み(trait)は作らなかった: 木・ガントで十数か所の分岐を手でなぞる形が2つそろった段階で、カンバン・カレンダーを足すときに切り出す。

推定の列の名前(start・begin・from・開始、end・due・finish・until・to・期限・終了)はコードの定数でなく locales の表(`GanttStartNames`・`GanttEndNames`。空白区切り)に置き、`Lang::ALL` の全部の言語から集める。訳ではなく、どの言語の画面でも同じ名前を推定に使う(コードに日本語のリテラルを置かない SR-23 を守り、言語を足せば名前も足せる)。

時間軸は「マス」(単位の1つ。日・週・月)と「字」の2層で持つ。日と週は1マスが1字、月は1マスが3字で、字は旬(1〜10日・11〜20日・21日〜末日)を表す(`gantt::col_chars`・`col_of`・`col_days`)。最初の版は月も1マス1字で、見出しに月の一の位しか書けず(「123456789012」)、年の「YYYY」も4マスで次の年とくっつき(「20262027」)、棒がどれも1字の塊になった(2026-11-01 の gantt-axis で直した)。マスの関数(`cell_of`・`cell_start`・`floor_unit`・`fit_unit`)はそのままにし、`h` `l`・単位の決め方・`t` で戻す数はマスで数える。見出しは2行。日は上の行に月の始まるマス(と左端)の「YYYY-MM」、下の行に週の始まりの曜日(設定の `dates.week_start`)の日の数だけ(毎日の一の位を並べると数字の列になって読めない。週の単位のマスの始まりと同じ日に置き、単位を変えても目印の日がずれないようにする)。週は上に年の始まる週(と左端)の「YYYY」、下に1日を含む週の月の数「MM」(ほかは `·`)。月は上に1月のマス(と左端)の「YYYY」、下にマスの先頭から月の数を右寄せの2字。上の行の字は次の切れ目の1字手前までに収まる形(「YYYY-MM」→「MM」→ 出さない)を選ぶので、1つの字がほかの切れ目を潰さない(最初の版は字の後の6マスを飛ばしていて、原点が月末だと次の月の字が出なかった。1字の空きを残さなかった間は、ちょうど収まる字が次の字とくっつき、原点から次の月まで7字で「2026-112026-12」、次の年まで4週で「20262027」になった)。今日の字は、下の行が空なら `▾`、数に重なれば数を残してその字を反転で示す(数の片方が消えると 10・11・12 の区別がつかない)。区切りは縦線の字でなく地の帯で見せる: 日・週の単位は月ごと(週は週の最後の日の月。月の数「MM」を1日を含む週に書くのと揃える)、月の単位は年ごとに地の色を交互に変え、日の単位の土日は帯の上にも重ねて少し強める(`look::axis_band`)。1字が1日なので線の字はその日のマスを使ってしまい、棒の通る所で切れて点線に見え、今日の印 `┊` とも見分けにくいため。帯は見出しの2行と選んでいない行に付け、選んでいる行は行の全体を選びの見た目にする。帯はテーマ(色の表)のあるときだけ敷き、テーマが `default` と色なしでは付けない(SR-40 の「default は端末の地を変えない」を守る。端末の地は既定では問い合わせないので、暗い地とみなして塗ると明るい端末で読めない。人の決定 gantt-bands-default-theme)。色はテーマの地を明暗の逆へ RGB の各成分で寄せる。段は土日 → 帯 → 帯の上の土日の順に決め(帯が主で、土日は帯より地に近い)、寄せ幅は土日 9・帯 18・帯の上の土日 27 から(前の段より 9 は先から)始めて、地・選びの地・決めた段のどれとも成分で 8 以上離れる所まで寄せる(月の境が週末に当たると帯と土日が隣り合うので、その差も要る。選びの地は今日の数の反転と選んでいる行の地で、帯と近いと sumi・slate で帯の月の今日の印が見えなかった)。最初は地と選びの地の間で作ったが、2つが近いテーマ(sumi・slate など)で差が 3〜8 しか出ず見えなかった。256 色では寄せた色が地やほかの段と同じ番号に落ちる(sumi では4つの段が全部 234)うえ、番号が違っても色がほぼ同じ組(立方体の 188 = 215 と灰の 253 = 218 で差 3。dozy-pink・solarized-light)があるので、番号を色に戻し(`theme::indexed_rgb`)、地・選びの地・ほかの段のどれとも成分の差が 8 以上で、一行おきの色の 237(`theme::paint` が塗り替える)でない所まで寄せる。今日の印の字(数の反転)には帯の地を付けない(地を持つ反転は `theme::paint` が塗らず、classic の見た目で素の反転のまま残る)。分かっている限り: 256 色で `[look.colors]` の地と選びの地をほぼ原色(青・赤など)に上書きすると、寄せ幅の上限(160)に届いて2つの段が同じ番号になることがある(11 のテーマと色の名前の上書きでは起きない)。時間軸の行は `fit_spans` で窓の中の幅に切る(曖昧な字を2桁で数える設定 CV-6 では `█` `┊` `◆` `·` が2桁になり、`relmap::panel` は広すぎる行を切らないので、右の枠を押し出していた。字の位置が見出しとずれるのは前から)。月の単位の旬の範囲は、日付の端 9999-12 より右のマスでは空にする(`cell_start` が端で止まり、旬の始まりを終わりで抑えると (12/30, 12/30) が何度もできて ◆ が並んだ)。メッセージ行の案内のうち表の選んでいる行とセルのもの(見出しの行・読むだけの理由など)は、ガント・木・関係マップの画面と、そこから `?` で開いたヘルプでは出さない(ビューの未対応の理由は出す。開けないビューの案内は表の欄と `[` `]` を指すので出さない。`bands::message` の `on_other_screen`。表の画面のカーソルが見出しの行にあると、ガントの下に「見出しの行」の案内が残っていた)。棒の塗り(`█`)は棒の全長(字の数)に対する割合で、棒の始まりの字から数える(左にはみ出していても同じ)。クリックの行の数は、描画 `ui::draw` が高さを1つ減らして `render` を呼ぶのと同じく `popup::screen` から出す(木と同じ。`size.1` から出すと流れた状態で1つずれる)。`days` から出した日は `types::clamp_date` で日付の範囲に収め(年が負になると `civil` が読めない)、時間軸を動かすときの原点は `floor_unit` で単位の境に保つ(9999-12 の端で `add_months` が月末を返す)。パレットから選んだガントに無い動作は、ガントで選んでいる行を表で選んでから効かせる(木の画面は表の行のまま。ガントは j/k で行を動かすので、強調している行と違う行に書かないため)。

下の帯の「今押せるキー」(gantt.band-edit-keys)は `gantt::hints` が作る。表のキーの順位(`keymap::ranked`)から棒を動かす4つの動作を抜き、動かせる棒を選んでいるときだけ「`H/L` 棒を動かす」「`>/<` 伸ばす・縮める」の2つにまとめて Enter の次に入れる。4つの説明は1つずつで長く(長いものは 40 字ほど)、順位も後ろで、幅 160 の端末でも帯に出なかった。動かせるかは `gantt_edit` が断る棒(読むだけの起動・自分の日付の無い行・`editable` で直せない列)と同じ見方で決める。ヘルプの並び(順位)は変えない。

## 親子の印(navigation.parent-child-config・cell-edit.children-done*・gantt.children-span-*。2026-11-24)

印の `[[table]]` の `children = { key, status, done, closed, on_done, gantt, closed_by, on_reopen }` は、ほかの画面の型の設定と同じく `workspace::marker_children`(`marker_setting` の共通の読み方。`WsTable`・`Scope` に欄は足さない)で読み、型は核の `tree::ChildrenSpec`。画面の側は `src/ui/children.rs`。

- **並べるキー**: `App::tree_key` が、ビューの設定の `tree`(利用者が選んだキー)、無ければ `tree_off` でない限り印の `key` を返し、`arrange_tree`・`Z`・ガントがそれを読む。`Settings.tree` は印に従う間は None のままにし、オフにしたことだけを `Settings.tree_off`(見た目の状態と views.toml に残る)に持つ。ビューの設定の写しは今の並びを `tree` に入れて見せ(区画は「オン」、キーも選び直せる)、反映のときに `settle_tree` が「印と同じキー → None」「オフ → tree_off」に戻す。選ばなかった案: 開くときに印のキーを `Settings.tree` に書き込む(見た目の状態に残るので、印のキーを直しても古いキーで並び続け、オフにしたビューと区別できない)。
- **親の完了**: 表の直し(1行・一括)・カンバンの札・vim の `.` が `App::stage_children` を通る。状態の列を完了の値に直す親があれば、子孫を表の全部の行(`src.rows()`)の `key` のリンクでたどり(ビューの絞り込みやまとまりで見えない子も数える)、同じ1手の値(子を一緒に done にするなど)は直したあとの値で終わったかを見る。`complete` は子孫を `set_each_then` に渡す行に足すので、列の `then` の値も取り消しも親と同じ1手。`refuse` は断った親を外し、断った親が別の親の子孫なら、その親を数え直す。完了の値は `closed` に書かなくても終わったとみなす(書き忘れで `refuse` の親がいつまでも完了にできなくならないように)。
- **ガントのはみ出し**: `Gantt.over` に、行ごとの「子としてはみ出す日数」と「親として子孫がはみ出す日数」(どちらも一番多いもの)を持ち、名前の欄の右端に ` !` を付ける(印の無い行の形は変えない)。比べる親は表の全部の行の親子でたどった祖先のうち、自分の日付を両方持つものだけ(子孫の範囲の棒の親とは比べない)。日数は `gantt::overrun`(始まりより前の日数と終わりより後の日数の和。親を広げるのに要る日数)。`extend` は直したあとの子の範囲を、自分の日付を持つ祖先の全部に広げる書き込みを `gantt_edit` の `set_cells` の同じ1手に入れ、書く前に全部のセルを確かめる(片方だけ書かない)。祖先の前からのずれは直さない。
- **閉じた印と親を戻すとき(cell-edit.children-closed-by・cell-edit.children-reopen。2026-11-25)**: `closed_by`・`on_reopen` は `ChildrenSpec` に項目を足さず、`tree::ReopenSpec` に分けて `workspace::marker_children_reopen` が同じ `children` の値から読む(項目を並べて `ChildrenSpec` を組み立てる前からの試験が、main の試験を今のコードで回す検査で組み立てられなくなるため。`read_children` は2つを知っている項目として通すだけ)。閉じた印のセルは `Changes::set_each_then_with` の `also` で状態の列と同じ1手に入れ、状態の列を直せなかった行には書かない。書く値は子のノートから親を指す wiki の形(名前が1つでなければパス。REL-3 の `link_value`)。戻す子孫は、終わったとみなす値(`is_closed`)から終わっていない値に直す親の子孫のうち、`closed_by` を親を指す列と同じ解き方(`tree_parent`。リストなら先頭)で解いた行き先がその親のもの。戻す値は親と同じ値で、印は空(`NewValue::Null`)。`ask` は `stage_children` が何もためずに直し(列と行と値)を `App::children_ask` に覚えて `asked` を返し、呼んだ側(セルの確定は入力を閉じたあと、カンバンの札と vim の `.` はその場)が `open_children_ask` でパレットの続きの入力 `Ask::ChildrenReopen` を開く。選んだら覚えた直しを答えつきで `stage_children_with` に通すので、「子も戻す」も親と同じ1手。Esc(`close_palette`)は覚えた直しを捨てる。戻す子孫が無ければ聞かない。選ばなかった案: 親だけを先にためて、「子も戻す」を選んだら子を足す(取り消しが2手に分かれ、「親だけ」との違いが取り消しで見えなくなる)。
- **親の候補(relations.link-input-no-cycle。2026-11-25)**: `key` の列のセルの候補(1つの値の `link_list`・リストの列の `link_cands`)から、直す行(一括なら選んだ行の全部)と、その子孫(`App::not_parents`。表の全部の行の親子でたどる)を外す。選ぶと親子が輪になるため。リストの列で今の要素が外す行を指していれば、外せるように候補に残す。印の `children` の無い表と、ほかのリンクの列は今のまま。

## カンバンの画面(KB-1〜KB-6。kanban)

木・ガントと同じ仕組みで足した: 印の `[[table]]` の設定(`kanban = { column }`)→ 画面の型のメニュー(REL-8)→ 画面のモジュール(`src/ui/kanban.rs`)。核(`src/kanban.rs`)は画面に依存せず、列の名前からの推定(KB-1。名前は locales の `KanbanColumnNames`。リストの列は選ばない)とレーンの順(`lane_keys`: 列の決まり CE-36 の `values` の順 → 決まりに無い値を表の行に現れた順 → 値の無い行の「(なし)」)を持つ。決まりの値は行が無くても空のレーンにする(札を動かす先が見えるように。状態の名前の順は進みの順にならない)。行は表の `rows`(ビューを通った行)の全部で、畳みは見ない。行の値は `plain`(ためた値を重ねた文字)の前後の空白を除いたもの。札の名前は `view::note_name`(表のノートの欄と同じ名前の見せ方から、字下げ・印・WBS の番号を除いたもの。`label` から切り出した)。

札を動かす(`H` `L`)のは、表のセルの直しと同じ `Changes::set`(ためる変更)。保存の確かめ(差分)と外の変化の確かめ(WB)を通すため、すぐには書かない。書く値は列の型に合わせる(数・真偽・日付の列はその型。CE-36 の候補と同じ)。読むだけ(`--readonly`)と読むだけのセル(`input::editable` の理由)は動かさずに理由を出す。「(なし)」のレーンへは動かさない(動かすと消すを同じキーにすると、押し間違いで値が消える)。動かしたあとは `regrid` で表を組み直し、続けて `rebuild_kanban` が選んでいた札の行をレーンを問わず探し直すので、札は動かした先のレーンで選ばれたままになる(外の変化 KB-5 も同じ道)。レーンごとに選んでいる札を覚え、レーンを行き来しても位置を失わない。`u` `U` はカンバンのまま取り消し・やり直しをし、`Ctrl+s` は表に戻って表の保存の確かめを開く(確かめの画面から戻る先を表にそろえる)。

描き方: 枠の題「カンバン: <列の見出し>」(displayName。推定なら「(推定)」)。1行目はレーンの見出し「値 (数)」、2行目は区切り、3行目から札。札の頭の2字は `>`(選んでいる札)と `*`(ためた変更のある行)で、色に頼らない(SR-15)。レーンの幅は中の幅を等分し、14 字より狭くなるなら 14 字で入る数だけ並べ、選んでいるレーンが見える所まで横にずらす。中の幅の両端の1字は隠れたレーンの印(`‹` `›`)に取っておく(見出しの字と重ならないように)。

札の値と高さ(kanban.card-*。2026-11-25): 印の `kanban` の `card`(列の名前の並び。`note.` は読むときに除く)は `KanbanSpec.card` に持ち、推定では空。札1枚は `CardBody`(名前の行と値の行)で、名前は頭の行の幅(レーンの幅から印の2字)と続きの行の幅(さらに2字下げ)で2行まで折り返す(`wrap_name`。空白のある名前は、頭の行の後ろの半分にある空白で切る)。値は `cell::shown`(表のセルと同じ見せ方)を通し、値の無い列(`plain` が空でためた値も無い)は行にしない。札では直さないので読むだけの印 `#` は外す。ビューに出していない列の型は `App::kind_of` が `column_kind` で引き、組み直しのときに `card` の列も `col_kinds` に入れておく(日付の形と型の合わない印がセルと同じになる。毎回の描画で行を数え直さない)。札の高さが違うので、レーンの縦の流しは前の「選んでいる札の番号 − 見せる行の数」でなく、`LaneCards::first` が選んでいる札から上へ高さ(と札の間の空き)を足して、入る所までを見せ始めにする(選んでいる札を下の端に寄せる形は前と同じで、1行の札では前と同じ位置)。描画とクリックはどちらも `LaneCards::slots`(行ごとに、どの札の何行目か・札の間の空き・札の無い所)を使い、数え方がずれない。札の組み立ては、見せる札と選んでいる札から上の分だけをする(レーンの全部の札の値を毎回は引かない)。`card` を書いた表は札の名前を太字にし、札の間を1行空ける。`card` を書かない表は空けず、前の札の並びのまま(名前の折り返しだけが効く。`tbd~kanban.card-name-wrap-always`)。札に出す列を画面で選ぶ窓は作らなかった(`tbd~kanban.card-picker`): 札の列は表ごとの印の設定で、画面から印の `[[table]]` の中の値を書き換える口が無く(ページの保存は区画を丸ごと足すか置き換える)、グラフの画面の切り替えのように印を書き換えない形にすると開き直すと消えるため。

画面の型の分岐は `src/ui/screens.rs` に集めた(ガントのときに「同じ形が3つそろってから」と見送った分。木・ガント・カンバンで3つになった)。`Screen`(型の一覧と、モード・名前と記号・切り替えの動作)、`App::screen`(状態を持つ型)、`App::leave_screens`(ほかの型の状態を閉じる)、`App::on_screen`(メニューとパレットをその型から開いている間も含めて、その型を見ているか)。画面の型のメニュー・ヘッダーのボタン・メッセージ行の案内・パレットとメニューを閉じたときの戻り先・型を開くときにほかを閉じる所が、これを読む。型を足すときは `Screen` と各 `match` に1行ずつ足せばよく、閉じ忘れて2つの型の状態が残ることを防ぐ。画面の型の trait(描画・動作・クリックを型ごとの実装に分ける)は作らなかった: 画面ごとの状態の型と描画の引数がまだ揃っておらず、分岐の判定と閉じる所を集めれば抜けは防げる。

## グラフの画面(CH-1〜CH-9。chart)

木・ガント・カンバンと同じ仕組みで足した: 印の `[[table]]` の設定(`chart = { … }` か、その配列)→ 画面の型のメニュー(REL-8)→ 画面のモジュール(`src/ui/chart.rs`)。分岐は `screens.rs` に1行ずつ足すだけで済んだ。核(`src/chart.rs`)は画面に依存せず、種類(`ChartKind`)と集め方(`Agg`)の名前、印の設定の形(`ChartSpec`。鍵はどれも省ける)、分ける列の推定(`infer_x`)、数え(`Acc`: 行の数と、数のある行の合計・最小・最大)、数の列の区間(`bins`)、数と日付の区切りの書き方を持つ。日付の区切りはガントの `gantt::Unit` と `cell_of` を使い、区切りの始まりは `chart::floor_bucket` で決める(週は設定の `dates.week_start` の曜日。カレンダーと日付の言葉の週にそろえる)。ガントの週のマスと今週も同じ曜日から始める(gantt.week-start。どちらも `gantt::floor_unit_in` で切り下げる)。週の始まりを取らない `floor_unit`・`fit_unit` は月曜から数える形のまま残し、画面からは使わない(前からの試験が呼ぶ純関数で、形を変えると main の試験を今のコードで回す検査がコンパイルできずに全部落ちるため)。

印の読みは `workspace::marker_setting` を使い、`read_charts` が値を表か表の配列として読む(配列の k 番目の警告の場所は `table[n].chart[k]`)。ほかの型と違い、書き方の違うグラフは丸ごと外す(グラフは鍵が多く、知らない鍵を無視して残すと、書いた人の意図と違う種類や列で開くため)。列が表にあるかは、ほかの型と同じく列が出そろってから確かめる(行の無い表では決めない)。

種類は1つの画面の型の中で切り替える(`Tab`)。種類ごとに型を分けると、メニューとキーが種類の数だけ増え、同じ行の数え方を型ごとに持つことになる。画面の状態は、印の鍵と推定で全部埋めた `Resolved`(種類・分ける列・値の列・集め方・内訳・単位・累積)と、それで数えた `Data`。キー(`x` `y` `a` `b` `+` `-`)は `Resolved` を変えて数え直すだけで、印は書き換えない(設定のファイルを画面から書く道を増やさない。書き出しは次の段)。種類を変えるときは、今の列を保ち、その種類に合わない列だけ合う最初の列に替える(折れ線と散布図の横は日付か数、散布図の縦は横と別の数か日付、ほかの値は数、内訳は文字・チェックボックス・リスト)。替えられなければその種類を飛ばして理由を出す。

数え方は頻度表(NV-9)にそろえた: 値は `setting_value`(ためた値を重ねる)と `settings::value_keys`(リストは要素ごと、空は値の無い行)、項目の名前は頻度表の `labels`(リンクは行き先の名前)で、列の決まりに `icons`・`labels` のある値だけ、まとまりの見出しと同じ `Resolved::value_shown` の「字 名前」に置き換える(chart.value-labels。絞り込みの鍵は値のまま)。だから棒の `Enter` は、頻度表の `Enter` と同じ同じ値の絞り込み(`freq::push_same`。リストの列は「含む」)で、件数と絞ったあとの行の数が合う。日付と数の区切りは同じ値の絞り込みでは表せないので絞らない。項目の順は、列の決まり(CE-36)があればその順(行の無い値も 0 の棒にする。カンバンのレーンと同じ)、ほかは値の大きい順で、値の無い行は最後。

描き方: 線と点は自前で行(`Line`)に組む。ratatui の `Chart` の部品は使わない(画面はどれも行を組んで描き、East Asian Ambiguous を幅2で数える描画 CV-6 もその形を前提にするため。部品をバッファに描いてから行に写すと、目盛りの日本語の幅と色なしの点の字を部品の外で直すことになる)。折れ線は点字(U+2800〜。幅は1)の格子にブレゼンハムで線を引き、区切りの点に内訳ごとの点の字(`*` `+` `o` `x` `#`)を重ねる。色が無くても点の字で線を見分けられる(SR-15)。積み上げ棒は内訳ごとの塗りの字(`█` `▓` `▒` `░` `▞`、あふれた分 `:`)で、境は累積で丸めて字の数のずれをためない。内訳の上限からあふれた分の名前は「ほか」(英語は rest)にする。「その他」「other」は値の名前によく使われ(お金の分類など)、凡例で値の名前と見分けられなくなるため。ヒートマップは値 ÷ 最大を 4 段の濃さの字にし、色があれば地の色も付ける。枠の文字が ASCII のとき(`borders = "ascii"` と CV-6)は、塗りと濃さも ASCII の字にする。

色(chart.series-contrast・chart.heatmap-readable): 画面の地(テーマの地、無ければ端末に問い合わせた地、分からなければ黒)が明るいか暗いかで、内訳の色の組を分ける。暗い地には今までの明るい組、明るい地には濃い組で、どちらも全部のテーマの地と WCAG の対比 3 以上(試験がテーマを全部回して確かめる。テーマを足しても同じ試験が回る)。積み上げ棒の地を埋めない塗りの字(`▓` `░` など)は、字の点が粗く棒の形が地に溶けるので、地を内訳の色へ 22% 寄せる。ヒートマップの地は、画面の地からテーマのアクセント(列の見出しの色)へ 0.3・0.55・0.8・1.0 寄せた4段で、明るい地では濃く、暗い地では明るくなる。256 色では近い番号に丸めるので、前の段と同じ番号になったら番号が変わるまで寄せ、アクセントでも変わらなければアクセントから文字の色へ寄せる(dozy-pink の4段目)。数の字は、テーマの文字の色か地の色のうち、地との対比が 4.5 以上のものを先に使い、どちらも届かなければ黒か白へ 1/4 ずつ寄せる(テーマの色味を残すため、いきなり黒と白にしない)。濃さの字(`░` 〜 `█`)は色なしの表示のためのもので、色のあるときはます目の地と同じ色で描き、左端の塊に見せない。前は暗い地の端末向けの紺の段を決め打ちし、数の字はテーマの文字の色のままだったので、明るい地のテーマで数が読めず、段も逆に見えた。

件数と合計の折れ線は、行の無い区切りを 0 にする(平均・最小・最大は値なしで線を切る)。日付の単位は、区切りの並ぶ向きの長さ(棒・ヒートマップは縦の行、折れ線は横の字)に収まる最も細かい単位で開き、折れ線は区切りが字より多くならないところまで粗くする。

キーは `P`(plot。`G` は表の最後の行)。

## ノートの削除とごみ箱(cell-edit.delete-note-*・cell-edit.new-note-undo・cell-edit.trash-restore-*。2026-11-18)

- **置き場**: `<根>/.trash/mdgrid/<消した時刻(UNIX 秒)>-<通し番号>/<根からの相対パス>`(`src/trash.rs`)。`.trash/` は Obsidian のごみ箱と同じ名前で、mdgrid は中を探さない(base-view の隠しフォルダの決まり)。置き場の決まりは cell-edit.delete-note-trash-place。
- **台帳を持たない**: 元の場所と消した時刻は、ごみ箱の中のパスから読む。台帳のファイルを別に持つと、ファイルと台帳が食い違ったときにどちらを信じるかの決まりが要るので持たない。
- **置き換えない**: ごみ箱へは、1回の削除ごとに `create_dir`(既にあれば失敗)で作ったフォルダへ `rename` する。戻すときは `hard_link`(行き先があれば失敗)で元の場所に名前を作ってから、ごみ箱の側の名前を消す(`rename` は行き先を黙って置き換えるので使わない)。ハードリンクを作れない置き場では、行き先が無いことを見てから `rename` する。
- **読み込み口**: `Source::trash`・`restore`・`trashed` を足した(既定は「できない」。CSV は既定のまま)。Markdown は、動かしたあと `vault.forget`・`vault.reload` と `rebuild` で行を合わせる(ノートの名前の変更と同じ流れ)。
- **取り消しとやり直しの順**: ノートの作成・削除の手は画面の側(`src/ui/note_trash.rs`)に積み、ためる変更の手(`Changes`)とは別に持つ。積むときに `Changes::undo_depth`(やり直しの側は `redo_depth`)を覚え、それより新しいためる変更の手が残っている間は、そちらを先に戻す。保存でためる変更の手が減っても、ノートにためた変更が残っていればそちらを先に戻す。やり直しの積みは、ためる変更の新しい手で消えるのと同じに消す(`Changes::redo_epoch` で古くなったかを見る)。
- **一覧から戻す手は積まない**: 「ごみ箱から戻す」は取り消しの手にしない(戻したノートを消したければ「ノートを消す」)。戻したノートを指していた手は積みから外す。
- **リンク**: 消すノートを指すリンクは書き換えず、確かめの問いに、指しているノートの数を添える(cell-edit.delete-note-links)。
- **Obsidian が `.trash/` の直下に置いたノート**は、元の場所が分からないので一覧に出さない。

## vim モード(screen.vim-*・screen.quit-discard。2026-11-21)

- **入り切り**: 設定の `[keys]` の `vim`(tbd~screen.vim-setting-name の仮の置き場)。`[keys]` の下はモードの表(`[keys.<モード>]`)を並べる場所なので、`vim` だけは真偽の値として読む(`config::read_keys`)。偽(既定)ではキーの表に何も足さず、振る舞いは今と同じ。
- **足すキーも1つの表**: vim モードのキーは `keymap::VIM_BINDINGS` に置き、`App::configure` が `vim = true` のときだけ既定の表の写しに足してから割り当て直しを当てる。だから `[keys.table]`・`[keys.input_normal]` で割り当て直せて、ヘルプとキーの文書にも同じ表から出る(screen.vim-remap)。割り当て直しで動作の名前とその動作が使えるモードを引くときは、`keymap::defaults()`(`BINDINGS` と `VIM_BINDINGS`)を見る。
- **ノーマルのモードはキーの表のモード**: 入力の欄のノーマルのモードは `Mode::InputNormal` として表に持つが、`App::mode` にはしない(どの欄を開いているかは `App::mode` が持ち続ける。描画・確定・取りやめは欄のモードのまま)。ノーマルのモードの間は、キーを先に `input_normal` の表で引き(前置きの `d`・`c` もここ)、表に無い文字は入れず、Enter・Esc・矢印・Ctrl のキーは欄のモードの表で今までどおりに当てる(`src/ui/vim.rs`)。挿入のモードで欄の「抜ける動作」(`keymap::exits`。既定は Esc)のキーを押すとノーマルのモードへ移る。
- **カーソルを持たない欄**: 探す・検索・パレット・設定の値・絞り込みの窓の値は、今まで末尾にだけ打つ欄だった。vim の状態(`Vim::cursor`)にカーソルを持ち、ノーマルのモードで動かしたときと、途中に挿入するときだけ使う(末尾にあるあいだは今までの道で打つ)。欄の構造体には手を入れない。
- **語の切れ目**: vim の `utf_class` と同じ分け方(空白・記号・英数字、ひらがな・カタカナ・漢字などの文字の種類ごと)。`会議の見積もり` の `dw` が `会議` だけを消すのはこのため。
- **欄の中の取り消し**: 欄を開いてから閉じるまでの、欄の中だけの積み。挿入のモードの間の打ち込みと、替える操作(`cw` など)とそのあとの打ち込みは1手にまとめる。表のためる変更の取り消しとは別。
- **`dd` は「ノートを消す」そのもの**: `d d` は動作 `delete_note` の割り当てで、範囲を選んでいる間は `d d` の前置きのキーだけで同じ動作にする(`d d` を外せば範囲の `d` も効かない)。vim モードでは「ノートを消す」は印を付けた行(無ければ選んでいる行)を対象にし、パレットから選んでも同じ(キーで消え方を変えない)。何件でも、取り消しは1手(`NoteOp` は複数のノートを持つ)。vim モードでないときは今までどおり選んでいる行だけ。
- **写した値**: `y`・`Y` を押したときに、OS のクリップボードと別に、mdgrid の中で写した値(セルの値か、行のノートのキーの値)を覚える。`p` はそれを入れる列の型で読み直して(日付の列は日付の値にして、囲まずに書く形を保つ)ためる。行の値は `Changes::set_cells` で1手にまとめ、列の `then` は入れない(写した行の値をそのまま入れるため)。セルの値は、打って入れたのと同じく `then` も入れる。
- **`.` で繰り返す値**: 1つのセルにためた値(入力の確定・チェックボックス・空にする・リストの選択・`p`)を列と一緒に覚え、選んでいる行の同じ列に入れる。まとめて入れた値は覚え直さない。
- **数字は回数にしない**: 数字だけのキーは名前の条件の切り替え(condition.digit-keys)に使うので、vim モードは数字のキーを足さない。
- **カーソルの形**: `App::cursor_shape` を画面を回す側が見て、変わったときだけ端末に送る。一度でも送ったら、終えるときに端末の既定の形に戻す(vim モードでなければ何も送らない)。
- **`q!`**: パレットの `q!` は、ためた変更を捨てて確かめずに終わる。vim モードによらない(`q` は今までどおり確かめる)。

## 開き直し(resume.*。2026-11-20)

- **命令の中身は1つ**: Herdr に伝える命令と、`--continue` のための最後の画面は、同じ `Reopen`(`src/resume.rs`)から作る。中身は開いている表の絶対パス(フォルダ1つなら `--raw`)・今のビュー・起動で渡した `-w`・`--readonly`・`--no-color`・`--config`。画面の状態(App)には足さず、画面を回す側(`src/reopen.rs` の `Resumer`)が、イベントをさばくたびに App から今のビューを読んで、前と違うときだけ伝えて書く。表を移ったときは `run_switching` が `Resumer::switch` で表を差し替える。
- **フォルダには `--raw`**: 立て直しのときに、フォルダの印の `open` が別の表に置き換えないため。`.md` を渡した起動も、その時点でフォルダに置き換えているので同じになる。mdgrid のビューのあるフォルダで既定の表を見ているときは、既定の表の名前を `--view` に添える(既定のビュー NV-25 に置き換わらないため)。
- **Herdr の呼び方**: Herdr の道具は裏の糸で1つずつ走らせ、たまった呼び出しは最後の1つだけを走らせる(古い命令は新しい命令で置き換わる)。1回は 10 秒まで待って止める。2 秒にしたら、作ったばかりの実行ファイルの初めての起動(macOS の確かめ)の途中で止めることがあった。終えるときは窓を返す呼び出しを 2.5 秒まで待ち、返らなければ待たずに終わる(Herdr は、窓がシェルの入力待ちに戻ったのを見て命令を消すので、返せなくても残らない)。失敗は黙って捨てる(Herdr の手引きのとおり)。`--seq` は時刻のマイクロ秒を下限に、続けて伝えたら1ずつ足す。
- **窓の題(resume.herdr-title。2026-11-21)**: 題は命令と一緒に決める。`Resumer::tick` が命令と題の文字を `Reporter::update` に渡し、裏の糸は「伝えたいこと」(`Want`: 命令と題、または窓を返す)の最後の1つだけを、`Sent::plan` で呼び出しの並びにして走らせる。たまった古い呼び出しを捨てても題と命令がずれないよう、並びは毎回、命令と題の両方を伝え直す形にする。`Sent` は裏の糸が実際に伝えたこと(窓を持っているか・題を出しているか)を持ち、それで並びを決める。
  - 並び: 窓を初めて持つときは `report-agent` → `report-metadata --title`、持っている間は `--title` → `report-agent`、終えるときと命令を受け付けられないときは `--clear-title` → `release-agent`。題には `--agent mdgrid` を付けるので(Herdr の手引きでは、`--agent` は題を守る印)、題は窓を mdgrid が持っている間に伝える。持っている間に題を先にするのは、移った命令が届いた時点で題も替わっているようにするため(試験が命令の届いたのを見て呼び出しを数えるとき、あとから題が来て数がずれない)。
  - 題を消すのは、題を伝えたあとだけ。窓を返しても Herdr は題を消さず、窓が閉じるまで残すので、返す前に消す(resume.herdr-title-clear)。
  - 題の文字は、命令と同じ決まり(`herdr_accepts`。アポストロフィ・制御文字が無いこと)で確かめ、受け付けられなければ題を伝えず、前の題があれば消す(古い表の名前を残さない)。命令は題に関わらず伝える。命令を受け付けられないときは題も伝えない(窓を返すので)。
  - 題の形(resume.herdr-title-form): 表の名前は `resume::table_name`。開いた表と同じ実体のパスの範囲(App の `scope`)の表があればその `name`(同じパスがいくつもあれば、今のビューを `view` に書いた表)、無ければ `.base` のファイル名(拡張子を除く)かフォルダの名前(`workspace::stem_of`。フォルダを複数開いていれば `, ` でつなぐ)。ビューは開き直す命令の `--view` と同じもの(既定の表なら既定の表の名前)を ` · ` で添え、`view_names` が1つ以下の表(ビューのタブを出さない表)では添えない。タブの見せ方の設定(`display.tabs`)には依らない。ページ(stack.page-open)の名前はまだ無い(縦に並べる画面ができたら、ページを開いているときはページの名前にする)。
  - `tick` はイベントをさばくたびに走るので、題を決めるのは命令かビューの数が変わったときだけにする(範囲の表と比べるのに実体のパスを引くため)。
- **最後の画面の鍵**: 起動したときの今のフォルダの実体のパス。見た目の状態と同じ置き場の `last/<鍵の SHA-256>.toml` に置き、中に鍵を持って別のフォルダのものを読まない。`--continue` は `main` の頭で引数を読み替える(パスと旗を足して `--continue` を外す)。ビューだけは開いてから `select_view_named` で選び、無ければ先頭のビューと理由を出す(立て直しをエラーで止めない)。
- **画面の `--view`**: `.base` のビューに無ければ、開いてから mdgrid のビューを探す(cli.view-own-views)。`--print` と同じ探し方。
- **名前のあるセッションとの関係**: セッション(cli.session-*)は名前を付けて設定の置き場に置くもの、最後の画面は名前の無い1つで状態の置き場に置くもの。セッションを作ったとき(2026-11-21)は `Reopen` を広げず、セッションで開いた表も開いている表とビューで命令を作る(resume.* の要求がセッションに触れないため。`-s <名前>` で開き直すかは人の判断を待つ)。

## 最近開いた表(cli.recent-*。2026-11-22)

- **覚える側**: 画面を回す側の `Resumer::tick`(`src/reopen.rs`)が、開き直しの命令と同じ `Reopen` を使い、表かビューが前に覚えたものと違うときだけ `recent::record` で書く。App には何も足さない。フォルダを複数開いた表は1つのパスで開き直せないので覚えない。引数なしの起動の一覧(先頭に「今のフォルダ」)を開いている間は覚えない(一覧で選んだ表を開いたことにして、毎回の今のフォルダで並びを埋めない)。
- **置き場と形**: 見た目の状態の置き場の `recent.toml`(`[[table]]` に `path`・`view`・`at`)。新しい順に並べ、同じ実体のパスは1つにして先頭へ移し、20 件で切る。日時は書くときの地域の時刻の `YYYY-MM-DD HH:MM` を文字のまま持つ(並びの順は配列の順で、日時は見せるだけ)。読めないファイルは無いのと同じ。
- **無くなった表**: 一覧と `--recent` に出す前に `recent::present` が `openable`(印に中身を書いた表も開ける)で確かめ、外したものがあれば書き直す。
- **一覧**: パレットの「最近開いた表」と起動の一覧の「最近」は、今開いている表を除く(選んでも同じ表を開き直すだけなので)。行は「名前 · ビュー」の欄をそろえ、日時、フォルダの順(長いフォルダで日時が切れないように、フォルダを最後に)。選んだ表へは登録した表と同じ `switch_place`(ためた変更は終わる手順で確かめ、`-w` は引き継がない)。

## お気に入りのワークスペースと切り替え(workspace.favorite-*・workspace.switch-*。2026-11-22)

- **お気に入りの置き場**: `workspaces.toml` の最上位の `favorites`(`src/favorites.rs`)。書き換えは区画の見出しより前の `favorites` の行(複数行の配列も)だけを置き換え、無ければ先頭に足す。読み直して並びかワークスペースが狙いと違えば、並びから書き直す。ワークスペースの書き換え(`workspace::checked`)が並びから書き直すときも `favorites` の行を引き継ぐ。項目は名前(`workspaces.toml` のワークスペース)か、`~` で始まるか絶対のパス(印の根)。どちらでもない・無くなった項目は、一覧を開いたときに警告して外す。
- **一覧はモード `switch`**: パレットの続きの入力(`Ask`)にせず、画面の型のメニューと同じ小さな窓のモードにした。数字のキーを打ち込みでなく選ぶキーにするため(`choose_1`〜`choose_9`。割り当て直せる)。どの画面の型の描画も `switch::overlay` を呼び、`on_screen` は一覧を開いている間も開いた元の型を描く。閉じると `screen().mode()` に戻る。キー `W` と、パレットから選んだ `workspace_switch` は `App::apply` の頭で受け、画面の型を閉じずに一覧を開く。関係マップにもパレット(`:`)を足し、関係マップに無い動作は表の画面に戻ってする(ほかの型と同じ)。
- **今いる所の目印(`Spot`)**: `workspaces.toml` のワークスペース(`-w` を含む)は名前、印と検知の範囲は根の実体のパス、範囲の無い表はその表のパス。お気に入りと「最初に開いた所」もこの目印で比べる。
- **見た表の記録(`Visits`)**: App は表ごとに作り直すので、`run_switching` が移る前に `leave_visit` で今の目印・表・ビューを足し、次の App に渡す。最初に開いた所は起動のときに `mark_origin` で覚え、起動の一覧で選んで移ったときだけ移った先で覚え直す。お気に入りへ移る先は、記録のそのワークスペースの最後の表とビュー、無ければ印の `open` か最初の表。`workspaces.toml` のワークスペースへは `-w` をその名前にして、印と範囲の無い所へは `-w` を外して開き直す。
- **ためた変更**: 移るのは登録した表と同じ終わる手順(`begin_quit`)。表でない画面の型から移ろうとして確かめで取りやめたら、その型の画面に戻る(`review_action` の Back は `screen().mode()` へ。前は表へ戻していたが、終わる手順へは表の画面からしか入らなかったので、表では同じ)。
- **移る道は1つ(2026-11-25)**: 切り替えの一覧とパレットの「ワークスペースを開く」(workspace.open-favorites)は、どれも `switch::go_to(Dest, Picker)` で移る。`Dest` は最初に開いた所かお気に入り、`Picker` は閉じる窓。今いる所を選んだら、窓を閉じて「今いるワークスペース」と知らせる(workspace.switch-hint。数字のキーでも Enter でもクリックでも)。パレットの「ワークスペースを開く」は、`workspaces.toml` のワークスペースの名前(続けて表を選ぶ。今まで通り)に、最初に開いた所と印のワークスペースのお気に入りを足した `Ask::WsOpen { items, dests }`。
- **引き出しと切り替えの一覧の分け方(2026-11-25 に人が決め直した)**: 引き出しはワークスペースの中の切り替えで、今の範囲のページと表だけを並べる(workspace.drawer-scope-only)。前に並べていたほかのワークスペースの畳んだ見出しと、最初に開いた所・お気に入りの行は、切り替えの一覧に寄せた。お気に入りの出し入れも切り替えの一覧で行う: 今いる所がお気に入りでなければ末尾に入れる行(`Row::AddHere`)を置き、`a`(動作 `workspace_favorite`)かその行で入れ、`d`(`remove_item`)で選んだお気に入りを外し、`K`・`J`(`move_item_up`・`move_item_down`。並べ替えの窓と同じ流儀)で隣と入れ替える(`favorites::swap`。最上位の `favorites` の行だけを書き換える)。どれも一覧を開いたまま読み直し(`refresh_switcher`)、選びは入れた所・動かしたお気に入りに置く。下の帯の順位は Enter・数字・a・d・Esc・K・J で、幅 100 の日本語の帯では a のあとが省かれる(窓の下の縁には Enter・a・d・Esc を全部出す)。
- **パンくずの2つのボタン(screen.breadcrumb-switch・screen.breadcrumb-drawer。2026-11-25)**: パンくずは「W 範囲 › w 表 ▾」の、範囲の名前のボタン(切り替えの一覧)と表の名前のボタン(引き出し)に分け、間の「 › 」はどちらにも入れない(`bands::crumb_parts`・`crumb_part_at`)。範囲の名前は表でない画面の型の上にそのまま切り替えの一覧を開き(W のキーと同じ)、表の名前は今まで通り表の画面に戻ってから引き出しを開く。引き出しを開いているときに範囲の名前を押したら引き出しを閉じて切り替えの一覧を、切り替えの一覧を開いているときに表の名前を押したら一覧を閉じて引き出しを開く(同じボタンなら閉じるだけ)。
- **下の帯の数字のキー**: 数字は番号ごとに別の動作(`choose_0`〜`choose_9`)で、`keymap::hints` は動作ごとに1つ出すので、まとめて「0〜9 番号の所へ移る」と出すには `switch::hints` が今までのキーの Enter の後ろに足す(`stack::close_hints` と同じく、下の帯の組み立てがモードで替える)。範囲は割り当ての表から引く(外した数字は除く)。

## 試験の実行ファイル(2026-11-20)

- **2つに束ねる**: 結合の試験(`tests/test_*.rs`)は、`tests/all.rs` が `#[path]` で取り込んで1つの実行ファイル `all` にする。環境変数(HOME・XDG_CONFIG_HOME)を書き換える `tests/test_config.rs` だけは、ほかの試験と同じプロセスで並べて走らせないように別の実行ファイルにする。`Cargo.toml` は `autotests = false` で、この2つを `[[test]]` に書く。ファイルの場所と試験の関数の名前は変えないので、trace-gate の印・weaken・JUnit の照らし方(`#[path]` の取り込み先の名前で照らす)はそのまま効く。
- **なぜ**: ファイルごとに実行ファイルを作ると、リンクと、macOS が新しい実行ファイルを初めて起動するときの検査が、ファイルの数(115)だけ掛かっていた。検査は1つで最大 120 秒、2回目は 0.02 秒(実測)。束ねる前は `ci.sh` の試験の部分だけで 27 分を超えていた。束ねたあとは、組み直した直後の1回目が 441 秒(新しい実行ファイル5つの検査)、2回目は `cargo test` で 36 秒、`cargo nextest run` で 18 秒。残る待ちは、端末のアプリを macOS のシステム設定の「デベロッパツール」に入れると消える。
- **足し忘れ**: `autotests = false` なので、新しい試験のファイルは `tests/all.rs` に `#[path]` の2行を足す。足し忘れたファイルは、`tests/all.rs` の `every_test_file_is_built` が落とす。
- **プロセス全体の値**: 束ねた試験は1つのプロセスの中で並んで走る。環境変数・今のフォルダ・言語の設定のようなプロセス全体の値を書き換える試験は、`test_config` と同じく別の `[[test]]` にする(nextest は試験ごとに別のプロセスで走らせるのでぶつからないが、`cargo test` は同じプロセスの糸で走らせる)。

## lint と書式(2026-11-21)

- **道具はリポが持つ**: `flake.nix` の `devShells.default` に、Rust(cargo・rustc・clippy・rustfmt・cargo-nextest・cargo-deny)、Python(python3・ruff)、シェル(shellcheck・shfmt)、GitHub Actions(actionlint)の道具を並べる。版は `flake.lock` が固めるので、どのマシンと CI でも同じになり、mdgrid を開発しないマシンには何も入らない(dotfiles には入れない)。`nix develop` か direnv の `use flake` で入る。
- **Rust**: `Cargo.toml` の `[lints]` で、clippy の既定の組に `pedantic` を足し、`rust_2018_idioms` も入れる。このアプリに合わないもの(画面の座標の数の型の変換、公開しない API の文書、日本語の文書で誤検知の多い `doc_markdown` など)は、`Cargo.toml` に理由を書いて外す。`dbg!`・`todo!`・`unimplemented!` は止め、`unsafe` には `// SAFETY:` を書かせる。関数の引数の形を変える直し(`needless_pass_by_value`・`ref_option`)は呼ぶ側も変わるので、いったん外して、あとの PR の候補にする。rustfmt は既定のまま(Rust の標準の形)。
- **Python**: `ruff.toml`。ruff の既定の規則に import の並び(`I`)を足し、書式も ruff。macOS の python3(3.9)で動く道具があるので、書き換えは 3.9 の文法まで。`tools/trace_gate.py` は人が取り込む写しなので対象外。
- **シェル**: shellcheck と shfmt(字下げ2文字、`case` の中も字下げ)。
- **Actions**: actionlint(`.github/actionlint.yaml`)。forge-gate が作る `trace-gate.yml` は手で直せないので、わざとの書き方の知らせ(SC2016)だけを許す。`run` の中に `${{ }}` を直に埋めず、`env` で渡す。
- **回し方**: `nix develop` の中で回す(clippy は版ごとに新しい検査が入るので、`flake.lock` の rustc と CI の安定版にそろえる。手元の別の版の clippy では、知らない検査の名前の警告が出ることがある)。Rust は `ci.sh` の fmt と clippy、それ以外は `scripts/lint.sh`(`--fix` で直せるものを直す)。CI は `lint` のジョブ(Rust)と `lint-scripts` のジョブ(`nix develop -c sh scripts/lint.sh`)。

## 名前を付けた条件(condition.*・navigation.expr-filter。2026-11-21)

- **置き換えは読むときに**: `cond("名前")` は、式を読むとき(`expr::parse`)に、その条件の式(真かどうか。節 `IsTrue`)に置き換える。評価(`expr::eval`)は条件を知らず、`.base` の式・`--print --filter`・画面の式・ほかの条件の式のどれでも同じに効く。今の条件は起動のはじめに `condition::install` で1度だけ置き(言語の `i18n::set_default` と同じ形)、試験は糸ごとの `condition::scoped` で置き換える。表の `.base` を読む前に置くので、`main` は設定の文から条件だけを先に読む(`condition::peek`。警告は設定を読み直すところで出す)。
- **条件どうし**: 条件の式は、まず参照を残したまま読み(`parse_open`)、輪を探して外し(輪ごとに警告1つ)、参照の先から置き換える。無い名前・外した条件を指す条件も警告して外す。置き換えたあとの深さも、式の深さの上限(MAX_DEPTH)で止める。
- **無い名前は読めない式**: 誤りの種類(`ExprError`)は足さずに `Syntax` にして、理由に無い名前とある名前を入れる。既にある試験が誤りの種類を全部の場合で数えているので、種類を足すと main の試験が組み立てられなくなる(reverse-run)。
- **書いた順**: toml の表は名前の順に並ぶ(`preserve_order` を入れていない)。数字の重なりの「先に書いた条件」は、設定の文の `[conditions.名前]` の見出しの順で決める(`condition::written_order`)。
- **画面の段**: 表の組み立ての重ね(`grid.rs` の `overlay`)の頭で、いつも隠す条件の行を除いて条件ごとに数え(「N/M行」の M に数えない)、数字のキーの条件・式の絞り込み・ビューの設定の名前の条件は、検索や同じ値の絞り込みと同じ `passes` で絞る。ガント・カンバン・グラフ・木は `App.rows` を使うので同じく効く。表を移ると `App` を作り直すので、数字のキーと式の絞り込みは解ける。
- **数字のキー**: 動作 `condition_1`〜`condition_9` を表のモードの `1`〜`9` に置く。割り当て直しはほかのキーと同じ。ヘルプとパレットは、条件の無い数字を出さず、名前は条件の見せる名前にする。
- **`--print` などの段**: `print::table` の結果から、いつも隠す条件と mdgrid のビューの名前の条件の行を除く(`main.rs` の `drop_by_conditions`)。`--count` はその行の数だけを出す。`--pick` は画面と同じ道を通る。

## 台帳の印に書く条件・通知の欄・知らせ(condition.marker・condition.digit-order・condition.columns-present。2026-11-25)

- **重ねる所は1つ**: 開く表から上へたどって最初に見つかった印(`workspace::marker_table`。スターターを重ねた文)の `[conditions]`・`[[notices]]`・`[[notify]]` を、`config::for_table` が全体の設定に重ねた `Config` を返す。画面は `App::start` で、`--print`・`--count`・`--check` は `print_view` で、`--print-config --resolved` は出す前に呼ぶ。`--notices`・`--notify`・`--runs` は画面と同じ道(`open_app` → `start`)を通る。重ねたかどうかと出どころは `Config.ledger` に持つ(`Config` は試験が `..Config::default()` で組むので、項目を足しても main の試験は組み立てられる)。
- **条件は1回で組み直す**: 印の条件を先に、全体の条件のうち同じ名前の無いものを後に並べ、`Conditions::build` と同じ手順で1回で組む(`Conditions::layered`)。全体の条件の式の `cond()` も印の条件を指すのは、このため。数字の重なりは、どのファイル(層)の条件かを添えて、同じファイルの中だけで見る。印から来た名前は `Conditions` の中に持つ(`Condition` は変えない)。
- **式を読む前に置く**: `cond()` は式を読むときに置き換わるので、`.base` を読む `open_target` の前に、開く表の条件を `condition::install` で置く(`open_app` は設定を読む順を `open_target` の前に移した)。ほかの表を数える・確かめる裏の糸(`count_tables`・`notify_hits`)は、表ごとにその表の印を全体の条件に重ね直し(`config::table_conditions`)、糸の中で `condition::scoped` に置いてからその表を開く。どの表の行も、その表で効く条件で数える。全体の条件は `CondState.global` に持って糸に渡す。
- **数字は振る**: `key` は振る順。`Conditions::digit_order` が印の条件・全体の条件の順に `key` の小さい順で並べ、画面の `App::digits` が、今の表で使えない条件と、書いた数字を割り当て直した条件(condition.digit-remap の警告を出す)を飛ばし、割り当て直した数字も飛ばして 1 から振る。キーを押したとき・帯・ヘルプ・パレットは、どれも `digits` から引く。
- **列があるか**: 式が使う列は `Expr::columns`(ノートのキーと `formula.*`。`file.*` は入れない)。今の表の行は `.base` の最上位の絞り込み(`Base::keeps`)を通った行で、ノートのキーは値を持つ行(空の値も含む)が1つでもあれば、`formula.x` は表の式に x があれば、ある。画面はこの答えを列ごとに `CondState.present` に覚え、表の形(`data_gen`・読んだ数・読み終えたか・ためた変更の数)が変わったら数え直す(ヘルプは描くたびに数字を引くので、毎回は数えない)。裏の糸とコマンドは `main.rs` の `table_has` で同じ判定をする。印の表に中身を書いた表は印の根のノートを全部読むので、ほかの表のノートだけにある列を、ある列と数えないためにも行で見る。
- **いつも隠す条件は列を見ない**: far のように、始まりの列だけがある表でも隠したい条件があるため(tbd~condition.columns-hide)。

## 通知の欄(notice.*。2026-11-21)

- **置き場**: 欄は画面のいちばん上(`bands::top_lines` の頭)に置き、`top_rows` に欄の行の数を足す。どの画面の型も `top_lines` と `top_rows` から組むので、欄の分だけそろって下がる。項目が無ければ 0 行で、今の画面と1字も変わらない。
- **数え方**: 今の表だけを数える項目は、その場で今の表の行(`.base` の最上位の絞り込み `Base::keeps` と、いつも隠す条件のあと)を数える。ほかの表を含む項目は、裏の糸が表をファイルから開いて数える(`main.rs` の `count_tables`。`--notices` も同じ関数)。結果は世代つきで送り、古い世代の結果は捨てる。描く前の `notice_poll` で拾うので、拾うまでは「…」。
- **数え直し**: 読み込みが終わったとき・保存で書いたとき・外の変更を読んだときは `dirty` を立て、次の `notice_poll` で数え直す。日付は `poll` のたびに時計から読み、変わったら今日を進めて組み直す。エディタから戻ったときは、その場で読み直す。
- **欄から移る**: 今の表の項目は、数字のキーの条件と同じ `cond.on` に入れる(ほかの表へ移ると解ける)。ほかの表は、当たる行のある表を並べ、選んだ表へ移ってからその条件を入れる(`switch_cond`)。ためた変更があれば、引き出しと同じく終わる手順で確かめる。
- **`--notices`**: 画面と同じ道(`open_app`)で App を作り、読み終えてから、裏の糸の結果を待って出す。範囲(ワークスペース)の決め方が画面と同じになる。
- **起動の直後のキー**: テーマの `auto` の地の色の問い合わせが、起動の直後の1打を吸うことがある(前からの振る舞い)。疑似端末の試験は、移った印が出るまで押し直す。

## 知らせ(notify.*。2026-11-23)

- **核と画面を分ける**: 設定の読み・記録と比べて鳴らす行を決める・まとめる・出す先へ渡すのは `src/notify.rs`(表を知らない)。表をファイルから開いて条件に当たる行を選び、題と文を埋めるのは `main.rs` の `notify_hits`(通知の欄の `count_tables` と同じ道。`--notify` も使う)。開いている間の時と糸は `src/ui/notify.rs`。
- **いつ確かめるか**: 開いたとき(`configure_notify` が `want` を立てる)・保存で書いたとき(`review.rs` が `want` を立てる)・最後に確かめ始めてから `INTERVAL`(1分。tbd~notify.interval)が過ぎたとき。画面を回すたびに `notify_poll` が見る。確かめている途中は次を始めない。表は画面の App の行でなくファイルから読むので、ためた変更は数えず、外のエディタで直したノートは拾う。
- **記録**: 見た目の状態の置き場の `notify.json`(`{"version":1,"seen":{"<条件の名前>\t<表の見分け>":[行の見分け…]}}`)。表の見分けは開くパスの並び、行の見分けはノートのファイルのパス、CSV・TSV は表のパスと行の値の並び(tbd~notify.csv-row-id)。記録に無い表は初めての確かめで、当たっている行を覚えるだけ(notify.first-silent)。新しい記録は、当たり続けている行と鳴らせた行だけ(外れた行は忘れ、鳴らせなかった行は次にまた鳴らす)。読めない表の記録は触らない。読めない記録は空として読み、どの表も初めてになる(notify.state-broken)。書くのは設定と同じく一時ファイルからの名前の変更。
- **錠**: 同じ置き場の `notify.lock` を `File::lock`(標準の、ファイルの錠。MSRV 1.90 で使える)で取り、記録を読んでから鳴らして書くまで持つ。画面と `--notify`、2つの `--notify` が同時に確かめても、同じ行は1回だけ鳴る。錠を待つのは裏の糸なので、画面は止まらない。
- **まとめ**: 1つの項目の1回の確かめで、範囲の表の全部を合わせた新しい行の数が `group_over` を超えたら1件にする。題は条件の見せる名前。どれかの出す先に届けば、まとめた行を全部覚える。
- **端末の合図**: 環境変数で端末を見分け、合図を1つだけ作る(`detect_term`)。kitty は OSC 99、iTerm2 は OSC 9、Ghostty・WezTerm・foot・Windows Terminal は OSC 777、ほかは BEL。tmux の中では TERM と TERM_PROGRAM が tmux のものになるので、外の端末が残す環境変数(`KITTY_WINDOW_ID`・`ITERM_SESSION_ID`・`GHOSTTY_RESOURCES_DIR`・`WEZTERM_PANE`・`WT_SESSION`)でも見分け、DCS `tmux;` で包む(ESC は2つに重ねる)。画面では、糸が作ったバイトを受け取り、描いたあとに端末へ書く(描く途中の出力に混ぜない)。
- **OS の通知**: macOS は `osascript -`(台本は標準入力、題と文は引数)。Linux は `notify-send --app-name=mdgrid -- 題 文`。Windows と WSL は Windows PowerShell 5.1(`powershell.exe -EncodedCommand`)の WinRT のトーストで、登録の要らない PowerShell の AUMID で出す。題と文は環境変数で渡し、台本の文字に混ぜない(WSL は `WSLENV` に名前を足す)。WSL は Linux より先に、`WSL_DISTRO_NAME` かカーネルの版の名前(`/proc/sys/kernel/osrelease`)の `microsoft` で見分ける。却下: PowerShell 7(`pwsh`)は WinRT の型を読めない。却下: 通知の crate(依存を足さず、OS に元からある仕組みだけで出す)。
- **プログラムの動かし方**: シェルを通さず、出力は捨て、`TIMEOUT`(10 秒。tbd~notify.timeout)で止める。Windows では窓を出さない旗(`CREATE_NO_WINDOW`)を付ける。値の制御文字は、どの出す先でも空白にしてから渡す(`clean`)。
- **`--notify`**: 画面と同じ道(`open_app`)で App を作り、`notify_once` がその場で確かめる(表の読み方・範囲・いつも隠す条件は画面と同じ `notify_jobs`)。端末の合図は標準出力が端末のときだけ送る。理由は標準エラーに1行ずつ出し、届けられなかったか読めない表があれば終了コード 1。読めない表の理由には表の名前を添える。試験の出す先は、どの OS でも動くよう実行ファイル自身の `--new-note` にする(`tests/test_notify_e2e.rs`)。
- **試験**: 画面の試験は出す先を `TestOutlet` に差し替え(`#[cfg(test)]`)、記録は一時のフォルダに置く(App の `notify.dir` の既定は None で、main が状態の置き場を入れる。試験が本物の置き場に書かないため)。時刻は `notify_start(today, now)` に渡す。

## ルーチンの回(routine.*。2026-11-24)

- **核と画面を分ける**: 決まりの読みと回の日は `src/recurrence.rs`(語の表と読み方だけで、文言を持たない。datewords と同じく英語と日本語の形をどちらも読むので、日本語のリテラルを許す表に入れた)。印の `runs` の読み・回の名前と中身・記録と錠は `src/routine.rs`(表を知らない)。表をファイルから読んでルーチンの行を集める(`rows`。通知の欄・知らせと同じく `open_target` で開き、`.base` の最上位の絞り込みを通す)のと、開いている間の時と糸は `src/ui/routine.rs`。`--runs` は `main.rs` の `runs_cli` が画面と同じ道(`open_app`)で App を作り、`routine_once` を呼ぶ。
- **どの表がルーチンの表か**: 開いたとき(`App::start`)に、範囲の表(`scope_places`。範囲が無ければ登録した表。知らせの既定の範囲と同じ)と開いた表を、実体のパスで重ならないようにして `workspace::marker_runs` に渡す。`runs` は印の `[[table]]` を `marker_setting` で読む(tree・gantt と同じ形)。`WsTable`・`Scope` に欄を足さない(錠のある試験がその形を直に作るため)。範囲を読むとき(`read_tables`)は、印の `runs` を知らない項目にしない。`folder` と `body` は印の根からの相対で、`newnote::note_path`・`read_body` が外へ出るパスを断る。
- **回の日**: `Rule::latest(start, today)` は、今日から遡って最初に決まりに当たる日(毎週は 7 日、毎月は 62 日。31 日の回は、31 日の無い月を挟むと 2 か月前になる)。`every` は起点から `(today - start) / N * N` 日目。起点の列は `start` に決め打ちし、設定の鍵は足さない。
- **回のノート**: フロントマターは `writeback::apply(b"", …)` で作る(新しいノート・add-frontmatter と同じ書き方)。並びはリンク `routine`・日 `date`・`set`・`copy` で、同じ鍵は先のものを使う。名前の雛形は `{routine}` で切って、残りを `newnote::expand_name_ids` で回の日として埋める(ルーチンの名前の中の `{…}` を変数と読まないため)。本文の雛形も同じ切り方で `expand_ids`。`copy` の日付の形だけの値は日付のまま書く(引用符で囲まない)。
- **記録**: 見た目の状態の置き場の `routine.json`(`{"version":1,"made":{"<表の実体のパス>\t<ルーチンのノートのパス>":"YYYY-MM-DD"},"dismissed":{…}}`)。鍵ごとに、作った一番新しい回の日だけを持つ(一番新しい回だけを作るので、それより前か同じ日の回は作らない)。同じ名前のノートが既にあった回も、作ったとして記録する(上書きはしない)。作れなかった回は記録しない。読めない記録は空として読み、書くときに作り直した旨を出す。書くのは一時ファイルからの名前の変更(`config::write_atomic`)。ルーチンの表が無ければ、置き場に何も作らない。
- **錠**: 同じ置き場の `routine.lock` を `File::lock` で取り、記録を読んでから作って書くまで持つ(notify と同じ)。作るのは `newnote::create`(`create_new`)なので、錠の外で同じ名前のノートができても上書きしない(`AlreadyExists` は作れなかったとせず、記録だけする)。
- **作る操作は数え直す(2026-11-25)**: パレットの「ルーチンの回を作る」と下の帯のクリック(`start_routine_runs`)は、開いたときに数えた `due` を使い回さず、その時に表をファイルから読み直して `routine::pending` で数える。`pending` は `due` と違い、その日にやめた回も入れる(静かにするのは下の帯の知らせだけ)。読むのは画面の糸の中(裏の糸にしない)で、人が選んだ操作なので待ってよく、確かめの窓をすぐ開ける。今日は最後に確かめた日(`routine.day`。時計と違えばその日の 0 時)。並べた回は `RoutineState.asked` に持ち、確かめの答えで作るのもやめるのもこの並び。やめたら `due` からも外し、作ったら作れなかった回だけを `due` に残す。
- **画面**: `routine_poll` を画面を回すたびに呼ぶ。確かめるのは、まだ確かめていない日(開いたときと、日付が変わったとき)だけ。裏の糸が表を読み、`routine::due`(書かない)で来ている回を決める。同じ名前のノートが既にある回は、画面では数えない(`--runs` が記録する)。数は下の帯の問題の数の後ろに出し、クリックは `bands::routine_at`。作る操作は `Ask::RoutineRuns` の確かめで、`y` なら `routine::make`(錠を取り、記録を読み直してから作る。`--runs` が先に作っていれば作らない)。作ったノートのうち開いている表のフォルダの中のものは `Source::reload` で行にし、git にも渡す(新しいノートと同じ)。作れなかった回は帯の数に残し、理由はメッセージ行に出す。
- **やめた回**: 記録の `dismissed`(鍵ごとに、やめた日と回の日)に置き、その日のうちは `due` が外す。表を切り替えて App を作り直しても、mdgrid を開き直しても同じ。日が変わると忘れる(同じ回でも、次の日にはまた知らせる)。`--runs` は見ない(定期実行への登録を、作ってよいという合図とみなす。routine.cli)。
- **コマンドの表**: `routine_runs` はノートを書くが `READ_COMMANDS` に置き、ルーチンの表が無いときと読むだけの起動では `App::command_shown` が隠す。`COMMANDS` に置くと、その全部が既定の設定でパレットとヘルプに出ることを確かめる main の試験が落ちる(reverse-run。横断の検索と同じ)。
- **`--runs`**: `--readonly` と一緒なら、開く前に理由1行と終了コード 2。読めない表・読めない決まり・作れなかった回・記録を書けないときは、標準エラーに1行ずつ出して終了コード 1(決まりが読めないときも、人が直さないと回が来ないので 1 にした)。作り直した記録の旨は標準エラーに出して 0。印の警告は、ほかの画面を出さない起動と同じく `mdgrid:` を付けて標準エラーへ。
- **仮決め**: 名前 `runs`・`--runs`(tbd~routine.names)、決まりの形(tbd~routine.rule-syntax)、初めての回を作る(tbd~routine.first-run)、消した回は作り直さない(tbd~routine.deleted-run)、キーは割り当てない(tbd~routine.offer-key)、ルーチンの `last_done`・`next_due` は書き換えない(tbd~routine.next-due)。

## ワークスペースを横断して探す(navigation.across-*。2026-11-21)

- **入れ切り**: 設定の `[search] across`(全体だけ)が真のときだけ、パレットとヘルプにコマンド `across_search` を出す(`App::command_shown`)。コマンドは読むだけの起動でも出す `READ_COMMANDS` に置く。書くコマンドの表(`COMMANDS`)に置くと、既定の設定でパレットに出ることを確かめる main の試験が落ちる(reverse-run)。
- **入力と結果の場所**: パレットの続きの入力(`Ask::Across`)をそのまま使い、入力の行の下に、進み・表ごとの見出しと当たった行・読めなかった表を重ねる(`across.rs` の `across_lines`)。Enter は、打った語が探した語と違えば探し直し、同じなら選んだ行を開く。↑↓ は当たった行だけを動く(見出しは選ばない)。
- **読み方**: 裏の糸が範囲(`scope_places`)の表を1つずつファイルから開き(`main.rs` の `open_target`。通知の欄の `count_tables` と同じ道)、読み終えた表ごとに世代つきで送る。描く前の `across_poll` で拾い、読んでいる間は画面の待ちを 50 ミリ秒に縮める。Esc と探し直しは止める旗を立て、糸は表の合間と読み込みの合間に見て抜ける。
- **当て方**: 表の行は通知の欄と同じく、`.base` の最上位の絞り込み(`Base::keeps`)と、いつも隠す条件のあと。当てるのは表の中の検索(`nav::matches`。smartcase の部分一致)と同じく、ノートの名前と列の素の値。列は、`.base` なら開くビュー(登録したビューか先頭のビュー)の列、フォルダと CSV は表の既定の列(ノートのキー)。見せる名前は、表の行に共通のフォルダと `.md` を除く。
- **開く**: 選んだ行の表へは、ほかの道と同じく `switch_to`・`switch_select` で移り、検索の語は移る先のパスと組にして `switch_search` で渡す(移る先と違う表では使わない。保存の確かめで戻ったら捨てる)。移った表は `search_for` で検索の欄に語を入れ、読み終えてからその行を選ぶ。今の表の行なら、その場で語を入れて選ぶ。

## 縦に並べる(stack.*。2026-11-22)

- **表ごとの状態は今までの `App` のまま**: 並べた表の1つ1つが、1つの表を開いたときと同じ `App`。選んでいる表の App が画面を回す側の持つ App で、`Stack`(`src/ui/stack.rs`)を持ち、ほかの表の App を並び(`tables`。選んでいる表の場所は空き)に持つ。選ぶ表を移るとき(`Ctrl+w` `j`・`k`・クリック)は、App の中身を入れ替える(`std::mem::swap`)。だから検索・絞り込み・並べ替え・ビュー・セルの編集・新しいノートは、どれも今までの道のまま選んでいる表にだけ効き(stack.each-table)、画面を回す側(イベントのループ・開き直し・git)は今までどおり1つの App を見る。却下: 表の状態を App から抜き出して1つの App に複数持たせる(`self.rows` などを読む数百の関数を書き換えることになる)。
- **ためた変更は1つ**: `Changes` は選んでいる表の App が持ち、ほかの表は空のものを持つ。ほかの表を組み直す・描く・読み直す(poll)ときだけ、`with_table` がその表の App に渡して返す。同じノートの直しは1つの行の変更に重なり、基準(WB-4)も1つなので、1回の保存で両方を書く(stack.shared-pending)。取り消しの積みも1つ(どの表で `u` を押しても、最後にためた直しから戻る)。ほかの表は、ためた変更の世代(`Changes::generation`。変えうる操作のたびに増える)が前に組んだときと違えば、描く前に組み直す(`stack_sync`)。`Changes` の型は変えない(試験が `app.changes` を直に使うため。reverse-run)。却下: 表ごとに別の `Changes`(同じノートを2つの表で直すと、片方を書いたあともう片方の保存が外の変更として止まる)。
- **保存は全部の表をまとめた読み込み口で**: `AllSources` は並べた表の読み込み口を借りてまとめた `Source` で、行ごとに、その行を出している表(無ければ読んだ表。選んでいる表が先)へ渡す。差分の確認・保存・外の変更の上に書く・捨てる(`review.rs`)は `save_parts` でこれと `Changes` を受け取り、今までの `Changes::previews`・`save` をそのまま使う。書き戻しの約束(外の変更で止める。write-back.conflict-stop)は各表の読み込み口のまま。差分の確認の項目には表の名前を持たせ(`ReviewItem.table`)、並べた順にまとめて表の見出しを挟む(stack.save-all)。書いたあと、同じノートを読んでいるほかの表は読み直す(`stack_after_save`。次の poll を待つと、その間の直しの基準が古くなる)。
- **閉じる表の変更**: 閉じる表のためた変更のうち、残る表に出ていない行だけを確かめる(stack.close-pending)。確かめは終了の確認と同じ `Mode::Quit` を使い、帯の文と下の帯のキーの言葉だけを替える(`close_band`・`close_hints`)。保存は差分の確認をその表の行に絞り(`Stack.closing`)、書けたら閉じる。残る表に出ている行は、閉じても残る表の変更として見えて書けるので確かめない。
- **画面は区画に分けて描く**: 表の区画は、その表の App を区画の高さの画面として描く(`App::size` を区画の大きさにする)。区画では、ヘッダーを見出しの1行(`stack::title_line`)にし、上の余白・通知の欄・ビューの帯・設定の帯・下の帯・メッセージ行を出さない(`stack::in_region` を `bands::top_rows`・`App::shows`・`view::table_height`・`view::render` が見る)。通知の欄(notice.layout)は画面のいちばん上に、下の帯とメッセージ行は画面の下に1つずつで、どれも選んでいる表のもの(`stack::render`。区画は欄の行の数 `Stack.top` だけ下から始まる)。クリックは `stack_point` が区画の中の位置に直してから今までの道で受ける。`ui::draw` は `&App` のまま(試験が使う)で、画面を回す側は `ui::draw_screen`(`&mut App`。ほかの表を組み直してから描く)を呼ぶ。
- **ページの中の引き出し(stack.page-drawer-leave。2026-11-25)**: ページを開いているあいだは、引き出しの今の印をページの見出しだけに付け、表の行には付けない。表を選んだら、選んでいる表と同じでも閉じるだけにせず、`switch_to` で移る(main は `open_app` で1つの表を開き直すので、並びとページは無くなる)。
- **画面いっぱいに出すもの**: 表でない画面の型(ガント・カンバン・グラフ・木・関係マップ)と、ヘルプ・パレット・保存と終了の確認・詳細・ビューの設定・引き出し・絞り込みと並べ替えの窓・新しいノートの窓は、選んでいる表だけを画面いっぱいに出す(`stack::full_screen`)。窓の置き方と当たりの計算をそのまま使うため。モードを替えるたびに(`set_mode`)、区画と画面いっぱいの間を行き来したら大きさを合わせ直す(`fit_stack_size`)。区画の中で描くのは、表・セルの入力・候補とリストの選択・操作の一覧・頻度表・列の型の窓と、メッセージ行で打つ検索と、画面の上の通知の欄へ移った間(欄のクリックも画面の上の欄で受ける)。
- **高さ**: 高さの比(`Stack.weights`)で分け、端数は上の表から1行ずつ足す。表ごとに見出しと3行(`MIN_H`)。入らなければ、選んでいる表を残して下の表から見出しの1行だけにする。`Ctrl+w` `+`・`-` は、今の高さを比にしてから、選んでいる表と下の表(いちばん下なら上の表)の間で1行動かす。ページの `height` には比を最大公約数で割って書く(`page::ratio`)。
- **表を開くのは画面を抜けてから**: 「下に表を並べる」とページを開くのは、登録した表へ移るとき(CLI-19)と同じく、画面を一度抜けて main の `open_app` で開いてから戻る(`StackRequest`)。設定の読み込みと端末の地の色の問い合わせを、画面の中でしないため。ページを開くときは、ためた変更があれば表を移るときと同じく終わる手順で確かめる。
- **ページの置き場**: 範囲(Scope)の型は変えない(試験が全部の項目を書いて組み立てるため)。ページは `App.pages` と置き場 `App.page_home`(`workspace::PageHome`)に持ち、範囲を決めたあとに読む(`workspace::page_home`・`load_pages`)。読み書きは `src/page.rs`(形と読み方)と `src/workspace.rs`(文字のまま書き足す)。workspaces.toml は、そのワークスペースの区画の末尾に `[[workspace.page]]` の区画を足し(同じ名前なら置き換え)、読み直してワークスペースと表が前と同じで、書いたページが読めるときだけ書く。並びから書き直す道(`checked`)は、前の文のページの値を写して落とさない。印は、ファイルの末尾に区画を足すだけ(同じ名前なら、その区画だけを置き換える)で、ほかの行・並び・コメントは1字も変えない(stack.page-save-marker)。toml_edit は使わない(依存を足さず、今の文字のまま書き換える道に合わせる)。
- **開き直し**: `Reopen.page` を足し、ページを開いていれば `-w <ワークスペース> --page <名前>`(印のワークスペースは `<表のパス> --page <名前>`)にし、窓の題はページの名前(stack.page-resume・resume.herdr-title-form)。その場で並べた形は、選んでいる表とビュー(`Reopen::retarget`)。表を足したり閉じたりしたら、ページの名前は外す(並びがページと違うため)。
- **未確定**: 表の間のキー(tbd~stack.window-keys)とページの名前(tbd~stack.page-name)は仮のまま。ページの表の並びの鍵 `tables`(`{ table, view, height }` の並び)は、要求に名前が無いので仮に決めた。

## 文言の台帳と言語(screen.locales-*。2026-11-22)

- **台帳のファイルが言語**: `locales/<符号>.toml` を足せば言語が増える(build.rs が言語の値・設定の値・言語ごとの `const fn` を作る)。符号は言語(`ja`)か言語と地域(`zh-TW`・`pt-BR`)。英語の台帳が名前と並びの正本で、ほかの台帳の欠け・余り・空・差し込みの番号の食い違いは組み立てで落とす。組み込みのスターターの `[name]` も全部の言語が要る。
- **選び方**: `language` が `"auto"` なら `LC_ALL`・`LC_MESSAGES`・`LANG` の順で最初の空でない値から、言語と地域が合う台帳、無ければ言語が合う台帳(中国語は TW・HK・MO なら繁体字、ほかは簡体字。ポルトガル語は pt-BR)、どれも無ければ英語(`i18n::locale_index`)。符号は大文字小文字を問わない。
- **台帳の外に文言を持たない**: 設定の項目の説明は `Cfg<道筋>`、`--print-config` の頭・範囲・例の見出しも台帳に置き、`--print-config` は今の言語で出す(`schema::Item` の `en`・`ja` は台帳の英日を引いた値で、文書との突き合わせの試験が使う)。
- **試しの言語**: 環境変数 `MDGRID_PSEUDO_LOCALE` を付けると、英語の文の頭に `⟦` を付けた言語(`Lang::Pseudo`。言語の一覧には入れない)になる。台帳の外に文言が残っていれば、`⟦` の無い文か日本語の字として試験で見つかる(tests/test_locales_e2e.rs)。
- **はみ出し**: 部品の文字は `width::fit` で幅に切る。どの言語でも 80 桁でヘッダーの右の端のボタンと設定の画面のボタンが残ることを、言語ごとに試験で見る。
- **訳さないもの**: キーの名前・旗・設定の項目と値・式・ファイルの名前・ノートの中身。日付の言葉(`src/datewords.rs`)は日本語と英語だけを読む。推定に使う列の名前の並び(`GanttStartNames` など)は全部の言語の台帳を合わせて使う。


## ファイルを読む口(write-back.refuse-not-text・write-back.refuse-not-text-any-file・cli.read-text-only。2026-11-23)

- **口は1つ**: mdgrid が読むファイル(ノート・CSV と TSV の台帳・`.base`・設定・ワークスペースの印・状態・雛形・`--apply` の入力・通知の連絡のノート)は、どれも `src/textfile.rs` を通す。`std::fs::read`・`std::fs::read_to_string` は `clippy.toml` の `disallowed-methods` で止まるので、読む所を足すときに判定を通し忘れない(試験のクレートと build.rs は許す)。
- **文字のファイルの決まり**: 先頭の 8,000 バイト(`frontmatter::SNIFF_LEN`。git が文字か否かを見る幅)に NUL がある・UTF-16 の BOM で始まる・UTF-8 として読めない、のどれかなら文字のファイルでない(`frontmatter::text_kind`・`textfile::check`)。頭の印(`MZ`・`PK` など)では決めない。同じ字で始まるふつうのノートを誤って止めるため。
- **読み方は3つ**: 文字として使うものは `read_text`(文字のファイルでなければ `InvalidData` の誤りで、文は理由)。書き戻すために元のバイトが要るノートと CSV は `read_bytes` で読み、使う前に `frontmatter::parse`(ノート)か `textfile::check`(CSV)を通す。読んで書き換えるもの(`places.toml`・`workspaces.toml` など)は `read_text_for_edit` で、無ければ空、読めない・文字のファイルでなければ誤りにして書かない(読めない中身を空とみなして上書きしない)。
- **ノートの理由**: 読むだけの理由の型 `ReadOnly` には種類を足さず、`NotUtf8` を「UTF-8 のテキストとして読めない」に広げて、理由の文だけを中身で分ける(`source::markdown::read_only_reason`)。種類を足すと、網羅の `match` を持つ試験(tests/test_safety_docs.rs)が前のコードで組み立てられず、新しい P0 の f2p の門が回らないため。
- **エディタと URL**: エディタに渡す前にもファイルの頭を見て、文字のファイルでなければ渡さない(UTF-16 は渡す)。ブラウザに渡す URL は `folder_tree::url_refusal` で、最後の部分の拡張子(`\` は `/` と読み、`%` を戻す)と向きを変える文字を見る。

## 名前の見せ方をそろえる(base-view.display-name-everywhere・screen.note-name-everywhere・cli.print-group-order。2026-11-25)

同じ列・同じノート・同じ表が、画面の所によって2つの名前で見えていた。見せる所ごとに直すのでなく、引く口を1つにして、新しい所もそこを通す。

- **列の名前は `App::title`**: 組んだ表の列の見出し(`built.titles`)、無ければ `.base` の displayName(`Base::column_title`。ビューに出していない列にも効く)、無ければ Obsidian の既定。帯の札(`Settings::chips_titled` に渡す)・ビューの設定の絞り込みの一覧・まとまりの見出し・カンバンとガントの題・新しいノートの窓の欄・木の画面の列の見出しが使う。`Settings::chips` は id のままの文を返す(核は画面の見出しを知らない)。ノートに書く値・式・設定・`--print` の鍵は id。
- **ノートの名前は `view::note_name` と `view::note_name_of`**: 表の行のノートは `note_name`(`.md` を除き、note_path と note_name)、表の行でない名前(関係マップのリンクの組・リンクの列の候補・横断の検索の結果)は `note_name_of` で `display.note_name` だけを当てる(フォルダは残し、最後の部分だけ。`NoteName::apply_last`)。今の表の設定を当てる(リンクのセルの行き先と同じ)。ほかの表の範囲に書いた note_name は見ない。数えるのと当てるのは元の名前で、見せる所だけを変える(関係マップの「よく指される」は元の名前ごとに数える)。
- **表の名前は `bands::table_name`**: 印の範囲で、開いたパスの `[[table]]` にパスと違う `name` があればその名前、無ければ `.base` の名前かフォルダの名前。ヘッダーのパンくずと通知の欄の今の表が使う。最近開いた表は `Recent::name` が、そのパスの上の印を読んで同じ引き方をする(`resume::table_name`)。workspaces.toml で名前を付けた表のパンくずは、まだフォルダの名前のまま。
- **`--print` の並び**: `print::table_with` が値の候補(`[rules.<列>] values`)を受け、`.base` の groupBy は `settings::order_groups_by`(画面の `order_base_groups` と同じ関数)、mdgrid のビューのグループは `settings::apply_with` で並べる。候補は `resolve_table`(画面と同じ範囲の重ね)から読み、`--count` と `--check` では読まない。`print::table` は候補を見ない形のまま残す(試験と `--apply` が使う)。
- **値を選ぶ窓**: `App::with_value_labels`(セルを直すときと新しいノートの窓の候補)と `App::value_choice_text`(リストの値の付け外し・絞り込みの窓とビューの設定の値の一覧・値の件数の窓)が、`[rules.<列>] labels` の名前のある値の項目の文字を「名前 値」にする(`Resolved::value_choice`。navigation.value-labels-picker)。書く値(`Item::value`)と絞る値は値のまま。
- **値の名前をセルにも(navigation.value-labels-cells。2026-11-25)**: 人が「出す」と決めた。セルの文字は `cell::shown` の中で、日付・リンクでない値に `cell::label_of`(`Resolved::value_shown` を1行にして無害にしたもの)を当てる。札の部品は `CellPart::Chips` の要素を (値, 見せる文字) の組で持ち、色と形の記号は値から、文字は見せる文字から作る(`chips::parts_named`)。名前で色が変わると、done の緑のような意味の色と、設定の値の色(`[look.colors.values]`)が効かなくなるため。値を見せるほかの所(詳細の `detail_text`・木の画面の右の欄)は `cell::shown_plain` か `cell::labeled_text` を通す。比べる・書く所(`App::plain`・まとまりの鍵・カンバンのレーンの値・同じ値の絞り込み・コピー・書き出し)は値のまま。`\` の簡易の絞り込みは、値と見せる名前のどちらでも当てる(`tbd~navigation.value-labels-match` の仮決め。`/` の検索は前からセルの文字でも当てていた)。名前は values にある値にだけ当てるので、values に無い値は値のまま `!` が付く。
- **ほかの表の列の名前は `App::titles_of` と `App::title_in`**: 関係マップとつながった行の一覧は、ほかの表の列を見せる。表のフォルダ(`relations::Table`)から、同じフォルダになる範囲の表(`link_places`)を探し、そのパスが `.base`(印の中の表も)なら `relations::table_base` で読んで `Base::column_title`、`.base` でなく今の表なら `App::title`、ほかは既定の見出し。関係マップは開くときに表ごとの名前を1回だけ引いて `RelMap` に持ち、箱は `relmap::layout_titled`、狭い画面の一覧は `relmap::link_text_as` に渡す(つながりを結ぶ名前と添字はキーのまま)。つながった行の一覧は表ごとに1回だけ引く。
- **関係マップの逆向き(relations.map-reverse)**: `relmap::reverse_of` が、行き先の表から元の表へ戻るつながりのうち、組 (a, b) に対して (b, a) を持つものを返す。表の形(元と行き先の表)だけで決めると、同じ表の中のつながりは全部が互いの逆向きになっていた。組は1つのつながりで最大 500 なので、大きな表では指し返す組を取りこぼすことがある。

## 見た目の状態の列の並びと印の order(screen.state-store-arranged-only・screen.state-store-follow-order。2026-11-25)

開いただけで列の並びを書くと、あとで人が印の `order` に足した列が右の端に回っていた(状態の並びに無い列は右に足す決まり)。

- **書くのは人が動かしたときだけ**: `persist_state` は、並び(隠した列を戻した位置に入れた並び)が状態の無いときの並びと同じで隠した列も無ければ、並びを空にして書く。比べる相手は、印のビューの `order`(`App::marker_order`。mdgrid のビューと `order` の無いビューは None)、無ければ組んだ表の列の並び(`built.natural`)。幅・畳んだまとまり・ビューの設定は今までどおり書く(どれも人が変えたものしか入らない)。動かしたかどうかの旗を持たないのは、動かして戻したときも状態の無いときと同じにするため。
- **控え**: 並びを書くときは、そのときの印の `order` を状態のファイルの最上位の `base_order` に書く(`config::save_state_with_base`)。`ViewState` に項目を足さないのは、試験が `ViewState` を項目の全部で組み立てていて、main の試験がコンパイルできなくなるため(reverse-run)。
- **合わせ方(`config::follow_order`)**: 開くとき、控えと今の `order` が違えば、今の `order` から始め、人が動かした列(控えから覚えた並びへの最長の共通の並びに入らない列)のうち、印も動かしていない列(控えから今の `order` への最長の共通の並びに入る列)だけを、覚えた並びで直前にあった列の後ろ(無ければ先頭)へ移す。直前の列を印が動かしていれば食い違うので移さない。最長の共通の並びが2つ以上あるときは、控えの前の方の列を残す方を取る(abc → bac は「b を先頭へ」と読む)。印が消した列は戻さない。隠した列は名前で持つので、並びを合わせても隠したまま。
- **控えの無い前の版の状態**: 前の版は開いただけでも並びを書いていたので、人が動かした並びと区別できない。印の `order` があれば捨てて `order` に合わせる(tbd~screen.state-store-legacy-order)。
- mdgrid のビュー(views.toml)は、前から定義の指紋(`ViewState::view`)が変われば定義の並びに戻すので、この合わせ方は使わない。

## 一覧の窓のキー(screen.checklist-keys・screen.window-key-hints。2026-11-25)

チェックを入れるつもりの Enter で窓が閉じ、入れたはずのチェックが入らなかった。窓の中で何を押せばよいかも、画面の端の下の帯まで目を移さないと分からなかった。

- **チェックの窓の Enter は付け外し**: リストの値の付け外し(`listpick.rs`)と絞り込みの窓の値の一覧(`filters.rs` の `Step::Values`)では、Enter(`run`)も Space(`toggle`)と同じく選んだ行のチェックを付け外しし、窓は開いたまま。決めるのは、どの行でも Tab(`commit`。絞り込みの窓には `[keys.filter_window]` の Tab を足した)と、窓の一番下に足した「[ 決める ]」の行での Enter かクリック。リストの値の付け外しの検索中は今までどおり、Space は空白、Enter は選んだ候補を付けるだけで外さない(cell-edit.list-candidates-no-dup の「打って足す」を、外すことにしないため)。
- **形は変えない**: `Pick::rows` と `PickRow` は変えず(試験がそのまま組み立てる)、「[ 決める ]」の行は選びの添字 `rows().len()` で表す。描くときの中身の行は `listpick::Body`(候補・当たり無し・決める)。絞り込みの窓は値の一覧の行の並びの最後に足す(`sel > counts.len()` が決める行)。
- **新しいノートの窓のリストの欄も同じ**(2026-11-25 に変えた): 前は Enter で欄を決めて次へ進め、「[ 決める ]」の行を出さなかったが、値を入れるつもりの Enter で何も選ばないままノートができた。今はセルの窓と同じキーと「[ 決める ]」の行・下の縁のキーで、決める・やめるだけを `new_note_pick_action` が先に受ける(Tab と「[ 決める ]」の Enter は次の欄へ、最後の欄なら作る。Shift+Tab は前の欄。リストの選択の Shift+Tab はセルの窓では何もしない)。クリックも `pick_apply` で同じ道を通す。
- **スターターのオプションの一覧**: starter.pick-options の画面がまだ無い。作るときに同じキー(Enter・Space で付け外し、「[ 決める ]」か Tab で決める)にする。
- **下の縁のキーはキーの表から**: 窓ごとに並べたい動作の並び(`(Action, Option<Msg>)`)を持ち、`popup::key_hints` が今のキーの表(割り当て直したあと)の最初のキーと表示名で「キー 動作」の字を作る。キーを外した動作は出さない。名前は表の表示名で、段で意味の変わる所だけ文言を添える(絞り込みの窓の値の一覧の Enter は「入れる・外す」、段の中の Esc は「戻る」、並べ替えの窓の Enter は縁の短い名前「向き・足す」)。リストの値の付け外しのキーの表示名は、縁にそのまま出せる形にした(Enter「入れる・外す」・Tab「決める」・Esc「やめる」)。
- **並べ方とクリック**: 左下の角・横の線1字・空白1字のあとに、案内を2桁あけて並べ、右に空白1字・横の線1字を残す(ガントの操作の帯と同じ形)。入らない案内は後ろから省き、縁の線と字を重ねない(`bottom_hints`)。クリックの当たりは描くときと同じ置き場(`hint_at`)で、その動作を窓のモードの動作として渡す(キーを押したのと同じ道)。値の候補の窓と操作の一覧は、下の縁の左に今までの「選んでいる番号/数」を置き、その後ろに並べる。値の候補の窓を入力の欄の上に出すときは下の縁が入力の欄の側になるが、そこにもキーを並べる(窓は今までどおり縁で閉じる)。
- **窓の幅**: 下の縁も窓の中身として、キーが全部入る幅まで窓を広げる(`widen_for_hints`)。右へはみ出すなら、中身と同じく左へずらす(各窓の置き方のまま)。画面が狭くて入らないときだけ省く。並べ替えの窓はボタンの左の端にそろえて置く(NV-23)ので、幅 80 の端末でボタンから右に収まるよう、Enter と Esc だけを短い名前で並べる(外すのは各決まりの行の右の端の「×」と下の帯にある)。引き出しは今までの幅の上限(40)の中で広げる。
- **出す窓**: 値の候補(`list.rs`。Enter 確定・Esc 取り消し)、リストの値の付け外し、絞り込みの窓(段ごと。並びでは d の外すも)、並べ替えの窓(並びと列を選ぶ段)、切り替えの一覧、引き出し、画面の型のメニュー、操作の一覧(どれも Enter と Esc)。列の型の窓と頻度表は要求の一覧に無いので出さない。

## まとまりごとの集計と数の見せ方(base-view.summaries-per-group・base-view.summary-labels・cell-view.number-format*。2026-11-25)

お金の台帳で、月ごと・分類ごとの小計が出ず(まとまりと集計を両方書くと下の帯に「未対応」が出続けた)、金額が「98000」と出た。

- **小計の置き場は見出しの行の右**: まとまりの末尾に行を足す形(表の道具の一部の形)は、まとまりごとに1行ずつ表が縮み、畳むと見えなくなる。見出しの行の右なら行を増やさず、畳んでも残るので、全部を畳むとまとまりごとの合計の一覧になる。小計は表の下の集計の行と同じ「名前 値」で、同じ列の位置にそろえる(数の列は右寄せ)。見出しの字は最初の小計の列の手前まで(入らなければ `…` で切る)。切れないように、`layout` がノートの欄を、一番長い見出しが入る幅まで、1列の上限(画面の幅の3割)を越えない範囲で広げる(`heading_room`)。ノートの欄を出さない表と横に送った表では広げず、切れることがある。
- **計算**: 小計はまとまりの行(`App::groups` の範囲。ビューの設定・簡易の絞り込み・親子の並べ直しのあと)を、表の下の集計と同じ `compute_summaries_in` で数え、`Built::group_summaries` に `groups` と同じ順で持つ。見出しの件数と同じ行を数える。`.base` の `groupBy` でも、ビューの設定のまとまり(NV-21)でも同じ。
- **集計の名前**: ビューの `summary_labels`(mdgrid の足したキー。`tab_break` と同じく Obsidian は無視する)を `View::summary_labels` に読み、`Grid::summary_labels` で画面に渡す。文字の値だけを読み、数や真偽は書かなかったことにする。`.base` の最上位の `summaries` は式の集計の名前を決める所で、組み込みの集計に名前を付ける書き方は Obsidian に無い。
- **数の見せ方は表のプロファイルの項目**: `[format.<列>]` は `[types]`・`[rules]` と同じく全体・ワークスペース・表・ビューに書け、項目ごとに狭い範囲が勝つ(`numfmt::FormatLayer` → `numfmt::NumFormat`)。決まり(`[rules]`)に入れないのは、計算の列(`formula.*`)にも書きたく、決まりは計算の列に書けないため。当てるのは画面の字だけ(`cell::number_text`・`number_val_text`・`view::summary_cell`・グラフの `vnum`)で、値・並べ替え・絞り込み・式・編集の欄・`--print` は数のまま。名前のある値(`labels`・`icons`)は名前が先。件数を数える集計(`Summary::counts`)と件数のグラフには当てない。
- **丸め**: `decimals` は桁をずらして `f64::round`(0.5 は 0 から遠い方)で丸めてから書く(`{:.n}` は2進の値で偶数へ丸めるので、0.25 が 0.2 になる)。整数(`Value::Int`)は f64 にせずに桁を区切る(2^53 を越える数も桁を落とさない)。指数でしか書けない大きな数は値の字のまま頭と末尾だけ付ける。
- **頭と末尾の字と集計の名前の字**: 印は人から受け取った台帳にも入っているので、`read_format` は制御の文字と文字の向きを変える書式の文字(`numfmt::unsafe_char`)を含む `prefix`・`suffix` を警告して使わない。描く所(`cell::number_text`・`number_val_text`・`view::summary_cell`・グラフの `vnum`)でも `sanitize` を通して二重に守る。`summary_labels` は `.base` の読み取りに警告の道が無いので、描くときの `sanitize` で見える字にする。
- **言語の区切り**: 文言の台帳の `NumSample`(`1,234.5` の形の見本)から、1 と 2 の間を桁の区切り、4 と 5 の間を小数点として読む。台帳は空白だけの文を許さない(フランス語の区切りは狭い空き U+202F)ので、区切りの字を1つずつの文言にせず見本にした。試しの言語(頭に `⟦`)も同じ読み方で英語の区切りになる。

## つながったノートの値(base-view.lookup-*・base-view.linked-from・base-view.list-*・base-view.expr-step-budget・cell-view.lookup-shown。2026-11-25)

読書・人と会議・調べ物の台帳で、リンクの先の値を見たい・この行を指すノートを数えたいが、式は自分の行の値しか読めなかった。表の道具の `.base` の書き方(`asFile()`・`.properties`・`filter`・`map`・`reduce`)を先に受け、足りない所だけを足した(リストの `max`・`min`・`sum` と `file.linkedFrom`)。

- **ノートの値 `Val::File(stem)`**: stem は根からのパスの `.md` を除いたもの(`file.links` の値と同じ形)。文字にすると stem(`join`・`--print`)。名前にするのは画面だけ(`cell::computed_text`)。
- **引く口 `expr::Notes`**: 式の評価はほかのノートを知らないので、`Links.notes` に口を添える。`base::SrcNotes` が `Source::note_row`(stem → 行の鍵。Markdown が読み込みのたびに1回作る表)・`Source::file`・ためた値を重ねた `prop`(画面は `App::prop`)で引く。
- **リンクの文字の解き方**: `relations::parse`(REL-1 と同じ値まるごとの形)のあと、`[[…]]` と `[…](…)` は `links::Index::resolve`、ただのパスは `Index::path_note`(名前では探さない)。`asFile()` とリンクの文字に付けたノートの項目はこの行のフォルダから、`.properties` で引いた値は引いたノートのフォルダから解いて `File` にする(`Cx::linkify`)。`==`・`!=`・`contains*` は、片方にノートがあれば、もう片方のリンクの文字もノートにしてから比べる(`Cx::as_notes`)。純関数の `equal` は文脈を持たないので、比べる前に寄せる。
- **`file.linkedFrom(キー)`**: `links::Raw` にフロントマターのどのキーから拾ったかを持たせ、`Index::build` が行き先 → キー → 元の stem の表(`keyed`)を被リンクと一緒に作る。行ごとに全部のノートのフロントマターを見直さない。キーで引くので、本文のリンクは入らない。保存したノートから作るので、ためた値で足したリンクは保存してから数える(tbd~lookup-saved-links)。
- **filter・map・reduce**: 読むときに、1つ目の引数の中の素の `value`・`index`(reduce の中なら `acc` も)を `Node::Var` にする(`Parser::lambda`)。`note.value` と外の `value` は今までどおりフロントマターのキー。評価は要素を `Cx.scope` に積み、内側から探す。
- **手数の上限**: 入れ子の filter・map で評価の回数が掛け算で増え、印に書いた式1つで画面を固められた。`Cx.steps` を1つの評価(`eval_checked`)で共有し、`ev` の入口で1手、たどる被リンク・行き先とリストのメソッドは要素の数、重ね除きは比べた回数を数える。越えたら Null を返して畳み、`eval_checked` が理由を返す。`base::Ctx::eval` がそれを `mark_bad` に渡すので、セルは評価できない式の印(BV-7)になり、絞り込みならビューを開かない。formula は行ごとに1回だけ評価して覚えるので、参照し合っても数を重ねない。上限は 20 万手(tbd~expr-step-budget-size)。上限はセルごとなので、上限まで使う式の列を数千行で並べ替えると、行の数だけ掛かる(表の組み立て全体の上限は持たない)。
- **引いた値の見せ方**: `Expr::lookup_key` が、式の外側が `.properties.<キー>`(とそれを map・filter・sort・max などで包んだもの)ならそのキーを返し、`App::lookup_key` 経由で画面がそのキーの値の名前を当てる。決まりは今の表のもの(tbd~lookup-labels-from)。日付は式の列でもセルと同じく `dates.format` で見せる(前は式の列だけ `YYYY-MM-DD` のままだった。既定の形は同じ)。
- **範囲**: 引けるのは読み込み口のルートフォルダの下のノート。`[[table]] path` の表はそのフォルダが根なので、ほかのフォルダのノートは引けない(tbd~lookup-scope)。

## ブログ(site.blog-*。2026-11-23)

- **道具を足さない**: サイトを組む `scripts/pages.py` に足し、静的サイトの生成器は入れない。記事の Markdown を HTML にする所は `scripts/mdhtml.py` に分け、標準ライブラリだけで書く(Pages の workflow と試験に依存を足さないため)。扱う形は記事に要るもの(見出し・段落・入れ子の箇条書き・表・コードのブロック・引用・リンク・画像・強調・コードの字)に絞る。生の HTML は文字として出し、リンクと画像の行き先は http・https・mailto と相対の道筋だけを受ける(記事に書いた字がスクリプトとして動かないため)。
- **フロントマター**: 記事の鍵は `title`・`date`・`description`・`updated`・`draft` だけで、`鍵: 値` の行だけを読む(YAML の読み手を足さない)。名前・足りない鍵・日付の形の誤りは、作り始める前に全部並べて止める。下書きも確かめる(公開に切り替えたときに止まらないため)。
- **出さないあいだ**: 公開する記事がどの言語にも無ければ、一覧・フィード・言語を決めて移る `blog/` を作らず、上の帯のリンクも出さない。仕組みを main に入れても、最初の記事を公開すると決めるまでサイトに跡を出さないため。片方の言語に記事があれば、両方の一覧を作る(もう一方は「まだ無い」の1文)。見た目も `scripts/site/blog.css` に分け、ブログのページにだけ埋める(どのページにも埋める `site.css` に置くと、注釈ごと全部のページに出るため)。
- **URL と言語**: URL の形は `tbd~site.blog-url` の仮決め(一覧 `<言語>/blog/`・記事 `<言語>/blog/<名前>.html`・フィード `<言語>/blog/feed.xml`・言語を決めて移る `blog/`)。言語の切り替えは、もう一方の言語に同じ名前の記事があればその記事、無ければその言語の一覧を指す。hreflang は訳のある記事だけに付け(x-default は英語の版)、`scripts/check_site.py` も訳の無い記事には求めない。画像は `docs/blog/images/` に置き、言語ごとの `blog/images/` に写す(記事からは `images/<名前>` で指し、道筋を書き換えない)。
- **試験**: `pages.py --blog-only` で、ブログと言語の入口とサイトの地図だけを作れる(説明書と撮った画面が要らない)。`tests/test_pages_blog.rs` は、記事を試験の中に書いてこれで組み、作ったファイルを見る。Python 3.11 以上が無ければ飛ばす。
