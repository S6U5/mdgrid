//! 表の画面で行の URL を開くキー(screen.open-url-key)。期待値は要求の受け入れの例から取った。

use super::app::{App, ColorMode};
use super::external::OpenLog;
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, screen, Tmp};
use mdgrid::source::markdown::Markdown;

/// 開く口に渡った URL。
type Seen = std::rc::Rc<std::cell::RefCell<Vec<String>>>;

/// `url: https://example.com/mail` の行と、URL の無い行。
const NOTES: &[(&str, &str)] = &[
    ("mail.md", "---\nurl: https://example.com/mail\n---\n"),
    ("memo.md", "---\nnote: x\n---\n"),
];

/// 設定 `config`(config.toml の中身)で表を開き、開く口を記録に替える。設定の読みの警告は無いこと。
fn boot(tag: &str, config: &str) -> (Tmp, App, Seen) {
    let tmp = Tmp::new(tag);
    for (n, t) in NOTES {
        tmp.write(n, t);
    }
    let (cfg, warnings) = mdgrid::config::parse(config).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let src = Markdown::open(&[tmp.notes()]).unwrap();
    let mut a = App::new(Box::new(src), ColorMode::None);
    a.resize(120, 30);
    a.start(Startup {
        config: cfg,
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
    let log = OpenLog::default();
    let seen = log.0.clone();
    a.opener = Box::new(log);
    (tmp, a, seen)
}

/// `name` のノートの行を選ぶ。
fn select(a: &mut App, name: &str) {
    a.row = (0..a.slots.len())
        .find(|&i| {
            matches!(a.slots[i], super::grid::Slot::Row(r)
            if a.src.label(&a.rows[r]).trim_end_matches(".md") == name)
        })
        .unwrap_or_else(|| panic!("{name} の行が表に無い"));
}

// [utest -> req~screen.open-url-key~1]
#[test]
fn test_shift_o_opens_the_url_of_the_row() {
    // `url: https://example.com/mail` の行を選んだ表で `O` → その URL を開く口に渡し、メッセージ行に URL を出す。
    let (_t, mut a, log) = boot("open-url-o", "");
    select(&mut a, "mail");
    ch(&mut a, 'O');
    assert_eq!(a.mode, Mode::Table);
    assert_eq!(
        *log.borrow(),
        vec!["https://example.com/mail".to_string()],
        "{:?}",
        a.message
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("https://example.com/mail"), "{m}");
    assert!(screen(&a).contains("https://example.com/mail"));
}

// [utest -> req~screen.open-url-key~1]
#[test]
fn test_open_url_can_be_bound_in_the_table() {
    // 設定に `[keys.table]` の `"u" = "open_url"` → 起動で警告を出さず、`u` で選んだ行の URL を開く。
    let (_t, mut a, log) = boot("open-url-u", "[keys.table]\nu = \"open_url\"\n");
    assert!(a.message.is_none(), "{:?}", a.message);
    select(&mut a, "mail");
    ch(&mut a, 'u');
    assert_eq!(
        *log.borrow(),
        vec!["https://example.com/mail".to_string()],
        "{:?}",
        a.message
    );
}
