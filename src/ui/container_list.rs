use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::app_state::AppState;
use crate::core::types::{Container, ContainerState, HealthStatus, SortField, SortState};
use crate::ui::formatters::{format_bytes, format_bytes_per_sec, format_time_elapsed};
use crate::ui::render::UiStyles;
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
};

/// Braille characters for sparkline vertical bars (0-4 rows filled)
/// Using bottom-aligned braille patterns for vertical bar effect
const BRAILLE_BARS: [char; 5] = [
    '⠀', // 0: empty (U+2800)
    '⣀', // 1: bottom row (U+28C0)
    '⣤', // 2: bottom two rows (U+28E4)
    '⣶', // 3: bottom three rows (U+28F6)
    '⣿', // 4: all rows filled (U+28FF)
];

/// Braille characters with tick marker - dot 7 (bottom-left) removed to create a "hole"
/// For filled bars, the missing dot makes the tick position visible
const BRAILLE_BARS_WITH_TICK: [char; 5] = [
    '⡀', // 0: empty + bottom tick (U+2840) - shows dot for visibility
    '⢀', // 1: bottom row minus dot 7 (U+2880) - hole in left
    '⢤', // 2: bottom two rows minus dot 7 (U+28A4) - hole in left
    '⢶', // 3: bottom three rows minus dot 7 (U+28B6) - hole in left
    '⢿', // 4: all rows minus dot 7 (U+28BF) - hole in left
];

/// Interval for tick markers (every N positions)
const TICK_INTERVAL: usize = 5;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Renders the container list view
pub fn render_container_list(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    app_state: &mut AppState,
    styles: &UiStyles,
    show_host_column: bool,
) {
    let width = area.width;

    // Determine if we should show progress bars based on terminal width
    let show_progress_bars = width >= 128;

    // Get global tick counter from wall clock time (half-seconds since epoch)
    // Dividing by 2 makes ticks advance every 2 seconds for smoother animation
    // This ensures all containers have synchronized tick markers
    let global_tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 2)
        .unwrap_or(0);

    // Use pre-sorted list instead of sorting every frame
    let rows: Vec<Row> = app_state
        .sorted_container_keys
        .iter()
        .filter_map(|key| app_state.containers.get(key))
        .map(|c| create_container_row(c, styles, show_host_column, show_progress_bars, global_tick))
        .collect();

    let header = create_header_row(styles, show_host_column, app_state.sort_state);
    let table = create_table(
        rows,
        header,
        app_state.sorted_container_keys.len(),
        styles,
        show_host_column,
        show_progress_bars,
    );

    f.render_stateful_widget(table, area, &mut app_state.table_state);
}

/// Creates a table row for a single container
fn create_container_row<'a>(
    container: &'a Container,
    styles: &UiStyles,
    show_host_column: bool,
    show_progress_bars: bool,
    global_tick: u64,
) -> Row<'a> {
    // Check if container is running
    let is_running = container.state == ContainerState::Running;

    // Only show stats for running containers
    let (cpu_bar, cpu_style) = if is_running {
        let display = if show_progress_bars {
            create_cpu_sparkline(
                &container.stats.cpu_history,
                container.stats.cpu,
                20,
                global_tick,
            )
        } else {
            format!("{:5.1}%", container.stats.cpu)
        };
        (display, get_percentage_style(container.stats.cpu, styles))
    } else {
        (String::new(), Style::default())
    };

    let (memory_bar, memory_style) = if is_running {
        let display = if show_progress_bars {
            create_memory_sparkline(
                &container.stats.memory_history,
                container.stats.memory_used_bytes,
                container.stats.memory_limit_bytes,
                20,
                global_tick,
            )
        } else {
            format!("{:5.1}%", container.stats.memory)
        };
        (
            display,
            get_percentage_style(container.stats.memory, styles),
        )
    } else {
        (String::new(), Style::default())
    };

    let network_tx = if is_running {
        format_bytes_per_sec(container.stats.network_tx_bytes_per_sec)
    } else {
        String::new()
    };

    let network_rx = if is_running {
        format_bytes_per_sec(container.stats.network_rx_bytes_per_sec)
    } else {
        String::new()
    };

    // Format time elapsed since creation - show "N/A" for non-running containers
    let time_elapsed = if is_running {
        format_time_elapsed(container.created.as_ref())
    } else {
        "N/A".to_string()
    };

    // Get status icon and color (health takes priority over state)
    let (icon, icon_style) = get_status_icon(&container.state, &container.health, styles);

    let mut cells = vec![
        Cell::from(container.id.as_str()).style(styles.container_id),
        Cell::from(icon).style(icon_style),
        Cell::from(container.name.as_str()),
    ];

    if show_host_column {
        cells.push(Cell::from(container.host_id.as_str()));
    }

    cells.extend(vec![
        Cell::from(cpu_bar).style(cpu_style),
        Cell::from(memory_bar).style(memory_style),
        Cell::from(Line::styled(network_tx, styles.network_tx).right_aligned()),
        Cell::from(Line::styled(network_rx, styles.network_rx).right_aligned()),
        Cell::from(time_elapsed).style(styles.created),
    ]);

    Row::new(cells)
}

