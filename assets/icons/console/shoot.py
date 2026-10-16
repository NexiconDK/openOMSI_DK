"""Take a picture of every icon box on the page, on nothing (the strip cell behind
the icon gives the colour, so the picture keeps its transparency)."""

import pathlib
import sys

from playwright.sync_api import sync_playwright

page_path = pathlib.Path(sys.argv[1]).resolve()
out_dir = pathlib.Path(sys.argv[2])
out_dir.mkdir(parents=True, exist_ok=True)

with sync_playwright() as p:
    browser = p.chromium.launch()
    page = browser.new_page(viewport={"width": 200, "height": 1200})
    page.goto(page_path.as_uri())
    for box in page.query_selector_all(".box"):
        name = box.get_attribute("id")
        box.screenshot(path=str(out_dir / (name + ".png")), omit_background=True)
        print(name)
    browser.close()
