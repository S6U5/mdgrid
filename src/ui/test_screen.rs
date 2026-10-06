use super::keymap::Action;
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Color;
use ratatui::Terminal;
use std::path::{Path, PathBuf};

// ---- 材料 ----

/// 一時フォルダ。ノートは `<一時>/notes` に置く(ヘッダーのフォルダの名前を固定にする)。
pub(super) struct Tmp(pub(super) PathBuf);

impl Tmp {
    pub(super) fn new(name: &str) -> Tmp {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("mdgrid-ui-{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(dir.join("notes")).unwrap();
        Tmp(dir)
    }

    pub(super) fn notes(&self) -> PathBuf {
        self.0.join("notes")
    }

    pub(super) fn write(&self, name: &str, text: &str) {
        std::fs::write(self.notes().join(name), text).unwrap();
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn app_of(tmp: &Tmp, color: ColorMode) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), color);
    app.resize(80, 24);
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

pub(super) fn make(name: &str, notes: &[(&str, &str)]) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    let app = app_of(&tmp, ColorMode::None);
    (tmp, app)
}

pub(super) fn buffer(app: &App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|f| draw(f, app)).unwrap();
    term.backend().buffer().clone()
}

/// バッファを行ごとの文字列にする(幅2の文字の後ろの空きの桁は飛ばす。行末の空白は除く)。
pub(super) fn text(buf: &Buffer) -> String {
    let a = buf.area;
    let mut out = String::new();
    for y in 0..a.height {
        let mut line = String::new();
        let mut x = 0;
        while x < a.width {
            let s = buf[(x, y)].symbol();
            line.push_str(s);
            x += width::width(s).max(1) as u16;
        }
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

pub(super) fn screen(app: &App) -> String {
    text(&buffer(app))
}

/// `tests/golden/<name>.txt` と比べる。`UPDATE_GOLDEN=1` で書く。
pub(super) fn golden(name: &str, got: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "ゴールデンが無い: {} (UPDATE_GOLDEN=1 で作る)",
            path.display()
        )
    });
    assert_eq!(got, want, "ゴールデン {name} と違う");
}

pub(super) fn press(app: &mut App, code: KeyCode) {
    app.key(KeyEvent::new(code, KeyModifiers::NONE));
}

pub(super) fn ch(app: &mut App, c: char) {
    press(app, KeyCode::Char(c));
}

pub(super) fn col_named(app: &mut App, name: &str) {
    app.col = app.cols.iter().position(|c| c == name).unwrap();
    app.scroll_into_view();
}

// ---- ゴールデン ----

#[test]
fn test_sr_1_five_bands() {
    // [SR-1] ヘッダー(フォルダの名前と行数)・タブ・表・下の帯(モード・位置・未保存・キー)・メッセージ行。
    let (_t, app) = make(
        "sr1",
        &[
            (
                "a.md",
                "---\ntitle: 会議のメモ\nstatus: 進行中\n---\n本文\n",
            ),
            ("b.md", "---\ntitle: 買い物\nstatus: 完了\n---\n"),
            ("c.md", "---\ntitle: 読書\ntags: [本, 秋]\n---\n"),
        ],
    );
    let s = screen(&app);
    assert!(s.lines().next().unwrap().contains("notes  3行"));
    assert!(s.contains("未保存 0"));
    golden("sr_1", &s);
}

#[test]
fn test_cv_1_null_empty_missing() {
    // [CV-1] null は `∅`、空の文字列は `""`、キーが無いときは空欄。
    let (_t, app) = make(
        "cv1",
        &[
            ("a-null.md", "---\ntitle: a\ndue:\n---\n"),
            ("b-empty.md", "---\ntitle: b\ndue: \"\"\n---\n"),
            ("c-missing.md", "---\ntitle: c\n---\n"),
        ],
    );
    let s = screen(&app);
    assert!(s.contains('∅') && s.contains("\"\""));
    golden("cv_1", &s);
}

#[test]
fn test_cv_3_newline_value() {
    // [CV-3] 改行を含む値は1行目と `…⏎`。
    let (_t, app) = make(
        "cv3",
        &[("a.md", "---\ntitle: a\nmemo: |\n  一行目\n  二行目\n---\n")],
    );
    let s = screen(&app);
    assert!(s.contains("一行目…⏎"), "{s}");
    golden("cv_3", &s);
}

#[test]
fn test_cv_4_numbers_right_aligned() {
    // [CV-4] 数の列は右寄せ、チェックボックスの列は中央寄せ。
    let (_t, app) = make(
        "cv4",
        &[
            ("a.md", "---\ncount: 5\ndone: true\n---\n"),
            ("b.md", "---\ncount: 1200\ndone: false\n---\n"),
            ("c.md", "---\ncount: 3.5\n---\n"),
        ],
    );
    let s = screen(&app);
    assert!(s.contains("     5 "), "{s}");
    golden("cv_4", &s);
}

