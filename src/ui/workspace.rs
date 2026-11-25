//! ワークスペースの画面の側(`impl App` の続き。WS-2・WS-4・WS-6): 範囲を起動で決め、リレーション・関係マップ・
//! タブの表を範囲から取る。パレットのコマンドで作る・足す・外す・開く。範囲の決め方は核の mdgrid::workspace。

use super::app::App;
use super::native_io::Ask;
use super::switch::Dest;
use mdgrid::favorites::Favorite;
use mdgrid::i18n::Msg;
use mdgrid::places::Place;
use mdgrid::workspace::{self, Scope, WsTable};
use std::path::{Path, PathBuf};

impl App {
    /// 範囲を決める(WS-6)。`target` は起動の対象(複数のフォルダなら改行で区切った最初)。誤りは警告の文。
    pub(crate) fn resolve_scope(
        &mut self,
        target: &Path,
        detect: &[workspace::Detect],
    ) -> Option<String> {
        let first = target
            .to_string_lossy()
            .lines()
            .next()
            .map_or_else(|| target.to_path_buf(), PathBuf::from);
        match workspace::resolve(
            &first,
            self.workspace_choice.as_deref(),
            &self.workspaces,
            detect,
        ) {
            Ok(s) => {
                let warn = s.as_ref().and_then(|s| s.warnings.first().cloned());
                self.scope = s;
                warn
            }
            Err(e) => Some(e),
        }
    }

    /// 範囲の表(WS-4)。範囲が無ければ登録した表(CLI-18)。
    /// リンクと関係マップの範囲の表: 今の範囲の表のうち、ノートの表(フォルダか `.base`)。CSV・TSV の表は
    /// 行がノートでなく、ノートとの間のリンクの持ち方も決まっていないので入れない(SC-17)。
    pub(crate) fn link_places(&self) -> Vec<Place> {
        self.scope_places()
            .into_iter()
            .filter(|p| !mdgrid::source::csv::Csv::handles(&p.path))
            .collect()
    }

    pub(crate) fn scope_places(&self) -> Vec<Place> {
        match &self.scope {
            Some(Scope { tables, .. }) => tables
                .iter()
                .map(|t| Place {
                    name: t.name.clone(),
                    group: String::new(),
                    path: t.path.clone(),
                    view: t.view.clone(),
                })
                .collect(),
            None => self.registered.clone(),
        }
    }
}

/// パレットの「ワークスペースを開く」の1項目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WsOpenItem {
    /// workspaces.toml のワークスペース(続けて表を選ぶ。WS-2)。
    Named(String),
    /// 最初に開いた所か、お気に入りの印のワークスペース(切り替えの一覧と同じ道で移る。workspace.open-favorites)。
    Go(Dest),
}

/// 足す先の候補(見せる文と、選んだときの名前)。打った語が今の名前と同じでなければ、先頭に「+ 新しい
/// ワークスペース」(places の分類と同じ形)。
pub(crate) fn pick_items(names: &[String], query: &str) -> (Vec<String>, Vec<String>) {
    let q = query.trim();
    let (mut shown, mut values) = (Vec::new(), Vec::new());
    if !q.is_empty() && !names.iter().any(|n| n == q) {
        shown.push(Msg::WsNewItem.fill(&[&q]));
        values.push(q.to_string());
    }
    let lower = q.to_lowercase();
    for n in names {
        if n.to_lowercase().contains(&lower) {
            shown.push(n.clone());
            values.push(n.clone());
        }
    }
    (shown, values)
}

impl App {
    /// 今の表(WS-2 で足す1つ)。設定のフォルダが無い・フォルダを複数開いていれば理由を出して None。
    fn ws_current_table(&mut self) -> Option<WsTable> {
        if self.nv.dir.is_none() {
            self.message = Some(Msg::WsNoConfigDir.into());
            return None;
        }
        let target = self.nv.target.clone();
        if target.to_string_lossy().contains('\n') {
            self.message = Some(Msg::WsMulti.into());
            return None;
        }
        let path = std::fs::canonicalize(&target).unwrap_or(target);
        let name = workspace::stem_of(&path);
        Some(WsTable {
            name,
            path,
            ..WsTable::default()
        })
    }

    /// WS-2: この表で新しいワークスペースを作る(名前の欄)。
    pub(crate) fn start_ws_new(&mut self) {
        if self.ws_current_table().is_some() {
            self.open_ask(Ask::WsName);
        }
    }

    /// WS-2: この表をワークスペースに足す(今の名前から選ぶか新しく打つ)。
    pub(crate) fn start_ws_add(&mut self) {
        if self.ws_current_table().is_some() {
            let names = self.workspaces.iter().map(|w| w.name.clone()).collect();
            self.open_ask(Ask::WsPick(names));
        }
    }

