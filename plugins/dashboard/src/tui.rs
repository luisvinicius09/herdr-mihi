//! The on-demand popup as an ambient "agent screen": a centered dashboard card (needs-you hero,
//! workspace rollups, per-agent durations + project·branch + tokens) floating in a native effect
//! field, with filter/sort/jump navigation. Read-only until you jump; the only mutation is a jump.
use crate::config::{Config, Icons, Theme};
use crate::effects::{Effect, Field};
use crate::model::{self, Group, Row, Status};
use crate::shared::{sanitize, socket::Client};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn rgb(t: (u8, u8, u8)) -> Color {
    Color::Rgb(t.0, t.1, t.2)
}

/// Display (column) width of a string — for aligning the box borders.
fn dw(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(s)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Attention,
    Recent,
    Name,
}
impl SortMode {
    fn name(self) -> &'static str {
        match self {
            SortMode::Attention => "attention",
            SortMode::Recent => "recent",
            SortMode::Name => "name",
        }
    }
    fn next(self) -> SortMode {
        match self {
            SortMode::Attention => SortMode::Recent,
            SortMode::Recent => SortMode::Name,
            SortMode::Name => SortMode::Attention,
        }
    }
}

/// One agent tile's rendered data (the roster is a flat list of these — each tile self-labels
/// its workspace, so there are no separate header rows).
struct Entry {
    pane_id: String,
    agent: String,
    activity: String,
    status: Status,
    since_ms: u64,
    project: String,
    workspace: String,
    git: GitInfo,
    tokens: Vec<(String, String)>,
}

struct App {
    groups: Vec<Group>,
    entries: Vec<Entry>,
    list: ListState,
    icons: Icons,
    theme: Theme,
    idle_dim_s: u64,
    effect: Effect,
    field: Field,
    state_dir: String,
    since: HashMap<String, (Status, u64)>, // pane -> (status, since_ms)
    git: HashMap<String, GitInfo>,         // cwd -> git info (cached)
    filter: String,
    filter_mode: bool,
    attention_only: bool,
    sort: SortMode,
    start: Instant,
    last_input: Instant,
    first_load: bool,
}

pub fn run() {
    let cfg = Config::load();
    let sock = std::env::var("HERDR_SOCKET_PATH").unwrap_or_default();
    let state_dir = std::env::var("HERDR_PLUGIN_STATE_DIR")
        .unwrap_or_else(|_| "/tmp/herdr-mihi-dashboard".to_string());
    let effect = load_effect(&state_dir, &cfg);

    let groups = match load_groups(&sock, &cfg) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("herdr-mihi.dashboard: {e}");
            pause();
            return;
        }
    };

    let seed = (std::process::id() as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) | 1;
    let field = Field::new(effect, seed, cfg.theme.accent);
    let mut app = App::new(&cfg, groups, effect, field, state_dir);

    let mut terminal = ratatui::init();
    let jump = run_app(&mut terminal, &mut app, &sock, &cfg);
    ratatui::restore();

    if let Some(pane_id) = jump {
        if !sock.is_empty() {
            let _ = Client::new(sock).agent_focus(&pane_id);
        }
    }
}

fn load_groups(sock: &str, cfg: &Config) -> Result<Vec<Group>, String> {
    if sock.is_empty() {
        return Err("HERDR_SOCKET_PATH not set".into());
    }
    let raw = Client::new(sock.to_string())
        .snapshot_raw()
        .map_err(|e| format!("session.snapshot failed: {e}"))?;
    let snap = model::snapshot_from_raw(&raw).ok_or("could not parse session.snapshot")?;
    Ok(model::build_view(&snap, cfg.max_rows))
}

fn load_effect(state_dir: &str, cfg: &Config) -> Effect {
    if let Ok(s) = std::fs::read_to_string(format!("{state_dir}/effect")) {
        return Effect::parse(s.trim());
    }
    Effect::parse(&cfg.effect)
}
fn save_effect(state_dir: &str, e: Effect) {
    let _ = std::fs::create_dir_all(state_dir);
    let _ = std::fs::write(format!("{state_dir}/effect"), e.name());
}