#[test]
fn test_cv_5_long_value_capped() {
    // [CV-5] 列の幅の上限は画面の幅の3割(79 桁の3割 = 23)、はみ出す値は `…`。
    let long = "長い説明の文章がここに続いていて画面の三割を超えてしまう";
    let (_t, app) = make(
        "cv5",
        &[("a.md", &format!("---\ntitle: a\nsummary: {long}\n---\n"))],
    );
    let lay = view::layout(&app);
    assert_eq!(lay.widths[1], 23);
    let s = screen(&app);
    assert!(s.contains('…'));
    golden("cv_5", &s);
}

// ---- 不変条件 ----

#[test]
fn test_sr_9_lines_fit_and_no_escape() {
    // [SR-9] 全角・絵文字・結合文字・制御文字の値でも、全行が幅以下で、ESC を出さない。
    let (_t, mut app) = make(
        "sr9",
        &[
            (
                "a.md",
                "---\ntitle: 全角の文字ＡＢＣ\nemoji: 家族👨‍👩‍👧と👍🏽\n---\n",
            ),
            (
                "b.md",
                "---\ntitle: cafe\u{301} e\u{301}\nemoji: ｶﾞｷﾞｸﾞ\n---\n",
            ),
            (
                "c.md",
                "---\ntitle: x\u{1b}[31mred\u{7}\nemoji: tab\there\n---\n",
            ),
            (
                "d-日本語の長いファイルの名前がここに続いていて.md",
                "---\ntitle: あ\n---\n",
            ),
        ],
    );
    // 制御文字は見える文字(ESC → `␛`)に置き換わる。
    let s = screen(&app);
    assert!(s.contains("x␛[31mred␇"), "{s}");
    for step in 0..6 {
        for w in [80u16, 41, 20, 9, 5, 4, 3, 2] {
            app.resize(w, 24);
            let lines = view::render(&app, (w - 1) as usize, 23);
            for l in &lines {
                let lw: usize = l.spans.iter().map(|s| width::width(&s.content)).sum();
                assert!(lw <= (w - 1) as usize, "幅 {w} で {lw}: {l:?}");
            }
            let mut term = Terminal::new(TestBackend::new(w, 24)).unwrap();
            term.draw(|f| draw(f, &app)).unwrap();
            let buf = term.backend().buffer();
            for y in 0..24 {
                for x in 0..w {
                    let s = buf[(x, y)].symbol();
                    assert!(!s.contains(char::is_control), "制御文字 {s:?}");
                    assert!(
                        !s.chars().any(|c| matches!(
                            c,
                            '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}'
                                | '\u{2066}'..='\u{2069}' | '\u{2028}' | '\u{2029}'
                                | '\u{feff}'
                        )),
                        "双方向の制御など {s:?}"
                    );
                    if x == w - 1 || y == 23 {
                        assert_eq!(s, " ", "最下行と右端は空ける");
                    }
                }
            }
        }
        app.resize(80, 24);
        if step % 2 == 0 {
            ch(&mut app, 'j');
        } else {
            ch(&mut app, 'l');
        }
    }
}

// ---- キー列 ----

#[test]
fn test_sr_13_j_and_down_same() {
    // [SR-13] `j` と ↓ で同じ動き。[SR-17] 全角の ｊ も同じ。
    let notes: Vec<(String, String)> = (0..5)
        .map(|i| (format!("n{i}.md"), format!("---\nn: {i}\n---\n")))
        .collect();
    let refs: Vec<(&str, &str)> = notes
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let (_t, mut a) = make("sr13", &refs);
    ch(&mut a, 'j');
    assert_eq!(a.row, 1);
    press(&mut a, KeyCode::Down);
    assert_eq!(a.row, 2);
    ch(&mut a, 'ｊ');
    assert_eq!(a.row, 3);
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    assert_eq!(a.row, 0);
    ch(&mut a, 'G');
    assert_eq!(a.row, 4);
    ch(&mut a, 'ｋ');
    assert_eq!(a.row, 3);
    // [SR-16] 前置き `g` のあとの表に無いキーは、前置きを捨ててそのキーだけで引く。
    ch(&mut a, 'g');
    ch(&mut a, 'k');
    assert_eq!(a.row, 2);
    assert!(a.prefix.is_none());
    ch(&mut a, 'g');
    ch(&mut a, 'g');
    assert_eq!(a.row, 0);
}

#[test]
fn test_sr_17_fullwidth_and_marks() {
    // [SR-17] 全角の ｊ は j、`、` は `,`、`・` は `/` として引く(`,`・`/` の動作はタスク 10)。
    let (_t, mut a) = make(
        "sr17",
        &[("a.md", "---\nx: 1\n---\n"), ("b.md", "---\nx: 2\n---\n")],
    );
    ch(&mut a, 'ｊ');
    assert_eq!(a.row, 1);
    let ev = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
    assert_eq!(keymap::key_name(&ev('、')).as_deref(), Some(","));
    assert_eq!(keymap::key_name(&ev('・')).as_deref(), Some("/"));
}

