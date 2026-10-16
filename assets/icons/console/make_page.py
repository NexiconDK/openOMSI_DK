"""Build a page that holds the tablet's strip icons, each in a box of its own,
so chromium can take a picture of every one of them.

The markup is lifted out of tablet.html itself - the template with the strip cells
and the <defs> with the gradients - so the pictures are the tablet's own icons and
not a copy of them that can drift.
"""

import html
import re
import sys

src = open(sys.argv[1], encoding="utf-8").read()
out = sys.argv[2]

def template(name):
    return re.search(r'<template id="%s">(.*?)</template>' % name, src, re.S).group(1)


defs = re.search(r'<svg id="paint".*?>(.*?)</svg>', src, re.S).group(1)
cells = re.findall(r'<i\b([^>]*)>(.*?)</i>', template("strip-cells"), re.S)
cells += re.findall(r'<button\b([^>]*)>(.*?)</button>', template("foot-keys"), re.S)

# the title in the template -> the name the picture gets. The strip's icons are drawn at
# 46 on the tablet and the foot bar's at 40; here they are 37 and 32.
NAME = {
    "Gruppe check ind": ("crowd", 37),
    "Signal": ("signal", 37),
    "Printer": ("printer", 37),
    "GPS": ("gps", 37),
    u"Kortlæser": ("reader", 37),
    "Alarm": ("warn", 37),
    u"Hjælp": ("help", 37),
    "Barnevogn": ("pram", 32),
    "Op": ("arrow_up", 32),
    "Ned": ("arrow_down", 32),
}

boxes = []
sizes = {}
for attrs, body in cells:
    m = re.search(r'title="([^"]*)"', attrs)
    if not m:
        continue
    found = NAME.get(html.unescape(m.group(1)))
    if found is None:
        continue
    name, shown_at = found
    want = [(name, None)]
    # the two cells that have a second face: the signal crossed out when the page
    # cannot reach the game, and the card reader with the padlock on it
    if name == "signal":
        want.append(("signal_off", ".x"))
    if name == "reader":
        want.append(("reader_lock", ".lock"))
    for id_, shown in want:
        # the same weight on both rules (an id and a class), so the later one wins
        rules = "#%s .x, #%s .lock { display: none }" % (id_, id_)
        if shown:
            rules += " #%s %s { display: block }" % (id_, shown)
        sizes[id_] = shown_at * 2  # twice over, so the shrinking does the smoothing
        rules += " #%s, #%s svg { width: %dpx; height: %dpx }" % (id_, id_, sizes[id_], sizes[id_])
        boxes.append('<div class="box" id="%s"><style>%s</style>%s</div>' % (id_, rules, body))

page = """<!DOCTYPE html><html><head><meta charset="utf-8"><style>
  body { margin: 0; background: transparent; }
  .box svg { display: block; }
</style></head><body>
<svg width="0" height="0" style="position:absolute">%s</svg>
%s
</body></html>
""" % (defs, "\n".join(boxes))

open(out, "w", encoding="utf-8").write(page)
print("\n".join(re.findall(r'class="box" id="([^"]+)"', page)))
