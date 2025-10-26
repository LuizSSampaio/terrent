use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use ratatui::{Frame, widgets::Paragraph};

#[derive(Debug, Default, Clone)]
struct Model {
    running_state: RunningState,
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
    frame.render_widget(Paragraph::new("Terrent"), frame.area());
}

fn handle_event(_: &Model) -> Option<Message> {
    if event::poll(Duration::from_millis(250)).unwrap() {
        if let Event::Key(key) = event::read().unwrap() {
            if key.kind == event::KeyEventKind::Press {
                return handle_key(key);
            }
        }
    }
    None
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    match msg {
        Message::Quit => model.running_state = RunningState::Done,
    }
    None
}
