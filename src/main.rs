use std::process::{Command, exit};
use better_term::{Color, flush_styles};
use crate::hroc_behind_scenes::{get_num_0_1, get_in_range, get_decimal_in_range, get_num, prompt};

mod hroc_behind_scenes;

pub struct Burt {
    id: u32
}

impl Burt {
    fn new(id: u32) ->Self {
        Self {
            id,
        }
    }
}

fn main() {
    let mut burts: Vec<Burt> = Vec::new();

    let (term_width, term_height) = crossterm::terminal::size().expect("Failed to get terminal size");

    let mut range: u32;
    loop {
        range = get_num("Enter a number above 0 for how large the range should be");
        if range < 1 {
            eprintln!("The range can not be zero!");
            exit(1);
        }
        if range > term_width as u32 {
            println!("{}Warning: The range is greater than the current terminal width! this can cause rendering issues!\n\
                    Current terminal size: {}x{}", Color::Yellow, term_width, term_height);
            flush_styles();
            let new_value = prompt("Do you wish to set a different value?");
            if !new_value {
                break;
            }
        } else {
            break;
        }
    }
    let target = get_num(format!("Enter the target (between 0 and {})", range));
    if !(0..range).contains(&target) {
        eprintln!("The target must be within the range!");
        exit(1);
    }
    let rounds = get_num("Enter how many rounds there should be");
    let survival_rate = get_num_0_1("Enter the survival rate between 0 and 1");
    let burt_count = get_num("Enter the amount of starting burts");

    println!("Populating burts...");
    print!("[--------------------] 0%");
    for x in 0..burt_count {
        burts.push(Burt::new(x));
        let percent_decimal = x as f64 / (burt_count - 1) as f64;
        let percent = (percent_decimal * 100.0) as u32;

        let bar_completion = percent as usize / 5;
        let mut bar_uncomplete = (100 - (percent as usize)) / 5;
        let add = bar_completion + bar_uncomplete;
        if add < 20 {
            bar_uncomplete += 1;
        }
        if add > 20 {
            bar_uncomplete -= 1;
        }

        let color = if percent < 25 {
            Color::Red
        } else if percent < 50 {
            Color::Yellow
        } else if percent < 75 {
            Color::BrightYellow
        } else if percent < 100 {
            Color::Green
        } else {
            Color::BrightGreen
        };

        let completed_bar = format!("{}{}", color, "█".repeat(bar_completion));
        let uncompleted_bar = format!("{}{}", Color::BrightBlack, "-".repeat(bar_uncomplete));

        let default_color = Color::White;

        print!("\r{}[{}{}{}] {}{}{}%", default_color, completed_bar, uncompleted_bar, default_color, color, percent, default_color);
    }
    println!("\n{} Burts populated!", burt_count);
}