//! A copy button — copies `text` to the system clipboard on press, with a
//! brief "copied" confirmation. The button has a FIXED width (measured from the
//! longer of the two labels with the actual font), so it doesn't resize when
//! the label flips between "copy" and "copied".
//!
//! Cross-platform: native uses the system clipboard (via miniquad); on web the
//! generated `ply_bundle.js` must be patched so `sapp_set_clipboard` calls
//! `navigator.clipboard` — run `tools/patch-web-clipboard.py` after `plyx web`.

use ply_engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::components::*;
use crate::theme;

thread_local! {
    /// id → wall-clock time of the last copy (for the "copied" feedback).
    static COPIED_AT: RefCell<HashMap<u32, f64>> = RefCell::new(HashMap::new());
}

/// Measures `text` with the app's current default font (so widths match what
/// actually renders), in px.
fn measure(text: &str, size: u16) -> f32 {
    let fm = ply_engine::renderer::FONT_MANAGER.lock().unwrap();
    let font = fm.get_default();
    macroquad::text::measure_text(text, font, size, 1.0).width
}

/// Renders a button that copies `text` to the clipboard on press. Shows
/// `copied_label` for ~1.5s after a press, then `label` again. Fixed width so
/// the button never jumps.
pub fn copy_button(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    text: &'static str,
    label: &str,
    copied_label: &str,
) {
    let key = Id::from(id).id;
    let now = macroquad::time::get_time();
    let copied = COPIED_AT
        .with(|m| m.borrow().get(&key).map(|&t| now - t < 1.5))
        .unwrap_or(false);
    let shown = if copied { copied_label } else { label };

    // Stable width: longest label + horizontal padding (2 × pad_x ≈ 48) + buffer.
    let size = theme::theme().text.label_size;
    let width = measure(label, size)
        .max(measure(copied_label, size))
        + theme::px(52.0);

    // `gap` from copy_button.toml = left margin, so a preceding label never
    // overlaps the button (labels measure wider than their ink).
    let gap = crate::components::config::CopyButtonConfig::get()
        .gap
        .unwrap_or(0.0);
    let margin = theme::px(gap) as u16;

    ui.element()
        .width(fit!())
        .height(fit!())
        .layout(|l| l.padding((0, 0, 0, margin)))
        .children(|ui| {
            button_fixed(ui, shown, width, move || {
                miniquad::window::clipboard_set(text);
                COPIED_AT.with(|m| m.borrow_mut().insert(key, macroquad::time::get_time()));
            });
        });
}
