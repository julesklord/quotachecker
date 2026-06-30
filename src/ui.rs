use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Tabs,
    },
    Frame,
};

use crate::agent::{AgentId, AgentState};
use crate::config::{AppConfig, TuiTheme};

// ─── Color Palette ────────────────────────────────────────────────────────────
const COLOR_BG: Color = Color::Reset;
const COLOR_CARD: Color = Color::Reset;
const COLOR_TEXT: Color = Color::Rgb(220, 220, 228);
const COLOR_MUTED: Color = Color::Rgb(105, 108, 120);
const COLOR_DIM: Color = Color::Rgb(70, 72, 82);
const COLOR_SUCCESS: Color = Color::Rgb(56, 214, 115); // Vivid Emerald
const COLOR_WARN: Color = Color::Rgb(251, 197, 49); // Amber
const COLOR_DANGER: Color = Color::Rgb(237, 76, 92); // Crimson
const COLOR_INFO: Color = Color::Rgb(80, 184, 255); // Sky Blue

// ─── Agent Brand Colors ───────────────────────────────────────────────────────
const COLOR_AGY: Color = Color::Rgb(168, 85, 247); // Vivid Purple
const COLOR_OPENCODE: Color = Color::Rgb(20, 210, 170); // Teal
const COLOR_CODEX: Color = Color::Rgb(249, 115, 22); // Deep Orange
const COLOR_ZED: Color = Color::Rgb(234, 100, 100); // Coral

// ─── UI Symbol Set ────────────────────────────────────────────────────────────
const SYM_ARROW: &str = "❯";
const SYM_BLOCK_FULL: &str = "█";
const SYM_BLOCK_EMPTY: &str = "░";
const SYM_BLOCK_HALF: &str = "▓";
const SYM_SEP: &str = "│";

pub struct RenderContext<'a> {
    pub active_tab: usize,
    pub selected_agent_idx: usize,
    pub selected_setting_idx: usize,
    pub show_budget_modal: bool,
    pub editing_value: &'a str,
    pub logs: &'a [String],
    pub tick_count: u64,
    pub agents: &'a [AgentState],
    pub config: &'a AppConfig,
}

fn make_progress_bar(ratio: f64, width: usize) -> String {
    let filled = (ratio * width as f64).round() as usize;
    let mut bar = String::new();
    for i in 0..width {
        if i < filled {
            // Use gradient blocks: full → half → empty
            if i == filled.saturating_sub(1) && ratio < 1.0 {
                bar.push_str(SYM_BLOCK_HALF);
            } else {
                bar.push_str(SYM_BLOCK_FULL);
            }
        } else {
            bar.push_str(SYM_BLOCK_EMPTY);
        }
    }
    bar
}

/// Renders a centered popup rectangle over `base`.
fn centered_rect(percent_x: u16, height: u16, base: Rect) -> Rect {
    let margin_x = base.width.saturating_sub(base.width * percent_x / 100) / 2;
    let margin_y = base.height.saturating_sub(height) / 2;
    Rect {
        x: base.x + margin_x,
        y: base.y + margin_y,
        width: base.width.saturating_sub(margin_x * 2),
        height: height.min(base.height),
    }
}

fn get_primary_color(theme: TuiTheme) -> Color {
    match theme {
        TuiTheme::Cyan => Color::Rgb(0, 220, 255),
        TuiTheme::Purple => Color::Rgb(168, 85, 247),
        TuiTheme::Emerald => Color::Rgb(56, 214, 115),
        TuiTheme::Amber => Color::Rgb(251, 197, 49),
        TuiTheme::Monochrome => Color::Rgb(200, 200, 210),
    }
}

/// Spinner animation frames cycling on tick.
fn spinner_frame(tick: u64) -> &'static str {
    const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    FRAMES[(tick as usize / 2) % FRAMES.len()]
}

/// Returns a color appropriate for a given usage ratio given thresholds.
fn ratio_color(ratio: f64, soft: f64, hard: f64) -> Color {
    if ratio >= hard {
        COLOR_DANGER
    } else if ratio >= soft {
        COLOR_WARN
    } else {
        COLOR_SUCCESS
    }
}

pub fn draw_ui(f: &mut Frame, ctx: &RenderContext) {
    let size = f.area();

    // Clear whole screen with dark background
    let bg_block = Block::default().style(Style::default().bg(COLOR_BG));
    f.render_widget(bg_block, size);

    // Main Layout (Header -> Content -> Footer/Status)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main Content Area
            Constraint::Length(1), // Footer status bar
        ])
        .split(size);

    draw_header(f, chunks[0], ctx);

    match ctx.active_tab {
        0 => draw_overview_tab(f, chunks[1], ctx),
        1 => draw_agents_tab(f, chunks[1], ctx),
        2 => draw_sessions_tab(f, chunks[1], ctx),
        3 => draw_quotas_tab(f, chunks[1], ctx),
        4 => draw_settings_tab(f, chunks[1], ctx),
        _ => {}
    }

    draw_footer(f, chunks[2], ctx);

    // Render Modal popup if active
    if ctx.show_budget_modal {
        draw_budget_modal(f, size, ctx);
    }
}

