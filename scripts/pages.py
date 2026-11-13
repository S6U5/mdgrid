#!/usr/bin/env python3
"""GitHub Pages のサイトを作る(日英)。

    python3 scripts/pages.py [出力のフォルダ(既定 site)] [--shots 画面のフォルダ(既定 target/shots)]

先に画面を撮る(本物の mdgrid を疑似端末で動かす。cargo build --release のあと):

    python3 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang ja --env LANG=ja_JP.UTF-8
    python3 scripts/tui_shot.py docs/manual-scenarios.toml --out target/shots --lang en --env LANG=en_US.UTF-8

作るもの:
- index.html    言語を選ぶ入口(x-default)。覚えた選択かブラウザの言語で ja/・en/ へ移る
- ja/・en/      入口(説明書・カタログ・画面の一覧へ)と画面の一覧(gallery.html)。hreflang で互いを示す
- manual/ja・en 説明書(mdBook)。docs/manual/<言語>/*.md と参照のページ(キー・設定・安全・.base)を
                1冊にまとめ、ページの間のリンクを本の中に直す。本の外を指すリンクは GitHub の上のファイルへ
- catalog/      見た目のカタログ(docs/catalog/index.html をそのまま。SR-38)
- gallery.html  前の画面の一覧の URL(言語を決めて ja/・en/ の画面の一覧へ移る)
- images/ja・en 撮った画面の SVG
- sitemap.xml   検索エンジンに出すサイトの地図(公開するページだけ。検索の管理に直に出す)

どのページの head にも、正規の URL(canonical)と、貼ったときのカードのタグ(OGP・Twitter Card)を入れる。
カードの画像は docs/assets/og.png・docs/assets/ja/og.png(scripts/og_image.py で作り直せる)。

環境変数 GOOGLE_SITE_VERIFICATION があれば、入口(index.html)に検索の管理の持ち主の確認のタグを足す
(workflow は公開リポの Actions の変数から渡す。値はページに出るもので秘密ではない)。

画面の画像はリポに入れない(撮るたびに履歴が太るため)。mdbook が要る。Python 3.11 以上(tomllib)。
"""

import html
import json
import os
import re
import subprocess
import shutil
import sys
import tomllib
import urllib.parse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LANGS = ("ja", "en")
REPO = "https://github.com/S6U5/mdgrid"

TEXT = {
    "ja": {
        "title": "mdgrid",
        "lead": "フロントマター付きの Markdown のノートを、ターミナルの表で見て直す道具。",
        "manual": "説明書",
        "manual_desc": "見せ方の一覧・できることの一覧・はじめかた・作業の手引き・考え方・見た目・困ったとき、キーと設定の一覧。",
        "catalog": "見た目のカタログ",
        "catalog_desc": "テーマと部品の形を触って選び、config.toml に貼る設定を作る。",
        "views": "見せ方の一覧",
        "views_desc": "表・まとまり・木・関係マップ・値の分布・テーマ。どんな画面があるかを1枚ずつ。",
        "features": "できることの一覧",
        "features_desc": "タスク・ブックマーク・読書・人と会議の場面と、直す・保存する・絞る・つなぐ・作る・出す。機能ごとに画面1枚。",
        "gallery": "画面の一覧",
        "gallery_desc": "本物の mdgrid の画面を、場面ごとに見る(説明書と同じ写し)。",
        "repo": "リポジトリ",
        "demos": "動き(GIF)",
        "demos_desc": "リリースに付けた録画。",
        "back": "入口へ",
        "filter": "場面を絞る",
        "count": "{n} 場面",
        "keys": "押したキー",
        "lang": "言語",
        "start": "はじめかた",
        "copy": "写す",
        "copied": "写した",
        "mode": "明るい・暗いを切り替える",
    },
    "en": {
        "title": "mdgrid",
        "lead": "View and edit Markdown notes with frontmatter as a table in the terminal.",
        "manual": "Manual",
        "manual_desc": "Ways to see your notes, what mdgrid can do, getting started, how-tos, concepts, appearance, troubleshooting, and the key and setting lists.",
        "catalog": "Look catalog",
        "catalog_desc": "Try themes and part shapes, then copy the settings into config.toml.",
        "views": "Ways to see your notes",
        "views_desc": "Table, groups, tree, relation map, value counts, themes. One screen each.",
        "features": "What mdgrid can do",
        "features_desc": "Tasks, bookmarks, reading, people and meetings, then editing, saving, filtering, linking, creating, exporting. One screen per feature.",
        "gallery": "Screen gallery",
        "gallery_desc": "Real mdgrid screens, scene by scene (the same shots as the manual).",
        "repo": "Repository",
        "demos": "In motion (GIF)",
        "demos_desc": "Recordings attached to the latest release.",
        "back": "Home",
        "filter": "Filter scenes",
        "count": "{n} scenes",
        "keys": "Keys pressed",
        "lang": "Language",
        "start": "Get started",
        "copy": "Copy",
        "copied": "Copied",
        "mode": "Switch light and dark",
    },
}

DEMOS = [
    ("demo-values.gif", "型ごとの入力と保存", "Typed inputs and saving"),
    ("demo-new-note.gif", "新しいノートのフォーム", "New note form"),
    ("demo-export.gif", "CSV と --print に書き出す", "Export to CSV and --print"),
    ("demo-places.gif", "登録した表を切り替える", "Switching registered tables"),
    ("demo-relations.gif", "リレーション", "Relations"),
    ("demo-relmap.gif", "関係マップ", "Relation map"),
    ("demo-workspace.gif", "ワークスペース", "Workspaces"),
    ("demo-view-tabs.gif", "ビューのタブを整える", "Arranging view tabs"),
    ("demo-look.gif", "見た目を選んで保存", "Picking and saving a look"),
    ("demo-tree.gif", "親子の字下げ", "Parent/child indent"),
    ("demo-wbs.gif", "WBS", "WBS"),
    ("demo-themes.gif", "色のテーマ", "Color themes"),
]

# 見た目(CSS)と入口の動き(JS)は scripts/site/ に置き、各ページに埋める(1枚の HTML で完結し、外から何も読まない)。
CSS = (ROOT / "scripts/site/site.css").read_text(encoding="utf-8")
LANDING_JS = (ROOT / "scripts/site/landing.js").read_text(encoding="utf-8")

SITE = "https://s6u5.github.io/mdgrid/"
NAMES = {"ja": "日本語", "en": "English"}
KEY = "mdgrid-site-lang"

# 言語の切り替え(data-pick)を押したら、選んだ言語を覚える(入口の振り分けが使う)。読み書きの失敗は無視。
REMEMBER = """
document.addEventListener("click", function (e) {
  var a = e.target.closest && e.target.closest("a[data-pick]");
  if (a) { try { localStorage.setItem("%s", a.getAttribute("data-pick")); } catch (x) {} }
});
""" % KEY

# 画面の一覧の絞り込み。
FIND = """
var f = document.getElementById("find");
if (f) f.addEventListener("input", function () {
  var w = f.value.trim().toLowerCase();
  document.querySelectorAll("figure[data-search]").forEach(function (x) { x.hidden = w !== "" && x.dataset.search.indexOf(w) < 0; });
});
"""


def other_lang(l: str) -> str:
    return "en" if l == "ja" else "ja"


def alternates(rel: str, default: str = SITE) -> str:
    """`rel` は ja の版のサイトの中の道筋(例 "ja/gallery.html")。ja・en と x-default の hreflang。
    x-default(`default`)は言語を決めて移るページ(入口は根、画面の一覧は gallery.html)。そのページも同じ組を示す。"""
    out = [f'<link rel="alternate" hreflang="{l}" href="{SITE}{rel.replace("ja/", l + "/", 1)}">' for l in LANGS]
    out.append(f'<link rel="alternate" hreflang="x-default" href="{default}">')
    return "\n".join(out)


# 貼ったときのカードの画像(docs/assets/og.png・docs/assets/ja/og.png を site/assets/<言語>/og.png に写す)。
OG_W, OG_H = 1200, 630
OG_LOCALE = {"ja": "ja_JP", "en": "en_US"}


def social(lang: str, title: str, desc: str, url: str) -> str:
    """正規の URL(canonical)と、貼ったときのカードのタグ(OGP・Twitter Card)。`url` はそのページの公開の URL。"""
    t, d, u = html.escape(title), html.escape(desc), html.escape(url)
    img = f"{SITE}assets/{lang}/og.png"
    return "\n".join(
        [
            f'<link rel="canonical" href="{u}">',
            '<meta property="og:type" content="website">',
            '<meta property="og:site_name" content="mdgrid">',
            f'<meta property="og:locale" content="{OG_LOCALE[lang]}">',
            f'<meta property="og:url" content="{u}">',
            f'<meta property="og:title" content="{t}">',
            f'<meta property="og:description" content="{d}">',
            f'<meta property="og:image" content="{img}">',
            f'<meta property="og:image:width" content="{OG_W}">',
            f'<meta property="og:image:height" content="{OG_H}">',
            f'<meta property="og:image:alt" content="{html.escape(OG_ALT[lang])}">',
            '<meta name="twitter:card" content="summary_large_image">',
            f'<meta name="twitter:title" content="{t}">',
            f'<meta name="twitter:description" content="{d}">',
            f'<meta name="twitter:image" content="{img}">',
        ]
    )


