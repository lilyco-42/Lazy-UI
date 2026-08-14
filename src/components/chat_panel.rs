//! Chat panel — conversation history bubbles, quick questions and a text input.
//!
//! Convention over configuration: the panel only *reads* [`ChatPanelState`]
//! (history, questions, labels) and *reports* user actions through
//! [`ChatPanelEvents`]; the app owns the reply logic and state transitions.
//! Styling comes from `assets/components/chat_panel.toml`; unset fields fall
//! back to the M3 theme.
//!
//! Fills its parent: bubbles live in a scrollable top section, the quick
//! question row and the input row are pinned to the bottom. Works inside a
//! windowed layout region *and* as a full-window overlay (e.g. a desktop pet
//! with a transparent background).

use std::rc::Rc;
use std::cell::RefCell;

use ply_engine::prelude::*;

use super::config::{self, ChatPanelConfig};
use super::{button_id, text_field};
use crate::theme::{self, Theme};

/// One conversation turn (a bubble).
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub from_user: bool,
    pub text: String,
}

impl ChatMessage {
    pub fn user(text: impl Into<String>) -> Self {
        Self { from_user: true, text: text.into() }
    }
    pub fn pet(text: impl Into<String>) -> Self {
        Self { from_user: false, text: text.into() }
    }
}

/// Chat panel state. Lives in the app — immediate mode: the component reads it
/// every frame and never mutates it.
#[derive(Debug, Clone)]
pub struct ChatPanelState {
    /// Conversation history, oldest first. Only the last `max_bubbles` render.
    pub history: Vec<ChatMessage>,
    /// Id of the text input element (also the storage key for its value).
    pub input_id: &'static str,
    pub input_placeholder: &'static str,
    pub send_label: &'static str,
    /// Quick-question buttons shown above the input (click = ask).
    pub quick_questions: &'static [&'static str],
}

impl Default for ChatPanelState {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            input_id: "chat_panel_input",
            input_placeholder: "说点什么…",
            send_label: "发送",
            quick_questions: &[
                "在吗？",
                "吃饭了吗？",
                "想我了吗？",
                "心情不好",
                "晚安",
                "你会一直陪我吗？",
            ],
        }
    }
}

/// Actions the panel produced this frame. The caller drains it (e.g. right
/// after `ui.show()`) and answers each submitted text.
#[derive(Debug, Default)]
pub struct ChatPanelEvents {
    /// Texts to submit: quick-question clicks, the send button, and Enter in
    /// the input field.
    pub submitted: Vec<String>,
}

fn cfg() -> ChatPanelConfig {
    config::effective(
        config::Style::current().chat_panel,
        ChatPanelConfig::get(),
        ChatPanelConfig::merged,
    )
}

/// Chat panel: bubble history + quick-question row + input row.
///
/// Events accumulate into `events` (which is *not* cleared here, so callers
/// that process asynchronously — e.g. after `ui.show()` — can drain reliably).
pub fn chat_panel(ui: &mut Ui<'_, ()>, state: &ChatPanelState, events: &Rc<RefCell<ChatPanelEvents>>) {
    let c = cfg();
    let theme = theme::theme();

    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(c.background.map(Color::from).unwrap_or(theme::TRANSPARENT.into()))
        .layout(|l| {
            l.direction(TopToBottom)
                .gap(c.gap.unwrap_or(8.0) as u16)
                .padding(c.padding.unwrap_or(12.0) as u16)
        })
        .children(|ui| {
            // 1) History — scrollable, growing section (bubbles at the top).
            ui.element()
                .width(grow!())
                .height(grow!())
                .overflow(|o| o.scroll_y())
                .layout(|l| l.direction(TopToBottom).gap(c.bubble_gap.unwrap_or(6.0) as u16))
                .children(|ui| {
                    let max = c.max_bubbles.unwrap_or(6) as usize;
                    let start = state.history.len().saturating_sub(max);
                    for msg in state.history.iter().skip(start) {
                        bubble(ui, msg, &c, theme);
                    }
                });

            // 2) Quick questions — one text button per question.
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| {
                    l.direction(LeftToRight)
                        .gap(c.quick_gap.unwrap_or(6.0) as u16)
                        .align(Left, CenterY)
                })
                .children(|ui| {
                    for q in state.quick_questions {
                        let id = button_id(ui, q);
                        if ui.is_just_pressed(id) {
                            events.borrow_mut().submitted.push((*q).to_string());
                        }
                    }
                });

            // 3) Input row — text field + send button (Enter also submits).
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.direction(LeftToRight).gap(c.input_gap.unwrap_or(8.0) as u16))
                .children(|ui| {
                    text_field(ui, state.input_id, state.input_placeholder);
                    let send_id = button_id(ui, state.send_label);
                    let enter = macroquad::prelude::is_key_pressed(macroquad::prelude::KeyCode::Enter);
                    if ui.is_just_pressed(send_id) || enter {
                        let text = ui.get_text_value(state.input_id).trim().to_string();
                        if !text.is_empty() {
                            events.borrow_mut().submitted.push(text);
                            ui.set_text_value(state.input_id, "");
                        }
                    }
                });
        });
}

/// One message bubble, aligned left (pet) or right (user), width-limited so
/// long lines wrap instead of overflowing the window.
fn bubble(ui: &mut Ui<'_, ()>, msg: &ChatMessage, c: &ChatPanelConfig, theme: &Theme) {
    let (bg, fg, align_x) = if msg.from_user {
        (
            c.user_background.map(Color::from).unwrap_or(theme.colors.primary_container.into()),
            c.user_foreground.map(Color::from).unwrap_or(theme.colors.on_primary_container.into()),
            AlignX::Right,
        )
    } else {
        (
            c.pet_background.map(Color::from).unwrap_or(theme.colors.surface_container.into()),
            c.pet_foreground.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into()),
            AlignX::Left,
        )
    };
    let radius = c.bubble_radius.unwrap_or(theme.shapes.radius_md);
    let width = ply_engine::layout::Sizing::Percent(c.bubble_width.unwrap_or(0.72).clamp(0.2, 0.95));
    let pad_x = c.bubble_pad_x.unwrap_or(10.0) as u16;
    let pad_y = c.bubble_pad_y.unwrap_or(6.0) as u16;

    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(LeftToRight).align(align_x, CenterY))
        .children(|ui| {
            ui.element()
                .width(width)
                .height(fit!())
                .background_color(bg)
                .corner_radius(radius)
                .layout(|l| l.padding((pad_y, pad_x, pad_y, pad_x)))
                .children(|ui| {
                    ui.text(&msg.text, |t| t.font_size(theme.text.body_size).color(fg));
                });
        });
}