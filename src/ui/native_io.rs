//! mdgrid のビューの書き出しと取り込み(`impl App` の続き。BV-19)。パレットのコマンド
//! 「.base に書き出す」(続けてファイルの名前)と「.base のビューを取り込む」(続けて `.base` → ビュー)の
//! 続きの入力は、パレットの入力と候補の場所をそのまま使う(`Ask`)。書き出しは開いたフォルダの下の
//! 新しい名前のファイルだけに書き(既存の `.base` は書き換えない。BV-3)、落とした部分を知らせる。
//! 変換は核の `mdgrid::views`、タブとビューの保存は native_views.rs。読むだけ(WB-15)では書かない。

use super::app::App;
use super::help::{score, PaletteState};
use super::keymap::{Action, Mode};
use super::native_views::dropped_note;
use super::startup::READONLY;
use super::view;
use super::width::{fit, sanitize, Align};
use mdgrid::base::{self, Base};
use mdgrid::i18n::Msg;
use mdgrid::source::NewValue;
use mdgrid::views;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 取り込みで探す `.base` の深さと数の上限。
const MAX_DEPTH: usize = 8;
const MAX_FILES: usize = 1000;
/// 候補を一度に見せる数。
const ASK_ROWS: usize = 8;

/// パレットの続きの入力(BV-19)。
pub(crate) enum Ask {
    /// 書き出す `.base` のファイルの名前。
    ExportName,
    /// 足す列の名前(CE-28)。
    NewColumn,
    /// write-back.git-message-ask: コミットの文の1行目(空なら自動の文)。
    GitMessage,
    /// 画面の表を書き出すファイルの名前(OUT-2)。
    ExportTable,
    /// navigation.expr-filter: 絞り込む式(候補なし。空で決めると式の絞り込みを解く)。
    ExprFilter,
    /// notice.jump: 名前の条件に当たる行のある表(「表の名前 数」)。選ぶとその条件で絞って開く。
    NoticeTables {
        items: Vec<String>,
        places: Vec<mdgrid::places::Place>,
        cond: String,
    },
    /// 既にあるファイルへの上書きの確かめ(OUT-5)。続けて `y`。
    ExportOverwrite(PathBuf),
    /// 名前を変えるキー(CE-29)。続けて新しい名前。
    RenameKey(String),
    /// 消すキーとノートの数(CE-29)。続けて `y`。
    DeleteKey(String, usize),
    /// 名前を変えるノート(CE-34)。続けて新しい名前。
    RenameNote(mdgrid::source::RowId),
    /// 名前を変えたノート(新しいファイルの名前)を指していたリンクの書き換えと、その数(cell-edit.rename-note-links-setting
    /// の `"ask"`)。続けて `y`。
    RenameLinks {
        file: String,
        items: super::note_rename::Rewrites,
        n: usize,
    },
    /// 消すノート(vim モードでは印を付けた行の全部。screen.vim-dd)と、そのノートを指しているノートの数
    /// (cell-edit.delete-note-confirm)。続けて `y`。
    DeleteNote {
        rows: Vec<mdgrid::source::RowId>,
        links: usize,
    },
    /// ごみ箱のノートの一覧(cell-edit.trash-restore-list)。選ぶと元の場所へ戻す。
    RestoreNote {
        items: Vec<String>,
        notes: Vec<mdgrid::trash::Trashed>,
    },
    /// 取り込む `.base`(開いたフォルダからの相対パス)。
    ImportFile(Vec<String>),
    /// 取り込むビュー。
    ImportView {
        file: String,
        base: Box<Base>,
        names: Vec<String>,
    },
    /// 登録した表の一覧(CLI-19)。`picks` は行ごとの開く表(None は今のフォルダ。最近開いた表も入る)。
    OpenPlace {
        items: Vec<String>,
        picks: Vec<Option<mdgrid::places::Place>>,
    },
    /// 登録する名前(CLI-18)。
    PlaceName(mdgrid::places::Place),
    /// 登録する分類(CLI-18)。候補は今の分類。
    PlaceGroup {
        place: mdgrid::places::Place,
        groups: Vec<String>,
    },
    /// 同じ名前の登録の置き換えの確かめ(CLI-18)。続けて `y`。
    PlaceOverwrite(mdgrid::places::Place),
    /// 開くリンクの行き先(REL-4。セルに2つ以上あるとき)。
    OpenLink {
        items: Vec<String>,
        targets: Vec<PathBuf>,
    },
    /// つながった行(REL-5)。選ぶとそのノートを開く。
    LinkedRows {
        items: Vec<String>,
        notes: Vec<PathBuf>,
    },
    /// 新しいワークスペースの名前(WS-2)。
    WsName,
    /// 足すワークスペース(WS-2。今の名前から選ぶか新しく打つ)。
    WsPick(Vec<String>),
    /// 外すワークスペース(WS-2。この表が入っているもの)。
    WsRemove(Vec<String>),
    /// 開くワークスペース(WS-2)と、最初に開いた所・お気に入りの印のワークスペース(workspace.open-favorites)。
    /// `dests` は `items` と同じ並び。
    WsOpen {
        items: Vec<String>,
        dests: Vec<super::workspace::WsOpenItem>,
    },
    /// 開くワークスペースの表(WS-2)。
    WsTable {
        ws: String,
        items: Vec<String>,
        places: Vec<mdgrid::places::Place>,
    },
    /// スターターを選ぶ(ST-5)。`starters` は `items` と同じ並び。
    StarterPick {
        items: Vec<String>,
        starters: Vec<mdgrid::starter::Starter>,
    },
    /// 作る場所(ST-5。候補なし。既定はスターターの名前)。
    StarterDir(Box<mdgrid::starter::Starter>),
    /// 見本のノートを入れるか(ST-5)。続けて `y`/`n`。
    StarterSamples {
        starter: Box<mdgrid::starter::Starter>,
        root: PathBuf,
    },
    /// navigation.across-*: 横断して探す語。下に進みと表ごとの結果を出す(状態は `App.across`。across.rs)。
    Across,
    /// cli.session-screen-save: 保存するセッションの名前(候補なし)。
    SessionName,
    /// cli.session-screen-overwrite: 同じ名前のセッションの置き換えの確かめ。続けて `y`。
    SessionOverwrite(Box<mdgrid::session::Session>),
    /// cell-edit.new-linked-note: 選んだ行につながるノートのうち、どれを作るか。`opts` は `items` と同じ並び。
    LinkedNote {
        items: Vec<String>,
        opts: Vec<super::new_note::Linked>,
    },
    /// ai.ask-run: AI に頼む文(候補なし)。
    AiRequest,
    /// ai.proposal-notice: AI の案のうち、どれを見るか。`files` は `items` と同じ並び。
    Proposals {
        items: Vec<String>,
        files: Vec<std::path::PathBuf>,
    },
    /// ai.proposal-take・ai.proposal-discard: 選んだ案を入れるか捨てるか。
    ProposalAction {
        items: Vec<String>,
        file: std::path::PathBuf,
    },
    /// 最近開いた表(cli.recent-list)。`places` は `items` と同じ並び。
    OpenRecent {
        items: Vec<String>,
        places: Vec<mdgrid::places::Place>,
    },
    /// stack.add: 下に並べる表(今の範囲の表のうち、まだ並べていないもの)。
    StackTable {
        items: Vec<String>,
        places: Vec<mdgrid::places::Place>,
    },
    /// stack.page-save: 残すページの名前(候補なし)。
    PageName,
    /// routine.screen-create: 作るルーチンの回のノートの名前(印の根からの相対のパス)。続けて `y`。
    RoutineRuns(Vec<String>),
    /// cell-edit.children-reopen: 完了の親を戻すとき、親の完了で閉じた子も戻すか(「子も戻す」「親だけ」)。待っている
    /// 直しは `App::children_ask`(children.rs)。
    ChildrenReopen { items: Vec<String> },
}

