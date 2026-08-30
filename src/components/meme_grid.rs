//! MemeGrid — virtualized sticker/meme grid with async thumbnail loading and
//! on-hover GIF playback.
//!
//! Perf model follows commercial-framework consensus (docs/RESEARCH.md):
//! - Virtualized window: only visible rows (+ buffer) are built each frame
//!   (RecyclerView / Flutter ListView.builder).
//! - Decode-time downsampling: thumbnails decoded at `thumb_px`, never
//!   full-size uploads (Glide / Coil / Compose).
//! - Bounded LRU texture cache, evicting oldest entries (Coil memory cache).
//! - Directional prefetch via extra buffer rows (Glide RecyclerViewPreloader).
//! - Single active animation slot: only the hovered GIF plays (Telegram).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use indexmap::IndexSet;
use ply_engine::prelude::*;

/// Get current process private memory usage in MB (Windows).
#[cfg(target_os = "windows")]
fn process_mem_mb() -> f64 {
    #[repr(C)]
    struct PROCESS_MEMORY_COUNTERS { cb: u32, peak_working_set_size: usize,
        working_set_size: usize, quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize, quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize, pagefile_usage: usize,
        peak_pagefile_usage: usize, private_usage: usize }
    #[link(name = "psapi")]
    extern "system" { fn GetProcessMemoryInfo(
        h: *mut std::ffi::c_void, p: *mut PROCESS_MEMORY_COUNTERS, s: u32) -> i32; }
    unsafe {
        let mut counters: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let h = -1isize as *mut _; // GetCurrentProcess
        if GetProcessMemoryInfo(h, &mut counters, counters.cb) != 0 {
            return counters.private_usage as f64 / 1048576.0;
        }
    }
    0.0
}
#[cfg(not(target_os = "windows"))]
fn process_mem_mb() -> f64 { 0.0 }

/// Log memory breakdown every N seconds.
fn log_mem_budget(state: &MemeGridState) {
    static LAST_LOG: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    if now == LAST_LOG.swap(now, std::sync::atomic::Ordering::Relaxed) { return; }

    let total_mb = process_mem_mb();
    let tex_count = state.textures.borrow().map.len();
    let tex_est = tex_count * (state.thumb_px as usize) * (state.thumb_px as usize) * 4 / 1048576;
    let anim_frames = state.anim.as_ref().map(|(_, c, _)| c.frames.len()).unwrap_or(0);
    let anim_mb = state.anim.as_ref().map(|(_, c, _)| {
        c.frames.iter().map(|f| f.len()).sum::<usize>() as f64 / 1048576.0
    }).unwrap_or(0.0);
    let anim_cache_mb = state.anim_cache.bytes as f64 / 1048576.0;
    let item_paths_mb = state.items.iter().map(|i| i.id.len() + i.path.len()).sum::<usize>() as f64 / 1048576.0;

    eprintln!("[mem] {total_mb:.0}MB total | tex: {} items ~{tex_est}MB | anim: {} frames ~{anim_mb:.1}MB | anim_cache: ~{anim_cache_mb:.1}MB | item_paths: ~{item_paths_mb:.1}MB",
        tex_count, anim_frames);
}

use crate::components::config::{self, MemeGridConfig};
use crate::theme;

fn cfg() -> MemeGridConfig {
    config::effective(config::Style::current().meme_grid, MemeGridConfig::get(), MemeGridConfig::merged)
}

/// One meme entry. `id` must be unique per library (file stem by default).
#[derive(Clone, Debug)]
pub struct MemeItem {
    pub id: String,
    pub path: String,
    pub animated: bool,
}

impl MemeItem {
    pub fn from_path(path: impl Into<String>) -> Self {
        let path = path.into();
        // Detect actual animated format via magic bytes, not just extension
        let animated = Self::detect_animated(&path);
        // Full path as key: file stems can collide across folders.
        Self { id: path.clone(), path, animated }
    }

    fn detect_animated(path: &str) -> bool {
        use std::io::Read;
        let Ok(mut f) = std::fs::File::open(path) else { return false };
        let mut buf = [0u8; 4];
        if f.read_exact(&mut buf).is_err() { return false }
        // GIF magic: "GIF8" (GIF87a or GIF89a)
        buf == *b"GIF8"
    }
}

