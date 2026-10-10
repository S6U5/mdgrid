#!/usr/bin/env python3
"""GitHub Pages のサイト(scripts/pages.py が作る site/)を、公開の前にブラウザで確かめる。

    python3 scripts/check_site.py [site]

要るもの: Playwright の Python 版と Chromium(pip install playwright && playwright install chromium)。
確かめること:
- 全部のページの相対のリンクと画像の行き先があること
- 入口の振り分け: 日本語のブラウザは /ja/、英語は /en/、ほかの言語は選ぶ画面のまま、選んだ言語は覚える
- 説明書: 右上の切り替えが同じページの別の言語を指し、hreflang(ja・en・x-default)がある
- 入口と画面の一覧の画像と録画が読める、画面の一覧の絞り込み
- 見た目のカタログ: 見本の画面が描かれる、幅 390px で縮んで横にはみ出さない、拡大の切り替え、広い幅では切り替えを出さない
- 入口が幅 390px で横にはみ出さない
- どのページにもコンソールのエラー(とページの例外)が無い
落ちたら理由を並べて終了コード 1。
"""

import functools
import glob
import http.server
import os
import re
import sys
import threading

from playwright.sync_api import sync_playwright

FAILS: list[str] = []


def check(ok: bool, what: str) -> None:
    print(("ok   " if ok else "FAIL ") + what)
    if not ok:
        FAILS.append(what)


def links(site: str) -> None:
    bad = []
    for f in glob.glob(os.path.join(site, "**", "*.html"), recursive=True):
        text = open(f, encoding="utf-8").read()
        for m in re.finditer(r'(?:href|src)="([^"#?]+)', text):
            u = m.group(1)
            if re.match(r"^[a-z]+:", u) or u.startswith("/"):
                continue
            p = os.path.normpath(os.path.join(os.path.dirname(f), u))
            if not os.path.exists(p) and not os.path.exists(os.path.join(p, "index.html")):
                bad.append(f"{os.path.relpath(f, site)} → {u}")
    check(not bad, f"相対のリンクと画像の行き先がある({len(bad)} 件が無い){': ' + ', '.join(bad[:5]) if bad else ''}")


def serve(site: str):
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=site)
    handler.log_message = lambda *a: None
    httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd, f"http://127.0.0.1:{httpd.server_address[1]}/"


def errors_of(page) -> list[str]:
    errs: list[str] = []
    page.on("console", lambda m: errs.append(m.text) if m.type == "error" else None)
    page.on("pageerror", lambda e: errs.append(str(e)))
    return errs


def all_images_load(page) -> tuple[int, int]:
    """画像を全部すぐに読ませて、(読めた数, 全部の数)。"""
    page.evaluate("document.querySelectorAll('img').forEach(i => i.loading = 'eager')")
    page.wait_for_function(
        "[...document.images].every(i => i.complete)", timeout=60000
    )
    return page.evaluate(
        "[[...document.images].filter(i => i.naturalWidth > 0).length, document.images.length]"
    )


