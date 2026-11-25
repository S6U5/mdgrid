//! グラフの画面(CH-1・CH-2)の受け入れの試験と、グラフの画面の試験(test_chart2.rs・test_chart3.rs)が使う補助。
//! 値は作ったもの。記録: specs/_history/decidespec/_changes/2026-11-06-chart.md(タスク 1)。実装の前に書いた。
//!
//! まだ無い型・関数・App の欄は `Mode::Chart` のほかは使わず、キーとマウスを送り、画面の文字と今ある口
//! (`mode`・`message`・`rows`・`cur_row()`・`changes.count()`・`click`・`wheel`・`poll`)で確かめる。
//! 行番号や字の位置は決め打ちしない。グラフの枠は題の行(「グラフ:」で始まる題を持つ上の辺)から下の辺までで、
//! 枠の中の行(体)は、左の辺と右の辺の縦の線の間の文字として読む。体の1行目は `<n> 行`(外した行があれば
//! ` · 外した <m> 行`)、2行目は選んでいるものの説明、内訳があれば3行目に凡例、その下にグラフ。
//! 棒の行は `>`(選んでいる項目)か空白、名前、`│`、棒(塗りの字と端の 8 分の 1 の字)、空白、値。

use super::app::{App, ColorMode};
use super::keymap::Mode;
use super::startup::Startup;
use super::test_screen::{ch, col_named, text, typing, Tmp};
use super::width::width;
use mdgrid::config::Config;
use mdgrid::source::markdown::Markdown;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use ratatui::Terminal;
use std::path::PathBuf;

/// 画面の題の種類の名前(Tab の順)。
pub(super) const KINDS: [&str; 5] = ["棒", "積み上げ棒", "折れ線", "散布図", "ヒートマップ"];

/// 縦の線(枠の縦の辺・棒の区切り・折れ線の軸)。
pub(super) const SEPS: [char; 3] = ['│', '┃', '|'];

/// 枠の下の角(グラフの枠の終わり)。
const BOTTOM: [char; 6] = ['╰', '└', '┗', '╯', '┘', '┛'];

/// 棒と積み上げ棒の塗りの字(内訳の順。最後の `:` は「ほか」)。
pub(super) const FILLS: [char; 6] = ['█', '▓', '▒', '░', '▞', ':'];

/// 棒の端の 8 分の 1 の字(1/8 から 7/8)。
const EIGHTHS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// 折れ線と散布図の点の字(内訳の順)。
pub(super) const POINTS: [char; 5] = ['*', '+', 'o', 'x', '#'];

/// ノートの (ファイルの名前, 中身)。`keys` はフロントマターの中身(`\n` で区切る)。
pub(super) fn md(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(n, keys)| (format!("{n}.md"), format!("---\n{keys}\n---\n")))
        .collect()
}

/// CH-2・CH-4 の例: todo 2件(alpha・beta)・doing 1件(gamma)・done 3件(delta・epsilon・zeta)。
pub(super) fn status_notes() -> Vec<(String, String)> {
    md(&[
        ("alpha", "status: todo"),
        ("beta", "status: todo"),
        ("gamma", "status: doing"),
        ("delta", "status: done"),
        ("epsilon", "status: done"),
        ("zeta", "status: done"),
    ])
}

/// 印(根 = 一時フォルダ)の先頭に `top`(`[rules]`・`[types]` など)、[[table]] に `line` を書いて、`notes` を表として開き、
/// 読み終える。
pub(super) fn boot_full(
    tag: &str,
    notes: &[(String, String)],
    top: &str,
    line: &str,
    color: ColorMode,
    size: (u16, u16),
) -> (Tmp, App) {
    let tmp = Tmp::new(tag);
    for (n, t) in notes {
        tmp.write(n, t);
    }
    std::fs::create_dir_all(tmp.0.join(".mdgrid")).unwrap();
    std::fs::write(
        tmp.0.join(".mdgrid/workspace.toml"),
        format!("{top}\n[[table]]\nname = \"T\"\npath = \"notes\"\n{line}\n"),
    )
    .unwrap();
    let dir: PathBuf = tmp.notes();
    let src = Markdown::open(std::slice::from_ref(&dir)).unwrap();
    let mut app = App::new(Box::new(src), color);
    app.resize(size.0, size.1);
    app.start(Startup {
        config: Config::default(),
        warnings: Vec::new(),
        readonly: false,
        no_color: matches!(color, ColorMode::None),
        state_dir: Some(tmp.0.join("state")),
        config_dir: Some(tmp.0.join("config")),
        target: dir,
        base: None,
    });
    while !app.loaded() {
        app.load_step(100);
    }
    (tmp, app)
}

