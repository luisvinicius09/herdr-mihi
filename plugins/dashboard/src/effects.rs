//! Native, dependency-free "screensaver" effects (no python, no subprocess) — an ambient field
//! painted behind the dashboard card, cycled live with `e`. Everything here writes directly into
//! ratatui's frame buffer using a tiny hand-rolled xorshift PRNG.
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use ratatui::Frame;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    None,
    Rain,
    Stars,
    Beam,
    Snow,
    Wave,
    Life,
    Fireworks,
}

impl Effect {
    pub fn all() -> [Effect; 8] {
        [
            Effect::None,
            Effect::Rain,
            Effect::Stars,
            Effect::Beam,
            Effect::Snow,
            Effect::Wave,
            Effect::Life,
            Effect::Fireworks,
        ]
    }
    pub fn name(self) -> &'static str {
        match self {
            Effect::None => "none",
            Effect::Rain => "rain",
            Effect::Stars => "stars",
            Effect::Beam => "beam",
            Effect::Snow => "snow",
            Effect::Wave => "wave",
            Effect::Life => "life",
            Effect::Fireworks => "fireworks",
        }
    }
    pub fn parse(s: &str) -> Effect {
        match s.trim().to_lowercase().as_str() {
            "rain" => Effect::Rain,
            "stars" => Effect::Stars,
            "beam" => Effect::Beam,
            "snow" => Effect::Snow,
            "wave" => Effect::Wave,
            "life" => Effect::Life,
            "fireworks" => Effect::Fireworks,
            _ => Effect::None,
        }
    }
    pub fn next(self) -> Effect {
        let a = Self::all();
        let i = a.iter().position(|&e| e == self).unwrap_or(0);
        a[(i + 1) % a.len()]
    }
    pub fn animated(self) -> bool {
        self != Effect::None
    }
}

const RAIN_CHARS: &[u8] = b"01<>[]{}/\\|=+*#$%&";
const RAIN_TRAIL: i32 = 7;

/// A firework particle: position, velocity, remaining life.
#[derive(Clone, Copy)]
struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: i16,
}

/// A snowflake: position + fall/drift velocity.
#[derive(Clone, Copy)]
struct Flake {
    x: f32,
    y: f32,
    vy: f32,
    vx: f32,
}

pub struct Field {
    pub effect: Effect,
    rng: u64,
    w: u16,
    h: u16,
    heads: Vec<i32>, // rain: current head row per column (may be negative = above screen)
    step: Vec<u16>,  // rain: advance the head every `step` frames
    stars: Vec<(u16, u16, u16)>, // (x, y, phase)
    flakes: Vec<Flake>,
    life: Vec<u8>, // Conway grid (0/1), row-major w*h
    sparks: Vec<Spark>,
    flash: i16, // event flash frames remaining
    flash_color: (u8, u8, u8),
    frame: u64,
    accent: (u8, u8, u8),
}

impl Field {
    pub fn new(effect: Effect, seed: u64, accent: (u8, u8, u8)) -> Field {
        Field {
            effect,
            rng: seed | 1,
            w: 0,
            h: 0,
            heads: Vec::new(),
            step: Vec::new(),
            stars: Vec::new(),
            flakes: Vec::new(),
            life: Vec::new(),
            sparks: Vec::new(),
            flash: 0,
            flash_color: (0, 0, 0),
            frame: 0,
            accent,
        }
    }

    /// Event overlay: a spark burst at a random upper position (fires over any ambient effect).
    pub fn burst(&mut self) {
        let cx = self.rand_range(self.w.max(1) as u32) as f32;
        let cy = self.rand_range((self.h / 3).max(1) as u32) as f32;
        self.spawn_burst(cx, cy);
    }

    /// Event overlay: a brief full-field tint (e.g. red when an agent blocks).
    pub fn flash(&mut self, color: (u8, u8, u8)) {
        self.flash = 8;
        self.flash_color = color;
    }

    /// True while event overlays (sparks/flash) still need frames — keeps the loop animating
    /// even when the ambient effect is `None`.
    pub fn busy(&self) -> bool {
        !self.sparks.is_empty() || self.flash > 0
    }

    fn spawn_burst(&mut self, cx: f32, cy: f32) {
        if self.sparks.len() >= 500 {
            return;
        }
        let count = 16 + self.rand_range(16) as usize;
        for k in 0..count {
            let ang = (k as f32 / count as f32) * std::f32::consts::TAU;
            let spd = 0.4 + self.rand_f() * 0.8;
            let life = 12 + self.rand_range(14) as i16;
            self.sparks.push(Spark {
                x: cx,
                y: cy,
                vx: ang.cos() * spd,
                vy: ang.sin() * spd * 0.6,
                life,
            });
        }
    }

