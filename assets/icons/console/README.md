# (the fork) The icons of the driver console

These are the pictures of the console's screen in the bus
(`crates/omsi-app/src/console_screen.html`): the icons of the status strip along the top
and of the bar along its foot, the three marks the driving screen puts against a stop -
the red hand and the two blue arrows - and the gradient of its blue panel. The game holds
them itself - see `console_screen.rs` - so nothing is written into the player's folders
and the artwork cannot go missing.

They are not drawn by hand. They are the tablet's own
(`crates/omsi-app/src/tablet.html`: the `strip-cells` and `foot-keys` templates with the
`<defs>` of gradients next to them, the `HAND` and `ARROW` of its script, and the
`background` of `.drive .panel`), rasterised, because the tablet runs in a browser and the
screen in the bus runs in openOMSI's own little HTML engine - and that engine draws no
SVG, no CSS gradient, and no picture written into the page itself.

One of the cells has a second face, so it is two pictures: the card reader wears a
padlock while the readers are shut (`reader_lock`). `make_page.py` can draw the signal
crossed out as well, the way the tablet does while it cannot reach the game, but the
screen in the bus is drawn by the game itself and so never loses it.

Every icon is twice the size it is drawn at, so the shrinking does the smoothing: the
strip's are 74 and drawn at 37 (46 on the tablet), the foot bar's 64 and drawn at 32 (40
on the tablet), the stop marks 86 and drawn at 43 (54 on the tablet). `blue_panel.png` is
the odd one out: it is 16 wide because it is stretched over the panel, and it only runs
downwards.

## Making them again

After a change to an icon in `tablet.html`, these are made again, so the two screens keep
the same icons:

    python3 make_page.py ../../../crates/omsi-app/src/tablet.html /tmp/icons.html
    python3 shoot.py /tmp/icons.html .

`shoot.py` wants playwright with chromium (`pip install playwright && playwright install
chromium`). Neither script is part of the build.