/// 候補を打った語で絞る(パレットと同じあいまいな一致の点の順)。返すのは items の添字。
fn filtered(items: &[String], q: &str) -> Vec<usize> {
    let mut s: Vec<(i32, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, t)| score(q.trim(), t).map(|p| (p, i)))
        .collect();
    s.sort_by_key(|&(p, i)| (-p, i));
    s.into_iter().map(|(_, i)| i).collect()
}

/// フォルダの下の `.base`(相対パス。名前が `.` で始まるフォルダとファイルは見ない)。
fn base_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((d, depth)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            if out.len() >= MAX_FILES {
                break;
            }
            if e.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            let p = e.path();
            let Ok(ft) = e.file_type() else {
                continue;
            };
            if ft.is_dir() {
                if depth < MAX_DEPTH {
                    stack.push((p, depth + 1));
                }
            } else if p.extension().is_some_and(|x| x == "base") {
                if let Ok(rel) = p.strip_prefix(root) {
                    out.push(rel.to_string_lossy().into_owned());
                }
            }
        }
    }
    out.sort();
    out
}

/// 対象(フォルダか `.base`)から、書き出しと取り込みの置き場を決める。対象が空なら None。
pub(crate) fn export_dir_of(t: &Path) -> Option<PathBuf> {
    if t.as_os_str().is_empty() {
        return None;
    }
    if t.is_dir() {
        return Some(t.to_path_buf());
    }
    match t.parent() {
        Some(p) if !p.as_os_str().is_empty() => Some(p.to_path_buf()),
        _ => Some(PathBuf::from(".")),
    }
}

/// BV-19: 開いたフォルダがルートフォルダ(`.obsidian` のある上のフォルダ)の下なら、
/// その範囲の `file.inFolder("<ルートフォルダからのフォルダ>")`。ルートフォルダそのもの・ルートフォルダが無い・
/// フォルダでないときは None。
pub(crate) fn folder_filter(target: &Path) -> Option<String> {
    if !target.is_dir() {
        return None;
    }
    let real = target.canonicalize().ok()?;
    let root = mdgrid::vault::roots(std::slice::from_ref(&real))
        .ok()?
        .into_iter()
        .next()?;
    let rel = real.strip_prefix(&root).ok()?;
    if rel.as_os_str().is_empty() {
        return None;
    }
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let folder = parts.join("/").replace('\\', "\\\\").replace('"', "\\\"");
    Some(format!("file.inFolder(\"{folder}\")"))
}

