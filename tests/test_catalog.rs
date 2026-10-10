//! カタログ(SR-38): docs/catalog/index.html の名前の表が、本体の設定(src/style.rs・src/theme.rs・
//! src/config.rs)と同じか。カタログは1つのファイルで、外の何にも頼らない。

use mdgrid::config::Config;
use mdgrid::profile::ThemeSpec;
use mdgrid::style::{self, Band, Check, Frames, Preset, Rules, Select, Status, Style, Tags};
use mdgrid::theme::Theme;
use serde_json::Value;

fn catalog() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/catalog/index.html");
    std::fs::read_to_string(p).unwrap()
}

/// カタログの中の名前の表(`<script type="application/json" id="names">`)。
fn names() -> Value {
    let html = catalog();
    let head = r#"<script type="application/json" id="names">"#;
    let from = html.find(head).expect("名前の表が無い") + head.len();
    let to = from + html[from..].find("</script>").unwrap();
    serde_json::from_str(&html[from..to]).unwrap()
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn own(n: &[&str]) -> Vec<String> {
    n.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_sr_38_catalog_matches_config() {
    // [SR-38] カタログの項目と値の名前が本体の設定と同じ(どれかが増減・改名したら落ちる)。
    let n = names();
    let themes: Vec<String> = Theme::ALL.iter().map(|t| t.name().to_string()).collect();
    assert_eq!(strs(&n["themes"]), themes);
    assert_eq!(strs(&n["keys"]), own(style::KEYS));
    let v = &n["values"];
    assert_eq!(strs(&v["preset"]), own(Preset::NAMES));
    assert_eq!(strs(&v["status"]), own(Status::NAMES));
    assert_eq!(strs(&v["tags"]), own(Tags::NAMES));
    assert_eq!(strs(&v["check"]), own(Check::NAMES));
    assert_eq!(strs(&v["select"]), own(Select::NAMES));
    assert_eq!(strs(&v["rules"]), own(Rules::NAMES));
    assert_eq!(strs(&v["tabs"]), own(mdgrid::style::Tabs::NAMES));
    assert_eq!(strs(&v["frames"]), own(Frames::NAMES));
    assert_eq!(strs(&v["band"]), own(Band::NAMES));
    assert_eq!(strs(&v["links"]), own(mdgrid::style::Links::NAMES));
}

#[test]
fn test_sr_38_catalog_presets_match() {
    // [SR-38] カタログの組の中身が Style::of と同じ。既定(組・テーマ・auto の2つ・nerd_font)も同じ。
    let n = names();
    for name in Preset::NAMES {
        let s = Style::of(Preset::parse(name).unwrap());
        let p = &n["presets"][*name];
        let want = [
            ("status", s.status.name()),
            ("tags", s.tags.name()),
            ("check", s.check.name()),
            ("select", s.select.name()),
            ("rules", s.rules.name()),
            ("tabs", s.tabs.name()),
            ("frames", s.frames.name()),
            ("band", s.band.name()),
        ];
        for (k, w) in want {
            assert_eq!(p[k].as_str(), Some(w), "{name}.{k}");
        }
        assert_eq!(p["links"].as_str(), Some(s.links.name()), "{name}.links");
        assert_eq!(p["icons"].as_bool(), Some(s.icons), "{name}.icons");
    }
    let d = &n["defaults"];
    let c = Config::default();
    let r = c.resolved();
    assert_eq!(d["preset"].as_str(), Some(r.preset.name()));
    assert_eq!(d["theme"].as_str(), Some(r.theme.label().as_str()));
    // "auto" の明暗の既定(本体の ThemeSpec::AUTO)。
    let ThemeSpec::Pair { light, dark } = ThemeSpec::AUTO else {
        panic!("auto は明暗の組");
    };
    assert_eq!(d["theme_light"].as_str(), Some(light.name()));
    assert_eq!(d["theme_dark"].as_str(), Some(dark.name()));
    assert_eq!(
        d["nerd_font"].as_bool(),
        Some(c.terminal.nerd_font.resolve(None))
    );
}

#[test]
fn test_sr_38_catalog_is_self_contained() {
    // [SR-38] 1つのファイルで、外の何にも頼らない(外の URL・読み込みが無い)。
    let html = catalog();
    for bad in ["http://", "https://", "<link", " src=", "@import"] {
        assert!(!html.contains(bad), "外の読み込み: {bad}");
    }
}