/// List supported image files under `dir` (recursive).
pub fn scan_dir(dir: &str) -> Vec<MemeItem> {
    const EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if let Some(s) = path.to_str() {
                out.push(s.to_string());
            }
        }
    }
    let mut paths = Vec::new();
    walk(std::path::Path::new(dir), &mut paths);
    let total_files = paths.len();
    let mut items: Vec<MemeItem> = paths
        .into_iter()
        .filter(|p| {
            let ext = p.rsplit('.').next().unwrap_or("").to_lowercase();
            EXTS.contains(&ext.as_str())
        })
        .map(MemeItem::from_path)
        .collect();
    let anim_count = items.iter().filter(|i| i.animated).count();
    eprintln!("[meme_grid] scanned {} files, {} supported ({} animated), total path memory ~{:.1}MB",
        total_files, items.len(), anim_count,
        items.iter().map(|i| i.id.len() + i.path.len()).sum::<usize>() as f64 / 1e6);
    items.sort_by(|a, b| a.id.cmp(&b.id));
    items
}

/// Bounded LRU over GPU textures. O(1) touch via IndexSet order tracking.
struct LruTex {
    map: HashMap<String, Texture2D>,
    order: IndexSet<String>,
}

impl LruTex {
    fn new() -> Self {
        Self { map: HashMap::new(), order: IndexSet::new() }
    }

    fn get(&mut self, key: &str) -> Option<Texture2D> {
        let hit = self.map.get(key).cloned()?;
        if let Some(idx) = self.order.get_index_of(key) {
            let last = self.order.len() - 1;
            if idx < last {
                self.order.move_index(idx, last);
            }
        }
        Some(hit)
    }

