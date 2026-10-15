//! Windows の置き場と既定のエディタ(CLI-3・SR-8。specs/_changes/2026-10-15-windows-target.md)。
//! Windows かどうかと環境変数を引数で渡すので、どの OS でも Windows の分岐を確かめられる。

use super::*;
use std::ffi::OsString;

fn env_of(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<OsString> {
    move |k| {
        pairs
            .iter()
            .find(|(n, _)| *n == k)
            .map(|(_, v)| OsString::from(v))
    }
}

const ROAMING: &str = r"C:\Users\a\AppData\Roaming";
const LOCAL: &str = r"C:\Users\a\AppData\Local";

#[test]
fn test_cli_3_windows_config_in_appdata() {
    // [CLI-3] Windows で XDG_CONFIG_HOME が無ければ %APPDATA%\mdgrid\config.toml。HOME があっても使わない。
    let env = env_of(&[("APPDATA", ROAMING), ("HOME", "/home/a")]);
    assert_eq!(
        config_path_in(&env, true),
        Some(PathBuf::from(ROAMING).join("mdgrid").join("config.toml"))
    );
    // Windows でない環境では今までどおり ~/.config。
    assert_eq!(
        config_path_in(&env, false),
        Some(PathBuf::from("/home/a/.config/mdgrid/config.toml"))
    );
}

#[test]
fn test_cli_3_windows_xdg_first_and_none() {
    // [CLI-3] Windows でも絶対パスの XDG_CONFIG_HOME が先。相対の値は無視して APPDATA。
    let abs = std::env::temp_dir().join("xdg");
    let abs_s: &'static str = Box::leak(abs.to_string_lossy().into_owned().into_boxed_str());
    let pairs: &'static [(&'static str, &'static str)] =
        Box::leak(vec![("XDG_CONFIG_HOME", abs_s), ("APPDATA", ROAMING)].into_boxed_slice());
    assert_eq!(
        config_path_in(&env_of(pairs), true),
        Some(abs.join("mdgrid").join("config.toml"))
    );
    let rel = env_of(&[("XDG_CONFIG_HOME", "rel"), ("APPDATA", ROAMING)]);
    assert_eq!(
        config_path_in(&rel, true),
        Some(PathBuf::from(ROAMING).join("mdgrid").join("config.toml"))
    );
    // どれも無い・空 → 置き場なし(既定で動く)。
    assert_eq!(config_path_in(&env_of(&[]), true), None);
    assert_eq!(config_path_in(&env_of(&[("APPDATA", "")]), true), None);
}

#[test]
fn test_cli_3_windows_state_in_localappdata() {
    // [CLI-3] [SR-12] 見た目の状態は Windows では %LOCALAPPDATA%\mdgrid\(端末ごと。ローミングしない)。
    let env = env_of(&[("APPDATA", ROAMING), ("LOCALAPPDATA", LOCAL)]);
    assert_eq!(
        state_dir_in(&env, true),
        Some(PathBuf::from(LOCAL).join("mdgrid"))
    );
    assert_eq!(state_dir_in(&env_of(&[("APPDATA", ROAMING)]), true), None);
    assert_eq!(
        state_dir_in(&env_of(&[("HOME", "/home/a")]), false),
        Some(PathBuf::from("/home/a/.local/state/mdgrid"))
    );
}

#[test]
fn test_cli_3_windows_home_falls_back_to_userprofile() {
    // [CLI-3] `~` のホームは HOME、無ければ Windows では USERPROFILE。Windows でなければ USERPROFILE は見ない。
    let up = env_of(&[("USERPROFILE", r"C:\Users\a")]);
    assert_eq!(home_dir_in(&up, true), Some(PathBuf::from(r"C:\Users\a")));
    assert_eq!(home_dir_in(&up, false), None);
    let both = env_of(&[("HOME", "/h"), ("USERPROFILE", r"C:\Users\a")]);
    assert_eq!(home_dir_in(&both, true), Some(PathBuf::from("/h")));
    let empty_home = env_of(&[("HOME", ""), ("USERPROFILE", r"C:\Users\a")]);
    assert_eq!(
        home_dir_in(&empty_home, true),
        Some(PathBuf::from(r"C:\Users\a"))
    );
}

#[test]
fn test_sr_8_default_editor_per_os() {
    // [SR-8] どれも無いときのエディタは vi、Windows では notepad。
    assert_eq!(default_editor_for(false), "vi");
    assert_eq!(default_editor_for(true), "notepad");
    assert_eq!(DEFAULT_EDITOR, default_editor_for(cfg!(windows)));
    assert_eq!(resolve_editor(None, None, None), DEFAULT_EDITOR);
    // 何かあればそれが先(Windows の %EDITOR% も同じ)。
    assert_eq!(resolve_editor(None, None, Some("code -w")), "code -w");
}