#[test]
fn test_sr_18_backspace_and_undo() {
    // [SR-18] Backspace でセルを空にする(ためる変更)→「未保存 1」。[WB-10] `u` で戻して「未保存 0」、`U` でやり直し。
    let (_t, mut a) = make(
        "sr18",
        &[
            ("a.md", "---\ntitle: a\nstatus: 進行中\n---\n"),
            ("b.md", "---\ntitle: b\n---\n"),
        ],
    );
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("未保存 1"));
    // [SR-1] 狭い端末でも「未保存 N」は切れない。
    let narrow = view::render(&a, 12, 23);
    let foot: String = narrow[21]
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert!(foot.contains("未保存 1"), "{foot}");
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));
    ch(&mut a, 'U');
    assert_eq!(a.changes.count(), 1);
    a.key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
    assert_eq!(a.changes.count(), 0);
    a.key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
    assert_eq!(a.changes.count(), 1);
    // [SR-18] 未保存があると `q` では終わらない(終了の確認 WB-11)。
    ch(&mut a, 'q');
    assert!(!a.quit);
    press(&mut a, KeyCode::Esc);
    ch(&mut a, 'u');
    // [SR-18] キーの無いセル(b の status)で Backspace → 何も変わらない。
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Delete);
    assert_eq!(a.changes.count(), 0);
    ch(&mut a, 'q');
    assert!(a.quit);
}

#[test]
fn test_sr_18_read_only_cell_reason() {
    // [SR-18] 読むだけのセルでは理由を出して何もしない。[SR-15] 選ぶとメッセージ行に理由。
    // 材料は CE-8 の読むだけの形(2行にまたがるフローのリスト)。1行の `[a, b]` は CE-16 で書ける。
    let (_t, mut a) = make(
        "sr18ro",
        &[
            ("a.md", "---\ntags: [a,\n  b]\n---\n"),
            ("b.md", "---\ntags: [a, b]\n---\n"),
        ],
    );
    col_named(&mut a, "tags");
    let s = screen(&a);
    assert!(s.lines().nth(22).unwrap().contains("読むだけ"), "{s}");
    // [SR-15] 読むだけのセルは薄い表示に加えて `#` の印(NO_COLOR でも見分けられる)。
    // 複数行の値の中身の見せ方は CV の決まりなので問わず、セルの先頭の `#` だけを見る。
    let cell_text = |line: &str, name: &str| line.split(name).nth(1).unwrap().trim().to_string();
    let ro = cell_text(s.lines().nth(4).unwrap(), "a ");
    assert!(ro.starts_with('#'), "{s}");
    // [CE-16] 書ける形のリスト(1行の `[a, b]`)には `#` の印が無い。
    let rw = cell_text(s.lines().nth(5).unwrap(), "b ");
    assert_eq!(rw, "[a, b]", "{s}");
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 0);
    assert!(a.message.as_deref().unwrap().contains("空にできない"));
}

#[test]
fn test_sr_3_selected_column_stays_visible() {
    // [SR-3] 列が画面に収まらない表で右へ移り続けても、選んだ列が見える。
    let mut fm = String::from("---\n");
    for i in 0..20 {
        fm.push_str(&format!("prop{i:02}: value-of-property-{i:02}\n"));
    }
    fm.push_str("---\n");
    let (_t, mut a) = make("sr3", &[("a.md", &fm)]);
    for i in 0..20 {
        let s = screen(&a);
        let head = s.lines().nth(3).unwrap();
        assert!(head.contains(&format!("prop{i:02}")), "{i}: {head}");
        let row = s.lines().nth(4).unwrap();
        assert!(
            row.contains(&format!("value-of-property-{i:02}")),
            "{i}: {row}"
        );
        ch(&mut a, 'l');
    }
    assert!(a.left > 0);
    press(&mut a, KeyCode::Home);
    assert_eq!(a.left, 0);
    press(&mut a, KeyCode::End);
    assert!(screen(&a).contains("prop19"));
}

#[test]
fn test_sr_10_no_color() {
    // [SR-10] NO_COLOR のとき色を出さない(ためる変更のセルも)。TERM=dumb も色なし。
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert_eq!(
        ColorMode::detect(env(&[("NO_COLOR", "1")])),
        ColorMode::None
    );
    assert_eq!(ColorMode::detect(env(&[("TERM", "dumb")])), ColorMode::None);
    assert_eq!(
        ColorMode::detect(env(&[("COLORTERM", "truecolor")])),
        ColorMode::Rgb
    );
    // [SR-15] COLORTERM が無ければ 256 色。
    assert_eq!(
        ColorMode::detect(env(&[("TERM", "xterm")])),
        ColorMode::Indexed
    );

    let (_t, mut a) = make("sr10", &[("a.md", "---\nstatus: x\n---\n")]);
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.changes.count(), 1);
    let buf = buffer(&a);
    for c in buf.content() {
        assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset), "{c:?}");
    }
}