MODE_KEY = "mdgrid-site-theme"
# 描く前に: JS があることの印と、覚えた明るい・暗いの選択(ちらつかせない)。
EARLY = (
    'document.documentElement.classList.add("js");'
    'try{var m=localStorage.getItem("%s");if(m==="light"||m==="dark")document.documentElement.setAttribute("data-theme",m)}catch(e){}'
) % MODE_KEY
# 明るい・暗いの切り替え(上の帯のボタン)。選択は覚える。
MODE = """
(function () {
  var b = document.getElementById("mode");
  if (!b) return;
  b.addEventListener("click", function () {
    var r = document.documentElement, now = r.getAttribute("data-theme");
    if (!now) now = window.matchMedia && matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    var next = now === "dark" ? "light" : "dark";
    r.setAttribute("data-theme", next);
    try { localStorage.setItem("%s", next); } catch (e) {}
  });
})();
""" % MODE_KEY

# 名前の横の印(表の枠と、選んだセル)。タブの印(favicon)にも同じ形を使う。
LOGO = (
    '<svg viewBox="0 0 20 20" aria-hidden="true"><rect x="1.2" y="1.2" width="17.6" height="17.6" rx="4" fill="none" stroke="currentColor" stroke-width="1.7"/>'
    '<path d="M1.2 7.4h17.6M7.4 7.4v11.4" stroke="currentColor" stroke-width="1.7"/><rect x="9.7" y="9.7" width="6.8" height="3.4" rx="1" fill="currentColor"/></svg>'
)
FAVICON = "data:image/svg+xml," + urllib.parse.quote(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="7" fill="#1e1e1e"/>'
    '<g transform="translate(6 6)" color="#3fc3e6">' + LOGO.replace(' aria-hidden="true"', "").replace("<svg", '<svg width="20" height="20"') + "</g></svg>"
)
ICON_COPY = '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><rect x="5.5" y="5.5" width="8" height="8" rx="1.5"/><path d="M10.5 5.5v-2a1 1 0 0 0-1-1h-6a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2"/></svg>'
ICON_MODE = '<svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M8 1.8a6.2 6.2 0 0 1 0 12.4z" fill="currentColor"/></svg>'


def page(lang: str, title: str, body: str, desc: str, url: str, head: str = "", script: str = "", top: str = "", wide: bool = False) -> str:
    """1枚のページ。`url` はそのページの公開の URL(canonical とカードのタグに入れる)。`top` は上の帯、
    `wide` は幅いっぱいの段を自分で持つページ(入口)。"""
    main = body if wide else f'<div class="wrap">\n{body}\n</div>'
    GA = ANALYTICS + "\n" if ANALYTICS else ""
    head = social(lang, title, desc, url) + ("\n" + head if head else "")
    return f"""<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{html.escape(title)}</title>
<meta name="description" content="{html.escape(desc)}">
<link rel="icon" href="{FAVICON}">
{GA}{head}
<script>{EARLY}</script>
<style>{CSS}</style>
</head>
<body>
{top}
{main}
<script>{REMEMBER}{MODE}{script}</script>
</body>
</html>
"""


def lang_bar(lang: str, file: str) -> str:
    """同じページの別の言語へのリンク(今の言語は印だけ)。"""
    t = TEXT[lang]
    items = []
    for l in LANGS:
        if l == lang:
            items.append(f'<span class="opt on" aria-current="true">{NAMES[l]}</span>')
        else:
            items.append(f'<a class="opt" href="../{l}/{file}" hreflang="{l}" lang="{l}" data-pick="{l}">{NAMES[l]}</a>')
    return f'<div class="bar"><span class="muted">{t["lang"]}</span><div class="chips">{"".join(items)}</div></div>'


def build(out: Path, shots_dir: Path) -> None:
    scen = tomllib.loads((ROOT / "docs/manual-scenarios.toml").read_text(encoding="utf-8"))
    verify = verification()  # 値がおかしければ、作り始める前に止める
    global ANALYTICS
    ANALYTICS = analytics()
    if out.exists():
        shutil.rmtree(out)
    (out / "catalog").mkdir(parents=True)
    catalog = (ROOT / "docs/catalog/index.html").read_text(encoding="utf-8")
    # カタログは1ファイルのまま(docs/ の元は変えない)。head に正規の URL とカードのタグだけを足す。
    # ?lang= を付けた URL も catalog/ に正規化する。
    catalog = catalog.replace(
        "</head>", social("ja", head_text(catalog, "title"), head_text(catalog, "description"), SITE + "catalog/") + "\n</head>", 1
    )
    if ANALYTICS:
        catalog = catalog.replace("</head>", ANALYTICS + "\n</head>", 1)
    (out / "catalog/index.html").write_text(catalog, encoding="utf-8")
    shots = []
    for s in scen["shot"]:
        sid = s["id"]
        # README の動きの GIF を作るためのコマ(gif-*)は、画面の一覧に出さない。
        if sid.startswith("gif-"):
            continue
        missing = [l for l in LANGS if not (shots_dir / l / "images" / f"{sid}.svg").exists()]
        if missing:
            raise SystemExit(f"画面が無い: {sid}({shots_dir}/{missing[0]}/images。先に tui_shot.py で撮る)")
        shots.append(s)
    for l in LANGS:
        d = out / "images" / l
        d.mkdir(parents=True)
        for s in shots:
            shutil.copy(shots_dir / l / "images" / f"{s['id']}.svg", d / f"{s['id']}.svg")
        book(l, out / "manual" / l, shots_dir / l / "images")
        decorate_book(l, out / "manual" / l)

    # 録画(GIF): workflow がリリースから target/demos に取ってくる(リポには入れない)。サイトに写して
    # ページの上でその場で再生する。取ってきた分だけを並べる(新しい台本の録画は次のリリースまで無い)。
    demo_dir = ROOT / "target" / "demos"
    shown = [d for d in DEMOS if (demo_dir / d[0]).exists()]
    if shown:
        (out / "demos").mkdir()
        for d in shown:
            shutil.copy(demo_dir / d[0], out / "demos" / d[0])

    for l in LANGS:
        (out / "assets" / l).mkdir(parents=True)
        src = ROOT / ("docs/assets/ja/demo.gif" if l == "ja" else "docs/assets/demo.gif")
        shutil.copy(src, out / "assets" / l / "demo.gif")
        shutil.copy(ROOT / ("docs/assets/ja/og.png" if l == "ja" else "docs/assets/og.png"), out / "assets" / l / "og.png")
    for l in LANGS:
        (out / l).mkdir()
        (out / l / "index.html").write_text(landing(l, shown, shots), encoding="utf-8")
        (out / l / "gallery.html").write_text(gallery(l, shots), encoding="utf-8")
    (out / "index.html").write_text(chooser(verify=verify), encoding="utf-8")
    # 前の画面の一覧の URL(README の古いリンク)は、言語を決めて新しい URL へ。
    (out / "gallery.html").write_text(chooser("gallery.html"), encoding="utf-8")
    urls = sitemap(out)
    print(json.dumps({"out": str(out), "shots": len(shots), "sitemap": urls}))


def verification() -> str:
    """検索の管理の持ち主の確認のタグ(環境変数 GOOGLE_SITE_VERIFICATION の値)。無ければ空。
    値は英数字・-・_ だけを受ける(タグに変な字を入れない)。"""
    v = os.environ.get("GOOGLE_SITE_VERIFICATION", "").strip()
    if not v:
        return ""
    if not re.fullmatch(r"[A-Za-z0-9_-]+", v):
        raise SystemExit("GOOGLE_SITE_VERIFICATION は英数字・-・_ だけにする(メタタグの content の値だけを入れる)")
    return f'<meta name="google-site-verification" content="{v}">'


# アクセス解析(GA4)の計測のタグ。build() が analytics() で埋める(無ければ空)。
ANALYTICS = ""
HOST = SITE.split("/")[2]

# gtag.js は公開のホストでだけ読み込む(手元・CI の検査では Google へ何も送らず、数も汚さない)。
GTAG = """<script>
(function () {
  if (location.hostname !== "%s") return;
  var s = document.createElement("script"); s.async = true;
  s.src = "https://www.googletagmanager.com/gtag/js?id=%s"; document.head.appendChild(s);
  window.dataLayer = window.dataLayer || [];
  window.gtag = function () { dataLayer.push(arguments); };
  gtag("js", new Date()); gtag("config", "%s");
})();
</script>"""


def analytics() -> str:
    """アクセス解析(GA4)の計測のタグ(環境変数 GA_MEASUREMENT_ID の値)。無ければ空。
    値は G- と英大文字・数字だけを受ける(スクリプトに変な字を入れない)。"""
    v = os.environ.get("GA_MEASUREMENT_ID", "").strip()
    if not v:
        return ""
    if not re.fullmatch(r"G-[A-Z0-9]+", v):
        raise SystemExit("GA_MEASUREMENT_ID は G- で始まる測定 ID(例 G-ABC123XYZ)だけにする")
    return GTAG % (HOST, v, v)


