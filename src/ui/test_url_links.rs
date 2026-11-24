//! 切って見せた URL に付ける行き先(cell-view.url-full-target)。値は作ったもの。
//! 端末へ書く列(OSC 8)を、本物の crossterm の書き出しで作って読む。

use super::hyperlink::{linked, write};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, press, Tmp};
use super::*;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::{CrosstermBackend, TestBackend};
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;

const URL: &str = "https://example.com/docs/guide/chapter-12/section-3";
/// 24 桁に切った見え方(23 字と `…`)。
const CUT: &str = "https://example.com/doc…";

fn boot(name: &str, notes: &[(&str, &str)], extra: &str, table: &str) -> (Tmp, App) {
    let tmp = Tmp::new(name);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    if !table.is_empty() {
        std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
        std::fs::write(
            tmp.0.join(".mdgrid/workspace.toml"),
            format!("[[table]]\nname = \"BM\"\npath = \"notes\"\n{table}\n"),
        )
        .unwrap();
    }
    let toml = format!("[display]\ntop_padding = 0\nbar_padding = 0\n{extra}");
    let (config, warnings) = mdgrid::config::parse(&toml).unwrap();
    assert!(warnings.is_empty(), "設定の警告: {warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(100, 30);
    a.start(Startup {
        config,
        warnings: Vec::new(),
        readonly: false,
        no_color: true,
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: tmp.notes(),
        base: None,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    a.widths.insert("url".into(), 24);
    (tmp, a)
}

fn note(url: &str) -> String {
    format!("---\nurl: {url}\n---\n")
}

/// 画面を描き、端末へ書く行き先の列を作る。返すのは、書いた列の全部と、
/// 行き先ごとの(URL・包んだ文字を書いた順につないだもの・包んだ画面の行の数)。
fn written(a: &App) -> (String, Vec<(String, String, usize)>) {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    let done = t.draw(|f| super::draw(f, a)).unwrap();
    let links = linked(a, done.buffer, super::screen_area(done.area));
    let mut raw = Vec::<u8>::new();
    write(&mut CrosstermBackend::new(&mut raw), &links).unwrap();
    let bytes = String::from_utf8(raw).unwrap();
    let found = links
        .iter()
        .map(|(url, cells)| {
            let text: String = cells.iter().map(|(_, _, c)| c.symbol()).collect();
            let mut rows: Vec<u16> = cells.iter().map(|(_, y, _)| *y).collect();
            rows.dedup();
            (url.clone(), text, rows.len())
        })
        .collect();
    (bytes, found)
}

/// 端末へ書いた列の、行き先の始まり。
fn open(url: &str) -> String {
    format!("\x1b]8;;{url}\x1b\\")
}

// [utest -> req~cell-view.url-full-target~1]
/// 表で `…` に切った URL の文字に、全体の URL が行き先として付く。
#[test]
fn test_table_cut_url_links_to_full_url() {
    let (_tmp, a) = boot("url_links_table", &[("a.md", &note(URL))], "", "");
    assert!(super::test_screen::screen(&a).contains(CUT));
    let (bytes, found) = written(&a);
    assert_eq!(found, vec![(URL.to_string(), CUT.to_string(), 1)]);
    let at = bytes.find(&open(URL)).expect("行き先の始まりが無い");
    let end = bytes[at..]
        .find("\x1b]8;;\x1b\\")
        .expect("行き先の終わりが無い");
    assert!(bytes[at..at + end].contains("https://example.com/doc"));
}

// [utest -> req~cell-view.url-full-target~1]
/// 折り返した URL の行のどれにも、同じ全体の URL が行き先として付く。
#[test]
fn test_wrapped_url_links_every_line() {
    let (_tmp, a) = boot(
        "url_links_wrap",
        &[("a.md", &note(URL))],
        "row_extra_lines = 2\n",
        "",
    );
    let (_, found) = written(&a);
    // 51 字を 24 桁で折り返すと 24・24・3 の3行。
    assert_eq!(found, vec![(URL.to_string(), URL.to_string(), 3)]);
}

// [utest -> req~cell-view.url-full-target~1]
/// 木の画面の右の欄で `…` に切った URL の文字に、全体の URL が行き先として付く。
#[test]
fn test_tree_cut_url_links_to_full_url() {
    let text = format!("---\nfolder: bar\nurl: {URL}\n---\n");
    let (_tmp, mut a) = boot(
        "url_links_tree",
        &[("a.md", &text)],
        "",
        "tree = { path = \"folder\" }",
    );
    ch(&mut a, 'T');
    assert_eq!(a.mode, Mode::Tree);
    let (_, found) = written(&a);
    assert_eq!(found, vec![(URL.to_string(), CUT.to_string(), 1)]);
}

// [utest -> req~cell-view.url-full-target~1]
/// 詳細の表示で2行に折り返した URL のどちらの行にも、全体の URL が行き先として付く。
#[test]
fn test_detail_wrapped_url_links_both_lines() {
    let (_tmp, mut a) = boot("url_links_detail", &[("a.md", &note(URL))], "", "");
    a.resize(40, 20);
    ch(&mut a, 'K');
    assert_eq!(a.mode, Mode::Detail);
    let (_, found) = written(&a);
    assert_eq!(found, vec![(URL.to_string(), URL.to_string(), 2)]);
}

// [utest -> req~cell-view.url-full-target~1]
/// 開かない URL(実行ファイルを指す)と、値の全体が URL でない値には、行き先を付けない。
#[test]
fn test_refused_and_partial_urls_get_no_link() {
    let (_tmp, a) = boot(
        "url_links_refused",
        &[
            ("a.md", &note("https://example.com/files/report.pdf.exe")),
            ("b.md", &note("notes about https://example.com")),
        ],
        "",
        "",
    );
    let (bytes, found) = written(&a);
    assert_eq!(found, Vec::new());
    assert_eq!(bytes, "");
}

// [utest -> req~cell-view.url-full-target~1]
/// url のセルに重なった操作の一覧の窓の文字には、行き先を付けない。
#[test]
fn test_overlapped_cell_gets_no_link() {
    let second = "https://example.org/another/long/path/to/a/page/far/away";
    let (_tmp, mut a) = boot(
        "url_links_overlap",
        &[("a.md", &note(URL)), ("b.md", &note(second))],
        "",
        "",
    );
    let (_, before) = written(&a);
    assert_eq!(
        before.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(),
        vec![URL, second]
    );
    ch(&mut a, 'x');
    assert_eq!(a.mode, Mode::Menu);
    let (_, after) = written(&a);
    assert_eq!(
        after.iter().map(|f| f.0.as_str()).collect::<Vec<_>>(),
        vec![URL]
    );
    press(&mut a, KeyCode::Esc);
}

// [utest -> req~cell-view.url-full-target~1]
/// `TERM=dumb` では、行き先の列を端末へ書かない。
#[test]
fn test_dumb_terminal_writes_no_link() {
    let (_tmp, mut a) = boot("url_links_dumb", &[("a.md", &note(URL))], "", "");
    a.no_emoji = true;
    let (bytes, found) = written(&a);
    assert_eq!(found, Vec::new());
    assert_eq!(bytes, "");
}
