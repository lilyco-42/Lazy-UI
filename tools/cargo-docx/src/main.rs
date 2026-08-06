//! `cargo docx` — component reference documentation generator.
//!
//! Scans `<root>/src/components/*.rs`, extracts each public component function's
//! signature and `///` doc comment, merges in hand-written metadata (category,
//! parameter docs, examples), and emits a single self-contained
//! `docs/components.html` in a FastAPI/Swagger-style layout: sticky grouped
//! navigation, live search, and copy-able code examples.
//!
//! Install once: `cargo install --path tools/cargo-docx`
//! Then run anywhere: `cargo docx`

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Extracted source model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Param {
    name: String,
    ty: String,
}

#[derive(Debug, Clone)]
struct Component {
    module: String,
    name: String,
    doc_lines: Vec<String>,
    signature: String,
    params: Vec<Param>,
    ret: String,
    file: String,
    line: usize,
}

fn parse_params(sig: &str) -> Vec<Param> {
    let open = sig.find('(').unwrap_or(0);
    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut cur = String::new();
    let mut chars = sig[open + 1..].chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '(' | '<' | '[' | '{' => {
                depth += 1;
                cur.push(c);
            }
            ')' | '>' | ']' | '}' if depth > 0 => {
                depth -= 1;
                cur.push(c);
            }
            ')' if depth == 0 => break,
            ',' if depth == 0 => {
                params.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        params.push(cur.trim().to_string());
    }

    params
        .into_iter()
        .filter_map(|raw| {
            let colon = raw.find(':')?;
            let name = raw[..colon].trim().trim_start_matches("mut ").to_string();
            let ty = raw[colon + 1..].trim().trim_end_matches(',').trim().to_string();
            if name.is_empty() || name == "self" || name.starts_with('_') {
                return None;
            }
            Some(Param { name, ty })
        })
        .collect()
}

fn ret_of(sig: &str) -> String {
    let sig = sig.trim();
    let open = sig.find('(').unwrap_or(0);
    let mut depth = 0isize;
    let mut close = None;
    for (idx, ch) in sig[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return "()".to_string();
    };
    let after = &sig[close + 1..];
    let after = match after.find('{').or_else(|| after.find(';')) {
        Some(end) => &after[..end],
        None => after,
    };
    let after = after.trim().trim_start_matches("->").trim();
    if after.is_empty() || after == "()" {
        "()".to_string()
    } else {
        after.to_string()
    }
}

/// Cut a merged `pub fn ... { body }` down to just the declaration (params +
/// return type), dropping the function body.
fn decl_only(sig: &str) -> String {
    let sig = sig.trim();
    let open = sig.find('(').unwrap_or(0);
    let mut depth = 0isize;
    let mut close = None;
    for (idx, ch) in sig[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return sig.to_string();
    };
    let after = &sig[close + 1..];
    match after.find('{') {
        Some(brace) => sig[..close + 1 + brace].trim().to_string(),
        None => sig.to_string(),
    }
}