def browser_checks(base: str) -> None:
    with sync_playwright() as p:
        b = p.chromium.launch()

        # 入口の振り分け。
        for loc, want in [("ja-JP", "/ja/"), ("en-US", "/en/"), ("fr-FR", None)]:
            ctx = b.new_context(locale=loc)
            pg = ctx.new_page()
            pg.goto(base)
            pg.wait_for_timeout(300)
            path = pg.url[len(base) - 1 :]
            if want:
                check(path.startswith(want), f"{loc} のブラウザで / → {want}(今: {path})")
            else:
                check(path in ("/", ""), f"{loc} のブラウザで / は選ぶ画面のまま(今: {path})")
            ctx.close()
        ctx = b.new_context(locale="ja-JP")
        pg = ctx.new_page()
        pg.goto(base + "ja/")
        pg.click('nav.top a[data-pick="en"]')
        pg.wait_for_url("**/en/")
        pg.goto(base)
        pg.wait_for_timeout(300)
        check(pg.url.endswith("/en/"), f"English を選んだあと / → /en/(今: {pg.url})")
        ctx.close()

        ctx = b.new_context(locale="en-US", viewport={"width": 1280, "height": 900})
        # どのページにもコンソールのエラーが無い。
        pages = [
            "", "ja/", "en/", "ja/gallery.html", "en/gallery.html", "gallery.html",
            "catalog/?lang=ja", "catalog/?lang=en",
            "manual/ja/index.html", "manual/en/index.html", "manual/ja/tasks.html", "manual/en/keys.html",
        ]
        for u in pages:
            pg = ctx.new_page()
            errs = errors_of(pg)
            pg.goto(base + u)
            pg.wait_for_load_state("load")
            pg.wait_for_timeout(200)
            check(not errs, f"{u or '/'} にコンソールのエラーが無い{': ' + errs[0][:120] if errs else ''}")
            pg.close()

        # 説明書の言語の切り替えと hreflang。
        pg = ctx.new_page()
        pg.goto(base + "manual/ja/tasks.html")
        href = pg.get_attribute("a.lang-switch", "href")
        check(href == "../en/tasks.html", f"説明書の切り替えが同じページの英語版を指す(今: {href})")
        langs = pg.eval_on_selector_all('link[rel="alternate"]', "ls => ls.map(l => l.hreflang).sort().join(',')")
        check(langs == "en,ja,x-default", f"説明書に hreflang ja・en・x-default(今: {langs})")
        pg.close()

        # 画像と録画、画面の一覧の絞り込み。
        for u in ["ja/", "en/", "ja/gallery.html", "en/gallery.html"]:
            pg = ctx.new_page()
            pg.goto(base + u)
            ok, total = all_images_load(pg)
            check(total > 0 and ok == total, f"{u} の画像が全部読める({ok}/{total})")
            pg.close()
        pg = ctx.new_page()
        pg.goto(base + "en/gallery.html")
        total = pg.locator("figure[data-search]").count()
        pg.fill("#find", "calendar")
        shown = pg.locator("figure[data-search]:not([hidden])").count()
        check(0 < shown < total, f"画面の一覧の絞り込み(calendar で {shown}/{total})")
        pg.close()

        # カタログ: 広い幅では等倍で切り替えを出さない。
        pg = ctx.new_page()
        pg.goto(base + "catalog/?lang=en")
        check(pg.locator("#term .scr").count() == 1, "カタログの見本の画面が描かれる")
        check("vault" in (pg.text_content("#term") or ""), "カタログの見本の画面に表の中身がある")
        check(pg.is_hidden("#fitbtn"), "広い幅ではカタログの拡大の切り替えを出さない")
        check(pg.locator("#catalog .card").count() > 20, "カタログの部品の見本が並ぶ")
        pg.close()
        ctx.close()

        # スマホの幅。
        ctx = b.new_context(locale="ja-JP", viewport={"width": 390, "height": 844})
        pg = ctx.new_page()
        errs = errors_of(pg)
        pg.goto(base + "catalog/?lang=ja")
        pg.wait_for_timeout(300)
        zoom = pg.evaluate("document.querySelector('#term .scr').style.zoom")
        check(zoom not in ("", None) and float(zoom) < 1, f"390px でカタログの見本の画面が縮む(zoom {zoom})")
        sw = pg.evaluate("document.documentElement.scrollWidth")
        check(sw <= 390, f"390px でカタログが横にはみ出さない(幅 {sw})")
        pg.click("#fitbtn")
        zoom2 = pg.evaluate("document.querySelector('#term .scr').style.zoom")
        check(zoom2 == "", f"「拡大して見る」で等倍になる(zoom {zoom2!r})")
        # スマホの Safari の字の自動の拡大を止めている(縮めた見本の画面で半角の字だけが大きくならない)。
        adj = pg.evaluate("(() => { const s = getComputedStyle(document.documentElement); return s.webkitTextSizeAdjust || s.textSizeAdjust || ''; })()")
        check(adj == "100%", f"カタログが字の自動の拡大を止めている(text-size-adjust {adj!r})")
        check(not errs, f"390px のカタログにコンソールのエラーが無い{': ' + errs[0][:120] if errs else ''}")
        for u in ["ja/", "en/gallery.html", "manual/ja/tasks.html"]:
            pg.goto(base + u)
            pg.wait_for_timeout(200)
            sw = pg.evaluate("document.documentElement.scrollWidth")
            check(sw <= 390, f"390px で {u} が横にはみ出さない(幅 {sw})")
        ctx.close()
        b.close()


def main() -> int:
    site = sys.argv[1] if len(sys.argv) > 1 else "site"
    links(site)
    httpd, base = serve(site)
    try:
        browser_checks(base)
    finally:
        httpd.shutdown()
    if FAILS:
        print(f"\n{len(FAILS)} 件が落ちた", file=sys.stderr)
        return 1
    print("\n全部通った")
    return 0


if __name__ == "__main__":
    sys.exit(main())
