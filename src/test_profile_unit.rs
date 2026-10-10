//! 範囲ごとの重ね(SR-44)と、警告の形(CLI-3)。

use super::*;
use crate::style::Status;

fn layer(place: Place, label: &str, toml: &str) -> Layer {
    let t: toml::Table = toml.parse().unwrap();
    let mut w = Vec::new();
    let profile = Profile::read_scoped(&t, label, "", &[], &mut w);
    assert!(w.is_empty(), "{w:?}");
    Layer {
        origin: Origin::new(place, label),
        profile,
    }
}

fn resolve_all(layers: &[Layer]) -> Resolved {
    let mut w = Vec::new();
    let r = resolve(layers, &Vec::new(), &mut w);
    assert!(w.is_empty(), "{w:?}");
    r
}

#[test]
fn test_sr_44_narrow_place_wins() {
    // [SR-44] 狭い範囲が勝つ: 全体 sumi、ワークスペース nord、表 dracula → dracula。表が無ければ nord。
    let global = layer(Place::Config, "config.toml", "[look]\ntheme = \"sumi\"\n");
    let ws = layer(Place::Workspace, "ws", "[look]\ntheme = \"nord\"\n");
    let table = layer(
        Place::TableApp,
        "views.toml",
        "[look]\ntheme = \"dracula\"\n",
    );
    let r = resolve_all(&[global.clone(), ws.clone(), table]);
    assert_eq!(r.theme, ThemeSpec::Named(Theme::Dracula));
    assert_eq!(r.origin("look.theme"), Place::TableApp);
    let r = resolve_all(&[global.clone(), ws]);
    assert_eq!(r.theme, ThemeSpec::Named(Theme::Nord));
    assert_eq!(r.origin("look.theme"), Place::Workspace);
    let r = resolve_all(&[global]);
    assert_eq!(r.theme, ThemeSpec::Named(Theme::Sumi));
    assert_eq!(resolve_all(&[]).origin("look.theme"), Place::Default);
}

#[test]
fn test_sr_44_preset_drops_wider_parts_and_theme_drops_wider_roles() {
    // [SR-44] 組を書いた範囲は、広い範囲の部品の形を使わない。テーマを書いた範囲は、広い範囲の役割の色を使わない
    // (値の色は使う)。同じ範囲の部品の形と役割の色は使う。
    let global = layer(
        Place::Config,
        "config.toml",
        "[look.style]\nstatus = \"chip\"\n[look.colors]\naccent = \"orange\"\n[look.colors.values]\ndone = \"green\"\n",
    );
    let ws = layer(
        Place::Workspace,
        "ws",
        "[look]\npreset = \"saas\"\ntheme = \"nord\"\n",
    );
    let r = resolve_all(&[global.clone(), ws]);
    assert_eq!(
        r.style,
        Style::of(Preset::Saas),
        "広い範囲の status = chip は使わない"
    );
    assert_eq!(
        r.colors.role("accent"),
        None,
        "広い範囲の accent は使わない"
    );
    assert!(r.colors.value("done").is_some(), "値の色は使う");
    let same = layer(
        Place::Workspace,
        "ws",
        "[look]\npreset = \"saas\"\ntheme = \"nord\"\n[look.style]\nstatus = \"chip\"\n[look.colors]\naccent = \"orange\"\n",
    );
    let r = resolve_all(&[global.clone(), same]);
    assert_eq!(r.style.status, Status::Chip, "同じ範囲の部品の形は使う");
    assert!(
        r.colors.role("accent").is_some(),
        "同じ範囲の役割の色は使う"
    );
    // 組もテーマも書かない狭い範囲は、広い範囲の部品の形と役割の色をそのまま重ねる。
    let display_only = layer(Place::TableApp, "t", "[display]\nzebra = true\n");
    let r = resolve_all(&[global, display_only]);
    assert_eq!(r.style.status, Status::Chip);
    assert!(r.colors.role("accent").is_some() && r.display.zebra);
}