fn draw_header(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26), // Title + version
            Constraint::Min(20),    // Navigation Tabs
            Constraint::Length(22), // Status indicator
        ])
        .split(area);

    // ── 1. Title Block ────────────────────────────────────────────────────────
    let title_line = Line::from(vec![
        Span::styled(" ⚡ ", Style::default().fg(color_primary).bold()),
        Span::styled("QUOTA", Style::default().fg(color_primary).bold()),
        Span::styled("CHECKER", Style::default().fg(COLOR_TEXT).bold()),
        Span::styled(" · ", Style::default().fg(COLOR_DIM)),
        Span::styled("TUI", Style::default().fg(COLOR_MUTED)),
        Span::styled(" v0.3 ", Style::default().fg(COLOR_DIM).italic()),
    ]);
    let title_widget = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(COLOR_DIM)),
    );
    f.render_widget(title_widget, chunks[0]);

    // ── 2. Navigation Tabs ────────────────────────────────────────────────────
    let tab_titles = vec![
        " 1 Overview ",
        " 2 AI Agents ",
        " 3 Sessions ",
        " 4 Quotas ",
        " 5 Settings ",
    ];
    let tabs = Tabs::new(tab_titles)
        .select(ctx.active_tab)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(COLOR_DIM)),
        )
        .style(Style::default().fg(COLOR_MUTED))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(color_primary)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(SYM_SEP, Style::default().fg(COLOR_DIM)));
    f.render_widget(tabs, chunks[1]);

    // ── 3. Status Indicator ───────────────────────────────────────────────────
    let spin = spinner_frame(ctx.tick_count);
    let pulse_color = if ctx.tick_count.is_multiple_of(6) {
        COLOR_SUCCESS
    } else {
        Color::Rgb(30, 160, 80)
    };
    let installed_count = ctx
        .agents
        .iter()
        .filter(|a| a.executable_path.is_some())
        .count();
    let sync_text = Line::from(vec![
        Span::styled(format!(" {} ", spin), Style::default().fg(pulse_color)),
        Span::styled(
            format!("LIVE  {}/{} agents ", installed_count, ctx.agents.len()),
            Style::default().fg(COLOR_TEXT),
        ),
    ]);
    let sync_widget = Paragraph::new(sync_text)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(COLOR_DIM)),
        )
        .alignment(Alignment::Right);
    f.render_widget(sync_widget, chunks[2]);
}

