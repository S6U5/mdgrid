#!/usr/bin/env python3
"""サイトのページを貼ったときのカードの画像(1200×630)を作る。

    python3 scripts/og_image.py

作るもの: docs/assets/og.png(英語)と docs/assets/ja/og.png(日本語)。scripts/pages.py が site/assets/<言語>/og.png に
写し、どのページの og:image・twitter:image もこれを指す。

中身は、名前と入口の見出し(scripts/pages.py の入口の文 HOME の h1 の2行)と、本物の画面(docs/assets の hero.png の暗い側)。
入口の見出しか hero.png を変えたら作り直してコミットする(サイトを作るたびには作らない。画像はリポに置く)。

要るもの: Playwright の Python 版と Chromium(scripts/check_site.py と同じ)。日本語の字は Noto Sans JP が入っていれば
それで、無ければその機械の日本語の字で描く。
"""

import base64
import html
from pathlib import Path

from playwright.sync_api import sync_playwright

import pages

ROOT = Path(__file__).resolve().parent.parent
W, H = pages.OG_W, pages.OG_H
# hero.png は暗い画面と明るい画面を横に並べた1枚(2024×608)。左の暗い側(幅 988)だけを見せる。
HERO_W, DARK_W = 2024, 988
SHOT_W = W - 96

CARD = """<!doctype html>
<html lang="{lang}"><head><meta charset="utf-8"><style>
*{{box-sizing:border-box;margin:0}}
body{{width:{w}px;height:{h}px;overflow:hidden;background:#111318;color:#e3e5ea;padding:44px 48px 0;display:grid;gap:22px;align-content:start;
  font-family:"Noto Sans JP","Hiragino Sans","Yu Gothic UI","Segoe UI",system-ui,sans-serif}}
.name{{font:700 30px/1 ui-monospace,Menlo,Consolas,"DejaVu Sans Mono",monospace;color:#8fa5ff}}
h1{{font-size:{size}px;line-height:1.3;font-weight:700;letter-spacing:-.01em;}}
.shot{{width:{shot}px;height:400px;overflow:hidden;border-radius:14px 14px 0 0;border:1px solid #2a2f39;border-bottom:0;box-shadow:0 8px 40px rgba(0,0,0,.5)}}
.shot img{{display:block;width:{img}px;height:auto}}
</style></head><body>
<p class="name">mdgrid</p>
<h1>{h1}</h1>
<div class="shot"><img src="data:image/png;base64,{hero}" alt=""></div>
</body></html>
"""


def main() -> None:
    with sync_playwright() as p:
        b = p.chromium.launch()
        for lang, hero, out in (("en", "docs/assets/hero.png", "docs/assets/og.png"), ("ja", "docs/assets/ja/hero.png", "docs/assets/ja/og.png")):
            pg = b.new_page(viewport={"width": W, "height": H}, device_scale_factor=1)
            pg.set_content(
                CARD.format(
                    lang=lang, w=W, h=H, shot=SHOT_W, img=round(SHOT_W * HERO_W / DARK_W),
                    size=42,
                    h1="<br>".join(html.escape(x) for x in pages.HOME[lang]["h1"]),
                    hero=base64.b64encode((ROOT / hero).read_bytes()).decode(),
                )
            )
            pg.wait_for_function("document.fonts.status === 'loaded' && [...document.images].every(i => i.complete)")
            pg.screenshot(path=str(ROOT / out))
            pg.close()
            print(out)
        b.close()


if __name__ == "__main__":
    main()
