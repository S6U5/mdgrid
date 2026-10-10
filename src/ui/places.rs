//! 登録した表(`impl App` の続き。CLI-18・CLI-19): パレットの「この表を登録」(名前 → 分類 → 置き換えの
//! 確かめ)と「登録した表を開く」(分類ごとに並べた一覧)。選んだ表へは、終わる手順(WB-11 の確かめ)を
//! 通してから main が開き直す(`switch_to`)。

use super::app::App;
use super::native_io::Ask;
use super::width::width;
use mdgrid::i18n::Msg;
use mdgrid::places::{self, Place};
use std::path::Path;

/// 一覧の行の印(分類と名前の間)。
const SEP: &str = " › ";

/// パスを短く見せる(ホームのフォルダの下なら `~/…`)。
fn short(p: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) {
        if let Ok(rest) = p.strip_prefix(&home) {
            return Path::new("~").join(rest).to_string_lossy().into_owned();
        }
    }
    p.to_string_lossy().into_owned()
}

/// 一覧の行の頭(「分類 › 名前」か「名前」)。
fn head(p: &Place) -> String {
    if p.group.is_empty() {
        p.name.clone()
    } else {
        format!("{}{SEP}{}", p.group, p.name)
    }
}

/// 一覧の行(CLI-19): 「分類 › 名前   パス · ビュー」。分類・名前・パスのどれでも絞れるように全部を並べる。
/// 頭の欄を一覧の中のいちばん広いものにそろえ、パスを同じ桁から出す。
pub(crate) fn labels(list: &[&Place]) -> Vec<String> {
    let w = list.iter().map(|p| width(&head(p))).max().unwrap_or(0);
    list.iter()
        .map(|p| {
            let h = head(p);
            let pad = " ".repeat(w - width(&h));
            let mut s = format!("{h}{pad}   {}", short(&p.path));
            if let Some(v) = &p.view {
                s.push_str(" · ");
                s.push_str(v);
            }
            s
        })
        .collect()
}

/// 分類の欄の候補(見せる文と、選んだときの分類)。打った語が今の分類と同じでなければ、先頭に
/// 「+ 新しい分類」。打っていなければ最後に「分類なし」。
pub(crate) fn group_items(groups: &[String], query: &str) -> (Vec<String>, Vec<String>) {
    let q = query.trim();
    let (mut shown, mut values) = (Vec::new(), Vec::new());
    if !q.is_empty() && !groups.iter().any(|g| g == q) {
        shown.push(Msg::PlaceNewGroup.fill(&[&q]));
        values.push(q.to_string());
    }
    let lower = q.to_lowercase();
    for g in groups {
        if g.to_lowercase().contains(&lower) {
            shown.push(g.clone());
            values.push(g.clone());
        }
    }
    if q.is_empty() {
        shown.push(Msg::PlaceNoGroup.text().to_string());
        values.push(String::new());
    }
    (shown, values)
}

impl App {
    /// CLI-19: 登録した表の一覧を出す。`here` なら先頭に「今のフォルダ」(引数なしの起動)。
    pub(crate) fn start_open_places(&mut self, here: bool) {
        if self.registered.is_empty() {
            self.message = Some(Msg::PlaceNone.into());
            return;
        }
        let order = places::grouped(&self.registered);
        let mut items = Vec::new();
        let mut picks = Vec::new();
        if here {
            items.push(Msg::PlaceHere.text().to_string());
            picks.push(None);
        }
        let shown: Vec<&Place> = order.iter().map(|&i| &self.registered[i]).collect();
        items.extend(labels(&shown));
        picks.extend(order.into_iter().map(Some));
        self.open_ask(Ask::OpenPlace { items, picks });
    }

    /// CLI-19: 一覧で選んだ表へ移る。無いパスは理由を出して一覧に残る。今のフォルダは一覧を閉じるだけ。
    pub(crate) fn switch_place(&mut self, pick: Option<usize>) {
        let Some(place) = pick.and_then(|i| self.registered.get(i)).cloned() else {
            self.close_palette();
            return;
        };
        if !place.path.exists() {
            self.message = Some(Msg::PlaceMissing.fill(&[&short(&place.path)]));
            return;
        }
        self.close_palette();
        self.switch_to = Some(place);
        self.switch_select = None;
        self.switch_leave_workspace = true;
        self.begin_quit();
    }