#[test]
fn test_sr_15_pending_mark_and_256_colors() {
    // [SR-15] ためる変更のセルは色に加えて `*`。COLORTERM の無い端末では 256 色。
    let tmp = Tmp::new("sr15");
    tmp.write("a.md", "---\nstatus: x\n---\n");
    tmp.write("b.md", "---\nstatus: y\n---\n");
    let mut a = app_of(&tmp, ColorMode::Indexed);
    press(&mut a, KeyCode::Backspace);
    ch(&mut a, 'j');
    let s = screen(&a);
    assert!(s.lines().nth(4).unwrap().contains("*∅"), "{s}");
    let buf = buffer(&a);
    assert!(buf
        .content()
        .iter()
        .any(|c| matches!(c.fg, Color::Indexed(_))));
    assert!(!buf.content().iter().any(|c| matches!(c.fg, Color::Rgb(..))));
}

#[test]
fn test_bv_10_selection_kept_after_reload() {
    // [BV-10] 外でノートが消えても、選んだノートの行を選んだまま。
    let (t, mut a) = make(
        "bv10",
        &[
            ("a.md", "---\nx: 1\n---\n"),
            ("b.md", "---\nx: 2\n---\n"),
            ("c.md", "---\nx: 3\n---\n"),
        ],
    );
    ch(&mut a, 'j');
    let chosen = a.rows[1].clone();
    std::fs::remove_file(t.notes().join("a.md")).unwrap();
    for _ in 0..200 {
        a.poll();
        if a.rows.len() == 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(a.rows.len(), 2);
    assert_eq!(a.rows[a.row], chosen);
}

#[test]
fn test_bv_16_cancel_load() {
    // [BV-16] 読み込みの途中で Ctrl+G → 止まり、読んだ分の表が残る。
    let tmp = Tmp::new("bv16");
    for i in 0..50 {
        tmp.write(&format!("n{i:02}.md"), "---\nx: 1\n---\n");
    }
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.load_step(10);
    // [SR-4] 中止のキーの表示はキーの表から引く。
    let top = screen(&a).lines().next().unwrap().to_string();
    assert!(top.contains("^G で中止"), "{top}");
    // 前置き `g` のあとの Ctrl+G も中止として効く。
    ch(&mut a, 'g');
    a.key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::CONTROL));
    assert!(a.loaded() && a.cancelled);
    let n = a.rows.len();
    assert!(n < 50);
    a.load_step(100);
    assert_eq!(a.rows.len(), n);
    let _ = Action::CancelLoad;
}

// ---- 編集(タスク 11)と保存・終了の確認(タスク 12) ----

pub(super) fn ctrl(app: &mut App, c: char) {
    app.key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
}

pub(super) fn typing(app: &mut App, s: &str) {
    for c in s.chars() {
        ch(app, c);
    }
}

pub(super) fn read(tmp: &Tmp, name: &str) -> String {
    std::fs::read_to_string(tmp.notes().join(name)).unwrap()
}

/// 全行が幅以下(SR-9)。いくつかの幅で描いて確かめる。
pub(super) fn assert_fits(app: &mut App) {
    for w in [80u16, 41, 20, 9, 5, 3] {
        app.resize(w, 24);
        for l in view::render(app, (w - 1) as usize, 23) {
            let lw: usize = l.spans.iter().map(|s| width::width(&s.content)).sum();
            assert!(lw <= (w - 1) as usize, "幅 {w} で {lw}: {l:?}");
        }
        let mut term = Terminal::new(TestBackend::new(w, 24)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
    }
    app.resize(80, 24);
}

const TWO: &[(&str, &str)] = &[
    ("a.md", "---\ntitle: a\nstatus: todo\n---\n本文 a\n"),
    ("b.md", "---\ntitle: b\nstatus: doing\n---\n本文 b\n"),
];

#[test]
fn test_ce_1_enter_type_enter_pending() {
    // [CE-1] Enter → その場の入力ボックス → 打って Enter でためる変更。[WB-9] ファイルは変わらず「未保存 1」。
    let (t, mut a) = make("ce1", TWO);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Edit);
    // [SR-17] 端末のカーソルは入力の位置(値の末尾)に置く。
    let (cx, cy) = view::cursor(&a).unwrap();
    assert_eq!(cy, 4);
    let row = screen(&a).lines().nth(4).unwrap().to_string();
    assert!(row.contains("todo"), "{row}");
    for _ in 0..4 {
        press(&mut a, KeyCode::Backspace);
    }
    // 全角のまま入れる(表のモードの読み替えは効かない)。
    typing(&mut a, "済みｊ");
    let (cx2, _) = view::cursor(&a).unwrap();
    assert_eq!(a.input.as_ref().unwrap().text, "済みｊ", "{}", screen(&a));
    assert_eq!(cx2 as usize, cx as usize - 4 + 6);
    assert_fits(&mut a);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 1);
    assert!(view::cursor(&a).is_none());
    let s = screen(&a);
    assert!(s.contains("未保存 1") && s.contains("*済みｊ"), "{s}");
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    // 開いて何も変えずに Enter → 何もためない(数や真偽を文字列にしない)。
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_ce_1_esc_cancels() {
    // [CE-1] Esc で取り消し: 何もためない。
    let (t, mut a) = make("ce1esc", TWO);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "xyz");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));
    assert_eq!(read(&t, "a.md"), TWO[0].1);
}