    fn rand(&mut self) -> u64 {
        // xorshift64
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
    fn rand_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            (self.rand() % n as u64) as u32
        }
    }
    fn rand_f(&mut self) -> f32 {
        (self.rand() % 10_000) as f32 / 10_000.0
    }

    fn resize(&mut self, w: u16, h: u16) {
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        // rain
        self.heads = (0..w)
            .map(|_| -(self.rand_range(h.max(1) as u32) as i32))
            .collect();
        self.step = (0..w).map(|_| 2 + self.rand_range(6) as u16).collect();
        // stars: ~1 per 55 cells
        let stars = ((w as usize * h as usize) / 55).max(8);
        self.stars = (0..stars)
            .map(|_| {
                (
                    self.rand_range(w.max(1) as u32) as u16,
                    self.rand_range(h.max(1) as u32) as u16,
                    self.rand_range(12) as u16,
                )
            })
            .collect();
        // snow: ~1 per 45 cells
        let flakes = ((w as usize * h as usize) / 45).max(8);
        self.flakes = (0..flakes).map(|_| self.new_flake(true)).collect();
        // life: random seed (~28% alive)
        self.life = (0..(w as usize * h as usize))
            .map(|_| (self.rand_range(100) < 28) as u8)
            .collect();
        self.sparks.clear();
    }

    fn new_flake(&mut self, anywhere: bool) -> Flake {
        Flake {
            x: self.rand_range(self.w.max(1) as u32) as f32,
            y: if anywhere {
                self.rand_range(self.h.max(1) as u32) as f32
            } else {
                0.0
            },
            vy: 0.15 + self.rand_f() * 0.35,
            vx: (self.rand_f() - 0.5) * 0.3,
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        // Always age event overlays (sparks/flash) so bursts work over any ambient effect.
        self.age_sparks();
        if self.flash > 0 {
            self.flash -= 1;
        }
        match self.effect {
            Effect::Rain => self.tick_rain(),
            Effect::Snow => self.tick_snow(),
            Effect::Life => self.tick_life(),
            Effect::Fireworks => {
                if self.frame % 12 == 0 {
                    let cx = self.rand_range(self.w.max(1) as u32) as f32;
                    let cy = self.rand_range((self.h / 2).max(1) as u32) as f32;
                    self.spawn_burst(cx, cy);
                }
            }
            _ => {} // stars/beam/wave are driven by `frame`
        }
    }

    fn age_sparks(&mut self) {
        for s in self.sparks.iter_mut() {
            s.x += s.vx;
            s.y += s.vy;
            s.vy += 0.03; // gravity
            s.life -= 1;
        }
        self.sparks.retain(|s| s.life > 0);
    }

    fn tick_rain(&mut self) {
        let (frame, h) = (self.frame, self.h as i32);
        for i in 0..self.heads.len() {
            if self.step[i] == 0 || frame % self.step[i] as u64 == 0 {
                self.heads[i] += 1;
                if self.heads[i] - RAIN_TRAIL > h {
                    self.heads[i] = -(self.rand_range(self.h.max(1) as u32) as i32);
                    self.step[i] = 2 + self.rand_range(6) as u16;
                }
            }
        }
    }

    fn tick_snow(&mut self) {
        let (w, h) = (self.w as f32, self.h as f32);
        for i in 0..self.flakes.len() {
            let mut fk = self.flakes[i];
            fk.y += fk.vy;
            fk.x += fk.vx;
            if fk.x < 0.0 {
                fk.x += w;
            } else if fk.x >= w {
                fk.x -= w;
            }
            if fk.y >= h {
                fk = self.new_flake(false);
            }
            self.flakes[i] = fk;
        }
    }

    fn tick_life(&mut self) {
        let (w, h) = (self.w as usize, self.h as usize);
        if w == 0 || h == 0 || self.life.len() != w * h {
            return;
        }
        let mut next = vec![0u8; w * h];
        let mut alive = 0usize;
        for y in 0..h {
            for x in 0..w {
                let mut n = 0u8;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            n += self.life[ny as usize * w + nx as usize];
                        }
                    }
                }
                let cur = self.life[y * w + x];
                let live = (cur == 1 && (n == 2 || n == 3)) || (cur == 0 && n == 3);
                next[y * w + x] = live as u8;
                alive += live as usize;
            }
        }
        // reseed if the colony has (nearly) died out, so the field never goes blank.
        if alive < (w * h) / 200 {
            for c in next.iter_mut() {
                *c = (self.rand_range(100) < 28) as u8;
            }
        }
        self.life = next;
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        self.resize(area.width, area.height);
        match self.effect {
            Effect::None => {}
            Effect::Rain => self.render_rain(f, area),
            Effect::Stars => self.render_stars(f, area),
            Effect::Beam => self.render_beam(f, area),
            Effect::Snow => self.render_snow(f, area),
            Effect::Wave => self.render_wave(f, area),
            Effect::Life => self.render_life(f, area),
            Effect::Fireworks => {} // ambient fireworks are drawn by the shared spark overlay
        }
        // Event overlays, on top of any ambient effect.
        self.render_sparks(f, area);
        self.render_flash(f, area);
    }

    fn render_flash(&mut self, f: &mut Frame, area: Rect) {
        if self.flash <= 0 {
            return;
        }
        let intensity = 0.4 * (self.flash as f32 / 8.0);
        let bg = scale(self.flash_color, intensity);
        let buf = f.buffer_mut();
        for y in 0..area.height {
            for x in 0..area.width {
                if let Some(cell) = buf.cell_mut(Position::new(area.x + x, area.y + y)) {
                    cell.set_bg(bg);
                }
            }
        }
    }

    fn render_rain(&mut self, f: &mut Frame, area: Rect) {
        let buf = f.buffer_mut();
        for col in 0..self.w {
            let head = self.heads[col as usize];
            for t in 0..RAIN_TRAIL {
                let y = head - t;
                if y < 0 || y >= self.h as i32 {
                    continue;
                }
                let ch = RAIN_CHARS[(self.rand() as usize) % RAIN_CHARS.len()] as char;
                let color = if t == 0 {
                    Color::Rgb(0xd0, 0xe8, 0xff) // bright head
                } else {
                    scale(self.accent, 0.55 * (1.0 - t as f32 / RAIN_TRAIL as f32))
                };
                put(buf, area, col, y as u16, ch, color);
            }
        }
    }

    fn render_stars(&mut self, f: &mut Frame, area: Rect) {
        let (frame, accent) = (self.frame, self.accent);
        let buf = f.buffer_mut();
        for &(x, y, phase) in &self.stars {
            if x >= self.w || y >= self.h {
                continue;
            }
            let (ch, factor) = match ((frame / 2 + phase as u64) % 10) as u32 {
                0..=2 => ('.', 0.35),
                3..=5 => ('+', 0.6),
                6..=7 => ('*', 0.95),
                _ => continue, // off
            };
            put(buf, area, x, y, ch, scale(accent, factor));
        }
    }

    fn render_beam(&mut self, f: &mut Frame, area: Rect) {
        let w = self.w.max(1);
        let bx = (self.frame % w as u64) as u16;
        let accent = self.accent;
        let buf = f.buffer_mut();
        for y in 0..self.h {
            for (dx, factor, ch) in [(-1i32, 0.25, '│'), (0, 0.9, '█'), (1, 0.25, '│')] {
                let x = bx as i32 + dx;
                if x >= 0 && x < w as i32 {
                    put(buf, area, x as u16, y, ch, scale(accent, factor));
                }
            }
        }
    }

    fn render_snow(&mut self, f: &mut Frame, area: Rect) {
        let (w, h) = (self.w, self.h);
        let buf = f.buffer_mut();
        for fk in &self.flakes {
            let (x, y) = (fk.x as u16, fk.y as u16);
            if x >= w || y >= h {
                continue;
            }
            let ch = if fk.vy > 0.35 { '*' } else { '.' };
            put(buf, area, x, y, ch, Color::Rgb(0xcf, 0xdd, 0xf5));
        }
    }

    fn render_wave(&mut self, f: &mut Frame, area: Rect) {
        let (w, h, frame, accent) = (self.w, self.h, self.frame as f32, self.accent);
        let buf = f.buffer_mut();
        for y in 0..h {
            for x in 0..w {
                let v = (x as f32 * 0.25 + y as f32 * 0.16 + frame * 0.12).sin();
                if v > 0.72 {
                    put(buf, area, x, y, '~', scale(accent, 0.9));
                } else if v > 0.45 {
                    put(buf, area, x, y, '-', scale(accent, 0.4));
                }
            }
        }
    }

    fn render_life(&mut self, f: &mut Frame, area: Rect) {
        let (w, h) = (self.w as usize, self.h as usize);
        if self.life.len() != w * h {
            return;
        }
        let accent = self.accent;
        let buf = f.buffer_mut();
        for y in 0..h {
            for x in 0..w {
                if self.life[y * w + x] == 1 {
                    put(buf, area, x as u16, y as u16, '●', scale(accent, 0.7));
                }
            }
        }
    }

    fn render_sparks(&mut self, f: &mut Frame, area: Rect) {
        let (w, h) = (self.w, self.h);
        let buf = f.buffer_mut();
        for s in &self.sparks {
            let (x, y) = (s.x as i32, s.y as i32);
            if x < 0 || x >= w as i32 || y < 0 || y >= h as i32 {
                continue;
            }
            let ch = if s.life > 18 {
                '*'
            } else if s.life > 8 {
                '+'
            } else {
                '.'
            };
            let factor = (s.life as f32 / 26.0).clamp(0.2, 1.0);
            let color = Color::Rgb(
                (0xff as f32 * factor) as u8,
                (0xc0 as f32 * factor) as u8,
                (0x40 as f32 * factor) as u8,
            );
            put(buf, area, x as u16, y as u16, ch, color);
        }
    }
}

/// Set one cell's char + fg within `area` (bounds-checked).
fn put(buf: &mut ratatui::buffer::Buffer, area: Rect, x: u16, y: u16, ch: char, color: Color) {
    if x >= area.width || y >= area.height {
        return;
    }
    if let Some(cell) = buf.cell_mut(Position::new(area.x + x, area.y + y)) {
        cell.set_char(ch);
        cell.set_fg(color);
    }
}

fn scale(c: (u8, u8, u8), f: f32) -> Color {
    let f = f.clamp(0.0, 1.0);
    Color::Rgb(
        (c.0 as f32 * f) as u8,
        (c.1 as f32 * f) as u8,
        (c.2 as f32 * f) as u8,
    )
}
