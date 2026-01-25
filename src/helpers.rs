use ratatui::{layout::Flex, prelude::*};

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