/// 色なし・幅 120・高さ 30 で開く(題は枠の上の辺に出るので広めにする)。
pub(super) fn boot(tag: &str, notes: &[(String, String)], top: &str, line: &str) -> (Tmp, App) {
    boot_full(tag, notes, top, line, ColorMode::None, (120, 30))
}

/// App の大きさで描いた画面の文字。
pub(super) fn screen(a: &App) -> String {
    let (w, h) = a.size;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| super::draw(f, a)).unwrap();
    text(t.backend().buffer())
}

/// メッセージ行(画面の一番下の行)を除いた画面の文字。理由や警告はメッセージ行に出て、表に戻っても残るので、
/// どの画面かはこの残りで見る。
pub(super) fn above_message(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[..lines.len().saturating_sub(1)].join("\n")
}

/// 1行の理由が出ている(その文を返す)。
pub(super) fn reason(a: &App) -> String {
    let m = a
        .message
        .clone()
        .unwrap_or_else(|| panic!("理由が出ない\n{}", screen(a)));
    assert!(
        !m.trim().is_empty() && !m.contains('\n'),
        "理由は1行: {m:?}"
    );
    m
}

/// 画面の型のメニューを開いて描いた文字(閉じて戻す)。
pub(super) fn menu_text(a: &mut App) -> String {
    a.open_screen_menu();
    assert_eq!(a.mode, Mode::ScreenMenu, "{:?}", a.message);
    let s = screen(a);
    a.close_screen_menu();
    s
}

/// メニューの窓の、名前 `name` と切り替えのキー `key` の項目の、名前の最初の字の (桁, 行)。行の右の端の枠を除くと
/// キーで終わり、その手前が名前で終わる行。
pub(super) fn menu_item(menu: &str, name: &str, key: &str) -> Option<(u16, u16)> {
    menu.lines().enumerate().skip(1).find_map(|(y, l)| {
        let t = l.trim_end().trim_end_matches(SEPS).trim_end();
        let rest = t.strip_suffix(key)?;
        if !rest.ends_with(' ') || !rest.trim_end().ends_with(name) {
            return None;
        }
        let i = rest.trim_end().len() - name.len();
        Some((width(&l[..i]) as u16, y as u16))
    })
}

/// グラフの枠の題の行か(「グラフ」のすぐあとに `:` か `:`)。
fn is_chart_title(l: &str) -> bool {
    l.match_indices("グラフ")
        .any(|(i, w)| l[i + w.len()..].trim_start().starts_with([':', ':']))
}

/// 枠の中の1行: 左の辺と右の辺の間の文字と、その最初の字の画面の (桁, 行)。
#[derive(Debug, Clone)]
pub(super) struct Row {
    pub text: String,
    pub x: u16,
    pub y: u16,
}

/// 棒の行。
#[derive(Debug, Clone)]
pub(super) struct Bar {
    pub name: String,
    pub selected: bool,
    /// 棒の長さ(字の数。端の 8 分の 1 の字は分数)。
    pub len: f64,
    /// 棒の塗りの字(端の 8 分の 1 の字を除く)。
    pub fills: String,
    pub value: String,
    /// 名前の最初の字の画面の (桁, 行)。
    pub at: (u16, u16),
}

/// 描いたグラフの画面。
#[derive(Debug, Clone)]
pub(super) struct Chart {
    pub title: String,
    pub rows: Vec<Row>,
    pub raw: String,
}

/// 画面の文字からグラフの枠を読む(無ければ None)。
pub(super) fn parse(s: &str) -> Option<Chart> {
    let body = above_message(s);
    let lines: Vec<&str> = body.lines().collect();
    let ty = lines.iter().position(|l| is_chart_title(l))?;
    let tl = lines[ty];
    let i = tl
        .match_indices("グラフ")
        .find(|(i, w)| tl[i + w.len()..].trim_start().starts_with([':', ':']))
        .map(|(i, _)| i)?;
    let title = tl[i..]
        .trim_end_matches(|c: char| c.is_whitespace() || "─━═-╮┐┓╗+".contains(c))
        .to_string();
    let mut rows = Vec::new();
    for (k, l) in lines.iter().enumerate().skip(ty + 1) {
        if l.trim_start().starts_with(BOTTOM) {
            break;
        }
        let Some(a) = l.find(SEPS) else { break };
        let c = l[a..].chars().next().unwrap();
        let start = a + c.len_utf8();
        let rest = &l[start..];
        let inner = match rest.rfind(SEPS) {
            Some(b) => &rest[..b],
            None => rest,
        };
        rows.push(Row {
            text: inner.to_string(),
            x: width(&l[..start]) as u16,
            y: k as u16,
        });
    }
    Some(Chart {
        title,
        rows,
        raw: s.to_string(),
    })
}

