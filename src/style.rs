//! 部品の形の設定(SR-36): `look.preset`(組の名前)と、`[look.style]` の部品ごとの形。組で部品をまとめて選び、
//! 部品ごとの項目はそれを上書きする。既定の組は sumi。名前の並びはカタログ(docs/catalog/index.html。SR-38)と
//! 同じにし、試験で突き合わせる。

/// 名前つきの形の列挙を作る(設定の名前 ⇔ 値)。
macro_rules! named {
    ($(#[$m:meta])* $ty:ident { $($v:ident = $s:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $ty { $($v),+ }
        impl $ty {
            /// 設定で受ける名前の並び(カタログと同じ順)。
            pub const NAMES: &'static [&'static str] = &[$($s),+];
            pub fn name(self) -> &'static str { match self { $($ty::$v => $s),+ } }
            pub fn parse(s: &str) -> Option<$ty> { match s { $($s => Some($ty::$v),)+ _ => None } }
        }
    };
}

named!(
    /// くり返す短い値(status・owner など)の形。
    Status { Dot = "dot", Shape = "shape", Text = "text", Pill = "pill", Tint = "tint", Solid = "solid", Soft = "soft", Chip = "chip", Plain = "plain" }
);
named!(
    /// リストの値の形。
    Tags { Dots = "dots", Hash = "hash", Brackets = "brackets", Pill = "pill", Tint = "tint", Solid = "solid", Soft = "soft", Chip = "chip", Plain = "plain" }
);
named!(
    /// 真偽の値の形。
    Check { Box_ = "box", Tick = "tick", Bracket = "bracket", Text = "text" }
);
named!(
    /// 選んでいる行とセルの形。
    Select { Bar = "bar", Cross = "cross", Tint = "tint", Outline = "outline", Fill = "fill", Reverse = "reverse" }
);
named!(
    /// 表の線。
    Rules { None_ = "none", Header = "header", Columns = "columns", Grid = "grid" }
);
named!(
    /// ビューのタブの形。
    Tabs { Underline = "underline", Pill = "pill", Segment = "segment", Brackets = "brackets", Dim = "dim" }
);
named!(
    /// 窓の枠。
    Frames { Rounded = "rounded", Square = "square", Heavy = "heavy", Ascii = "ascii", None_ = "none" }
);
named!(
    /// 下の帯のキーの見せ方。
    Band { Keys = "keys", Boxed = "boxed", Quiet = "quiet" }
);
named!(
    /// リンクの見せ方。
    Links { Accent = "accent", Plain = "plain" }
);
named!(
    /// 組の名前。
    Preset { Sumi = "sumi", Slate = "slate", Saas = "saas", Paper = "paper", Grid = "grid", Classic = "classic", DozyPink = "dozy-pink" }
);

/// 部品の形の全部。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub preset: Preset,
    pub status: Status,
    pub tags: Tags,
    pub check: Check,
    pub select: Select,
    pub rules: Rules,
    pub tabs: Tabs,
    pub frames: Frames,
    pub band: Band,
    pub links: Links,
    pub icons: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style::of(Preset::Sumi)
    }
}

impl Style {
    /// 組の部品(カタログの「まとめて選ぶ」と同じ中身)。
    pub fn of(p: Preset) -> Style {
        use Preset as P;
        let (status, tags, check, select, rules, tabs, frames, band, icons) = match p {
            P::Sumi => (
                Status::Dot,
                Tags::Dots,
                Check::Tick,
                Select::Bar,
                Rules::Header,
                Tabs::Underline,
                Frames::Rounded,
                Band::Quiet,
                false,
            ),
            P::Slate => (
                Status::Shape,
                Tags::Hash,
                Check::Box_,
                Select::Bar,
                Rules::Header,
                Tabs::Pill,
                Frames::Rounded,
                Band::Keys,
                true,
            ),
            P::Saas => (
                Status::Tint,
                Tags::Tint,
                Check::Box_,
                Select::Tint,
                Rules::Header,
                Tabs::Segment,
                Frames::Rounded,
                Band::Quiet,
                false,
            ),
            P::Paper => (
                Status::Text,
                Tags::Plain,
                Check::Bracket,
                Select::Tint,
                Rules::Columns,
                Tabs::Brackets,
                Frames::Square,
                Band::Quiet,
                false,
            ),
            P::Grid => (
                Status::Plain,
                Tags::Plain,
                Check::Tick,
                Select::Outline,
                Rules::Grid,
                Tabs::Dim,
                Frames::Square,
                Band::Boxed,
                true,
            ),
            // テーマ dozy-pink に合う柔らかい組: 丸い札(描けなければ角を落とした札)・左の線・線なし・塗ったタブ。
            P::DozyPink => (
                Status::Pill,
                Tags::Pill,
                Check::Box_,
                Select::Bar,
                Rules::None_,
                Tabs::Pill,
                Frames::Rounded,
                Band::Quiet,
                false,
            ),
            P::Classic => (
                Status::Chip,
                Tags::Chip,
                Check::Box_,
                Select::Fill,
                Rules::None_,
                Tabs::Pill,
                Frames::Rounded,
                Band::Keys,
                true,
            ),
        };
        Style {
            preset: p,
            status,
            tags,
            check,
            select,
            rules,
            tabs,
            frames,
            band,
            links: Links::Accent,
            icons,
        }
    }
}

/// 設定の項目の名前(`[look.style]` の中)。カタログと突き合わせる。
pub const KEYS: &[&str] = &[
    "status", "tags", "check", "select", "rules", "tabs", "frames", "band", "links", "icons",
];

/// SR-36: `nerd_font = "auto"` の見分け。丸い端の字(U+E0B6・U+E0B4)を字体に頼らず自分で描く端末
/// (`TERM_PROGRAM` が ghostty か WezTerm)なら真。ほか(分からない端末・tmux の中など)は偽。
pub fn nerd_auto(term_program: Option<&str>) -> bool {
    matches!(
        term_program
            .map(|t| t.trim().to_ascii_lowercase())
            .as_deref(),
        Some("ghostty" | "wezterm")
    )
}

#[cfg(test)]
#[path = "test_style_unit.rs"]
mod tests;

#[cfg(test)]
#[path = "test_nerd_auto_unit.rs"]
mod nerd_tests;

#[cfg(test)]
#[path = "test_dozy_unit.rs"]
mod dozy_tests;
