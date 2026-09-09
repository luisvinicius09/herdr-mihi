//! herdr-mihi.picker — browse & install the collection, showing the exact commands first.
//! `open` (action) opens the pane; `pane` runs the ratatui TUI. Installs ONLY from the pinned
//! repo via `herdr plugin install … --ref <name>-latest`; no arbitrary sources, no AI, no telemetry.
mod catalog;

use std::collections::HashMap;
use std::io::{stdout, Write};
use std::process::Command;

use catalog::{Action, State};
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use tui_big_text::{BigText, PixelSize};

const OWNER_FALLBACK: &str = "luisvinicius09/herdr-mihi";

// True-color palette (herdr blue + friends).
const ACCENT: Color = Color::Rgb(0x07, 0x97, 0xff);
const GREEN: Color = Color::Rgb(0x46, 0xd3, 0x7f);
const YELLOW: Color = Color::Rgb(0xf3, 0xc0, 0x18);
const REDISH: Color = Color::Rgb(0xf4, 0x6b, 0x6b);
const DIM: Color = Color::Rgb(0x8a, 0x8a, 0x8a);
const SELBG: Color = Color::Rgb(0x10, 0x2a, 0x3c);

struct Icons {
    available: &'static str,
    installed: &'static str,
    update: &'static str,
}
fn icons(mode: &str) -> Icons {
    match mode {
        "unicode" => Icons {
            available: "○",
            installed: "●",
            update: "⬆",
        },
        "ascii" => Icons {
            available: "o",
            installed: "*",
            update: "^",
        },
        // nerd (default): Font Awesome glyphs present in any Nerd Font
        _ => Icons {
            available: "\u{f019}",
            installed: "\u{f00c}",
            update: "\u{f0aa}",
        },
    }
}

struct Row {
    p: catalog::CatalogPlugin,
    state: State,
    selected: bool,
}

enum Screen {
    Browse,
    Confirm,
}

struct App {
    owner: String,
    rows: Vec<Row>,
    list: ListState,
    screen: Screen,
    icons: Icons,
}

impl App {
    fn move_cursor(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let n = self.rows.len() as isize;
        let cur = self.list.selected().unwrap_or(0) as isize;
        self.list.select(Some((cur + delta).rem_euclid(n) as usize));
    }
    fn toggle(&mut self) {
        if let Some(r) = self.list.selected().and_then(|i| self.rows.get_mut(i)) {
            r.selected = !r.selected;
        }
    }
    fn toggle_all(&mut self) {
        let all = self.rows.iter().all(|r| r.selected);
        for r in &mut self.rows {
            r.selected = !all;
        }
    }
    fn any_selected(&self) -> bool {
        self.rows.iter().any(|r| r.selected)
    }
    fn selected_count(&self) -> usize {
        self.rows.iter().filter(|r| r.selected).count()
    }
}

fn herdr_bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

