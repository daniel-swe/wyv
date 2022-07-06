use std::{io::Stdout, path::Path};

use anyhow::Result;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{List, ListItem, Paragraph, Tabs, Wrap, Block, Borders},
    Terminal,
};

use crate::widgets::file_tree::{FileTree, FileTreeState};

pub fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.draw(|f| {
        let mut cut_size = f.size();
        cut_size.height -= 1;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)].as_ref())
            .split(cut_size);

        let titles = ["Tab1", "Tab2", "Tab3", "Tab4"]
            .iter()
            .cloned()
            .map(Spans::from)
            .collect();

        let file_tree = FileTree::new(&Path::new(".")).unwrap();
        f.render_stateful_widget(file_tree, chunks[0], &mut FileTreeState::default());

        let tabs = Tabs::new(titles)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::LightBlue))
            .divider(tui::symbols::line::VERTICAL);
        f.render_widget(tabs, chunks[1]);

        let text = vec![Spans::from(Span::styled(
            "Second line",
            Style::default().fg(Color::Red),
        ))];
        let bar = Paragraph::new(text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(
            bar,
            tui::layout::Rect {
                x: cut_size.x,
                y: cut_size.height,
                width: cut_size.width,
                height: 1,
            },
        )
    })?;

    Ok(())
}
