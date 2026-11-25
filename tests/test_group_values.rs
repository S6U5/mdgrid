//! `[rules.<列>]` の `icons`・`labels`(navigation.group-values-icon)の読み取り・警告・重ね・出力の試験。
//! 画面の見出しとカンバンのレーンの題は src/ui/test_group_values.rs。値は作ったもの。

use mdgrid::config::parse;
use mdgrid::print::{self, Format, ViewDef};
use mdgrid::profile::{self, Layer, Origin, Place, Profile};
use mdgrid::settings::{Dir, Group, Settings};
use mdgrid::source::markdown::Markdown;
use mdgrid::source::Source;
use mdgrid::views::NativeView;

/// 受け入れの例の設定: 候補 todo・doing、todo に字 ○、todo と doing に名前。
const ICONS: &str = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nicons = { todo = \"○\" }\nlabels = { todo = \"未着手\", doing = \"進行中\" }\n";

#[test]
fn test_group_values_icon_reads_icons_and_labels_without_warnings() {
    // [utest -> req~navigation.group-values-icon~2] icons と labels を警告なしで読み、値の見せる名前は「字 名前」、
    // 名前だけなら名前、書かない値は無い(値のまま見せる)。
    let (c, w) = parse(ICONS).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let r = c.resolved();
    assert_eq!(r.value_shown("status", "todo").as_deref(), Some("○ 未着手"));
    assert_eq!(r.value_shown("status", "doing").as_deref(), Some("進行中"));
    assert_eq!(r.value_shown("status", "done"), None);
    assert_eq!(r.value_shown("owner", "todo"), None);
}

#[test]
fn test_group_values_icon_unknown_value_warns_in_file_path_form() {
    // [utest -> req~navigation.group-values-icon~2] values に無い blocked の字は、cli.config-file-warning-form の形
    // (`config.toml: rules.status.icons.blocked:`)の警告1行で、その字は使わない。
    let text = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nicons = { todo = \"○\", blocked = \"!\" }\n";
    let (c, w) = parse(text).expect("読める");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].starts_with("config.toml: rules.status.icons.blocked: "),
        "{w:?}"
    );
    let r = c.resolved();
    assert_eq!(r.value_shown("status", "blocked"), None);
    assert_eq!(r.value_shown("status", "todo").as_deref(), Some("○ todo"));
}

#[test]
fn test_group_values_icon_unknown_label_warns_in_file_path_form() {
    // [utest -> req~navigation.group-values-icon~2] labels も同じ: values に無い値の名前は `rules.status.labels.blocked` の警告1行で使わない。
    let text = "[rules.status]\nvalues = [\"todo\"]\nlabels = { blocked = \"止まっている\" }\n";
    let (c, w) = parse(text).expect("読める");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(
        w[0].starts_with("config.toml: rules.status.labels.blocked: "),
        "{w:?}"
    );
    assert_eq!(c.resolved().value_shown("status", "blocked"), None);
}

#[test]
fn test_group_values_icon_wrong_shapes_warn_and_are_ignored() {
    // [utest -> req~cli.config-file-warning-form~2] 表でない icons と、文字列でない名前は、「ファイル: 道筋:」の形の警告で
    // 使わない(ほかの項目は読む)。
    let text = "[rules.status]\nvalues = [\"todo\", \"doing\"]\nicons = \"○\"\nlabels = { todo = 1, doing = \"進行中\" }\n";
    let (c, w) = parse(text).expect("読める");
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(
        w[0].starts_with("config.toml: rules.status.icons: "),
        "{w:?}"
    );
    assert!(
        w[1].starts_with("config.toml: rules.status.labels.todo: "),
        "{w:?}"
    );
    let r = c.resolved();
    assert_eq!(r.value_shown("status", "todo"), None);
    assert_eq!(r.value_shown("status", "doing").as_deref(), Some("進行中"));
}

#[test]
fn test_group_values_icon_narrow_scope_replaces_whole_map() {
    // [utest -> req~cell-edit.rules-scopes~1] 決まりは項目ごとに狭い範囲が勝つ: 狭い範囲に書いた icons は広い範囲の icons に
    // 丸ごと代わり(then と同じ)、狭い範囲に書かない labels は広い範囲のものが残る。
    let (c, w) = parse(ICONS).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let t: toml::Table = "[rules.status]\nicons = { doing = \"◐\" }\n"
        .parse()
        .unwrap();
    let mut w = Vec::new();
    let table = Profile::read_scoped(&t, "views.toml", "", &[], &mut w);
    assert!(w.is_empty(), "{w:?}");
    let layers = [
        c.layer(),
        Layer {
            origin: Origin::new(Place::TableApp, "views.toml (notes)"),
            profile: table,
        },
    ];
    let r = profile::resolve(&layers, &c.templates, &mut w);
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(r.value_shown("status", "todo").as_deref(), Some("未着手"));
    assert_eq!(
        r.value_shown("status", "doing").as_deref(),
        Some("◐ 進行中")
    );
}

#[test]
fn test_group_values_icon_resolved_config_reads_back() {
    // [utest -> req~cli.resolved-config-reads-back~1] `--print-config --resolved` の形に icons と labels が出て、警告なしに
    // 読み直せ、同じ見せ方になる。
    let (c, w) = parse(ICONS).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    let out = profile::resolved_toml(&c.resolved());
    let (back, w) = parse(&out).expect("読み直せる");
    assert!(w.is_empty(), "{w:?}\n{out}");
    let r = back.resolved();
    assert_eq!(r.value_shown("status", "todo").as_deref(), Some("○ 未着手"));
    assert_eq!(r.value_shown("status", "doing").as_deref(), Some("進行中"));
}

/// 一時フォルダ(消える)。
struct Tmp(std::path::PathBuf);

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn test_group_values_icon_print_csv_keeps_values() {
    // [utest -> req~navigation.group-values-icon~2, req~navigation.value-labels-cells~1] 同じ設定で、status でまとめたビューを `--print --format csv` の形にしても、
    // status の値は todo・doing のまま(見せる名前と字は出さない)。
    let (c, w) = parse(ICONS).expect("読める");
    assert!(w.is_empty(), "{w:?}");
    assert_eq!(
        c.resolved().value_shown("status", "todo").as_deref(),
        Some("○ 未着手"),
        "前提: 設定の名前が効いている"
    );
    let dir = std::env::temp_dir().join(format!(
        "mdgrid-group-values-print-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let _t = Tmp(dir.clone());
    std::fs::write(dir.join("alpha.md"), "---\nstatus: todo\n---\n").unwrap();
    std::fs::write(dir.join("beta.md"), "---\nstatus: doing\n---\n").unwrap();
    let mut src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    while !src.load(10_000).done {}
    let nv = NativeView {
        name: "V".into(),
        order: vec!["status".into()],
        settings: Settings {
            group: Group::By {
                col: "status".into(),
                dir: Dir::Asc,
                hide_empty: false,
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let t = print::table(&src, ViewDef::Native(&nv), 0, 0).expect("組める");
    let csv = print::render(&t, Format::Csv);
    let mut lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.first(), Some(&"status"), "{csv}");
    // 行の並びはこの試験では見ない(値だけを見る)。
    let mut values = lines.split_off(1);
    values.sort_unstable();
    assert_eq!(values, vec!["doing", "todo"], "{csv}");
}
