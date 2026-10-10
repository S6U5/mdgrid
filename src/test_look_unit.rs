//! 見た目の区画の look.toml(SR-43): 読み書き・テンプレート・壊れたファイル・config への重ね。

use crate::config::Config;
use crate::look::{self, Look, LookFile, Nerd, ThemeChoice};
use crate::style::{Preset, Style};
use crate::theme::Theme;
use std::path::PathBuf;

fn dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("mdgrid-look-{name}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn night() -> Look {
    Look {
        theme: Some(ThemeChoice::Named(Theme::Dracula)),
        preset: Some(Preset::DozyPink),
        nerd: Some(Nerd::On),
    }
}

#[test]
fn test_sr_43_look_round_trip_with_templates() {
    // [SR-43] 見た目とテンプレートを書いて読むと同じ。警告なし。空にするとファイルを消す。
    let d = dir("rt");
    let f = LookFile {
        look: Look {
            theme: Some(ThemeChoice::Auto),
            preset: Some(Preset::Paper),
            nerd: Some(Nerd::Auto),
        },
        templates: vec![("夜".into(), night()), ("昼".into(), Look::default())],
    };
    look::save(&d, &f).unwrap();
    let text = std::fs::read_to_string(d.join(look::FILE_NAME)).unwrap();
    assert!(text.contains("theme = \"auto\""), "{text}");
    assert!(text.contains("nerd_font = true"), "{text}");
    assert_eq!(look::load(&d), (f, Vec::new()));
    look::save(&d, &LookFile::default()).unwrap();
    assert!(!d.join(look::FILE_NAME).exists());
    assert_eq!(look::load(&d), (LookFile::default(), Vec::new()));
}

#[test]
fn test_sr_43_look_broken_and_bad_values_warn() {
    // [SR-43] 壊れた look.toml は警告して空。知らない項目と使えない値は警告して外す。名前の無いテンプレートは読まない。
    let d = dir("bad");
    std::fs::write(d.join(look::FILE_NAME), "theme = \n").unwrap();
    let (f, w) = look::load(&d);
    assert_eq!(f, LookFile::default());
    assert_eq!(w.len(), 1, "{w:?}");
    std::fs::write(
        d.join(look::FILE_NAME),
        "theme = \"neon\"\npreset = \"dozy-pink\"\nnerd_font = 3\ncolor = 1\n[[template]]\ntheme = \"nord\"\n[[template]]\nname = \"a\"\nx = 1\n",
    )
    .unwrap();
    let (f, w) = look::load(&d);
    assert_eq!(
        f.look,
        Look {
            preset: Some(Preset::DozyPink),
            ..Default::default()
        }
    );
    assert_eq!(f.templates.len(), 1);
    assert_eq!(w.len(), 5, "{w:?}");
}

#[test]
fn test_sr_43_apply_overrides_config() {
    // [SR-43] 重ねると config.toml のテーマ・組・丸い札の端に代わる。組を選べば部品ごとの形は組のもの。
    // 空の見た目は何も変えない。
    let mut c = Config::default();
    c.style.status = crate::style::Status::Text;
    let before = c.clone();
    look::apply(&mut c, &Look::default());
    assert_eq!(c.theme, before.theme);
    assert_eq!(c.style, before.style);
    look::apply(&mut c, &night());
    assert_eq!(c.theme, Theme::Dracula);
    assert!(!c.theme_auto);
    assert_eq!(c.style, Style::of(Preset::DozyPink));
    assert!(c.nerd_font && !c.nerd_font_auto);
    look::apply(
        &mut c,
        &Look {
            theme: Some(ThemeChoice::Auto),
            nerd: Some(Nerd::Auto),
            ..Default::default()
        },
    );
    assert!(c.theme_auto && c.nerd_font_auto);
}