# サイトの地図に載せないページ: 言語を選んで移るだけの入口と前の URL(検索の取得は en/ へ移されるので、
# 移る先の ja/・en/ を載せる)と、mdBook の目次の部品・404 のページ(印刷の版と目次は noindex でも外れる)。
NOT_IN_SITEMAP = {"index.html", "gallery.html"}
BOOK_PARTS = {"toc.html", "404.html"}


def folder_form(rel: str) -> str:
    """サイトの中の道筋を公開の URL の形にする(index.html はフォルダの形。canonical・hreflang・サイトの地図で同じ)。"""
    return rel[: -len("index.html")] if rel == "index.html" or rel.endswith("/index.html") else rel


def sitemap(out: Path) -> int:
    """作った site/ の HTML から sitemap.xml を書き、載せた URL の数を返す。noindex のページ(mdBook の印刷の版・目次)と
    NOT_IN_SITEMAP・BOOK_PARTS は載せない。URL の形は canonical・hreflang と同じ(index.html はフォルダの形。
    説明書の入口も manual/<言語>/)。lastmod は書かない(作るたびに全部変わり、正しい日付にならないため)。"""
    urls = []
    for f in sorted(out.rglob("*.html")):
        rel = f.relative_to(out).as_posix()
        if rel in NOT_IN_SITEMAP or f.name in BOOK_PARTS:
            continue
        if re.search(r'<meta[^>]+name="robots"[^>]+noindex', f.read_text(encoding="utf-8")):
            continue
        urls.append(SITE + folder_form(rel))
    body = "".join(f"  <url><loc>{html.escape(u)}</loc></url>\n" for u in urls)
    (out / "sitemap.xml").write_text(
        f'<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n{body}</urlset>\n',
        encoding="utf-8",
    )
    return len(urls)


ROOT_TITLE = "mdgrid — Markdown frontmatter as a table in the terminal"
OG_ALT = {
    "ja": "mdgrid の画面: Markdown のノートが行、フロントマターのキーが列の表",
    "en": "An mdgrid screen: Markdown notes as rows, frontmatter keys as columns",
}


def chooser(file: str = "", verify: str = "") -> str:
    """入口(x-default): 覚えた選択か、?lang= か、ブラウザの言語(navigator.languages)で /ja/・/en/ へ移る。
    合わなければ選ぶ画面のまま。移るのは location.replace(戻るで戻されない)。JS が無くても両方へのリンクが見える。
    `verify` は head に足す検索の管理の確認のタグ(入口だけ)。"""
    links = "".join(
        f'<a class="card" href="{l}/{file}" hreflang="{l}" lang="{l}" data-pick="{l}"><h2>{NAMES[l]}</h2>'
        f'<p class="muted">{html.escape(TEXT[l]["lead"])}</p></a>'
        for l in LANGS
    )
    route = """
(function () {
  var langs = ["ja", "en"], pick = null;
  try { var q = new URLSearchParams(location.search).get("lang"); if (langs.indexOf(q) >= 0) pick = q; } catch (e) {}
  if (!pick) { try { var s = localStorage.getItem("%s"); if (langs.indexOf(s) >= 0) pick = s; } catch (e) {} }
  if (!pick) {
    var nav = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language || ""];
    for (var i = 0; i < nav.length && !pick; i++) { var p = String(nav[i]).slice(0, 2).toLowerCase(); if (langs.indexOf(p) >= 0) pick = p; }
  }
  if (pick) location.replace(pick + "/%s" + location.hash);
})();
""" % (KEY, file)
    body = f"""
<h1>mdgrid</h1>
<p class="muted">Choose a language · 言語を選ぶ</p>
<div class="cards">{links}</div>
"""
    head = alternates("ja/" + file, SITE + file)
    if verify:
        head = verify + "\n" + head
    # 題には探す語を入れる(根の入口は検索の結果に出る)。前の画面の一覧の URL は画面の一覧の題。
    title = f'{TEXT["en"]["gallery"]} · mdgrid' if file else ROOT_TITLE
    desc = TEXT["en"]["gallery_desc"] if file else TEXT["en"]["lead"]
    return page("en", title, body, desc, SITE + file, head, route)