/// Config from `$HERDR_PLUGIN_CONFIG_DIR/.env` (env vars override).
fn load_cfg() -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(dir) = std::env::var("HERDR_PLUGIN_CONFIG_DIR") {
        if let Ok(c) = std::fs::read_to_string(format!("{dir}/.env")) {
            for line in c.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = l.split_once('=') {
                    m.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    m
}
fn cfg_get(m: &HashMap<String, String>, key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .or_else(|| m.get(key).cloned())
        .unwrap_or_else(|| default.to_string())
}

fn pause() {
    print!("press enter to close... ");
    let _ = stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}

/// `open` action: open the picker popup pane.
fn cmd_open() {
    let cfg = load_cfg();
    let w = cfg_get(&cfg, "POPUP_WIDTH", "90%");
    let h = cfg_get(&cfg, "POPUP_HEIGHT", "85%");
    let _ = Command::new(herdr_bin())
        .args([
            "plugin",
            "pane",
            "open",
            "--plugin",
            "herdr-mihi.picker",
            "--entrypoint",
            "picker",
            "--placement",
            "popup",
            "--width",
            &w,
            "--height",
            &h,
            "--focus",
        ])
        .status();
}

fn load_rows() -> Result<(String, Vec<Row>), String> {
    let root = std::env::var("HERDR_PLUGIN_ROOT").unwrap_or_else(|_| ".".into());
    let cat = catalog::load(&root)?;
    let owner = if cat.owner.is_empty() {
        OWNER_FALLBACK.to_string()
    } else {
        cat.owner.clone()
    };
    let out = Command::new(herdr_bin())
        .args(["plugin", "list", "--json"])
        .output()
        .map_err(|e| format!("`herdr plugin list` failed: {e}"))?;
    let installed = catalog::parse_installed(&String::from_utf8_lossy(&out.stdout));
    let rows = cat
        .plugins
        .into_iter()
        .filter(|p| p.name != "picker") // don't list ourselves
        .map(|p| {
            let state = catalog::state_for(&p.version, installed.get(&p.id).map(String::as_str));
            Row {
                p,
                state,
                selected: false,
            }
        })
        .collect();
    Ok((owner, rows))
}

fn cmd_pane() {
    let cfg = load_cfg();
    let (owner, rows) = match load_rows() {
        Ok(x) => x,
        Err(e) => {
            eprintln!("herdr-mihi.picker: {e}");
            pause();
            return;
        }
    };
    let mut app = App {
        owner,
        rows,
        list: ListState::default(),
        screen: Screen::Browse,
        icons: icons(&cfg_get(&cfg, "ICONS", "nerd")),
    };
    if !app.rows.is_empty() {
        app.list.select(Some(0));
    }

    let mut terminal = ratatui::init();
    let confirmed = run_app(&mut terminal, &mut app);
    ratatui::restore();

    if confirmed {
        run_selected(&app);
    }
}

/// Event loop. Returns true if the user confirmed (run the selection).
fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> bool {
    loop {
        let _ = terminal.draw(|f| ui(f, app));
        let code = match event::read() {
            Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => k.code,
            Ok(_) => continue,
            Err(_) => return false,
        };
        match app.screen {
            Screen::Browse => match code {
                KeyCode::Char('q') | KeyCode::Esc => return false,
                KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1),
                KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1),
                KeyCode::Char(' ') => app.toggle(),
                KeyCode::Char('a') => app.toggle_all(),
                KeyCode::Enter => {
                    if app.any_selected() {
                        app.screen = Screen::Confirm;
                    }
                }
                _ => {}
            },
            Screen::Confirm => match code {
                KeyCode::Char('y') => return true,
                KeyCode::Char('n') | KeyCode::Esc => app.screen = Screen::Browse,
                KeyCode::Char('q') => return false,
                _ => {}
            },
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Browse => ui_browse(f, app),
        Screen::Confirm => ui_confirm(f, app),
    }
}

fn ui_browse(f: &mut Frame, app: &App) {
    let [banner, body, legend, footer] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(f.area());

    let big = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .lines(vec!["herdr-mihi".into()])
        .style(Style::new().fg(ACCENT).bold())
        .build();
    f.render_widget(big, banner);

    let items: Vec<ListItem> = if app.rows.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "  everything's installed — nothing to pick",
            Style::new().fg(DIM),
        )))]
    } else {
        app.rows
            .iter()
            .map(|r| {
                let check = if r.selected {
                    Span::styled("[✓] ", Style::new().fg(GREEN).bold())
                } else {
                    Span::styled("[ ] ", Style::new().fg(DIM))
                };
                let (icon, istyle, label, ver) = match &r.state {
                    State::Available => (
                        app.icons.available,
                        Style::new().fg(DIM),
                        "available",
                        format!("v{}", r.p.version),
                    ),
                    State::Installed(v) => (
                        app.icons.installed,
                        Style::new().fg(GREEN),
                        "installed",
                        format!("v{v}"),
                    ),
                    State::Update { installed, latest } => (
                        app.icons.update,
                        Style::new().fg(YELLOW),
                        "update",
                        format!("v{installed}→v{latest}"),
                    ),
                };
                ListItem::new(Line::from(vec![
                    check,
                    Span::styled(format!("{icon} "), istyle),
                    Span::styled(
                        format!("{:<12}", r.p.name),
                        Style::new().fg(Color::White).bold(),
                    ),
                    Span::styled(format!("{:<10}", label), istyle),
                    Span::styled(format!("{:<14}", ver), Style::new().fg(DIM)),
                    Span::styled(r.p.description.clone(), Style::new().fg(DIM)),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(DIM))
                .title(Span::styled(" plugins ", Style::new().fg(ACCENT).bold())),
        )
        .highlight_symbol("➤ ")
        .highlight_style(
            Style::new()
                .bg(SELBG)
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    let mut st = app.list.clone();
    f.render_stateful_widget(list, body, &mut st);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(app.icons.installed, Style::new().fg(GREEN)),
            Span::styled(" installed   ", Style::new().fg(DIM)),
            Span::styled(app.icons.update, Style::new().fg(YELLOW)),
            Span::styled(" update   ", Style::new().fg(DIM)),
            Span::styled(app.icons.available, Style::new().fg(DIM)),
            Span::styled(
                " available   ·   space acts on the row's state",
                Style::new().fg(DIM),
            ),
        ])),
        legend,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {} selected ", app.selected_count()),
                Style::new().fg(Color::Black).bg(ACCENT).bold(),
            ),
            Span::styled(
                "  ↑/↓ move · space select · a all · enter review · q quit    ",
                Style::new().fg(DIM),
            ),
            Span::styled(format!("pinned to {}", app.owner), Style::new().fg(DIM)),
        ])),
        footer,
    );
}

