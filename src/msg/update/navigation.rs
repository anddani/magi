use crate::{
    model::Model,
    msg::{Message, NavigationAction, util::visible_lines_between},
};

pub fn update(model: &mut Model, action: NavigationAction) -> Option<Message> {
    match action {
        NavigationAction::MoveUp => move_up(model),
        NavigationAction::MoveDown => move_down(model),
        NavigationAction::HalfPageUp => half_page_up(model),
        NavigationAction::HalfPageDown => half_page_down(model),
        NavigationAction::ScrollLineDown => scroll_line_down(model),
        NavigationAction::ScrollLineUp => scroll_line_up(model),
        NavigationAction::MoveToTop => move_to_top(model),
        NavigationAction::MoveToBottom => move_to_bottom(model),
    }
}

fn move_up(model: &mut Model) -> Option<Message> {
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos > 0 {
        new_pos -= 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = new_pos;
            // Scroll up if cursor moves above viewport
            if model.ui_model.cursor_position < model.ui_model.scroll_offset {
                model.ui_model.scroll_offset = model.ui_model.cursor_position;
            }
            break;
        }
    }
    None
}

fn move_down(model: &mut Model) -> Option<Message> {
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos < max_pos {
        new_pos += 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = new_pos;
            // Scroll down if cursor moves below viewport
            let viewport_height = model.ui_model.viewport_height;
            if viewport_height > 0 {
                // Count visible lines from scroll_offset to cursor (exclusive)
                let visible_before_cursor = visible_lines_between(
                    &model.ui_model.lines,
                    model.ui_model.scroll_offset,
                    model.ui_model.cursor_position,
                    &model.ui_model.collapsed_sections,
                );
                // If visible lines before cursor >= viewport_height, cursor is below viewport
                if visible_before_cursor >= viewport_height {
                    // Scroll so cursor is at bottom of viewport
                    // Walk back from cursor to find scroll position with (viewport_height - 1)
                    // visible lines before cursor
                    let mut new_scroll = model.ui_model.cursor_position;
                    let mut visible_count = 0;
                    while new_scroll > 0 && visible_count < viewport_height - 1 {
                        new_scroll -= 1;
                        if !model.ui_model.lines[new_scroll]
                            .is_hidden(&model.ui_model.collapsed_sections)
                        {
                            visible_count += 1;
                        }
                    }
                    model.ui_model.scroll_offset = new_scroll;
                }
            }
            break;
        }
    }
    None
}

fn half_page_up(model: &mut Model) -> Option<Message> {
    let half_page = model.ui_model.viewport_height / 2;
    // Move up by half_page visible lines
    let mut visible_count = 0;
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos > 0 && visible_count < half_page {
        new_pos -= 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            visible_count += 1;
        }
    }
    // Make sure we land on a visible line (try backward first, then forward)
    while new_pos > 0 && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos -= 1;
    }
    // If still on a hidden line (reached beginning), search forward
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    while new_pos < max_pos
        && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos += 1;
    }
    model.ui_model.cursor_position = new_pos;
    // Scroll up if cursor moves above viewport
    if model.ui_model.cursor_position < model.ui_model.scroll_offset {
        model.ui_model.scroll_offset = model.ui_model.cursor_position;
    }
    None
}

fn half_page_down(model: &mut Model) -> Option<Message> {
    let half_page = model.ui_model.viewport_height / 2;
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    // Move down by half_page visible lines
    let mut visible_count = 0;
    let mut new_pos = model.ui_model.cursor_position;
    while new_pos < max_pos && visible_count < half_page {
        new_pos += 1;
        if !model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections) {
            visible_count += 1;
        }
    }
    // Make sure we land on a visible line (try forward first, then backward)
    while new_pos < max_pos
        && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos += 1;
    }
    // If still on a hidden line (reached end), search backward
    while new_pos > 0 && model.ui_model.lines[new_pos].is_hidden(&model.ui_model.collapsed_sections)
    {
        new_pos -= 1;
    }
    model.ui_model.cursor_position = new_pos;
    // Scroll down if cursor moves below viewport
    let viewport_height = model.ui_model.viewport_height;
    if viewport_height > 0 {
        let visible_before_cursor = visible_lines_between(
            &model.ui_model.lines,
            model.ui_model.scroll_offset,
            model.ui_model.cursor_position,
            &model.ui_model.collapsed_sections,
        );
        if visible_before_cursor >= viewport_height {
            let mut new_scroll = model.ui_model.cursor_position;
            let mut scroll_visible_count = 0;
            while new_scroll > 0 && scroll_visible_count < viewport_height - 1 {
                new_scroll -= 1;
                if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
                    scroll_visible_count += 1;
                }
            }
            model.ui_model.scroll_offset = new_scroll;
        }
    }
    None
}