fn extract(path: &Path) -> Vec<Component> {
    let src = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        if !line.starts_with("pub fn ") {
            i += 1;
            continue;
        }

        // Collect contiguous /// doc lines directly above.
        let mut doc_start = i;
        while doc_start > 0 {
            let prev = lines[doc_start - 1].trim();
            if prev.starts_with("///") {
                doc_start -= 1;
            } else {
                break;
            }
        }
        let doc_lines: Vec<String> = lines[doc_start..i]
            .iter()
            .map(|l| l.trim().trim_start_matches("///").trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Merge the function signature and body until the closing brace at the
        // top level (handles multiline params, closures, generics).
        let mut sig = String::new();
        let mut depth = 0isize;
        let mut j = i;
        let mut finished = false;
        while j < lines.len() {
            let l = lines[j];
            sig.push_str(l);
            sig.push(' ');
            for ch in l.chars() {
                match ch {
                    '(' | '<' | '[' | '{' => depth += 1,
                    ')' | '>' | ']' | '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth <= 0 {
                finished = true;
                break;
            }
            j += 1;
        }
        if !finished {
            i += 1;
            continue;
        }
        let sig = sig.trim().to_string();
        let (name, _) = sig["pub fn ".len()..]
            .split_once('(')
            .unwrap_or((sig.as_str(), ""));

        out.push(Component {
            module: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            name: name.trim().to_string(),
            doc_lines,
            signature: decl_only(&sig),
            params: parse_params(&sig),
            ret: ret_of(&sig),
            file: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            line: i + 1,
        });
        i = j + 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Hand-written metadata: categories, parameter docs, examples
// ---------------------------------------------------------------------------

struct RawMeta {
    name: &'static str,
    category: &'static str,
    summary: &'static str,
    param_docs: &'static [(&'static str, &'static str)],
    ret_doc: &'static str,
    example: &'static str,
}

const META: &[RawMeta] = &[
    // ---- Layout -----------------------------------------------------------
    RawMeta {
        name: "render",
        category: "Layout",
        summary: "Declarative app skeleton. Reads assets/app_layout.toml and AUTO-INFERS the flex composition; calls `content` once per region so you fill each region with real components.",
        param_docs: &[
            ("ui", "The current Ui context (from `ply.begin()`)."),
            ("content", "Closure called once per layout region with `(ui, &Region)`."),
        ],
        ret_doc: "()",
        example: "render(&mut ui, |ui, region| match region.role {\n    RegionRole::Sidebar => sidebar(ui, |ui| {\n        button_id(ui, \"Start\");\n    }),\n    RegionRole::Content => panel(ui, |ui| {\n        headline(ui, \"App\");\n    }),\n    RegionRole::Status => status_bar(ui, |ui| {\n        label(ui, \"Ready\");\n    }),\n    RegionRole::Progress => log_progress(ui, \"log\", 0.42),\n});",
    },
    RawMeta {
        name: "sidebar",
        category: "Layout",
        summary: "Left navigation rail. Fills its layout region (e.g. the 240px sidebar region in app_layout.toml). Configurable via assets/components/sidebar.toml.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("inner", "Closure that paints the nav items (buttons, dividers, …)."),
        ],
        ret_doc: "()",
        example: "sidebar(ui, |ui| {\n    button_id(ui, \"Launch\");\n    button_id(ui, \"Settings\");\n    divider(ui);\n});",
    },
    RawMeta {
        name: "panel",
        category: "Layout",
        summary: "Main content card, fills its layout region. Configurable via assets/components/panel.toml.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("inner", "Closure that paints the panel contents."),
        ],
        ret_doc: "()",
        example: "panel(ui, |ui| {\n    headline(ui, \"Settings\");\n    body(ui, \"Configure your app.\");\n});",
    },
    RawMeta {
        name: "status_bar",
        category: "Layout",
        summary: "Bottom status bar (full width, slim). Configurable via assets/components/status_bar.toml.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("inner", "Closure that paints status content (labels, buttons)."),
        ],
        ret_doc: "()",
        example: "status_bar(ui, |ui| {\n    label(ui, \"Ready\");\n    button_text(ui, \"Language\", || {});\n});",
    },
    RawMeta {
        name: "log_progress",
        category: "Layout",
        summary: "nvim-dialog style bottom progress bar pinned to the bottom of its region. `value` in 0.0..=1.0.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("value", "Progress fraction 0.0..=1.0."),
        ],
        ret_doc: "()",
        example: "log_progress(ui, \"log_progress\", 0.42);",
    },
    RawMeta {
        name: "divider",
        category: "Layout",
        summary: "Thin horizontal divider line (M3 outline_variant).",
        param_docs: &[("ui", "The current Ui context.")],
        ret_doc: "()",
        example: "divider(ui);",
    },
    // ---- Buttons ----------------------------------------------------------
    RawMeta {
        name: "button",
        category: "Buttons",
        summary: "High-emphasis filled button (M3 primary).",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("label", "Button text."),
            ("on_click", "Callback invoked on press."),
        ],
        ret_doc: "()",
        example: "button(ui, \"Save\", || save());",
    },
    RawMeta {
        name: "button_tonal",
        category: "Buttons",
        summary: "Medium-emphasis tonal button (M3 secondary container).",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("label", "Button text."),
            ("on_click", "Callback invoked on press."),
        ],
        ret_doc: "()",
        example: "button_tonal(ui, \"Share\", || share());",
    },
    RawMeta {
        name: "button_outlined",
        category: "Buttons",
        summary: "Outlined button — transparent fill with a 1px outline.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("label", "Button text."),
            ("on_click", "Callback invoked on press."),
        ],
        ret_doc: "()",
        example: "button_outlined(ui, \"Cancel\", || close());",
    },
    RawMeta {
        name: "button_text",
        category: "Buttons",
        summary: "Low-emphasis text button — no fill, primary text.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("label", "Button text."),
            ("on_click", "Callback invoked on press."),
        ],
        ret_doc: "()",
        example: "button_text(ui, \"Learn more\", || open_help());",
    },
    RawMeta {
        name: "button_id",
        category: "Buttons",
        summary: "Label-only button — convention over configuration: no callback, auto-generated id derived from the label. Returns the Id; poll it with `ui.is_just_pressed(id)`.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("label", "Button text — also the source of the auto id."),
        ],
        ret_doc: "Id — poll with `ui.is_just_pressed(id)` to detect activation.",
        example: "let id = button_id(ui, \"hello\");\nif ui.is_just_pressed(id) { … }",
    },
    // ---- Selection ----------------------------------------------------------
    RawMeta {
        name: "checkbox",
        category: "Selection",
        summary: "M3 checkbox. Returns the NEW checked state — store it back.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id (`impl Into<Id>`)."),
            ("checked", "Current checked state."),
            ("label", "Row label."),
        ],
        ret_doc: "bool — the new checked state.",
        example: "let c = checkbox(ui, \"remember\", remember, \"Remember me\");\nremember = c;",
    },
    RawMeta {
        name: "switch",
        category: "Selection",
        summary: "M3 switch. Returns the NEW checked state — store it back.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("checked", "Current checked state."),
            ("label", "Row label."),
        ],
        ret_doc: "bool — the new checked state.",
        example: "let s = switch(ui, \"notify\", notify, \"Notifications\");\nnotify = s;",
    },
    RawMeta {
        name: "radio",
        category: "Selection",
        summary: "Single radio row. Returns true if it was activated this frame.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("selected", "Whether this option is selected."),
            ("label", "Option label."),
        ],
        ret_doc: "bool — true if activated this frame.",
        example: "let sel = radio(ui, (\"gender\", 0), sel == 0, \"Male\");",
    },
    RawMeta {
        name: "radio_group",
        category: "Selection",
        summary: "Radio group. Returns the newly selected index.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Group id (stable `&'static str`)."),
            ("options", "Option labels."),
            ("selected", "Currently selected index."),
        ],
        ret_doc: "usize — the newly selected index.",
        example: "let sel = radio_group(ui, \"gender\", &[\"Male\", \"Female\", \"Other\"], sel);",
    },
    RawMeta {
        name: "selectable",
        category: "Selection",
        summary: "M3 selectable list row. Returns true if activated this frame.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("selected", "Whether the row is selected."),
            ("label", "Row label."),
        ],
        ret_doc: "bool — true if activated this frame.",
        example: "let hit = selectable(ui, \"row\", row_sel, \"Item A\");",
    },
    RawMeta {
        name: "tabs",
        category: "Selection",
        summary: "M3 tabs. Returns the newly selected index.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Tabs id (stable `&'static str`)."),
            ("items", "Tab labels."),
            ("selected", "Currently selected index."),
        ],
        ret_doc: "usize — the newly selected index.",
        example: "let tab = tabs(ui, \"tab\", &[\"Home\", \"Discover\", \"Mine\"], tab);",
    },
    RawMeta {
        name: "combo",
        category: "Selection",
        summary: "M3 dropdown (ComboBox). Returns the newly selected index. Open/close state is kept internally.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Combo id (stable `&'static str`)."),
            ("options", "Dropdown option labels."),
            ("selected", "Currently selected index."),
        ],
        ret_doc: "usize — the newly selected index.",
        example: "let sel = combo(ui, \"theme\", &[\"Light\", \"Dark\", \"System\"], sel);",
    },
    RawMeta {
        name: "listbox",
        category: "Selection",
        summary: "M3 scrollable list box. Returns the newly selected index.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Listbox id (stable `&'static str`)."),
            ("options", "Row labels."),
            ("selected", "Currently selected index."),
            ("visible", "Number of rows visible before scrolling."),
        ],
        ret_doc: "usize — the newly selected index.",
        example: "let sel = listbox(ui, \"files\", &[\"a.txt\", \"b.txt\"], sel, 4);",
    },
    // ---- Input ---------------------------------------------------------------
    RawMeta {
        name: "slider",
        category: "Input",
        summary: "M3 slider. Returns the dragged value. Drag the handle or click anywhere on the track.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("label", "Slider label shown above the track."),
            ("value", "Current value."),
            ("min", "Minimum value."),
            ("max", "Maximum value."),
        ],
        ret_doc: "f32 — the new value.",
        example: "let v = slider(ui, \"volume\", \"Volume\", v, 0.0, 1.0);",
    },
    RawMeta {
        name: "text_field",
        category: "Input",
        summary: "M3 filled text field. Value lives in Ply under `id`; read it with `ui.get_text_value(id)`.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id (also the value key)."),
            ("placeholder", "Placeholder text."),
        ],
        ret_doc: "() — read value with `ui.get_text_value(id)`.",
        example: "text_field(ui, \"name\", \"Your name\");\nlet name = ui.get_text_value(\"name\");",
    },
    RawMeta {
        name: "text_field_outlined",
        category: "Input",
        summary: "M3 outlined text field. Same value API as `text_field`.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id (also the value key)."),
            ("placeholder", "Placeholder text."),
        ],
        ret_doc: "() — read value with `ui.get_text_value(id)`.",
        example: "text_field_outlined(ui, \"email\", \"you@example.com\");",
    },
    // ---- Feedback --------------------------------------------------------------
    RawMeta {
        name: "progress",
        category: "Feedback",
        summary: "M3 linear progress indicator. `fraction` in 0.0..=1.0.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("fraction", "Progress 0.0..=1.0."),
        ],
        ret_doc: "()",
        example: "progress(ui, progress_value);",
    },
    RawMeta {
        name: "tooltip",
        category: "Feedback",
        summary: "M3 tooltip: wraps arbitrary content and shows a label bubble on hover.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("id", "Stable element id."),
            ("text", "Tooltip label."),
            ("inner", "Closure that paints the hovered content."),
        ],
        ret_doc: "()",
        example: "tooltip(ui, \"tt_hint\", \"Save your work\", |ui| {\n    button_outlined(ui, \"Hover me\", || {});\n});",
    },
    // ---- Typography -------------------------------------------------------------
    RawMeta {
        name: "headline",
        category: "Typography",
        summary: "Headline text (28px) — page titles.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("text", "Text to render."),
        ],
        ret_doc: "()",
        example: "headline(ui, \"Settings\");",
    },
    RawMeta {
        name: "title",
        category: "Typography",
        summary: "Title text (22px) — section titles.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("text", "Text to render."),
        ],
        ret_doc: "()",
        example: "title(ui, \"General\");",
    },
    RawMeta {
        name: "body",
        category: "Typography",
        summary: "Body text (16px) — default content.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("text", "Text to render."),
        ],
        ret_doc: "()",
        example: "body(ui, \"Welcome to the demo.\");",
    },
    RawMeta {
        name: "label",
        category: "Typography",
        summary: "Label text (14px, muted) — captions and annotations.",
        param_docs: &[
            ("ui", "The current Ui context."),
            ("text", "Text to render."),
        ],
        ret_doc: "()",
        example: "label(ui, \"v1.0.0\");",
    },
];