fn ui_confirm(f: &mut Frame, app: &App) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "These are the ONLY commands that will run:",
            Style::new().fg(Color::White).bold(),
        )),
        Line::raw(""),
    ];
    for r in app.rows.iter().filter(|r| r.selected) {
        let action = catalog::action_for(&r.state);
        let args = catalog::command_args(&app.owner, &r.p, &action);
        let color = match action {
            Action::Install => ACCENT,
            Action::Uninstall => REDISH,
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("herdr {}", args.join(" ")), Style::new().fg(color)),
        ]));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        format!(
            "source: {} (pinned) · no network beyond it · nothing runs until you confirm",
            app.owner
        ),
        Style::new().fg(DIM),
    )));
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled(" y ", Style::new().fg(Color::Black).bg(GREEN).bold()),
        Span::styled(" run these    ", Style::new().fg(DIM)),
        Span::styled(" n ", Style::new().fg(Color::Black).bg(DIM).bold()),
        Span::styled(" back    ", Style::new().fg(DIM)),
        Span::styled(" q ", Style::new().fg(Color::Black).bg(DIM).bold()),
        Span::styled(" cancel", Style::new().fg(DIM)),
    ]));

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .border_style(Style::new().fg(YELLOW))
                    .title(Span::styled(" confirm ", Style::new().fg(YELLOW).bold())),
            )
            .wrap(Wrap { trim: false }),
        f.area(),
    );
}

/// After leaving the TUI, run each selected action as plain terminal output.
fn run_selected(app: &App) {
    let herdr = herdr_bin();
    println!("Running {} action(s)...\n", app.selected_count());
    for r in app.rows.iter().filter(|r| r.selected) {
        let action = catalog::action_for(&r.state);
        let verb = match action {
            Action::Install => "install/update",
            Action::Uninstall => "uninstall",
        };
        println!("> {verb}: {}", r.p.name);
        let status = Command::new(&herdr)
            .args(catalog::command_args(&app.owner, &r.p, &action))
            .status();
        match status {
            Ok(s) if s.success() => println!("  ok: {}\n", r.p.name),
            Ok(s) => println!("  FAILED: {} (exit {:?})\n", r.p.name, s.code()),
            Err(e) => println!("  FAILED: {} ({e})\n", r.p.name),
        }
    }
    print!("Done. Press enter to close... ");
    let _ = stdout().flush();
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("open") => cmd_open(),
        Some("pane") => cmd_pane(),
        _ => {
            eprintln!("usage: run.sh {{open|pane}}");
            std::process::exit(2);
        }
    }
}
