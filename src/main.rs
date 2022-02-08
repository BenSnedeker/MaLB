#![feature(int_abs_diff)]

use std::{env, thread};
use std::io::stdout;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use crossterm::{event, execute, terminal};
use crossterm::cursor::MoveTo;
use crossterm::terminal::{ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::event::{Event as CEvent, KeyCode};
use log::{info, LevelFilter};
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::Terminal;
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, BorderType, ListState, Paragraph, Tabs};
use tlogger::{init_logger, set_default_level, TuiLoggerLevelOutput, TuiLoggerSmartWidget};
use crate::burt::{BurtGang, get_burt_gang, populate_burts};
use crate::ui::{draw_burts, draw_home, Event, MenuItem};

pub(crate) mod input;
mod ui;
mod burt;

pub const TRAIN_STICKY: bool = true;

fn main() {
    // get arguments
    let args: Vec<String> = env::args().collect();

    // clear the screen and set terminal position
    execute!(stdout(), terminal::Clear(ClearType::All), MoveTo(0,0)).expect("Failed to clear screen! Is this terminal supported?");

    // initialize the burts
    let mut burt_gang = if args.contains(&"-d".to_string()) || args.contains(&"--default".to_string()) {
        let default_range = 100;
        BurtGang::new(populate_burts(150,
                                     default_range.clone(), true),
                      default_range, 7, 150,
                      0.01, 0.25)
    } else {
        get_burt_gang()
    };

    let starting_burt_count = burt_gang.burts.len() as u32;
    let starting_range = burt_gang.range;
    let starting_target = burt_gang.target;
    let starting_survival_rate = burt_gang.survival_rate;
    let starting_mutation_rate = burt_gang.mutation_rate;
    let starting_generations = burt_gang.generations;

    //println!("{}", &burt_gang);

    // initialize logger
    init_logger(LevelFilter::Trace).unwrap();
    set_default_level(LevelFilter::Trace);
    info!(target:"MaLB", "Starting renderer");

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

    // input handling thread
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

    let default_footer_txt = format!("MaLB v{} 2022 created and maintained by Eric Shreve and Ben Snedeker", env!("CARGO_PKG_VERSION"));
    let mut footer_txt = default_footer_txt.clone();
    let default_footer_col = Color::LightCyan;
    let mut footer_col = default_footer_col.clone();

    let error_time = Duration::from_millis(3000);
    let mut error_start: Option<Instant> = None;

    let generation_delay = Duration::from_millis(0);
    let mut last_gen_run: Option<Instant> = None;
    let mut running = false;

    // start the main loop
    loop {
        // draw the UI
        let footer = Paragraph::new(footer_txt.clone())
            .style(Style::default().fg(footer_col))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Info")
                    .border_type(BorderType::Plain)
            );
        terminal.draw(|mut rect| {
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
                    draw_home(&mut rect, &chunks, &mut burt_gang);
                }
                MenuItem::Burts => {
                    draw_burts(&mut rect, &chunks, &burt_gang, &mut burt_list_state);

                    if input_mode {
                        input_mode_prompt = format!("Enter a Burt ID");
                    }

                    if input_ready {
                        // handle burt id search
                        for c in user_input.chars() {
                            if !c.is_numeric() {
                                footer_txt = format!("Invalid Input: must be a number!");
                                footer_col = Color::LightRed;
                                user_input = String::new();
                                error_start = Some(Instant::now());
                                return;
                            }
                        }

                        let number = user_input.parse::<usize>();

                        if number.is_err() {
                            footer_txt = format!("Invalid Input: must be a number!");
                            footer_col = Color::LightRed;
                            input_ready = false;
                            user_input = String::new();
                            error_start = Some(Instant::now());
                            return;
                        }

                        let n = number.unwrap();

                        if n >= burt_gang.len() {
                            footer_txt = format!("Number must be a valid burt id! {} is too large!", n);
                            footer_col = Color::LightRed;
                            input_ready = false;
                            user_input = String::new();
                            error_start = Some(Instant::now());
                            return;
                        }

                        burt_list_state.select(Some(n));

                        input_ready = false;
                        user_input = String::new();
                    }
                }
                MenuItem::Log => {
                    let tui_sm = TuiLoggerSmartWidget::default()
                        .style_error(Style::default().fg(Color::Red))
                        .style_debug(Style::default().fg(Color::Green))
                        .style_warn(Style::default().fg(Color::Yellow))
                        .style_trace(Style::default().fg(Color::Magenta))
                        .style_info(Style::default().fg(Color::Cyan))
                        .output_separator(": ".to_string())
                        .output_timestamp(Some("%H:%M:%S ".to_string()))
                        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
                        .output_file(false)
                        .output_line(false)
                        .output_target(true);
                    rect.render_widget(tui_sm, chunks[1]);
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
            }
            if !input_mode {
                rect.render_widget(footer, chunks[2]);
            }
            input_mode_prompt = format!("Input");
        }).expect("Failed to draw frame with TUI");

        // handle keypresses for the UI
        let event_poll = rx.recv_timeout(Duration::from_millis(200));
        if event_poll.is_ok() {
            match event_poll.unwrap() {
                Event::Input(event) => {
                    if event.code == KeyCode::Char('c') {
                        if event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                            // add ctrl+c functionality
                            // if the program is processing for a long time this wont complete until it's done processing
                            break;
                        }
                    }
                    if input_mode {
                        match event.code {
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
                            KeyCode::Char('s') => running = !running,
                            KeyCode::Char('r') => {
                                burt_gang = BurtGang::new(populate_burts(starting_burt_count.clone(),
                                                                         starting_range.clone(), false),
                                                          starting_range.clone(),
                                                          starting_target.clone(), starting_generations.clone(),
                                                          starting_survival_rate.clone(),
                                                          starting_mutation_rate.clone())
                            }
                            KeyCode::Char('h') => {
                                active_menu_item = MenuItem::Home;
                            },
                            KeyCode::Char('b') => {
                                active_menu_item = MenuItem::Burts;
                            },
                            KeyCode::Char('t') => {
                                input_mode = !input_mode;
                            }
                            KeyCode::Char('l') => {
                                active_menu_item = MenuItem::Log;
                            },
                            KeyCode::Char('e') => {
                                info!(target:"MalB", "User forced run of training generation: {}/{}",
                                    burt_gang.current_generation, burt_gang.generations);
                                burt_gang.train(TRAIN_STICKY);
                            },
                            KeyCode::Down => {
                                if let Some(selected) = burt_list_state.selected() {
                                    if selected >= burt_gang.len() - 1 {
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

        // handle running training
        if running && burt_gang.current_generation < burt_gang.generations {
            if last_gen_run.is_some() {
                if last_gen_run.unwrap().elapsed() >= generation_delay {
                    burt_gang.train(TRAIN_STICKY);
                    last_gen_run = Some(Instant::now());
                }
            } else {
                burt_gang.train(TRAIN_STICKY);
                last_gen_run = Some(Instant::now());
            }
        } else {
            // training is complete, do something here
        }

        // handle input
        let burt_list_mode = if let MenuItem::Burts = active_menu_item { true } else { false };
        if input_ready && !burt_list_mode {
            if !user_input.is_empty() {
                let mut cmd_args = user_input.split(" ").collect::<Vec<&str>>();
                let cmd = cmd_args.remove(0);

                match cmd.to_ascii_lowercase().as_str() {
                    "change" => {
                        if cmd_args.len() < 2 {
                            footer_txt = format!("change takes a variable name and a new value!");
                            footer_col = Color::LightRed;
                            error_start = Some(Instant::now());
                            input_ready = false;
                            user_input = String::new();
                            continue;
                        }

                        let var = cmd_args.get(0).unwrap();
                        let value = cmd_args.get(1).unwrap();

                        match *var {
                            "range" => {
                                let parsed = value.parse::<u32>();
                                if parsed.is_err() {
                                    footer_txt = format!("Invalid value: range expects a value above 0!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                let n = parsed.unwrap();
                                burt_gang.range = n;
                            }
                            "target" => {
                                let parsed = value.parse::<u32>();
                                if parsed.is_err() {
                                    footer_txt = format!("Invalid value: target expects a value above 0!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                let n = parsed.unwrap();
                                burt_gang.target = n;
                            }
                            "generations" => {
                                let parsed = value.parse::<u32>();
                                if parsed.is_err() {
                                    footer_txt = format!("Invalid value: generations expects a value above 0!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                let n = parsed.unwrap();

                                burt_gang.generations = n;
                            }
                            "survival_rate" => {
                                let decimal = value.parse::<f32>();
                                if decimal.is_err() {
                                    footer_txt = format!("Invalid value: survival_rate expects a value between 0 and 1!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                let n = decimal.unwrap();
                                if n < 0.0 || n >= 1.0 {
                                    footer_txt = format!("Invalid value: survival_rate expects a value between 0 and 1!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                burt_gang.survival_rate = n;
                            }
                            "mutation_rate" => {
                                let decimal = value.parse::<f32>();
                                if decimal.is_err() {
                                    footer_txt = format!("Invalid value: mutation_rate expects a value between 0 and 1!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                let n = decimal.unwrap();
                                if n < 0.0 || n >= 1.0 {
                                    footer_txt = format!("Invalid value: mutation_rate expects a value between 0 and 1!");
                                    footer_col = Color::LightRed;
                                    error_start = Some(Instant::now());
                                    input_ready = false;
                                    user_input = String::new();
                                    continue;
                                }
                                burt_gang.mutation_rate = n;
                            }
                            _ => {
                                footer_txt = format!("Invalid variable!");
                                footer_col = Color::LightRed;
                                error_start = Some(Instant::now());
                                input_ready = false;
                                user_input = String::new();
                                continue;
                            }
                        }
                    }
                    _ => {
                        footer_txt = format!("Invalid Command!");
                        footer_col = Color::LightRed;
                        error_start = Some(Instant::now());
                    }
                }
            }

            input_ready = false;
            user_input = String::new();
        }

        if error_start.is_some() {
            let start = error_start.unwrap();
            if start.elapsed() >= error_time {
                error_start = None;
                footer_txt = default_footer_txt.clone();
                footer_col = default_footer_col.clone();
            }
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