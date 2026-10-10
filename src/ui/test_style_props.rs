//! 部品の形(SR-36)・色(SR-40・SR-41)の性質の試験: どんな値・幅・高さ・形・色の設定でも、落ちず、
//! 表の行の幅が画面の幅を超えない(札・丸い端・+N・全角の字・絵文字・空の値を含む)。

use super::test_screen::{app_of, Tmp};
use super::*;
use mdgrid::style::{Band, Check, Frames, Rules, Select, Status, Tabs, Tags};
use proptest::prelude::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// 値の候補(境目になりやすいもの)。
fn value() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("done".to_string()),
        Just("進行中".to_string()),
        Just("ＡＢＣ".to_string()),
        Just("✈️".to_string()),
        Just("a".repeat(60)),
        "[a-z ]{1,12}",
        "[ぁ-ん]{1,6}",
    ]
}

fn pick(names: &'static [&'static str]) -> impl Strategy<Value = &'static str> {
    proptest::sample::select(names.to_vec())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 96, ..ProptestConfig::default() })]

    #[test]
    fn test_sr_36_any_style_fits_the_screen(
        statuses in proptest::collection::vec(value(), 1..6),
        tags in proptest::collection::vec(proptest::collection::vec(value(), 0..5), 1..6),
        status in pick(Status::NAMES),
        tag in pick(Tags::NAMES),
        check in pick(Check::NAMES),
        select in pick(Select::NAMES),
        rules in pick(Rules::NAMES),
        tabs in pick(Tabs::NAMES),
        frames in pick(Frames::NAMES),
        band in pick(Band::NAMES),
        nerd in any::<bool>(),
        icons in any::<bool>(),
        wide in any::<bool>(),
        w in 8u16..130,
        h in 4u16..40,
        col in 0usize..4,
        open in any::<bool>(),
    ) {
        // [SR-36][SR-40][SR-41] 札を作る値(くり返す status)とリスト、どの形でも、表の行は幅 w に収まる。
        let tmp = Tmp::new("sr36_props");
        let n = statuses.len().max(tags.len());
        for i in 0..n {
            let s = &statuses[i % statuses.len()];
            let t = &tags[i % tags.len()];
            let list = t.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>().join(", ");
            tmp.write(
                &format!("n{i}.md"),
                &format!("---\nstatus: {s:?}\ntags: [{list}]\ndone: {}\nx: {i}\n---\n", i % 2 == 0),
            );
        }
        let mut a = app_of(&tmp, ColorMode::Rgb);
        let cfg = format!(
            "nerd_font = {nerd}\nambiguous_wide = {wide}\n\
             [style]\nstatus = \"{status}\"\ntags = \"{tag}\"\ncheck = \"{check}\"\nselect = \"{select}\"\n\
             rules = \"{rules}\"\ntabs = \"{tabs}\"\nframes = \"{frames}\"\nband = \"{band}\"\nicons = {icons}\n\
             [colors]\naccent = \"#e0a458\"\n[colors.values]\ndone = \"green\"\n\"進行中\" = \"#ff3366\"\n"
        );
        let (c, warn) = mdgrid::config::parse(&cfg).unwrap();
        prop_assert!(warn.is_empty(), "{:?}", warn);
        a.configure(&c);
        a.refresh_if_needed();
        a.resize(w, h);
        a.col = col.min(a.cols.len().saturating_sub(1));
        if open {
            // 候補の窓も開いて描く(status の列で Enter)。
            if let Some(j) = a.cols.iter().position(|c| c == "status") {
                a.col = j;
                super::test_screen::press(&mut a, ratatui::crossterm::event::KeyCode::Enter);
            }
        }
        for line in view::render(&a, w as usize, h as usize) {
            prop_assert!(line.width() <= w as usize, "{} > {w}: {:?}", line.width(), line);
        }
        // 画面全体も落ちずに描ける。
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        t.draw(|f| super::draw(f, &a)).unwrap();
    }
}