/// Seed durations from the watcher's `agents.tsv` so "blocked 4m" reflects history before you opened.
fn seed_since(state_dir: &str) -> HashMap<String, (Status, u64)> {
    let mut m = HashMap::new();
    if let Ok(c) = std::fs::read_to_string(format!("{state_dir}/agents.tsv")) {
        for line in c.lines() {
            let mut it = line.split('\t');
            if let (Some(p), Some(s), Some(ms)) = (it.next(), it.next(), it.next()) {
                if let Ok(ms) = ms.trim().parse::<u64>() {
                    m.insert(p.to_string(), (Status::parse(Some(s)), ms));
                }
            }
        }
    }
    m
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

impl App {
    fn new(
        cfg: &Config,
        groups: Vec<Group>,
        effect: Effect,
        field: Field,
        state_dir: String,
    ) -> App {
        let since = seed_since(&state_dir);
        let mut app = App {
            groups: Vec::new(),
            entries: Vec::new(),
            list: ListState::default(),
            icons: cfg.icons,
            theme: cfg.theme,
            idle_dim_s: cfg.idle_dim_s,
            effect,
            field,
            state_dir,
            since,
            git: HashMap::new(),
            filter: String::new(),
            filter_mode: false,
            attention_only: false,
            sort: SortMode::Attention,
            start: Instant::now(),
            last_input: Instant::now(),
            first_load: true,
        };
        app.ingest(groups);
        app.list.select(app.first_agent());
        app
    }

    /// Absorb a fresh snapshot: update durations, fire event effects on transitions, rebuild view.
    fn ingest(&mut self, groups: Vec<Group>) {
        let now = now_ms();
        let mut new_since = HashMap::new();
        let mut done_events = 0usize;
        let mut blocked_event = false;
        for g in &groups {
            for r in &g.rows {
                let prev = self.since.get(&r.pane_id).copied();
                let changed = prev.map(|(s, _)| s != r.status).unwrap_or(true);
                let since_ms = match prev {
                    Some((s, ms)) if s == r.status => ms,
                    _ => now,
                };
                new_since.insert(r.pane_id.clone(), (r.status, since_ms));
                if !self.first_load && prev.is_some() && changed {
                    match r.status {
                        Status::Done => done_events += 1,
                        Status::Blocked => blocked_event = true,
                        _ => {}
                    }
                }
            }
        }
        self.since = new_since;
        // persist per-pane since-times so each agent's timer survives close/reopen (independently),
        // even when the background watcher isn't running.
        self.persist_since();
        self.groups = groups;
        for _ in 0..done_events.min(3) {
            self.field.burst();
        }
        if blocked_event {
            self.field.flash((0xff, 0x33, 0x33));
        }
        self.first_load = false;
        self.rebuild();
    }

    /// Write `pane\tstatus\tsince_ms` to STATE_DIR/agents.tsv (shared with the watcher).
    fn persist_since(&self) {
        let mut s = String::new();
        for (pane, (st, ms)) in &self.since {
            s.push_str(pane);
            s.push('\t');
            s.push_str(st.label());
            s.push('\t');
            s.push_str(&ms.to_string());
            s.push('\n');
        }
        let _ = std::fs::create_dir_all(&self.state_dir);
        let _ = std::fs::write(format!("{}/agents.tsv", self.state_dir), s);
    }

    /// Rebuild `entries` from `groups` applying filter/attention/sort; cache git branches.
    fn rebuild(&mut self) {
        // ensure git info cached (mutates only self.git; no other self borrow held)
        let cwds: Vec<String> = self
            .groups
            .iter()
            .flat_map(|g| g.rows.iter().map(|r| r.cwd.clone()))
            .collect();
        for c in cwds {
            if !c.is_empty() && !self.git.contains_key(&c) {
                let gi = read_git(&c);
                self.git.insert(c, gi);
            }
        }

        let now = now_ms();
        let mut entries = Vec::new();
        for g in &self.groups {
            let mut rows: Vec<&Row> = g
                .rows
                .iter()
                .filter(|r| row_matches(r, &self.filter, self.attention_only, &g.workspace))
                .collect();
            sort_rows(&mut rows, self.sort);
            for r in rows {
                let since_ms = self.since.get(&r.pane_id).map(|(_, ms)| *ms).unwrap_or(now);
                entries.push(Entry {
                    pane_id: r.pane_id.clone(),
                    agent: r.agent.clone(),
                    activity: r.activity.clone(),
                    status: r.status,
                    since_ms,
                    project: basename(&r.cwd),
                    workspace: g.workspace.clone(),
                    git: self.git.get(&r.cwd).cloned().unwrap_or_default(),
                    tokens: r.tokens.clone(),
                });
            }
        }
        self.entries = entries;
        let n = self.entries.len();
        if self.list.selected().map(|i| i >= n).unwrap_or(true) {
            self.list.select(self.first_agent());
        }
    }

    fn cycle_effect(&mut self) {
        self.effect = self.effect.next();
        self.field.effect = self.effect;
        save_effect(&self.state_dir, self.effect);
    }

    fn first_agent(&self) -> Option<usize> {
        if self.entries.is_empty() {
            None
        } else {
            Some(0)
        }
    }
    fn step(&mut self, forward: bool) {
        let n = self.entries.len();
        if n == 0 {
            return;
        }
        let cur = self.list.selected().unwrap_or(0);
        let i = if forward {
            (cur + 1) % n
        } else {
            (cur + n - 1) % n
        };
        self.list.select(Some(i));
    }
    /// Jump selection to the next agent that needs attention (blocked/done).
    fn step_attention(&mut self) {
        let n = self.entries.len();
        if n == 0 {
            return;
        }
        let cur = self.list.selected().unwrap_or(0);
        for k in 1..=n {
            let i = (cur + k) % n;
            if matches!(self.entries[i].status, Status::Blocked | Status::Done) {
                self.list.select(Some(i));
                return;
            }
        }
    }
    fn selected_pane(&self) -> Option<String> {
        self.entries
            .get(self.list.selected()?)
            .map(|e| e.pane_id.clone())
    }
    fn select_pane(&mut self, pane_id: &str) {
        for (i, e) in self.entries.iter().enumerate() {
            if e.pane_id == pane_id {
                self.list.select(Some(i));
                return;
            }
        }
    }
    fn totals(&self) -> (usize, usize, usize, usize) {
        model::counts(&self.groups)
    }
}

fn reload(app: &mut App, sock: &str, cfg: &Config) {
    let keep = app.selected_pane();
    if let Ok(groups) = load_groups(sock, cfg) {
        app.ingest(groups);
        if let Some(p) = keep {
            app.select_pane(&p);
        }
    }
}

fn row_matches(r: &Row, filter: &str, attention_only: bool, workspace: &str) -> bool {
    if attention_only && !matches!(r.status, Status::Blocked | Status::Done) {
        return false;
    }
    if filter.is_empty() {
        return true;
    }
    let f = filter.to_lowercase();
    let hay = format!(
        "{} {} {} {}",
        r.agent.to_lowercase(),
        r.activity.to_lowercase(),
        basename(&r.cwd).to_lowercase(),
        workspace.to_lowercase()
    );
    hay.contains(&f)
}

fn sort_rows(rows: &mut [&Row], mode: SortMode) {
    match mode {
        SortMode::Attention => rows.sort_by(|a, b| {
            a.status
                .rank()
                .cmp(&b.status.rank())
                .then(b.seq.cmp(&a.seq))
                .then(a.agent.cmp(&b.agent))
        }),
        SortMode::Recent => rows.sort_by(|a, b| b.seq.cmp(&a.seq).then(a.agent.cmp(&b.agent))),
        SortMode::Name => rows.sort_by(|a, b| a.agent.to_lowercase().cmp(&b.agent.to_lowercase())),
    }
}

fn basename(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("")
        .to_string()
}

fn fmt_dur(ms: u64) -> String {
    let s = ms / 1000;
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else if s < 86400 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

/// Git info read entirely from `.git` files — **no `git` subprocess**. Dirty-tree status is
/// intentionally omitted (it would require shelling out to `git`, against the trust posture).
#[derive(Clone, Default)]
struct GitInfo {
    branch: String,       // branch name, or short sha when detached
    detached: bool,       // HEAD is a raw sha (not on a branch)
    state: String,        // "rebasing" / "merging" / "cherry-pick" / … / "" if none
    stash: usize,         // stash entries
    synced: Option<bool>, // Some(true)=matches upstream, Some(false)=diverged, None=unknown
}

fn resolve_gitdir(cwd: &str) -> Option<PathBuf> {
    if cwd.is_empty() {
        return None;
    }
    let dotgit = Path::new(cwd).join(".git");
    if dotgit.is_dir() {
        return Some(dotgit);
    }
    if dotgit.is_file() {
        // worktree/submodule: ".git" is a file "gitdir: <path>"
        let s = std::fs::read_to_string(&dotgit).ok()?;
        let p = s.trim().strip_prefix("gitdir:").map(str::trim)?;
        return Some(if Path::new(p).is_absolute() {
            PathBuf::from(p)
        } else {
            Path::new(cwd).join(p)
        });
    }
    None
}

fn read_git(cwd: &str) -> GitInfo {
    let mut gi = GitInfo::default();
    let Some(gd) = resolve_gitdir(cwd) else {
        return gi;
    };
    if let Ok(head) = std::fs::read_to_string(gd.join("HEAD")) {
        let head = head.trim();
        if let Some(b) = head.strip_prefix("ref: refs/heads/") {
            gi.branch = b.to_string();
        } else if head.len() >= 7 {
            gi.branch = head[..7].to_string();
            gi.detached = true;
        }
    }
    gi.state = git_state(&gd);
    if let Ok(s) = std::fs::read_to_string(gd.join("logs/refs/stash")) {
        gi.stash = s.lines().filter(|l| !l.trim().is_empty()).count();
    }
    if !gi.detached && !gi.branch.is_empty() {
        gi.synced = git_sync(&gd, &gi.branch);
    }
    gi
}

fn git_state(gd: &Path) -> String {
    for (marker, name) in [
        ("rebase-merge", "rebasing"),
        ("rebase-apply", "rebasing"),
        ("MERGE_HEAD", "merging"),
        ("CHERRY_PICK_HEAD", "cherry-pick"),
        ("REVERT_HEAD", "reverting"),
        ("BISECT_LOG", "bisecting"),
    ] {
        if gd.join(marker).exists() {
            return name.to_string();
        }
    }
    String::new()
}

/// Resolve a ref's sha from a loose ref file, else `packed-refs`.
fn ref_sha(gd: &Path, refname: &str) -> Option<String> {
    if let Ok(s) = std::fs::read_to_string(gd.join(refname)) {
        let t = s.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    let packed = std::fs::read_to_string(gd.join("packed-refs")).ok()?;
    for line in packed.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        if let Some((sha, name)) = line.split_once(' ') {
            if name.trim() == refname {
                return Some(sha.trim().to_string());
            }
        }
    }
    None
}

/// Best-effort "is this branch level with its upstream?" via a sha compare (no commit walking).
fn git_sync(gd: &Path, branch: &str) -> Option<bool> {
    let cfg = std::fs::read_to_string(gd.join("config")).ok()?;
    let (mut remote, mut merge) = (String::new(), String::new());
    let header = format!("[branch \"{branch}\"]");
    let mut in_branch = false;
    for line in cfg.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_branch = l == header;
            continue;
        }
        if in_branch {
            if let Some((k, v)) = l.split_once('=') {
                match k.trim() {
                    "remote" => remote = v.trim().to_string(),
                    "merge" => merge = v.trim().to_string(),
                    _ => {}
                }
            }
        }
    }
    if remote.is_empty() || merge.is_empty() {
        return None;
    }
    let up = merge.strip_prefix("refs/heads/").unwrap_or(&merge);
    let local = ref_sha(gd, &format!("refs/heads/{branch}"))?;
    let upstream = ref_sha(gd, &format!("refs/remotes/{remote}/{up}"))?;
    Some(local == upstream)
}

