// Compile-time locale catalogs from `assets/i18n/{en,zh-CN}.toml`.
// Missing keys fall back to English (`fallback`), then to the key itself.
rust_i18n::i18n!("assets/i18n", fallback = "en");

use demo::components::*;
use demo::{fonts, theme};
use ply_engine::prelude::*;
use rust_i18n::t;
use std::cell::Cell;
use std::rc::Rc;

/// Locale identifiers backed by `assets/i18n/*.toml`.
mod locale {
    pub const EN: &str = "en";
    pub const ZH: &str = "zh-CN";
}

thread_local! {
    /// Set when the user clicks the language toggle; processed at the top of
    /// the next frame (an `.await` point) so the default font can be swapped.
    static PENDING_LOCALE: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// Minimal stderr logger so rust-i18n's `log-miss-tr` warnings are visible.
struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record) {
        eprintln!("[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

static LOGGER: StderrLogger = StderrLogger;

fn init_logger() {
    log::set_logger(&LOGGER).ok();
    log::set_max_level(log::LevelFilter::Warn);
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Hello lilyco42!".to_owned(),
            window_width: 800,
            window_height: 600,
            high_dpi: true,
            sample_count: 1,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    init_logger();

    // English default: small Latin font (system first, embedded fallback).
    let mut ply = Ply::<()>::new(fonts::en_font()).await;

    let remember = Rc::new(Cell::new(false));
    let notify = Rc::new(Cell::new(true));
    let radio_sel = Rc::new(Cell::new(0usize));
    let slider_val = Rc::new(Cell::new(0.5f32));
    let progress_val = Rc::new(Cell::new(0.25f32));
    let tab_sel = Rc::new(Cell::new(0usize));
    let combo_sel = Rc::new(Cell::new(0usize));
    let list_sel = Rc::new(Cell::new(0usize));
    let sel_save = Rc::new(Cell::new(false));
    let count = Rc::new(Cell::new(0i32));

    // Idle frame pacing: this is a mostly-static UI, so when the user is not
    // interacting we drop from the vsync-synced 60fps to a low idle rate. This
    // cuts the SurfaceView `dequeueBuffer` wait (the main bottleneck found in
    // the perf trace) and CPU/GPU work on battery. Touch/key activity bumps the
    // frame budget back up for the next few frames.
    let mut idle_grace_frames: u32 = 0;
    let mut last_frame_time = std::time::Instant::now();

    loop {
        let has_input = !touches().is_empty()
            || is_mouse_button_pressed(MouseButton::Left)
            || is_key_pressed(KeyCode::F12);
        if has_input {
            idle_grace_frames = 5;
        }

        // Slow to ~15fps when idle, keep full speed near interaction.
        if idle_grace_frames == 0 {
            let target = std::time::Duration::from_secs_f64(1.0 / 15.0);
            let elapsed = last_frame_time.elapsed();
            if elapsed < target {
                std::thread::sleep(target - elapsed);
            }
        } else {
            idle_grace_frames -= 1;
        }
        last_frame_time = std::time::Instant::now();

        // Process a pending language switch: swap the active locale and the
        // global default font (the CJK font is a glyph superset for Chinese).
        if let Some(locale) = PENDING_LOCALE.with(|c| c.replace(None)) {
            rust_i18n::set_locale(locale);
            let font = match locale {
                locale::ZH => fonts::zh_font(),
                _ => fonts::en_font(),
            };
            ply_engine::renderer::FontManager::load_default(font).await;
        }
        clear_background(Color::from(theme::theme().colors.surface).into());
        if is_key_pressed(KeyCode::F12) {
            let current = ply.is_debug_mode();
            ply.set_debug_mode(!current);
        }
        let mut ui = ply.begin();

        render(&mut ui, |ui, region| match region.role {
            RegionRole::Sidebar => {
                sidebar(ui, |ui| {
                    body(ui, t!("sidebar").as_ref());
                    button_id(ui, t!("launch").as_ref());
                    button_id(ui, t!("settings").as_ref());
                    button_id(ui, t!("about").as_ref());
                    divider(ui);
                    title(ui, t!("components").as_ref());
                    for name in [
                        t!("cat_buttons"),
                        t!("cat_forms"),
                        t!("cat_data"),
                        t!("cat_misc"),
                    ] {
                        button_id(ui, name.as_ref());
                    }
                });
            }
            RegionRole::Content => {
                panel(ui, |ui| {
                    headline(ui, t!("app_title").as_ref());
                    body(ui, t!("app_subtitle").as_ref());
                    divider(ui);

                    section(ui, t!("sec_buttons").as_ref(), |ui| {
                        ui.element()
                            .width(grow!())
                            .height(fit!())
                            .layout(|l| l.direction(LeftToRight).gap(8).align(Left, Top))
                            .children(|ui| {
                                button(ui, t!("btn_filled").as_ref(), || {});
                                button_tonal(ui, t!("btn_tonal").as_ref(), || {});
                                button_outlined(ui, t!("btn_outlined").as_ref(), || {});
                                button_text(ui, t!("btn_text").as_ref(), || {});
                            });
                    });

                    section(ui, t!("sec_counter").as_ref(), |ui| {
                        counter(ui, count.clone());
                    });

                    section(ui, t!("sec_text_fields").as_ref(), |ui| {
                        text_field(ui, "name", t!("ph_name").as_ref());
                        text_field_outlined(ui, "email", t!("ph_email").as_ref());
                        let name = ui.get_text_value("name").to_string();
                        if !name.is_empty() {
                            let greeting = t!("greeting", name = &name);
                            body(ui, greeting.as_ref());
                        }
                    });

                    section(ui, t!("sec_checkbox").as_ref(), |ui| {
                        let c =
                            checkbox(ui, "remember", remember.get(), t!("remember_me").as_ref());
                        remember.set(c);
                    });

                    section(ui, t!("sec_switch").as_ref(), |ui| {
                        let s = switch(
                            ui,
                            "notify",
                            notify.get(),
                            t!("enable_notifications").as_ref(),
                        );
                        notify.set(s);
                    });

                    section(ui, t!("sec_radio").as_ref(), |ui| {
                        let r = radio_group(
                            ui,
                            "gender",
                            &[
                                t!("gender_male").as_ref(),
                                t!("gender_female").as_ref(),
                                t!("gender_other").as_ref(),
                            ],
                            radio_sel.get(),
                        );
                        radio_sel.set(r);
                    });

                    section(ui, t!("sec_slider").as_ref(), |ui| {
                        let v = slider(
                            ui,
                            "volume",
                            t!("volume").as_ref(),
                            slider_val.get(),
                            0.0,
                            1.0,
                        );
                        slider_val.set(v);
                        let cur = t!("slider_value", value = format!("{v:.2}"));
                        label(ui, cur.as_ref());
                    });

                    section(ui, t!("sec_progress").as_ref(), |ui| {
                        progress(ui, progress_val.get());
                    });

                    section(ui, t!("sec_tabs").as_ref(), |ui| {
                        let t = tabs(
                            ui,
                            "tab",
                            &[
                                t!("tab_home").as_ref(),
                                t!("tab_discover").as_ref(),
                                t!("tab_mine").as_ref(),
                            ],
                            tab_sel.get(),
                        );
                        tab_sel.set(t);
                    });

                    section(ui, t!("sec_combobox").as_ref(), |ui| {
                        let c = combo(
                            ui,
                            "theme",
                            &[
                                t!("theme_light").as_ref(),
                                t!("theme_dark").as_ref(),
                                t!("theme_system").as_ref(),
                            ],
                            combo_sel.get(),
                        );
                        combo_sel.set(c);
                    });

                    section(ui, t!("sec_listbox").as_ref(), |ui| {
                        let items: Vec<String> = (b'A'..=b'F')
                            .map(|b| t!("list_item", letter = (b as char).to_string()).into_owned())
                            .collect();
                        let refs: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
                        let l = listbox(ui, "files", &refs, list_sel.get(), 4);
                        list_sel.set(l);
                    });

                    section(ui, t!("sec_selectable").as_ref(), |ui| {
                        let s = selectable(
                            ui,
                            "save_local",
                            sel_save.get(),
                            t!("save_locally").as_ref(),
                        );
                        sel_save.set(s);
                    });

                    section(ui, t!("sec_tooltip").as_ref(), |ui| {
                        tooltip(ui, "tt_hint", t!("tooltip_text").as_ref(), |ui| {
                            button_outlined(ui, t!("hover_me").as_ref(), || {});
                        });
                    });
                });
            }
            RegionRole::Status => {
                status_bar(ui, |ui| {
                    label(ui, t!("status_ready").as_ref());
                    button_text(ui, t!("lang_toggle").as_ref(), || {
                        let next = if &*rust_i18n::locale() == locale::ZH {
                            locale::EN
                        } else {
                            locale::ZH
                        };
                        PENDING_LOCALE.with(|c| c.set(Some(next)));
                    });
                });
            }
            RegionRole::Progress => {
                log_progress(ui, "log_progress", progress_val.get());
            }
        });

        ui.show(|_| {}).await;
        next_frame().await;
    }
}

fn section(ui: &mut Ui<'_, ()>, name: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    title(ui, name);
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding(12))
        .background_color(theme::theme().colors.surface_container_low)
        .corner_radius(theme::theme().shapes.radius_md)
        .children(inner);
}

// Kotlin-Compose style counter: Row { Button("-"); Text(count); Button("+") }
fn counter(ui: &mut Ui<'_, ()>, value: Rc<Cell<i32>>) {
    let theme = theme::theme();
    ui.element()
        .id("counter")
        .width(fit!())
        .height(fit!())
        .background_color(theme.colors.surface_variant)
        .corner_radius(theme.shapes.radius_md)
        .layout(|l| {
            l.direction(LeftToRight)
                .gap(4)
                .align(CenterX, CenterY)
                .padding(4)
        })
        .children(|ui| {
            step_button(ui, ("counter", 0), "-", value.clone(), -1);
            ui.element()
                .width(fixed!(56.0))
                .height(fixed!(36.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(&value.get().to_string(), |t| {
                        t.font_size(theme.text.title_size)
                            .color(theme.colors.on_surface)
                    });
                });
            step_button(ui, ("counter", 1), "+", value.clone(), 1);
        });
}

fn step_button(
    ui: &mut Ui<'_, ()>,
    id: (&'static str, u32),
    symbol: &str,
    value: Rc<Cell<i32>>,
    delta: i32,
) {
    let theme = theme::theme();
    ui.element()
        .id(id)
        .width(fixed!(36.0))
        .height(fixed!(36.0))
        .on_press(move |_, _| {
            value.set(value.get() + delta);
        })
        .accessibility(|a| a.button(symbol))
        .children(|ui| {
            let bg = if ui.pressed() {
                theme::PRESSED_PRIMARY
            } else if ui.hovered() || ui.focused() {
                theme::HOVER_PRIMARY
            } else {
                theme.colors.primary
            };
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(bg)
                .corner_radius(18.0)
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(symbol, |t| t.font_size(20).color(theme.colors.on_primary));
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    // Single test: rust-i18n's locale is global, so a parallel test would race.
    #[test]
    fn catalogs_resolve_and_switch() {
        rust_i18n::set_locale(locale::EN);
        assert_eq!(t!("sidebar").as_ref(), "Sidebar");
        assert_eq!(t!("greeting", name = "World").as_ref(), "Hi, World!");
        assert_eq!(
            t!("slider_value", value = format!("{:.2}", 0.5)).as_ref(),
            "Current: 0.50"
        );
        assert_eq!(t!("list_item", letter = "A").as_ref(), "Item A");

        rust_i18n::set_locale(locale::ZH);
        assert_eq!(t!("sidebar").as_ref(), "侧边栏");
        assert_eq!(t!("greeting", name = "世界").as_ref(), "你好, 世界!");

        // Missing key degrades to the key itself.
        assert_eq!(t!("no_such_key").as_ref(), "no_such_key");

        rust_i18n::set_locale(locale::EN);
    }
}
