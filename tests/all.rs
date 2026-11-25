//! 結合の試験を1つの実行ファイルにまとめる。tests/ の各ファイルは今の場所のまま `#[path]` で取り込む。
//! ファイルごとに実行ファイルを作ると、リンクと、macOS が新しい実行ファイルを初めて起動するときの検査が
//! ファイルの数だけ掛かる(1つで最大 120 秒。docs/design.md「試験の実行ファイル」)。環境変数を書き換える
//! tests/test_config.rs は、ほかの試験とぶつからないように別の実行ファイルにする(Cargo.toml の [[test]])。
//! Cargo.toml は `autotests = false` なので、新しい試験のファイルはここに足す(足し忘れは下の試験が落とす)。

#![allow(clippy::disallowed_methods)]

// 疑似端末の道具(tests/pty/mod.rs)は、ここで1回だけ読み、各ファイルは `use crate::pty;` で使う。
mod pty;

#[path = "test_across_e2e.rs"]
mod test_across_e2e;
#[path = "test_action_menu_e2e.rs"]
mod test_action_menu_e2e;
#[path = "test_add_frontmatter.rs"]
mod test_add_frontmatter;
#[path = "test_agent_skill.rs"]
mod test_agent_skill;
#[path = "test_apply.rs"]
mod test_apply;
#[path = "test_apply_review.rs"]
mod test_apply_review;
#[path = "test_base.rs"]
mod test_base;
#[path = "test_bases_docs.rs"]
mod test_bases_docs;
#[path = "test_catalog.rs"]
mod test_catalog;
#[path = "test_catalog_colors.rs"]
mod test_catalog_colors;
#[path = "test_catalog_nerd.rs"]
mod test_catalog_nerd;
#[path = "test_changes.rs"]
mod test_changes;
#[path = "test_chart_docs.rs"]
mod test_chart_docs;
#[path = "test_chart_e2e.rs"]
mod test_chart_e2e;
#[path = "test_chart_fix_e2e.rs"]
mod test_chart_fix_e2e;
#[path = "test_check_cli.rs"]
mod test_check_cli;
#[path = "test_check_cli_more.rs"]
mod test_check_cli_more;
#[path = "test_cli_polish.rs"]
mod test_cli_polish;
#[path = "test_cliff.rs"]
mod test_cliff;
#[path = "test_column_types_config.rs"]
mod test_column_types_config;
#[path = "test_completions.rs"]
mod test_completions;
#[path = "test_condition_e2e.rs"]
mod test_condition_e2e;
#[path = "test_config_docs.rs"]
mod test_config_docs;
#[path = "test_config_docs_values.rs"]
mod test_config_docs_values;
#[path = "test_config_v2.rs"]
mod test_config_v2;
#[path = "test_csv_apply.rs"]
mod test_csv_apply;
#[path = "test_csv_e2e.rs"]
mod test_csv_e2e;
#[path = "test_date_format.rs"]
mod test_date_format;
#[path = "test_date_unquoted.rs"]
mod test_date_unquoted;
#[path = "test_date_words_e2e.rs"]
mod test_date_words_e2e;
#[path = "test_default_headings.rs"]
mod test_default_headings;
#[path = "test_display_options.rs"]
mod test_display_options;
#[path = "test_docs_order.rs"]
mod test_docs_order;
#[path = "test_e2e.rs"]
mod test_e2e;
#[path = "test_editor_setting.rs"]
mod test_editor_setting;
#[path = "test_empty_frontmatter.rs"]
mod test_empty_frontmatter;
#[path = "test_example_ai_human.rs"]
mod test_example_ai_human;
#[path = "test_example_studio.rs"]
mod test_example_studio;
#[path = "test_examples.rs"]
mod test_examples;
#[path = "test_expr.rs"]
mod test_expr;
#[path = "test_features_page.rs"]
mod test_features_page;
#[path = "test_flow_frontmatter.rs"]
mod test_flow_frontmatter;
#[path = "test_formula_mark.rs"]
mod test_formula_mark;
#[path = "test_frontmatter.rs"]
mod test_frontmatter;
#[path = "test_gantt_docs.rs"]
mod test_gantt_docs;
#[path = "test_gantt_e2e.rs"]
mod test_gantt_e2e;
#[path = "test_group_values.rs"]
mod test_group_values;
#[path = "test_hwheel_e2e.rs"]
mod test_hwheel_e2e;
#[path = "test_i18n.rs"]
mod test_i18n;
#[path = "test_install_docs.rs"]
mod test_install_docs;
#[path = "test_kanban_docs.rs"]
mod test_kanban_docs;
#[path = "test_kanban_e2e.rs"]
mod test_kanban_e2e;
#[path = "test_language.rs"]
mod test_language;
#[path = "test_language_lib.rs"]
mod test_language_lib;
#[path = "test_ledger_conditions_e2e.rs"]
mod test_ledger_conditions_e2e;
#[path = "test_links.rs"]
mod test_links;
#[path = "test_list_write.rs"]
mod test_list_write;
#[path = "test_locales.rs"]
mod test_locales;
#[path = "test_locales_e2e.rs"]
mod test_locales_e2e;
#[path = "test_look_e2e.rs"]
mod test_look_e2e;
#[path = "test_marker_base.rs"]
mod test_marker_base;
#[path = "test_marker_base_name.rs"]
mod test_marker_base_name;
#[path = "test_marker_base_switch.rs"]
mod test_marker_base_switch;
#[path = "test_marker_open.rs"]
mod test_marker_open;
#[path = "test_modes_e2e.rs"]
mod test_modes_e2e;
#[path = "test_new_note.rs"]
mod test_new_note;
#[path = "test_new_note_cli.rs"]
mod test_new_note_cli;
#[path = "test_no_japanese_literals.rs"]
mod test_no_japanese_literals;
#[path = "test_not_text_files.rs"]
mod test_not_text_files;
#[path = "test_note_form_e2e.rs"]
mod test_note_form_e2e;
#[path = "test_note_name.rs"]
mod test_note_name;
#[path = "test_note_path.rs"]
mod test_note_path;
#[path = "test_note_trash_e2e.rs"]
mod test_note_trash_e2e;
#[path = "test_notice_e2e.rs"]
mod test_notice_e2e;
#[path = "test_notify_e2e.rs"]
mod test_notify_e2e;
#[path = "test_open_md_file.rs"]
mod test_open_md_file;
#[path = "test_pages.rs"]
mod test_pages;
#[path = "test_pages_blog.rs"]
mod test_pages_blog;
#[path = "test_pages_check.rs"]
mod test_pages_check;
#[path = "test_pages_images.rs"]
mod test_pages_images;
#[path = "test_paste_e2e.rs"]
mod test_paste_e2e;
#[path = "test_pick.rs"]
mod test_pick;
#[path = "test_places_e2e.rs"]
mod test_places_e2e;
#[path = "test_poll_round_robin.rs"]
mod test_poll_round_robin;
#[path = "test_print.rs"]
mod test_print;
#[path = "test_print_filter_sort.rs"]
mod test_print_filter_sort;
#[path = "test_print_group_order.rs"]
mod test_print_group_order;
#[path = "test_print_tsv.rs"]
mod test_print_tsv;
#[path = "test_print_with_path.rs"]
mod test_print_with_path;
#[path = "test_readonly_permission.rs"]
mod test_readonly_permission;
#[path = "test_recent_e2e.rs"]
mod test_recent_e2e;
#[path = "test_relations_e2e.rs"]
mod test_relations_e2e;
#[path = "test_relmap_names_e2e.rs"]
mod test_relmap_names_e2e;
#[path = "test_resume_e2e.rs"]
mod test_resume_e2e;
#[path = "test_review6.rs"]
mod test_review6;
#[path = "test_review_fixes.rs"]
mod test_review_fixes;
#[path = "test_root_folder.rs"]
mod test_root_folder;
#[path = "test_routine_e2e.rs"]
mod test_routine_e2e;
#[path = "test_row_lines_config.rs"]
mod test_row_lines_config;
#[path = "test_row_lines_views.rs"]
mod test_row_lines_views;
#[path = "test_rules_config.rs"]
mod test_rules_config;
#[path = "test_safety_docs.rs"]
mod test_safety_docs;
#[path = "test_scalar_list_edit.rs"]
mod test_scalar_list_edit;
#[path = "test_scalar_list_expr.rs"]
mod test_scalar_list_expr;
#[path = "test_session_e2e.rs"]
mod test_session_e2e;
#[path = "test_settings.rs"]
mod test_settings;
#[path = "test_settings_sections_e2e.rs"]
mod test_settings_sections_e2e;
#[path = "test_skip_hidden.rs"]
mod test_skip_hidden;
#[path = "test_source.rs"]
mod test_source;
#[path = "test_stack_e2e.rs"]
mod test_stack_e2e;
#[path = "test_starter_cli.rs"]
mod test_starter_cli;
#[path = "test_starter_e2e.rs"]
mod test_starter_e2e;
#[path = "test_starter_examples.rs"]
mod test_starter_examples;
#[path = "test_starter_layer.rs"]
mod test_starter_layer;
#[path = "test_switch_e2e.rs"]
mod test_switch_e2e;
#[path = "test_tag_keys.rs"]
mod test_tag_keys;
#[path = "test_theme_auto_e2e.rs"]
mod test_theme_auto_e2e;
#[path = "test_tree_base_e2e.rs"]
mod test_tree_base_e2e;
#[path = "test_tree_e2e.rs"]
mod test_tree_e2e;
#[path = "test_trust_notices.rs"]
mod test_trust_notices;
#[path = "test_types.rs"]
mod test_types;
#[path = "test_unwritable.rs"]
mod test_unwritable;
#[path = "test_value_labels_e2e.rs"]
mod test_value_labels_e2e;
#[path = "test_vault.rs"]
mod test_vault;
#[path = "test_views.rs"]
mod test_views;
#[path = "test_views_new_note_keys.rs"]
mod test_views_new_note_keys;
#[path = "test_views_page.rs"]
mod test_views_page;
#[path = "test_views_page_more.rs"]
mod test_views_page_more;
#[path = "test_views_skill.rs"]
mod test_views_skill;
#[path = "test_vim_e2e.rs"]
mod test_vim_e2e;
#[path = "test_where_formula.rs"]
mod test_where_formula;
#[path = "test_windows_writeback.rs"]
mod test_windows_writeback;
#[path = "test_workspace_cli.rs"]
mod test_workspace_cli;
#[path = "test_writeback.rs"]
mod test_writeback;
#[path = "test_writeback_faults.rs"]
mod test_writeback_faults;
#[path = "test_writeback_fixes.rs"]
mod test_writeback_fixes;
#[path = "test_writeback_props.rs"]
mod test_writeback_props;
#[path = "test_ws_print.rs"]
mod test_ws_print;

/// tests/ の `test_*.rs` は、どれもこの束(`#[path]`)か、別の実行ファイル(Cargo.toml の `[[test]]`)で走る。
#[test]
fn every_test_file_is_built() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let all = std::fs::read_to_string(dir.join("all.rs")).unwrap();
    let cargo = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
    )
    .unwrap();
    let mut missing = Vec::new();
    for e in std::fs::read_dir(&dir).unwrap() {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        if !(name.starts_with("test_")
            && std::path::Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("rs")))
        {
            continue;
        }
        let bundled = all.contains(&format!("#[path = \"{name}\"]"));
        let alone = cargo.contains(&format!("path = \"tests/{name}\""));
        if !bundled && !alone {
            missing.push(name);
        }
    }
    missing.sort();
    assert!(
        missing.is_empty(),
        "どの試験の実行ファイルにも入っていない: {missing:?}"
    );
}