    /// CLI-18: 今の表を登録する。名前の欄に、今のビューかフォルダの名前を入れて始める。
    pub(crate) fn start_register_place(&mut self) {
        if self.nv.dir.is_none() {
            self.message = Some(Msg::PlaceNoConfigDir.into());
            return;
        }
        let target = self.nv.target.clone();
        if target.to_string_lossy().contains('\n') {
            self.message = Some(Msg::PlaceMulti.into());
            return;
        }
        let path = std::fs::canonicalize(&target).unwrap_or(target);
        let view = self.current_view_name();
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = view.clone().unwrap_or(stem);
        let place = Place {
            name: String::new(),
            group: String::new(),
            path,
            view,
        };
        self.open_ask(Ask::PlaceName(place));
        if let Some(p) = &mut self.palette {
            p.query = name;
        }
    }

    /// 今のビューの名前(`.base` のビューか mdgrid のビュー)。既定の表なら None。
    fn current_view_name(&self) -> Option<String> {
        if let Some(nv) = self.native_view() {
            return Some(nv.name.clone());
        }
        let b = self.base.as_ref()?;
        let name = b.base.views.get(b.view)?.name.trim().to_string();
        (!name.is_empty()).then_some(name)
    }

    /// 名前の欄で決めた: 分類の欄へ。
    pub(crate) fn place_named(&mut self, mut place: Place, query: &str) {
        let name = query.trim();
        if name.is_empty() {
            self.message = Some(Msg::PlaceEmptyName.into());
            return;
        }
        place.name = name.to_string();
        let groups = places::groups(&self.registered);
        self.open_ask(Ask::PlaceGroup { place, groups });
    }

    /// 分類の欄で決めた: 同じ名前があれば置き換えを確かめ、無ければ書く。
    pub(crate) fn place_grouped(&mut self, mut place: Place, group: String) {
        place.group = group;
        if self.registered.iter().any(|p| p.name == place.name) {
            self.open_ask(Ask::PlaceOverwrite(place));
        } else {
            self.save_place(place);
        }
    }

    /// places.toml に書いて読み直す。
    pub(crate) fn save_place(&mut self, place: Place) {
        self.close_palette();
        let Some(dir) = self.nv.dir.clone() else {
            self.message = Some(Msg::PlaceNoConfigDir.into());
            return;
        };
        let done = if place.group.is_empty() {
            Msg::PlaceSavedNoGroup.fill(&[&place.name])
        } else {
            Msg::PlaceSaved.fill(&[&place.name, &place.group])
        };
        match places::save(&dir, place) {
            Ok(()) => {
                self.registered = places::load(&dir).0;
                self.links.invalidate();
                self.message = Some(done);
            }
            Err(e) => self.message = Some(Msg::PlaceSaveError.fill(&[&e])),
        }
    }

    /// 開き直したあと、登録のビューを名前で選ぶ(`.base` のビューか mdgrid のビュー)。無ければ理由。
    pub fn select_view_named(&mut self, name: &str) {
        let names = self.view_names();
        // 既定の表は、前の名前や別の言語の名前でも選べる(書いてある places.toml のため)。ビューが既定の表
        // だけなら、もう開いている。
        let old_default = super::native_views::is_default_tab(name) && self.base.is_none();
        if old_default && names.is_empty() {
            return;
        }
        match names
            .iter()
            .position(|v| v == name)
            .or(old_default.then_some(0))
        {
            Some(i) => self.select_view(i),
            None => self.message = Some(Msg::PlaceNoView.fill(&[&name])),
        }
    }

    /// 終わる: ためた変更が無ければすぐ、あれば保存する・捨てる・戻るを確かめる(WB-11)。
    pub(crate) fn begin_quit(&mut self) {
        if self.changes.count() == 0 {
            self.quit = true;
        } else {
            self.set_mode(super::keymap::Mode::Quit);
        }
    }
}
