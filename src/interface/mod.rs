use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Frame, widgets::Paragraph};

#[derive(Debug, Default, Clone)]
struct Model {
    running_state: RunningState,
    last_ctrl_c: Option<Instant>,
    show_exit_message: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq, Eq)]
enum Message {
    Quit,
    CtrlC,
    ClearExitMessage,
}

pub fn init() {
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&mut model, f));

        let mut message = handle_event(&model);

        while message.is_some() {
            message = update(&mut model, message.unwrap());
        }
    }

    ratatui::restore();
}

fn view(model: &mut Model, frame: &mut Frame) {
    let text = if model.show_exit_message {
        "Terrent\n\nPress Ctrl+C again to quit, or any other key to continue."
    } else {
        "Terrent"
    };
    frame.render_widget(Paragraph::new(text), frame.area());
}

fn handle_event(model: &Model) -> Option<Message> {
    // Check if we should clear the exit message due to timeout
    if model.show_exit_message {
        if let Some(last_ctrl_c) = model.last_ctrl_c {
            if last_ctrl_c.elapsed() > Duration::from_secs(3) {
                return Some(Message::ClearExitMessage);
            }
        }
    }

    if event::poll(Duration::from_millis(250)).unwrap() {
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == event::KeyEventKind::Press {
                return handle_key(key, model);
            }
        }
    }
    None
}

fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(last_ctrl_c) = model.last_ctrl_c {
                // If Ctrl+C was pressed within the last 2 seconds, quit
                if last_ctrl_c.elapsed() < Duration::from_secs(2) {
                    return Some(Message::Quit);
                }
            }
            // First Ctrl+C or too much time passed, show warning
            Some(Message::CtrlC)
        },
        _ => {
            // Any other key clears the exit message
            if model.show_exit_message {
                Some(Message::ClearExitMessage)
            } else {
                None
            }
        }
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => model.running_state = RunningState::Done,
        Message::CtrlC => {
            model.last_ctrl_c = Some(Instant::now());
            model.show_exit_message = true;
        },
        Message::ClearExitMessage => {
            model.show_exit_message = false;
            model.last_ctrl_c = None;
        },
    }
    None
}
