//! ノートの欄の名前の正規表現 `display.note_name`(SR-47。SR-29・SR-46 の `display.note_path` と重ねる)の画面の
//! 受け入れ試験。一致すれば組 `name`・1つ目の組・一致した部分を除いた残りの順で出し、一致しないときと空になるときは
//! 名前のまま。フォルダは note_path に従って残し、正規表現は最後の名前の部分にだけ当てる。検索は根からのパスのまま。
//! 記録 specs/_decisions/2026-10-15-note-name.md。設定の読み込みは tests/test_note_name.rs。
//! 実装を見ずに、設定の TOML・画面の文字・キーで確かめる(行と桁の位置は決め打ちしない)。

use super::startup::Startup;
use super::test_screen::{ch, press, text, typing, Tmp};
use super::*;
use mdgrid::config::{parse, Config};
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

// ---- 材料 ----

/// `files` の(パス, 中身)のノートを置いた一時フォルダ。
fn notes(name: &str, files: &[(&str, &str)]) -> Tmp {
    let tmp = Tmp::new(name);
    for (p, body) in files {
        if let Some((dir, _)) = p.rsplit_once('/') {
            std::fs::create_dir_all(tmp.notes().join(dir)).unwrap();
        }
        tmp.write(p, body);
    }
    tmp
}

/// 設定の TOML を読む(警告が無いことも確かめる)。
fn cfg_ok(text: &str) -> Config {
    let (c, w) = parse(text).expect("設定の TOML は読める");
    assert!(w.is_empty(), "設定の警告: {w:?}\n---\n{text}");
    c
}

/// フォルダを main と同じ道筋で開く。
fn boot(tmp: &Tmp, config: Config) -> App {
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut app = App::new(Box::new(src), ColorMode::None);
    app.resize(100, 30);
    app.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    app
}

// ---- 画面の読み取り ----

/// 今の大きさで描いた画面の文字。
fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| draw(f, a)).unwrap();
    text(term.backend().buffer())
}

/// `word`(その行だけにある値)を含む表の行(最初の1つ)。
fn row_line(a: &App, word: &str) -> String {
    let s = screen(a);
    s.lines()
        .find(|l| l.contains(word))
        .unwrap_or_else(|| panic!("「{word}」の行が無い:\n{s}"))
        .to_string()
}

/// 値 `word` の行のノートの欄(行の最初の語から、選んだ行の印 `>` を除く。試験の名前には空白を入れない)。
fn note_cell(a: &App, word: &str) -> String {
    let line = row_line(a, word);
    line.split(|c: char| c.is_whitespace() || c == '│')
        .find(|t| !t.is_empty())
        .unwrap_or("")
        .trim_start_matches('>')
        .to_string()
}

/// 1つのノート `file`(値 `v1` で行を見分ける)を、`note_name` の正規表現と `note_path` で開く。
fn one(tag: &str, file: &str, path: &str, re: &str) -> (Tmp, App) {
    let tmp = notes(tag, &[(file, "---\nstatus: v1\n---\n")]);
    let toml = format!("[display]\nnote_path = \"{path}\"\nnote_name = '{re}'\n");
    let a = boot(&tmp, cfg_ok(&toml));
    (tmp, a)
}

// ---- SR-47: 正規表現の当て方 ----

#[test]
fn test_sr_47_strip_leading_date() {
    // [SR-47] note_path = "name"・note_name = '^\d{8}_' で plan/20260301_見積を出す.md → 欄は 見積を出す。
    let (_t, a) = one(
        "sr47strip",
        "plan/20260301_見積を出す.md",
        "name",
        r"^\d{8}_",
    );
    let s = screen(&a);
    assert_eq!(note_cell(&a, "v1"), "見積を出す", "{s}");
    assert!(!s.contains("20260301"), "日付は出さない:\n{s}");
    assert!(!s.contains("plan/"), "フォルダは出さない:\n{s}");
}

#[test]
fn test_sr_47_named_group_name() {
    // [SR-47] 名前の付いた組 `name` があればその部分(1つ目の組より先)。
    let (_t, a) = one(
        "sr47named",
        "plan/20260301_見積を出す.md",
        "name",
        r"^(\d{8})_(?P<name>.+)$",
    );
    assert_eq!(note_cell(&a, "v1"), "見積を出す", "{}", screen(&a));
}

#[test]
fn test_sr_47_first_group() {
    // [SR-47] 組 `name` が無く組があれば1つ目の組: '^(.+)_v\d+$' で 設計書_v12.md → 設計書。
    let (_t, a) = one("sr47first", "設計書_v12.md", "name", r"^(.+)_v\d+$");
    let s = screen(&a);
    assert_eq!(note_cell(&a, "v1"), "設計書", "{s}");
    assert!(!s.contains("_v12"), "{s}");
}

