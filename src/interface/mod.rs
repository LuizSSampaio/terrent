use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{Frame, text::Line, widgets::Paragraph};
use tui_widgets::popup::Popup;

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
    ClearExitPopup,
}

pub fn init() {
    let mut terminal = ratatui::init();
    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        let _ = terminal.draw(|f| view(&mut model, f)).unwrap();

        let mut message = handle_event(&model);

        while message.is_some() {
            message = update(&mut model, message.unwrap());
        }
    }

    ratatui::restore();
}

fn view(model: &mut Model, frame: &mut Frame) {
    let main_text = "Terrent";
    frame.render_widget(Paragraph::new(main_text), frame.area());

    if model.show_exit_message {
        let popup = Popup::new("Press Ctrl+C again to quit, or any other key to continue.")
            .title(Line::from("Confirm Exit").centered());
        frame.render_widget(&popup, frame.area());
    }
}

fn handle_event(model: &Model) -> Option<Message> {
    if model.show_exit_message
        && let Some(last_ctrl_c) = model.last_ctrl_c
        && last_ctrl_c.elapsed() > Duration::from_secs(3)
    {
        return Some(Message::ClearExitPopup);
    }

    if event::poll(Duration::from_millis(250)).unwrap()
        && let Event::Key(key) = event::read().unwrap()
        && key.kind == event::KeyEventKind::Press
    {
        return handle_key(key, model);
    }
    None
}

fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') => Some(Message::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(last_ctrl_c) = model.last_ctrl_c
                && last_ctrl_c.elapsed() < Duration::from_secs(2)
            {
                return Some(Message::Quit);
            }
            Some(Message::CtrlC)
        }
        _ => {
            if model.show_exit_message {
                Some(Message::ClearExitPopup)
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
        }
        Message::ClearExitPopup => {
            model.show_exit_message = false;
            model.last_ctrl_c = None;
        }
    }
    None
}