/// 体の1行目 `<n> 行`(` · 外した <m> 行`)を読む。
fn parse_counts(t: &str) -> Option<(usize, Option<usize>)> {
    let t = t.trim();
    let i = t.find(" 行")?;
    let n: usize = t[..i].trim().parse().ok()?;
    let rest = t[i + " 行".len()..].trim();
    if rest.is_empty() {
        return Some((n, None));
    }
    let r = rest
        .strip_prefix('·')?
        .trim()
        .strip_prefix("外した")?
        .trim();
    let m: usize = r.strip_suffix('行')?.trim().parse().ok()?;
    Some((n, Some(m)))
}

/// 棒の行を読む(棒の区切りの縦の線が無い行・棒の字でない字のある行は None)。
fn bar_row(r: &Row) -> Option<Bar> {
    let t = r.text.as_str();
    let i = t.find(SEPS)?;
    let sep = t[i..].chars().next()?;
    let left = &t[..i];
    let right = t[i + sep.len_utf8()..].trim_end();
    let (bar, value) = match right.rfind(' ') {
        Some(j) => (&right[..j], right[j + 1..].to_string()),
        None => ("", right.to_string()),
    };
    if value.is_empty()
        || !bar
            .chars()
            .all(|c| c == ' ' || FILLS.contains(&c) || EIGHTHS.contains(&c))
    {
        return None;
    }
    let l = left.trim();
    let (selected, name) = match l.strip_prefix('>') {
        Some(rest) => (true, rest.trim()),
        None => (false, l),
    };
    if name.is_empty() {
        return None;
    }
    let len = bar
        .chars()
        .map(|c| {
            if FILLS.contains(&c) {
                1.0
            } else if let Some(k) = EIGHTHS.iter().position(|e| *e == c) {
                (k + 1) as f64 / 8.0
            } else {
                0.0
            }
        })
        .sum();
    let ni = t.find(name).unwrap_or(0);
    Some(Bar {
        name: name.to_string(),
        selected,
        len,
        fills: bar.chars().filter(|c| FILLS.contains(c)).collect(),
        value,
        at: (r.x + width(&t[..ni]) as u16, r.y),
    })
}

impl Chart {
    /// 題から「グラフ:」と「(推定)」を除いた残り。
    fn core(&self) -> String {
        let t = self.title.replace("(推定)", "");
        let t = t.trim();
        let t = t.strip_prefix("グラフ").unwrap_or(t).trim_start();
        t.trim_start_matches([':', ':']).trim().to_string()
    }