# 入口の文(日英)。docs/writing.md の読む人の順: 何が見えるか → 1分で試す → 自分の場面 → できること → 見た目 → 次に読むもの。
# 特徴は具体的にできることを見出しにし、本物の画面(撮った場面)を添える。
HOME = {
    "ja": {
        "tagline": "Markdown のノートのフォルダを、コマンド1つで表として開けます。",
        "h1": ["Markdown と CSV を、", "ターミナルでデータベースに。"],
        "sub": "Markdown のノートのフォルダや CSV を、ターミナルでデータベースのように開きます。絞る、並べる、まとめる、リンクでつなぐ。直した値だけを、元のファイルに書き戻します。",
        "start": "はじめかた",
        "try_link": "1分で試す",
        "facts": ["Linux・macOS・Windows", "端末で動く実行ファイル1つ", "MIT / Apache-2.0"],
        "bin": "リリースから実行ファイルを取る",
        "rust": "cargo で入れるなら Rust 1.90 以上",
        "releases": "リリース",
        "steps": [
            ("Markdown のフォルダ", "ノートが行に、フロントマターのキーが列に並ぶ。", "table", "mdgrid ~/notes/Tasks"),
            ("CSV のファイル", "1行目が列の名前になる。.tsv も同じ。", "csv-table", "mdgrid equipment.csv"),
            ("つながった表", "ノートのリンクが表をつなぐ。R で関係マップ。", "demo-relmap", "mdgrid ~/notes/Tasks"),
        ],
        # 見出し(*_h)は人が書き直す前提の、説明だけの仮の文。
        "db_eyebrow": "データベースとして",
        "db_h": "絞る・並べる・まとめる・1枚を見る",
        "db_p": "ファイルはそのままで、表の側だけが動きます。どれもキー1つか2つで、Esc で元の表に戻ります。",
        "db": [
            ("filter", "絞る", "打った語で絞る", "\\ を押して打つと、その語を含む行だけが残ります。列の値を選んで絞ることも、条件をビューに残すこともできます。", "quick-filter"),
            ("sort", "並べる", "列で並べ替える", "列の上で s。もう一度押すと逆順になります。ノートのファイルの並びは変わりません。", "sort-column"),
            ("group", "まとめる", "値ごとのまとまり", "status ごと、project ごとに行をまとめます。見出しは畳めて、件数が付きます。", "demo-group"),
            ("views", "ビュー", "見方に名前を付けて残す", "絞り込み・並べ替え・列の組をビューにして、タブで切り替えます。.base のビューもそのまま開きます。", "base-tabs"),
            ("freq", "値の分布", "列の値を数える", "% で、その列にどの値が何件あるか。空のままのノートが何枚あるかも分かります。", "demo-freq"),
            ("detail", "1枚を見る", "ノート1枚を、表から出ずに読む", "K で、選んだノートの全部のプロパティと本文が1枚に出ます。Enter でそのまま直せます。", "detail"),
        ],
        "more": "ほかに: 型ごとの入力(カレンダー・タグ・チェックボックス)、まとめて直す、親子の字下げと WBS、型と決まりの検査、英語と日本語の画面。",
        "rel_eyebrow": "リレーション",
        "rel_h": "リンクが、表と表のリレーションになる",
        "rel_p": "フロントマターにノートへのリンクを書くと、その列は行き先の表へのリレーションになります。セルから行き先のノートを開き、そのノートを指している行をたどれます。R を押すと、表とつながりが1枚の図に。",
        "rel_file": "タスクのノートの2行",
        "rel_note": "リレーションはノートどうしの間のものです。CSV の表は、まだこの範囲に入りません。",
        "rel_more": "リンクで表をつなぐ",
        "ai_eyebrow": "分割した画面で",
        "ai_h": "AI の隣の pane に置く",
        "ai_p": "ターミナルを割って、片方で AI エージェントに仕事を頼み、もう片方に mdgrid を開いておきます。エージェントがノートを書き換えると、表が追いかけます。人は表で確かめて、違っていればその場で直します。",
        "ai_left": "AI エージェント(架空の画面)",
        "ai_right": "mdgrid(本物の画面)",
        "ai_ask": "カタログのリンク切れを直して、タスクを閉じて",
        # (ラベル, コマンドかパス, 結果)。結果は本物の mdgrid で確かめた形(9 行・--check は問題が無ければ何も出さず終了コード 0)。
        "ai_log": [
            ("読む", "mdgrid Tasks.base --print --format json", "9 行"),
            ("書く", "Tasks/Fix broken links on the catalog site.md", None),
            ("確かめる", "mdgrid Tasks --check", "問題なし(終了コード 0)"),
        ],
        "ai_done": "リンクを直して、タスクを done にしました。右の表で確かめてください。",
        "ai_hint": "Enter 送る · Esc 止める",
        "ai_fits": [
            ("1秒ごとに読み直す", "mdgrid の外でノートが変わると、表に出ます。間隔は設定の poll_ms で変えられます。"),
            ("黙って上書きしない", "こちらが直している途中のノートを外で書き換えられたら、そのノートの保存は止まり、どちらを残すかを選べます。"),
            ("エージェント向けのスキル", "mdgrid の CLI の使い方を教えるスキルを、リポの plugins/mdgrid に置いています。"),
        ],
        "ai_note": "左の pane は、説明のために作った架空のエージェントの画面です。tmux や WezTerm など、画面を割れる端末ならどれでも同じ置き方ができます。",
        "cli_eyebrow": "ターミナルの道具として",
        "cli_h": "ターミナルで完結する",
        "cli_fits": [
            ("tmux", "コピー(y)が、tmux の中からでも手元のクリップボードに届きます。"),
            ("WezTerm・Ghostty", "丸い札などの飾りを、端末を見分けて自動で描きます。ほかの端末では崩れない形に。"),
            ("エディタ", "e で、選んだノートをふだんのエディタで開きます。vim・Neovim・Helix・Emacs など $EDITOR に入れたもの、code -w も。閉じると表が読み直します。"),
            ("マウス", "操作はキーが基本で、行やセルのクリックとホイールも効きます。"),
        ],
        "cli_p": "mdgrid は実行ファイル1つです。常駐するものも、別に立てるデータベースもありません。データはノートと CSV のファイルそのものなので、git で差分が追え、ふだんのエディタで同じフォルダを開いたままにできます。",
        "cli": [
            ("フォルダごとに1コマンド", "alias tasks='mdgrid ~/notes/Tasks'"),
            ("表を標準出力へ(csv・tsv・json・md)", "mdgrid ~/notes/Tasks --print --format json | jq ."),
            ("画面で選んだノートを、次のコマンドへ", "mdgrid ~/notes/Tasks --pick path | xargs -o vi"),
            ("型と決まりに合わない値を探す", "mdgrid ~/notes/Tasks --check"),
            ("読むだけで開く", "mdgrid ~/notes/Tasks --readonly"),
        ],
        "cli_more": "ほかのコマンドと組む",
        "how_eyebrow": "安心して直せる",
        "how_h": "書くのは、直した値だけ",
        "how_p": "見本のタスクのノートを5枚並べました。行を選ぶと、左にそのノートの中身が出ます。status を選び直してみてください。ファイルに書かれるのがその1行だけだと分かります。CSV でも同じ約束です。",
        "how_file": "ノート(ただの Markdown)",
        "how_sheet": "mdgrid の表",
        "how_note_col": "ノート",
        "how_say": {
            "unsaved": "未保存",
            "clean": "行を選ぶか、status を選び直す。",
            "dirty": "書くのは {n} 行。キーの順も、コメントも、本文も、1バイトも変えません。",
        },
        "how_undo": "元に戻す",
        "how_note": "この図はブラウザの中の模型です。本物の mdgrid は、Ctrl+S でこの差分を見せてから書きます。",
        "how_more": "安全のしくみ",
        "try_eyebrow": "1分で試す",
        "try_h": "見本を開いて、1つ直すまで。",
        "try": [
            ("入れる", "リリースのアーカイブを展開して PATH の通った所に置くのがいちばん早い道です。Rust があるなら次の1行でも入ります(ビルドに数分)。"),
            ("見本を写して開く", "見本はリポの examples にあります。一時フォルダに写してから開くので、保存しても元の見本は変わりません。"),
            ("この表が出れば成功", "「Northwind Studio」のタスクの表です。"),
        ],
        "try_keys": [("j k", "動く"), ("Enter", "直す"), ("Ctrl+S", "差分を見て保存"), ("q", "終わる")],
        "try_more": "入れ方のほかの道と画面の読み方",
        "scene_eyebrow": "自分の場面で",
        "scene_h": "空のフォルダから、台帳の形で始める。",
        "scene_p": "4つのスターターが入っています。印と雛形と見本を書き出すので、最初の1枚から表で見られます。ノートがもうあるなら、そのフォルダを渡すだけです。",
        "scenes": [
            ("tasks", "タスク", "1件1ノートのタスク", "未完了・進行中・待ち・完了のタブと、期限、優先度、タグ。終わったものの status を done にするのは、セルで Enter を押して選ぶだけ。", "starter-tasks"),
            ("reading", "読書", "本・記事・映画・講演の記録", "読みたい・読んでいる・読んだのタブと、種類・著者・評価の列。rating を付け忘れたノートは、空のセルが並ぶのですぐ見つかります。", "starter-reading"),
            ("people", "人と会議", "人の表と、会議の表", "会議のノートの出席者が、人のノートへのリンクになります。R を押すと、2つの表とつながりが図で見えます。", "starter-people"),
            ("bookmarks", "ブックマーク", "フォルダで眺めるブックマーク", "folder の列からフォルダの木を組みます。T で木が開き、右に選んだフォルダの行。Enter で URL を開けます。", "tree"),
        ],
        "look_eyebrow": "見た目",
        "look_h": "色は、設定の1行。",
        "look_p": "名前を押すと、そのテーマの本物の画面に替わります。変わるのは色だけで、* や > などの印はそのまま。枠や印の形まで決めるなら、カタログで触って選べます。",
        "next_eyebrow": "次に読むもの",
        "next_h": "見て選ぶ",
        "last_h": "いつものフォルダで開く。",
        "last_p": "入れたら、ノートのフォルダを渡すだけです。設定は要りません。",
        "foot": "mdgrid は MIT と Apache-2.0 のどちらかで使えます。",
        "see": "見て選ぶ",
    },
    "en": {
        "tagline": "Turn each folder of Markdown notes into a table you open with one command.",
        "h1": ["Markdown and CSV,", "a database in your terminal."],
        "sub": "mdgrid opens a folder of Markdown notes, or a CSV file, like a database in your terminal. Filter, sort, group, and link tables. Only the values you change are written back to the files.",
        "start": "Get started",
        "try_link": "Try it in a minute",
        "facts": ["Linux, macOS, Windows", "One binary, runs in the terminal", "MIT / Apache-2.0"],
        "bin": "Get a binary from Releases",
        "rust": "cargo needs Rust 1.90 or newer",
        "releases": "Releases",
        "steps": [
            ("A folder of Markdown", "Notes line up as rows, frontmatter keys as columns.", "table", "mdgrid ~/notes/Tasks"),
            ("A CSV file", "The first line names the columns. .tsv works the same way.", "csv-table", "mdgrid equipment.csv"),
            ("Linked tables", "Links between notes join the tables. Press R for the relation map.", "demo-relmap", "mdgrid ~/notes/Tasks"),
        ],
        "db_eyebrow": "As a database",
        "db_h": "Filter, sort, group, and read one note",
        "db_p": "The files stay as they are; only the table moves. Each of these is a key or two away, and Esc brings the table back.",
        "db": [
            ("filter", "Filter", "Filter by what you type", "Press \\ and type: only the rows containing that text remain. You can also filter by picking values of a column, and keep the conditions in a view.", "quick-filter"),
            ("sort", "Sort", "Sort by a column", "Press s on a column, and again to reverse. The order of the note files does not change.", "sort-column"),
            ("group", "Group", "Rows grouped by value", "Group rows by status or by project. Headings fold and carry a count.", "demo-group"),
            ("views", "Views", "Name a way of looking and keep it", "Save filters, sorting and a set of columns as a view, and switch views with tabs. Views in a .base file open as they are.", "base-tabs"),
            ("freq", "Value counts", "Count the values of a column", "Press % to see which values a column holds and how many notes have each, including how many are still empty.", "demo-freq"),
            ("detail", "One note", "Read one note without leaving the table", "Press K to see every property and the body of the selected note on one screen. Enter edits right there.", "detail"),
        ],
        "more": "Also: typed inputs (calendar, tags, checkboxes), bulk edits, parent/child indent and WBS, type and rule checks, English and Japanese screens.",
        "rel_eyebrow": "Relations",
        "rel_h": "A link becomes a relation between tables",
        "rel_p": "Write a link to another note in the frontmatter and that column becomes a relation to the table the note lives in. From a cell you can open the target note and follow the rows that point at it. Press R to see the tables and their links in one map.",
        "rel_file": "Two lines of a task note",
        "rel_note": "Relations are between notes. CSV tables are not part of this yet.",
        "rel_more": "Linking tables",
        "ai_eyebrow": "In a split terminal",
        "ai_h": "Keep it in the pane next to your AI agent",
        "ai_p": "Split the terminal: ask an AI agent to do the work in one pane and keep mdgrid open in the other. When the agent rewrites a note, the table follows. You check the result in the table and fix it right there if it is off.",
        "ai_left": "AI agent (a made-up screen)",
        "ai_right": "mdgrid (real screen)",
        "ai_ask": "Fix the broken links on the catalog and close the task",
        "ai_log": [
            ("Read", "mdgrid Tasks.base --print --format json", "9 rows"),
            ("Write", "Tasks/Fix broken links on the catalog site.md", None),
            ("Check", "mdgrid Tasks --check", "no problems (exit code 0)"),
        ],
        "ai_done": "Links fixed and the task set to done. Have a look at the table on the right.",
        "ai_hint": "Enter to send · Esc to stop",
        "ai_fits": [
            ("Rereads every second", "When a note changes outside mdgrid, the table shows it. The interval is the poll_ms setting."),
            ("Never overwrites silently", "If a note you are editing is rewritten outside, saving stops for that note and you choose which version to keep."),
            ("A skill for agents", "A skill that teaches agents the mdgrid CLI lives in plugins/mdgrid in the repository."),
        ],
        "ai_note": "The left pane is a made-up agent screen, drawn for this page. Any terminal that splits, such as tmux or WezTerm, works the same way.",
        "cli_eyebrow": "As a terminal tool",
        "cli_h": "Everything happens in the terminal",
        "cli_fits": [
            ("tmux", "Copying with y reaches your local clipboard from inside tmux too."),
            ("WezTerm, Ghostty", "Rounded chips and similar marks are drawn where the terminal is known to render them, and fall back to plain shapes elsewhere."),
            ("Editors", "Press e to open the selected note in your editor: vim, Neovim, Helix, Emacs or whatever $EDITOR holds, and code -w too. Close it and the table rereads the note."),
            ("Mouse", "Keys come first, and clicking rows and cells and the wheel work as well."),
        ],
        "cli_p": "mdgrid is a single binary. Nothing runs in the background and there is no database to set up. The data is the note and CSV files themselves, so git shows the diffs and your usual editor can keep the same folder open.",
        "cli": [
            ("One command per folder", "alias tasks='mdgrid ~/notes/Tasks'"),
            ("Print the table to standard output (csv, tsv, json, md)", "mdgrid ~/notes/Tasks --print --format json | jq ."),
            ("Hand the notes you pick to the next command", "mdgrid ~/notes/Tasks --pick path | xargs -o vi"),
            ("Find values that do not fit the column type or a rule", "mdgrid ~/notes/Tasks --check"),
            ("Open read-only", "mdgrid ~/notes/Tasks --readonly"),
        ],
        "cli_more": "Working with other commands",
        "how_eyebrow": "Safe to edit",
        "how_h": "Only the value you change is written",
        "how_p": "Here are five task notes from the sample. Pick a row and its file shows on the left. Then change a status: the one line that would be written lights up, and nothing else moves. CSV files get the same promise.",
        "how_file": "A note (plain Markdown)",
        "how_sheet": "The table in mdgrid",
        "how_note_col": "note",
        "how_say": {
            "unsaved": "unsaved",
            "clean": "Pick a row, or change a status.",
            "dirty": "{n} line(s) to write. Key order, comments and the body stay byte for byte.",
        },
        "how_undo": "Undo",
        "how_note": "This is a model in the browser. The real mdgrid shows you this diff on Ctrl+S before it writes.",
        "how_more": "How saving stays safe",
        "try_eyebrow": "Try it in a minute",
        "try_h": "From the sample to your first edit.",
        "try": [
            ("Install", "The quickest way is to unpack a release archive and put mdgrid on your PATH. With Rust, this one line works too (the build takes a few minutes)."),
            ("Copy the sample and open it", "The samples live in the repository's examples folder. You open a copy in a temporary folder, so saving never changes the original."),
            ("You should see this table", "It is the task table of \"Northwind Studio\"."),
        ],
        "try_keys": [("j k", "move"), ("Enter", "edit"), ("Ctrl+S", "review and save"), ("q", "quit")],
        "try_more": "Other ways to install, and how to read the screen",
        "scene_eyebrow": "For your own notes",
        "scene_h": "Start from an empty folder, already shaped as a table.",
        "scene_p": "Four starters are built in. Each writes a marker, a note template and a few samples, so the first note already shows up in a table. If you have notes, just pass their folder.",
        "scenes": [
            ("tasks", "Tasks", "One note per task", "Tabs for open, doing, waiting and done, with due date, priority and tags. Marking a task done is Enter on its status cell and one pick.", "starter-tasks"),
            ("reading", "Reading", "Books, articles, films, talks", "Tabs for the queue, what you are on now and what is done, with columns for kind, author and rating. Notes with no rating stand out as empty cells.", "starter-reading"),
            ("people", "People and meetings", "A people table and a meetings table", "The attendees of a meeting note link to people notes. Press R to see the two tables and the links between them.", "starter-people"),
            ("bookmarks", "Bookmarks", "Bookmarks you browse by folder", "The folder column becomes a tree. Press T for the tree, with the rows of the selected folder on the right. Enter opens the URL.", "tree"),
        ],
        "look_eyebrow": "Looks",
        "look_h": "Colors are one line of config.",
        "look_p": "Press a name to see the real screen in that theme. Only the colors change; marks such as * and > stay. To choose borders and marks too, try them in the catalog.",
        "next_eyebrow": "Read next",
        "next_h": "See and choose",
        "last_h": "Open the folder you already have.",
        "last_p": "Once installed, pass your notes folder. There is nothing to set up.",
        "foot": "mdgrid is available under MIT or Apache-2.0.",
        "see": "See and choose",
    },
}
TRY = "git clone https://github.com/S6U5/mdgrid && cd mdgrid\ncp -R examples/studio /tmp/mdgrid-sample\nmdgrid /tmp/mdgrid-sample/Tasks"
INSTALL = "cargo install mdgrid --locked"