fn run_app(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    sock: &str,
    cfg: &Config,
) -> Option<String> {
    let mut last_refresh = Instant::now();
    loop {
        let _ = terminal.draw(|f| ui(f, app));

        let animated = app.effect.animated() || app.field_busy();
        let wait = if animated {
            Duration::from_millis(cfg.frame_ms.max(16))
        } else if cfg.refresh_ms > 0 {
            Duration::from_millis(cfg.refresh_ms)
        } else {
            Duration::from_secs(3600)
        };

        if event::poll(wait).unwrap_or(false) {
            if let Ok(Event::Key(k)) = event::read() {
                if k.kind != KeyEventKind::Press {
                    continue;
                }
                app.last_input = Instant::now();
                if app.filter_mode {
                    match k.code {
                        KeyCode::Esc | KeyCode::Enter => app.filter_mode = false,
                        KeyCode::Backspace => {
                            app.filter.pop();
                            app.rebuild();
                        }
                        KeyCode::Char(c) => {
                            app.filter.push(c);
                            app.rebuild();
                        }
                        _ => {}
                    }
                    continue;
                }
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return None,
                    KeyCode::Char('j') | KeyCode::Down => app.step(true),
                    KeyCode::Char('k') | KeyCode::Up => app.step(false),
                    KeyCode::Tab => app.step_attention(),
                    KeyCode::Char('e') => app.cycle_effect(),
                    KeyCode::Char('a') => {
                        app.attention_only = !app.attention_only;
                        app.rebuild();
                    }
                    KeyCode::Char('s') => {
                        app.sort = app.sort.next();
                        app.rebuild();
                    }
                    KeyCode::Char('/') => app.filter_mode = true,
                    KeyCode::Char('r') => {
                        reload(app, sock, cfg);
                        last_refresh = Instant::now();
                    }
                    KeyCode::Enter => {
                        if let Some(p) = app.selected_pane() {
                            return Some(p);
                        }
                    }
                    _ => {}
                }
            }
        } else {
            if animated {
                app.field.tick();
            }
            if cfg.refresh_ms > 0 && last_refresh.elapsed() >= Duration::from_millis(cfg.refresh_ms)
            {
                reload(app, sock, cfg);
                last_refresh = Instant::now();
            }
        }
    }
}