/// パレットの入力の前置き(続きの入力のときだけ)。
pub(crate) fn ask_lead(app: &App) -> String {
    match &app.nv.ask {
        None => String::new(),
        Some(Ask::ExportName) => Msg::AskExportName.into(),
        Some(Ask::NewColumn) => Msg::AskNewColumn.into(),
        Some(Ask::GitMessage) => Msg::AskGitMessage.into(),
        Some(Ask::ExportTable) => Msg::AskExportTable.into(),
        Some(Ask::ExprFilter) => Msg::AskExprFilter.into(),
        Some(Ask::NoticeTables { .. }) => Msg::AskNoticeTables.into(),
        Some(Ask::ExportOverwrite(p)) => {
            Msg::AskExportOverwrite.fill(&[&sanitize(&p.display().to_string())])
        }
        Some(Ask::RenameKey(k)) => Msg::AskRenameKey.fill(&[&sanitize(k)]),
        Some(Ask::DeleteKey(k, n)) => Msg::AskDeleteKey.fill(&[&sanitize(k), n]),
        Some(Ask::RenameNote(_)) => Msg::AskRenameNote.text().to_string(),
        Some(Ask::RenameLinks { n, .. }) => Msg::AskRenameLinks.fill(&[n]),
        Some(Ask::DeleteNote { .. }) => Msg::AskDeleteNote.into(),
        Some(Ask::RestoreNote { .. }) => Msg::AskRestoreNote.into(),
        Some(Ask::ImportFile(_)) => Msg::AskImportFile.into(),
        Some(Ask::ImportView { file, .. }) => Msg::AskImportView.fill(&[&sanitize(file)]),
        Some(Ask::OpenPlace { .. }) => format!("{}: ", Msg::PlaceOpen.text()),
        Some(Ask::PlaceName(_)) => Msg::PlaceAskName.into(),
        Some(Ask::PlaceGroup { .. }) => Msg::PlaceAskGroup.into(),
        Some(Ask::PlaceOverwrite(p)) => Msg::PlaceAskOverwrite.fill(&[&sanitize(&p.name)]),
        Some(Ask::OpenLink { .. }) => Msg::AskOpenLink.into(),
        Some(Ask::LinkedRows { .. }) => Msg::AskLinkedRows.into(),
        Some(Ask::WsName) => Msg::AskWsName.into(),
        Some(Ask::WsPick(_)) => Msg::AskWsPick.into(),
        Some(Ask::WsRemove(_)) => Msg::AskWsRemove.into(),
        Some(Ask::WsOpen { .. }) => Msg::AskWsOpen.into(),
        Some(Ask::WsTable { ws, .. }) => Msg::AskWsTable.fill(&[&sanitize(ws)]),
        Some(Ask::StarterPick { .. }) => Msg::AskStarterPick.into(),
        Some(Ask::StarterDir(_)) => Msg::AskStarterDir.into(),
        Some(Ask::StarterSamples { .. }) => Msg::AskStarterSamples.into(),
        Some(Ask::Across) => Msg::AskAcross.into(),
        Some(Ask::SessionName) => Msg::AskSessionName.into(),
        Some(Ask::SessionOverwrite(s)) => Msg::AskSessionOverwrite.fill(&[&sanitize(&s.name)]),
        Some(Ask::OpenRecent { .. }) => Msg::AskOpenRecent.into(),
        Some(Ask::LinkedNote { .. }) => Msg::AskLinkedNote.into(),
        Some(Ask::Proposals { .. }) => Msg::AskProposals.into(),
        Some(Ask::AiRequest) => Msg::AskAiAsk.into(),
        Some(Ask::ProposalAction { .. }) => Msg::AskProposalAction.into(),
        Some(Ask::StackTable { .. }) => Msg::AskStackTable.into(),
        Some(Ask::PageName) => Msg::AskPageName.into(),
        Some(Ask::RoutineRuns(_)) => Msg::AskRoutineRuns.fill(&[&app.runs_word()]),
        Some(Ask::ChildrenReopen { .. }) => Msg::AskChildrenReopen.into(),
    }
}