fn meta_for(name: &str) -> RawMetaLike {
    META.iter()
        .find(|m| m.name == name)
        .map(|m| RawMetaLike {
            category: m.category,
            param_docs: m.param_docs,
            example: m.example,
            ret_doc: m.ret_doc,
            summary: m.summary,
        })
        .unwrap_or_else(|| RawMetaLike {
            category: "Uncategorized",
            param_docs: &[],
            example: "",
            ret_doc: "",
            summary: "",
        })
}

#[derive(Clone, Copy)]
struct RawMetaLike {
    category: &'static str,
    param_docs: &'static [(&'static str, &'static str)],
    example: &'static str,
    ret_doc: &'static str,
    summary: &'static str,
}

/// Category for a component: curated META category when known, otherwise the
/// module name (title-cased) so unknown components still get grouped.
fn category_of(c: &Component) -> String {
    let cat = meta_for(&c.name).category;
    if cat != "Uncategorized" {
        return cat.to_string();
    }
    let mut chars = c.module.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Misc".to_string(),
    }
}

// ---------------------------------------------------------------------------
// HTML rendering
// ---------------------------------------------------------------------------

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_attr(s: &str) -> String {
    esc(s).replace('\n', " ")
}

fn tok(seg: &str, class: &str) -> String {
    format!("<span class=\"{class}\">{}</span>", esc(seg))
}

/// Lightweight Rust syntax highlighter using the Ply docs site palette
/// (Catppuccin Mocha): keywords mauve, functions blue, types yellow, strings
/// green, numbers/constants peach, comments gray, operators teal, lifetimes
/// cyan.
fn highlight_rust(src: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
        "type", "unsafe", "unsized", "use", "where", "while",
    ];
    const PAIRS: &[&str] = &["::", "->", "=>", "..", "<=", ">=", "==", "!=", "&&", "||"];

    let bytes: Vec<(usize, char)> = src.char_indices().collect();
    let n = bytes.len();
    let at = |i: usize| if i < n { bytes[i].1 } else { '\0' };
    let seg = |a: usize, b: usize| -> &str {
        if a >= b || a >= n {
            return "";
        }
        let sa = bytes[a].0;
        let sb = if b >= n { src.len() } else { bytes[b].0 };
        &src[sa..sb]
    };

    let mut i = 0usize;
    let mut out = String::new();
    while i < n {
        let c = at(i);
        if c.is_whitespace() {
            out.push(c);
            i += 1;
            continue;
        }
        // comments
        if c == '/' && at(i + 1) == '/' {
            let mut j = i;
            while j < n && at(j) != '\n' {
                j += 1;
            }
            out.push_str(&tok(seg(i, j), "c-cmt"));
            i = j;
            continue;
        }
        if c == '/' && at(i + 1) == '*' {
            let mut j = i + 2;
            while j + 1 < n && !(at(j) == '*' && at(j + 1) == '/') {
                j += 1;
            }
            j = (j + 2).min(n);
            out.push_str(&tok(seg(i, j), "c-cmt"));
            i = j;
            continue;
        }
        // string literals
        if c == '"' {
            let mut j = i + 1;
            while j < n && at(j) != '"' {
                if at(j) == '\\' {
                    j += 1;
                }
                j += 1;
            }
            j = (j + 1).min(n);
            out.push_str(&tok(seg(i, j), "c-str"));
            i = j;
            continue;
        }
        // char literal or lifetime
        if c == '\'' {
            if at(i + 1) == '\\' && at(i + 2) != '\0' && at(i + 3) == '\'' {
                out.push_str(&tok(seg(i, i + 4), "c-str"));
                i += 4;
                continue;
            }
            if at(i + 1) != '\0' && at(i + 2) == '\'' {
                out.push_str(&tok(seg(i, i + 3), "c-str"));
                i += 3;
                continue;
            }
            let mut j = i + 1;
            while j < n && (at(j).is_alphanumeric() || at(j) == '_') {
                j += 1;
            }
            out.push_str(&tok(seg(i, j), "c-life"));
            i = j;
            continue;
        }
        // numbers
        if c.is_ascii_digit() || (c == '.' && at(i + 1).is_ascii_digit()) {
            let mut j = i;
            if c == '0'
                && (at(i + 1) == 'x' || at(i + 1) == 'X' || at(i + 1) == 'b' || at(i + 1) == 'o')
            {
                j = i + 2;
                while j < n && (at(j).is_ascii_hexdigit() || at(j) == '_') {
                    j += 1;
                }
            } else {
                while j < n && (at(j).is_ascii_digit() || at(j) == '_') {
                    j += 1;
                }
                if at(j) == '.' {
                    j += 1;
                    while j < n && (at(j).is_ascii_digit() || at(j) == '_') {
                        j += 1;
                    }
                }
                if at(j) == 'e' || at(j) == 'E' {
                    j += 1;
                    if at(j) == '+' || at(j) == '-' {
                        j += 1;
                    }
                    while j < n && at(j).is_ascii_digit() {
                        j += 1;
                    }
                }
            }
            out.push_str(&tok(seg(i, j), "c-num"));
            i = j;
            continue;
        }
        // identifiers
        if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while j < n && (at(j).is_alphanumeric() || at(j) == '_') {
                j += 1;
            }
            let word = seg(i, j);
            if KEYWORDS.contains(&word) {
                let class = if word == "true" || word == "false" {
                    "c-bool"
                } else {
                    "c-kw"
                };
                out.push_str(&tok(word, class));
            } else if !word.is_empty()
                && word.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_')
            {
                out.push_str(&tok(word, "c-const"));
            } else if word.starts_with(|ch: char| ch.is_uppercase()) {
                out.push_str(&tok(word, "c-type"));
            } else if at(j) == '(' || at(j) == '!' {
                out.push_str(&tok(word, "c-fn"));
            } else {
                out.push_str(&tok(word, "c-var"));
            }
            i = j;
            continue;
        }
        // operators / punctuation
        let two = seg(i, i + 2);
        let len = if PAIRS.contains(&two) {
            2
        } else if c == '.' && seg(i, i + 3) == "..." {
            3
        } else {
            1
        };
        let piece = seg(i, i + len);
        let class = if piece == "_" {
            "c-var"
        } else if piece == "#" {
            "c-kw"
        } else {
            "c-punc"
        };
        out.push_str(&tok(piece, class));
        i += len;
    }
    out
}

