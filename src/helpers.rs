use std::fs::create_dir_all;

use anyhow::{Context, Result};
use ratatui::{layout::Flex, prelude::*};
use tracing_subscriber::{EnvFilter, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center);

    let horizontal = Layout::horizontal([Constraint::Length(width)]).flex(Flex::Center);

    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

pub fn calc_height(msg: &str, width: u16, area: Rect, footer: bool) -> u16 {
    let available_width = width.saturating_sub(4) as usize; // Account for borders and padding
    let mut total_lines = 0;

    for line in msg.lines() {
        if line.is_empty() {
            total_lines += 1;
        } else {
            // Calculate wrapped lines for this text line
            let chars = line.chars().count();
            let wrapped_lines = (chars / available_width).max(1);
            total_lines += wrapped_lines;
        }
    }

    // Add lines for footer message
    if footer {
        total_lines += 2; // "Press ESC to clear error" + empty line
    }

    // Add padding for borders
    (total_lines as u16)
        .saturating_add(4)
        .min(area.height.saturating_sub(4))
}

pub fn init_log() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let app_name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "rust-browser".to_string());
    let cache_dir = dirs::cache_dir().unwrap_or_default();
    let log_dir = cache_dir.join(&app_name).join("logs");
    create_dir_all(&log_dir)?;

    let appender = tracing_appender::rolling::daily(log_dir, app_name);
    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .context("Failed to create filter")?;

    tracing_subscriber::registry()
        .with(filter)
        .with(layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Ok(guard)
}

/// Helper function for handling text rendering (Vec<Line> line wrappin)
pub fn parse_text<'l>(lines: &mut Vec<Line<'l>>, text: String, width: usize) {
    if !text.is_empty() {
        for line in text.lines() {
            if line.len() <= width {
                lines.push(Line::from(line.to_string()));
            } else {
                let words: Vec<&str> = line.split_whitespace().collect();
                let mut curr_line = String::new();

                for word in words {
                    if curr_line.len() + word.len() < width {
                        if !curr_line.is_empty() {
                            curr_line.push(' ');
                        }
                        curr_line.push_str(word);
                    } else {
                        if !curr_line.is_empty() {
                            lines.push(Line::from(curr_line.clone()));
                        }
                        curr_line = word.to_string();
                    }
                }
                if !curr_line.is_empty() {
                    lines.push(Line::from(curr_line));
                }
            }
        }
    }

    lines.push(Line::from(""));
}