#[test]
fn test_ce_1_click_outside_commits() {
    // [CE-1] 入力の外をクリック → 確定してためる変更。入力の中のクリックは何もしない。
    let (_t, mut a) = make("ce1click", TWO);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "!");
    let (cx, cy) = view::cursor(&a).unwrap();
    a.click(cx - 1, cy);
    assert_eq!(a.mode, keymap::Mode::Edit);
    // [SR-6] 別の行のセルのクリック → 確定して、その行を選ぶだけ(編集は始めない)。
    a.click(cx - 1, cy + 1);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(a.row, 1);
    // [SR-6] 選んだ行のセルのクリック → 編集を始める。
    a.click(cx - 1, cy + 1);
    assert_eq!(a.mode, keymap::Mode::Edit);
}

#[test]
fn test_ce_11_revert_and_tab() {
    // [CE-11] 編集中の Ctrl+R で編集前の値に戻す。
    let (_t, mut a) = make("ce11", TWO);
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    typing(&mut a, "-changed");
    ctrl(&mut a, 'r');
    assert_eq!(a.input.as_ref().unwrap().text, "a");
    // [CE-11] Tab → 確定して右のセルの入力が開く。
    typing(&mut a, "2");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, keymap::Mode::Edit);
    assert_eq!(a.cols[a.col], "status");
    assert_eq!(a.input.as_ref().unwrap().text, "todo");
    assert_eq!(a.changes.count(), 1);
    // Shift+Tab → 確定して左へ(ためた値が入力に出る)。
    a.key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(a.cols[a.col], "title");
    assert_eq!(a.input.as_ref().unwrap().text, "a2");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.changes.count(), 1);
}

#[test]
fn test_ce_12_newline_value_not_opened() {
    // [CE-12] 改行を含む値は1行の入力を開かず、案内を出す。
    let (_t, mut a) = make(
        "ce12",
        &[
            ("a.md", "---\nmemo: \"一行目\\n二行目\"\n---\n"),
            ("b.md", "---\nmemo: |\n  x\n  y\n---\n"),
        ],
    );
    col_named(&mut a, "memo");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert!(
        a.message.as_deref().unwrap().contains("改行"),
        "{:?}",
        a.message
    );
    ch(&mut a, 'j');
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert!(
        a.message.as_deref().unwrap().contains("改行"),
        "{:?}",
        a.message
    );
}

#[test]
fn test_ce_8_read_only_not_opened() {
    // [CE-8] [WB-5] 読むだけのセルでは開かず、理由をメッセージ行に。
    // [CE-19] CE-8 の形のリスト(複数行にまたがるフロー)も開かずに理由。
    // 書ける形のリスト(d.md の `tags: [x, y]`)の画面の振る舞いはタスク 2(リストの選択の画面)で
    // 確かめる。ここでは核の Cell.lock が無いことだけを見る [CE-16]。
    // [WB-3] フロントマターの無いノート(e.md)は読むだけでなく、入力が開く。
    let (_t, mut a) = make(
        "ce8",
        &[
            ("a.md", "---\ntitle: !!str 12\ntags: [x,\n  y]\n---\n"),
            ("b.md", "---\ntitle: b\n本文\n"),
            ("c.md", "---\ntitle: c\ntitle: d\n---\n"),
            ("d.md", "---\ntitle: d\ntags: [x, y]\n---\n"),
            ("e.md", "本文だけ\n"),
        ],
    );
    let cases = [
        (0, "title", "タグつき"),
        (0, "tags", "複数行"),
        (1, "title", "閉じ"),
        (2, "title", "同じキー"),
    ];
    for (row, col, want) in cases {
        a.row = row;
        col_named(&mut a, col);
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.mode, keymap::Mode::Table, "{col}");
        let s = screen(&a);
        let msg = s.lines().nth(22).unwrap();
        assert!(msg.contains("読むだけ") && msg.contains(want), "{msg}");
    }
    assert_eq!(a.changes.count(), 0);
    // [CE-16] 書ける形のリストのセルは読むだけでない(lock なし)。
    let d = a.rows[3].clone();
    assert_eq!(a.src.label(&d), "d.md");
    let c = a.src.get(&d, "tags");
    assert!(c.lock.is_none(), "writable list is locked: {:?}", c.lock);
    // [WB-3] フロントマターの無いノートのセルは lock なしで、Enter で入力が開く(読むだけの理由は出ない)。
    let e = a.rows[4].clone();
    assert_eq!(a.src.label(&e), "e.md");
    let c = a.src.get(&e, "title");
    assert!(
        c.lock.is_none(),
        "no-frontmatter note is locked: {:?}",
        c.lock
    );
    a.row = 4;
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Edit, "{:?}", a.message);
    assert!(a.input.is_some(), "input opens on no-frontmatter note");
    let s = screen(&a);
    assert!(!s.lines().nth(22).unwrap().contains("読むだけ"), "{s}");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_ce_9_empty_commit_writes_null() {
    // [CE-9] 空で確定 → 保存で `key:`(キーは残す)。
    let (t, mut a) = make("ce9", &[("a.md", "---\nstatus: done\ntitle: a\n---\n")]);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    for _ in 0..4 {
        press(&mut a, KeyCode::Backspace);
    }
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.changes.count(), 1);
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), "---\nstatus:\ntitle: a\n---\n");
    assert_eq!(a.changes.count(), 0);
}