fn code_html(s: &str) -> String {
    format!("<pre><code>{}</code></pre>", highlight_rust(s))
}

/// Interactive visual preview for a component: plain HTML/CSS/JS mockups in the
/// M3 light palette, wired up by `initPreviews()` in the generated page so the
/// docs feel alive without a WASM instance per component.
fn preview_of(name: &str) -> &'static str {
    match name {
        "render" => r#"<div class="pv-app"><div class="pv-sidebar"><span class="pv-h2" style="font-size:14px;padding:4px 6px">App</span><button class="pv-btn text" style="padding:6px 10px;width:100%;font-size:12px">Launch</button><button class="pv-btn text" style="padding:6px 10px;width:100%;font-size:12px">Settings</button></div><div class="pv-app-main"><div class="pv-panel"><span class="pv-h2" style="font-size:15px">Settings</span><span class="pv-body" style="font-size:12px">Configure your app.</span></div><div class="pv-status"><span class="pv-label">Ready</span><span class="pv-label">v1.0.0</span></div></div></div><div class="pv-progressbar"><div class="pv-fill" style="width:42%"></div></div>"#,
        "sidebar" => r#"<div class="pv-sidebar" style="width:150px"><span class="pv-h2" style="font-size:14px;padding:4px 6px">App</span><button class="pv-btn text" style="padding:6px 10px;font-size:12px">Launch</button><button class="pv-btn text" style="padding:6px 10px;font-size:12px">Settings</button><div class="pv-div"></div><span class="pv-label" style="padding:2px 6px">v1.0.0</span></div>"#,
        "panel" => r#"<div class="pv-panel" style="min-width:200px"><span class="pv-h2" style="font-size:15px">Settings</span><span class="pv-body" style="font-size:12px">Configure your app.</span><button class="pv-btn" style="align-self:flex-start;padding:6px 14px;font-size:12px">Save</button></div>"#,
        "status_bar" => r#"<div class="pv-status" style="min-width:260px"><span class="pv-label">Ready</span><button class="pv-btn text" style="padding:2px 8px;font-size:11px">Language</button></div>"#,
        "log_progress" => r#"<div class="pv-col" style="min-width:240px"><span class="pv-label">Installing packages… 42%</span><div class="pv-progress" style="width:100%"><div class="pv-fill" style="width:42%"></div></div></div>"#,
        "divider" => r#"<div class="pv-col" style="width:180px"><span class="pv-label">Section A</span><div class="pv-div"></div><span class="pv-label">Section B</span></div>"#,
        "button" => r#"<button class="pv-btn">Save</button>"#,
        "button_tonal" => r#"<button class="pv-btn tonal">Share</button>"#,
        "button_outlined" => r#"<button class="pv-btn outlined">Cancel</button>"#,
        "button_text" => r#"<button class="pv-btn text">Learn more</button>"#,
        "button_id" => r#"<button class="pv-btn text">hello</button>"#,
        "checkbox" => r#"<div class="pv-row"><span class="pv-check on"></span><span class="pv-itemlabel">Remember me</span></div>"#,
        "switch" => r#"<div class="pv-row"><span class="pv-switch on"></span><span class="pv-itemlabel">Notifications</span></div>"#,
        "radio" => r#"<div class="pv-row"><span class="pv-radio on"></span><span class="pv-itemlabel">Male</span></div>"#,
        "radio_group" => r#"<div class="pv-radio-group"><div class="pv-rowradio"><span class="pv-radio on"></span><span class="pv-itemlabel">Male</span></div><div class="pv-rowradio"><span class="pv-radio"></span><span class="pv-itemlabel">Female</span></div><div class="pv-rowradio"><span class="pv-radio"></span><span class="pv-itemlabel">Other</span></div></div>"#,
        "selectable" => r#"<div class="pv-selectable on"><span class="pv-itemlabel">Item A</span></div>"#,
        "tabs" => r#"<div class="pv-tabs"><span class="pv-tab active">Home</span><span class="pv-tab">Discover</span><span class="pv-tab">Mine</span></div>"#,
        "combo" => r#"<div class="pv-combo"><span class="pv-sel">Light</span><span class="arr">▼</span><div class="pv-menu"><div class="pv-opt">Light</div><div class="pv-opt">Dark</div><div class="pv-opt">System</div></div></div>"#,
        "listbox" => r#"<div class="pv-list" style="width:150px"><div class="pv-item">a.txt</div><div class="pv-item sel">b.txt</div><div class="pv-item">c.txt</div></div>"#,
        "slider" => r#"<div class="pv-slider"><span class="pv-label">Volume</span><input type="range" class="pv-range" min="0" max="100" value="60"><span class="pv-label pv-val">60</span></div>"#,
        "text_field" => r#"<input class="pv-field" placeholder="Your name">"#,
        "text_field_outlined" => r#"<input class="pv-field out" placeholder="you@example.com">"#,
        "progress" => r#"<div class="pv-progress"><div class="pv-fill" style="width:42%"></div></div>"#,
        "tooltip" => r#"<div class="pv-tipwrap"><span class="pv-tip">Save your work</span><button class="pv-btn outlined" style="font-size:12px">Hover me</button></div>"#,
        "headline" => r#"<span class="pv-h1">Settings</span>"#,
        "title" => r#"<span class="pv-h2">General</span>"#,
        "body" => r#"<span class="pv-body">Welcome to the demo.</span>"#,
        "label" => r#"<span class="pv-label">v1.0.0</span>"#,
        _ => "",
    }
}