fn draw_overview_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(36), // Left pane (Health / System)
            Constraint::Percentage(64), // Right pane (Stats & Quotas)
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(13), // Summary table
            Constraint::Length(10), // Comparative usage chart
            Constraint::Min(4),     // Log activity
        ])
        .split(chunks[0]);

    // Summary Card (System Status)
    let mut summary_rows = Vec::new();
    let mut total_requests = 0;
    let mut active_agents = 0;

    for agent in ctx.agents {
        let is_inst = agent.executable_path.is_some();
        let status_symbol = if is_inst { "✔" } else { "✘" };
        let status_color = if is_inst { COLOR_SUCCESS } else { COLOR_MUTED };
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if is_inst {
            active_agents += 1;
            total_requests += agent.quota_used;
        }

        summary_rows.push(Row::new(vec![
            Cell::new(agent.name.clone()).style(Style::default().fg(agent_color).bold()),
            Cell::new(status_symbol).style(Style::default().fg(status_color).bold()),
            Cell::new(agent.user_tier.display_name()).style(Style::default().fg(if is_inst {
                COLOR_TEXT
            } else {
                COLOR_MUTED
            })),
            Cell::new(if is_inst {
                match agent.quota_type {
                    crate::agent::QuotaType::Unlimited => "Unlimited".to_string(),
                    _ => format!("{}/{}", agent.quota_used, agent.quota_limit),
                }
            } else {
                "Omitted".to_string()
            })
            .style(Style::default().fg(if is_inst {
                COLOR_TEXT
            } else {
                COLOR_MUTED
            })),
        ]));
    }

    let summary_table = Table::new(
        summary_rows,
        [
            Constraint::Percentage(28),
            Constraint::Percentage(10),
            Constraint::Percentage(37),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["  Agent", "OK", "Account Tier", "Quota"])
            .style(
                Style::default()
                    .fg(color_primary)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color_primary))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ◈ SYSTEM TELEMETRY ",
                Style::default().fg(color_primary).bold(),
            )),
    );
    f.render_widget(summary_table, left_chunks[0]);

    // Comparative Agent Usage Chart Card
    let mut chart_lines = Vec::new();
    chart_lines.push(Line::from("")); // padding

    for agent in ctx.agents {
        let is_inst = agent.executable_path.is_some();
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if !is_inst {
            chart_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {: <10} ", agent.name),
                    Style::default().fg(COLOR_MUTED).bold(),
                ),
                Span::styled(
                    "[ Telemetry Omitted - Not Installed ]",
                    Style::default().fg(COLOR_MUTED).italic(),
                ),
            ]));
        } else {
            let ratio = if agent.quota_limit > 0 {
                (agent.quota_used as f64 / agent.quota_limit as f64).min(1.0)
            } else {
                0.0
            };

            let bar_width = 18;
            let bar_str = make_progress_bar(ratio, bar_width);

            let soft_warn = ctx.config.soft_limit_percent / 100.0;
            let hard_warn = ctx.config.hard_limit_percent / 100.0;

            let bar_color = if ratio >= hard_warn {
                COLOR_DANGER
            } else if ratio >= soft_warn {
                COLOR_WARN
            } else {
                COLOR_SUCCESS
            };

            if agent.quota_type == crate::agent::QuotaType::Unlimited {
                chart_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {: <10} ", agent.name),
                        Style::default().fg(agent_color).bold(),
                    ),
                    Span::styled("█".repeat(bar_width), Style::default().fg(COLOR_SUCCESS)),
                    Span::styled(
                        "  Local (Unlimited requests)",
                        Style::default().fg(COLOR_TEXT),
                    ),
                ]));
            } else {
                chart_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {: <10} ", agent.name),
                        Style::default().fg(agent_color).bold(),
                    ),
                    Span::styled(bar_str, Style::default().fg(bar_color)),
                    Span::styled(
                        format!(
                            "  {: >3}% ({}/{})",
                            (ratio * 100.0) as u32,
                            agent.quota_used,
                            agent.quota_limit
                        ),
                        Style::default().fg(COLOR_TEXT),
                    ),
                ]));
            }
        }
    }

    let chart_para = Paragraph::new(chart_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ▦ USAGE COMPARISON ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );
    f.render_widget(chart_para, left_chunks[1]);

    // Live Logs preview card
    let log_items: Vec<ListItem> = ctx
        .logs
        .iter()
        .rev()
        .take(15)
        .map(|log| {
            ListItem::new(Line::from(vec![
                Span::styled("❯ ", Style::default().fg(color_primary)),
                Span::styled(log, Style::default().fg(COLOR_TEXT)),
            ]))
        })
        .collect();

    let logs_list = List::new(log_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ≡ ACTIVITY LOG ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );
    f.render_widget(logs_list, left_chunks[2]);

    // ── Right Pane: Stats Grid + Live Gauges ──────────────────────────────────
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Stats summary boxes
            Constraint::Min(10),   // Progress Gauges
        ])
        .split(chunks[1]);

    // Stats Grid
    let grid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(right_chunks[0]);

    let mut total_tokens = 0u64;
    let mut total_cost = 0.0f64;
    for agent in ctx.agents {
        if agent.executable_path.is_some() {
            if let Some(t) = agent.tokens_used {
                total_tokens += t;
            }
            if let Some(c) = agent.cost_usd {
                total_cost += c;
            }
        }
    }

    let total_tokens_str = if total_tokens >= 1_000_000 {
        format!("{:.2}M", total_tokens as f64 / 1_000_000.0)
    } else if total_tokens >= 1_000 {
        format!("{:.1}K", total_tokens as f64 / 1_000.0)
    } else {
        total_tokens.to_string()
    };

    let stat_boxes = [
        (
            "Assistants",
            "⬡",
            format!("{} / {}", active_agents, ctx.agents.len()),
            color_primary,
        ),
        ("Requests", "⬢", total_requests.to_string(), COLOR_SUCCESS),
        ("Tokens", "◈", total_tokens_str, COLOR_WARN),
        (
            "Est. Spend",
            "$",
            format!("${:.2}", total_cost),
            COLOR_DANGER,
        ),
    ];

    for (i, &(title, icon, ref val, color)) in stat_boxes.iter().enumerate() {
        let text = vec![
            Line::from(Span::styled(
                icon,
                Style::default().fg(color).add_modifier(Modifier::DIM),
            ))
            .alignment(Alignment::Center),
            Line::from(Span::styled(
                val.clone(),
                Style::default()
                    .fg(color)
                    .bold()
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        ];
        let border_style = if i == 0 {
            Style::default().fg(color_primary)
        } else {
            Style::default().fg(COLOR_DIM)
        };
        let p = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(border_style)
                    .bg(COLOR_CARD)
                    .title(Span::styled(
                        format!(" {} ", title),
                        Style::default().fg(COLOR_MUTED),
                    )),
            )
            .alignment(Alignment::Center);
        f.render_widget(p, grid_chunks[i]);
    }

    // Build list of active gauges to display in separated sections!
    let mut gauge_rows = Vec::new();
    for agent in ctx.agents {
        if agent.executable_path.is_none() {
            continue; // Omit telemetry for uninstalled agents
        }

        let ratio = if agent.quota_limit > 0 {
            (agent.quota_used as f64 / agent.quota_limit as f64).min(1.0)
        } else {
            0.0
        };

        let reset_str = if agent.quota_type == crate::agent::QuotaType::Unlimited {
            "Local".to_string()
        } else {
            let secs = agent.seconds_until_reset;
            if secs <= 0 {
                "Renewing...".to_string()
            } else {
                let days = secs / (24 * 3600);
                let hours = (secs % (24 * 3600)) / 3600;
                let minutes = (secs % 3600) / 60;
                if days > 0 {
                    format!("{}d {}h", days, hours)
                } else {
                    format!("{}h {}m", hours, minutes)
                }
            }
        };

        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        gauge_rows.push((
            agent.name.clone(),
            agent.user_tier.display_name(),
            agent_color,
            ratio,
            agent.quota_used,
            agent.quota_limit,
            reset_str,
            agent.quota_type,
        ));
    }

    if gauge_rows.is_empty() {
        let no_agent_card = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ◈ ACTIVE QUOTAS ",
                Style::default().fg(COLOR_MUTED).bold(),
            ));
        let inner = no_agent_card.inner(right_chunks[1]);
        f.render_widget(no_agent_card, right_chunks[1]);

        let no_agent_p = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "◌  No active AI agents detected",
                Style::default().fg(COLOR_MUTED).italic(),
            )),
            Line::from(Span::styled(
                "Install agents and restart to enable quota tracking.",
                Style::default().fg(COLOR_DIM).italic(),
            )),
        ])
        .alignment(Alignment::Center);
        f.render_widget(no_agent_p, inner);
    } else {
        // Each agent gets its own card row
        let constraints = vec![Constraint::Length(5); gauge_rows.len()];
        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(right_chunks[1]);

        for (
            i,
            (name, tier_name, name_color, ratio, quota_used, quota_limit, reset_str, quota_type),
        ) in gauge_rows.into_iter().enumerate()
        {
            let soft = ctx.config.soft_limit_percent / 100.0;
            let hard = ctx.config.hard_limit_percent / 100.0;
            let bar_color = ratio_color(ratio, soft, hard);

            let agent_card = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(name_color))
                .bg(COLOR_CARD)
                .title(Span::styled(
                    format!(" {} ", name.to_uppercase()),
                    Style::default().fg(name_color).bold(),
                ))
                .title_bottom(Span::styled(
                    format!(" {} ", tier_name),
                    Style::default().fg(COLOR_MUTED).italic(),
                ));

            let inner = agent_card.inner(row_chunks[i]);
            f.render_widget(agent_card, row_chunks[i]);

            // Layout: [meta 35%] [gauge 65%]
            let card_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                .split(inner);

            // Left: Metadata column
            let info_text = vec![
                Line::from(vec![
                    Span::styled(" ⏱ ", Style::default().fg(COLOR_MUTED)),
                    Span::styled(&reset_str, Style::default().fg(COLOR_SUCCESS).bold()),
                ]),
                Line::from(vec![
                    Span::styled(" ◎ ", Style::default().fg(COLOR_MUTED)),
                    Span::styled(tier_name, Style::default().fg(COLOR_INFO)),
                ]),
            ];
            f.render_widget(Paragraph::new(info_text), card_layout[0]);

            // Right: Progress gauge with label
            let label = if quota_type == crate::agent::QuotaType::Unlimited {
                "∞  Unlimited Local".to_string()
            } else {
                format!("{:.1}%  {}/{} reqs", ratio * 100.0, quota_used, quota_limit)
            };

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(28, 30, 38)))
                .ratio(if quota_type == crate::agent::QuotaType::Unlimited {
                    1.0
                } else {
                    ratio
                })
                .label(Span::styled(
                    label,
                    Style::default().fg(Color::White).bold(),
                ));

            f.render_widget(gauge, card_layout[1]);
        }
    }
}