# 仕組みの図に出すノート(見本 examples/studio/Tasks の本物のファイルを読む)と、表に出す列。
MODEL_NOTES = [
    "Write the ordering estimate",
    "Build the menu page",
    "Photograph the seasonal pastries",
    "Design the booking screens",
    "Review privacy requirements with the clinic",
]
MODEL_COLS = ["status", "project", "owner", "due", "priority"]
MODEL_STATUS = ["todo", "doing", "done"]


def version() -> str:
    """Cargo.toml の版(入口に出す)。"""
    return tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]


def model_notes() -> list:
    """図のノート: ファイルの行そのものと、表に出す値(リンクの [[ ]] と引用符は外して名前だけ)。"""
    out = []
    for name in MODEL_NOTES:
        lines = (ROOT / "examples/studio/Tasks" / f"{name}.md").read_text(encoding="utf-8").rstrip("\n").split("\n")
        note = {"name": name, "path": f"Tasks/{name}.md", "lines": lines}
        for line in lines[1 : lines.index("---", 1)]:
            m = re.match(r"([A-Za-z_]+):\s*(.*)$", line)
            if m and m.group(1) in MODEL_COLS:
                note[m.group(1)] = re.sub(r'^"?\[\[(.*)\]\]"?$', r"\1", m.group(2))
        if note.get("status") not in MODEL_STATUS:
            raise SystemExit(f"図のノートの status が {MODEL_STATUS} に無い: {name}")
        out.append(note)
    return out


def cmd(text: str, lang: str, prompt: str = "$ ") -> str:
    """コマンドの箱。行の頭の $ は写す文字に入れない。写すボタンは JS で動く(無ければ選んで写せる)。"""
    t = TEXT[lang]
    lines = "\n".join(f'<span class="ps">{prompt}</span>{html.escape(l)}' if prompt else html.escape(l) for l in text.split("\n"))
    return (
        f'<div class="cmd"><pre><code data-text="{html.escape(text)}">{lines}</code></pre>'
        f'<button type="button" data-copy data-done="{t["copied"]}">{ICON_COPY}<span>{t["copy"]}</span></button></div>'
    )


def nav(lang: str, file: str) -> str:
    """どのページにも同じ上の帯(説明書・カタログ・画面の一覧・GitHub、言語と明暗の切り替え、はじめかた)。"""
    t = TEXT[lang]
    o = other_lang(lang)
    return (
        f'<nav class="top"><a class="brand" href="./">{LOGO}mdgrid</a>'
        f'<div class="links"><a href="../manual/{lang}/">{t["manual"]}</a><a href="../catalog/?lang={lang}">{t["catalog"]}</a>'
        f'<a href="gallery.html">{t["gallery"]}</a><a href="{REPO}">GitHub</a></div>'
        f'<span class="grow"></span><div class="tools">'
        f'<a class="opt" href="../{o}/{file}" hreflang="{o}" lang="{o}" data-pick="{o}">{NAMES[o]}</a>'
        f'<button class="opt mode" id="mode" type="button" aria-label="{t["mode"]}" title="{t["mode"]}">{ICON_MODE}</button>'
        f'<a class="go" href="../manual/{lang}/getting-started.html">{t["start"]}</a></div></nav>'
    )


