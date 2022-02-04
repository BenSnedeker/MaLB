use std::fmt::{Display, Formatter};
use better_term::{Color, flush_styles};
use pbars::{BarType, hide_cursor, PBar, show_cursor};
use crate::input::{get_decimal, get_num, prompt};

pub struct Burt {
    id: u32
}

impl Burt {
    pub fn new(id: u32) -> Self {
        Self {
            id,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id.clone()
    }
}

pub struct BurtGang {
    pub burts: Vec<Burt>,
    pub range: u32,
    pub target: u32,
    pub generations: u32,
    pub current_generation: u32,
    pub survival_rate: f32,
    pub mutation_rate: f32,
}

impl BurtGang {
    pub fn new(burts: Vec<Burt>, range: u32, target: u32, generations: u32, survival_rate: f32, mutation_rate: f32) -> Self {
        Self {
            burts,
            range,
            target,
            current_generation: 0,
            generations,
            survival_rate,
            mutation_rate,
        }
    }

    pub fn iter(&self) -> core::slice::Iter<Burt>{
        self.burts.iter()
    }

    pub fn get(&self, x: usize) -> &Burt {
        self.burts.get(x).expect("Failed to get burt from gang!")
    }

    pub fn len(&self) -> usize {
        self.burts.len()
    }
}

impl Iterator for BurtGang {
    type Item = Burt;

    fn next(&mut self) -> Option<Self::Item> {
        self.burts.pop()
    }
}

impl Display for BurtGang {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // printing out the values with style
        let lines = vec![
                         format!("Range:         {}", self.range),
                         format!("Target:        {}", self.range),
                         format!("Generations:   {}", self.generations),
                         format!("Survival rate: {}", self.survival_rate),
                         format!("Mutation rate: {}", self.mutation_rate),
                         format!("# of burts:    {}", self.burts.len())];

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
        write!(f, "┌{t}┐\n{}\n└{t}┘\n", new_lines.join("\n"), t = "─".repeat(new_lines.first().unwrap().len() - 6))
    }
}

pub fn populate_burts(burt_count: u32) -> Vec<Burt> {
    let mut burts = Vec::new();
    // print the progress bar and begin populating Burts
    println!("Populating Burts...");
    hide_cursor();
    let mut pbar = PBar::new_at_cursor(BarType::Bar, true, true, 20)
        .expect("Failed to get cursor position: is this terminal supported?");
    // make each new burt with x being their id, and update the progress bar
    for x in 0..burt_count {
        // push the new burt into the vector
        burts.push(Burt::new(x));

        // get the percentage of completion
        let percent = x as f64 / (burt_count - 1) as f64;
        pbar.update((percent * 100.0) as u8);
        pbar.draw();
        println!();
        println!("Burts: {} / {}", x, burt_count);
    }
    // clear the output styles
    flush_styles();
    // print out completion message and show the cursor
    println!("\nPopulated {} Burts!", burt_count);
    show_cursor();
    burts
}

pub fn get_burt_gang() -> BurtGang {
    // get the terminal size for displaying
    let (term_width, term_height) = crossterm::terminal::size().expect("Failed to get terminal size");

    // the input variables
    let mut range: u32;
    let mut target: u32;
    let mut generations: u32;
    let mut survival_rate: f32;
    let mut mutation_rate: f32;
    let mut burt_count: u32;

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

    BurtGang::new(populate_burts(burt_count), range, target, generations, survival_rate, mutation_rate)
}