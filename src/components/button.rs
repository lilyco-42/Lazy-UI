//! Buttons — filled / tonal / outlined / text (M3).
//!
//! Styling comes from `assets/components/button.toml` (the component's CSS);
//! unset fields fall back to the M3 theme. Per-call overrides via
//! [`config::Style::with`] merge over the stylesheet, CSS-cascade style.

use ply_engine::prelude::*;

use crate::components::config::{self, ButtonConfig, ButtonStateConfig};
use crate::theme::{self, TRANSPARENT};

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
) {
    let theme = theme::theme();
    let height = cfg.height.unwrap_or(theme.shapes.button_height);
    let radius = cfg.radius.unwrap_or(height * 0.5);

    let mut el = ui.element();
    if let Some(id) = id {
        el = el.id(id);
    }
    el.width(fit!())
        .height(fixed!(height))
        .corner_radius(radius)
        .on_press(move |_, _| on_click())
        .accessibility(|a| a.button(label))
        .children(|ui| {
            let state: Color = if ui.pressed() {
                p.pressed
            } else if ui.hovered() {
                p.hover
            } else {
                p.bg
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
    let state = cfg.filled.unwrap_or_default();
    let p = resolve_palette(&state,
        Palette {
            bg: theme.colors.primary.into(),
            hover: theme::HOVER_PRIMARY.into(),
            pressed: theme::PRESSED_PRIMARY.into(),
            fg: theme.colors.on_primary.into(),
            border: None,
        },
    );
    rounded_btn(ui, None, label, on_click, &cfg, p);
}

/// Medium-emphasis tonal button (secondary container).
pub fn button_tonal(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = cfg.tonal.unwrap_or_default();
    let p = resolve_palette(&state,
        Palette {
            bg: theme.colors.secondary_container.into(),
            hover: theme::HOVER_TONAL.into(),
            pressed: theme::PRESSED_TONAL.into(),
            fg: theme.colors.on_secondary_container.into(),
            border: None,
        },
    );
    rounded_btn(ui, None, label, on_click, &cfg, p);
}

/// Outlined button.
pub fn button_outlined(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = cfg.outlined.unwrap_or_default();
    let p = resolve_palette(&state,
        Palette {
            bg: TRANSPARENT.into(),
            hover: theme::HOVER_OUTLINED.into(),
            pressed: theme::PRESSED_OUTLINED.into(),
            fg: theme.colors.primary.into(),
            border: Some(theme.colors.outline.into()),
        },
    );
    rounded_btn(ui, None, label, on_click, &cfg, p);
}

/// Low-emphasis text button.
pub fn button_text(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let cfg = button_cfg();
    let theme = theme::theme();
    let state = cfg.text.unwrap_or_default();
    let p = resolve_palette(&state,
        Palette {
            bg: TRANSPARENT.into(),
            hover: theme::HOVER_TEXT.into(),
            pressed: theme::PRESSED_TEXT.into(),
            fg: theme.colors.primary.into(),
            border: None,
        },
    );
    rounded_btn(ui, None, label, on_click, &cfg, p);
}

/// Label-only button — convention over configuration: no callback, auto-generated
/// id derived from the label. Returns the `Id`; poll it with `ui.is_just_pressed(id)`
/// or `ui.is_just_pressed("label")` to detect activation.
///
/// `button_id(ui, "hello")` ≈ Compose `Button(onClick = null)`.
pub fn button_id(ui: &mut Ui<'_, ()>, label: &str) -> Id {
    let cfg = button_cfg();
    let theme = theme::theme();
    let id: Id = Id::from((label, 0u32));
    let state = cfg.text.unwrap_or_default();
    let p = resolve_palette(&state,
        Palette {
            bg: TRANSPARENT.into(),
            hover: theme::HOVER_TEXT.into(),
            pressed: theme::PRESSED_TEXT.into(),
            fg: theme.colors.primary.into(),
            border: None,
        },
    );
    rounded_btn(ui, Some(id.clone()), label, || {}, &cfg, p);
    id
}
