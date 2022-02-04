use std::{env, thread};
use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::{event, execute, terminal};
use crossterm::cursor::MoveTo;
use crossterm::terminal::{ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::event::{Event as CEvent, KeyCode};
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Row, Table, Tabs};
use crate::burt::{BurtGang, get_burt_gang, populate_burts};
use crate::ui::{Event, MenuItem};

pub(crate) mod input;
mod ui;
mod burt;

fn main() {

    // get arguments
    let args: Vec<String> = env::args().collect();

    // clear the screen and set terminal position
    execute!(stdout(), terminal::Clear(ClearType::All), MoveTo(0,0)).expect("Failed to clear screen! Is this terminal supported?");

    // initialize the burts
    let burt_gang = if args.contains(&"-d".to_string()) || args.contains(&"--default".to_string()) {
        BurtGang::new(populate_burts(10000), 100, 7, 150, 0.5, 0.25)
    } else {
        get_burt_gang()
    };

    println!("{}", &burt_gang);

    thread::sleep(Duration::from_millis(3000));

    // enable terminal raw mode and set up the terminal
    enable_raw_mode().expect("Failed to enable raw mode; is this terminal supported?");

    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, event::EnableMouseCapture).expect("Failed to setup terminal; Is this terminal supported?");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to setup terminal; Is this terminal supported?");
    terminal.clear().expect("Failed to clear the terminal");

    // Initialize the event loop for the UI
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    thread::spawn(move || {
        let mut last_tick = Instant::now();

        loop {
            let timeout = tick_rate.checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("Failed to poll events") {
                if let CEvent::Key(key) = event::read().expect("Failed to read events.") {
                    tx.send(Event::Input(key)).expect("Failed to send event to main thread");
                }
                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).expect("Failed to send tick update");
                    last_tick = Instant::now();
                }
            }
        }
    });

    // render loop variables
    let menu_titles = vec!["Home", "Burts", "Log", "Quit"];
    let mut active_menu_item = MenuItem::Home;

    let mut burt_list_state = ListState::default();
    burt_list_state.select(Some(0));

    let mut input_mode = false;
    let mut user_input = String::new();
    let mut input_ready = false;
    let mut input_mode_prompt = format!("Input");

    let footer_txt = format!("MaLB v{} 2022 created and maintained by Eric Shreve and Ben Snedeker", env!("CARGO_PKG_VERSION"));

    // start the render loop
    loop {
        terminal.draw(|rect| {

            let footer = Paragraph::new(footer_txt.clone())
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Info")
                        .border_type(BorderType::Plain)
                );
            // setup the layout
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                ].as_ref())
                .split(size);

            // create the menu bar
            let menu: Vec<Spans> = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED)
                        ),
                        Span::styled(rest, Style::default().fg(Color::White))
                    ])
                }).collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);

            // handle the main page
            match active_menu_item {
                MenuItem::Home => {
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
                        Spans::from(vec![Span::raw("Press 'h' for Home, 'b' for Burts, 'l' for Logs, and 'q' for Quit")]),
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
                        ]);
                    rect.render_widget(home, home_chunks[0]);
                    rect.render_widget(home_details, home_chunks[1]);
                }
                MenuItem::Burts => {
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
                                .expect("there is always a selected burt"),
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
                        Span::raw(format!("Burt #{}", selected_burt.get_id())), // id
                        Span::raw("?".to_string()), // score
                        Span::raw("?".to_string()), // guess
                        Span::raw("?".to_string()), // mu
                        Span::raw("?".to_string()), // sigma
                    ])])
                        .header(Row::new(vec![
                            Span::styled(
                                "ID",
                                Style::default().add_modifier(Modifier::BOLD),
                            ),
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
                                .title("Burt Details")
                                .border_type(BorderType::Plain),
                        )
                        .widths(&[
                            Constraint::Percentage(10),
                            Constraint::Percentage(10),
                            Constraint::Percentage(20),
                            Constraint::Percentage(5),
                            Constraint::Percentage(20),
                        ]);
                    rect.render_stateful_widget(burts_list_left, burts_chunks[0], &mut burt_list_state);
                    rect.render_widget(burt_detail, burts_chunks[1]);

                    if input_mode {
                        input_mode_prompt = format!("Enter a Burt ID");
                    }
                }
                MenuItem::Log => {
                    let home = Paragraph::new(vec![
                        Spans::from(vec![Span::raw("")]),
                        Spans::from(vec![Span::raw("The Logs feature is not currently implemented!")]),
                        Spans::from(vec![Span::raw("")]),
                    ])
                        .alignment(Alignment::Center)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .style(Style::default().fg(Color::White))
                                .title("Home")
                                .border_type(BorderType::Plain),
                        );
                    rect.render_widget(home, chunks[1]);
                }
            }

            if input_mode {
                let input = Paragraph::new(user_input.clone())
                    .style(Style::default().fg(Color::Gray))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::White))
                            .title(input_mode_prompt.clone())
                            .border_type(BorderType::Double)
                    );
                rect.set_cursor(
                    chunks[2].x + user_input.len() as u16 + 1,
                    chunks[2].y + 1,
                );
                rect.render_widget(input, chunks[2]);

                if input_ready {

                    // todo: use the input to get the burt ID

                    input_ready = false;
                    user_input = String::new();
                }
                input_mode_prompt = format!("Input");
            } else {
                rect.render_widget(footer, chunks[2]);
                input_mode = false;
            }
        }).expect("Failed to draw frame with TUI");

        // handle keypresses for the UI
        match rx.recv().expect("Failed to receive keypress") {
            Event::Input(event) => {
                if event.code == KeyCode::Char('`') {
                    break;
                }
                if input_mode {
                    match event.code {
                        KeyCode::Char('?') => {
                            input_mode = false;
                        }
                        KeyCode::Char(c) => {
                            user_input.push(c);
                        }
                        KeyCode::Enter => {
                            input_ready = true;
                            input_mode = !input_mode;
                        }
                        KeyCode::Esc => {
                            input_mode = !input_mode;
                        }
                        KeyCode::Backspace => {
                            if user_input.len() > 0 {
                                user_input.remove(user_input.len() - 1);
                            }
                        }
                        _ => {}
                    }
                } else {
                    match event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                        KeyCode::Char('b') => active_menu_item = MenuItem::Burts,
                        KeyCode::Char('?') => {
                            input_mode = !input_mode;
                        }
                        KeyCode::Char('l') => active_menu_item = MenuItem::Log,
                        KeyCode::Down => {
                            if let Some(selected) = burt_list_state.selected() {
                                let amnt_burts = burt_gang.len();
                                if selected >= amnt_burts - 1 {
                                    burt_list_state.select(Some(0));
                                } else {
                                    burt_list_state.select(Some(selected + 1));
                                }
                            }
                        }
                        KeyCode::Up => {
                            if let Some(selected) = burt_list_state.selected() {
                                let amnt_burts = burt_gang.len();
                                if selected > 0 {
                                    burt_list_state.select(Some(selected - 1));
                                } else {
                                    burt_list_state.select(Some(amnt_burts - 1));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Tick => {}
        }

    }

    // restore terminal
    disable_raw_mode().expect("Failed to restore terminal");
    terminal.clear().expect("Failed to restore terminal");
    execute!(
        terminal.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture,
        MoveTo(0,0),
        terminal::Clear(ClearType::All)
    ).expect("Failed to restore terminal");
    terminal.show_cursor().expect("Failed to restore terminal");

}