    /// 題を ` · ` で切った部分。
    fn segs(&self) -> Vec<String> {
        self.core()
            .split(" · ")
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// 題の種類(棒・積み上げ棒・折れ線・散布図・ヒートマップ)。
    pub(super) fn kind(&self) -> String {
        self.segs()
            .into_iter()
            .find(|s| KINDS.contains(&s.as_str()))
            .unwrap_or_else(|| panic!("題に種類が無い: {:?}\n{}", self.title, self.raw))
    }

    /// 題の、印のグラフの名前(種類より前の部分。無ければ None)。
    pub(super) fn name(&self) -> Option<String> {
        let segs = self.segs();
        let k = segs.iter().position(|s| KINDS.contains(&s.as_str()))?;
        (k > 0).then(|| segs[..k].join(" · "))
    }

    /// 題の、分ける列(`<x> ごとの…` か、散布図の `<x> × <y>` の x)。
    pub(super) fn x(&self) -> String {
        let segs = self.segs();
        if let Some(s) = segs.iter().find(|s| s.contains("ごとの")) {
            return s[..s.find("ごとの").unwrap()].trim().to_string();
        }
        if let Some(s) = segs.iter().find(|s| s.contains(" × ")) {
            return s[..s.find(" × ").unwrap()].trim().to_string();
        }
        panic!("題に分ける列が無い: {:?}\n{}", self.title, self.raw)
    }

    /// 題の値(`件数`・`<列> の合計` など。散布図は縦の列)。
    pub(super) fn value(&self) -> String {
        let segs = self.segs();
        if let Some(s) = segs.iter().find(|s| s.contains("ごとの")) {
            let i = s.find("ごとの").unwrap() + "ごとの".len();
            return s[i..].trim().to_string();
        }
        if let Some(s) = segs.iter().find(|s| s.contains(" × ")) {
            let i = s.find(" × ").unwrap() + " × ".len();
            return s[i..].trim().to_string();
        }
        panic!("題に値が無い: {:?}\n{}", self.title, self.raw)
    }

    /// 題の内訳の列(` · 内訳 <列>`。無ければ None)。
    pub(super) fn by(&self) -> Option<String> {
        self.segs()
            .iter()
            .find_map(|s| s.strip_prefix("内訳").map(|r| r.trim().to_string()))
    }

    /// 題に推定と出ているか。
    pub(super) fn inferred(&self) -> bool {
        self.title.contains("推定")
    }

    /// 体の1行目(`<n> 行`)の番号。
    fn count_at(&self) -> usize {
        self.rows
            .iter()
            .position(|r| parse_counts(&r.text).is_some())
            .unwrap_or_else(|| panic!("体に「<n> 行」の行が無い:\n{}", self.raw))
    }

    /// 体の1行目の (行の数, 外した行の数)。
    pub(super) fn counts(&self) -> (usize, Option<usize>) {
        parse_counts(&self.rows[self.count_at()].text).unwrap()
    }

    /// 体の2行目(選んでいるものの説明)。
    pub(super) fn desc(&self) -> String {
        self.rows
            .get(self.count_at() + 1)
            .map(|r| r.text.trim().to_string())
            .unwrap_or_default()
    }

    /// 体の3行目(内訳があるときの凡例)。
    pub(super) fn legend(&self) -> String {
        self.rows
            .get(self.count_at() + 2)
            .map(|r| r.text.trim().to_string())
            .unwrap_or_default()
    }

    /// 選びの説明より下の行。
    pub(super) fn below(&self) -> Vec<&Row> {
        self.rows.iter().skip(self.count_at() + 2).collect()
    }

    /// 棒の行(上から)。
    pub(super) fn bars(&self) -> Vec<Bar> {
        self.below().into_iter().filter_map(bar_row).collect()
    }

    /// 棒 `name`。
    pub(super) fn bar(&self, name: &str) -> Bar {
        self.bars()
            .into_iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("棒 {name} が無い:\n{}", self.raw))
    }

    /// 棒の (名前, 値)。上から。
    pub(super) fn items(&self) -> Vec<(String, String)> {
        self.bars().into_iter().map(|b| (b.name, b.value)).collect()
    }

    /// 選んでいる棒の名前(`>` の付いた行)。
    pub(super) fn selected(&self) -> Vec<String> {
        self.bars()
            .into_iter()
            .filter(|b| b.selected)
            .map(|b| b.name)
            .collect()
    }

    /// 折れ線・散布図の格子の行(選びの説明より下で、軸の縦の線のある行)。
    pub(super) fn plot(&self) -> Vec<&Row> {
        self.below()
            .into_iter()
            .filter(|r| r.text.contains(SEPS))
            .collect()
    }

    /// 格子の中の点の字 `c` の数(軸の縦の線より右だけ)。
    pub(super) fn points(&self, c: char) -> usize {
        self.plot()
            .iter()
            .map(|r| {
                let i = r.text.find(SEPS).unwrap();
                r.text[i..].chars().filter(|x| *x == c).count()
            })
            .sum()
    }

    /// 格子の中の点の字 `c` の桁(体の中の桁。左から)。
    pub(super) fn point_cols(&self, c: char) -> Vec<usize> {
        let mut out = Vec::new();
        for r in self.plot() {
            let i = r.text.find(SEPS).unwrap();
            for (j, x) in r.text.char_indices().skip_while(|(j, _)| *j < i) {
                if x == c {
                    out.push(width(&r.text[..j]));
                }
            }
        }
        out.sort_unstable();
        out
    }

