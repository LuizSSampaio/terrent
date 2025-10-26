use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

mod components;

use components::popup::{PopUp, WidgetItem};

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
        let message_lines = vec![
            Line::from(Span::styled(
                "Press Ctrl+C again to quit,",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "or any other key to continue.",
                Style::default().fg(Color::Gray),
            )),
        ];

        let widgets = vec![WidgetItem::new(2, 30, move |area, buf| {
            let paragraph = Paragraph::new(message_lines.clone());
            paragraph.render(area, buf);
        })];

        let popup = PopUp::new(Some("Confirm Exit".to_string()), widgets);
        popup.render(frame);
    }
}

fn handle_event(model: &Model) -> Option<Message> {
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
                if last_ctrl_c.elapsed() < Duration::from_secs(2) {
                    return Some(Message::Quit);
                }
            }
            Some(Message::CtrlC)
        }
        _ => {
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
        }
        Message::ClearExitMessage => {
            model.show_exit_message = false;
            model.last_ctrl_c = None;
        }
    }
    None
}
