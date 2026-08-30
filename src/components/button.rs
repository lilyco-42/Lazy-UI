//! Buttons — filled / tonal / outlined / text (M3).
//!
//! Styling comes from `assets/components/button.toml` (the component's CSS);
//! unset fields fall back to the M3 theme. Per-call overrides via
//! [`config::Style::with`] merge over the stylesheet, CSS-cascade style.

use ply_engine::prelude::*;

use crate::components::config::{self, ButtonConfig, ButtonStateConfig};
use crate::theme::{self, TRANSPARENT};
use std::cell::RefCell;
use std::collections::HashMap;

// Per-button cross-frame highlight factor. Immediate mode rebuilds the tree
// every frame, so hover/pressed color transitions need to remember where each
// button's intensity was last frame. Keyed by id string (explicit id, else
// label — fine for a purely-visual micro-interaction).
thread_local! {
    static BTN_HL: RefCell<HashMap<String, f32>> = RefCell::new(HashMap::new());
}

/// The highlight factor transitions toward `target` on each frame. A fixed
/// per-frame gain (not frame-time) keeps this headless-safe: `macroquad`
/// frame-time/clock reads panic when called off the main `#[macroquad::main]`
/// thread, which the headless test harness runs on. The visual cost of a fixed
/// gain is negligible for a 150ms micro-interaction.
fn highlight(key: &str, target: f32) -> f32 {
    BTN_HL.with(|m| {
        let mut m = m.borrow_mut();
        let v = m.entry(key.to_string()).or_insert(0.0);
        // Exponential approach: closes ~12% of the remaining gap each frame.
        *v += (target - *v) * 0.12;
        *v
    })
}

/// The resolved palette for one button render (colors already merged).
#[derive(Clone, Copy)]
struct Palette {
    bg: Color,
    hover: Color,
    pressed: Color,
    fg: Color,
    border: Option<Color>,
}

fn resolve_palette(state: &ButtonStateConfig, fallback: Palette) -> Palette {
    Palette {
        bg: state.background.map(Color::from).unwrap_or(fallback.bg),
        hover: state.hover.map(Color::from).unwrap_or(fallback.hover),
        pressed: state.pressed.map(Color::from).unwrap_or(fallback.pressed),
        fg: state.foreground.map(Color::from).unwrap_or(fallback.fg),
        border: state.border.map(Color::from).or(fallback.border),
    }
}

/// Effective button config: per-call attrs > `<name>.toml` > theme defaults.
fn button_cfg() -> ButtonConfig {
    config::effective(config::Style::current().button, ButtonConfig::get(), ButtonConfig::merged)
}

fn rounded_btn(
    ui: &mut Ui<'_, ()>,
    id: Option<Id>,
    label: &str,
    mut on_click: impl FnMut() + 'static,
    cfg: &ButtonConfig,
    p: Palette,
    width: ply_engine::layout::Sizing,
) {
    let theme = theme::theme();
    let height = cfg.height.unwrap_or(theme.shapes.button_height);
    let radius = cfg.radius.unwrap_or(height * 0.5);

    let mut el = ui.element();
    if let Some(id) = id {
        el = el.id(id);
    }
    el.width(width)
        .height(fixed!(height))
        .corner_radius(radius)
        .on_press(move |_, _| on_click())
        .accessibility(|a| {
            a.button(label)
                .ring_color(theme.colors.primary)
                .ring_width(2)
        })
        .children(|ui| {
            let target = if ui.pressed() {
                1.0
            } else if ui.hovered() || ui.focused() {
                0.5
            } else {
                0.0
            };
            let hl = highlight(label, target);
            // Two-segment crossfade: bg -> hover (hl 0..0.5), hover -> pressed (hl 0.5..1).
            let state = if hl <= 0.5 {
                p.bg.lerp_srgb(p.hover, hl * 2.0)
            } else {
                p.hover.lerp_srgb(p.pressed, (hl - 0.5) * 2.0)
            };
            let mut el = ui
                .element()
                .width(grow!())
                .height(grow!())
                .background_color(state)
                .corner_radius(radius)
                .layout(|l| {
                    l.padding((0, cfg.pad_x.unwrap_or(24.0) as u16, 0, cfg.pad_x.unwrap_or(24.0) as u16))
                        .align(CenterX, CenterY)
                });
            if let Some(bc) = p.border {
                el = el.border(|b| b.all(1).color(bc));
            }
            el.children(|ui| {
                ui.text(label, |t| {
                    t.font_size(cfg.font_size.unwrap_or(theme.text.label_size))
                        .color(p.fg)
                });
            });
        });
}