    /// 格子に点字(U+2800〜U+28FF)の線があるか。
    pub(super) fn has_braille(&self) -> bool {
        self.plot().iter().any(|r| {
            r.text
                .chars()
                .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
        })
    }

    /// 格子の左の目盛りの数(軸の縦の線の左に数だけがある行)。
    pub(super) fn scale(&self) -> Vec<f64> {
        self.plot()
            .iter()
            .filter_map(|r| {
                let i = r.text.find(SEPS).unwrap();
                r.text[..i].trim().parse::<f64>().ok()
            })
            .collect()
    }

    /// 選んでいる区切りの印 `^` の桁(体の中の桁)。
    pub(super) fn caret(&self) -> Option<usize> {
        self.below()
            .iter()
            .find_map(|r| r.text.find('^').map(|i| width(&r.text[..i])))
    }

    /// ヒートマップの、縦の値 `name` の行(選びの説明より下で、`>` と空白のあとに名前で始まる行)。
    pub(super) fn heat_row(&self, name: &str) -> Row {
        self.below()
            .into_iter()
            .find(|r| {
                let t = r.text.trim_start();
                let t = t.strip_prefix('>').unwrap_or(t).trim_start();
                t.strip_prefix(name)
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '│', '┃', '|']))
            })
            .cloned()
            .unwrap_or_else(|| panic!("ヒートマップに {name} の行が無い:\n{}", self.raw))
    }

    /// 行の中の数(名前と濃さの字と縦の線を除いた語のうち、数として読めるもの)。左から。
    pub(super) fn numbers(r: &Row) -> Vec<f64> {
        r.text
            .split(|c: char| c.is_whitespace() || "░▒▓█│┃|>".contains(c))
            .filter_map(|w| w.parse::<f64>().ok())
            .collect()
    }
}

/// 今の画面のグラフ(無ければ落ちる)。
pub(super) fn chart(a: &App) -> Chart {
    let s = screen(a);
    parse(&s).unwrap_or_else(|| panic!("グラフの枠(題「グラフ: …」)が無い: {:?}\n{s}", a.mode))
}

/// `P` でグラフを開き、開いた画面を返す(開かなければ落ちる)。
pub(super) fn open(a: &mut App) -> Chart {
    ch(a, 'P');
    assert_eq!(
        a.mode,
        Mode::Chart,
        "P でグラフが開かない: {:?}\n{}",
        a.message,
        screen(a)
    );
    chart(a)
}

/// グラフを開いたままか。
pub(super) fn assert_chart(a: &App) {
    let s = screen(a);
    assert!(
        a.mode == Mode::Chart && parse(&s).is_some(),
        "グラフのまま: {:?} {:?}\n{s}",
        a.mode,
        a.message
    );
}

/// 表に戻ったか(メッセージ行を除いた所で見る: ヘッダーの画面の型は表で、グラフの枠の題が無い)。
pub(super) fn assert_table(a: &App) {
    let s = screen(a);
    assert_eq!(a.mode, Mode::Table, "{:?}\n{s}", a.message);
    let body = above_message(&s);
    assert!(body.contains("表 ▾"), "ヘッダーの画面の型は表:\n{s}");
    assert!(
        !body.lines().any(is_chart_title),
        "表に戻るとグラフの枠の題は消える:\n{s}"
    );
}

/// 行のノートの名前(拡張子なし)。
fn lab(a: &App, r: &mdgrid::source::RowId) -> String {
    a.src.label(r).trim_end_matches(".md").to_string()
}

/// 表の行の名前(表の順)。
pub(super) fn row_names(a: &App) -> Vec<String> {
    a.rows.iter().map(|r| lab(a, r)).collect()
}

/// 表で選んでいる行の名前。
pub(super) fn cur(a: &App) -> String {
    lab(a, &a.cur_row().expect("表の行"))
}

/// 表の行より上(ヘッダー・ビューの帯・設定の帯・列の見出し)の文字。
pub(super) fn top_rows(a: &App) -> String {
    screen(a)
        .lines()
        .take(super::view::data_y(a))
        .collect::<Vec<_>>()
        .join("\n")
}

/// (名前, 値) の並び。
pub(super) fn want(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(n, v)| (n.to_string(), v.to_string()))
        .collect()
}