    fn put(&mut self, key: String, tex: Texture2D, cap: usize) {
        if self.map.contains_key(&key) {
            if let Some(idx) = self.order.get_index_of(&key) {
                let last = self.order.len() - 1;
                if idx < last {
                    self.order.move_index(idx, last);
                }
            }
            self.map.insert(key, tex);
            return;
        }
        self.order.insert(key.clone());
        self.map.insert(key, tex);
        while self.order.len() > cap.max(1) {
            if let Some(old) = self.order.shift_remove_index(0) {
                self.map.remove(&old);
            }
        }
        static LOG_THRESHOLD: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        let count = self.map.len();
        if count <= LOG_THRESHOLD.load(std::sync::atomic::Ordering::Relaxed) || count.is_power_of_two() {
            eprintln!("[meme_grid] tex_cache: {} items (cap {})", count, cap);
            let next = if count.is_power_of_two() { count * 2 } else { LOG_THRESHOLD.load(std::sync::atomic::Ordering::Relaxed) };
            LOG_THRESHOLD.store(next, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Decoded animated clip for the single active GIF slot.
struct AnimClip {
    frames: Vec<Vec<u8>>, // RGBA8
    delays: Vec<f32>,     // seconds per frame
    w: u16,
    h: u16,
    idx: usize,
    t: f32,
}

/// Bounded animation cache (decoded GIF frames). Evicts oldest entries when
/// total decoded byte size exceeds budget. Prevents unbounded memory growth
/// when many large GIFs are hovered in sequence.
struct AnimCache {
    map: HashMap<String, (Vec<Vec<u8>>, Vec<f32>, u16, u16)>,
    order: IndexSet<String>,
    bytes: usize,
    budget: usize,
}

impl AnimCache {
    fn new(budget: usize) -> Self {
        Self { map: HashMap::new(), order: IndexSet::new(), bytes: 0, budget }
    }

    fn get(&mut self, key: &str) -> Option<&(Vec<Vec<u8>>, Vec<f32>, u16, u16)> {
        let val = self.map.get(key)?;
        if let Some(idx) = self.order.get_index_of(key) {
            let last = self.order.len() - 1;
            if idx < last {
                self.order.move_index(idx, last);
            }
        }
        Some(val)
    }

    fn insert(&mut self, key: String, val: (Vec<Vec<u8>>, Vec<f32>, u16, u16)) {
        // Compute byte size: sum of all frame RGBA buffers.
        let frame_bytes: usize = val.0.iter().map(|f| f.len()).sum();
        if self.map.contains_key(&key) {
            if let Some(idx) = self.order.get_index_of(&key) {
                let last = self.order.len() - 1;
                if idx < last {
                    self.order.move_index(idx, last);
                }
            }
            self.map.insert(key, val);
            return;
        }
        self.order.insert(key.clone());
        self.map.insert(key, val);
        self.bytes += frame_bytes;
        // Evict oldest entries until under budget.
        while self.bytes > self.budget && self.order.len() > 1 {
            if let Some(old) = self.order.shift_remove_index(0) {
                if let Some((frames, _, _, _)) = self.map.remove(&old) {
                    let removed: usize = frames.iter().map(|f| f.len()).sum();
                    self.bytes = self.bytes.saturating_sub(removed);
                }
            }
        }
    }
}

/// Limit concurrent image decodes to cap peak memory from full-res decode buffers.
static DECODE_INFLIGHT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const MAX_DECODE_INFLIGHT: usize = 4;

/// Per-library UI state. Keep one instance per grid across frames.
pub struct MemeGridState {
    pub items: Vec<MemeItem>,
    textures: Rc<RefCell<LruTex>>,
    inflight: Rc<RefCell<HashSet<String>>>,
    failed: Rc<RefCell<HashSet<String>>>,
    cache_cap: usize,
    thumb_px: u32,
    /// The one playing animation: (item id, clip, gpu texture).
    anim: Option<(String, AnimClip, Texture2D)>,
    /// Bounded cache of decoded animations (id → frames+delays).
    anim_cache: AnimCache,
    anim_inflight: Option<String>,
    /// Hovered item index from the previous frame.
    prev_hover: Option<usize>,
}

impl MemeGridState {
    pub fn new(items: Vec<MemeItem>) -> Self {
        Self {
            items,
            textures: Rc::new(RefCell::new(LruTex::new())),
            inflight: Rc::new(RefCell::new(HashSet::new())),
            failed: Rc::new(RefCell::new(HashSet::new())),
            cache_cap: 256,
            thumb_px: 128,
            anim: None,
            anim_cache: AnimCache::new(64 * 1024 * 1024), // 64MB byte budget
            anim_inflight: None,
            prev_hover: None,
        }
    }

    fn ensure_static_tex(&mut self, index: usize) {
        let Some(item) = self.items.get(index) else { return };
        let (id, path) = (item.id.clone(), item.path.clone());
        if self.failed.borrow().contains(&id) {
            return;
        }
        if self.textures.borrow_mut().get(&id).is_some() {
            return;
        }
        if self.inflight.borrow().contains(&id) {
            return;
        }
        if DECODE_INFLIGHT.load(std::sync::atomic::Ordering::Relaxed) >= MAX_DECODE_INFLIGHT {
            return;
        }
        DECODE_INFLIGHT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inflight.borrow_mut().insert(id.clone());

        let thumb_px = self.thumb_px;
        let textures = Rc::clone(&self.textures);
        let inflight = Rc::clone(&self.inflight);
        let failed = Rc::clone(&self.failed);
        let id_clone = id.clone();
        let spawn_result = jobs::spawn(
            format!("meme:{id}"),
            move || async move {
                use image::ImageReader;
                let reader = ImageReader::open(&path).ok()?;
                let reader = reader.with_guessed_format().ok()?;
                let img = reader.decode().ok()?;
                let thumb = img.thumbnail(thumb_px, thumb_px);
                let rgba = thumb.to_rgba8();
                Some((rgba.width() as u16, rgba.height() as u16, rgba.into_raw()))
            },
            {
                move |decoded: Option<(u16, u16, Vec<u8>)>| {
                    inflight.borrow_mut().remove(&id);
                    DECODE_INFLIGHT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    if let Some((w, h, bytes)) = decoded {
                        let tex = Texture2D::from_rgba8(w, h, &bytes);
                        textures.borrow_mut().put(id.clone(), tex, 512);
                        static DECODED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                        let n = DECODED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        if n < 10 {
                            eprintln!("[meme_grid] texture #{n} uploaded {w}x{h}");
                        }
                    } else {
                        failed.borrow_mut().insert(id);
                    }
                }
            },
        );
        if let Err(_e) = spawn_result {
            self.inflight.borrow_mut().remove(&id_clone);
            DECODE_INFLIGHT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Spawn decode of every frame for the hovered GIF (single slot).
    /// `bg` is the cell background color used to composite transparent GIF frames.
    fn ensure_anim(&mut self, index: usize, bg: Color) {
        let Some(item) = self.items.get(index) else { return };
        if !item.animated || self.anim_inflight.as_deref() == Some(item.id.as_str()) {
            return;
        }
        if self.anim.as_ref().is_some_and(|(id, _, _)| *id == item.id) {
            return;
        }
        // Check animation cache first — avoid re-decoding GIFs we've seen before.
        if let Some((frames, delays, w, h)) = self.anim_cache.get(&item.id).cloned() {
            let tex = Texture2D::from_rgba8(w, h, &frames[0]);
            self.anim = Some((
                item.id.clone(),
                AnimClip { frames, delays, w, h, idx: 0, t: 0.0 },
                tex,
            ));
            return;
        }
        self.anim_inflight = Some(item.id.clone());
        eprintln!("[meme_grid] ensure_anim spawned for index={index} id={}", &item.id[item.id.len().saturating_sub(30)..]);

        let id = item.id.clone();
        let path = item.path.clone();
        let id_clone = id.clone();
        // Background color bytes for alpha compositing (premultiply)
        let bg_r = (bg.r * 255.0) as u8;
        let bg_g = (bg.g * 255.0) as u8;
        let bg_b = (bg.b * 255.0) as u8;
        if let Err(e) = jobs::spawn(
            format!("anim:{id}"),
            move || async move {
                use std::io::{BufReader, Seek};
                use image::AnimationDecoder;
                let file = std::fs::File::open(&path).ok()?;
                let mut file = BufReader::new(file);
                file.seek(std::io::SeekFrom::Start(0)).ok()?;
                // Use magic bytes to detect format, not file extension
                let mut header = [0u8; 4];
                std::io::Read::read_exact(&mut file, &mut header).ok()?;
                file.seek(std::io::SeekFrom::Start(0)).ok()?;
                let is_gif = header[0] == b'G' && header[1] == b'I' && header[2] == b'F' && header[3] == b'8';
                if !is_gif { return None; }
                let decoder = image::codecs::gif::GifDecoder::new(file).ok()?;
                let mut frames = Vec::new();
                let mut delays = Vec::new();
                let (mut w, mut h) = (0u16, 0u16);
                for frame in decoder.into_frames() {
                    let Ok(frame) = frame else { break };
                    w = frame.buffer().width() as u16;
                    h = frame.buffer().height() as u16;
                    delays.push((frame.delay().numer_denom_ms().0.max(20) as f32) / 1000.0);
                    // Alpha-composite RGBA frame onto solid background color.
                    // This prevents transparent GIF areas from rendering as black/white.
                    let rgba = frame.buffer();
                    let mut composited = Vec::with_capacity(rgba.len());
                    for chunk in rgba.chunks_exact(4) {
                        let a = chunk[3] as u16;
                        let inv = 255 - a;
                        let r = ((chunk[0] as u16 * a + bg_r as u16 * inv) / 255) as u8;
                        let g = ((chunk[1] as u16 * a + bg_g as u16 * inv) / 255) as u8;
                        let b = ((chunk[2] as u16 * a + bg_b as u16 * inv) / 255) as u8;
                        composited.extend_from_slice(&[r, g, b, 255]);
                    }
                    frames.push(composited);
                }
                if frames.is_empty() { None } else { Some((w, h, frames, delays)) }
            },
            {
                move |decoded: Option<(u16, u16, Vec<Vec<u8>>, Vec<f32>)>| {
                    // Cannot touch `self` here; route through the thread-local
                    // bridge consumed by `poll_anim` next frame.
                    ANIM_COMPLETED.with(|slot| {
                        *slot.borrow_mut() =
                            decoded.map(|(w, h, frames, delays)| (id.clone(), w, h, frames, delays));
                    });
                }
            },
        ) {
            self.anim_inflight = None;
            eprintln!("[meme_grid] anim spawn FAILED for {id_clone}: {e}");
        }
    }

    fn poll_anim(&mut self) {
        let ready = ANIM_COMPLETED.with(|slot| slot.borrow_mut().take());
        if let Some((id, w, h, frames, delays)) = ready {
            let frame_bytes: usize = frames.iter().map(|f| f.len()).sum();
            eprintln!("[meme_grid] poll_anim loaded {} frames {}x{} ({:.1}MB decoded, cache {:.1}MB/{:.0}MB budget)",
                frames.len(), w, h, frame_bytes as f64 / 1e6,
                self.anim_cache.bytes as f64 / 1e6, self.anim_cache.budget as f64 / 1e6);
            // Cache decoded frames so re-hovering this item skips decode.
            self.anim_cache.insert(id.clone(), (frames.clone(), delays.clone(), w, h));
            let tex = Texture2D::from_rgba8(w, h, &frames[0]);
            self.anim = Some((
                id,
                AnimClip { frames, delays, w, h, idx: 0, t: 0.0 },
                tex,
            ));
            self.anim_inflight = None;
        }
    }

    /// Advance the active animation by `dt`; returns its texture if still live.
    /// Only creates a new GPU texture when the frame index actually changes.
    fn tick_anim(&mut self, dt: f32) -> Option<(String, Texture2D)> {
        let (id, clip, tex) = self.anim.as_mut()?;
        let old_idx = clip.idx;
        clip.t += dt;
        while clip.t >= clip.delays[clip.idx] {
            clip.t -= clip.delays[clip.idx];
            clip.idx = (clip.idx + 1) % clip.frames.len();
        }
        if clip.idx != old_idx {
            let new_tex = Texture2D::from_rgba8(clip.w, clip.h, &clip.frames[clip.idx]);
            *tex = new_tex;
        }
        Some((id.clone(), tex.clone()))
    }
}

thread_local! {
    /// Bridge for job callbacks that cannot capture `&mut MemeGridState`.
    static ANIM_COMPLETED: RefCell<Option<(String, u16, u16, Vec<Vec<u8>>, Vec<f32>)>> =
        const { RefCell::new(None) };
}

/// Draw the virtualized grid. Returns the clicked item index this frame.
pub fn meme_grid(ui: &mut Ui<'_, ()>, state: &mut MemeGridState) -> Option<usize> {
    let c = cfg();
    let theme = theme::theme();
    let cell = c.cell_size.unwrap_or(96.0);
    let gap = c.gap.unwrap_or(8.0);
    let pad = c.padding.unwrap_or(12.0);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let buffer_rows = c.buffer_rows.unwrap_or(1);
    let bg = c.background.map(Color::from).unwrap_or(theme.colors.surface_container_low.into());
    let selected_border = c.selected_border.map(Color::from).unwrap_or(theme.colors.primary.into());
    let placeholder = c.placeholder.map(Color::from).unwrap_or(theme.colors.surface_container_high.into());

    state.cache_cap = c.cache_capacity.unwrap_or(256);
    state.thumb_px = c.thumb_px.unwrap_or(192);

    // --- animation lifecycle (uses last frame's hover) ---------------------
    state.poll_anim();
    let dt = get_frame_time();
    let active_anim = state.prev_hover
        .and_then(|idx| state.items.get(idx).cloned())
        .filter(|item| item.animated)
        .and_then(|item| {
            // Clear stale animation when hover target changes — prevents old
            // GIF's last frame from persisting as background on the new cell.
            if state.anim.as_ref().is_some_and(|(id, _, _)| *id != item.id) {
                state.anim = None;
            }
            state.ensure_anim(state.prev_hover.unwrap(), bg);
            let matches = state.anim.as_ref().is_some_and(|(id, _, _)| *id == item.id);
            if matches { state.tick_anim(dt) } else { None }
        });

    let grid_id = Id::new("meme_grid");
    // Scroll data persists across frames (engine keeps container registry).
    let scd = ui.scroll_container_data(grid_id.clone());
    let scroll_y = scd.as_ref().map(|d| (-d.scroll_position.y).max(0.0)).unwrap_or(0.0);
    let vp_w = scd.as_ref().map(|d| d.scroll_container_dimensions.width).unwrap_or(800.0);
    let vp_h = scd.as_ref().map(|d| d.scroll_container_dimensions.height).unwrap_or(600.0);

    // Log process memory breakdown once per second for profiling.
    log_mem_budget(state);

    let cols = (((vp_w - pad * 2.0 + gap) / (cell + gap)).floor() as usize).max(1);
    let row_h = cell + gap;
    let total_rows = state.items.len().div_ceil(cols);
    let first_row = ((scroll_y / row_h).floor() as usize).saturating_sub(buffer_rows);
    let vis_rows = (vp_h / row_h).ceil() as usize + buffer_rows * 2 + 1;
    let last_row = (first_row.saturating_add(vis_rows)).min(total_rows);

    let skip_px = first_row as f32 * row_h;
    let tail_px = (total_rows - last_row) as f32 * row_h;

    let mut selected: Option<usize> = None;
    let mut hovered: Option<usize> = None;

    ui.element()
        .id(grid_id)
        .width(grow!())
        .height(grow!())
        .background_color(bg)
        .corner_radius(radius)
        .overflow(|o| o.scroll_y())
        .layout(|l| l.direction(TopToBottom).padding((pad as u16, pad as u16, pad as u16, pad as u16)))
        .children(|ui| {
            // Virtualization spacers keep engine scroll extent correct while
            // only visible rows exist in the element tree.
            ui.element().width(grow!()).height(fixed!(skip_px.max(0.0))).empty();
            for row in first_row..last_row {
                ui.element()
                    .width(fit!())
                    .height(fixed!(cell))
                    .layout(|l| l.direction(LeftToRight).gap(gap as u16))
                    .children(|ui| {
                        let start = row * cols;
                        let end = (start + cols).min(state.items.len());
                        for i in start..end {
                            draw_cell(ui, state, i, cell, radius, bg, placeholder,
                                      selected_border, &active_anim, &mut hovered);
                            if ui.is_just_pressed(Id::from(("meme_grid", i as u32))) {
                                selected = Some(i);
                            }
                        }
                    });
            }
            ui.element().width(grow!()).height(fixed!(tail_px.max(0.0))).empty();
        });

    // Prefetch the row just below the window (Glide-style directional preload).
    if last_row < total_rows {
        let start = last_row * cols;
        let end = (start + cols).min(state.items.len());
        for i in start..end {
            state.ensure_static_tex(i);
        }
    }

    state.prev_hover = hovered;

    // Immediately clear stale animation when hover changes — prevents the
    // previous GIF's last frame from bleeding into the next cell for one frame.
    if let Some(h) = hovered {
        let new_id = state.items.get(h).map(|i| i.id.as_str());
        if state.anim.as_ref().is_some_and(|(id, _, _)| Some(id.as_str()) != new_id) {
            state.anim = None;
        }
    } else if state.anim.is_some() {
        state.anim = None;
    }

    selected
}

#[allow(clippy::too_many_arguments)]
fn draw_cell(
    ui: &mut Ui<'_, ()>,
    state: &mut MemeGridState,
    index: usize,
    cell: f32,
    radius: f32,
    bg: Color,
    placeholder: Color,
    selected_border: Color,
    active_anim: &Option<(String, Texture2D)>,
    hovered: &mut Option<usize>,
) {
    let Some(item) = state.items.get(index) else { return };
    let item_id = item.id.clone();
    let cid = Id::from(("meme_grid", index as u32));
    state.ensure_static_tex(index);

    let static_tex = state.textures.borrow_mut().get(&item_id);
    let anim_here = active_anim
        .as_ref()
        .filter(|(id, _)| *id == item_id)
        .map(|(_, t)| t.clone());
    let tex = anim_here.clone().or(static_tex);
    let is_animating = anim_here.is_some();

    let hit = Rc::new(RefCell::new(false));
    let hit_flag = Rc::clone(&hit);
    let mut el = ui.element()
        .id(cid)
        .width(fixed!(cell))
        .height(fixed!(cell))
        .background_color(bg)
        .corner_radius(radius)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(|_, _| {});
    if is_animating {
        el = el.border(|b| b.all(2).color(selected_border));
    }
    el.children(move |ui| {
        *hit_flag.borrow_mut() = ui.hovered();
        match tex {
            Some(tex) => {
                // Contain-fit: preserve aspect inside the square cell.
                let tw = tex.width().max(1.0);
                let th = tex.height().max(1.0);
                let scale = (cell / tw).min(cell / th);
                let iw = tw * scale;
                let ih = th * scale;
                ui.element()
                    .width(fixed!(iw))
                    .height(fixed!(ih))
                    .corner_radius(radius)
                    .image(tex)
                    .empty();
            }
            None => {
                // Placeholder while decoding.
                ui.element()
                    .width(fixed!(cell))
                    .height(fixed!(cell))
                    .background_color(placeholder)
                    .corner_radius(radius)
                    .empty();
            }
        }
    });
    if *hit.borrow() {
        *hovered = Some(index);
    }
}