fn scroll_line_down(model: &mut Model) -> Option<Message> {
    // Scroll viewport down by one visible line (C-e in Vim)
    let max_pos = model.ui_model.lines.len().saturating_sub(1);
    let viewport_height = model.ui_model.viewport_height;
    if viewport_height == 0 {
        return None;
    }

    // Find the next visible line after current scroll_offset
    let mut new_scroll = model.ui_model.scroll_offset;
    while new_scroll < max_pos {
        new_scroll += 1;
        if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
            break;
        }
    }

    // Only scroll if there's content below to show
    if new_scroll <= max_pos
        && !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections)
    {
        model.ui_model.scroll_offset = new_scroll;

        // If cursor is now above viewport, move it to the top visible line
        if model.ui_model.cursor_position < model.ui_model.scroll_offset {
            model.ui_model.cursor_position = model.ui_model.scroll_offset;
        }
    }
    None
}

fn scroll_line_up(model: &mut Model) -> Option<Message> {
    // Scroll viewport up by one visible line (C-y in Vim)
    if model.ui_model.scroll_offset == 0 {
        return None;
    }

    let viewport_height = model.ui_model.viewport_height;
    if viewport_height == 0 {
        return None;
    }

    // Find the previous visible line before current scroll_offset
    let mut new_scroll = model.ui_model.scroll_offset;
    while new_scroll > 0 {
        new_scroll -= 1;
        if !model.ui_model.lines[new_scroll].is_hidden(&model.ui_model.collapsed_sections) {
            break;
        }
    }

    model.ui_model.scroll_offset = new_scroll;

    // If cursor is now below viewport, move it to the bottom visible line
    let visible_before_cursor = visible_lines_between(
        &model.ui_model.lines,
        model.ui_model.scroll_offset,
        model.ui_model.cursor_position,
        &model.ui_model.collapsed_sections,
    );

    if visible_before_cursor >= viewport_height {
        // Find the last visible line in the viewport
        let mut new_cursor = model.ui_model.scroll_offset;
        let mut visible_count = 0;
        let max_pos = model.ui_model.lines.len().saturating_sub(1);
        while new_cursor < max_pos && visible_count < viewport_height - 1 {
            new_cursor += 1;
            if !model.ui_model.lines[new_cursor].is_hidden(&model.ui_model.collapsed_sections) {
                visible_count += 1;
            }
        }
        // Make sure we're on a visible line
        while new_cursor > 0
            && model.ui_model.lines[new_cursor].is_hidden(&model.ui_model.collapsed_sections)
        {
            new_cursor -= 1;
        }
        model.ui_model.cursor_position = new_cursor;
    }
    None
}

fn move_to_top(model: &mut Model) -> Option<Message> {
    for (i, line) in model.ui_model.lines.iter().enumerate() {
        if !line.is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = i;
            model.ui_model.scroll_offset = i;
            break;
        }
    }
    None
}

fn move_to_bottom(model: &mut Model) -> Option<Message> {
    for i in (0..model.ui_model.lines.len()).rev() {
        if !model.ui_model.lines[i].is_hidden(&model.ui_model.collapsed_sections) {
            model.ui_model.cursor_position = i;

            // Adjust scroll so cursor is visible at bottom of viewport
            let viewport_height = model.ui_model.viewport_height;
            if viewport_height > 0 {
                let visible_before_cursor = visible_lines_between(
                    &model.ui_model.lines,
                    model.ui_model.scroll_offset,
                    model.ui_model.cursor_position,
                    &model.ui_model.collapsed_sections,
                );
                if visible_before_cursor >= viewport_height {
                    let mut new_scroll = model.ui_model.cursor_position;
                    let mut visible_count = 0;
                    while new_scroll > 0 && visible_count < viewport_height - 1 {
                        new_scroll -= 1;
                        if !model.ui_model.lines[new_scroll]
                            .is_hidden(&model.ui_model.collapsed_sections)
                        {
                            visible_count += 1;
                        }
                    }
                    model.ui_model.scroll_offset = new_scroll;
                }
            }
            break;
        }
    }
    None
}