/// 棒の長さが最大の棒の `num/den` か(1字の幅をもたせる)。
pub(super) fn assert_ratio(c: &Chart, name: &str, max: &str, num: f64, den: f64) {
    let full = c.bar(max).len;
    let got = c.bar(name).len;
    let expect = full * num / den;
    assert!(
        full >= 6.0,
        "最大の棒({max})は数字の比を見られる長さ: {full}\n{}",
        c.raw
    );
    assert!(
        (got - expect).abs() <= 1.0,
        "{name} の棒は {max} の {num}/{den}(全長 {full} → {expect}): {got}\n{}",
        c.raw
    );
}

// ---- CH-1: 印のグラフ ----

#[test]
fn test_ch_1_setting_bar_opens_with_p() {
    // [utest -> req~chart.config-one~1, req~chart.config-names~1, req~chart.config-defaults~1] chart = { kind = "bar", x = "status" } → 警告なし。P で status ごとの件数の棒。
    // 項目は件数の大きい順(done 3・todo 2・doing 1)で、棒の値は件数。
    let (_t, mut a) = boot(
        "ch1bar",
        &status_notes(),
        "",
        "chart = { kind = \"bar\", x = \"status\" }",
    );
    assert_eq!(a.message, None, "警告が無い");
    let c = open(&mut a);
    assert_eq!(c.kind(), "棒", "{}", c.raw);
    assert_eq!(c.x(), "status", "{}", c.raw);
    assert_eq!(c.value(), "件数", "{}", c.raw);
    assert_eq!(
        c.items(),
        want(&[("done", "3"), ("todo", "2"), ("doing", "1")]),
        "{}",
        c.raw
    );
}

#[test]
fn test_ch_1_array_brackets_switch_saved_charts() {
    // [utest -> req~chart.config-array~1, req~chart.keys-charts~1] chart = [{ name = "状態", x = "status" }, { name = "月ごと", kind = "line", x = "due" }] →
    // P で1つ目(題に名前「状態」と棒)。] で2つ目の折れ線(題に「月ごと」と due)。[ で1つ目に戻る。
    let notes = md(&[
        ("alpha", "status: todo\ndue: 2026-10-03"),
        ("beta", "status: done\ndue: 2026-10-20"),
        ("gamma", "status: done\ndue: 2026-11-02"),
    ]);
    let (_t, mut a) = boot(
        "ch1array",
        &notes,
        "[types]\ndue = \"date\"\n",
        "chart = [{ name = \"状態\", x = \"status\" }, { name = \"月ごと\", kind = \"line\", x = \"due\" }]",
    );
    assert_eq!(a.message, None, "警告が無い");
    let c = open(&mut a);
    assert_eq!(c.name().as_deref(), Some("状態"), "{}", c.raw);
    assert_eq!(c.kind(), "棒");
    assert_eq!(c.x(), "status");
    ch(&mut a, ']');
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(c.name().as_deref(), Some("月ごと"), "{}", c.raw);
    assert_eq!(c.kind(), "折れ線", "{}", c.raw);
    assert_eq!(c.x(), "due", "{}", c.raw);
    ch(&mut a, '[');
    let c = chart(&a);
    assert_eq!(c.name().as_deref(), Some("状態"), "{}", c.raw);
    assert_eq!(c.kind(), "棒");
}

#[test]
fn test_ch_1_unknown_kind_warns_and_that_chart_is_not_shown() {
    // [utest -> req~chart.config-invalid~1, req~chart.config-missing-column~2] kind = "pie" → 読み終えて警告1行(CLI-3 の形。場所 table[1].chart)で、そのグラフは出ない。
    // P は推定のグラフで開き、題に名前「円」は無い。
    let (_t, mut a) = boot(
        "ch1pie",
        &status_notes(),
        "",
        "chart = { name = \"円\", kind = \"pie\", x = \"status\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].chart"),
        "警告: {m:?}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    let c = open(&mut a);
    assert!(!c.title.contains("円"), "外したグラフは出ない: {}", c.raw);
    assert!(c.inferred(), "設定が無いので推定: {}", c.raw);
}