impl App {
    fn field_busy(&self) -> bool {
        // keep ticking briefly so event bursts/flashes finish even when the ambient effect is None
        self.field.busy()
    }
}

fn status_color(theme: &Theme, s: Status) -> Color {
    match s {
        Status::Blocked => rgb(theme.blocked),
        Status::Done => rgb(theme.done),
        Status::Working => rgb(theme.working),
        Status::Idle | Status::Unknown => rgb(theme.idle),
    }
}

fn glyph(icons: &Icons, s: Status) -> &'static str {
    match s {
        Status::Blocked => icons.blocked,
        Status::Done => icons.done,
        Status::Working => icons.working,
        Status::Idle => icons.idle,
        Status::Unknown => icons.unknown,
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let theme = app.theme;
    let (cr, cg, cb) = theme.canvas;
    let canvas = Color::Rgb(cr, cg, cb);
    let on_canvas = Style::default().bg(canvas);
    let dimmed = app.idle_dim_s > 0 && app.last_input.elapsed().as_secs() >= app.idle_dim_s;
    let accent = if dimmed {
        rgb(theme.dim)
    } else {
        rgb(theme.accent)
    };
    let dim = rgb(theme.dim);
    let c_blocked = rgb(theme.blocked);
    let c_done = rgb(theme.done);
    let c_working = rgb(theme.working);
    let area = f.area();

    f.render_widget(Block::default().style(on_canvas), area);
    app.field.render(f, area);

    let cw = ((area.width as u32 * 7 / 10) as u16)
        .clamp(44, 120)
        .min(area.width);
    let ch = ((area.height as u32 * 8 / 10) as u16)
        .clamp(12, 44)
        .min(area.height);
    let card = Rect::new(
        area.x + area.width.saturating_sub(cw) / 2,
        area.y + area.height.saturating_sub(ch) / 2,
        cw,
        ch,
    );
    f.render_widget(Block::default().style(on_canvas), card);

    let rows = Layout::vertical([
        Constraint::Length(1), // title strip
        Constraint::Length(1), // hero + counts
        Constraint::Length(1), // spacer
        Constraint::Min(0),    // roster
        Constraint::Length(1), // stats
        Constraint::Length(1), // footer keys
    ])
    .split(card);

    // title strip
    f.render_widget(
        Paragraph::new(" herdr · agents ").style(
            Style::default()
                .bg(accent)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        rows[0],
    );

    // hero + counts
    let (b, d, w, i) = app.totals();
    let need = b + d;
    let hero = if need > 0 {
        Span::styled(
            format!(" ▲ {need} need you "),
            Style::default()
                .fg(if b > 0 { c_blocked } else { c_done })
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" ✓ all clear ", Style::default().fg(c_done))
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            hero,
            Span::styled(format!("   {b} blocked"), Style::default().fg(c_blocked)),
            Span::styled(format!("  {d} done"), Style::default().fg(c_done)),
            Span::styled(format!("  {w} working"), Style::default().fg(c_working)),
            Span::styled(format!("  {i} idle"), Style::default().fg(dim)),
        ]))
        .style(on_canvas),
        rows[1],
    );
    f.render_widget(Paragraph::new("").style(on_canvas), rows[2]);

    // roster panel (the middle border)
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            " agents ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ))
        .style(on_canvas);
    if app.entries.is_empty() {
        let msg = if app.filter.is_empty() {
            "no agents".to_string()
        } else {
            format!("no matches for \"{}\"", app.filter)
        };
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().bg(canvas).fg(dim))
                .block(block),
            rows[3],
        );
    } else {
        let now = now_ms();
        let width = rows[3].width.saturating_sub(2); // inside the panel border
                                                     // subtle lift for the selected tile (keeps accent borders visible)
        let sel = Color::Rgb(
            cr.saturating_add(22),
            cg.saturating_add(26),
            cb.saturating_add(34),
        );
        let items: Vec<ListItem> = app
            .entries
            .iter()
            .map(|e| row_item(e, &app.icons, now, &theme, width))
            .collect();
        let list = List::new(items)
            .block(block)
            .style(on_canvas)
            .highlight_style(Style::default().bg(sel).add_modifier(Modifier::BOLD));
        f.render_stateful_widget(list, rows[3], &mut app.list);
    }

    // stats line
    let up = fmt_dur(app.start.elapsed().as_millis() as u64);
    let total: usize = app.groups.iter().map(|g| g.rows.len()).sum();
    f.render_widget(
        Paragraph::new(format!(
            " {total} agents · {} workspaces · up {up}{}",
            app.groups.len(),
            if dimmed { " · idle" } else { "" }
        ))
        .style(Style::default().bg(canvas).fg(dim)),
        rows[4],
    );

    // footer — keys on the left, current sort on the right
    if app.filter_mode {
        f.render_widget(
            Paragraph::new(format!(
                " / filter: {}▌   (enter/esc to apply) ",
                app.filter
            ))
            .style(Style::default().bg(canvas).fg(dim)),
            rows[5],
        );
    } else {
        let left = format!(
            " ↑↓ move · ⏎ jump · ⇥ next · / filter{} · a attn:{} · s sort · e {} · r · q ",
            if app.filter.is_empty() {
                String::new()
            } else {
                format!(":{}", app.filter)
            },
            if app.attention_only { "on" } else { "off" },
            app.effect.name(),
        );
        let right = format!("sort:{} ", app.sort.name());
        let pad = (rows[5].width as usize).saturating_sub(dw(&left) + dw(&right));
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(left, Style::default().fg(dim)),
                Span::raw(" ".repeat(pad)),
                Span::styled(
                    right,
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
            ]))
            .style(on_canvas),
            rows[5],
        );
    }
}

