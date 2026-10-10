#!/usr/bin/env python3
"""The fork's release notes: what FORK.md says now that it did not say at the previous
release, under the headings it stands under.

    fork_notes.py OLD NEW > changes.md

OLD is FORK.md as the previous release had it (may be missing or empty: then everything
is new), NEW is FORK.md now. A bullet that was reworded counts as new (it says something
else now). With nothing new: a line saying so.
"""
import sys


def sections(text):
    """[(heading, [bullet, ...]), ...] in the file's order; a bullet's continuation lines
    stay with it."""
    out = []
    heading = None
    for line in text.splitlines():
        if line.startswith("### "):
            heading = line[4:].strip()
            out.append((heading, []))
        elif heading is None:
            continue
        elif line.startswith("- "):
            out[-1][1].append(line.rstrip())
        elif line.startswith("  ") and out[-1][1]:
            out[-1][1][-1] += "\n" + line.rstrip()
    return out


def main():
    old_path, new_path = sys.argv[1], sys.argv[2]
    try:
        with open(old_path, encoding="utf-8") as f:
            old = f.read()
    except OSError:
        old = ""
    with open(new_path, encoding="utf-8") as f:
        new = f.read()
    known = {b.strip() for _, bullets in sections(old) for b in bullets}
    parts = []
    for heading, bullets in sections(new):
        fresh = [b for b in bullets if b.strip() not in known]
        if fresh:
            parts.append(f"### {heading}\n" + "\n".join(fresh))
    if parts:
        print("\n\n".join(parts))
    else:
        print("Små rettelser.")


if __name__ == "__main__":
    main()