/// High-emphasis filled button. `button(ui, "Save", || save())`
pub fn button(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = variant_state(&cfg, ButtonKind::Filled);
    let p = variant_palette(&theme, ButtonKind::Filled, &state);
    rounded_btn(ui, None, label, on_click, &cfg, p, fit!());
}

/// Medium-emphasis tonal button (secondary container).
pub fn button_tonal(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = variant_state(&cfg, ButtonKind::Tonal);
    let p = variant_palette(&theme, ButtonKind::Tonal, &state);
    rounded_btn(ui, None, label, on_click, &cfg, p, fit!());
}

/// Outlined button.
pub fn button_outlined(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = variant_state(&cfg, ButtonKind::Outlined);
    let p = variant_palette(&theme, ButtonKind::Outlined, &state);
    rounded_btn(ui, None, label, on_click, &cfg, p, fit!());
}

/// Low-emphasis text button.
pub fn button_text(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = variant_state(&cfg, ButtonKind::Text);
    let p = variant_palette(&theme, ButtonKind::Text, &state);
    rounded_btn(ui, None, label, on_click, &cfg, p, fit!());
}

/// The M3 button emphasis used by a label-only (id) button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Filled,
    Tonal,
    Outlined,
    Text,
}

/// The palette fallback for a variant, matched up with its `cfg.<variant>`.
fn variant_palette(theme: &theme::Theme, kind: ButtonKind, state: &ButtonStateConfig) -> Palette {
    match kind {
        ButtonKind::Filled => resolve_palette(state, Palette {
            bg: theme.colors.primary.into(),
            hover: theme::HOVER_PRIMARY.into(),
            pressed: theme::PRESSED_PRIMARY.into(),
            fg: theme.colors.on_primary.into(),
            border: None,
        }),
        ButtonKind::Tonal => resolve_palette(state, Palette {
            bg: theme.colors.secondary_container.into(),
            hover: theme::HOVER_TONAL.into(),
            pressed: theme::PRESSED_TONAL.into(),
            fg: theme.colors.on_secondary_container.into(),
            border: None,
        }),
        ButtonKind::Outlined => resolve_palette(state, Palette {
            bg: TRANSPARENT.into(),
            hover: theme::HOVER_OUTLINED.into(),
            pressed: theme::PRESSED_OUTLINED.into(),
            fg: theme.colors.primary.into(),
            border: Some(theme.colors.outline.into()),
        }),
        ButtonKind::Text => resolve_palette(state, Palette {
            bg: TRANSPARENT.into(),
            hover: theme::HOVER_TEXT.into(),
            pressed: theme::PRESSED_TEXT.into(),
            fg: theme.colors.primary.into(),
            border: None,
        }),
    }
}

/// The `cfg` palette subfield for a variant.
fn variant_state(cfg: &ButtonConfig, kind: ButtonKind) -> ButtonStateConfig {
    match kind {
        ButtonKind::Filled => cfg.filled.unwrap_or_default(),
        ButtonKind::Tonal => cfg.tonal.unwrap_or_default(),
        ButtonKind::Outlined => cfg.outlined.unwrap_or_default(),
        ButtonKind::Text => cfg.text.unwrap_or_default(),
    }
}

/// Label-only button — convention over configuration: no callback, auto-generated
/// id derived from the label. Returns the `Id`; poll it with `ui.is_just_pressed(id)`
/// or `ui.is_just_pressed("label")` to detect activation.
///
/// `button_id(ui, "hello")` ≈ Compose `Button(onClick = null)`.
pub fn button_id(ui: &mut Ui<'_, ()>, label: &str) -> Id {
    button_id_kind(ui, label, ButtonKind::Text)
}

/// Label-only button in an explicit M3 emphasis. `按钮()` uses [`ButtonKind::Filled`]
/// so a plain call reads as the primary action while still returning its `Id`.
pub fn button_id_kind(ui: &mut Ui<'_, ()>, label: &str, kind: ButtonKind) -> Id {
    let cfg = button_cfg();
    let theme = theme::theme();
    let id: Id = Id::from((label, 0u32));
    let state = variant_state(&cfg, kind);
    let p = variant_palette(&theme, kind, &state);
    rounded_btn(ui, Some(id.clone()), label, || {}, &cfg, p, fit!());
    id
}

/// High-emphasis filled button with a FIXED pixel width. Use when the label can
/// change length (e.g. a copy button flipping between "copy" and "✓ copied")
/// and the button must not jump around.
pub fn button_fixed(ui: &mut Ui<'_, ()>, label: &str, width: f32, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = variant_state(&cfg, ButtonKind::Filled);
    let p = variant_palette(&theme, ButtonKind::Filled, &state);
    rounded_btn(ui, None, label, on_click, &cfg, p, fixed!(width));
}