def model(lang: str, notes: list) -> str:
    """仕組みの図(ノートのファイルと表)。JS が無いときは、最初のノートを選んだ静かな図として読める。"""
    h = HOME[lang]
    first = notes[0]
    file_lines, inside, ended = [], False, False
    for raw in first["lines"]:
        m = re.match(r"([A-Za-z_]+)(:.*)$", raw)
        if raw == "---":
            inside = not inside and not ended
            ended = ended or not inside
            file_lines.append(f'<span class="l fm">{raw}</span>')
        elif inside and m:
            file_lines.append(f'<span class="l"><span class="k" data-key="{m.group(1)}">{m.group(1)}</span>{html.escape(m.group(2))}</span>')
        else:
            file_lines.append(f'<span class="l{"" if inside else " body"}">{html.escape(raw)}</span>')
    head = f'<th scope="col">{h["how_note_col"]}</th>' + "".join(
        f'<th scope="col" data-col="{c}"{" class=num" if c == "priority" else ""}>{c}</th>' for c in MODEL_COLS
    )
    rows = []
    for i, n in enumerate(notes):
        opts = "".join(f'<option{" selected" if s == n["status"] else ""}>{s}</option>' for s in MODEL_STATUS)
        rows.append(
            f'<tr{" class=sel" if i == 0 else ""}><td><button class="name" type="button" title="{html.escape(n["name"])}">{html.escape(n["name"])}</button></td>'
            f'<td data-col="status"><span class="pick" data-v="{n["status"]}"><i></i>'
            f'<select aria-label="status: {html.escape(n["name"])}">{opts}</select></span></td>'
            f'<td class="link" data-col="project">{html.escape(n.get("project", ""))}</td>'
            f'<td class="link" data-col="owner">{html.escape(n.get("owner", ""))}</td>'
            f'<td data-col="due">{html.escape(n.get("due", ""))}</td>'
            f'<td class="num" data-col="priority">{html.escape(n.get("priority", ""))}</td></tr>'
        )
    say = html.escape(json.dumps(h["how_say"], ensure_ascii=False))
    data = json.dumps([{"path": n["path"], "lines": n["lines"], "status": n["status"]} for n in notes], ensure_ascii=False).replace("</", "<\\/")
    return f"""<div id="model" data-say="{say}">
<div class="model">
  <section class="pane file" aria-label="{h["how_file"]}"><header><b>{html.escape(first["path"])}</b><span>{h["how_file"]}</span></header>
  <pre><code>{"".join(file_lines)}</code></pre></section>
  <div class="join" aria-hidden="true">⇄</div>
  <section class="pane sheet" aria-label="{h["how_sheet"]}"><header><b>~/notes/Tasks</b><span>{h["how_sheet"]}</span></header>
  <div class="scroll"><table><thead><tr>{head}</tr></thead><tbody>{"".join(rows)}</tbody></table></div>
  <div class="tally" aria-live="polite"><b>{h["how_say"]["unsaved"]} 0</b><span>{h["how_say"]["clean"]}</span><button type="button" hidden>{h["how_undo"]}</button></div></section>
</div>
<p class="note">{html.escape(h["how_note"])}</p>
<script type="application/json" id="model-data">{data}</script>
</div>"""