    /// WS-2: この表をワークスペースから外す(入っているものから選ぶ)。
    pub(crate) fn start_ws_remove(&mut self) {
        let Some(t) = self.ws_current_table() else {
            return;
        };
        let names: Vec<String> = self
            .workspaces
            .iter()
            .filter(|w| {
                w.tables
                    .iter()
                    .any(|x| workspace::same_path(&x.path, &t.path))
            })
            .map(|w| w.name.clone())
            .collect();
        if names.is_empty() {
            self.message = Some(Msg::WsNotIn.into());
            return;
        }
        self.open_ask(Ask::WsRemove(names));
    }

    /// WS-2: ワークスペースを開く(名前を選び、次にその表を選ぶ)。workspace.open-favorites: 起動したときに開いた
    /// 所から離れていれば「最初に開いた所」を先頭に、お気に入りの印のワークスペースを末尾に並べ、選んだら切り替えの
    /// 一覧と同じ道で移る(workspaces.toml のお気に入りは名前の項目のまま)。並べるものが無ければ作り方。
    pub(crate) fn start_ws_open(&mut self) {
        let favs = self.resolved_favorites().0;
        let (mut items, mut dests) = (Vec::new(), Vec::new());
        if self.origin_away() {
            items.push(self.origin_label());
            dests.push(WsOpenItem::Go(Dest::Origin));
        }
        for w in &self.workspaces {
            items.push(w.name.clone());
            dests.push(WsOpenItem::Named(w.name.clone()));
        }
        for f in favs {
            if matches!(f, Favorite::Marker { .. }) {
                items.push(f.name().to_string());
                dests.push(WsOpenItem::Go(Dest::Fav(f)));
            }
        }
        if items.is_empty() {
            self.message = Some(Msg::WsNone.into());
            return;
        }
        self.open_ask(Ask::WsOpen { items, dests });
    }

    /// 今の表を `name` のワークスペースに足して読み直す(無ければ作る)。
    pub(crate) fn ws_add_current(&mut self, name: &str) {
        if name.is_empty() {
            self.message = Some(Msg::PlaceEmptyName.into());
            return;
        }
        let Some(table) = self.ws_current_table() else {
            return;
        };
        self.close_palette();
        let dir = self.nv.dir.clone().expect("checked by ws_current_table");
        let label = table.name.clone();
        match workspace::add(&dir, name, table) {
            Ok(()) => {
                self.ws_reload(&dir);
                self.message = Some(Msg::WsAdded.fill(&[&label, &name]));
            }
            Err(e) => self.message = Some(Msg::WsSaveError.fill(&[&e])),
        }
    }

    /// 今の表を `name` のワークスペースから外して読み直す。
    pub(crate) fn ws_remove_current(&mut self, name: &str) {
        let Some(table) = self.ws_current_table() else {
            return;
        };
        self.close_palette();
        let dir = self.nv.dir.clone().expect("checked by ws_current_table");
        match workspace::remove(&dir, name, Some(&table.path)) {
            Ok(true) => {
                self.ws_reload(&dir);
                self.message = Some(Msg::WsRemoved.fill(&[&table.name, &name]));
            }
            Ok(false) => self.message = Some(Msg::WsNotIn.into()),
            Err(e) => self.message = Some(Msg::WsSaveError.fill(&[&e])),
        }
    }

    /// workspaces.toml を読み直し、範囲を決め直す(リンクの読みも捨てる)。
    fn ws_reload(&mut self, dir: &Path) {
        self.workspaces = workspace::load(dir).0;
        let target = self.nv.target.clone();
        let detect = self.workspace_detect.clone();
        if let Some(e) = self.resolve_scope(&target, &detect) {
            self.message = Some(e);
        }
        // stack.page-store: 範囲が変われば、ページも読み直す(警告は起動のときに出したので出さない)。
        self.load_pages(&target, Some(dir));
        self.links.invalidate();
    }

    /// WS-2: 選んだワークスペースの表の一覧を出す。
    pub(crate) fn ws_open_tables(&mut self, name: &str) {
        let Some(ws) = self.workspaces.iter().find(|w| w.name == name) else {
            self.message = Some(Msg::AskNoMatch.into());
            return;
        };
        if ws.tables.is_empty() {
            self.message = Some(Msg::WsEmpty.fill(&[&name]));
            return;
        }
        let places: Vec<Place> = ws
            .tables
            .iter()
            .map(|t| Place {
                name: t.name.clone(),
                group: String::new(),
                path: t.path.clone(),
                view: t.view.clone(),
            })
            .collect();
        let items = super::places::labels(&places.iter().collect::<Vec<_>>());
        let ws = ws.name.clone();
        self.open_ask(Ask::WsTable { ws, items, places });
    }

    /// WS-2: ワークスペースの表へ移る(終わる手順を通してから main が開き直す。範囲はそのワークスペース)。
    pub(crate) fn ws_switch(&mut self, ws: String, place: Place) {
        // WS-9: 印に中身を書いた表は、ファイルが無くても開ける。
        if !mdgrid::workspace::openable(&place.path) {
            self.message = Some(Msg::PlaceMissing.fill(&[&place.path.display()]));
            return;
        }
        self.close_palette();
        self.switch_to = Some(place);
        self.switch_select = None;
        self.switch_workspace = Some(ws);
        self.begin_quit();
    }
}