#[test]
fn test_ch_1_array_keeps_good_charts_when_one_is_bad() {
    // [utest -> req~chart.config-invalid~1] 配列の1つ目が知らない種類 → 警告1行で1つ目だけを外し、2つ目(状態)で開く。] [ でも「円」は出ない。
    let (_t, mut a) = boot(
        "ch1arraybad",
        &status_notes(),
        "",
        "chart = [{ name = \"円\", kind = \"pie\" }, { name = \"状態\", x = \"status\" }]",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(m.contains("table[1].chart"), "警告: {m:?}");
    let c = open(&mut a);
    assert_eq!(c.name().as_deref(), Some("状態"), "{}", c.raw);
    for k in [']', '[', ']'] {
        ch(&mut a, k);
        assert_chart(&a);
        let c = chart(&a);
        assert!(!c.title.contains("円"), "{k}: {}", c.raw);
    }
}

#[test]
fn test_ch_1_missing_column_warns_once_after_loading() {
    // [utest -> req~chart.config-missing-column~2] x = "none"(表に無い列)→ 読み終えて警告1行(CLI-3 の形。場所 table[1].chart と列の名前 none)。
    // そのグラフは外れ、P は推定のグラフで開く(題に名前「無い列」は無い)。警告は1回だけ(開き直しても・
    // 変化の無い poll でも出し直さない)。
    let (_t, mut a) = boot(
        "ch1none",
        &status_notes(),
        "",
        "chart = { name = \"無い列\", x = \"none\" }",
    );
    let m = a.message.clone().unwrap_or_default();
    assert!(
        m.contains(".mdgrid/workspace.toml: table[1].chart") && m.contains("none"),
        "警告: {m:?}"
    );
    assert!(!m.contains('\n'), "警告は1行: {m:?}");
    a.message = None;
    let c = open(&mut a);
    assert!(!c.title.contains("無い列"), "{}", c.raw);
    assert_eq!(c.x(), "status", "推定の列: {}", c.raw);
    press_back(&mut a);
    a.poll();
    open(&mut a);
    press_back(&mut a);
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("table[1].chart"), "警告は1回: {m:?}");
}

/// `Esc` で表に戻る。
fn press_back(a: &mut App) {
    super::test_screen::press(a, KeyCode::Esc);
    assert_table(a);
}

#[test]
fn test_ch_1_empty_table_does_not_warn_and_p_gives_reason() {
    // [utest -> req~chart.config-missing-column~2] 行が1つも無い表で x = "none" → 列があるかを決めず、読み終えても警告しない(GA-1 と同じ)。
    // 画面の型のメニューに「グラフ」(キー P)が出て、P は開かずに理由1行。
    let (_t, mut a) = boot("ch1empty", &[], "", "chart = { x = \"none\" }");
    assert!(a.rows.is_empty(), "前提: 0件");
    let m = a.message.clone().unwrap_or_default();
    assert!(!m.contains("table[1].chart"), "警告しない: {m:?}");
    let menu = menu_text(&mut a);
    assert!(
        menu_item(&menu, "グラフ", "P").is_some(),
        "メニューに「グラフ」:\n{menu}"
    );
    a.message = None;
    ch(&mut a, 'P');
    assert_eq!(a.mode, Mode::Table, "開かない: {:?}", a.message);
    reason(&a);
}

// ---- CH-2: 開く・推定・題 ----

#[test]
fn test_ch_2_infers_status_bar_and_title_says_so() {
    // [utest -> req~chart.open-title~1] 設定なしで status の列(todo 2・doing 1・done 3)→ P で棒が3本。題に棒・status・件数と「推定」、
    // 体の1行目に数えた行の数「6 行」(外した行は無い)。
    let (_t, mut a) = boot("ch2infer", &status_notes(), "", "");
    assert_eq!(a.message, None, "{:?}", a.message);
    let c = open(&mut a);
    assert_eq!(c.kind(), "棒", "{}", c.raw);
    assert_eq!(c.x(), "status", "{}", c.raw);
    assert_eq!(c.value(), "件数", "{}", c.raw);
    assert!(c.inferred(), "題に推定: {}", c.raw);
    assert_eq!(c.counts(), (6, None), "{}", c.raw);
    assert_eq!(c.bars().len(), 3, "{}", c.raw);
}

#[test]
fn test_ch_2_infers_rule_column_without_status() {
    // [utest -> req~chart.open-infer~2] 状態の名前の列が無い → 決まり(values)のある列(phase)を推定する。最初の列(title)ではない。
    let notes = md(&[
        ("alpha", "title: a\nphase: plan"),
        ("beta", "title: b\nphase: build"),
        ("gamma", "title: c\nphase: plan"),
    ]);
    let (_t, mut a) = boot(
        "ch2rule",
        &notes,
        "[rules.phase]\nvalues = [\"plan\", \"build\"]\n",
        "",
    );
    let c = open(&mut a);
    assert_eq!(c.x(), "phase", "{}", c.raw);
    assert!(c.inferred(), "{}", c.raw);
    assert_eq!(
        c.items(),
        want(&[("plan", "2"), ("build", "1")]),
        "{}",
        c.raw
    );
}

