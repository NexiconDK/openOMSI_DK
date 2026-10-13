//! The game's own panels over the picture, made of the plugins' cards (`omsi.ui`'s elements,
//! laid out and drawn by `plugin_ui`). (From openOMSI 0.2.28, commit 3ecd79b4, MIT - the fork
//! keeps only OMSI 2's quick menu behind a tap of Alt; its trip summary, destination list and
//! camera settings are its own.)
//!
//! They have a `UiState` of their own with one owner, the game. Each frame the clicks of the
//! last are worked off, the open panel is set anew (a panel that stays the same is not laid
//! out again) and drawn. While it is open the mouse is its: a click on it goes to it, a click
//! beside it closes it. Nothing open: one `Option` looked at, nothing more.

use omsi_plugin::ui::{Align, Element, Kind, Panel, Rgba, RowAlign, UiState, Weight};

pub(crate) mod quick_menu;

/// The panels' owner in their `UiState` (there is no other).
const GAME: u64 = 1;

/// The panels and what they show.
#[derive(Default)]
pub(crate) struct GamePanels {
    ui: UiState,
    draw: crate::plugin_ui::PluginPanels,
    /// Alt is down and nothing else has been pressed with it: let go, it opens the quick menu.
    alt_alone: bool,
    /// The panel was set to the `UiState` last frame (to be taken away once when it closes).
    shown: bool,
    pub(crate) quick: Option<quick_menu::QuickMenu>,
}

impl GamePanels {
    /// Whether any panel is open.
    pub(crate) fn any(&self) -> bool {
        self.quick.is_some()
    }

    /// Whether the panels have the mouse.
    pub(crate) fn focused(&self) -> bool {
        self.ui.focused()
    }
}

// --- the elements --------------------------------------------------------------------------

pub(super) fn el(id: Option<&str>, kind: Kind) -> Element {
    Element { id: id.map(str::to_string), color: None, grow: false, clickable: false, kind }
}

pub(super) fn text(t: &str, size: f32, weight: Weight) -> Element {
    el(None, Kind::Text { text: omsi_ui::tr(t).into_owned(), size, weight, align: Align::Left, wrap: true })
}

pub(super) fn note(t: &str) -> Element {
    Element { color: Some(Rgba([170, 170, 170, 255])), ..text(t, 12.0, Weight::Regular) }
}

pub(super) fn button(id: &str, t: &str, icon: Option<&str>) -> Element {
    el(Some(id), Kind::Button { text: omsi_ui::tr(t).into_owned(), icon: icon.map(str::to_string) })
}

pub(super) fn row(children: Vec<Element>, align: RowAlign) -> Element {
    el(None, Kind::Row { children, gap: 8.0, align })
}

/// A row of children that share its width alike.
pub(super) fn grow_row(children: Vec<Element>) -> Element {
    row(children.into_iter().map(|c| Element { grow: true, ..c }).collect(), RowAlign::Start)
}

/// The game's card: its dark colour, a little see-through over the picture (`alpha`).
pub(super) fn card(anchor: omsi_plugin::ui::Anchor, x: f32, y: f32, width: f32, alpha: u8, children: Vec<Element>) -> Panel {
    Panel { anchor, x, y, width, padding: 14.0, gap: 8.0, background: Some(Rgba([14, 16, 20, alpha])), radius: 12.0, accent: None, visible: true, clickable: false, draggable: false, children }
}

// --- the frame -----------------------------------------------------------------------------

impl crate::App {
    /// Work off what was clicked, set the open panel and draw it. Nothing open and nothing
    /// shown: nothing done.
    pub(crate) fn frame_game_panels(&mut self, dt: f32, hud: [f32; 4], vr_active: bool) {
        if !self.menus.game_panels.any() && !self.menus.game_panels.shown {
            return;
        }
        let clicks = self.menus.game_panels.ui.take_clicks(GAME);
        for c in clicks {
            if c.panel == "quick" {
                quick_menu::click(self, c.element.as_deref().unwrap_or(""));
            }
        }
        let quick = self.menus.game_panels.quick.as_ref().map(|q| q.panel(self));
        let gp = &mut self.menus.game_panels;
        gp.shown = quick.is_some();
        match quick {
            Some(p) => {
                let _ = gp.ui.set(GAME, "quick", p);
            }
            None => {
                gp.ui.remove(GAME, "quick");
            }
        }
        let (Some(r), Some(scene)) = (self.renderer.as_ref(), self.scene.as_mut()) else { return };
        let dpi = self.window.as_ref().map_or(1.0, |w| w.scale_factor() as f32);
        let size = crate::ui::size_factor(hud[3], dpi, self.settings.ui_scale, self.settings.ui_scale_window);
        let map_open = self.menus.navigator.as_ref().is_some_and(|n| n.map_open());
        let hidden = vr_active || self.menus.game_menu.is_some() || self.menus.chooser.is_some() || map_open;
        let frame = crate::plugin_ui::PanelsFrame {
            hud,
            scale: dpi * size,
            hidden,
            cursor: self.input.cursor,
            dt,
            backdrop: crate::ui::backdrop(self.settings.ui_opacity),
            navigator: None,
        };
        let gp = &mut self.menus.game_panels;
        gp.draw.frame(r, scene, &mut gp.ui, &frame);
        // (the mouse is the panels' while one is open and shown)
        let open = gp.any() && !hidden;
        if open != gp.ui.focused() {
            gp.ui.set_focus(GAME, open);
        }
    }

    /// Whether the game's panels have the mouse.
    pub(crate) fn game_panels_mouse(&self) -> bool {
        self.menus.game_panels.focused()
    }

    /// The left button while the panels have the mouse: a press on a tile goes to it, a press
    /// beside the menu closes it. True when the click was the panels' (the bus does not get it).
    pub(crate) fn game_panel_click(&mut self, pressed: bool) -> bool {
        let gp = &mut self.menus.game_panels;
        if !gp.ui.focused() {
            return false;
        }
        if !pressed {
            return true;
        }
        let (x, y) = self.input.cursor;
        match gp.draw.press_at(&gp.ui, x, y) {
            Some(pr) if pr.hit => gp.ui.click(pr.owner, &pr.panel, pr.element.as_deref()),
            Some(_) => {}
            None => gp.quick = None,
        }
        true
    }

    /// Esc: the panel on top closes. True when one did.
    pub(crate) fn close_game_panel(&mut self) -> bool {
        self.menus.game_panels.quick.take().is_some()
    }

    /// Alt pressed and let go with nothing else between: OMSI 2's quick menu opens (or
    /// closes). Every key comes through here, pressed and released, before its own work.
    pub(crate) fn alt_tap(&mut self, code: winit::keyboard::KeyCode, pressed: bool, repeat: bool) {
        use winit::keyboard::KeyCode;
        let alt = matches!(code, KeyCode::AltLeft | KeyCode::AltRight);
        let gp = &mut self.menus.game_panels;
        match (alt, pressed) {
            (true, true) if !repeat => gp.alt_alone = true,
            (true, false) if std::mem::take(&mut gp.alt_alone) => {
                if self.menus.game_menu.is_none() && self.menus.chooser.is_none() && !self.vr_active() {
                    quick_menu::toggle(self);
                }
            }
            (false, _) if pressed => gp.alt_alone = false,
            _ => {}
        }
    }

    /// A mouse button went down: Alt held with it is no tap.
    pub(crate) fn alt_tap_spoilt(&mut self) {
        self.menus.game_panels.alt_alone = false;
    }
}