def landing(lang: str, shown, shots) -> str:
    t, h = TEXT[lang], HOME[lang]
    i = 1 if lang == "ja" else 2
    img = lambda sid: f"../images/{lang}/{sid}.svg"
    man = f"../manual/{lang}/"

    # 最初の画面: Markdown のフォルダ・CSV のファイル・つながった表の3場面(本物の画面)。枠の題は、その場面のコマンド。
    frames = "".join(
        f'<a href="gallery.html#{sid}"{" class=on" if k == 0 else ""}><img{"" if k == 0 else " loading=lazy"} width="856" height="520" src="{img(sid)}" alt="{html.escape(title + ": " + text)}"></a>'
        for k, (title, text, sid, _) in enumerate(h["steps"])
    )
    steps = "".join(
        f'<button class="step" type="button" data-cmd="{html.escape(run)}" aria-pressed="{"true" if k == 0 else "false"}"><span class="n">{k + 1}</span><b>{html.escape(title)}</b><span class="t">{html.escape(text)}</span></button>'
        for k, (title, text, sid, run) in enumerate(h["steps"])
    )
    under = f'<a href="{REPO}/releases/latest">{h["bin"]}</a> · {h["rust"]}'
    facts = "".join(f"<li>{html.escape(x)}</li>" for x in [f"v{version()}"] + h["facts"])

    # 1分で試す。
    keys = " ".join(f"<kbd>{k}</kbd> {html.escape(w)}" for k, w in h["try_keys"])
    (t1, p1), (t2, p2), (t3, p3) = h["try"]
    tries = f"""
<li><h3>{t1}</h3><p>{html.escape(p1)}</p>{cmd(INSTALL, lang)}<p class="small">{under}</p></li>
<li><h3>{t2}</h3><p>{html.escape(p2)}</p>{cmd(TRY, lang)}</li>
<li><h3>{t3}</h3><a class="screen" href="gallery.html#table"><img loading="lazy" width="856" height="520" src="{img("table")}" alt="{html.escape(t3)}"></a><p>{html.escape(p3)} {keys}</p><p class="small"><a href="{man}getting-started.html">{html.escape(h["try_more"])}</a></p></li>"""

    # タブで切り替える画面の組(データベースとして・自分の場面)。JS が無ければ全部が縦に並ぶ。
    def tabset(pre: str, label: str, items, extra=lambda sid: "") -> str:
        tabs = "".join(
            f'<button class="tab" type="button" role="tab" id="tab-{pre}{sid}" aria-controls="{pre}{sid}" aria-selected="{"true" if k == 0 else "false"}">{html.escape(name)}</button>'
            for k, (sid, name, _, _, _) in enumerate(items)
        )
        panels = "".join(
            f'<div class="scene" role="tabpanel" id="{pre}{sid}" aria-labelledby="tab-{pre}{sid}"><div class="say"><h3>{html.escape(title)}</h3><p>{html.escape(text)}</p>{extra(sid)}</div>'
            f'<a class="screen" href="gallery.html#{shot}"><img loading="lazy" width="856" height="520" src="{img(shot)}" alt="{html.escape(title)}"></a></div>'
            for sid, name, title, text, shot in items
        )
        return f'<div class="tabs" role="tablist" aria-label="{html.escape(label)}">{tabs}</div>\n{panels}'

    db = tabset("db-", h["db_eyebrow"], h["db"])
    starter = lambda sid: cmd(f"mdgrid ~/{sid} --init-workspace --starter {sid} --with-samples\nmdgrid ~/{sid}", lang)
    scenes = tabset("scene-", h["scene_eyebrow"], h["scenes"], starter)

    # 分割した画面: 左は架空のエージェントの画面(どの製品の画面も写さない。名前も sample-agent)、右は本物の画面。
    ai_steps = ""
    for what, arg, result in h["ai_log"]:
        tail = f'<span class="res">└ {html.escape(result)}</span>' if result else '<span class="diff"><span class="del">- status: todo</span><span class="add">+ status: done</span></span>'
        ai_steps += f'<div class="act"><span class="lab">▸ {html.escape(what)}</span><span class="arg">{html.escape(arg)}</span>{tail}</div>'
    ai_log = (
        f'<div class="agent-ui"><p class="who">sample-agent v1.0 · ~/notes/ai-human</p>'
        f'<p class="ask">{html.escape(h["ai_ask"])}</p>{ai_steps}'
        f'<p class="done">{html.escape(h["ai_done"])}</p>'
        f'<div class="box" aria-hidden="true"><span>&gt;</span><i></i></div><p class="hint">{html.escape(h["ai_hint"])}</p></div>'
    )

    # ターミナルの道具として: 本物のオプションの使い方を、1つの端末に並べる(# の行は説明。写すのはコマンドだけ)。
    cli_lines = "\n".join(f'<span class="cm"># {html.escape(why)}</span>\n<span class="ps">$ </span>{html.escape(run)}' for why, run in h["cli"])
    cli_text = html.escape("\n".join(run for _, run in h["cli"]))
    cli = (
        f'<div class="cmd tall"><pre><code data-text="{cli_text}">{cli_lines}</code></pre>'
        f'<button type="button" data-copy data-done="{t["copied"]}">{ICON_COPY}<span>{t["copy"]}</span></button></div>'
    )

    # 見た目: テーマの場面(theme-*)を名前で並べ、押すと画面を差し替える。JS が無ければ画面の一覧のその場面へ行く。
    looks = [(s["id"][len("theme-"):], s["title"][lang].split(": ", 1)[-1]) for s in shots if s["id"].startswith("theme-")]
    look_tabs = "".join(
        f'<a class="tab" role="button" href="gallery.html#theme-{lid}" data-look="{lid}" aria-pressed="{"true" if k == 0 else "false"}">{html.escape(name)}</a>'
        for k, (lid, name) in enumerate(looks)
    )
    look0 = looks[0][0]
    look_code = (
        f'<div class="cmd"><pre><code id="look-code" data-text="[look]&#10;theme = &quot;{look0}&quot;"><span class="cm">[look]</span>\n<span class="v">theme = "{look0}"</span></code></pre>'
        f'<button type="button" data-copy data-done="{t["copied"]}">{ICON_COPY}<span>{t["copy"]}</span></button></div>'
    )

    demos = "".join(
        f'<figure class="demo"><img loading="lazy" src="../demos/{d[0]}" alt="{html.escape(d[i])}">'
        f"<figcaption><b>{html.escape(d[i])}</b></figcaption></figure>"
        for d in shown
    )
    card = lambda href, title, desc: f'<a class="card" href="{href}"><h3>{title}</h3><p class="muted">{html.escape(desc)}</p><span class="arrow" aria-hidden="true">→</span></a>'
    body = f"""
<main>
<header class="hero">
  <div class="in">
    <p class="prompt"><b>$</b> mdgrid ~/notes/Tasks<i></i></p>
    <h1><span>{h["h1"][0]}</span>{"" if lang == "ja" else " "}<span>{h["h1"][1]}</span></h1>
    <p class="lead">{html.escape(h["sub"])}</p>
    {cmd(INSTALL, lang)}
    <p class="under">{under}</p>
    <div class="cta"><a class="btn" href="{man}getting-started.html">{h["start"]} →</a><a class="btn ghost" href="#try">{h["try_link"]}</a><a class="btn ghost" href="{REPO}">GitHub</a></div>
    <ul class="facts">{facts}</ul>
  </div>
  <div class="stage">
    <div class="term"><div class="bar-t"><i></i><i></i><i></i><span>mdgrid ~/notes/Tasks</span></div><div class="frames">{frames}</div></div>
    <div class="steps">{steps}</div>
  </div>
</header>

<section class="sec tint" id="db"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["db_eyebrow"]}</p><h2>{h["db_h"]}</h2><p>{html.escape(h["db_p"])}</p></div>
  {db}
  <p class="also">{html.escape(h["more"])} <a href="{man}features.html">{t["features"]} →</a></p>
</div></section>

<section class="sec" id="relations"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["rel_eyebrow"]}</p><h2>{h["rel_h"]}</h2><p>{html.escape(h["rel_p"])}</p></div>
  <div class="rel rise">
    <div class="say">
      <section class="pane file" aria-label="{h["rel_file"]}"><header><b>Tasks/Build the appointment API.md</b><span>{h["rel_file"]}</span></header>
      <pre><code><span class="l"><span class="k">project</span>: "[[Clinic Booking App]]"</span><span class="l"><span class="k">owner</span>: "[[Sam Ortiz]]"</span></code></pre></section>
      <p class="note">{html.escape(h["rel_note"])}</p>
      <a class="more" href="{man}features.html">{html.escape(h["rel_more"])} →</a>
    </div>
    <a class="screen" href="gallery.html#demo-relmap"><img loading="lazy" width="856" height="520" src="{img("demo-relmap")}" alt="{html.escape(h["rel_h"])}"></a>
  </div>
</div></section>

<section class="sec tint" id="ai"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["ai_eyebrow"]}</p><h2>{h["ai_h"]}</h2><p>{html.escape(h["ai_p"])}</p></div>
  <div class="term panes rise"><div class="bar-t"><i></i><i></i><i></i><span>~/notes</span></div>
    <div class="split-t">
      <div class="pane-t agent"><p class="cap">{h["ai_left"]}</p>{ai_log}</div>
      <div class="pane-t"><p class="cap">{h["ai_right"]}</p><a href="gallery.html#demo-ai-human"><img loading="lazy" width="856" height="520" src="{img("demo-ai-human")}" alt="{html.escape(h["ai_right"])}"></a></div>
    </div>
  </div>
  <dl class="fits rise">{"".join(f"<div><dt>{html.escape(k)}</dt><dd>{html.escape(v)}</dd></div>" for k, v in h["ai_fits"])}</dl>
  <p class="note">{html.escape(h["ai_note"])}</p>
</div></section>

<section class="sec" id="cli"><div class="in">
  <div class="split">
    <div class="head rise"><p class="eyebrow">{h["cli_eyebrow"]}</p><h2>{h["cli_h"]}</h2><p>{html.escape(h["cli_p"])}</p>
      <a class="more" href="{man}tasks.html">{html.escape(h["cli_more"])} →</a></div>
    {cli}
  </div>
  <dl class="fits rise">{"".join(f"<div><dt>{html.escape(k)}</dt><dd>{html.escape(v)}</dd></div>" for k, v in h["cli_fits"])}</dl>
</div></section>

<section class="sec tint" id="how"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["how_eyebrow"]}</p><h2>{h["how_h"]}</h2><p>{html.escape(h["how_p"])}</p></div>
  {model(lang, model_notes())}
  <p class="also"><a class="more" href="{man}safety.html">{html.escape(h["how_more"])} →</a></p>
</div></section>

<section class="sec" id="try"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["try_eyebrow"]}</p><h2>{h["try_h"]}</h2></div>
  <ol class="try rise">{tries}</ol>
</div></section>

<section class="sec tint" id="scenes"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["scene_eyebrow"]}</p><h2>{h["scene_h"]}</h2><p>{html.escape(h["scene_p"])}</p></div>
  {scenes}
</div></section>

<section class="sec" id="look"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["look_eyebrow"]}</p><h2>{h["look_h"]}</h2><p>{html.escape(h["look_p"])}</p></div>
  <div class="look">
    <a class="screen" href="gallery.html#theme-{look0}"><img id="look-img" loading="lazy" width="856" height="520" src="{img("theme-" + look0)}" alt="{html.escape(looks[0][1])}"></a>
    <div class="side"><div class="tabs">{look_tabs}</div>{look_code}<a class="btn ghost" href="../catalog/?lang={lang}">{t["catalog"]} →</a></div>
  </div>
</div></section>

<section class="sec tint" id="next"><div class="in">
  <div class="head rise"><p class="eyebrow">{h["next_eyebrow"]}</p><h2>{h["next_h"]}</h2></div>
  <div class="cards five">
    {card(man + "views.html", t["views"], t["views_desc"])}
    {card(man + "features.html", t["features"], t["features_desc"])}
    {card(man, t["manual"], t["manual_desc"])}
    {card("../catalog/?lang=" + lang, t["catalog"], t["catalog_desc"])}
    {card("gallery.html", t["gallery"], t["gallery_desc"])}
  </div>
</div></section>
{f'<section class="sec" id="demos"><div class="in"><div class="head"><p class="eyebrow">{t["demos"]}</p><h2>{t["demos_desc"]}</h2></div><div class="demos">{demos}</div></div></section>' if shown else ""}
<section class="sec last"><div class="in">
  <h2>{h["last_h"]}</h2><p class="lead">{html.escape(h["last_p"])}</p>
  {cmd(INSTALL + chr(10) + "mdgrid ~/notes/Tasks", lang)}
  <div class="cta"><a class="btn" href="{man}getting-started.html">{h["start"]} →</a><a class="btn ghost" href="{REPO}/releases/latest">{h["releases"]}</a></div>
</div></section>
</main>
<footer class="foot"><div class="in"><span>{html.escape(h["foot"])}</span><span class="grow"></span>
  <a href="{man}">{t["manual"]}</a><a href="{REPO}">GitHub</a><a href="{REPO}/releases">{h["releases"]}</a><a href="https://crates.io/crates/mdgrid">crates.io</a></div></footer>
"""
    return page(
        lang, f"mdgrid — {h['tagline']}", body, h["sub"], f"{SITE}{lang}/",
        alternates("ja/") + "\n" + software_ld(lang), LANDING_JS, top=nav(lang, ""), wide=True,
    )


def software_ld(lang: str) -> str:
    """言語の入口に置く構造化データ(schema.org の SoftwareApplication)。文は入口の文と同じ。"""
    data = {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "mdgrid",
        "description": HOME[lang]["sub"],
        "url": f"{SITE}{lang}/",
        "image": f"{SITE}assets/{lang}/og.png",
        "inLanguage": lang,
        "applicationCategory": "DeveloperApplication",
        "operatingSystem": "Linux, macOS, Windows",
        "license": ["https://spdx.org/licenses/MIT.html", "https://spdx.org/licenses/Apache-2.0.html"],
        "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
        "downloadUrl": f"{REPO}/releases/latest",
        "sameAs": [REPO, "https://crates.io/crates/mdgrid"],
    }
    # </script> で閉じられないように、< を \u003c にして埋める。
    return '<script type="application/ld+json">' + json.dumps(data, ensure_ascii=False).replace("<", "\\u003c") + "</script>"


def gallery(lang: str, shots) -> str:
    t = TEXT[lang]
    figs = []
    for s in shots:
        sid = s["id"]
        # 題と説明は {ja, en} の表か、両方に同じ文字。
        pick = lambda v: v.get(lang, sid) if isinstance(v, dict) else v
        title = pick(s.get("title", sid))
        text = pick(s.get("text", ""))
        keys = " ".join(s.get("keys", []))
        search = " ".join([sid, keys, title, text]).lower()
        k = f'<code class="muted">{t["keys"]}: {html.escape(keys)}</code>' if keys else ""
        figs.append(
            f'<figure id="{sid}" data-search="{html.escape(search)}">'
            f'<a href="../images/{lang}/{sid}.svg"><img loading="lazy" alt="{html.escape(title)}" src="../images/{lang}/{sid}.svg"></a>'
            f'<figcaption><b>{html.escape(title)}</b><span class="muted">{html.escape(text)}</span>{k}</figcaption></figure>'
        )
    body = f"""
<h1>{t["gallery"]}</h1>
<p class="muted">{html.escape(t["gallery_desc"])} {t["count"].format(n=len(shots))}</p>
<input id="find" type="search" aria-label="{t["filter"]}" placeholder="{t["filter"]}">
<div class="shots">{''.join(figs)}</div>
"""
    return page(
        lang, f'{t["gallery"]} · mdgrid', body, t["gallery_desc"], f"{SITE}{lang}/gallery.html",
        alternates("ja/gallery.html", SITE + "gallery.html"), FIND, top=nav(lang, "gallery.html"),
    )


