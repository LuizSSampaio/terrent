pub mod components;

use std::time::Duration;

use components::confirmation_popup::ConfirmationMessage;
use components::{ConfirmationPopup, ConfirmationResult};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Frame, widgets::Paragraph};

#[derive(Debug, Clone)]
struct Model {
    running_state: RunningState,
    exit_confirmation: ConfirmationPopup,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            running_state: RunningState::default(),
            exit_confirmation: ConfirmationPopup::new(
                "Confirm Exit",
                "Are you sure you want to quit?",
            ),
        }
    }
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
    ShowExitConfirmation,
    ExitConfirmation(ConfirmationMessage),
}

pub fn init() {
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        let _ = terminal.draw(|f| view(&mut model, f)).unwrap();

        let mut message = handle_event(&mut model);

        while message.is_some() {
            message = update(&mut model, message.unwrap());
        }
    }

    ratatui::restore();
}

fn view(model: &mut Model, frame: &mut Frame) {
    let main_text = "Terrent";
    frame.render_widget(Paragraph::new(main_text), frame.area());

    model.exit_confirmation.render(frame, frame.area());
}

fn handle_event(model: &mut Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).unwrap()
        && let Event::Key(key) = event::read().unwrap()
        && key.kind == event::KeyEventKind::Press
    {
        return handle_key(key, model);
    }
    None
}

fn handle_key(key: event::KeyEvent, model: &mut Model) -> Option<Message> {
    if model.exit_confirmation.is_visible() {
        if let Some(msg) = model.exit_confirmation.handle_key(key) {
            return Some(Message::ExitConfirmation(msg));
        }
        return None;
    }

    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Message::ShowExitConfirmation)
        }
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => model.running_state = RunningState::Done,
        Message::ShowExitConfirmation => {
            model.exit_confirmation.show();
        }
        Message::ExitConfirmation(confirmation_msg) => {
            if let Some(result) = model.exit_confirmation.update(confirmation_msg) {
                match result {
                    ConfirmationResult::Yes => model.running_state = RunningState::Done,
                    ConfirmationResult::No | ConfirmationResult::Cancelled => {
                        model.exit_confirmation.hide();
                    }
                }
            }
        }
    }
    None
}
