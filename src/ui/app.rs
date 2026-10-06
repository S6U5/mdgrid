//! 画面の状態(`App`)と、意図(`Action`)を受けて状態を変える `App::apply`。
//! 描画(view)も入力(keymap)も、状態を直接は書き換えない。
//! `apply` は振り分けだけにし、機能ごとの処理は `impl App` の続きとして別のファイルに置く
//! (grid.rs: 表の組み立てと列、nav.rs: 移動・検索・絞り込み・選択、detail.rs: 詳細の表示、
//! input.rs: 入力ボックス、calendar.rs: 日付のカレンダー、review.rs: 保存の確認、new_note.rs: 新しいノート、help.rs・external.rs)。
//! 核には `Source`・`Changes`・`Base` だけを通して触れる(SC-14)。

use super::detail::Detail;
use super::external::Clipboard;
use super::grid::{BaseFile, Drag, Slot};
use super::help::{HelpState, PaletteState};
use super::input::Input;
use super::keymap::{self, Action, Binding, Mode};
use super::nav::Prompt;
use super::review::Review;
use super::settings::Draft;
use super::view;
use mdgrid::changes::Changes;
use mdgrid::config::Config;
use mdgrid::i18n::Msg;
use mdgrid::print::today_now;
use mdgrid::source::{RowId, Source};
use mdgrid::theme::Theme;
use mdgrid::types::{DateFormat, Kind, WeekStart};
use mdgrid::vault::Progress;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::{HashMap, HashSet};

/// 色の出し方(SR-10・SR-15)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    None,
    Indexed,
    Rgb,
}

impl ColorMode {
    /// 環境変数から決める: `NO_COLOR`(空でない)か `TERM=dumb` で色なし、`COLORTERM` が truecolor / 24bit でなければ 256 色。
    pub fn detect(env: impl Fn(&str) -> Option<String>) -> ColorMode {
        if env("NO_COLOR").is_some_and(|v| !v.is_empty()) || env("TERM").as_deref() == Some("dumb")
        {
            return ColorMode::None;
        }
        match env("COLORTERM").as_deref() {
            Some("truecolor") | Some("24bit") => ColorMode::Rgb,
            _ => ColorMode::Indexed,
        }
    }
}

/// `TERM=dumb` か(SR-10)。そこでは色に加えて絵文字も出さない(値の中の絵文字は `?` に置き換えて描く)。
pub fn dumb_terminal(env: impl Fn(&str) -> Option<String>) -> bool {
    env("TERM").as_deref() == Some("dumb")
}

