use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, WidgetRef},
};
use tui_widgets::popup::{Popup, SizedWidgetRef};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ConfirmationChoice {
    Yes,
    #[default]
    No,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationResult {
    Yes,
    No,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationMessage {
    Confirm,
    Cancel,
    ToggleChoice,
}

#[derive(Debug, Clone)]
pub struct ConfirmationPopup {
    title: String,
    content: String,
    yes_label: String,
    no_label: String,
    selected: ConfirmationChoice,
    visible: bool,
}

#[derive(Debug)]
struct ConfirmationBody<'a> {
    content: &'a str,
    yes_label: &'a str,
    no_label: &'a str,
    selected: &'a ConfirmationChoice,
}

impl ConfirmationPopup {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
            selected: ConfirmationChoice::default(),
            visible: false,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.selected = ConfirmationChoice::default();
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn update(&mut self, msg: ConfirmationMessage) -> Option<ConfirmationResult> {
        match msg {
            ConfirmationMessage::ToggleChoice => {
                self.selected = match self.selected {
                    ConfirmationChoice::Yes => ConfirmationChoice::No,
                    ConfirmationChoice::No => ConfirmationChoice::Yes,
                };
                None
            }
            ConfirmationMessage::Confirm => {
                self.visible = false;
                Some(match self.selected {
                    ConfirmationChoice::Yes => ConfirmationResult::Yes,
                    ConfirmationChoice::No => ConfirmationResult::No,
                })
            }
            ConfirmationMessage::Cancel => {
                self.visible = false;
                Some(ConfirmationResult::Cancelled)
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<ConfirmationMessage> {
        if !self.visible {
            return None;
        }

        match key.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l')
            | KeyCode::Tab => Some(ConfirmationMessage::ToggleChoice),
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.selected = ConfirmationChoice::Yes;
                Some(ConfirmationMessage::Confirm)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.selected = ConfirmationChoice::No;
                Some(ConfirmationMessage::Confirm)
            }
            KeyCode::Enter => Some(ConfirmationMessage::Confirm),
            KeyCode::Esc => Some(ConfirmationMessage::Cancel),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(ConfirmationMessage::Cancel)
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        let body = ConfirmationBody {
            content: &self.content,
            yes_label: &self.yes_label,
            no_label: &self.no_label,
            selected: &self.selected,
        };

        let popup = Popup::new(body)
            .title(Line::from(self.title.clone()).centered())
            .style(Style::default().bg(Color::Black));

        frame.render_widget(&popup, area);
    }
}

impl WidgetRef for ConfirmationBody<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Content area
                Constraint::Length(1), // Button area
                Constraint::Length(1), // Hint area
            ])
            .split(area);

        let content_paragraph = Paragraph::new(self.content)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        content_paragraph.render_ref(chunks[0], buf);

        let button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let yes_style = if *self.selected == ConfirmationChoice::Yes {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };

        let yes_text = if *self.selected == ConfirmationChoice::Yes {
            format!("[ {} ]", self.yes_label)
        } else {
            format!("  {}  ", self.yes_label)
        };

        let yes_button = Paragraph::new(yes_text)
            .alignment(Alignment::Center)
            .style(yes_style);
        yes_button.render_ref(button_chunks[0], buf);

        let no_style = if *self.selected == ConfirmationChoice::No {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };

        let no_text = if *self.selected == ConfirmationChoice::No {
            format!("[ {} ]", self.no_label)
        } else {
            format!("  {}  ", self.no_label)
        };

        let no_button = Paragraph::new(no_text)
            .alignment(Alignment::Center)
            .style(no_style);
        no_button.render_ref(button_chunks[1], buf);

        let hint = Line::from(vec![
            Span::styled("Arrow/Tab", Style::default().fg(Color::DarkGray)),
            Span::raw(": Navigate | "),
            Span::styled("Enter", Style::default().fg(Color::DarkGray)),
            Span::raw(": Confirm | "),
            Span::styled("Esc", Style::default().fg(Color::DarkGray)),
            Span::raw(": Cancel"),
        ])
        .centered();

        Paragraph::new(hint).render_ref(chunks[2], buf);
    }
}

impl SizedWidgetRef for ConfirmationBody<'_> {
    fn width(&self) -> usize {
        let content_width = self.content.len();
        let buttons_width = self.yes_label.len() + self.no_label.len() + 10;
        let min_width = 50;

        content_width.max(buttons_width).max(min_width)
    }

    fn height(&self) -> usize {
        // Content area (3) + Button area (1) + Hint area (1)
        5
    }
}