pub(super) fn edit_cell(a: &mut App, row: usize, col: &str, text: &str) {
    a.row = row;
    col_named(a, col);
    press(a, KeyCode::Enter);
    a.input.as_mut().unwrap().text.clear();
    a.input.as_mut().unwrap().cursor = 0;
    typing(a, text);
    press(a, KeyCode::Enter);
}

#[test]
fn test_wb_9_review_then_save() {
    // [WB-9] 2つのセルを直す → ファイルは変わらず「未保存 2」。Ctrl+S → 2ファイルの差分 → 確定で書かれ「未保存 0」。
    let (t, mut a) = make("wb9", TWO);
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 1, "title", "新しい題");
    assert_eq!(a.changes.count(), 2);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    assert_eq!(read(&t, "b.md"), TWO[1].1);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, keymap::Mode::Confirm);
    let s = screen(&a);
    assert!(
        s.contains("-status: todo") && s.contains("+status: done"),
        "{s}"
    );
    assert!(s.contains("+title: 新しい題"), "{s}");
    golden("wb_9", &s);
    assert_fits(&mut a);
    // 確定までは書かない。
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(read(&t, "a.md"), TWO[0].1.replace("todo", "done"));
    assert_eq!(
        read(&t, "b.md"),
        TWO[1].1.replace("title: b", "title: 新しい題")
    );
    let s = screen(&a);
    assert!(s.contains("未保存 0"), "{s}");
    assert!(s.lines().nth(22).unwrap().contains("保存した 2件"), "{s}");
    // 変更が無ければ確認を開かない。
    ctrl(&mut a, 's');
    assert_eq!(a.mode, keymap::Mode::Table);
}

#[test]
fn test_wb_17_revert_clears_pending() {
    // [WB-17] status を done に直して todo に戻す → 「未保存 0」、セルに `*` が無い、Ctrl+S は確認を開かず「変更が無い」。
    let (t, mut a) = make("wb17", TWO);
    edit_cell(&mut a, 0, "status", "done");
    assert_eq!(a.changes.count(), 1);
    let s = screen(&a);
    assert!(s.contains("未保存 1") && s.contains("*done"), "{s}");
    edit_cell(&mut a, 0, "status", "todo");
    assert_eq!(a.changes.count(), 0);
    let s = screen(&a);
    assert!(s.contains("未保存 0"), "{s}");
    assert!(!s.contains("*todo") && !s.contains('*'), "{s}");
    ctrl(&mut a, 's');
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.message.as_deref(), Some("保存する変更が無い"));
    assert_eq!(read(&t, "a.md"), TWO[0].1);
    // [WB-17] 外す操作も1手: `u` で done に戻る。
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("未保存 1"));
}

#[test]
fn test_wb_17_external_same_value_clears_pending() {
    // [WB-17] status に done をためたまま、外で同じ done に直す → poll → 「未保存 0」・`*` なし。`u` で戻る。
    let (t, mut a) = make("wb17ext", TWO);
    edit_cell(&mut a, 0, "status", "done");
    assert!(screen(&a).contains("未保存 1"));
    let outside = "---\ntitle: a\nstatus: done\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    assert_eq!(a.changes.count(), 0);
    let s = screen(&a);
    assert!(s.contains("未保存 0") && !s.contains('*'), "{s}");
    assert_eq!(read(&t, "a.md"), outside);
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 1);
    assert!(screen(&a).contains("未保存 1"));
}