fn render_component(c: &Component) -> String {
    let meta = meta_for(&c.name);
    let mut h = String::new();

    let preview = preview_of(&c.name);
    let preview_html = if preview.is_empty() {
        String::new()
    } else {
        format!("<div class=\"pv-preview\"><div class=\"pv\">{preview}</div></div>")
    };

    let doc = if c.doc_lines.is_empty() {
        String::new()
    } else {
        let d = c.doc_lines.join(" ");
        format!("<p class=\"doc\">{}</p>", esc(&d))
    };

    let summary = if meta.summary.is_empty() {
        String::new()
    } else {
        format!("<p class=\"summary\">{}</p>", esc(meta.summary))
    };

    let mut rows = String::new();
    for p in &c.params {
        let doc = meta
            .param_docs
            .iter()
            .find(|(n, _)| *n == p.name)
            .map(|(_, d)| *d)
            .unwrap_or("");
        rows.push_str(&format!(
            "<tr><td class=\"pname\"><code>{}</code></td><td class=\"ptype\"><code>{}</code></td><td>{}</td></tr>",
            esc(&p.name),
            esc(&p.ty),
            esc(doc)
        ));
    }
    if rows.is_empty() {
        rows = "<tr><td colspan=\"3\" class=\"muted\">no parameters</td></tr>".to_string();
    }
    let params_html = format!("<table class=\"params\"><thead><tr><th>Parameter</th><th>Type</th><th>Description</th></tr></thead><tbody>{rows}</tbody></table>");

    let d = if meta.ret_doc.is_empty() { "" } else { meta.ret_doc };
    let ret_html = format!(
        "<p class=\"ret\"><span class=\"pill\">→</span> <code>{}</code> <span class=\"retdoc\">{}</span></p>",
        esc(&c.ret),
        esc(d)
    );

    let example_html = if meta.example.is_empty() {
        String::new()
    } else {
        format!(
            "<div class=\"example\"><div class=\"exhead\"><span>Example</span><button class=\"copy\" onclick=\"copyCode(this)\">Copy</button></div>{}</div>",
            code_html(meta.example)
        )
    };

    let _ = write!(
        h,
        "<article class=\"card\" data-name=\"{}\" data-module=\"{}\" id=\"fn-{}\">
  <header>
    <div class=\"fnline\">
      <code class=\"fname\">{}</code>
      <span class=\"module\">{}</span>
      <span class=\"loc\">{}:{}</span>
    </div>
  </header>
  {}
  {}
  {}
  {}
  {}
  {}
  {}
</article>",
        html_attr(&c.name),
        html_attr(&c.module),
        html_attr(&c.name),
        esc(&c.name),
        esc(&c.module),
        esc(&c.file),
        c.line,
        preview_html,
        summary,
        doc,
        code_html(&c.signature),
        params_html,
        ret_html,
        example_html,
    );
    h
}