/// Render one agent as a bordered tile sized to `width` columns.
fn row_item<'a>(e: &'a Entry, icons: &Icons, now: u64, theme: &Theme, width: u16) -> ListItem<'a> {
    let w = (width as usize).max(24);
    let color = status_color(theme, e.status);
    let attn = matches!(e.status, Status::Blocked | Status::Done);
    let idle = matches!(e.status, Status::Idle | Status::Unknown);
    let accent = rgb(theme.accent);
    let dim = rgb(theme.dim);
    let bs = Style::default().fg(if attn { color } else { accent }); // tile border
    let dur = fmt_dur(now.saturating_sub(e.since_ms));

    // line 1 — top border with a colored title
    let g = format!("{} ", glyph(icons, e.status));
    let sd = format!(" · {} {dur} ", e.status.label().to_uppercase());
    let name_style = if idle {
        Style::default().fg(dim)
    } else {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    };
    let titlew = 2 + dw(&g) + dw(&e.agent) + dw(&sd);
    let fill1 = w.saturating_sub(titlew + 1);
    let l1 = Line::from(vec![
        Span::styled("╭ ", bs),
        Span::styled(g, Style::default().fg(color)),
        Span::styled(e.agent.clone(), name_style),
        Span::styled(sd, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled("─".repeat(fill1), bs),
        Span::styled("╮", bs),
    ]);

    // line 2 — activity
    let act = sanitize::truncate_cols(&sanitize::clean(&e.activity), w.saturating_sub(4));
    let inner2 = format!(" {act}");
    let pad2 = w.saturating_sub(1 + dw(&inner2) + 1);
    let l2 = Line::from(vec![
        Span::styled("│", bs),
        Span::styled(
            inner2,
            Style::default().fg(if idle { dim } else { Color::Gray }),
        ),
        Span::raw(" ".repeat(pad2)),
        Span::styled("│", bs),
    ]);

    // line 3 — project ⎇branch (·sync) · git-state · stash · tokens … pane id
    let mut m: Vec<Span> = vec![Span::styled("│ ", bs)];
    let mut mw = 2usize;
    if !e.project.is_empty() {
        mw += dw(&e.project);
        m.push(Span::styled(e.project.clone(), Style::default().fg(accent)));
    }
    if !e.git.branch.is_empty() {
        let s = format!(
            " {}{}",
            if e.git.detached { "@" } else { "⎇" },
            e.git.branch
        );
        mw += dw(&s);
        m.push(Span::styled(s, Style::default().fg(accent)));
        match e.git.synced {
            Some(true) => {
                let s = " ✓".to_string();
                mw += dw(&s);
                m.push(Span::styled(s, Style::default().fg(rgb(theme.done))));
            }
            Some(false) => {
                let s = " ↕".to_string();
                mw += dw(&s);
                m.push(Span::styled(s, Style::default().fg(rgb(theme.working))));
            }
            None => {}
        }
    }
    if !e.git.state.is_empty() {
        let s = format!("  ⚠{}", e.git.state);
        mw += dw(&s);
        m.push(Span::styled(
            s,
            Style::default()
                .fg(rgb(theme.working))
                .add_modifier(Modifier::BOLD),
        ));
    }
    if e.git.stash > 0 {
        let s = format!("  ⌥{}", e.git.stash);
        mw += dw(&s);
        m.push(Span::styled(s, Style::default().fg(dim)));
    }
    for (k, v) in &e.tokens {
        let s = format!("  {k}:{v}");
        mw += dw(&s);
        m.push(Span::styled(s, Style::default().fg(dim)));
    }
    let pane = format!(" {} ", e.pane_id);
    let pad3 = w.saturating_sub(mw + dw(&pane) + 1);
    m.push(Span::raw(" ".repeat(pad3)));
    m.push(Span::styled(pane, Style::default().fg(dim)));
    m.push(Span::styled("│", bs));
    let l3 = Line::from(m);

    // line 4 — bottom border with the workspace footer, right-aligned
    let ws = if e.workspace.is_empty() {
        String::new()
    } else {
        format!(" {} ", e.workspace)
    };
    let fill4 = w.saturating_sub(1 + dw(&ws) + 2);
    let l4 = Line::from(vec![
        Span::styled(format!("╰{}", "─".repeat(fill4)), bs),
        Span::styled(ws, Style::default().fg(dim)),
        Span::styled("─╯", bs),
    ]);

    ListItem::new(vec![l1, l2, l3, l4])
}

fn pause() {
    print!("press enter to close... ");
    let _ = stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}