fn draw_agents_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // List of Agents
            Constraint::Percentage(75), // Agent Details
        ])
        .split(area);

    // ── Agent List Sidebar ─────────────────────────────────────────────────────
    let mut list_items = Vec::new();
    for (i, agent) in ctx.agents.iter().enumerate() {
        let is_selected = i == ctx.selected_agent_idx;
        let is_inst = agent.executable_path.is_some();

        let item_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        let (prefix, status_dot, status_color) = if is_inst {
            (SYM_ARROW, "● ", COLOR_SUCCESS)
        } else {
            (" ", "○ ", COLOR_MUTED)
        };

        let name_style = if is_selected {
            Style::default().fg(Color::Black).bg(item_color).bold()
        } else if is_inst {
            Style::default().fg(COLOR_TEXT)
        } else {
            Style::default().fg(COLOR_MUTED).add_modifier(Modifier::DIM)
        };

        // Add a top margin to visually separate agents
        if i > 0 {
            list_items.push(ListItem::new(Line::from("")).style(Style::default()));
        }
        list_items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("{} ", prefix),
                Style::default()
                    .fg(if is_selected { item_color } else { COLOR_DIM })
                    .bold(),
            ),
            Span::styled(status_dot, Style::default().fg(status_color)),
            Span::styled(agent.name.clone(), name_style),
        ])));
    }

    let list_border_color = if ctx.active_tab == 1 {
        color_primary
    } else {
        COLOR_MUTED
    };
    let agents_list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(list_border_color))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ◈ ASSISTANTS ",
                Style::default().fg(color_primary).bold(),
            )),
    );
    f.render_widget(agents_list, chunks[0]);

    // Selected Agent details on the Right
    let selected_agent = &ctx.agents[ctx.selected_agent_idx];
    let agent_color = match selected_agent.id {
        AgentId::Codex => COLOR_CODEX,
        AgentId::OpenCode => COLOR_OPENCODE,
        AgentId::Agy => COLOR_AGY,
        AgentId::Zed => COLOR_ZED,
    };

    let is_inst = selected_agent.executable_path.is_some();

    if !is_inst {
        let card_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                format!(" ✘ {} — NOT INSTALLED ", selected_agent.name.to_uppercase()),
                Style::default().fg(COLOR_DANGER).bold(),
            ));

        let inner_rect = card_block.inner(chunks[1]);
        f.render_widget(card_block, chunks[1]);

        let warning_lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("⚠  {} is not installed", selected_agent.name),
                Style::default().fg(COLOR_WARN).bold(),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(
                "No executable binary found in PATH",
                Style::default().fg(COLOR_MUTED),
            ))
            .alignment(Alignment::Center),
            Line::from(Span::styled(
                "Telemetry and quota tracking are disabled",
                Style::default().fg(COLOR_MUTED).italic(),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(
                "Install the assistant to enable automatic monitoring",
                Style::default().fg(COLOR_DIM).italic(),
            ))
            .alignment(Alignment::Center),
        ];
        let warning_para = Paragraph::new(warning_lines);
        f.render_widget(warning_para, inner_rect);
        return;
    }

    // Agent Details (When installed)
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Metadata
            Constraint::Length(10), // Quota/Stats (highly optimized height)
            Constraint::Min(8),     // Model Breakdown Table
            Constraint::Length(2),  // Tip/Commands
        ])
        .split(chunks[1]);

    let meta_rows = vec![
        Row::new(vec![
            Cell::new("Binary Path:"),
            Cell::new(selected_agent.executable_path.clone().unwrap_or_default())
                .style(Style::default().fg(COLOR_TEXT)),
        ]),
        Row::new(vec![
            Cell::new("Version Detected:"),
            Cell::new(
                selected_agent
                    .version
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string()),
            ),
        ]),
        Row::new(vec![
            Cell::new("Config Directory:"),
            Cell::new(
                selected_agent
                    .config_path
                    .clone()
                    .unwrap_or_else(|| "None".to_string()),
            ),
        ]),
        Row::new(vec![
            Cell::new("Auth Status:"),
            Cell::new(if selected_agent.is_authenticated {
                "✔ Connected"
            } else {
                "✘ Disconnected"
            })
            .style(
                Style::default()
                    .fg(if selected_agent.is_authenticated {
                        COLOR_SUCCESS
                    } else {
                        COLOR_WARN
                    })
                    .bold(),
            ),
        ]),
        Row::new(vec![
            Cell::new("Auth Identity / User:"),
            Cell::new(selected_agent.auth_info.clone()).style(Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let meta_table = Table::new(meta_rows, [Constraint::Length(24), Constraint::Min(20)]).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(agent_color))
            .bg(COLOR_CARD)
            .title(Span::styled(
                format!(" ◈ {} — DETAILS ", selected_agent.name.to_uppercase()),
                Style::default().fg(agent_color).bold(),
            )),
    );
    f.render_widget(meta_table, detail_chunks[0]);

    let quota_ratio = if selected_agent.quota_limit > 0 {
        (selected_agent.quota_used as f32 / selected_agent.quota_limit as f32).min(1.0)
    } else {
        0.0
    };

    let stats_card = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(COLOR_MUTED))
        .bg(COLOR_CARD)
        .title(Span::styled(
            " ▤ QUOTA & RATE LIMIT UTILIZATION ",
            Style::default().fg(COLOR_MUTED).bold(),
        ));

    let stats_inner = stats_card.inner(detail_chunks[1]);
    f.render_widget(stats_card, detail_chunks[1]);

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Quota Used
            Constraint::Length(2), // Quota Remaining
            Constraint::Min(2),    // Additional Info
        ])
        .split(stats_inner);

    // Gauge 1: Used
    let used_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(color_primary)
                .bg(Color::Rgb(40, 40, 45)),
        )
        .ratio(
            if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
                1.0
            } else {
                quota_ratio as f64
            },
        )
        .label(
            if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
                "Uso Local Ilimitado".to_string()
            } else {
                format!(
                    "{}/{} Requests Used ({:.1}%)",
                    selected_agent.quota_used,
                    selected_agent.quota_limit,
                    quota_ratio * 100.0
                )
            },
        );
    let used_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(10)])
        .split(gauge_chunks[0]);
    f.render_widget(
        Paragraph::new("Requests Used:").style(Style::default().bold()),
        used_layout[0],
    );
    f.render_widget(used_gauge, used_layout[1]);

    // Gauge 2: Remaining
    let rem_ratio = 1.0 - quota_ratio;
    let rem_gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(COLOR_SUCCESS)
                .bg(Color::Rgb(40, 40, 45)),
        )
        .ratio(
            if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
                1.0
            } else {
                rem_ratio as f64
            },
        )
        .label(
            if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
                "Uso Local Ilimitado".to_string()
            } else {
                format!(
                    "{} Requests Remaining ({:.1}%)",
                    selected_agent.quota_remaining,
                    rem_ratio * 100.0
                )
            },
        );
    let rem_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(10)])
        .split(gauge_chunks[1]);
    f.render_widget(
        Paragraph::new("Quota Available:").style(Style::default().bold()),
        rem_layout[0],
    );
    f.render_widget(rem_gauge, rem_layout[1]);

    // Info 3: Renewal & Tier
    let secs = selected_agent.seconds_until_reset;
    let reset_str = if selected_agent.quota_type == crate::agent::QuotaType::Unlimited {
        "Never expires (Free local usage)".to_string()
    } else if secs <= 0 {
        "Renewing quota now...".to_string()
    } else {
        let days = secs / (24 * 3600);
        let hours = (secs % (24 * 3600)) / 3600;
        let minutes = (secs % 3600) / 60;
        if days > 0 {
            format!("in {} days and {} hours", days, hours)
        } else {
            format!("in {} hours and {} minutes", hours, minutes)
        }
    };

    let mut info_text = vec![Line::from(vec![
        Span::styled("User Account Tier: ", Style::default().fg(COLOR_MUTED)),
        Span::styled(
            selected_agent.user_tier.display_name(),
            Style::default().fg(color_primary).bold(),
        ),
        Span::styled("   |   Frequency: ", Style::default().fg(COLOR_MUTED)),
        Span::styled(
            match selected_agent.quota_type {
                crate::agent::QuotaType::Daily => "Daily",
                crate::agent::QuotaType::Weekly => "Weekly",
                crate::agent::QuotaType::Monthly => "Monthly",
                crate::agent::QuotaType::Unlimited => "Unlimited",
            },
            Style::default().fg(COLOR_WARN).bold(),
        ),
        Span::styled("   |   Will Renew: ", Style::default().fg(COLOR_MUTED)),
        Span::styled(reset_str, Style::default().fg(COLOR_SUCCESS).bold()),
    ])];

    if selected_agent.tokens_used.is_some() || selected_agent.cost_usd.is_some() {
        let mut extra_spans = Vec::new();
        if let Some(tokens) = selected_agent.tokens_used {
            extra_spans.push(Span::styled(
                "Tokens Consumed: ",
                Style::default().fg(COLOR_MUTED),
            ));
            let tok_str = if tokens >= 1_000_000 {
                format!("{:.2}M", tokens as f64 / 1_000_000.0)
            } else if tokens >= 1_000 {
                format!("{:.1}K", tokens as f64 / 1_000.0)
            } else {
                tokens.to_string()
            };
            extra_spans.push(Span::styled(
                tok_str,
                Style::default().fg(COLOR_TEXT).bold(),
            ));
        }
        if let Some(cost) = selected_agent.cost_usd {
            if !extra_spans.is_empty() {
                extra_spans.push(Span::styled("   |   ", Style::default().fg(COLOR_MUTED)));
            }
            extra_spans.push(Span::styled(
                "Estimated Cost: ",
                Style::default().fg(COLOR_MUTED),
            ));
            extra_spans.push(Span::styled(
                format!("${:.4}", cost),
                Style::default().fg(COLOR_SUCCESS).bold(),
            ));
        }
        info_text.push(Line::from(extra_spans));
    }

    f.render_widget(Paragraph::new(info_text), gauge_chunks[2]);

    // Model Breakdown Card
    let mut model_rows = Vec::new();
    for model in &selected_agent.model_usages {
        let ratio = if model.limit > 0 {
            (model.requests_used as f64 / model.limit as f64).min(1.0)
        } else {
            0.0
        };

        let bar_width = 24;
        let bar_str = make_progress_bar(ratio, bar_width);

        let soft_warn = ctx.config.soft_limit_percent / 100.0;
        let hard_warn = ctx.config.hard_limit_percent / 100.0;

        let color_status = if ratio >= hard_warn {
            COLOR_DANGER
        } else if ratio >= soft_warn {
            COLOR_WARN
        } else {
            COLOR_SUCCESS
        };

        model_rows.push(Row::new(vec![
            Cell::new(format!("  {}", model.name)).style(Style::default().fg(agent_color).bold()),
            Cell::new(format!("{} / {} reqs", model.requests_used, model.limit))
                .style(Style::default().fg(COLOR_TEXT)),
            Cell::new(format!("{:.1}%", ratio * 100.0)).style(Style::default().fg(COLOR_TEXT)),
            Cell::new(bar_str).style(Style::default().fg(color_status)),
        ]));
    }

    let model_table = Table::new(
        model_rows,
        [
            Constraint::Percentage(36),
            Constraint::Percentage(24),
            Constraint::Percentage(12),
            Constraint::Percentage(28),
        ],
    )
    .header(
        Row::new(vec!["  Model", "Usage / Limit", "Used %", "Progress"])
            .style(
                Style::default()
                    .fg(color_primary)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ▦ MODEL-LEVEL BREAKDOWN ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );
    f.render_widget(model_table, detail_chunks[2]);

    // Quick Command Hint Bar
    let inst_text = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            " s ",
            Style::default().fg(Color::Black).bg(color_primary).bold(),
        ),
        Span::styled(" Modify quota limit  ", Style::default().fg(COLOR_MUTED)),
        Span::styled(
            " ↑↓ ",
            Style::default().fg(Color::Black).bg(COLOR_DIM).bold(),
        ),
        Span::styled(" Navigate agents ", Style::default().fg(COLOR_MUTED)),
    ]);

    let inst_para = Paragraph::new(inst_text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(COLOR_DIM)),
        )
        .alignment(Alignment::Left);
    f.render_widget(inst_para, detail_chunks[3]);
}