#[test]
fn test_sr_47_no_match_keeps_name() {
    // [SR-47] 一致しない名前はそのまま: '^\d{8}_' で 覚え書き.md → 覚え書き。一致する行と並べても崩れない。
    let tmp = notes(
        "sr47nomatch",
        &[
            ("覚え書き.md", "---\nstatus: v1\n---\n"),
            ("20260302_下書き.md", "---\nstatus: v2\n---\n"),
        ],
    );
    let a = boot(
        &tmp,
        cfg_ok("[display]\nnote_path = \"name\"\nnote_name = '^\\d{8}_'\n"),
    );
    let s = screen(&a);
    assert_eq!(note_cell(&a, "v1"), "覚え書き", "{s}");
    assert_eq!(note_cell(&a, "v2"), "下書き", "{s}");
}

#[test]
fn test_sr_47_empty_result_keeps_name() {
    // [SR-47] 結果が空になる正規表現は名前のまま: '^.*$'(全部を除く)・'^(x*)'(空の組)。
    let (_t, a) = one("sr47empty", "棚卸し.md", "name", "^.*$");
    assert_eq!(note_cell(&a, "v1"), "棚卸し", "{}", screen(&a));
    let (_u, b) = one("sr47empty2", "棚卸し.md", "name", "^(x*)");
    assert_eq!(note_cell(&b, "v1"), "棚卸し", "{}", screen(&b));
}

#[test]
fn test_sr_47_common_keeps_folders() {
    // [SR-47][SR-29] note_path = "common" と '^\d{8}_' で inbox/20260303_p.md・old/inbox/20260304_q.md →
    // inbox/p・old/inbox/q(フォルダは残し、最後の名前の部分にだけ当てる)。
    let tmp = notes(
        "sr47common",
        &[
            ("inbox/20260303_p.md", "---\nstatus: v1\n---\n"),
            ("old/inbox/20260304_q.md", "---\nstatus: v2\n---\n"),
        ],
    );
    let a = boot(
        &tmp,
        cfg_ok("[display]\nnote_path = \"common\"\nnote_name = '^\\d{8}_'\n"),
    );
    let s = screen(&a);
    assert_eq!(note_cell(&a, "v1"), "inbox/p", "{s}");
    assert_eq!(note_cell(&a, "v2"), "old/inbox/q", "{s}");
    // フォルダの名前に一致しそうな正規表現でも、当てるのは最後の部分だけ。
    let tmp = notes(
        "sr47common2",
        &[
            ("20260305_box/20260306_r.md", "---\nstatus: v1\n---\n"),
            ("other/s.md", "---\nstatus: v2\n---\n"),
        ],
    );
    let a = boot(
        &tmp,
        cfg_ok("[display]\nnote_path = \"common\"\nnote_name = '^\\d{8}_'\n"),
    );
    let s = screen(&a);
    assert_eq!(note_cell(&a, "v1"), "20260305_box/r", "{s}");
}

#[test]
fn test_sr_47_search_still_matches_date() {
    // [SR-47][NV-1] 欄から日付を除いても、`\` の検索で 20260307 と打つとその行が当たる(検索は根からのパスのまま)。
    let tmp = notes(
        "sr47search",
        &[
            ("todo/20260307_片付け.md", "---\nstatus: v1\n---\n"),
            ("todo/20260308_買い出し.md", "---\nstatus: v2\n---\n"),
        ],
    );
    let mut a = boot(
        &tmp,
        cfg_ok("[display]\nnote_path = \"name\"\nnote_name = '^\\d{8}_'\n"),
    );
    ch(&mut a, '\\');
    typing(&mut a, "20260307");
    press(&mut a, KeyCode::Enter);
    let labels: Vec<String> = a.rows.iter().map(|r| a.src.label(r)).collect();
    assert_eq!(labels, vec!["todo/20260307_片付け.md".to_string()]);
    assert_eq!(note_cell(&a, "v1"), "片付け", "{}", screen(&a));
}

#[test]
fn test_sr_47_ledger_look() {
    // [SR-47][SR-29][SR-46] 台帳の形: todo/20261015_資料を作る.md・archive/todo/20261001_会議の準備.md
    // (frontmatter に title と status)、`[display] note_path = "name"`・`note_name = '^\d{8}_'` → 欄は
    // 資料を作る・会議の準備。画面にフォルダは出ず、表の行に作成日の数字も出ない。
    let tmp = notes(
        "sr47ledger",
        &[
            (
                "todo/20261015_資料を作る.md",
                "---\ntitle: 配布の資料\nstatus: 進行中\n---\n",
            ),
            (
                "archive/todo/20261001_会議の準備.md",
                "---\ntitle: 定例の段取り\nstatus: 完了\n---\n",
            ),
        ],
    );
    let a = boot(
        &tmp,
        cfg_ok("[display]\nnote_path = \"name\"\nnote_name = '^\\d{8}_'\n"),
    );
    let s = screen(&a);
    assert_eq!(note_cell(&a, "配布の資料"), "資料を作る", "{s}");
    assert_eq!(note_cell(&a, "定例の段取り"), "会議の準備", "{s}");
    assert!(!s.contains("todo/"), "フォルダは出さない:\n{s}");
    assert!(!s.contains("archive/"), "フォルダは出さない:\n{s}");
    for word in ["資料を作る", "会議の準備"] {
        let row = row_line(&a, word);
        for date in ["20261015", "20261001"] {
            assert!(!row.contains(date), "行に作成日を出さない: {row:?}\n{s}");
        }
    }
}