def decorate_book(lang: str, dest: Path) -> None:
    """mdBook の各ページに、右上の同じページの別の言語への切り替えを書き足す。章のページには、hreflang(ja・en・x-default)・
    正規の URL・説明文(本文の最初の段落から)・カードのタグも書き足す(検索に出さない印刷の版・目次・404 には足さない)。
    URL は入口をフォルダの形(manual/<言語>/)にする。x-default は英語の版(言語を決めて移るページが説明書には無いため)。"""
    other = other_lang(lang)
    for f in dest.glob("*.html"):
        name = f.name
        text = f.read_text(encoding="utf-8")
        shape = folder_form(name)
        alt = ""
        if name not in BOOK_PARTS and name != "print.html":
            alt = "\n".join(
                [f'<link rel="alternate" hreflang="{l}" href="{SITE}manual/{l}/{shape}">' for l in LANGS]
                + [f'<link rel="alternate" hreflang="x-default" href="{SITE}manual/en/{shape}">']
            )
            desc = first_paragraph(text) or TEXT[lang]["manual_desc"]
            text, n = re.subn(r'<meta name="description" content="[^"]*">', lambda m: f'<meta name="description" content="{html.escape(desc)}">', text, count=1)
            if n == 0:
                alt = f'<meta name="description" content="{html.escape(desc)}">\n' + alt
            alt = social(lang, head_text(text, "title"), desc, f"{SITE}manual/{lang}/{shape}") + "\n" + alt
        text = text.replace("</head>", (alt + "\n" if alt else "") + (ANALYTICS + "\n" if ANALYTICS else "") + "</head>", 1)
        switch = (
            f'<a class="lang-switch" href="../{other}/{name}" hreflang="{other}" lang="{other}" data-pick="{other}" '
            f'title="{NAMES[other]}" style="font-size:14px;margin-inline-end:10px;text-decoration:none">{NAMES[other]}</a>'
        )
        text = text.replace('<div class="right-buttons">', '<div class="right-buttons">' + switch, 1)
        text = text.replace("</body>", f"<script>{REMEMBER}</script>\n</body>", 1)
        f.write_text(text, encoding="utf-8")


def head_text(text: str, what: str) -> str:
    """HTML の head から、題(`title`)か説明文(`description`)の字を取る(無ければ空)。"""
    pat = r"<title>(.*?)</title>" if what == "title" else r'<meta name="description" content="([^"]*)"'
    m = re.search(pat, text, re.S)
    return html.unescape(m.group(1)).strip() if m else ""


DESC_MAX = {"ja": 120, "en": 160}


def first_paragraph(text: str) -> str:
    """mdBook のページの本文の最初の段落(リンクだけの行は飛ばす)を、説明文の長さにして返す。長ければ文の切れ目で切り、
    切れ目が無ければ字数で切って … を付ける。"""
    main = re.search(r"<main>(.*?)</main>", text, re.S)
    for m in re.finditer(r"<p>(.*?)</p>", main.group(1) if main else "", re.S):
        p = " ".join(html.unescape(re.sub(r"<[^>]+>", "", m.group(1))).split())
        # リンクだけの行(章の頭の「English · 目次」など)は説明文にしない。
        rest = html.unescape(re.sub(r"<[^>]+>", "", re.sub(r"<a\b.*?</a>", "", m.group(1), flags=re.S)))
        if len(re.sub(r"[\s·()()]", "", rest)) < 10:
            continue
        limit = DESC_MAX["ja" if re.search(r"[\u3040-\u30ff\u4e00-\u9fff]", p) else "en"]
        if len(p) <= limit:
            return p
        cut = max(p.rfind("。", 0, limit) + 1, p.rfind(". ", 0, limit) + 1)
        return p[:cut].strip() if cut > limit // 3 else p[: limit - 1].rstrip() + "…"
    return ""


# ---------- 説明書(mdBook) ----------

# 本に入れるページ(題は各ページの最初の見出しから)。参照のページは docs/ の言語ごとの版を本の名前で入れる。
PAGES = ["index.md", "views.md", "features.md", "getting-started.md", "tasks.md", "concepts.md", "appearance.md", "themes.md", "faq.md"]
REFS = {
    "ja": {"keys.md": "docs/keys.ja.md", "config.md": "docs/config.ja.md", "safety.md": "docs/safety.ja.md", "obsidian-bases.md": "docs/obsidian-bases.ja.md"},
    "en": {"keys.md": "docs/keys.md", "config.md": "docs/config.md", "safety.md": "docs/safety.md", "obsidian-bases.md": "docs/obsidian-bases.md"},
}
BOOK_TITLE = {"ja": "mdgrid の説明書", "en": "mdgrid manual"}
REF_HEAD = {"ja": "参照", "en": "Reference"}
LINK = re.compile(r"(\]\()([^)\s]+)(\))")


def first_heading(text: str, default: str) -> str:
    for line in text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return default


def rewrite(text: str, src: str, lang: str) -> str:
    """`src`(リポの中のパス)のページのリンクを、本の中のページ・画像・カタログ・GitHub に直す。"""
    book_of = {v: k for k, v in REFS[lang].items()}
    other = "en" if lang == "ja" else "ja"
    other_of = {v: k for k, v in REFS[other].items()}
    base = os.path.dirname(src)

    def fix(m: re.Match) -> str:
        target = m.group(2)
        if re.match(r"^[a-z]+:", target) or target.startswith("#"):
            return m.group(0)
        path, _, frag = target.partition("#")
        frag = "#" + frag if frag else ""
        repo = os.path.normpath(os.path.join(base, path))
        if repo.endswith("/shots.md") and repo.startswith("docs/manual/"):
            # 画面の一覧は、サイトの画面の一覧へ。
            new = f"../../{lang}/gallery.html"
        elif repo.startswith(f"docs/manual/{lang}/images/"):
            new = "images/" + os.path.basename(repo)
        elif repo.startswith(f"docs/manual/{lang}/") and repo.endswith(".md"):
            new = os.path.basename(repo)
        elif repo.startswith(f"docs/manual/{other}/") and repo.endswith(".md"):
            new = f"../{other}/" + os.path.basename(repo).replace(".md", ".html")
        elif repo in book_of:
            new = book_of[repo]
        elif repo in other_of:
            new = f"../{other}/" + other_of[repo].replace(".md", ".html")
        elif repo == "docs/catalog/index.html":
            new = "../../catalog/?lang=" + lang
        elif repo.startswith(".."):
            return m.group(0)
        else:
            new = f"{REPO}/blob/main/{repo}"
        return m.group(1) + new + frag + m.group(3)

    out, fence = [], False
    for line in text.split("\n"):
        if line.lstrip().startswith("```"):
            fence = not fence
        out.append(line if fence else LINK.sub(fix, line))
    return "\n".join(out)


def book(lang: str, dest: Path, images: Path) -> None:
    src = ROOT / "target" / "book" / lang
    if src.exists():
        shutil.rmtree(src)
    (src / "src").mkdir(parents=True)
    shutil.copytree(images, src / "src" / "images")
    summary = [f"# {BOOK_TITLE[lang]}", ""]
    for name in PAGES:
        rel = f"docs/manual/{lang}/{name}"
        text = (ROOT / rel).read_text(encoding="utf-8")
        # リポで読む人への「Pages で読める」の案内は、Pages の本では外す。
        text = re.sub(r" · (画面つきでは|Read it with the screenshots at) .*$", "", text, flags=re.M)
        (src / "src" / name).write_text(rewrite(text, rel, lang), encoding="utf-8")
        title = first_heading(text, name)
        summary.append(f"[{title}]({name})" if name == "index.md" else f"- [{title}]({name})")
    summary += ["", f"# {REF_HEAD[lang]}", ""]
    for name, rel in REFS[lang].items():
        text = (ROOT / rel).read_text(encoding="utf-8")
        (src / "src" / name).write_text(rewrite(text, rel, lang), encoding="utf-8")
        summary.append(f"- [{first_heading(text, name)}]({name})")
    (src / "src" / "SUMMARY.md").write_text("\n".join(summary) + "\n", encoding="utf-8")
    (src / "book.toml").write_text(
        f'[book]\ntitle = "{BOOK_TITLE[lang]}"\nlanguage = "{lang}"\n\n'
        f'[output.html]\ngit-repository-url = "{REPO}"\nsite-url = "/mdgrid/manual/{lang}/"\n',
        encoding="utf-8",
    )
    subprocess.run(["mdbook", "build", str(src), "--dest-dir", str(dest.resolve())], check=True)


if __name__ == "__main__":
    args = sys.argv[1:]
    shots = ROOT / "target" / "shots"
    if "--shots" in args:
        i = args.index("--shots")
        shots = Path(args[i + 1]).resolve()
        del args[i : i + 2]
    build(Path(args[0]) if args else ROOT / "site", shots)
