use super::*;
use std::collections::HashSet;

#[test]
fn test_cli_12_keys_item_lists_every_mode() {
    // [CLI-12] 設定の項目の表の keys の説明(--print-config と文書の元)に、どのモードの名前もある。
    let item = mdgrid::config::ITEMS
        .iter()
        .find(|i| i.path == "keys")
        .unwrap();
    for m in Mode::ALL {
        assert!(item.en.contains(m.name()), "en に {} が無い", m.name());
        assert!(item.ja.contains(m.name()), "ja に {} が無い", m.name());
    }
}

#[test]
fn test_sr_16_no_duplicate_keys_per_mode() {
    // [SR-16] 同じモードの中で1つのキーに2つの動作を割り当てない。
    let mut seen = HashSet::new();
    for b in BINDINGS {
        assert!(seen.insert((b.mode, b.key)), "重複: {:?} {}", b.mode, b.key);
    }
    // [SR-16] 前置きのキー(`g`)は、それだけで動作を持たない。
    for b in BINDINGS {
        assert!(
            !is_prefix(BINDINGS, b.mode, b.key),
            "前置きと重なる: {}",
            b.key
        );
    }
}

#[test]
fn test_sr_4_one_table_drives_dispatch_and_hints() {
    // [SR-4] 振り分けと下の帯は同じ表から作る: 下の帯に出る動作は表で引ける。
    for b in BINDINGS {
        assert_eq!(lookup(BINDINGS, b.mode, b.key), Some(b.action));
        if b.rank > 0 {
            assert!(hints(BINDINGS, b.mode).iter().any(|h| h.ends_with(b.label)));
        }
    }
}

#[test]
fn test_sr_13_default_keys() {
    // [SR-13] vim 風と矢印の両方、Bases の主なキー。
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "j"),
        lookup(BINDINGS, Mode::Table, "Down")
    );
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "k"),
        lookup(BINDINGS, Mode::Table, "Up")
    );
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "h"),
        lookup(BINDINGS, Mode::Table, "Left")
    );
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "l"),
        lookup(BINDINGS, Mode::Table, "Right")
    );
    assert_eq!(lookup(BINDINGS, Mode::Table, "g g"), Some(Action::Top));
    assert_eq!(lookup(BINDINGS, Mode::Table, "G"), Some(Action::Bottom));
    assert_eq!(lookup(BINDINGS, Mode::Table, "Tab"), Some(Action::NextCell));
    assert_eq!(lookup(BINDINGS, Mode::Table, "Ctrl+z"), Some(Action::Undo));
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Backspace"),
        Some(Action::Clear)
    );
    // [SR-18] 読み込みの中止は Ctrl+G(Ctrl+C ではない)。
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Ctrl+g"),
        Some(Action::CancelLoad)
    );
    // [OUT-1] Ctrl+C はコピー。
    assert_eq!(lookup(BINDINGS, Mode::Table, "Ctrl+c"), Some(Action::Copy));
    // [SR-13] 検索の `/` `n` `N`。[NV-2] `\`、[NV-7] `0` `$` Ctrl+D Ctrl+U。
    assert_eq!(lookup(BINDINGS, Mode::Table, "/"), Some(Action::Search));
    assert_eq!(lookup(BINDINGS, Mode::Table, "n"), Some(Action::SearchNext));
    assert_eq!(lookup(BINDINGS, Mode::Table, "N"), Some(Action::SearchPrev));
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "\\"),
        Some(Action::QuickFilter)
    );
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "0"),
        Some(Action::FirstColumn)
    );
    assert_eq!(lookup(BINDINGS, Mode::Table, "$"), Some(Action::LastColumn));
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Ctrl+d"),
        Some(Action::HalfPageDown)
    );
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Ctrl+u"),
        Some(Action::HalfPageUp)
    );
    // [SR-18] Ctrl+A 全部を選ぶ、Esc 選択を解く、`*` 強調、`,` 同じ値の行だけ。
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Ctrl+a"),
        Some(Action::SelectAll)
    );
    assert_eq!(lookup(BINDINGS, Mode::Table, "Esc"), Some(Action::Escape));
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "*"),
        Some(Action::HighlightSame)
    );
    assert_eq!(lookup(BINDINGS, Mode::Table, ","), Some(Action::FilterSame));
    // [NV-5] Shift+矢印は範囲。
    let shift_down = KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT);
    assert_eq!(key_name(&shift_down), Some("Shift+Down".into()));
    // [SR-16] 編集で Ctrl+R は編集前の値、表ではやり直し。
    assert_eq!(lookup(BINDINGS, Mode::Edit, "Ctrl+r"), Some(Action::Revert));
    assert_eq!(lookup(BINDINGS, Mode::Table, "Ctrl+r"), Some(Action::Redo));
}

#[test]
fn test_sr_16_edit_mode_leaves_letters_for_input() {
    // [SR-16] [CE-1] 編集のモードとパレットには文字1つのキーを割り当てない(打った文字は入力に入る)。
    for b in BINDINGS.iter().filter(|b| {
        matches!(
            b.mode,
            Mode::Edit | Mode::Palette | Mode::Search | Mode::Filter
        )
    }) {
        assert!(b.key.chars().count() > 1, "文字のキー: {}", b.key);
    }
}

#[test]
fn test_sr_17_fullwidth_keys() {
    // [SR-17] 全角の ｊ は j、`、` は `,`、`・` は `/`。
    let k = |c| key_name(&KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    assert_eq!(k('ｊ'), Some("j".into()));
    assert_eq!(k('Ｇ'), Some("G".into()));
    assert_eq!(k('、'), Some(",".into()));
    assert_eq!(k('・'), Some("/".into()));
    let shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
    assert_eq!(key_name(&shift_g), Some("G".into()));
    let ctrl_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL);
    assert_eq!(key_name(&ctrl_r), Some("Ctrl+r".into()));
    let ctrl_shift_z = KeyEvent::new(
        KeyCode::Char('Z'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    assert_eq!(key_name(&ctrl_shift_z), Some("Ctrl+Shift+z".into()));
}

#[test]
fn test_sr_16_palette_and_help_keys() {
    // [SR-16] `:` と Ctrl+P はパレット、`?` はヘルプ。$EDITOR は `e`、コピーは `y` と Ctrl+C(OUT-1)。
    assert_eq!(lookup(BINDINGS, Mode::Table, ":"), Some(Action::Palette));
    assert_eq!(
        lookup(BINDINGS, Mode::Table, "Ctrl+p"),
        Some(Action::Palette)
    );
    assert_eq!(lookup(BINDINGS, Mode::Table, "?"), Some(Action::Help));
    assert_eq!(lookup(BINDINGS, Mode::Table, "e"), Some(Action::OpenEditor));
    assert_eq!(lookup(BINDINGS, Mode::Table, "y"), Some(Action::Copy));
    // [SR-14] 動作の名前はパレットで探す鍵なので、動作ごとに違う。
    let actions: HashSet<Action> = BINDINGS.iter().map(|b| b.action).collect();
    let names: HashSet<&str> = actions.iter().map(|a| a.name()).collect();
    assert_eq!(actions.len(), names.len());
}