/// 続きの入力の候補(全部)と、打った語で絞った添字。
fn ask_items(app: &App, query: &str) -> (Vec<String>, Vec<usize>) {
    let items: &[String] = match &app.nv.ask {
        Some(Ask::ImportFile(files)) => files,
        Some(Ask::ImportView { names, .. }) => names,
        Some(Ask::OpenPlace { items, .. }) => items,
        Some(Ask::OpenLink { items, .. }) => items,
        Some(Ask::LinkedRows { items, .. }) => items,
        Some(Ask::RestoreNote { items, .. }) => items,
        Some(Ask::WsRemove(items) | Ask::WsOpen { items, .. }) => items,
        Some(Ask::WsTable { items, .. }) => items,
        Some(Ask::StarterPick { items, .. }) => items,
        Some(Ask::NoticeTables { items, .. }) => items,
        Some(Ask::OpenRecent { items, .. }) => items,
        Some(Ask::LinkedNote { items, .. }) => items,
        Some(Ask::Proposals { items, .. } | Ask::ProposalAction { items, .. }) => items,
        Some(Ask::StackTable { items, .. }) => items,
        Some(Ask::ChildrenReopen { items }) => items,
        Some(Ask::WsPick(names)) => {
            // WS-2: 打った語は新しいワークスペースにもなる(places の分類と同じ形)。
            let shown = super::workspace::pick_items(names, query).0;
            let idx = (0..shown.len()).collect();
            return (shown, idx);
        }
        Some(Ask::PlaceGroup { groups, .. }) => {
            // CLI-18: 打った語は新しい分類にもなるので、絞った候補をそのまま並べる。
            let shown = super::places::group_items(groups, query).0;
            let idx = (0..shown.len()).collect();
            return (shown, idx);
        }
        _ => &[],
    };
    (items.to_vec(), filtered(items, query))
}