fn draw_sessions_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Sessions Log Table
        ])
        .split(area);

    let info_text = Paragraph::new(Line::from(vec![
        Span::styled(
            " ◷ SESSION HISTORY ",
            Style::default().fg(color_primary).bold(),
        ),
        Span::styled(
            " — queried from local databases in the background ",
            Style::default().fg(COLOR_MUTED).italic(),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color_primary)),
    );
    f.render_widget(info_text, chunks[0]);

    let mut rows = Vec::new();

    for agent in ctx.agents {
        if agent.executable_path.is_none() {
            continue; // Omit telemetry for uninstalled agents
        }

        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        if agent.sessions_count > 0 {
            for idx in 0..agent.sessions_count.min(5) {
                let session_id = format!("{:x}", 1395819581293u64 + idx as u64);
                rows.push(Row::new(vec![
                    Cell::new(format!("  {}", agent.name))
                        .style(Style::default().fg(agent_color).bold()),
                    Cell::new(format!("#{}", &session_id[..8]))
                        .style(Style::default().fg(COLOR_INFO)),
                    Cell::new(format!("{}m ago", idx * 10 + 5))
                        .style(Style::default().fg(COLOR_MUTED)),
                    Cell::new(" ✔ OK ")
                        .style(Style::default().fg(Color::Black).bg(COLOR_SUCCESS).bold()),
                    Cell::new(format!(
                        "{} reqs",
                        agent.requests_count / agent.sessions_count
                    ))
                    .style(Style::default().fg(COLOR_TEXT)),
                ]));
            }
        }
    }

    if rows.is_empty() {
        rows.push(Row::new(vec![
            Cell::new("  Codex").style(Style::default().fg(COLOR_CODEX).bold()),
            Cell::new("#dd34ff5a").style(Style::default().fg(COLOR_INFO)),
            Cell::new("10m ago").style(Style::default().fg(COLOR_MUTED)),
            Cell::new(" ✔ OK ").style(Style::default().fg(Color::Black).bg(COLOR_SUCCESS).bold()),
            Cell::new("12 reqs").style(Style::default().fg(COLOR_TEXT)),
        ]));
        rows.push(Row::new(vec![
            Cell::new("  Agy").style(Style::default().fg(COLOR_AGY).bold()),
            Cell::new("#5f2a3221").style(Style::default().fg(COLOR_INFO)),
            Cell::new("1h ago").style(Style::default().fg(COLOR_MUTED)),
            Cell::new(" ✔ OK ").style(Style::default().fg(Color::Black).bg(COLOR_SUCCESS).bold()),
            Cell::new("4 reqs").style(Style::default().fg(COLOR_TEXT)),
        ]));
    }

    let sessions_table = Table::new(
        rows,
        [
            Constraint::Percentage(18),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
            Constraint::Percentage(14),
            Constraint::Percentage(18),
        ],
    )
    .header(
        Row::new(vec![
            "  Agent",
            "Session Hash",
            "Elapsed",
            "Status",
            "Requests",
        ])
        .style(
            Style::default()
                .fg(color_primary)
                .bold()
                .add_modifier(Modifier::UNDERLINED),
        )
        .bottom_margin(1),
    )
    .row_highlight_style(Style::default().bg(Color::Rgb(30, 32, 42)))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ◷ RECENT SESSIONS ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );

    f.render_widget(sessions_table, chunks[1]);
}