#[test]
fn test_wb_17_save_drops_same_before_review() {
    // [WB-17] 読んだ値と同じセルだけがためてある(上書きで外れたものを `u` で戻した)→ Ctrl+S は確認を開かず
    // 「保存する変更が無い」、未保存 0、ファイルは変わらない。
    let (t, mut a) = make("wb17save", TWO);
    edit_cell(&mut a, 0, "status", "done");
    let row = a.rows[0].clone();
    let outside = "---\ntitle: a\nstatus: done\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    let changed = a.src.changed();
    a.changes.note_external(&changed);
    a.changes.overwrite(a.src.as_mut(), &row).unwrap();
    ch(&mut a, 'u');
    assert_eq!(a.changes.count(), 1);
    ctrl(&mut a, 's');
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.message.as_deref(), Some("保存する変更が無い"));
    assert_eq!(a.changes.count(), 0);
    assert!(screen(&a).contains("未保存 0"));
    assert_eq!(read(&t, "a.md"), outside);
}

#[test]
fn test_wb_9_review_back_keeps_changes() {
    // [WB-9] 差分の画面で Esc → 書かずに表へ戻り、ためた変更は残る。
    let (t, mut a) = make("wb9back", TWO);
    edit_cell(&mut a, 0, "status", "done");
    ctrl(&mut a, 's');
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
}

/// 外でノートを書き換える(更新時刻を進めて、変化の検出に確実に見つけさせる)。
pub(super) fn edit_outside(tmp: &Tmp, name: &str, text: &str) {
    let p = tmp.notes().join(name);
    let old = std::fs::metadata(&p).unwrap().modified().unwrap();
    std::fs::write(&p, text).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&p)
        .unwrap()
        .set_modified(old + std::time::Duration::from_secs(10))
        .unwrap();
}

#[test]
fn test_wb_16_external_change_overwrite() {
    // [WB-16] status をためたまま外で title を直す → 行に印。保存 → 止まり、差分の画面に印と2つの選択肢。
    let (t, mut a) = make("wb16", TWO);
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 1, "status", "done");
    let outside = "---\ntitle: a outside\nstatus: todo\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    assert!(screen(&a).contains("!a "));
    ctrl(&mut a, 's');
    let s = screen(&a);
    assert!(s.contains("!a.md") && s.contains("外で変更"), "{s}");
    // [WB-14] 確定 → a は止まり(未保存は残る)、b は書ける。画面に残って選ばせる。
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, keymap::Mode::Confirm);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(read(&t, "a.md"), outside);
    assert_eq!(read(&t, "b.md"), TWO[1].1.replace("doing", "done"));
    let s = screen(&a);
    assert!(
        s.contains("止めた") && s.contains("o このファイルを外の変更の上に書く"),
        "{s}"
    );
    assert!(s.contains("d ためた変更を捨てる"), "{s}");
    golden("wb_16", &s);
    assert_fits(&mut a);
    // 外の変更の上に書く → 書かれて、外の変更も残る。
    ch(&mut a, 'o');
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 0);
    assert_eq!(read(&t, "a.md"), outside.replace("todo", "done"));
}

#[test]
fn test_wb_16_external_change_discard() {
    // [WB-16] ためた変更を捨てる → その行の変更が消え、ファイルは外の変更のまま。
    let (t, mut a) = make("wb16d", TWO);
    edit_cell(&mut a, 0, "status", "done");
    let outside = "---\ntitle: a outside\nstatus: todo\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    ctrl(&mut a, 's');
    ch(&mut a, 'd');
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 0);
    assert_eq!(read(&t, "a.md"), outside);
}

#[test]
fn test_wb_11_quit_confirm() {
    // [WB-11] 未保存 1 で `q` → 保存する・捨てる・戻る の確認。
    let (t, mut a) = make("wb11", TWO);
    edit_cell(&mut a, 0, "status", "done");
    ch(&mut a, 'q');
    assert!(!a.quit);
    assert_eq!(a.mode, keymap::Mode::Quit);
    let s = screen(&a);
    assert!(
        s.contains("s 保存する") && s.contains("d 捨てて終わる"),
        "{s}"
    );
    assert!(s.contains("Esc 戻る"), "{s}");
    golden("wb_11", &s);
    assert_fits(&mut a);
    // 戻る → 変更は残り、終わらない。
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert!(!a.quit);
    assert_eq!(a.changes.count(), 1);
    // 捨てる → 書かずに終わる。
    ch(&mut a, 'q');
    ch(&mut a, 'd');
    assert!(a.quit);
    assert_eq!(read(&t, "a.md"), TWO[0].1);
}