/// Creates a text-based progress bar with percentage (legacy, kept for tests)
#[cfg(test)]
fn create_progress_bar(percentage: f64, width: usize) -> String {
    // Clamp the bar visual to 100%, but display the actual percentage value
    let bar_percentage = percentage.clamp(0.0, 100.0);
    let filled_width = ((bar_percentage / 100.0) * width as f64).round() as usize;
    let empty_width = width.saturating_sub(filled_width);

    let bar = format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width));

    format!("{} {:5.1}%", bar, percentage)
}

/// Creates a text-based progress bar with memory used/limit display (legacy, kept for tests)
#[cfg(test)]
fn create_memory_progress_bar(percentage: f64, used: u64, limit: u64, width: usize) -> String {
    // Clamp the bar visual to 100%, but display the actual percentage value
    let bar_percentage = percentage.clamp(0.0, 100.0);
    let filled_width = ((bar_percentage / 100.0) * width as f64).round() as usize;
    let empty_width = width.saturating_sub(filled_width);

    let bar = format!("{}{}", "█".repeat(filled_width), "░".repeat(empty_width));

    format!("{} {}/{}", bar, format_bytes(used), format_bytes(limit))
}

/// Box drawing character for sparkline borders
const SPARKLINE_BORDER: char = '│';

/// Creates a braille-based sparkline from historical percentage values
/// Each character represents one sample, with height indicating the percentage
/// Tick markers march with the data based on global_tick (wall clock time)
fn create_sparkline(history: &VecDeque<f64>, width: usize, global_tick: u64) -> String {
    let mut sparkline = String::with_capacity(width + 2); // +2 for borders
    let history_len = history.len();

    // Opening border
    sparkline.push(SPARKLINE_BORDER);

    // Pad with empty chars if history is shorter than width
    // Padding positions don't get ticks - ticks only appear in actual data
    let padding = width.saturating_sub(history_len);
    for _ in 0..padding {
        sparkline.push(BRAILLE_BARS[0]);
    }

    // Convert each percentage to a braille bar character
    // Tick position is based on global_tick so ticks march synchronized across all containers
    for (i, &value) in history.iter().enumerate() {
        let bar_index = percentage_to_bar_index(value);
        // Calculate tick position based on global time and position in history
        // As global_tick advances, tick positions shift left (newer tick enters from right)
        let tick_position = global_tick.saturating_sub(history_len as u64) + i as u64;
        if tick_position % TICK_INTERVAL as u64 == 0 {
            sparkline.push(BRAILLE_BARS_WITH_TICK[bar_index]);
        } else {
            sparkline.push(BRAILLE_BARS[bar_index]);
        }
    }

    // Closing border
    sparkline.push(SPARKLINE_BORDER);

    sparkline
}

/// Maps a percentage (0-100) to a braille bar index (0-4)
fn percentage_to_bar_index(percentage: f64) -> usize {
    let clamped = percentage.clamp(0.0, 100.0);
    if clamped < 12.5 {
        0 // empty
    } else if clamped < 25.0 {
        1 // 1 row
    } else if clamped < 50.0 {
        2 // 2 rows
    } else if clamped < 75.0 {
        3 // 3 rows
    } else {
        4 // full
    }
}