/// 続きの入力を表の上に重ねる(パレットと同じ場所: 2行目に入力、その下に候補)。
pub(crate) fn ask_overlay(app: &App, lines: &mut [Line<'static>], w: usize) {
    let (Some(p), Some(ask)) = (&app.palette, &app.nv.ask) else {
        return;
    };
    let mut out = vec![view::band(
        &format!(": {}{}", ask_lead(app), sanitize(&p.query)),
        w,
    )];
    let room = lines.len().saturating_sub(3).clamp(1, ASK_ROWS);
    if let Ask::Across = ask {
        // navigation.across-results・navigation.across-progress: 進みと表ごとの結果(across.rs)。
        out.extend(super::across::across_lines(
            app,
            w,
            lines.len().saturating_sub(4),
        ));
    } else if matches!(
        ask,
        Ask::PlaceName(_)
            | Ask::PlaceOverwrite(_)
            | Ask::WsName
            | Ask::GitMessage
            | Ask::StarterSamples { .. }
            | Ask::SessionName
            | Ask::SessionOverwrite(_)
            | Ask::PageName
            | Ask::AiRequest
    ) {
        // 候補の無い入力。
    } else if let Ask::RoutineRuns(names) = ask {
        // routine.screen-create: 作るノートの名前を1行に1つ(入りきらない分は数だけ)。
        out.push(Line::from(fit(
            &Msg::RoutineRunsWhich.fill(&[&names.len()]),
            w,
            Align::Left,
        )));
        let room = lines.len().saturating_sub(5).clamp(1, ASK_ROWS);
        let shown = if names.len() > room { room - 1 } else { room };
        for n in names.iter().take(shown) {
            out.push(Line::from(fit(
                &format!("  {}", sanitize(n)),
                w,
                Align::Left,
            )));
        }
        if names.len() > shown {
            let more = Msg::RoutineRunsMore.fill(&[&(names.len() - shown)]);
            out.push(Line::from(fit(&format!("  {more}"), w, Align::Left)));
        }
    } else if let Ask::DeleteNote { rows, links } = ask {
        // cell-edit.delete-note-confirm: 消すノートの名前を、問いの下の1行に出す(長い名前で問いが切れないように)。
        let labels: Vec<String> = rows.iter().map(|r| app.src.label(r)).collect();
        let all = labels.join(Msg::ListSep.text());
        let t = match (labels.as_slice(), *links) {
            ([label], 0) => Msg::DeleteNoteWhich.fill(&[label]),
            ([label], n) => Msg::DeleteNoteWhichLinks.fill(&[label, &n]),
            (_, 0) => Msg::DeleteNotesWhich.fill(&[&labels.len(), &all]),
            (_, n) => Msg::DeleteNotesWhichLinks.fill(&[&labels.len(), &all, &n]),
        };
        out.push(Line::from(fit(&sanitize(&t), w, Align::Left)));
    } else if let Ask::ExportName | Ask::StarterDir(_) = ask {
        // 候補は無く、相対パスの基準のフォルダを1行。
        let dir = app
            .export_dir()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let msg = if matches!(ask, Ask::ExportName) {
            Msg::AskExportWhere
        } else {
            Msg::AskStarterWhere
        };
        let t = msg.fill(&[&dir]);
        out.push(Line::from(fit(&sanitize(&t), w, Align::Left)));
    } else {
        let (items, idx) = ask_items(app, &p.query);
        if idx.is_empty() {
            out.push(Line::from(fit(Msg::AskNoMatch.text(), w, Align::Left)));
        }
        let start = p.sel.saturating_sub(room - 1);
        for (k, &i) in idx.iter().enumerate().skip(start).take(room) {
            let sel = k == p.sel;
            let t = format!("{} {}", if sel { ">" } else { " " }, sanitize(&items[i]));
            let st = if sel {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            out.push(Line::from(Span::styled(fit(&t, w, Align::Left), st)));
        }
    }
    out.push(Line::from(" ".repeat(w)));
    for (k, l) in out.into_iter().enumerate() {
        // ヘッダーのすぐ下(SR-1 の上の余白の分だけ下がる)。
        if let Some(slot) = lines.get_mut(super::bands::head_y(app) as usize + 1 + k) {
            *slot = l;
        }
    }
}

impl App {
    /// 書き出しと取り込みの `.base` の置き場: 開いたフォルダ(`.base` で開いたならその `.base` のフォルダ。
    /// `mdgrid tasks.base` のように相対パスで開いてフォルダの部分が空なら今のフォルダ)。
    pub(crate) fn export_dir(&self) -> Option<PathBuf> {
        export_dir_of(&self.nv.target)
    }

    pub(crate) fn open_ask(&mut self, ask: Ask) {
        self.palette = Some(PaletteState::default());
        self.nv.ask = Some(ask);
        self.set_mode(Mode::Palette);
    }

    /// パレットの「.base に書き出す」: 続けてファイルの名前を打つ。
    pub(crate) fn start_export(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.export_dir().is_none() {
            self.message = Some(Msg::ExportNoDir.into());
            return;
        }
        self.open_ask(Ask::ExportName);
    }

    /// CE-28: どのノートにも無いキーの列を足す。続けて名前を打つ。読むだけの起動では足さない。
    pub(crate) fn start_add_column(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        self.open_ask(Ask::NewColumn);
    }

    /// CE-28: 打った名前の列を表の右に足して選ぶ。もうある列ならそこへ移る。書けない名前は理由を出して入力を残す。
    fn add_extra_column(&mut self, query: &str) {
        let name = query.trim();
        let name = name.strip_prefix("note.").unwrap_or(name).trim();
        if name.is_empty()
            || name.contains(['\n', '\r'])
            || name.starts_with("file.")
            || name.starts_with("formula.")
        {
            self.message = Some(Msg::AddColumnBad.into());
            return;
        }
        self.close_palette();
        if let Some(j) = self.cols.iter().position(|c| c == name) {
            self.col = j;
            self.scroll_into_view();
            return;
        }
        if !self.built.extra_cols.iter().any(|c| c == name) {
            self.built.extra_cols.push(name.to_string());
        }
        self.hidden.retain(|(c, _, _)| c != name);
        self.refresh();
        if let Some(j) = self.cols.iter().position(|c| c == name) {
            self.col = j;
            self.scroll_into_view();
        }
        self.message = Some(Msg::AddColumnAdded.fill(&[&name]));
    }

    /// CE-29: 今の列のキー(ノートのキーの列でなければ理由を出して None)。読むだけの起動でも None。
    fn key_column(&mut self) -> Option<String> {
        if self.readonly {
            self.message = Some(READONLY.into());
            return None;
        }
        let col = self.cols.get(self.col)?.clone();
        if col.starts_with("file.") || col.starts_with("formula.") {
            self.message = Some(Msg::KeyOpComputed.into());
            return None;
        }
        Some(col)
    }

    /// CE-29: そのキーを持つ全部のノート(読み込んだ全部。ビューの絞り込みの前)。
    fn rows_with_key(&self, key: &str) -> Vec<mdgrid::source::RowId> {
        self.src
            .rows()
            .into_iter()
            .filter(|r| self.src.get(r, key).value.is_some())
            .collect()
    }

    /// CE-29: 今の列のキーの名前を変える。続けて新しい名前を打つ。
    pub(crate) fn start_rename_key(&mut self) {
        let Some(col) = self.key_column() else {
            return;
        };
        if self.rows_with_key(&col).is_empty() {
            self.message = Some(Msg::KeyOpNoNotes.fill(&[&col]));
            return;
        }
        self.open_ask(Ask::RenameKey(col));
    }

    /// CE-29: 今の列のキーを消す。数を示して `y` で確かめる。
    pub(crate) fn start_delete_key(&mut self) {
        let Some(col) = self.key_column() else {
            return;
        };
        let n = self.rows_with_key(&col).len();
        if n == 0 {
            self.message = Some(Msg::KeyOpNoNotes.fill(&[&col]));
            return;
        }
        self.open_ask(Ask::DeleteKey(col, n));
    }

    fn rename_key_to(&mut self, from: &str, query: &str) {
        let to = query.trim();
        let to = to.strip_prefix("note.").unwrap_or(to).trim();
        if to.is_empty()
            || to.contains(['\n', '\r'])
            || to.starts_with("file.")
            || to.starts_with("formula.")
        {
            self.message = Some(Msg::KeyOpBadName.into());
            return;
        }
        self.close_palette();
        if to == from {
            return;
        }
        self.stage_key_op(from, NewValue::RenameKey(to.to_string()));
    }

    /// CE-29: そのキーを持つ全部のノートに、名前の変更か削除をためる(1手)。新しい名前のキーが既にあるノートと、
    /// 書けないノート・値は飛ばして数と理由を出す。
    fn stage_key_op(&mut self, key: &str, op: NewValue) {
        let mut reasons = Vec::new();
        let mut items = Vec::new();
        for r in self.rows_with_key(key) {
            // 同じセルに直した値があれば、名前の変更・削除で黙って捨てないように飛ばす。
            if let Some(p) = self.changes.pending(&r, key) {
                if !matches!(p, NewValue::RenameKey(_) | NewValue::DeleteKey) {
                    reasons.push(Msg::KeyOpPending.text().to_string());
                    continue;
                }
            }
            if let NewValue::RenameKey(to) = &op {
                // 大文字小文字だけ違う名前も同じキーとみなす(Obsidian と同じ)。ためた値の列も見る。
                let lower = to.to_lowercase();
                let clash = self
                    .src
                    .columns()
                    .iter()
                    .filter(|c| c.as_str() != key && c.to_lowercase() == lower)
                    .any(|c| {
                        self.src.get(&r, c).value.is_some() || self.changes.pending(&r, c).is_some()
                    })
                    || self.changes.pending(&r, to).is_some();
                if clash {
                    reasons.push(Msg::KeyOpExists.fill(&[to]));
                    continue;
                }
            }
            items.push((r, Some(op.clone())));
        }
        let skips = self.changes.set_each(self.src.as_ref(), key, &items);
        reasons.extend(skips.iter().map(|s| s.reason.clone()));
        let done = items.len() - skips.len();
        self.stay.extend(items.into_iter().map(|(r, _)| r));
        self.regrid = true;
        let mut msg = match &op {
            NewValue::RenameKey(to) => Msg::KeyRenamed.fill(&[&key, to, &done]),
            _ => Msg::KeyDeleted.fill(&[&key, &done]),
        };
        super::input::bulk_tail(&mut msg, 0, reasons);
        self.message = Some(msg);
    }

    /// パレットの「.base のビューを取り込む」: 続けて `.base` を選び、ビューを選ぶ。
    pub(crate) fn start_import(&mut self) {
        if self.readonly {
            self.message = Some(READONLY.into());
            return;
        }
        if self.nv.dir.is_none() {
            self.message = Some(Self::NO_DIR.into());
            return;
        }
        let Some(dir) = self.export_dir() else {
            self.message = Some(Msg::ImportNoDir.into());
            return;
        };
        let files = base_files(&dir);
        if files.is_empty() {
            self.message = Some(Msg::ImportNoFiles.fill(&[&dir.display()]));
            return;
        }
        self.open_ask(Ask::ImportFile(files));
    }

    /// 続きの入力の中の動作(パレットのキー): 決める・やめる・候補の選択・1字消す。
    pub(crate) fn ask_action(&mut self, action: Action) {
        let query = self
            .palette
            .as_ref()
            .map(|p| p.query.clone())
            .unwrap_or_default();
        let n = match self.nv.ask {
            // navigation.across-results: ↑↓ は結果の当たった行を選ぶ。
            Some(Ask::Across) => self.across_hit_count(),
            _ => ask_items(self, &query).1.len(),
        };
        let Some(p) = &mut self.palette else {
            return self.close_palette();
        };
        match action {
            Action::Close => {
                // cell-edit.rename-note-links-setting: リンクを書き換えるかの問いを閉じたら、名前は変えたまま書き換えない。
                let kept = match &self.nv.ask {
                    Some(Ask::RenameLinks { file, n, .. }) => {
                        Some(Msg::RenameNoteKeptLinks.fill(&[file, n]))
                    }
                    _ => None,
                };
                self.close_palette();
                self.message = Some(kept.unwrap_or_else(|| Msg::AskCancelled.into()));
            }
            Action::Down => p.sel = (p.sel + 1).min(n.saturating_sub(1)),
            Action::Up => p.sel = p.sel.saturating_sub(1),
            Action::DeleteBack => {
                p.query.pop();
                p.sel = 0;
            }
            Action::Run => self.ask_run(&query),
            _ => {}
        }
    }

    fn ask_run(&mut self, query: &str) {
        let sel = self.palette.as_ref().map_or(0, |p| p.sel);
        let picked = ask_items(self, query).1.get(sel).copied();
        match &self.nv.ask {
            Some(Ask::ExportName) => self.export_to(query),
            Some(Ask::NewColumn) => self.add_extra_column(query),
            Some(Ask::GitMessage) => {
                let first = query.trim().to_string();
                self.close_palette();
                self.git_flush(Some(&first));
            }
            Some(Ask::ExportTable) => self.export_table_to(query),
            Some(Ask::ExprFilter) => self.run_expr_filter(query),
            Some(Ask::Across) => self.across_run(query, sel),
            Some(Ask::NoticeTables { places, cond, .. }) => {
                let Some(place) = picked.and_then(|k| places.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                let cond = cond.clone();
                self.notice_open_table(place, cond);
            }
            Some(Ask::ExportOverwrite(path)) => {
                let path = path.clone();
                self.close_palette();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.export_table_write(&path);
                } else {
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            Some(Ask::RenameKey(from)) => {
                let from = from.clone();
                self.rename_key_to(&from, query);
            }
            Some(Ask::RenameNote(row)) => {
                let row = row.clone();
                self.rename_note_to(&row, query);
            }
            Some(Ask::RenameLinks { .. }) => {
                let Some(Ask::RenameLinks { file, items, n }) = self.nv.ask.take() else {
                    return;
                };
                self.close_palette();
                self.confirm_rename_links(&file, items, n, query.trim().eq_ignore_ascii_case("y"));
            }
            Some(Ask::DeleteNote { rows, .. }) => {
                let rows = rows.clone();
                self.confirm_delete_note(rows, query.trim().eq_ignore_ascii_case("y"));
            }
            Some(Ask::RoutineRuns(_)) => {
                // 閉じるときにやめたことにしない(close_palette)よう、問いを先に外す。
                self.nv.ask = None;
                self.close_palette();
                self.confirm_routine_runs(query.trim().eq_ignore_ascii_case("y"));
            }
            // cell-edit.children-reopen: 最初の項目が「子も戻す」、次が「親だけ」。
            Some(Ask::ChildrenReopen { .. }) => match picked {
                Some(k) => self.children_reopen_chosen(k == 0),
                None => self.message = Some(Msg::AskNoMatch.into()),
            },
            Some(Ask::RestoreNote { notes, .. }) => {
                let Some(item) = picked.and_then(|k| notes.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.restore_note(item);
            }
            Some(Ask::DeleteKey(key, _)) => {
                let key = key.clone();
                self.close_palette();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.stage_key_op(&key, NewValue::DeleteKey);
                } else {
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            Some(Ask::ImportFile(files)) => {
                let Some(rel) = picked.map(|i| files[i].clone()) else {
                    self.message = Some(Msg::ImportNoMatchFile.into());
                    return;
                };
                let Some(dir) = self.export_dir() else {
                    return;
                };
                let parsed = mdgrid::textfile::read_text(dir.join(&rel))
                    .map_err(|e| e.to_string())
                    .and_then(|t| base::parse(&t));
                match parsed {
                    Err(e) => self.message = Some(Msg::ImportUnreadable.fill(&[&rel, &e])),
                    Ok(b) if b.views.is_empty() => {
                        self.message = Some(Msg::ImportNoViews.fill(&[&rel]));
                    }
                    Ok(b) => {
                        // 名前の無いビューも選べるように、候補には番号を出す。
                        let names = b
                            .views
                            .iter()
                            .enumerate()
                            .map(|(k, v)| match v.name.trim() {
                                "" => Msg::ImportUnnamed.fill(&[&(k + 1)]),
                                _ => v.name.clone(),
                            })
                            .collect();
                        self.nv.ask = Some(Ask::ImportView {
                            file: rel,
                            base: Box::new(b),
                            names,
                        });
                        self.palette = Some(PaletteState::default());
                    }
                }
            }
            Some(Ask::ImportView { file, base, names }) => {
                let Some(i) = picked else {
                    self.message = Some(Msg::ImportNoMatchView.into());
                    return;
                };
                let (file, from) = (file.clone(), names[i].clone());
                let (mut nv, dropped) = views::from_base(base, i);
                // 名前の無いビューは「.base のファイル名 番号」。
                if nv.name.trim().is_empty() {
                    let stem = Path::new(&file)
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    nv.name = format!("{stem} {}", i + 1);
                }
                let res = self.edit_views(|app, list| {
                    nv.name = app.unique_name(list, &nv.name);
                    let name = nv.name.clone();
                    list.push(nv);
                    Ok(name)
                });
                let (list, name) = match res {
                    Ok(r) => r,
                    Err(e) => return self.message = Some(e),
                };
                let keep = self.native_view().map(|v| v.name.clone());
                self.adopt_views(list, keep.as_deref());
                self.close_palette();
                if let Some(j) = self.nv.views.iter().position(|v| v.name == name) {
                    self.select_view(self.base_tabs() + j);
                }
                self.message =
                    Some(Msg::Imported.fill(&[&file, &from, &name, &dropped_note(&dropped)]));
            }
            Some(Ask::OpenPlace { picks, .. }) => {
                let Some(k) = picked else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                // workspace.switch-origin: 起動の一覧(先頭に今のフォルダ)で選んで移ったら、移った先を最初に開いた所に。
                let launch = picks.iter().any(Option::is_none);
                let pick = picks[k].clone();
                self.switch_place(pick);
                if launch && self.switch_to.is_some() {
                    self.visits.retarget = true;
                }
            }
            Some(Ask::LinkedNote { opts, .. }) => {
                let Some(o) = picked.and_then(|k| opts.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.close_palette();
                self.make_linked(o);
            }
            Some(Ask::Proposals { files, .. }) => {
                let Some(file) = picked.and_then(|k| files.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.close_palette();
                self.choose_proposal(file);
            }
            Some(Ask::ProposalAction { file, .. }) => {
                let file = file.clone();
                match picked {
                    Some(0) => self.take_proposal(&file),
                    Some(1) => self.discard_proposal(&file),
                    _ => self.message = Some(Msg::AskNoMatch.into()),
                }
            }
            Some(Ask::OpenRecent { places, .. }) => {
                let Some(place) = picked.and_then(|k| places.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.switch_place(Some(place));
            }
            Some(Ask::PlaceName(place)) => {
                let place = place.clone();
                self.place_named(place, query);
            }
            Some(Ask::PlaceGroup { place, groups }) => {
                let values = super::places::group_items(groups, query).1;
                let Some(group) = picked.and_then(|k| values.get(k)).cloned() else {
                    return;
                };
                let place = place.clone();
                self.place_grouped(place, group);
            }
            Some(Ask::OpenLink { targets: paths, .. } | Ask::LinkedRows { notes: paths, .. }) => {
                let Some(p) = picked.and_then(|k| paths.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.open_note(&p);
            }
            Some(Ask::WsName) => {
                let name = query.trim().to_string();
                self.ws_add_current(&name);
            }
            Some(Ask::WsPick(names)) => {
                let values = super::workspace::pick_items(names, query).1;
                let Some(name) = picked.and_then(|k| values.get(k)).cloned() else {
                    return;
                };
                self.ws_add_current(&name);
            }
            Some(Ask::WsRemove(items)) => {
                let Some(name) = picked.and_then(|k| items.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.ws_remove_current(&name);
            }
            Some(Ask::WsOpen { dests, .. }) => {
                let Some(dest) = picked.and_then(|k| dests.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                match dest {
                    super::workspace::WsOpenItem::Named(name) => self.ws_open_tables(&name),
                    // workspace.open-favorites: 切り替えの一覧と同じ道で移る。
                    super::workspace::WsOpenItem::Go(d) => {
                        self.go_to(d, super::switch::Picker::Palette);
                    }
                }
            }
            Some(Ask::WsTable { ws, places, .. }) => {
                let Some(place) = picked.and_then(|k| places.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                let ws = ws.clone();
                self.ws_switch(ws, place);
            }
            Some(Ask::PlaceOverwrite(place)) => {
                let place = place.clone();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.save_place(place);
                } else {
                    self.close_palette();
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            // ST-5: スターター → 作る場所 → 見本を入れるか → 作って開く(starter.rs)。
            Some(Ask::StarterPick { starters, .. }) => {
                let Some(s) = picked.and_then(|k| starters.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.starter_picked(s);
            }
            Some(Ask::StarterDir(s)) => {
                let s = (**s).clone();
                self.starter_dir(s, query);
            }
            Some(Ask::StarterSamples { starter, root }) => {
                let (s, root) = ((**starter).clone(), root.clone());
                self.close_palette();
                match query.trim().to_ascii_lowercase().as_str() {
                    "y" => self.starter_make(s, root, true),
                    "n" => self.starter_make(s, root, false),
                    _ => self.message = Some(Msg::AskCancelled.into()),
                }
            }
            // cli.session-screen-save・cli.session-screen-overwrite: 名前 → (同じ名前があれば)置き換えの確かめ → 書く。
            Some(Ask::SessionName) => self.session_named(query),
            Some(Ask::AiRequest) => self.send_ai_ask(query),
            Some(Ask::SessionOverwrite(s)) => {
                let s = (**s).clone();
                if query.trim().eq_ignore_ascii_case("y") {
                    self.save_session(&s);
                } else {
                    self.close_palette();
                    self.message = Some(Msg::AskCancelled.into());
                }
            }
            // stack.add: 選んだ表を今の表の下に並べる(stack.rs)。
            Some(Ask::StackTable { places, .. }) => {
                let Some(place) = picked.and_then(|k| places.get(k)).cloned() else {
                    self.message = Some(Msg::AskNoMatch.into());
                    return;
                };
                self.stack_add_picked(place);
            }
            // stack.page-save: 打った名前でページを残す(stack.rs)。
            Some(Ask::PageName) => self.page_save_to(query),
            None => self.close_palette(),
        }
    }

    /// 今のビューを新しい `.base` に書き出す(BV-19)。既存のファイルの名前なら書かずに理由(BV-3)。
    fn export_to(&mut self, query: &str) {
        let name = query.trim();
        if name.is_empty() {
            self.message = Some(Msg::ExportNameEmpty.into());
            return;
        }
        if name.contains(['/', '\\']) || name.starts_with('.') {
            self.message = Some(Msg::ExportNameOnly.into());
            return;
        }
        let Some(dir) = self.export_dir() else {
            return;
        };
        let file = if std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("base"))
        {
            name.to_string()
        } else {
            format!("{name}.base")
        };
        let path = dir.join(&file);
        if path.exists() {
            self.message = Some(Msg::ExportExists.fill(&[&file]));
            return;
        }
        let cols = self.column_list();
        let (mut nv, mut dropped) = self.as_native(&self.settings, &cols, true);
        // BV-19: `.base` はルートフォルダの全部のノートを相手にするので、下のフォルダの表なら、その範囲を絞り込みに足す。
        if let Some(f) = folder_filter(&self.nv.target) {
            if !nv.filters_expr.iter().any(|e| e.trim() == f) {
                nv.filters_expr.insert(0, f);
            }
        }
        let kind = |c: &str| Some(self.column_kind(c));
        let (yaml, more) = views::to_base_typed(&nv, &kind);
        dropped.extend(more);
        let written = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .and_then(|mut f| f.write_all(yaml.as_bytes()));
        if let Err(e) = written {
            self.message = Some(Msg::CannotWriteFile.fill(&[&file, &e]));
            return;
        }
        self.close_palette();
        self.message = Some(Msg::Exported.fill(&[&nv.name, &file, &dropped_note(&dropped)]));
    }
}