#[test]
fn test_ch_2_menu_offers_chart_with_p_and_opens_it() {
    // [utest -> req~chart.open-screen~1, req~relations.screen-menu-open~2] 画面の型のメニューに「グラフ」(切り替えのキー P)が出て、選ぶとグラフが開く。P で表に戻る。
    let (_t, mut a) = boot("ch2menu", &status_notes(), "", "");
    let s = screen(&a);
    let (x, y) = s
        .lines()
        .enumerate()
        .find_map(|(y, l)| l.find("表 ▾").map(|i| (width(&l[..i]) as u16, y as u16)))
        .unwrap_or_else(|| panic!("ヘッダーのメニュー:\n{s}"));
    a.click(x, y);
    assert_eq!(a.mode, Mode::ScreenMenu, "{:?}", a.message);
    let s = screen(&a);
    let (x, y) = menu_item(&s, "グラフ", "P")
        .unwrap_or_else(|| panic!("窓に「グラフ」と切り替えのキー P:\n{s}"));
    a.click(x, y);
    assert_chart(&a);
    let c = chart(&a);
    assert_eq!(c.kind(), "棒", "{}", c.raw);
    ch(&mut a, 'P');
    assert_table(&a);
}

#[test]
fn test_ch_2_counts_only_view_rows() {
    // [utest -> req~chart.open-screen~1] 今のビューの行(絞り込みのあと。TR-1 と同じ)だけを数える: priority = p2 で絞る → 3行で、
    // todo・doing・done が1つずつ(同じ数は名前の順)。
    let notes = md(&[
        ("alpha", "status: todo\npriority: p2"),
        ("beta", "status: todo\npriority: p1"),
        ("gamma", "status: doing\npriority: p2"),
        ("delta", "status: done\npriority: p2"),
        ("epsilon", "status: done\npriority: p1"),
        ("zeta", "status: done\npriority: p1"),
    ]);
    let (_t, mut a) = boot("ch2view", &notes, "", "");
    assert_eq!(row_names(&a)[0], "alpha", "前提: 名前の順");
    a.row = 0;
    col_named(&mut a, "priority");
    ch(&mut a, ',');
    assert_eq!(
        row_names(&a),
        ["alpha", "delta", "gamma"],
        "前提: p2 で絞る"
    );
    let c = open(&mut a);
    assert_eq!(c.counts(), (3, None), "{}", c.raw);
    assert_eq!(
        c.items(),
        want(&[("doing", "1"), ("done", "1"), ("todo", "1")]),
        "{}",
        c.raw
    );
}

#[test]
fn test_ch_2_table_without_columns_gives_reason() {
    // [utest -> req~chart.open-refuse~2] 列が1つも無い表(フロントマターの無いノートだけ)→ P は開かずに理由1行。メニューにも出ない。
    let notes = vec![
        ("alpha.md".to_string(), "# alpha\n本文\n".to_string()),
        ("beta.md".to_string(), "# beta\n".to_string()),
    ];
    let (_t, mut a) = boot("ch2nocol", &notes, "", "");
    assert_eq!(a.rows.len(), 2, "前提: 行はある");
    assert!(a.src.columns().is_empty(), "前提: 列が無い");
    let menu = menu_text(&mut a);
    assert!(menu_item(&menu, "グラフ", "P").is_none(), "{menu}");
    a.message = None;
    ch(&mut a, 'P');
    assert_table(&a);
    reason(&a);
}

#[test]
fn test_ch_2_empty_view_gives_reason() {
    // [utest -> req~chart.open-refuse~2] ビューの絞り込みで行が1つも無い(列はある)→ P は開かずに理由1行。
    let (_t, mut a) = boot("ch2noview", &status_notes(), "", "");
    ch(&mut a, '\\');
    typing(&mut a, "zzzz");
    super::test_screen::press(&mut a, KeyCode::Enter);
    assert!(a.rows.is_empty(), "前提: 絞って0行");
    a.message = None;
    ch(&mut a, 'P');
    assert_eq!(a.mode, Mode::Table, "開かない: {:?}", a.message);
    reason(&a);
}
