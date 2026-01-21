use crate::{
    keys::handle_key,
    msg::{update::update, util::is_external_command},
};
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::{
    config::Config,
    errors::MagiResult,
    git::GitInfo,
    model::{Model, RunningState, UiModel},
    msg::Message,
    view::view,
};

const EVENT_POLL_TIMEOUT_MILLIS: u64 = 250;

pub fn run() -> MagiResult<()> {
    let terminal = ratatui::init();
    let result = run_loop(terminal);
    ratatui::restore();
    result
}

/// Main run loop which polls events (messages), transforms the model,
/// and renders the UI.
fn run_loop(mut terminal: DefaultTerminal) -> MagiResult<()> {
    // Load config and resolve theme
    let config = Config::load();
    let theme = config.resolve_theme();

    let git_info = GitInfo::new()?;
    let lines = git_info.get_lines()?;
    let collapsed_sections = lines
        .iter()
        .filter_map(|line| line.section.clone())
        .filter(|line| line.default_collapsed())
        .collect::<HashSet<_>>();
    let initial_ui_model = UiModel {
        lines,
        cursor_position: 0,
        scroll_offset: 0,
        viewport_height: 0,
        collapsed_sections,
        visual_mode_anchor: None,
        search_query: String::new(),
        search_mode_active: false,
    };

    let mut model = Model {
        git_info,
        running_state: RunningState::Running,
        ui_model: initial_ui_model,
        theme,
        popup: None,
        toast: None,
        select_result: None,
    };

    while model.running_state != RunningState::Done {
        // Check if toast has expired and clear it
        if let Some(ref toast) = model.toast {
            if Instant::now() >= toast.expires_at {
                model.toast = None;
            }
        }

        // Handle special states that need terminal control
        if let RunningState::LaunchExternalCommand(msg) = model.running_state {
            model.running_state = RunningState::Running;

            // Suspend TUI to allow external command to be run without
            // Ratatui being rendered.
            ratatui::restore();

            // Process external command (blocking)
            let mut current_msg = update(&mut model, msg);

            // Resume TUI
            terminal = ratatui::init();

            // Process the commit result message(s)
            while let Some(m) = current_msg {
                current_msg = update(&mut model, m);
            }
            continue;
        }

        // Update viewport height for scrolling calculations (subtract 2 for borders)
        let terminal_height = terminal.size()?.height as usize;
        model.ui_model.viewport_height = terminal_height.saturating_sub(2);

        // Render view
        terminal.draw(|f| view(&model, f))?;

        // Handle event
        let mut current_msg = handle_event(&model)?;

        // If the message is an external command, we want to update the running state and skip the
        // update processing below in order to pause Ratatui rendering.
        if let Some(msg) = current_msg.take_if(|msg| is_external_command(msg)) {
            model.running_state = RunningState::LaunchExternalCommand(msg);
            continue;
        }

        // Process updates
        while let Some(msg) = current_msg {
            current_msg = update(&mut model, msg);
        }
    }
    Ok(())
}

/// Blocks for [`EVENT_POLL_TIMEOUT_MILLIS`] waiting for a key event.
/// If a key event occurred during this time, return what [`Message`]
/// it should trigger.
fn handle_event(model: &Model) -> MagiResult<Option<Message>> {
    if event::poll(Duration::from_millis(EVENT_POLL_TIMEOUT_MILLIS))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, model));
            }
        }
    }
    Ok(None)
}
