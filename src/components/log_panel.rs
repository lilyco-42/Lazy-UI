//! Stateful `LogPanel` — demonstrates the [`Component`] trait (prompt 2.5's
//! "日志组件"): it owns an accumulating line buffer that persists across
//! frames, appends a line every few seconds, caps the buffer, and renders the
//! whole log inside a scrollable `panel`.
//!
//! Clicking the panel collapses it to just the newest line (so the section's
//! close button above stays reachable instead of being pushed away as the log
//! grows); clicking again expands back to the full log.

use ply_engine::prelude::*;
use std::collections::VecDeque;

use crate::components::component::Component;
use crate::components::{body, divider, label, panel_opt, title};

/// A self-updating log panel. Appends a line every `interval` seconds, keeps at
/// most `max_lines`, and auto-scrolls (the enclosing `panel` scrolls).
pub struct LogPanel {
    lines: VecDeque<String>,
    last: f64,
    seq: u32,
    interval: f64,
    max_lines: usize,
    /// Click-to-collapse: when true only the newest line is shown.
    collapsed: bool,
}

impl LogPanel {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::new(),
            last: macroquad::time::get_time(),
            seq: 0,
            interval: 2.0,
            max_lines: 200,
            collapsed: false,
        }
    }

    /// Appends a log line, capping the buffer at `max_lines`.
    pub fn push(&mut self, line: String) {
        self.lines.push_back(line);
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }
}

impl Default for LogPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for LogPanel {
    fn on_mount(&mut self, _ui: &mut Ui<'_, ()>) {
        self.push("[mount] 日志面板已挂载".into());
    }

    fn render(&mut self, ui: &mut Ui<'_, ()>) {
        let now = macroquad::time::get_time();
        if now - self.last >= self.interval {
            self.last = now;
            self.seq += 1;
            self.push(format!("[{:>8.2}] tick #{}", now, self.seq));
        }

        // The whole panel box is the toggle target: clicking collapses to the
        // last line / expands back. Kept out of the `children` closure so the
        // state flip reads cleanly after the element is laid out this frame.
        let toggle_id = Id::new("log_panel_toggle");
        panel_opt(ui, Some(toggle_id.clone()), |ui| {
            title(ui, "日志面板 (stateful)");
            divider(ui);
            if self.collapsed {
                if let Some(last) = self.lines.back() {
                    body(ui, last);
                }
                label(ui, &format!("▶ 点击展开 (共 {} 条)", self.lines.len()));
            } else {
                for line in &self.lines {
                    body(ui, line);
                }
                label(ui, "▼ 点击折叠日志");
            }
        });
        if ui.is_just_pressed(toggle_id.clone()) {
            self.collapsed = !self.collapsed;
        }
    }
}
