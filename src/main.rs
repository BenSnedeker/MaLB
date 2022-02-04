use std::env;
use std::io::stdout;
use better_term::{Color, flush_styles};
use crossterm::{cursor, execute};
use crossterm::terminal::{Clear, ClearType};
use rand::{Rng, thread_rng};
use crate::input::{get_num, prompt, get_decimal};

mod input;

pub struct Burt {
    id: u32
}

impl Burt {
    fn new(id: u32) -> Self {
        Self {
            id,
        }
    }

    fn get_id(&self) -> u32 {
        self.id.clone()
    }
}

fn main() {
    // get the environment args for parsing
    let args = env::args().collect::<Vec<String>>();

    // clear the screen for printing
    execute!(stdout(), Clear(ClearType::All), cursor::MoveTo(0,0)).expect("Failed to set up terminal! Are you using a supported terminal?");

    // the vec to hold the burts
    let mut burts: Vec<Burt> = Vec::new();

    // get the terminal size for displaying
    let (term_width, term_height) = crossterm::terminal::size().expect("Failed to get terminal size");

    // the input variables
    let mut range: u32;
    let mut target: u32;
    let mut generations: u32;
    let mut survival_rate: f32;
    let mut mutation_rate: f32;
    let mut burt_count: u32;

    // populate the input variables
    if args.contains(&"-d".to_string()) || args.contains(&"--defaults".to_string()) {
        // set the defaults of the variables
        range = term_width as u32;
        target = thread_rng().gen_range(0..range);
        generations = 1000;
        survival_rate = 0.50;
        mutation_rate = 0.25;
        burt_count = 100000;
    } else {
        // get the user input for the variables
        loop {
            // get the desired range from the user
            let input = get_num("Enter a number above 0 for how large the range should be");
            if input.is_none() {
                continue;
            }
            range = input.unwrap();
            // if it is 0, error and try again
            if range < 1 {
                println!("{}Warning: The range can not be zero!", Color::Yellow);
                flush_styles();
                continue;
            }
            // if the range is greater than the terminal width, warn the user and ask if they want to change the range
            if range > term_width as u32 {
                println!("{}Warning: The range is greater than the current terminal width! this can cause rendering issues!\n\
                    Current terminal size: {}x{}", Color::Yellow, term_width, term_height);
                flush_styles();
                let new_value = prompt("Are you sure you wish to use this number?");
                if !new_value {
                    continue;
                }
            }
            break
        }
        loop {
            // get the desired target number from the user
            let input = get_num(format!("Enter the target (between 0 and {})", range));
            if input.is_none() {
                continue;
            }
            target = input.unwrap();
            // if the target is not in the range, ask for a new number
            if !(0..range).contains(&target) {
                println!("{}Warning: The target must be within the range! Range: 0 to {}", Color::Yellow, range);
                flush_styles();
            } else {
                break;
            }
        }
        loop {
            // get the desired amount of rounds / generations from the user
            // this *can* be zero, but it will mean the program does nothing
            let input = get_num("Enter how many generations there should be");
            if input.is_none() {
                continue;
            }
            generations = input.unwrap();
            // if the burt count is greater than 100k warn the user about high memory and cpu usage
            if generations > 100000 {
                println!("{}Warning: You have entered a very high amount of rounds! This can result in excessive CPU usage.", Color::Yellow);
                flush_styles();
                let new_value = prompt("Are you sure you wish to use that many?");
                if !new_value {
                    continue;
                }
            }
            break;
        }
        loop {
            // get the desired survival rate from the user
            let input = get_decimal("Enter the survival rate between 0 and 1");
            if input.is_none() {
                continue;
            }
            survival_rate = input.unwrap();
            // if it is 1, 0, or outside that range, get a different number
            if survival_rate >= 1.0 || survival_rate <= 0.0 {
                println!("{}Warning: The survival rate must be between 0 and 1, not including 0 and 1.", Color::Yellow);
                flush_styles();
            } else {
                break;
            }
        }
        loop {
            // get the desired mutation rate from the user
            let input = get_decimal("Enter the mutation rate between 0 and 1");
            if input.is_none() {
                continue;
            }
            mutation_rate = input.unwrap();
            // if it is 1, 0, or outside that range, get a different number
            if mutation_rate >= 1.0 || mutation_rate <= 0.0 {
                println!("{}Warning: The mutation rate must be between 0 and 1, not including 0 and 1.", Color::Yellow);
                flush_styles();
            } else {
                break;
            }
        }
        loop {
            // get the desired amount of burts to use
            let input = get_num("Enter how many Burts should be used");
            if input.is_none() {
                continue;
            }
            burt_count = input.unwrap();
            // if the burt count is greater than 100k warn the user about high memory and cpu usage
            if burt_count > 100000 {
                println!("{}Warning: You have entered a very high burt count! This can use large amounts of CPU and Memory (RAM).", Color::Yellow);
                flush_styles();
                let new_value = prompt("Are you sure you wish to use that many?");
                if !new_value {
                    continue;
                }
            }
            break;
        }
        println!();
    }

    // printing out the values with style
    let lines = vec![format!("Range:         {}", range),
                     format!("Target:        {}", range),
                     format!("Generations:   {}", generations),
                     format!("Survival rate: {}", survival_rate),
                     format!("Mutation rate: {}", mutation_rate),
                     format!("# of burts:    {}",   burt_count)];

    // get the longest line
    let mut longest = 0;
    for l in &lines {
        if l.len() > longest {
            longest = l.len();
        }
    }

    // the lines to print out
    let mut new_lines = Vec::new();

    // generate the | after
    for l in &lines {
        new_lines.push(format!("│ {}{} │", l.clone(), " ".repeat(longest - l.len())));
    }

    // print out the values of the variables
    println!("┌{t}┐\n{}\n└{t}┘\n", new_lines.join("\n"), t = "─".repeat(new_lines.first().unwrap().len() - 6));

    // print the progress bar and begin populating Burts
    println!("Populating Burts...");
    execute!(stdout(), cursor::Hide).expect("Failed to hide the cursor! This terminal may not be supported!");
    print!("[{}] 0%", "█".repeat(20));

    // make each new burt with x being their id, and update the progress bar
    for x in 0..burt_count {
        // push the new burt into the vector
        burts.push(Burt::new(x));

        // get the percentage of completion
        let percent_decimal = x as f64 / (burt_count - 1) as f64;
        let percent = (percent_decimal * 100.0) as u32;

        // set how complete the bar should be
        let bar_completion = percent as usize / 5;
        let mut bar_uncomplete = (100 - (percent as usize)) / 5;
        // handle if the bar needs to be resized because of rounding issues
        let add = bar_completion + bar_uncomplete;
        if add < 20 {
            bar_uncomplete += 1;
        }
        if add > 20 {
            bar_uncomplete -= 1;
        }

        // get the current completion color of the bar
        // todo(eric): Maybe make this a scale of red -> green using RGB?
        let red = 255 - (percent_decimal * 200.0) as u8;
        let green = ((percent_decimal * 200.0) as u8);
        let color = Color::RGB(red, green, 25);

        // create the different parts of the bar
        let completed_bar = format!("{}{}", color, "█".repeat(bar_completion));
        let uncompleted_bar = format!("{}{}", Color::BrightBlack, "█".repeat(bar_uncomplete));

        // Print out the bar
        print!("\r{dc}[{}{}{dc}] {}{}{dc}%", completed_bar, uncompleted_bar, color, percent,
        dc = Color::White);
    }
    // clear the output styles
    flush_styles();
    // print out completion message and show the cursor
    println!("\nPopulated {} Burts!", burt_count);
    execute!(stdout(), cursor::Show).expect("Failed to reshow cursor! This terminal may not be supported!");

    println!("The program doesn't do anything past this yet!");
}