/// Creates a CPU sparkline with percentage suffix
fn create_cpu_sparkline(history: &VecDeque<f64>, current: f64, width: usize, global_tick: u64) -> String {
    let sparkline = create_sparkline(history, width, global_tick);
    format!("{} {:5.1}%", sparkline, current)
}

/// Creates a memory sparkline with used/limit suffix
fn create_memory_sparkline(
    history: &VecDeque<f64>,
    used: u64,
    limit: u64,
    width: usize,
    global_tick: u64,
) -> String {
    let sparkline = create_sparkline(history, width, global_tick);
    format!("{} {}/{}", sparkline, format_bytes(used), format_bytes(limit))
}

/// Returns the status icon and color based on container health (if available) or state
fn get_status_icon(
    state: &ContainerState,
    health: &Option<HealthStatus>,
    styles: &UiStyles,
) -> (String, Style) {
    // Prioritize health status if container has health checks configured
    if let Some(health_status) = health {
        let icon = styles.icons.health(health_status).to_string();
        let style = match health_status {
            HealthStatus::Healthy => Style::default().fg(Color::Green),
            HealthStatus::Unhealthy => Style::default().fg(Color::Red),
            HealthStatus::Starting => Style::default().fg(Color::Yellow),
        };
        return (icon, style);
    }

    // Use state-based icon if no health check is configured
    let icon = styles.icons.state(state).to_string();
    let style = match state {
        ContainerState::Running => Style::default().fg(Color::Green),
        ContainerState::Paused => Style::default().fg(Color::Yellow),
        ContainerState::Restarting => Style::default().fg(Color::Yellow),
        ContainerState::Removing => Style::default().fg(Color::Yellow),
        ContainerState::Exited => Style::default().fg(Color::Red),
        ContainerState::Dead => Style::default().fg(Color::Red),
        ContainerState::Created => Style::default().fg(Color::Cyan),
        ContainerState::Unknown => Style::default().fg(Color::Gray),
    };
    (icon, style)
}

/// Returns the appropriate style based on percentage value
fn get_percentage_style(value: f64, styles: &UiStyles) -> Style {
    if value > 80.0 {
        styles.high
    } else if value > 50.0 {
        styles.medium
    } else {
        styles.low
    }
}

/// Creates the table header row
fn create_header_row(
    styles: &UiStyles,
    show_host_column: bool,
    sort_state: SortState,
) -> Row<'static> {
    let sort_symbol = sort_state.direction.symbol();
    let sort_field = sort_state.field;

    let mut headers = vec![
        "ID".to_string(),
        "".to_string(), // Status icon column (no header text)
        if sort_field == SortField::Name {
            format!("Name {}", sort_symbol)
        } else {
            "Name".to_string()
        },
    ];

    if show_host_column {
        headers.push("Host".to_string());
    }

    headers.extend(vec![
        if sort_field == SortField::Cpu {
            format!("CPU % {}", sort_symbol)
        } else {
            "CPU %".to_string()
        },
        if sort_field == SortField::Memory {
            format!("Memory % {}", sort_symbol)
        } else {
            "Memory %".to_string()
        },
        "NetTx/s".to_string(),
        "NetRx/s".to_string(),
        if sort_field == SortField::Uptime {
            format!("Created {}", sort_symbol)
        } else {
            "Created".to_string()
        },
    ]);

    Row::new(headers).style(styles.header).bottom_margin(1)
}