fn build_html(components: &[Component], live_url: Option<&str>) -> String {
    // Category order: curated META order first, then any module-derived groups.
    let mut categories: Vec<String> = Vec::new();
    for m in META {
        if !categories.contains(&m.category.to_string()) {
            categories.push(m.category.to_string());
        }
    }
    for c in components {
        let cat = category_of(c);
        if !categories.contains(&cat) {
            categories.push(cat);
        }
    }

    let mut sections = String::new();
    for cat in &categories {
        let items: Vec<&Component> = components
            .iter()
            .filter(|c| category_of(c).as_str() == cat.as_str())
            .collect();
        if items.is_empty() {
            continue;
        }
        let mut nav = String::new();
        let mut cards = String::new();
        for c in &items {
            nav.push_str(&format!(
                "<a class=\"navlink\" href=\"#fn-{}\" data-name=\"{}\">{}</a>",
                html_attr(&c.name),
                html_attr(&c.name),
                esc(&c.name)
            ));
            cards.push_str(&render_component(c));
        }
        let _ = write!(
            sections,
            "<section class=\"group\" data-cat=\"{}\">
  <h2 class=\"cat\">{}</h2>
  <nav class=\"subnav\">{}</nav>
  <div class=\"cards\">{}</div>
</section>",
            html_attr(cat),
            esc(cat),
            nav,
            cards
        );
    }

    let total = components.len();
    let cats = categories.len();

    let live = match live_url {
        Some(url) => format!(
            "<section class=\"group\" id=\"live\">\n  <h2 class=\"cat\">Live Demo</h2>\n  <p class=\"summary\">The real app compiled to WASM. Click around — every component is interactive.</p>\n  <div class=\"liveframe\"><iframe src=\"{url}\" loading=\"lazy\" allow=\"autoplay\" tabindex=\"0\"></iframe></div>\n</section>"
        ),
        None => String::new(),
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ply M3 Component Reference</title>
<style>
@font-face {{
  font-family:'Geist';
  src:url('https://plyx.iz.rs/fonts/Geist-Variable.woff2') format('woff2');
  font-weight:100 900;
}}
@font-face {{
  font-family:'Geist Mono';
  src:url('https://plyx.iz.rs/fonts/GeistMono-Variable.woff2') format('woff2');
  font-weight:100 900;
}}
:root {{
  --bg:#16171E; --bg-2:#101218; --surface:#1E1E2E; --surface-2:#313244; --border:#2A2B3C;
  --text:#CDD6F4; --subtext:#A6ADC8; --muted:#7F849C;
  --primary:#89B4FA; --teal:#94E2D5;
  --c-kw:#CBA6F7; --c-fn:#89B4FA; --c-type:#F9E2AF; --c-str:#A6E3A1; --c-num:#FAB387;
  --c-const:#FAB387; --c-cmt:#9399B2; --c-punc:#94E2D5; --c-life:#89DCEB; --c-bool:#F38BA8; --c-var:#CDD6F4;
  --code-bg:#1E1E2E; --code-fg:#CDD6F4;
}}
* {{ box-sizing:border-box; margin:0; padding:0; }}
html {{ scroll-behavior:smooth; }}
body {{
  font-family:'Geist','Segoe UI',system-ui,-apple-system,sans-serif;
  background:var(--bg); color:var(--text); line-height:1.6;
}}
code, pre {{ font-family:'Geist Mono','JetBrains Mono','Cascadia Code',Consolas,monospace; }}
.app {{ display:grid; grid-template-columns:300px 1fr; min-height:100vh; }}
/* Sidebar */
.sidebar {{
  position:sticky; top:0; height:100vh; overflow-y:auto;
  background:var(--bg-2); border-right:1px solid var(--border);
  padding:24px 20px 48px;
}}
.logo {{ display:flex; align-items:center; gap:10px; margin-bottom:2px; }}
.logo-badge {{
  width:26px; height:26px; border-radius:8px; flex:none;
  background:linear-gradient(135deg,#89B4FA,#CBA6F7);
  color:#0E1016; font-weight:700; font-size:14px; display:flex; align-items:center; justify-content:center;
}}
.sidebar h1 {{ font-size:18px; color:var(--text); letter-spacing:-.01em; }}
.sidebar .sub {{ font-size:12px; color:var(--muted); margin-bottom:18px; }}
.search {{
  width:100%; padding:9px 12px; border:1px solid var(--surface-2);
  border-radius:8px; font-size:13px; margin-bottom:18px;
  background:var(--surface); color:var(--text); font-family:inherit;
}}
.search::placeholder {{ color:var(--muted); }}
.search:focus {{ outline:2px solid var(--primary); outline-offset:-1px; }}
.catnav {{ margin-bottom:18px; }}
.catnav h3 {{ font-size:11px; text-transform:uppercase; letter-spacing:.1em; color:var(--muted); margin:16px 0 6px; }}
.catnav a {{ display:block; padding:4px 10px; border-radius:6px; color:var(--subtext); text-decoration:none; font-size:13px; }}
.catnav a:hover {{ background:var(--surface); color:var(--text); }}
.catnav a.active {{ background:var(--primary); color:#0E1016; font-weight:600; }}
/* Main */
.main {{ padding:32px 44px 96px; max-width:1000px; }}
.statbar {{ display:flex; gap:8px; margin-bottom:24px; }}
.badge {{ background:var(--surface); color:var(--subtext); font-size:12px; padding:4px 12px; border-radius:999px; border:1px solid var(--border); }}
.group {{ margin-bottom:56px; }}
.cat {{ font-size:24px; font-weight:700; color:var(--text); letter-spacing:-.01em; margin-bottom:14px; border-bottom:1px solid var(--border); padding-bottom:10px; }}
.subnav {{ display:flex; flex-wrap:wrap; gap:6px; margin:12px 0; }}
.subnav a {{ font-size:12px; padding:3px 10px; border:1px solid var(--border); border-radius:999px; color:var(--muted); text-decoration:none; }}
.subnav a:hover {{ border-color:var(--primary); color:var(--primary); }}
/* Cards */
.card {{
  background:var(--surface); border:1px solid var(--border); border-radius:12px;
  padding:22px 26px; margin:18px 0;
}}
.card .fnline {{ display:flex; align-items:baseline; gap:10px; flex-wrap:wrap; margin-bottom:12px; }}
.fname {{ font-size:19px; font-weight:600; color:var(--primary); }}
.module {{ font-size:11px; color:var(--muted); background:var(--bg-2); padding:2px 8px; border-radius:999px; border:1px solid var(--border); }}
.loc {{ font-size:11px; color:var(--muted); margin-left:auto; }}
.summary {{ font-size:15px; margin-bottom:8px; }}
.doc {{ font-size:13px; color:var(--subtext); margin-bottom:12px; font-style:italic; }}
/* Preview */
.pv-preview {{ background:var(--code-bg); border:1px solid var(--border); border-radius:10px; padding:16px; margin-bottom:16px; }}
.pv {{ background:#FEF7FF; border-radius:10px; padding:20px; display:flex; align-items:center; justify-content:center; gap:12px; flex-wrap:wrap; color:#1D1B20; font-family:'Geist','Segoe UI',system-ui,sans-serif; }}
.pv-row {{ display:flex; align-items:center; gap:8px; cursor:pointer; }}
.pv-col {{ display:flex; flex-direction:column; gap:8px; align-items:flex-start; }}
.pv-rowradio {{ display:flex; align-items:center; gap:8px; cursor:pointer; }}
.pv-radio-group {{ display:flex; flex-direction:column; gap:8px; align-items:flex-start; }}
.pv-btn {{ display:inline-flex; align-items:center; justify-content:center; padding:8px 18px; border-radius:999px; font-size:13px; font-weight:500; background:#6750A4; color:#fff; border:1px solid transparent; white-space:nowrap; line-height:1.2; cursor:pointer; font-family:inherit; transition:filter .1s, transform .1s; }}
.pv-btn.tonal {{ background:#EADDFF; color:#21005D; }}
.pv-btn.outlined {{ background:transparent; color:#6750A4; border-color:#79747E; }}
.pv-btn.text {{ background:transparent; color:#6750A4; }}
.pv-btn:active, .pv-btn.pressed {{ filter:brightness(.88); transform:translateY(1px); }}
.pv-check {{ width:18px; height:18px; border-radius:4px; border:2px solid #79747E; background:#fff; position:relative; flex:none; cursor:pointer; }}
.pv-check.on {{ background:#6750A4; border-color:#6750A4; }}
.pv-check.on::after {{ content:'✓'; position:absolute; inset:0; color:#fff; font-size:12px; line-height:15px; text-align:center; }}
.pv-switch {{ width:44px; height:24px; border-radius:999px; background:#CAC4D0; position:relative; flex:none; cursor:pointer; }}
.pv-switch.on {{ background:#6750A4; }}
.pv-switch::after {{ content:''; position:absolute; top:3px; left:3px; width:18px; height:18px; border-radius:50%; background:#fff; box-shadow:0 1px 2px rgba(0,0,0,.3); transition:left .15s; }}
.pv-switch.on::after {{ left:23px; }}
.pv-radio {{ width:18px; height:18px; border-radius:50%; border:2px solid #79747E; flex:none; position:relative; cursor:pointer; }}
.pv-radio.on {{ border-color:#6750A4; }}
.pv-radio.on::after {{ content:''; position:absolute; inset:3px; border-radius:50%; background:#6750A4; }}
.pv-itemlabel {{ font-size:13px; color:#1D1B20; }}
.pv-selectable {{ border-radius:8px; padding:8px 12px; min-width:130px; cursor:pointer; }}
.pv-selectable.on {{ background:#EADDFF; }}
.pv-selectable.on .pv-itemlabel {{ color:#21005D; }}
.pv-tabs {{ display:flex; gap:2px; }}
.pv-tab {{ padding:8px 14px; font-size:13px; color:#49454F; border-bottom:2px solid transparent; cursor:pointer; }}
.pv-tab.active {{ color:#6750A4; border-bottom-color:#6750A4; font-weight:500; }}
.pv-combo {{ position:relative; display:flex; align-items:center; justify-content:space-between; gap:18px; padding:8px 12px; border:1px solid #79747E; border-radius:8px; font-size:13px; color:#1D1B20; background:#fff; min-width:150px; cursor:pointer; }}
.pv-combo .arr {{ color:#49454F; font-size:10px; }}
.pv-menu {{ display:none; position:absolute; top:calc(100% + 4px); left:0; right:0; background:#fff; border:1px solid #CAC4D0; border-radius:8px; box-shadow:0 6px 18px rgba(0,0,0,.18); z-index:5; overflow:hidden; }}
.pv-combo.open .pv-menu {{ display:block; }}
.pv-opt {{ padding:8px 12px; font-size:13px; color:#1D1B20; cursor:pointer; }}
.pv-opt:hover {{ background:#EADDFF; color:#21005D; }}
.pv-list {{ border:1px solid #79747E; border-radius:10px; overflow:hidden; min-width:140px; background:#fff; }}
.pv-item {{ padding:8px 12px; font-size:13px; color:#1D1B20; cursor:pointer; }}
.pv-item:hover {{ background:#F3EDF7; }}
.pv-item.sel {{ background:#EADDFF; color:#21005D; }}
.pv-slider {{ display:flex; align-items:center; gap:10px; }}
.pv-range {{ -webkit-appearance:none; appearance:none; width:160px; height:4px; border-radius:999px; background:linear-gradient(to right,#6750A4 0%,#6750A4 var(--v,60%),#CAC4D0 var(--v,60%),#CAC4D0 100%); outline:none; }}
.pv-range::-webkit-slider-thumb {{ -webkit-appearance:none; appearance:none; width:16px; height:16px; border-radius:50%; background:#6750A4; border:2px solid #fff; box-shadow:0 1px 2px rgba(0,0,0,.3); cursor:pointer; }}
.pv-range::-moz-range-thumb {{ width:16px; height:16px; border-radius:50%; background:#6750A4; border:2px solid #fff; box-shadow:0 1px 2px rgba(0,0,0,.3); cursor:pointer; }}
.pv-val {{ min-width:22px; text-align:center; }}
.pv-field {{ padding:10px 12px; background:#E6E0E9; border:0; border-radius:8px 8px 0 0; border-bottom:2px solid #6750A4; font-size:13px; color:#1D1B20; min-width:180px; font-family:inherit; outline:none; }}
.pv-field.out {{ background:transparent; border:1px solid #79747E; border-radius:8px; }}
.pv-field::placeholder {{ color:#49454F; }}
.pv-progress {{ width:160px; height:6px; border-radius:999px; background:#CAC4D0; position:relative; overflow:hidden; }}
.pv-fill {{ position:absolute; left:0; top:0; bottom:0; border-radius:999px; background:#6750A4; }}
.pv-tipwrap {{ position:relative; display:inline-flex; }}
.pv-tip {{ display:none; position:absolute; bottom:calc(100% + 8px); left:50%; transform:translateX(-50%); background:#313033; color:#fff; font-size:11px; padding:4px 8px; border-radius:6px; white-space:nowrap; }}
.pv-tipwrap:hover .pv-tip {{ display:block; }}
.pv-tip::after {{ content:''; position:absolute; top:100%; left:50%; transform:translateX(-50%); border:5px solid transparent; border-top-color:#313033; }}
.pv-h1 {{ font-size:26px; font-weight:600; color:#1D1B20; }}
.pv-h2 {{ font-size:18px; font-weight:600; color:#1D1B20; }}
.pv-body {{ font-size:14px; color:#1D1B20; }}
.pv-label {{ font-size:12px; color:#49454F; }}
.pv-div {{ width:100%; border-top:1px solid #CAC4D0; }}
.pv-sidebar {{ display:flex; flex-direction:column; gap:4px; padding:10px; background:#F3EDF7; border-radius:10px 0 0 10px; width:120px; }}
.pv-panel {{ background:#fff; border:1px solid #CAC4D0; border-radius:10px; padding:12px; display:flex; flex-direction:column; gap:6px; }}
.pv-status {{ display:flex; justify-content:space-between; padding:6px 12px; background:#E6E0E9; border-radius:0 0 10px 10px; }}
.pv-app {{ display:flex; border:1px solid #CAC4D0; border-radius:10px; overflow:hidden; background:#FEF7FF; min-width:280px; }}
.pv-app-main {{ flex:1; display:flex; flex-direction:column; gap:8px; padding:10px; }}
.pv-progressbar {{ height:8px; background:#E6E0E9; position:relative; }}
/* Code */
pre {{
  background:var(--code-bg); color:var(--code-fg); border-radius:8px;
  padding:14px 16px; overflow-x:auto; font-size:13px; margin:12px 0; line-height:1.55;
}}
pre code {{ font-family:inherit; }}
.c-kw {{ color:var(--c-kw); }}
.c-fn {{ color:var(--c-fn); font-style:italic; }}
.c-type {{ color:var(--c-type); font-style:italic; }}
.c-str {{ color:var(--c-str); }}
.c-num {{ color:var(--c-num); }}
.c-const {{ color:var(--c-const); }}
.c-cmt {{ color:var(--c-cmt); font-style:italic; }}
.c-punc {{ color:var(--c-punc); }}
.c-life {{ color:var(--c-life); }}
.c-bool {{ color:var(--c-bool); }}
.c-var {{ color:var(--c-var); }}
.params {{ width:100%; border-collapse:collapse; font-size:13px; margin:12px 0; }}
.params th {{ text-align:left; font-size:11px; text-transform:uppercase; color:var(--muted); padding:6px 10px; border-bottom:1px solid var(--surface-2); }}
.params td {{ padding:8px 10px; border-bottom:1px solid var(--border); vertical-align:top; color:var(--subtext); }}
.params .pname {{ white-space:nowrap; }}
.params .ptype {{ white-space:nowrap; color:var(--teal); }}
.ret {{ font-size:13px; margin:10px 0; }}
.pill {{ display:inline-block; background:var(--surface-2); color:var(--primary); border-radius:999px; padding:1px 8px; margin-right:4px; font-size:12px; }}
.retdoc {{ color:var(--muted); font-size:12px; }}
.example {{ margin-top:14px; }}
.exhead {{ display:flex; justify-content:space-between; align-items:center; margin-bottom:6px; }}
.exhead span {{ font-size:11px; text-transform:uppercase; color:var(--muted); }}
.copy {{
  font-size:11px; padding:4px 12px; border:1px solid var(--surface-2); border-radius:999px;
  background:var(--bg-2); color:var(--primary); cursor:pointer; font-family:inherit;
}}
.copy:hover {{ background:var(--surface-2); }}
.muted {{ color:var(--muted); font-size:13px; }}
.empty {{ text-align:center; color:var(--muted); padding:40px; display:none; }}
/* Live demo */
.liveframe {{ background:var(--code-bg); border:1px solid var(--border); border-radius:12px; overflow:hidden; height:520px; margin-bottom:8px; }}
.liveframe iframe {{ width:100%; height:100%; border:0; display:block; }}
/* responsive */
@media (max-width:900px) {{
  .app {{ grid-template-columns:1fr; }}
  .sidebar {{ position:static; height:auto; }}
}}
</style>
</head>
<body>
<div class="app">
  <aside class="sidebar">
    <div class="logo"><span class="logo-badge">P</span><h1>Ply Components</h1></div>
    <p class="sub">{total} functions · {cats} groups</p>
    <input class="search" id="search" type="search" placeholder="Filter components…">
    <nav class="catnav" id="catnav"></nav>
  </aside>
  <main class="main">
    <div class="statbar">
      <span class="badge">{total} component functions</span>
      <span class="badge">{cats} categories</span>
    </div>
    {live}
    <div class="empty" id="empty">No components match.</div>
    <div id="sections">{sections}</div>
  </main>
</div>
<script>
function copyCode(btn) {{
  var pre = btn.closest('.example').querySelector('pre');
  navigator.clipboard.writeText(pre.innerText).then(function() {{
    var old = btn.textContent; btn.textContent = 'Copied!';
    setTimeout(function() {{ btn.textContent = old; }}, 1200);
  }});
}}
(function() {{
  var catnav = document.getElementById('catnav');
  var groups = Array.prototype.slice.call(document.querySelectorAll('.group'));
  groups.forEach(function(g, gi) {{
    var links = Array.prototype.slice.call(g.querySelectorAll('.subnav a'));
    if (links.length === 0) return;
    var h = document.createElement('h3'); h.textContent = g.getAttribute('data-cat');
    catnav.appendChild(h);
    links.forEach(function(a) {{
      var copy = a.cloneNode(true); copy.className = 'navlink-cat';
      copy.addEventListener('click', function(e) {{ e.preventDefault();
        var target = document.querySelector(a.getAttribute('href'));
        if (target) target.scrollIntoView({{ behavior:'smooth', block:'start' }});
      }});
      catnav.appendChild(copy);
    }});
  }});
  // Search
  var q = document.getElementById('search');
  var empty = document.getElementById('empty');
  q.addEventListener('input', function() {{
    var term = q.value.trim().toLowerCase();
    var any = false;
    document.querySelectorAll('.card').forEach(function(card) {{
      var hit = !term || card.getAttribute('data-name').toLowerCase().indexOf(term) !== -1
              || card.getAttribute('data-module').toLowerCase().indexOf(term) !== -1
              || card.innerText.toLowerCase().indexOf(term) !== -1;
      card.style.display = hit ? '' : 'none';
      if (hit) any = true;
    }});
    empty.style.display = any ? 'none' : 'block';
  }});
  // Intersection observer for active nav highlight
  var obs = new IntersectionObserver(function(entries) {{
    entries.forEach(function(en) {{
      if (en.isIntersecting) {{
        var id = en.target.getAttribute('id').replace('fn-','');
        document.querySelectorAll('.catnav a').forEach(function(a) {{
          a.classList.toggle('active', a.getAttribute('href') === '#fn-' + id);
        }});
      }}
    }});
  }}, {{ rootMargin:'-40% 0px -55% 0px' }});
  document.querySelectorAll('.card').forEach(function(c) {{ obs.observe(c); }});
}})();
// Interactive previews (plain DOM, no WASM)
document.addEventListener('input', function(e) {{
  var r = e.target;
  if (r.classList && r.classList.contains('pv-range')) {{
    var pct = (r.value - r.min) / (r.max - r.min) * 100;
    r.style.setProperty('--v', pct + '%');
    var s = r.closest('.pv-slider');
    if (s) {{
      var v = s.querySelector('.pv-val');
      if (v) v.textContent = r.value;
    }}
  }}
}});
document.addEventListener('click', function(e) {{
  var t = e.target;
  if (t.closest('.pv-radio-group')) {{
    var g = t.closest('.pv-radio-group');
    var r = t.closest('.pv-rowradio');
    if (r) {{
      g.querySelectorAll('.pv-radio').forEach(function(x) {{ x.classList.remove('on'); }});
      r.querySelector('.pv-radio').classList.add('on');
    }}
    return;
  }}
  if (t.closest('.pv-opt')) {{
    var c = t.closest('.pv-combo');
    c.querySelector('.pv-sel').textContent = t.closest('.pv-opt').textContent;
    c.classList.remove('open');
    e.stopPropagation();
    return;
  }}
  if (t.closest('.pv-combo')) {{
    t.closest('.pv-combo').classList.toggle('open');
    e.stopPropagation();
    return;
  }}
  if (t.closest('.pv-tab')) {{
    var tb = t.closest('.pv-tabs');
    tb.querySelectorAll('.pv-tab').forEach(function(x) {{ x.classList.remove('active'); }});
    t.closest('.pv-tab').classList.add('active');
    return;
  }}
  if (t.closest('.pv-selectable')) {{
    t.closest('.pv-selectable').classList.toggle('on');
    return;
  }}
  if (t.closest('.pv-item') && t.closest('.pv-item').parentElement.classList.contains('pv-list')) {{
    var list = t.closest('.pv-item').parentElement;
    list.querySelectorAll('.pv-item').forEach(function(x) {{ x.classList.remove('sel'); }});
    t.closest('.pv-item').classList.add('sel');
    return;
  }}
  if (t.closest('.pv-btn')) {{
    var b = t.closest('.pv-btn');
    b.classList.add('pressed');
    setTimeout(function() {{ b.classList.remove('pressed'); }}, 150);
    return;
  }}
  if (t.closest('.pv-check, .pv-switch, .pv-radio')) {{
    t.closest('.pv-check, .pv-switch, .pv-radio').classList.toggle('on');
    return;
  }}
  var row = t.closest('.pv-row');
  if (row) {{
    var ctl = row.querySelector('.pv-check, .pv-switch, .pv-radio');
    if (ctl) ctl.classList.toggle('on');
  }}
}});
</script>
</body>
</html>
"#,
        total = total,
        cats = cats,
        live = live,
        sections = sections,
    );
    html
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

fn print_usage() {
    println!(
        "cargo docx — generate FastAPI-style interactive HTML docs for your components

Usage:
  cargo docx [OPTIONS]

Scans <root>/src/components/*.rs for `pub fn` components and writes a
self-contained docs/components.html (sticky grouped nav, live search, copy
buttons, parameter tables, return types, curated examples).

Options:
  -d, --dir <PATH>   components directory (default: walk up from the current
                     dir to find the nearest src/components)
  -o, --out <PATH>   output HTML file (default: <root>/docs/components.html)
  -l, --live <PATH>  WASM web build directory (plyx web output). Its contents
                     are copied to <docs>/live/ and embedded as an iframe at
                     the top of the docs page. Serve over HTTP (WASM needs it):
                       python -m http.server 8000
  -h, --help         print this help
"
    );
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // cargo invokes external subcommands as `cargo-docx docx <args...>`, so
    // strip the leading subcommand name when present.
    if args.first().map(|a| a == "docx").unwrap_or(false) {
        args.remove(0);
    }
    let mut dir_arg: Option<PathBuf> = None;
    let mut out_arg: Option<PathBuf> = None;
    let mut live_arg: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--dir" => {
                i += 1;
                dir_arg = args.get(i).map(PathBuf::from);
            }
            "-o" | "--out" => {
                i += 1;
                out_arg = args.get(i).map(PathBuf::from);
            }
            "-l" | "--live" => {
                i += 1;
                live_arg = args.get(i).map(PathBuf::from);
            }
            "-h" | "--help" => {
                print_usage();
                return;
            }
            other => {
                eprintln!("cargo docx: unknown argument `{other}`");
                print_usage();
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let dir = match dir_arg {
        Some(d) => d,
        None => discover_components_dir(&std::env::current_dir().unwrap_or_default())
            .unwrap_or_else(|| {
                eprintln!(
                    "cargo docx: no src/components found under the current directory (or use --dir)"
                );
                std::process::exit(1);
            }),
    };
    if !dir.is_dir() {
        eprintln!("cargo docx: components dir not found: {}", dir.display());
        std::process::exit(1);
    }

    let mut components: Vec<Component> = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map_or(false, |e| e == "rs"))
        .filter(|p| p.file_name().and_then(|n| n.to_str()) != Some("mod.rs"))
        .collect();
    entries.sort();
    for e in entries {
        components.extend(extract(&e));
    }
    components.sort_by(|a, b| a.name.cmp(&b.name));

    let out = match out_arg {
        Some(o) => o,
        None => {
            let root = dir
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| dir.parent().unwrap_or(Path::new(".")));
            root.join("docs").join("components.html")
        }
    };
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).unwrap();
        if let Some(live) = &live_arg {
            let live_dst = parent.join("live");
            if live_dst.exists() {
                fs::remove_dir_all(&live_dst).unwrap();
            }
            copy_dir_all(live, &live_dst).unwrap();
        }
    }
    let live_url = live_arg
        .map(|_| "live/index.html".to_string())
        .or_else(|| {
            if out.parent().map_or(false, |p| p.join("live").join("index.html").is_file()) {
                Some("live/index.html".to_string())
            } else {
                None
            }
        });
    fs::write(&out, build_html(&components, live_url.as_deref())).unwrap();

    let documented = components
        .iter()
        .filter(|c| meta_for(&c.name).category != "Uncategorized")
        .count();
    println!(
        "Wrote {} — {} components documented, {} discovered ({} without metadata).",
        out.display(),
        documented,
        components.len(),
        components.len() - documented
    );
    let unmeta: Vec<&str> = components
        .iter()
        .filter(|c| meta_for(&c.name).category == "Uncategorized")
        .map(|c| c.name.as_str())
        .collect();
    if !unmeta.is_empty() {
        println!("Without metadata: {}", unmeta.join(", "));
    }
}

/// Walk up from `start` looking for the nearest `src/components` directory.
fn discover_components_dir(start: &Path) -> Option<PathBuf> {
    let mut d = start.to_path_buf();
    loop {
        let cand = d.join("src").join("components");
        if cand.is_dir() {
            return Some(cand);
        }
        if !d.pop() {
            return None;
        }
    }
}

/// Recursively copy a directory tree.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let e = entry?;
        let to = dst.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir_all(&e.path(), &to)?;
        } else {
            fs::copy(e.path(), to)?;
        }
    }
    Ok(())
}
