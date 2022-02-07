use std::io::Stdout;
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Row, Table};
use crate::BurtGang;

pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Home,
    Burts,
    Log,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> Self {
        match input {
            MenuItem::Home => 0,
            MenuItem::Burts => 1,
            MenuItem::Log => 2,
        }
    }
}

pub fn draw_home(rect: &mut Frame<CrosstermBackend<Stdout>>, chunks: &Vec<Rect>, burt_gang: &mut BurtGang) {
    let home_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [Constraint::Percentage(70), Constraint::Percentage(30)].as_ref(),
        )
        .split(chunks[1]);

    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "MaLB",
            Style::default().fg(Color::LightYellow),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'h' for Home, 'b' for Burts, 'l' for Logs, 't' to run a command, and 'q' for Quit")]),
    ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Home")
                .border_type(BorderType::Plain),
        );

    let home_details = Table::new(vec![Row::new(vec![
        Span::raw(format!("{}", burt_gang.target)), // target
        Span::raw(format!("{}", burt_gang.range)), // range
        Span::raw(format!("{} / {}", burt_gang.current_generation, burt_gang.generations)), // generation
        Span::raw(format!("{}", burt_gang.survival_rate)), // survival rate
        Span::raw(format!("{}", burt_gang.mutation_rate)), // mutation rate
        Span::raw(format!("{}", burt_gang.len())), // burt count
        Span::raw(format!("{}", burt_gang.av_guess_display())), // average guess
        Span::raw(format!("{}", burt_gang.av_score_display())), // average score
    ])])
        .header(Row::new(vec![
            Span::styled(
                "Target",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Range",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Generation",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Survival Rate",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Mutation Rate",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Burt Count",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Average Guess",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Average Score",
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Details")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]);
    rect.render_widget(home, home_chunks[0]);
    rect.render_widget(home_details, home_chunks[1]);
}

pub fn draw_burts(rect: &mut Frame<CrosstermBackend<Stdout>>, chunks: &Vec<Rect>, burt_gang: &BurtGang, burt_list_state: &mut ListState) {
    let burts_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
        )
        .split(chunks[1]);
    let burts = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Burts")
        .border_type(BorderType::Plain);

    let burt_list = &burt_gang;
    let items: Vec<_> = burt_list
        .iter()
        .map(|burt| {
            ListItem::new(Spans::from(vec![Span::styled(
                format!("Burt #{}", burt.get_id().clone()),
                Style::default(),
            )]))
        })
        .collect();

    let selected_burt = burt_list
        .get(
            burt_list_state
                .selected()
                .expect("This shouldn't be reachable, as there is always a selected burt!"),
        )
        .clone();

    let burts_list_left = List::new(items).block(burts).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let burt_detail = Table::new(vec![
        Row::new(vec![
            //Span::raw(format!("Burt #{}", selected_burt.get_id())), // id - moved to title
            Span::raw(format!("{}", selected_burt.get_score_display())), // score
            Span::raw(format!("{}", selected_burt.get_guess_display())), // guess
            Span::raw(format!("{}", selected_burt.get_mu())), // mu
            Span::raw(format!("{}", selected_burt.get_sigma())), // sigma
        ])])
        .header(Row::new(vec![
            Span::styled(
                "Score",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Guess",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Mu",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Sigma",
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title(format!("Burt #{}", selected_burt.get_id()))
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ]);
    rect.render_stateful_widget(burts_list_left, burts_chunks[0], burt_list_state);
    rect.render_widget(burt_detail, burts_chunks[1]);
}