#[test]
fn test_wb_11_quit_save() {
    // [WB-11] 保存する → 差分の画面 → 確定で書いて終わる。
    let (t, mut a) = make("wb11s", TWO);
    edit_cell(&mut a, 0, "status", "done");
    ch(&mut a, 'q');
    ch(&mut a, 's');
    assert_eq!(a.mode, keymap::Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert!(a.quit);
    assert_eq!(read(&t, "a.md"), TWO[0].1.replace("todo", "done"));
}

#[test]
fn test_wb_16_overwrite_only_selected_file() {
    // [WB-16] `o` は選んだファイルだけを書き、残りのためた変更は Enter に任せる(画面に残る)。
    let (t, mut a) = make("wb16one", TWO);
    edit_cell(&mut a, 0, "status", "done");
    edit_cell(&mut a, 1, "status", "done");
    let outside = "---\ntitle: a outside\nstatus: todo\n---\n本文 a\n";
    edit_outside(&t, "a.md", outside);
    a.poll();
    ctrl(&mut a, 's');
    assert!(screen(&a).contains("o このファイルを外の変更の上に書く"));
    ch(&mut a, 'o');
    assert_eq!(read(&t, "a.md"), outside.replace("todo", "done"));
    assert_eq!(read(&t, "b.md"), TWO[1].1);
    assert_eq!(a.changes.count(), 1);
    assert_eq!(a.mode, keymap::Mode::Confirm);
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "b.md"), TWO[1].1.replace("doing", "done"));
    assert_eq!(a.mode, keymap::Mode::Table);
}

#[test]
fn test_wb_16_overwrite_refuses_unseen_content() {
    // [WB-16] [WB-9] 差分を見せたあとにまた外で変わったら、`o` は書かずに差分を作り直す。
    let (t, mut a) = make("wb16seen", TWO);
    edit_cell(&mut a, 0, "status", "done");
    edit_outside(
        &t,
        "a.md",
        "---\ntitle: a outside\nstatus: todo\n---\n本文 a\n",
    );
    a.poll();
    ctrl(&mut a, 's');
    let again = "---\ntitle: a again\nstatus: todo\n---\n本文 a\n";
    edit_outside(&t, "a.md", again);
    ch(&mut a, 'o');
    assert_eq!(read(&t, "a.md"), again);
    assert_eq!(a.mode, keymap::Mode::Confirm);
    assert_eq!(a.changes.count(), 1);
    let s = screen(&a);
    assert!(
        s.contains("また外で変わった") && s.contains("title: a again"),
        "{s}"
    );
    // Enter で保存しても止まる(見せていない内容の上に書かない)。
    press(&mut a, KeyCode::Enter);
    assert_eq!(read(&t, "a.md"), again);
    // 見直した差分の上なら書ける。
    ch(&mut a, 'o');
    assert_eq!(read(&t, "a.md"), again.replace("todo", "done"));
    assert_eq!(a.changes.count(), 0);
}

#[test]
fn test_bv_9_no_poll_while_editing() {
    // [BV-9] [CE-1] 編集中は外の変化を当てない(入力ボックスが別のセルに移らない)。閉じたあとの poll で当てる。
    let (t, mut a) = make("bv9edit", TWO);
    col_named(&mut a, "status");
    press(&mut a, KeyCode::Enter);
    let before = view::cursor(&a);
    std::fs::remove_file(t.notes().join("a.md")).unwrap();
    a.poll();
    assert_eq!(a.rows.len(), 2);
    assert_eq!(view::cursor(&a), before);
    assert_eq!(a.input.as_ref().unwrap().text, "todo");
    press(&mut a, KeyCode::Esc);
    for _ in 0..200 {
        a.poll();
        if a.rows.len() == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(a.rows.len(), 1);
}

#[test]
fn test_ce_11_tab_skips_read_only() {
    // [CE-11] Tab で読むだけのセルは飛ばし、同じ行の次の直せるセルの入力を開く。
    // 読むだけの材料は CE-8 の形(複数行にまたがるフローのリスト)。1行の `[x]` は CE-16 で書ける。
    let (_t, mut a) = make(
        "ce11ro",
        &[(
            "a.md",
            "---\ntitle: a\ntags: [x,\n  w]\nstatus: todo\nmemo: [y,\n  z]\n---\n",
        )],
    );
    col_named(&mut a, "title");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, keymap::Mode::Edit);
    assert_eq!(a.cols[a.col], "status");
    // 右に直せるセルが無ければ、確定して表に戻り理由を出す。
    typing(&mut a, "!");
    press(&mut a, KeyCode::Tab);
    assert_eq!(a.mode, keymap::Mode::Table);
    assert_eq!(a.changes.count(), 1);
    let msg = a.message.clone().unwrap();
    assert!(
        msg.contains("直せるセルが無い") && msg.contains("複数行"),
        "{msg}"
    );
    // Shift+Tab も同じく飛ばす。
    press(&mut a, KeyCode::Enter);
    a.key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(a.cols[a.col], "title");
    assert_eq!(a.mode, keymap::Mode::Edit);
}