/// Creates the complete table widget
fn create_table<'a>(
    rows: Vec<Row<'a>>,
    header: Row<'static>,
    container_count: usize,
    styles: &UiStyles,
    show_host_column: bool,
    show_progress_bars: bool,
) -> Table<'a> {
    let mut constraints = vec![
        Constraint::Length(12), // Container ID
        Constraint::Length(1),  // Status icon
        Constraint::Min(8),     // Name (minimum 8, flexible)
    ];

    if show_host_column {
        constraints.push(Constraint::Length(20)); // Host
    }

    // Adjust column widths based on whether progress bars are shown
    let cpu_width = if show_progress_bars {
        30 // CPU sparkline (20 chars + 2 borders + " 100.0%")
    } else {
        7 // Just percentage (" 100.0%")
    };

    let mem_width = if show_progress_bars {
        35 // Memory sparkline (20 chars + 2 borders + " 999M/999M" + padding)
    } else {
        7 // Just percentage (" 100.0%")
    };

    constraints.extend(vec![
        Constraint::Length(cpu_width), // CPU
        Constraint::Length(mem_width), // Memory
        Constraint::Length(12),        // Network TX (1.23MB/s)
        Constraint::Length(12),        // Network RX (4.56MB/s)
        Constraint::Length(15),        // Created
    ]);

    // Build styled title: "datop" in purple, version in gray, count in yellow
    let title_left = Line::from(vec![
        Span::styled("datop", styles.title_name),
        Span::styled(format!(" v{}", VERSION), styles.title_help),
        Span::styled(" - ", styles.title_help),
        Span::styled(format!("{} containers", container_count), styles.title_count),
    ]);

    // Help text right-aligned in dark gray
    let title_right = Line::from(vec![
        Span::styled("'?' help, 'q' quit", styles.title_help),
    ])
    .right_aligned();

    Table::new(rows, constraints)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .padding(ratatui::widgets::Padding::proportional(1))
                .title_top(title_left)
                .title_top(title_right)
                .style(styles.border),
        )
        .row_highlight_style(styles.selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_memory_progress_bar_format() {
        let bar = create_memory_progress_bar(50.0, 512 * 1024 * 1024, 1024 * 1024 * 1024, 20);
        assert!(bar.contains("512 M/1 G"));
        assert!(bar.contains("██████████")); // 50% filled = 10 blocks
    }

    #[test]
    fn test_create_memory_progress_bar_zero() {
        let bar = create_memory_progress_bar(0.0, 0, 1024 * 1024 * 1024, 20);
        assert!(bar.contains("0 B/1 G"));
        assert!(bar.starts_with("░░░░░░░░░░░░░░░░░░░░")); // All empty
    }

    #[test]
    fn test_create_memory_progress_bar_full() {
        let bar = create_memory_progress_bar(100.0, 1024 * 1024 * 1024, 1024 * 1024 * 1024, 20);
        assert!(bar.contains("1 G/1 G"));
        assert!(bar.starts_with("████████████████████")); // All filled
    }

    #[test]
    fn test_create_memory_progress_bar_clamps_over_100() {
        // Bar visual should clamp at 100% even if percentage > 100
        let bar = create_memory_progress_bar(150.0, 1536 * 1024 * 1024, 1024 * 1024 * 1024, 20);
        assert!(bar.starts_with("████████████████████")); // Still fully filled
    }

    #[test]
    fn test_percentage_to_bar_index() {
        // Test boundary values for braille bar mapping
        assert_eq!(percentage_to_bar_index(0.0), 0, "0% should be empty");
        assert_eq!(percentage_to_bar_index(12.4), 0, "12.4% should be empty");
        assert_eq!(percentage_to_bar_index(12.5), 1, "12.5% should be 1 row");
        assert_eq!(percentage_to_bar_index(24.9), 1, "24.9% should be 1 row");
        assert_eq!(percentage_to_bar_index(25.0), 2, "25% should be 2 rows");
        assert_eq!(percentage_to_bar_index(49.9), 2, "49.9% should be 2 rows");
        assert_eq!(percentage_to_bar_index(50.0), 3, "50% should be 3 rows");
        assert_eq!(percentage_to_bar_index(74.9), 3, "74.9% should be 3 rows");
        assert_eq!(percentage_to_bar_index(75.0), 4, "75% should be full");
        assert_eq!(percentage_to_bar_index(100.0), 4, "100% should be full");
    }

    #[test]
    fn test_percentage_to_bar_index_clamps() {
        // Values outside 0-100 should be clamped
        assert_eq!(percentage_to_bar_index(-10.0), 0, "negative should clamp to 0");
        assert_eq!(percentage_to_bar_index(150.0), 4, "over 100 should clamp to full");
    }

    #[test]
    fn test_create_sparkline_empty_history() {
        let history = VecDeque::new();
        // With empty history, all positions are padding (no ticks in padding)
        let sparkline = create_sparkline(&history, 10, 0);
        // 10 content chars + 2 border chars = 12 total
        assert_eq!(sparkline.chars().count(), 12);
        let chars: Vec<char> = sparkline.chars().collect();
        // First and last are borders
        assert_eq!(chars[0], SPARKLINE_BORDER);
        assert_eq!(chars[11], SPARKLINE_BORDER);
        // Middle 10 chars are padding - no ticks
        for ch in &chars[1..11] {
            assert_eq!(*ch, BRAILLE_BARS[0]); // all empty, no ticks in padding
        }
    }

    #[test]
    fn test_create_sparkline_partial_history() {
        let mut history = VecDeque::new();
        history.push_back(80.0); // full bar
        history.push_back(30.0); // 2 rows

        // sample_count=2 means: history[0] is sample 0, history[1] is sample 1
        let sparkline = create_sparkline(&history, 5, 2);
        let chars: Vec<char> = sparkline.chars().collect();

        // 5 content chars + 2 border chars = 7 total
        assert_eq!(chars.len(), 7);
        // First and last are borders
        assert_eq!(chars[0], SPARKLINE_BORDER);
        assert_eq!(chars[6], SPARKLINE_BORDER);
        // Positions 1-3 should be padding (no ticks in padding)
        assert_eq!(chars[1], BRAILLE_BARS[0]);
        assert_eq!(chars[2], BRAILLE_BARS[0]);
        assert_eq!(chars[3], BRAILLE_BARS[0]);
        // Positions 4-5 should be from history
        // history[0] = sample 0 (tick), history[1] = sample 1 (no tick)
        assert_eq!(chars[4], BRAILLE_BARS_WITH_TICK[4]); // 80% with tick (sample 0)
        assert_eq!(chars[5], BRAILLE_BARS[2]); // 30% = 2 rows (sample 1, no tick)
    }

    #[test]
    fn test_create_sparkline_full_history() {
        let mut history = VecDeque::new();
        for i in 0..5 {
            history.push_back(i as f64 * 25.0); // 0, 25, 50, 75, 100
        }

        // sample_count=5: samples are 0,1,2,3,4 - tick only at sample 0
        let sparkline = create_sparkline(&history, 5, 5);
        let chars: Vec<char> = sparkline.chars().collect();

        // 5 content chars + 2 border chars = 7 total
        assert_eq!(chars.len(), 7);
        // First and last are borders
        assert_eq!(chars[0], SPARKLINE_BORDER);
        assert_eq!(chars[6], SPARKLINE_BORDER);
        // Content at positions 1-5
        assert_eq!(chars[1], BRAILLE_BARS_WITH_TICK[0]); // 0% with tick (sample 0)
        assert_eq!(chars[2], BRAILLE_BARS[2]); // 25%
        assert_eq!(chars[3], BRAILLE_BARS[3]); // 50%
        assert_eq!(chars[4], BRAILLE_BARS[4]); // 75%
        assert_eq!(chars[5], BRAILLE_BARS[4]); // 100%
    }

    #[test]
    fn test_create_sparkline_tick_on_filled_bar() {
        let mut history = VecDeque::new();
        // Fill with 60% values (3 rows)
        for _ in 0..10 {
            history.push_back(60.0);
        }

        // sample_count=10: samples 0-9, ticks at 0 and 5
        let sparkline = create_sparkline(&history, 10, 10);
        let chars: Vec<char> = sparkline.chars().collect();

        // 10 content chars + 2 border chars = 12 total
        assert_eq!(chars.len(), 12);
        // First and last are borders
        assert_eq!(chars[0], SPARKLINE_BORDER);
        assert_eq!(chars[11], SPARKLINE_BORDER);
        // Position 1 and 6 (content positions 0 and 5) should have tick-marked bars (with hole)
        assert_eq!(chars[1], BRAILLE_BARS_WITH_TICK[3]); // 60% with hole = ⢶ (sample 0)
        assert_eq!(chars[6], BRAILLE_BARS_WITH_TICK[3]); // 60% with hole = ⢶ (sample 5)
        // Other positions should be regular bars
        assert_eq!(chars[2], BRAILLE_BARS[3]); // 60% full = ⣶
        assert_eq!(chars[3], BRAILLE_BARS[3]); // 60% full = ⣶
        // Verify the hole character is different from the full bar
        assert_ne!(chars[1], chars[2]); // tick position differs from non-tick
    }

    #[test]
    fn test_create_cpu_sparkline_format() {
        let mut history = VecDeque::new();
        history.push_back(50.0);
        history.push_back(75.0);

        let result = create_cpu_sparkline(&history, 42.5, 5, 2);
        assert!(result.contains("42.5%"));
        assert_eq!(result.chars().filter(|c| *c == '%').count(), 1);
    }

    #[test]
    fn test_create_memory_sparkline_format() {
        let mut history = VecDeque::new();
        history.push_back(50.0);

        let result = create_memory_sparkline(&history, 512 * 1024 * 1024, 1024 * 1024 * 1024, 5, 1);
        assert!(result.contains("512 M/1 G"));
    }

    #[test]
    fn test_sparkline_marching_ticks() {
        // Simulate history buffer that marches as new samples come in
        let mut history = VecDeque::new();
        for _ in 0..5 {
            history.push_back(50.0); // 3 rows
        }

        // At sample_count=5: samples 0-4, tick at position 0
        // chars[0] and chars[6] are borders, content at chars[1..6]
        let sparkline1 = create_sparkline(&history, 5, 5);
        let chars1: Vec<char> = sparkline1.chars().collect();
        assert_eq!(chars1[0], SPARKLINE_BORDER);
        assert_eq!(chars1[1], BRAILLE_BARS_WITH_TICK[3]); // tick at sample 0
        assert_eq!(chars1[6], SPARKLINE_BORDER);

        // At sample_count=6: samples 1-5, tick at position 4 (sample 5)
        let sparkline2 = create_sparkline(&history, 5, 6);
        let chars2: Vec<char> = sparkline2.chars().collect();
        assert_eq!(chars2[1], BRAILLE_BARS[3]); // no tick at sample 1
        assert_eq!(chars2[5], BRAILLE_BARS_WITH_TICK[3]); // tick at sample 5

        // At sample_count=10: samples 5-9, tick at position 0 (sample 5)
        let sparkline3 = create_sparkline(&history, 5, 10);
        let chars3: Vec<char> = sparkline3.chars().collect();
        assert_eq!(chars3[1], BRAILLE_BARS_WITH_TICK[3]); // tick at sample 5
        assert_eq!(chars3[5], BRAILLE_BARS[3]); // no tick at sample 9
    }

    #[test]
    fn test_percentage_style_thresholds() {
        let styles = UiStyles::default();

        // Test low threshold (green)
        let low_style = get_percentage_style(30.0, &styles);
        assert_eq!(low_style.fg, Some(Color::Green));

        // Test medium threshold (yellow)
        let medium_style = get_percentage_style(65.0, &styles);
        assert_eq!(medium_style.fg, Some(Color::Yellow));

        // Test high threshold (red)
        let high_style = get_percentage_style(85.0, &styles);
        assert_eq!(high_style.fg, Some(Color::Red));

        // Test boundary cases
        assert_eq!(get_percentage_style(50.0, &styles).fg, Some(Color::Green));
        assert_eq!(get_percentage_style(50.1, &styles).fg, Some(Color::Yellow));
        assert_eq!(get_percentage_style(80.0, &styles).fg, Some(Color::Yellow));
        assert_eq!(get_percentage_style(80.1, &styles).fg, Some(Color::Red));
    }

    #[test]
    fn test_color_coding_boundaries() {
        let styles = UiStyles::default();

        // Test exact boundary values
        assert_eq!(
            get_percentage_style(0.0, &styles).fg,
            Some(Color::Green),
            "0% should be green"
        );
        assert_eq!(
            get_percentage_style(50.0, &styles).fg,
            Some(Color::Green),
            "50% should be green"
        );
        assert_eq!(
            get_percentage_style(50.1, &styles).fg,
            Some(Color::Yellow),
            "50.1% should be yellow"
        );
        assert_eq!(
            get_percentage_style(80.0, &styles).fg,
            Some(Color::Yellow),
            "80% should be yellow"
        );
        assert_eq!(
            get_percentage_style(80.1, &styles).fg,
            Some(Color::Red),
            "80.1% should be red"
        );
        assert_eq!(
            get_percentage_style(100.0, &styles).fg,
            Some(Color::Red),
            "100% should be red"
        );
    }
}
