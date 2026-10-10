//! 表の見せ方の切り替え(SR-20・SR-21)のレビューの指摘の試験(display-options の差し戻し1回目)。
//! 受け入れの試験は test_display_options.rs。ここは区切り線と幅2の Ambiguous・一行おきの色の数え方と色・
//! 低い画面の「表示」の節・帯を隠したときの `f`・タブを隠したときのビューの名前・書き出しの知らせの順。

use super::keymap::Mode;
use super::settings::Sec;
use super::startup::Startup;
use super::test_screen::{app_of, buffer, ch, press, screen, Tmp};
use super::*;
use mdgrid::config::parse;
use mdgrid::settings::{Dir, Group};
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Color;

fn vault(name: &str) -> Tmp {
    let tmp = Tmp::new(name);
    let notes = [
        ("a.md", "alpha", "todo"),
        ("b.md", "beta", "done"),
        ("c.md", "gamma", "todo"),
        ("d.md", "delta", "done"),
        ("e.md", "eps", "todo"),
    ];
    for (n, t, s) in notes {
        tmp.write(n, &format!("---\ntitle: {t}\nstatus: {s}\n---\n"));
    }
    tmp
}

fn with(name: &str, config: &str, color: ColorMode) -> (Tmp, App) {
    let tmp = vault(name);
    let mut a = app_of(&tmp, color);
    // 表の見せ方(SR-20)は今までの見た目(look = "classic")の上で確かめる。
    a.configure(
        &parse(&format!("{config}\n\n[look]\nmode = \"classic\""))
            .unwrap()
            .0,
    );
    (tmp, a)
}

// ---- 中1: 区切り線は ambiguous_wide でも幅1 ----

#[test]
fn test_sr_20_column_lines_with_ambiguous_wide_keep_positions() {
    // [SR-20][CV-6] `│` は Ambiguous。ambiguous_wide では幅1の `|` を引き、列の位置(描いた文字・クリック)がずれない。
    let (_t, mut a) = with(
        "dmamb",
        "[terminal]\nambiguous_wide = true\n\n[display]\ncolumn_lines = true\n",
        ColorMode::None,
    );
    let s = screen(&a);
    assert!(!s.contains('│'), "{s}");
    assert!(s.contains('|'), "{s}");
    let (lay, cols) = view::visible_layout(&a);
    assert!(cols.len() >= 2, "{s}");
    let (j1, w0) = (cols[1].0, cols[0].1);
    let x = (lay.data_x() + w0 + 1) as u16;
    let dy = view::data_y(&a) as u16;
    let buf = buffer(&a);
    // 列の見出しの行: 区切りの右が2つめの列の見出しの1文字目。
    assert_eq!(buf[(x - 1, dy - 1)].symbol(), "|");
    let title = a.cols[j1].chars().next().unwrap().to_string();
    assert_eq!(buf[(x, dy - 1)].symbol(), title, "{s}");
    // ノートの行も同じ桁に区切り。
    assert_eq!(buf[(x - 1, dy + 1)].symbol(), "|");
    // そこのクリックはその列を選ぶ。
    a.click(x, dy + 1);
    assert_eq!((a.row, a.col), (1, j1));
}

// ---- 中2・中3: 一行おきの色はノートの行の番号で数え、前景も決める ----

#[test]
fn test_sr_20_zebra_counts_note_rows_not_headings() {
    // [SR-20] グループの見出しがあっても、ノートの行の番号 2・4・… に色。見出しの行には付けない。
    let (_t, mut a) = with("dmzebra", "[display]\nzebra = true\n", ColorMode::Indexed);
    a.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: false,
    };
    a.refresh();
    assert!(!a.groups.is_empty());
    a.row = 0;
    let buf = buffer(&a);
    let dy = view::data_y(&a);
    for (k, slot) in a.slots.iter().enumerate() {
        let cell = &buf[(3u16, (dy + k) as u16)];
        match slot {
            grid::Slot::Row(r) if r % 2 == 1 => {
                assert_eq!(cell.bg, Color::Indexed(237), "行番号 {} に色", r + 1);
                assert_eq!(cell.fg, Color::Indexed(252), "前景も決める");
            }
            _ => assert_eq!(cell.bg, Color::Reset, "{k} 行目({slot:?})に色が無い"),
        }
    }
}

// ---- 中4: 低い画面でも「表示」の節の選んだ項目が見える ----