pub struct App {
    pub(crate) src: Box<dyn Source>,
    pub(crate) changes: Changes,
    pub(crate) mode: Mode,
    /// 開いた `.base` と選んだビュー(BV-13)。None なら既定の表(BV-1)。
    pub(crate) base: Option<BaseFile>,
    /// 組み立てた表の行(並べ替え・絞り込みの後。グループの順)。
    pub(crate) rows: Vec<RowId>,
    /// groupBy の見出しと rows の範囲(BV-5)。
    pub(crate) groups: Vec<(String, std::ops::Range<usize>)>,
    /// 畳んだグループの見出し(SR-2)。
    pub(crate) folded: HashSet<String>,
    /// 画面の行(見出しとノートの行)。`row`・`top` はこの添字。グループが無ければ rows と同じ並び。
    pub(crate) slots: Vec<Slot>,
    /// 未対応の列・並べ替えの説明(BV-7)。
    pub(crate) notes: Vec<String>,
    /// CE-28: 起動の間だけ足した列(どのノートにも無いキー)。どのビューでも表の右に出す。
    pub(crate) extra_cols: Vec<String>,
    /// 表の下の集計(列の id・集計・今の行で計算した値。BV-14)。空なら集計の行を出さない。
    pub(crate) summaries: Vec<(String, mdgrid::summary::Summary, mdgrid::expr::Val)>,
    /// 開けないビューの理由(BV-7・SC-8)。
    pub(crate) view_error: Option<String>,
    /// 表の列の順(列の id)。表の側が持ち、新しい列は右に足す。
    pub(crate) cols: Vec<String>,
    /// 列の id → 見出し(displayName。BV-5)。
    pub(crate) titles: HashMap<String, String>,
    /// SR-29: 読み込んだ全部の行に共通のフォルダ(末尾の `/` まで)。左のノートの欄はこれを除いて出す。
    pub(crate) label_prefix: String,
    /// 列の型(CV-4 の寄せ・CV-2 の `!`)。cols と同じ並び。
    pub(crate) kinds: Vec<Kind>,
    /// 列の id → 型。組み立て直しのたびに `.base` の結果の行の全部から決める(kinds とビューの設定が引く)。
    pub(crate) col_kinds: HashMap<String, Kind>,
    /// 手で決めた列の幅(SR-3・NV-4)。
    pub(crate) widths: HashMap<String, usize>,
    /// 選んだ行と列(slots・cols の添字)。
    pub(crate) row: usize,
    pub(crate) col: usize,
    /// 見えている先頭の行と、横のスクロールの先頭の列(SR-3)。
    pub(crate) top: usize,
    pub(crate) left: usize,
    pub(crate) message: Option<String>,
    pub(crate) progress: Progress,
    pub(crate) cancelled: bool,
    /// 前置きのキー(`g`)。
    pub(crate) prefix: Option<String>,
    pub(crate) color: ColorMode,
    /// 絵文字を出さない(`TERM=dumb`。SR-10)。描画の終わりで値の中の絵文字を `?` に置き換える。
    pub(crate) no_emoji: bool,
    /// East Asian Ambiguous を幅2で数える(CV-6)。
    pub(crate) ambiguous_wide: bool,
    /// 式の今日(日数)と今(UNIX 秒)。
    pub(crate) today: i64,
    pub(crate) now: i64,
    /// 端末の大きさ(最下行と右端の1桁を含む)。
    pub(crate) size: (u16, u16),
    /// 編集のモードの入力ボックス。
    pub(crate) input: Option<Input>,
    /// リストの選択(CE-16・CE-17)。
    pub(crate) pick: Option<super::listpick::Pick>,
    /// 保存の確認のモードの差分。
    pub(crate) review: Option<Review>,
    /// 終了の確認で「保存する」を選んだ(全部書けたら終わる)。
    pub(crate) quit_after_save: bool,
    /// キーの表(SR-4)。既定は keymap::BINDINGS の写し。振り分け・下の帯・ヘルプ・パレットはこれを引く。
    pub(crate) keys: Vec<Binding>,
    /// ヘルプ(SR-5)とパレット(SR-14)の状態。
    pub(crate) help: Option<HelpState>,
    pub(crate) palette: Option<PaletteState>,
    /// エディタで開くよう頼まれた行(SR-8)。端末を持つ main が `open_editor` で開く。
    pub(crate) editor_request: Option<RowId>,
    /// コピーの送り先(OUT-1)。試験では OS のクリップボードに書かないものに差し替わる。
    pub(crate) clipboard: Box<dyn Clipboard>,
    /// 列の境界のドラッグ(SR-3)。
    pub(crate) drag: Option<Drag>,
    /// ためた値が変わった(apply の終わりに表を組み立て直す。NV-12)。
    pub(crate) regrid: bool,
    /// 検索の語(NV-1)と、最後に移った一致の位置(画面の行, 0 はノートの名前・j + 1 は列 j)。
    pub(crate) search: Option<String>,
    pub(crate) search_at: Option<(usize, usize)>,
    /// 簡易の絞り込みの語(NV-2)。
    pub(crate) filter: Option<String>,
    /// 簡易の絞り込み・同じ値の絞り込み(NV-2・NV-8)の前の行の数(検索の欄の「N/M行」の M。NV-23)。
    pub(crate) unfiltered: usize,
    /// 表の上に検索の欄を出す(設定 `search_bar`。既定 true。NV-23)。
    pub(crate) search_bar: bool,
    /// 表の見せ方の設定(`[display]`。SR-20・SR-21)。ビューごとの上書きは `settings.display`(display.rs)。
    pub(crate) display: mdgrid::display::Display,
    /// 同じ値の行だけに絞る条件(NV-8 の `,` は1つに置き換え、NV-9 の頻度表は重ねる。全部を満たす行だけ)と、
    /// 同じ値の行の強調(`*`)。
    pub(crate) same: Vec<super::nav::Same>,
    pub(crate) same_mark: Option<(String, String)>,
    /// 検索・簡易の絞り込みの入力(下の行)。
    pub(crate) prompt: Option<Prompt>,
    /// 一時的な並べ替え(列, 降順か)(NV-3。.base は変えない)。
    pub(crate) sort: Option<(String, bool)>,
    /// 隠した列(id, 隠したときの位置, 固定した列だったか)。後に隠したものが後ろ(NV-4)。
    pub(crate) hidden: Vec<(String, usize, bool)>,
    /// 左に固定する列の数(ノートの列のほかに。NV-4)。
    pub(crate) frozen: usize,
    /// 印を付けた行(NV-5)と、`v`・Shift+矢印の範囲の錨。
    pub(crate) marked: HashSet<RowId>,
    pub(crate) anchor: Option<RowId>,
    /// 値を直したので、保存か移動の操作まで元の位置に留める行(NV-12)と、そのうち本来の位置と違う行(印)。
    pub(crate) stay: HashSet<RowId>,
    pub(crate) held: HashSet<RowId>,
    /// 詳細の表示(NV-6)。
    pub(crate) detail: Option<Detail>,
    /// CE-3 の候補の上限(設定 `candidates`。既定 20)。
    pub(crate) candidates: usize,
    /// BV-9 の読み直しの間隔(設定 `poll_ms`。既定 1000)。
    pub(crate) poll_ms: u64,
    /// 読むだけで開いた(`--readonly`。WB-15)。
    pub(crate) readonly: bool,
    /// 読むだけで表から外したキー(押したら「読むだけで開いている」と出す。WB-15)。
    pub(crate) locked: Vec<(Mode, &'static str)>,
    /// 見た目の状態の置き場と対象(SR-11・SR-12)。None なら読まず書かない。
    pub(crate) store: Option<super::startup::Store>,
    /// 読んだ見た目の状態(今のビュー)。列が表に現れたときに並びと隠す列を当てる(SR-12)。
    pub(crate) saved: mdgrid::config::ViewState,
    /// 今のビューに効いているビューの設定(NV-13〜NV-22)。
    pub(crate) settings: mdgrid::settings::Settings,
    /// ビューの設定の画面で編集している写し(反映するまで表には効かない。NV-13)。
    pub(crate) draft: Option<Draft>,
    /// 表の上の帯で選んだ項目(NV-22)。
    pub(crate) chip: usize,
    /// 日付の見せ方と打ち込みの形(設定 `date_format`。CE-22)と、カレンダーの週の始まり(CE-21)。
    pub(crate) date_format: DateFormat,
    pub(crate) week_start: WeekStart,
    /// mdgrid のビュー(BV-17〜BV-20): 定義・選んだもの・置き場と、書き出し・取り込みの入力。
    pub(crate) nv: super::native_views::Native,
    /// 新しいノート(CE-25〜CE-27。new_note.rs): 設定の決まりと、作っている途中の状態。
    pub(crate) note: super::new_note::NoteMaker,
    /// `--pick` で選んでいる(OUT-3)。Enter で `chosen` を決めて終わり、q と(印も範囲も無いときの)Esc で取りやめる。
    pub(crate) choosing: bool,
    /// `--pick` で選んだ行(表の並び)。None のまま終われば取りやめ。
    pub(crate) chosen: Option<Vec<RowId>>,
    /// その場の操作の一覧(SR-24。menu.rs)で選んでいる項目の添字。開いていなければ None。
    pub(crate) menu: Option<usize>,
    /// 列の値の頻度表(NV-9。freq.rs)。開いていなければ None。
    pub(crate) freq: Option<super::freq::Freq>,
    /// 画面のテーマ(SR-26・SR-27)。描き終えたバッファを ui/theme.rs が塗り替える。
    pub(crate) theme: Theme,
    /// 読み込みが終わったら選ぶノート(CLI-15。起動の引数に渡した `.md`)。終わりで一度だけ使う。
    pub select_after_load: Option<std::path::PathBuf>,
    /// BV-2: `.base` の上に `.obsidian/` が無く、根を `.base` のフォルダに推したときのその根。
    pub guessed_root: Option<std::path::PathBuf>,
    pub quit: bool,
}

impl App {
    pub fn new(src: Box<dyn Source>, color: ColorMode) -> App {
        let (today, now) = today_now(|k| std::env::var(k).ok());
        let mut app = App {
            src,
            changes: Changes::new(),
            mode: Mode::Table,
            base: None,
            rows: Vec::new(),
            groups: Vec::new(),
            folded: HashSet::new(),
            slots: Vec::new(),
            notes: Vec::new(),
            extra_cols: Vec::new(),
            summaries: Vec::new(),
            view_error: None,
            cols: Vec::new(),
            titles: HashMap::new(),
            label_prefix: String::new(),
            kinds: Vec::new(),
            col_kinds: HashMap::new(),
            widths: HashMap::new(),
            row: 0,
            col: 0,
            top: 0,
            left: 0,
            message: None,
            progress: Progress {
                loaded: 0,
                total: None,
                done: false,
            },
            cancelled: false,
            prefix: None,
            color,
            no_emoji: false,
            ambiguous_wide: false,
            today,
            now,
            size: (80, 24),
            input: None,
            pick: None,
            review: None,
            quit_after_save: false,
            keys: keymap::BINDINGS.to_vec(),
            help: None,
            palette: None,
            editor_request: None,
            clipboard: super::external::default_clipboard(),
            drag: None,
            regrid: false,
            search: None,
            search_at: None,
            filter: None,
            unfiltered: 0,
            search_bar: true,
            display: Default::default(),
            same: Vec::new(),
            same_mark: None,
            prompt: None,
            sort: None,
            hidden: Vec::new(),
            frozen: 0,
            marked: HashSet::new(),
            anchor: None,
            stay: HashSet::new(),
            held: HashSet::new(),
            detail: None,
            candidates: 20,
            poll_ms: 1000,
            readonly: false,
            locked: Vec::new(),
            store: None,
            saved: Default::default(),
            settings: Default::default(),
            draft: None,
            chip: 0,
            date_format: DateFormat::iso(),
            week_start: WeekStart::Sun,
            nv: Default::default(),
            note: Default::default(),
            choosing: false,
            chosen: None,
            menu: None,
            freq: None,
            theme: Theme::Default,
            select_after_load: None,
            guessed_root: None,
            quit: false,
        };
        app.refresh();
        app
    }