#[test]
fn test_sr_44_template_is_laid_under_the_place() {
    // [SR-44] use はテンプレートをその範囲の値の下に敷く。無い名前とテンプレートの中の use は警告。
    let mut night = Profile::default();
    night.look.theme = Some(ThemeSpec::Named(Theme::Dracula));
    night.look.preset = Some(Preset::DozyPink);
    let templates = vec![("夜".to_string(), night)];
    let t = layer(
        Place::TableHand,
        "ws: tasks",
        "use = \"夜\"\n[look]\npreset = \"grid\"\n",
    );
    let r = resolve(&[t], &templates, &mut Vec::new());
    assert_eq!(r.theme, ThemeSpec::Named(Theme::Dracula));
    assert_eq!(
        r.preset,
        Preset::Grid,
        "その範囲に書いた値がテンプレートより先"
    );
    assert_eq!(
        r.origins["look.theme"].template.as_deref(),
        Some("夜"),
        "出どころにテンプレートの名前"
    );
    let mut w = Vec::new();
    resolve(
        &[layer(Place::Config, "config.toml", "use = \"無い\"\n")],
        &templates,
        &mut w,
    );
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].starts_with("config.toml: use: "), "{w:?}");
}

#[test]
fn test_sr_44_global_items_are_refused_in_other_places() {
    // [SR-44] アプリ全体の項目はほかの範囲に書けば警告して無視する。知らない項目も警告。
    let t: toml::Table =
        "language = \"en\"\nfoo = 1\n[terminal]\ncolor = false\n[look]\ntheme = \"nord\"\n"
            .parse()
            .unwrap();
    let mut w = Vec::new();
    let p = Profile::read_scoped(&t, ".mdgrid/workspace.toml", "", &[], &mut w);
    assert_eq!(p.look.theme, Some(ThemeSpec::Named(Theme::Nord)));
    assert_eq!(w.len(), 3, "{w:?}");
    assert!(
        w.iter().all(|m| m.starts_with(".mdgrid/workspace.toml: ")),
        "{w:?}"
    );
    assert!(w.iter().any(|m| m.contains(": terminal: ")), "{w:?}");
}

#[test]
fn test_sr_44_profile_round_trips() {
    // [SR-44] 画面が書くプロファイルは、書いて読むと同じ。
    let mut p = Profile::default();
    p.use_ = Some("夜".into());
    p.look.theme = Some(ThemeSpec::Pair {
        light: Theme::Paper,
        dark: Theme::SaasDark,
    });
    p.look.preset = Some(Preset::Grid);
    p.look.style.links = Some(crate::style::Links::Plain);
    p.look
        .columns
        .insert("状態".into(), crate::cells::ColStyle::Chip);
    p.look.roles[7] = Some([1, 2, 3]);
    p.look.values.push(("done".into(), [4, 5, 6]));
    p.display.tabs = Some(TabsMode::Never);
    p.dates.week_start = Some(WeekStart::Mon);
    p.edit.candidates = Some(3);
    let t = p.to_table();
    let mut w = Vec::new();
    let back = Profile::read_scoped(&t, "x", "", &[], &mut w);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(back, p);
}

#[test]
fn test_cli_21_resolved_toml_names_the_origin() {
    // [CLI-21] 決まった値の文は、各項目の上に出どころのコメント。読み直せる。
    let ws = layer(
        Place::Workspace,
        "N (.mdgrid/workspace.toml)",
        "[look]\ntheme = \"nord\"\n",
    );
    let r = resolve_all(&[ws]);
    let text = resolved_toml(&r);
    assert!(
        text.contains("# N (.mdgrid/workspace.toml)\ntheme = \"nord\""),
        "{text}"
    );
    assert!(text.contains("# default\npreset = \"sumi\""), "{text}");
    let (_, w) = crate::config::parse(&text).unwrap();
    assert!(w.is_empty(), "{w:?}");
}