#[test]
fn test_sr_20_display_section_scrolls_on_low_screen() {
    // [SR-20][NV-18] 低い画面では節の中を流し、選んだ項目(`>` の印)がいつも見える。
    let (_t, mut a) = with("dmlow", "", ColorMode::None);
    a.resize(80, 14);
    ch(&mut a, 'o');
    assert_eq!(a.mode, Mode::Settings);
    for _ in 0..8 {
        if a.draft.as_ref().unwrap().sec == Sec::Display {
            break;
        }
        press(&mut a, KeyCode::Tab);
    }
    assert_eq!(a.draft.as_ref().unwrap().sec, Sec::Display);
    for (i, (_, name)) in mdgrid::display::ITEMS.iter().enumerate() {
        assert_eq!(a.draft.as_ref().unwrap().at(Sec::Display), i);
        let s = screen(&a);
        // NV-18: 選んだ項目は `>` の印(スイッチの形 `● オン` / `○ オフ`)。
        assert!(
            s.contains(&format!(">{name}")),
            "選んだ「{name}」が見えない:\n{s}"
        );
        press(&mut a, KeyCode::Down);
    }
    // 最後の項目(設定の帯)を切り替えても見えたまま。
    ch(&mut a, ' ');
    let s = screen(&a);
    let line = s
        .lines()
        .find(|l| l.contains(">設定の帯"))
        .unwrap_or_default();
    assert!(line.contains("○ オフ"), "{s}");
}

// ---- 中5: 帯を隠したままの f は選んだ条件をメッセージ行に出す ----

#[test]
fn test_sr_20_hidden_chips_f_shows_selected_condition() {
    // [SR-20][NV-22] 帯を隠していても、f と ←→ で選んだ条件が最下行に見え、外す前に分かる。
    let (_t, mut a) = with("dmchips", "[display]\nchips = false\n", ColorMode::None);
    a.settings.sorts.push(("status".into(), Dir::Desc));
    a.settings.group = Group::Off;
    a.refresh();
    ch(&mut a, 'f');
    assert_eq!(a.mode, Mode::Chips);
    // メッセージ行(最下行の1つ上)。
    let last = |a: &App| screen(a).lines().nth(22).unwrap_or("").to_string();
    assert!(last(&a).contains("並べ替え: status"), "{}", screen(&a));
    press(&mut a, KeyCode::Right);
    assert!(last(&a).contains("グループ: なし"), "{}", screen(&a));
    press(&mut a, KeyCode::Backspace);
    assert_eq!(a.settings.group, Group::Inherit, "選んだ条件を外す");
    // 外したあとも条件が残っていれば、次に選ばれた条件が見える(見えないまま続けて外さない)。
    assert_eq!(a.mode, Mode::Chips);
    let msg = a.message.clone().unwrap_or_default();
    assert!(msg.contains("外した: グループ: なし"), "{msg}");
    assert!(msg.contains("並べ替え: status"), "{msg}");
    press(&mut a, KeyCode::Backspace);
    assert!(a.settings.sorts.is_empty());
    assert_eq!(a.mode, Mode::Table);
}

// ---- 低: タブを隠したらヘッダーにビューの名前、切り替えを知らせる ----

#[test]
fn test_sr_20_hidden_tabs_header_shows_view_name() {
    // [SR-20][BV-13] タブを隠すと、ヘッダーに今のビューの名前。`]` で切り替えたらメッセージ行で知らせる。
    let tmp = vault("dmtabs");
    let p = tmp.notes().join("t.base");
    std::fs::write(
        &p,
        "views:\n  - type: table\n    name: 一つ目\n  - type: table\n    name: 二つ目\n",
    )
    .unwrap();
    let t = crate::open_target(std::slice::from_ref(&p), None).unwrap();
    let mut a = App::new(Box::new(t.src), ColorMode::None);
    a.resize(80, 24);
    a.start(Startup {
        config: parse("[display]\ntabs = false\n").unwrap().0,
        warnings: Vec::new(),
        readonly: false,
        no_color: false,
        state_dir: None,
        config_dir: None,
        target: p,
        base: t.base,
    });
    while !a.loaded() {
        a.load_step(100);
    }
    let head = |a: &App| screen(a).lines().next().unwrap_or("").to_string();
    assert!(head(&a).contains("ビュー 一つ目"), "{}", screen(&a));
    ch(&mut a, ']');
    assert!(head(&a).contains("ビュー 二つ目"), "{}", screen(&a));
    assert!(
        a.message.as_deref().is_some_and(|m| m.contains("二つ目")),
        "{:?}",
        a.message
    );
}

// ---- 低: 書き出しの知らせの順 ----

#[test]
fn test_sr_20_to_base_display_note_is_last() {
    // [SR-20][BV-19] 表示の切り替えを落とした知らせは、落としたものの一覧の最後。
    let mut v = mdgrid::views::NativeView {
        name: "v".into(),
        ..Default::default()
    };
    v.settings.group = Group::By {
        col: "status".into(),
        dir: Dir::Asc,
        hide_empty: true,
    };
    v.settings.display.row_numbers = Some(true);
    let (_, dropped) = mdgrid::views::to_base(&v);
    assert!(dropped.len() >= 2, "{dropped:?}");
    assert!(
        dropped.last().unwrap().contains("表示の切り替え"),
        "{dropped:?}"
    );
}
