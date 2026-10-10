//! 画面で選んだ全体の設定 ui.toml(SR-43・CLI-3)と、前の版の look.toml の読み取り(CLI-20)。

use crate::config::{Config, NerdFont};
use crate::profile::{self, Profile, ThemeSpec};
use crate::style::{Preset, Style};
use crate::theme::Theme;
use crate::uifile::{self, UiFile};
use std::path::PathBuf;

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-ui-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn night() -> Profile {
    let mut p = Profile::default();
    p.look.theme = Some(ThemeSpec::Named(Theme::Dracula));
    p.look.preset = Some(Preset::DozyPink);
    p
}

#[test]
fn test_sr_43_ui_round_trip_with_templates() {
    // [SR-43] 全体のプロファイル・丸い札の端・テンプレートを書いて読むと同じ。警告なし。空にするとファイルを消す。
    let d = dir("rt");
    let mut profile = Profile::default();
    profile.look.theme = Some(ThemeSpec::AUTO);
    profile.look.preset = Some(Preset::Paper);
    profile.display.row_numbers = Some(true);
    let f = UiFile {
        profile,
        nerd_font: Some(NerdFont::On),
        templates: vec![("夜".into(), night()), ("昼".into(), Profile::default())],
    };
    uifile::save(&d, &f).unwrap();
    let text = std::fs::read_to_string(d.join(uifile::FILE_NAME)).unwrap();
    assert!(text.contains("theme = \"auto\""), "{text}");
    assert!(text.contains("nerd_font = true"), "{text}");
    assert!(text.contains("[templates"), "{text}");
    let (back, w) = uifile::load(&d);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(back.profile, f.profile);
    assert_eq!(back.nerd_font, f.nerd_font);
    assert_eq!(back.templates.len(), 2);
    assert_eq!(profile::template(&back.templates, "夜"), Some(&night()));
    uifile::save(&d, &UiFile::default()).unwrap();
    assert!(!d.join(uifile::FILE_NAME).exists());
    assert_eq!(uifile::load(&d), (UiFile::default(), Vec::new()));
}

#[test]
fn test_sr_43_ui_broken_and_bad_values_warn() {
    // [SR-43] 壊れた ui.toml は警告して空。知らない項目と使えない値は「ui.toml: 道筋: 理由」の警告にして外す。
    let d = dir("bad");
    std::fs::write(d.join(uifile::FILE_NAME), "use = \n").unwrap();
    let (f, w) = uifile::load(&d);
    assert_eq!(f, UiFile::default());
    assert_eq!(w.len(), 1, "{w:?}");
    std::fs::write(
        d.join(uifile::FILE_NAME),
        "color = 1\n[look]\ntheme = \"neon\"\npreset = \"dozy-pink\"\n[terminal]\nnerd_font = 3\ncolor = false\n",
    )
    .unwrap();
    let (f, w) = uifile::load(&d);
    assert_eq!(f.profile.look.preset, Some(Preset::DozyPink));
    assert_eq!(f.profile.look.theme, None);
    assert_eq!(f.nerd_font, None);
    assert_eq!(w.len(), 4, "{w:?}");
    assert!(
        w.iter().all(|m| m.starts_with("ui.toml: ")),
        "どの警告もファイルの名前から: {w:?}"
    );
}

#[test]
fn test_cli_20_old_look_toml_is_read_and_moved() {
    // [CLI-20] ui.toml が無ければ前の版の look.toml を読む。書くと ui.toml に移して look.toml を消す。
    let d = dir("old");
    std::fs::write(
        d.join(uifile::OLD_FILE_NAME),
        "theme = \"dracula\"\npreset = \"dozy-pink\"\nnerd_font = true\n[[template]]\nname = \"夜\"\ntheme = \"nord\"\n[[template]]\ntheme = \"paper\"\n",
    )
    .unwrap();
    let (f, w) = uifile::load(&d);
    assert_eq!(w.len(), 1, "名前の無いテンプレート: {w:?}");
    assert_eq!(f.profile.look.theme, Some(ThemeSpec::Named(Theme::Dracula)));
    assert_eq!(f.profile.look.preset, Some(Preset::DozyPink));
    assert_eq!(f.nerd_font, Some(NerdFont::On));
    assert_eq!(f.templates.len(), 1);
    uifile::save(&d, &f).unwrap();
    assert!(d.join(uifile::FILE_NAME).exists());
    assert!(!d.join(uifile::OLD_FILE_NAME).exists());
    assert_eq!(uifile::load(&d).0, f);
}

#[test]
fn test_sr_43_ui_layer_overrides_config() {
    // [SR-43][SR-44] ui.toml の層は config.toml の上に重なる。組を書けば config.toml の部品ごとの形は使わない。
    let (c, _) =
        crate::config::parse("[look]\ntheme = \"sumi\"\n[look.style]\nstatus = \"text\"\n")
            .unwrap();
    let ui = UiFile {
        profile: night(),
        ..UiFile::default()
    };
    let r = profile::resolve(&[c.layer(), ui.layer()], &Vec::new(), &mut Vec::new());
    assert_eq!(r.theme, ThemeSpec::Named(Theme::Dracula));
    assert_eq!(r.style, Style::of(Preset::DozyPink));
    assert_eq!(r.origin("look.theme"), profile::Place::Ui);
    let r = profile::resolve(
        &[c.layer(), UiFile::default().layer()],
        &Vec::new(),
        &mut Vec::new(),
    );
    assert_eq!(r.style.status, crate::style::Status::Text);
    assert_eq!(r.origin("look.theme"), profile::Place::Config);
    let _ = Config::default();
}
