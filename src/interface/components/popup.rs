use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders},
};

type RenderFn<'a> = Box<dyn Fn(Rect, &mut Buffer) + 'a>;

pub struct WidgetItem<'a> {
    render_fn: RenderFn<'a>,
    height: u16,
    width: u16,
}

impl<'a> WidgetItem<'a> {
    pub fn new<F>(height: u16, width: u16, render_fn: F) -> Self
    where
        F: Fn(Rect, &mut Buffer) + 'a,
    {
        Self {
            render_fn: Box::new(render_fn),
            height,
            width,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        (self.render_fn)(area, buf);
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn width(&self) -> u16 {
        self.width
    }
}

pub struct PopUp<'a> {
    title: Option<String>,
    widgets: Vec<WidgetItem<'a>>,
}

impl<'a> PopUp<'a> {
    pub fn new(title: Option<String>, widgets: Vec<WidgetItem<'a>>) -> Self {
        Self { title, widgets }
    }

    pub fn render(&self, frame: &mut Frame) {
        let content_height: u16 = self.widgets.iter().map(|w| w.height()).sum();
        let content_width: u16 = self.widgets.iter().map(|w| w.width()).max().unwrap_or(20);

        let total_height = content_height + 2;
        let total_width = content_width + 2;

        let title_width = self.title.as_ref().map(|t| t.len() as u16 + 4).unwrap_or(0);
        let total_width = total_width.max(title_width);

        let area = Self::center(
            frame.area(),
            Constraint::Length(total_width),
            Constraint::Length(total_height),
        );

        let block = if let Some(ref title) = self.title {
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(title.as_str()).centered())
                .style(Style::default().fg(Color::White))
        } else {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
        };

        frame.render_widget(&block, area);

        let inner_area = block.inner(area);
        let constraints: Vec<Constraint> = self
            .widgets
            .iter()
            .map(|w| Constraint::Length(w.height()))
            .collect();

        if !constraints.is_empty() {
            let widget_areas = Layout::vertical(constraints).split(inner_area);
            for (widget, &widget_area) in self.widgets.iter().zip(widget_areas.iter()) {
                widget.render(widget_area, frame.buffer_mut());
            }
        }
    }

    fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
        let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
        let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
        area
    }
}