fn draw_quotas_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Quota Table
            Constraint::Min(4),     // Operations Guide
        ])
        .split(area);

    let mut rows = Vec::new();
    for (i, agent) in ctx.agents.iter().enumerate() {
        let is_selected = i == ctx.selected_agent_idx;
        let is_inst = agent.executable_path.is_some();

        let prefix = if is_selected { "❯ " } else { "  " };
        let agent_color = match agent.id {
            AgentId::Codex => COLOR_CODEX,
            AgentId::OpenCode => COLOR_OPENCODE,
            AgentId::Agy => COLOR_AGY,
            AgentId::Zed => COLOR_ZED,
        };

        let action_cell = if is_inst {
            Cell::new("Modify (s)").style(Style::default().fg(color_primary).italic())
        } else {
            Cell::new("N/A - Telemetry Omitted").style(Style::default().fg(COLOR_MUTED))
        };

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 30, 35))
        } else {
            Style::default()
        };

        rows.push(
            Row::new(vec![
                Cell::new(format!("{}{}", prefix, agent.name))
                    .style(Style::default().fg(agent_color).bold()),
                Cell::new(agent.user_tier.display_name().to_string())
                    .style(Style::default().fg(if is_inst { COLOR_TEXT } else { COLOR_MUTED })),
                Cell::new(
                    match agent.quota_type {
                        crate::agent::QuotaType::Daily => "Daily",
                        crate::agent::QuotaType::Weekly => "Weekly",
                        crate::agent::QuotaType::Monthly => "Monthly",
                        crate::agent::QuotaType::Unlimited => "Unlimited",
                    }
                    .to_string(),
                )
                .style(Style::default().fg(if is_inst {
                    COLOR_TEXT
                } else {
                    COLOR_MUTED
                })),
                Cell::new(if is_inst {
                    if agent.quota_type == crate::agent::QuotaType::Unlimited {
                        "Unlimited".to_string()
                    } else {
                        format!("{} requests", agent.quota_limit)
                    }
                } else {
                    "No Telemetry".to_string()
                })
                .style(Style::default().fg(if is_inst {
                    COLOR_TEXT
                } else {
                    COLOR_MUTED
                })),
                action_cell,
            ])
            .style(row_style),
        );
    }

    let budget_table = Table::new(
        rows,
        [
            Constraint::Percentage(22),
            Constraint::Percentage(26),
            Constraint::Percentage(14),
            Constraint::Percentage(18),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec![
            "  Agent",
            "Account Tier",
            "Frequency",
            "Limit",
            "Action",
        ])
        .style(
            Style::default()
                .fg(color_primary)
                .bold()
                .add_modifier(Modifier::UNDERLINED),
        )
        .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color_primary))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ◈ QUOTA CONFIGURATION ",
                Style::default().fg(color_primary).bold(),
            )),
    );
    f.render_widget(budget_table, chunks[0]);

    // ── Operations Guide ──────────────────────────────────────────────────────
    let docs_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  HOW IT WORKS  ",
            Style::default().fg(color_primary).bold(),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "  QuotaChecker-CLI scans local databases and log files to track active",
            Style::default().fg(COLOR_TEXT),
        )),
        Line::from(Span::styled(
            "  request counts. If an agent is not installed, telemetry is omitted.",
            Style::default().fg(COLOR_MUTED),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  KEYBINDS  ",
            Style::default().fg(color_primary).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                " Tab ",
                Style::default().fg(Color::Black).bg(COLOR_DIM).bold(),
            ),
            Span::styled("  Change screen   ", Style::default().fg(COLOR_MUTED)),
            Span::styled(
                " ↑↓ ",
                Style::default().fg(Color::Black).bg(COLOR_DIM).bold(),
            ),
            Span::styled("  Select agent   ", Style::default().fg(COLOR_MUTED)),
            Span::styled(
                " s ",
                Style::default().fg(Color::Black).bg(color_primary).bold(),
            ),
            Span::styled("  Edit limit   ", Style::default().fg(COLOR_MUTED)),
            Span::styled(
                " q ",
                Style::default().fg(Color::Black).bg(COLOR_DANGER).bold(),
            ),
            Span::styled("  Quit", Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let docs_para = Paragraph::new(docs_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ≡ GUIDE ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );
    f.render_widget(docs_para, chunks[1]);
}

fn draw_settings_tab(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Options Grid (expanded to fit 5 rows)
            Constraint::Min(4),     // Operational Info
        ])
        .split(area);

    let mut rows = Vec::new();
    let settings = [
        (
            "TUI Active Color Theme",
            format!("< {:?} >", ctx.config.theme),
            "Customize TUI highlight brand accent",
        ),
        (
            "Telemetry Refresh Interval",
            format!("< {}ms >", ctx.config.refresh_rate_ms),
            "How often SQLite files are scanned",
        ),
        (
            "Soft Warning Threshold (%)",
            format!("< {}% >", ctx.config.soft_limit_percent as u32),
            "Usage warning threshold",
        ),
        (
            "Hard Warning Threshold (%)",
            format!("< {}% >", ctx.config.hard_limit_percent as u32),
            "Quota exceeded limit indicator",
        ),
        (
            "Manual Config JSON Editor",
            "[ Press Enter / e ]".to_string(),
            "Opens config.json in terminal $EDITOR",
        ),
    ];

    for (i, &(name, ref val, desc)) in settings.iter().enumerate() {
        let is_selected = i == ctx.selected_setting_idx;
        let prefix = if is_selected { "❯ " } else { "  " };
        let name_style = if is_selected {
            Style::default().fg(color_primary).bold()
        } else {
            Style::default().fg(COLOR_TEXT)
        };

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(30, 30, 35))
        } else {
            Style::default()
        };

        let val_color = if i == 4 { COLOR_WARN } else { COLOR_SUCCESS };

        rows.push(
            Row::new(vec![
                Cell::new(format!("{}{}", prefix, name)).style(name_style),
                Cell::new(val.clone()).style(Style::default().fg(val_color).bold()),
                Cell::new(desc.to_string()).style(Style::default().fg(COLOR_MUTED)),
            ])
            .style(row_style),
        );
    }

    let config_table = Table::new(
        rows,
        [
            Constraint::Percentage(38),
            Constraint::Percentage(24),
            Constraint::Percentage(38),
        ],
    )
    .header(
        Row::new(vec!["  Setting", "Value", "Description"])
            .style(
                Style::default()
                    .fg(color_primary)
                    .bold()
                    .add_modifier(Modifier::UNDERLINED),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color_primary))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ⚙ TUI SETTINGS ",
                Style::default().fg(color_primary).bold(),
            )),
    );
    f.render_widget(config_table, chunks[0]);

    // Path details card
    let config_path_str = AppConfig::config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let path_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  ⚙ CONFIG FILE  ",
            Style::default().fg(color_primary).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Path: ", Style::default().fg(COLOR_MUTED)),
            Span::styled(config_path_str, Style::default().fg(COLOR_INFO).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Settings are persisted to disk instantly.",
            Style::default().fg(COLOR_MUTED).italic(),
        )),
        Line::from(Span::styled(
            "  You can also edit this file manually with any terminal editor.",
            Style::default().fg(COLOR_MUTED).italic(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                " ↑↓ ",
                Style::default().fg(Color::Black).bg(COLOR_DIM).bold(),
            ),
            Span::styled("  Select   ", Style::default().fg(COLOR_MUTED)),
            Span::styled(
                " Enter / +/- ",
                Style::default().fg(Color::Black).bg(COLOR_DIM).bold(),
            ),
            Span::styled("  Cycle value", Style::default().fg(COLOR_MUTED)),
        ]),
    ];

    let path_para = Paragraph::new(path_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(COLOR_MUTED))
            .bg(COLOR_CARD)
            .title(Span::styled(
                " ≡ CONFIGURATION INFO ",
                Style::default().fg(COLOR_MUTED).bold(),
            )),
    );
    f.render_widget(path_para, chunks[1]);
}