    /// 設定を当てる(CLI-3): CV-6 の `ambiguous_wide`、CE-3 の `candidates`、BV-9 の `poll_ms`、
    /// `color = false` で色なし(SR-10)、キーの割り当て直し(SR-13)、`search_bar`(NV-23)、`[display]`(SR-21)。
    /// キーの警告の文を返す。
    pub fn configure(&mut self, c: &Config) -> Vec<String> {
        self.ambiguous_wide = c.ambiguous_wide;
        self.search_bar = c.search_bar;
        self.display = c.display;
        self.candidates = c.candidates;
        self.poll_ms = c.poll_ms;
        self.date_format = c.date_format.clone();
        self.week_start = c.week_start;
        self.theme = c.theme;
        if !c.color {
            self.color = ColorMode::None;
        }
        self.keys = keymap::BINDINGS.to_vec();
        let warnings = keymap::rebind(&mut self.keys, &c.keys);
        self.scroll_into_view();
        warnings
    }

    /// 外での変化を見る間隔(BV-9。設定 `poll_ms`)。
    pub fn poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.poll_ms.max(1))
    }

    pub fn loaded(&self) -> bool {
        self.progress.done
    }

    /// 読み込みを少し進める(BV-16)。描画のたびに呼ぶ。
    pub fn load_step(&mut self, budget: usize) {
        if self.progress.done {
            return;
        }
        let before = self.progress.loaded;
        // SR-3: 読み込みの途中で先頭から動かしていなければ、組み直しても先頭に置く(最初に読んだノートを追うと、
        // 並べ替えで表の途中に流れる)。
        let at_top = self.row == 0 && self.top == 0;
        self.progress = self.src.load(budget);
        if self.progress.loaded != before || self.progress.done {
            self.refresh();
            if at_top && (self.row != 0 || self.top != 0) {
                self.row = 0;
                self.top = 0;
                self.scroll_into_view();
            }
        }
        if self.progress.done {
            self.select_pending();
            // BV-2: 推した根で0行なら、探した場所を一度出す(黙って空の表にしない)。
            if let Some(root) = self.guessed_root.take() {
                if self.rows.is_empty() && self.message.is_none() {
                    self.message = Some(Msg::VaultRootGuessed.fill(&[&root.display()]));
                }
            }
        }
    }

    /// CLI-15: 読み込みが終わったら(行が揃って並びが決まってから)、起動の引数に渡したノートの行を選ぶ。
    /// 途中で選ぶと並べ替えで行が動いて画面が跳ねるので、終わりで一度だけ。見えない・無いなら何もしない。
    fn select_pending(&mut self) {
        let Some(path) = self.select_after_load.take() else {
            return;
        };
        let real = path.canonicalize().unwrap_or(path);
        let id = RowId(real.to_string_lossy().into_owned());
        let at = self
            .rows
            .iter()
            .position(|r| *r == id)
            .and_then(|r| self.slots.iter().position(|s| *s == Slot::Row(r)));
        if let Some(i) = at {
            self.row = i;
            self.scroll_into_view();
        }
    }

    /// 外での変化を見る(BV-9)。ためた変更のある行には印を付け(WB-16)、選んだ行は保つ(BV-10)。
    /// 編集中は見ない(列や行が消えて入力ボックスが別のセルに移らないように)。入力を閉じたあとの poll で当てる。
    pub fn poll(&mut self) {
        if matches!(self.mode, Mode::Edit | Mode::ListPick) {
            return;
        }
        let changed = self.src.changed();
        if !changed.is_empty() {
            self.changes.note_external(&changed);
            // WB-17: 外で、ためた値と同じ値に直されたセルは外す。
            self.changes.drop_same(self.src.as_ref(), &changed);
            self.refresh();
            if self.review.is_some() {
                self.reopen_review_keeping_overlay();
            }
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        if self.size != (w, h) {
            self.size = (w, h);
            self.scroll_into_view();
            // SR-24: 一覧を開いたまま窓が入らなくなったら閉じ、開くときと同じ理由を出す(menu.rs)。
            self.close_menu_if_no_room();
            // NV-9: 頻度表も同じ(freq.rs)。
            self.close_freq_if_no_room();
        }
    }

    /// 表の中の行の数(画面の大きさから)。
    pub(crate) fn data_height(&self) -> usize {
        // NV-23・NV-16: 検索の欄と設定の帯が出ていれば、その分だけ表は下がる。タブを隠せば1行広い(SR-20)。
        view::table_height(self, self.size.1.saturating_sub(1) as usize)
    }

    /// 選んだセルが見えるように縦と横を流す(SR-3)。
    pub(crate) fn scroll_into_view(&mut self) {
        let h = self.data_height().max(1);
        if self.row < self.top {
            self.top = self.row;
        } else if self.row >= self.top + h {
            self.top = self.row + 1 - h;
        }
        self.top = self.top.min(self.slots.len().saturating_sub(1));
        // 固定した列(NV-4)は流さない。流すのはその右から。選んだ列と収まらない固定は、
        // 画面の上では一時的に減らす(`Layout.frozen`。選んだ列をいつも見せる)。
        self.frozen = self.frozen.min(self.cols.len());
        let lay = view::layout(self);
        let fz = lay.frozen;
        self.left = self.left.max(fz);
        if self.col >= fz && self.col < self.left {
            self.left = self.col;
        }
        while self.left < self.col && !lay.fits(self.left, self.col) {
            self.left += 1;
        }
        self.left = self
            .left
            .min(self.cols.len().saturating_sub(1))
            .max(fz.min(self.cols.len().saturating_sub(1)));
    }

    /// 選んだセル(ノートの行と列)。見出しの行では None。
    pub(crate) fn selected(&self) -> Option<(RowId, String)> {
        Some((self.cur_row()?, self.cols.get(self.col)?.clone()))
    }

    /// キーを受け取り、キーの表で意図に直して `apply` に渡す(SR-4)。
    pub fn key(&mut self, ev: KeyEvent) {
        if ev.kind == KeyEventKind::Release {
            return;
        }
        let Some(name) = keymap::key_name(&ev) else {
            return;
        };
        if matches!(
            self.mode,
            Mode::Edit
                | Mode::Palette
                | Mode::Search
                | Mode::Filter
                | Mode::SettingsText
                | Mode::ListPick
        ) {
            // 表にあるキーは動作に、無い文字は入力に(全角のまま入れる。SR-17 の読み替えは表のモードだけ)。
            let found = keymap::lookup(&self.keys, self.mode, &name).or_else(|| {
                keymap::unshifted(&name).and_then(|n| keymap::lookup(&self.keys, self.mode, &n))
            });
            if let Some(a) = found {
                self.apply(a);
            } else if let KeyCode::Char(c) = ev.code {
                if !ev
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                {
                    match self.mode {
                        Mode::Palette => self.palette_insert(c),
                        Mode::Search | Mode::Filter => self.prompt_insert(c),
                        Mode::SettingsText => self.settings_insert(c),
                        Mode::ListPick => self.list_pick_insert(c),
                        // CE-25: 作る場所を選ぶ欄では文字を受けない(自由入力に切り替えない)。
                        Mode::Edit if self.note_choosing() => {}
                        _ => self.insert(c),
                    }
                }
            }
            return;
        }
        if let Some(p) = self.prefix.take() {
            let seq = format!("{p} {name}");
            if self.dispatch(seq) {
                return;
            }
            // SR-25: 前置きのあとの Esc は前置きを取り消すだけ(選択の解除や一覧を閉じることはしない)。
            if name == "Esc" {
                return;
            }
            // 前置きのあとの表に無いキーは、前置きを捨ててそのキーだけで引き直す。
        }
        let plain = keymap::unshifted(&name);
        if !self.dispatch(name) {
            // Shift+←→ を割り当てていないモードでは ←→ と同じ(CE-23 より前の扱い)。
            if let Some(n) = plain {
                self.dispatch(n);
            }
        }
    }

    /// キーの名前を表で引いて振り分ける。前置きか動作が見つかれば true。
    fn dispatch(&mut self, name: String) -> bool {
        if keymap::is_prefix(&self.keys, self.mode, &name) {
            self.prefix = Some(name);
            return true;
        }
        match keymap::lookup(&self.keys, self.mode, &name) {
            Some(a) => {
                self.apply(a);
                true
            }
            None if self
                .locked
                .iter()
                .any(|(m, k)| *m == self.mode && *k == name) =>
            {
                self.message = Some(super::startup::READONLY.into());
                true
            }
            // SR-24: 一覧の上では、一覧のキーでなければ表のモードのキーとして読む(menu.rs)。
            None if self.mode == Mode::Menu => self.menu_key(&name),
            None => false,
        }
    }

    /// 意図を受けて状態を変える(振り分け)。
    pub fn apply(&mut self, action: Action) {
        self.message = None;
        match self.mode {
            Mode::Help => return self.help_action(action),
            Mode::Palette => return self.palette_action(action),
            Mode::Search | Mode::Filter => self.prompt_action(action),
            Mode::Detail => self.detail_action(action),
            Mode::Edit => {
                // CE-21: 日付の入力のカレンダーの操作(calendar.rs)が先。
                // CE-23・CE-24: 月・年のキーは、カレンダーが出ていなければ ←→・↑↓ と同じ。
                let action = self.calendar_fallback(action);
                // CE-25・CE-26: 新しいノートの入力の確定と取りやめ(new_note.rs)が先。
                if !self.new_note_input_action(action)
                    && !self.detail_input_action(action)
                    && !self.calendar_action(action)
                {
                    self.input_action(action)
                }
                self.calendar_follow();
            }
            Mode::Confirm | Mode::Quit => self.review_action(action),
            Mode::Settings | Mode::SettingsText => self.settings_action(action),
            Mode::Chips => self.chips_action(action),
            Mode::ListPick => {
                if !self.new_note_pick_action(action) {
                    self.list_pick_action(action)
                }
            }
            Mode::Menu => self.menu_action(action),
            Mode::Freq => self.freq_action(action),
            Mode::Table => self.table_action(action),
        }
        if std::mem::take(&mut self.regrid) {
            self.refresh();
        }
        self.scroll_into_view();
        // SR-12: 終わるときに見た目の状態を書く。
        if self.quit {
            self.persist_state();
        }
    }

    /// 表のモードの動作。行を移ったら、直した行を留めるのをやめて組み立て直す(NV-12)。
    fn table_action(&mut self, action: Action) {
        let before = self.row;
        self.table_action_inner(action);
        if self.row != before {
            self.moved();
        }
    }

    /// 移動の操作のあと(NV-12): 留めていた行を本来の位置へ。選んだ行は鍵で保つ。
    pub(crate) fn moved(&mut self) {
        if !self.stay.is_empty() {
            self.stay.clear();
            self.regrid = true;
        }
    }

    fn table_action_inner(&mut self, action: Action) {
        // OUT-3: `--pick` の Enter(見出しの行では開閉のまま)と Esc の取りやめ(nav.rs)。
        if self.choosing && self.choose_action(action) {
            return;
        }
        // WB-15: 読むだけでは、編集・空にする・取り消し・保存をしない(見出しの行の開閉は効く)。
        if self.readonly
            && (matches!(
                action,
                Action::Clear | Action::Save | Action::Undo | Action::Redo
            ) || (action == Action::Edit && self.cur_head().is_none()))
        {
            self.message = Some(super::startup::READONLY.into());
            return;
        }
        if self.nav_action(action) || self.column_action(action) {
            return;
        }
        let last_row = self.slots.len().saturating_sub(1);
        let last_col = self.cols.len().saturating_sub(1);
        let page = self.data_height().max(1);
        match action {
            Action::Up => self.row = self.row.saturating_sub(1),
            Action::Down => self.row = (self.row + 1).min(last_row),
            Action::Left => self.col = self.col.saturating_sub(1),
            Action::Right => self.col = (self.col + 1).min(last_col),
            Action::FirstColumn => self.col = 0,
            Action::LastColumn => self.col = last_col,
            Action::PageUp => self.row = self.row.saturating_sub(page),
            Action::PageDown => self.row = (self.row + page).min(last_row),
            Action::Top => self.row = 0,
            Action::Bottom => self.row = last_row,
            Action::NextCell => {
                if self.col < last_col {
                    self.col += 1;
                } else if self.row < last_row {
                    self.col = 0;
                    self.row += 1;
                }
            }
            Action::PrevCell => {
                if self.col > 0 {
                    self.col -= 1;
                } else if self.row > 0 {
                    self.col = last_col;
                    self.row -= 1;
                }
            }
            // SR-2: 見出しの行の Enter は開閉。
            Action::Edit => match self.cur_head() {
                Some(g) => self.toggle_group(g),
                None => self.open_edit(),
            },
            Action::Save => {
                if !self.drop_same_pending() {
                    self.message = Some(Msg::NothingToSave.text().into());
                } else {
                    self.open_review();
                }
            }
            Action::Clear => self.clear(),
            Action::Undo => {
                let rows = self.changes.rows();
                if self.changes.undo() {
                    self.keep_in_place(rows);
                } else {
                    self.message = Some(Msg::NothingToUndo.text().into());
                }
            }
            Action::Redo => {
                let rows = self.changes.rows();
                if self.changes.redo() {
                    self.keep_in_place(rows);
                } else {
                    self.message = Some(Msg::NothingToRedo.text().into());
                }
            }
            Action::Quit => {
                if self.changes.count() == 0 {
                    self.quit = true;
                } else {
                    // WB-11: 保存する・捨てる・戻る を確かめる。
                    self.set_mode(Mode::Quit);
                }
            }
            Action::CancelLoad => {
                if self.progress.done {
                    self.message = Some(Msg::LoadAlreadyDone.text().into());
                } else {
                    self.src.cancel();
                    self.progress = self.src.load(0);
                    self.progress.done = true;
                    self.cancelled = true;
                    self.refresh();
                    self.select_pending();
                    self.message = Some(Msg::LoadStopped.fill(&[&self.progress.loaded]));
                }
            }
            // ---- 表の形(grid.rs。BV-13・SR-3・NV-4) ----
            Action::PrevView => self.switch_view(false),
            Action::NextView => self.switch_view(true),
            Action::Narrower => self.resize_column(-1),
            Action::Wider => self.resize_column(1),
            // ---- ヘルプとパレット(help.rs)・エディタとコピー(external.rs) ----
            Action::Help => self.open_help(),
            Action::Palette => self.open_palette(),
            Action::OpenEditor => self.request_editor(),
            Action::Copy => self.copy_cell(),
            Action::CopyRow => self.copy_row(),
            Action::Detail => self.open_detail(),
            // ---- ビューの設定と帯(settings.rs。NV-13・NV-22) ----
            Action::ViewSettings => self.open_settings(),
            Action::FocusChips => self.focus_chips(),
            // ---- mdgrid のビューの書き出しと取り込み(native_views.rs。BV-19) ----
            Action::ExportBase => self.start_export(),
            Action::ImportBase => self.start_import(),
            // ---- キーの名前の変更と削除(native_io.rs。CE-29) ----
            Action::RenameKey => self.start_rename_key(),
            Action::DeleteKey => self.start_delete_key(),
            // ---- 新しいノート(new_note.rs。CE-25) ----
            Action::NewNote => self.start_new_note(),
            // ---- その場の操作の一覧(menu.rs。SR-24) ----
            Action::Menu => self.open_menu(),
            // ---- 列の値の頻度表(freq.rs。NV-9) ----
            Action::Freq => self.open_freq(),
            _ => {}
        }
    }

    /// 取り消し・やり直しで値が変わった行を、元の位置に留める(NV-12)。`before` は前にためた変更のあった行。
    fn keep_in_place(&mut self, before: Vec<RowId>) {
        self.stay.extend(before);
        self.stay.extend(self.changes.rows());
        self.regrid = true;
    }

    pub(crate) fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.prefix = None;
    }

    /// マウスのクリック(端末の桁と行。0 始まり)。キーと同じ意図に直して `apply` に渡す。
    /// 編集中に入力の外をクリックしたら確定を試み、確定できなければそのクリックを捨てる(CE-1)。
    /// 表では、選んでいない行のセルは選ぶだけ、選んだ行のセルは編集(見出しの行なら開閉)を始める(SR-6・SR-2)。
    /// 選んだ行のファイル名(行の名前の欄・`file.name` のセル)は `e` と同じくエディタで開く(SR-19)。
    /// タブの帯はビューの切り替え(BV-13)、検索の欄は簡易の絞り込みの入力(NV-23)、
    /// 列の見出しの境界はドラッグの始まり(SR-3)。
    pub fn click(&mut self, x: u16, y: u16) {
        match self.mode {
            // CE-25: 新しいノートの入力では、カレンダーの日のクリックだけを受ける(外のクリックで作らない)。
            Mode::Edit if self.note.flow.is_some() => {
                self.calendar_click(x, y);
                return;
            }
            Mode::Edit => {
                // 詳細の表示からの編集は、詳細の入力の位置で判定する(NV-6)。
                let inside = if self.detail.is_some() {
                    super::detail::in_detail_input(self, x, y)
                } else {
                    view::in_input(self, x, y)
                };
                if inside {
                    return;
                }
                // CE-3: 候補のリストの項目のクリックはその候補を選んで確定。
                if self.list_click(x, y) {
                    return;
                }
                // CE-20: カレンダーの日のクリックはその日を選ぶ(窓の中のほかの所は何もしない)。
                if self.calendar_click(x, y) {
                    return;
                }
                self.apply(Action::Commit);
                if self.mode != Mode::Table {
                    return;
                }
            }
            Mode::Table => {}
            // NV-23: 検索の欄で打っているとき、欄のクリックはそのまま、欄の外のクリックは欄を抜けて
            // (絞り込みは残す)表のクリックとして扱う。
            Mode::Filter if self.shows(mdgrid::display::Item::SearchBar) => {
                if view::bar_at(self, x, y) {
                    return;
                }
                self.apply(Action::Commit);
            }
            // NV-18: 設定の画面の項目とボタンのクリック。
            Mode::Settings => return self.settings_click(x, y),
            // SR-24: 一覧の項目のクリックで実行して閉じ、窓の外のクリックで閉じるだけ。
            Mode::Menu => return self.menu_click(x, y),
            // NV-9: 頻度表の項目のクリックで絞って閉じ、窓の外のクリックで閉じるだけ。
            Mode::Freq => return self.freq_click(x, y),
            Mode::Chips => {
                if let Some(i) = view::chip_at(self, x, y) {
                    self.message = None;
                    self.remove_chip(i);
                    self.refresh_if_needed();
                }
                return;
            }
            _ => return,
        }
        // CE-25: ヘッダーの「+ 新規」で新しいノートの名前の欄を開く。
        if super::new_note::button_at(self, x, y) {
            self.apply(Action::NewNote);
            return;
        }
        // NV-23: 検索の欄のクリックで欄に入る(`\` と同じ)。
        if view::bar_at(self, x, y) {
            self.apply(Action::QuickFilter);
            return;
        }
        // NV-22: 帯の項目のクリックでその条件を外す。
        if let Some(i) = view::chip_at(self, x, y) {
            self.message = None;
            self.remove_chip(i);
            self.refresh_if_needed();
            return;
        }
        if let Some(i) = view::tab_at(self, x, y) {
            self.message = None;
            self.select_view(i);
            return;
        }
        if let Some(d) = view::boundary_at(self, x, y) {
            self.drag = Some(d);
            return;
        }
        // NV-3: 列の見出しのクリックで一時的な並べ替え。
        if let Some(j) = view::header_at(self, x, y) {
            self.message = None;
            self.col = j;
            self.cycle_sort();
            self.refresh_if_needed();
            return;
        }
        if let Some((i, j)) = view::hit(self, x, y) {
            let same_row = i == self.row;
            self.row = i;
            self.col = j;
            self.message = None;
            if !same_row {
                self.moved();
                self.refresh_if_needed();
            }
            self.scroll_into_view();
            if same_row {
                // SR-19: 選んだ行のファイル名(行の名前の欄・`file.name` のセル)は `e` と同じ。
                let in_name = {
                    let lay = view::layout(self);
                    let x0 = lay.label_x();
                    (x0..x0 + lay.label_w).contains(&(x as usize))
                };
                let file_name = self.cols.get(j).is_some_and(|c| c == "file.name");
                if (in_name || file_name) && self.cur_row().is_some() {
                    self.apply(Action::OpenEditor);
                } else {
                    self.apply(Action::Edit);
                }
            }
        }
    }

    /// 組み立て直しを頼まれていれば組み立て直す。
    pub(crate) fn refresh_if_needed(&mut self) {
        if std::mem::take(&mut self.regrid) {
            self.refresh();
        }
        self.scroll_into_view();
    }

    /// ホイール(SR-6)。表では上下、保存の確認ではファイルの選択。
    pub fn wheel(&mut self, down: bool) {
        let a = match (self.mode, down) {
            (Mode::Table, true) => Action::Down,
            (Mode::Table, false) => Action::Up,
            (Mode::Confirm, true) => Action::NextFile,
            (Mode::Confirm, false) => Action::PrevFile,
            (Mode::Help, true) => Action::Down,
            (Mode::Help, false) => Action::Up,
            (Mode::Detail, true) => Action::Down,
            (Mode::Detail, false) => Action::Up,
            (Mode::Settings, true) => Action::Down,
            (Mode::Settings, false) => Action::Up,
            (Mode::Menu, true) => Action::Down,
            (Mode::Menu, false) => Action::Up,
            (Mode::Freq, true) => Action::Down,
            (Mode::Freq, false) => Action::Up,
            _ => return,
        };
        self.apply(a);
    }
}