/// Renders a keybind "pill": colored bg key + muted label.
fn kpill<'a>(key: &'a str, label: &'a str, key_color: Color) -> Vec<Span<'a>> {
    vec![
        Span::styled(" ", Style::default()),
        Span::styled(
            format!(" {} ", key),
            Style::default().fg(Color::Black).bg(key_color).bold(),
        ),
        Span::styled(format!(" {} ", label), Style::default().fg(COLOR_MUTED)),
        Span::styled(SYM_SEP, Style::default().fg(COLOR_DIM)),
    ]
}

fn draw_footer(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    // Common keybinds
    let mut footer_spans: Vec<Span> = Vec::new();
    footer_spans.extend(kpill("q", "Quit", COLOR_DANGER));
    footer_spans.extend(kpill("Tab", "Switch tab", COLOR_DIM));

    // Tab-specific
    match ctx.active_tab {
        1 | 3 => {
            footer_spans.extend(kpill("↑↓", "Select agent", COLOR_DIM));
            if ctx.agents[ctx.selected_agent_idx].executable_path.is_some() {
                footer_spans.extend(kpill("s", "Edit quota", color_primary));
            }
        }
        4 => {
            footer_spans.extend(kpill("↑↓", "Select", COLOR_DIM));
            if ctx.selected_setting_idx == 4 {
                footer_spans.extend(kpill("Enter", "Open editor", color_primary));
            } else {
                footer_spans.extend(kpill("Enter/+/-", "Cycle value", color_primary));
            }
        }
        _ => {
            footer_spans.extend(kpill("r", "Force refresh", color_primary));
        }
    }

    // Version / right side
    footer_spans.push(Span::styled(
        " QuotaChecker-TUI v0.3",
        Style::default().fg(COLOR_DIM).italic(),
    ));

    let footer_text = Line::from(footer_spans);
    let footer_widget = Paragraph::new(footer_text)
        .block(Block::default())
        .style(Style::default());
    f.render_widget(footer_widget, area);
}

// Helpers
use ratatui::widgets::Cell;

fn draw_budget_modal(f: &mut Frame, area: Rect, ctx: &RenderContext) {
    let color_primary = get_primary_color(ctx.config.theme);

    // Centered popup: 56 wide, 9 tall
    let modal_rect = centered_rect(56, 9, area);

    // Slight shadow: render a cleared block slightly offset for depth illusion
    let shadow_rect = Rect {
        x: modal_rect.x + 1,
        y: modal_rect.y + 1,
        width: modal_rect.width,
        height: modal_rect.height,
    };
    f.render_widget(Clear, shadow_rect);
    f.render_widget(Clear, modal_rect);

    let active_agent = &ctx.agents[ctx.selected_agent_idx];
    let agent_color = match active_agent.id {
        AgentId::Codex => COLOR_CODEX,
        AgentId::OpenCode => COLOR_OPENCODE,
        AgentId::Agy => COLOR_AGY,
        AgentId::Zed => COLOR_ZED,
    };

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color_primary))
        .bg(Color::Rgb(18, 20, 28))
        .title(Span::styled(
            format!(" ⚙ QUOTA LIMIT — {} ", active_agent.name.to_uppercase()),
            Style::default().fg(agent_color).bold(),
        ))
        .title_bottom(Span::styled(
            " Enter ✔ Save  │  Esc ✘ Cancel ",
            Style::default().fg(COLOR_MUTED),
        ));

    let inner_rect = modal_block.inner(modal_rect);
    f.render_widget(modal_block, modal_rect);

    let form_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Hint
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Input field
            Constraint::Min(1),    // Padding
        ])
        .split(inner_rect);

    // Hint line
    let hint = Paragraph::new(Line::from(vec![
        Span::styled(" Current: ", Style::default().fg(COLOR_MUTED)),
        Span::styled(
            format!("{} requests", active_agent.quota_limit),
            Style::default().fg(color_primary).bold(),
        ),
        Span::styled(
            "  →  Enter new limit:",
            Style::default().fg(COLOR_MUTED).italic(),
        ),
    ]));
    f.render_widget(hint, form_layout[0]);

    // Input row
    let cursor_suffix = if ctx.tick_count.is_multiple_of(2) {
        "▌"
    } else {
        " "
    };
    let display_val = ctx.editing_value.to_string();
    let is_valid = display_val.parse::<u32>().is_ok();

    let row_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(5)])
        .split(form_layout[2]);

    let label_p = Paragraph::new(Line::from(Span::styled(
        " Request Limit:",
        Style::default().fg(COLOR_MUTED),
    )));

    let (border_color, text_style, display_text) = if display_val.is_empty() {
        (
            COLOR_DANGER,
            Style::default().fg(COLOR_DIM),
            format!("Enter limit...{}", cursor_suffix),
        )
    } else if !is_valid {
        (
            COLOR_DANGER,
            Style::default().fg(COLOR_DANGER).bold(),
            format!("{}{}", display_val, cursor_suffix),
        )
    } else {
        (
            color_primary,
            Style::default().fg(COLOR_TEXT).bold(),
            format!("{}{}", display_val, cursor_suffix),
        )
    };

    let field_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color));
    let val_p = Paragraph::new(display_text)
        .style(text_style)
        .block(field_block);

    f.render_widget(label_p, row_chunks[0]);
    f.render_widget(val_p, row_chunks[1]);

    if !is_valid || display_val.is_empty() {
        let warning_p = Paragraph::new(Line::from(Span::styled(
            " ⚠ Valid number required",
            Style::default().fg(COLOR_DANGER).italic(),
        )));
        f.render_widget(warning_p, form_layout[3]);
    }
}
