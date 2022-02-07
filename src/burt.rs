use std::fmt::{Display, Formatter};
use better_term::{Color, flush_styles};
use pbars::{BarType, hide_cursor, PBar, show_cursor};
use rand::{Rng, thread_rng};
use rand_distr::{Normal, Distribution};
use crate::input::{get_decimal, get_num, prompt};

fn distance_from(target: u32, guess: u32) -> u32 {
    target.abs_diff(guess)
}

#[derive(Clone, Debug)]
pub struct Burt {
    id: u32,

    score: Option<u32>,
    guess: Option<u32>,

    mu: f32,    // aka the mean
    sigma: f32, // aka standard deviation
}

impl Burt {
    pub fn new(id: u32, range: u32) -> Self {
        Self {
            id,

            score: None,
            guess: None,

            mu: thread_rng().gen_range(0.0..range as f32),
            sigma: thread_rng().gen_range(0.0..range as f32),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.id.clone()
    }
    pub fn get_mu(&self) -> f32 {
        self.mu.clone()
    }
    pub fn get_sigma(&self) -> f32 {
        self.sigma.clone()
    }
    pub fn get_score_display(&self) -> String {
        format!("{}", if self.score.is_some() {
            self.score.unwrap().to_string()
        } else {
            String::from("?")
        })
    }
    pub fn get_guess_display(&self) -> String {
        format!("{}", if self.guess.is_some() {
            self.guess.unwrap().to_string()
        } else {
            String::from("?")
        })
    }

    pub fn training_think(&mut self, target: u32, range: u32) -> (u32, u32) {
        let output = self.think(1.0, range as f32) as u32;
        let score = distance_from(target, output);
        self.score = Some(score.clone());
        self.guess = Some(output);
        (output, score)
    }

    pub fn think(&mut self, input: f32, range: f32) -> f32 {
        // normal distribution
        let normal = Normal::new(self.mu, self.sigma)
            .expect(format!("Failed to create normal for Burt #{}", self.id).as_str());
        let mut number = range + 1.0;
        while number > range {
            number = normal.sample(&mut thread_rng());
        }

        input * number
    }

    pub fn reeducate(&mut self, mu: f32, sigma: f32) {
        // todo(eric): maybe average these out with the current mu and sigma?
        self.mu = mu;
        self.sigma = sigma;
    }

    pub fn mutate(&mut self, mutation_rate: f32, range: u32) {
        // todo(eric): This means that theoretically a generation can go by with no mutation
        let mu_mut_amt = thread_rng().gen_range(0.0..((range as f32) * mutation_rate));
        // mu
        if thread_rng().gen_bool(0.5) {
            self.mu += mu_mut_amt;
            if self.mu > range as f32 {
                self.mu = range as f32;
            }
        } else {
            self.mu -= mu_mut_amt;
            if self.mu < 0.0 {
                self.mu = 0.0;
            }
        }

        let sigma_mut_amt = thread_rng().gen_range(0.0..mutation_rate);
        // sigma
        if thread_rng().gen_bool(0.5) {
            self.sigma += sigma_mut_amt;
            if self.sigma > range as f32 {
                self.sigma = range as f32;
            }
        } else {
            self.sigma -= sigma_mut_amt;
            if self.sigma < 0.0 {
                self.sigma = 0.0;
            }
        }
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

    average_guess: Option<u32>,
    average_score: Option<u32>,
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

            average_guess: None,
            average_score: None,
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

    pub fn run_generation(&mut self) {
        // increase generation
        self.current_generation += 1;

        // store variables for averages
        let mut total_score: usize = 0;
        let mut total_guess: usize = 0;
        let mut runs: u32 = 0;

        // loop through the burts and have them guess
        for b in &mut self.burts {
            let (guess, score) = b.training_think(self.target, self.range);
            total_guess += guess as usize;
            total_score += score as usize;
            runs += 1;
        }

        // set the averages
        self.average_guess = Some((total_guess / runs as usize) as u32);
        self.average_score = Some((total_score / runs as usize) as u32);

        // todo(eric): get the worst burts and mutate them

        // sort the burts into [0] best -> [len() - 1] worst
        let mut sorted_burts: Vec<Burt> = Vec::new();

        // loop through the burts
        // worst case this will loop through all Burts twice :/ (can be in the thousands if not millions)
        while !self.burts.is_empty() {
            // get the current burt
            let b = self.burts.remove(0);
            let mut placed = false;
            // loop through the sorted burts
            for x in 0..sorted_burts.len() {
                // get the current sorted burt (sb)
                let sb = sorted_burts.get(x).unwrap();
                // if the current burt has a better or equal score
                if (&b).score.unwrap() <= sb.score.unwrap() {
                    // place the burt in the sorted burts list at the location
                    sorted_burts.insert(x, b.clone());
                    placed = true;
                    // break the loop
                    break;
                }
            }
            // if it hasn't been placed
            // push the burt at the end (its a bad burt )
            if !placed {
                sorted_burts.push(b);
            }
        }

        // with the sorted burt list, "re-educate" the lesser Burts
        let mut survival_amt = (sorted_burts.len() as f32 * self.survival_rate) as u32;
        if survival_amt < 1 {
            survival_amt = 1;
        }
        // split sorted_burts into 2 vectors:
        // original (sorted_burts) - contains the burts that survived
        // bad_burts               - contains the burts that need to be re-educated
        let mut bad_burts = sorted_burts.split_off(survival_amt as usize);

        // go through and un-sort the burts
        let mut new_burts: Vec<Burt> = Vec::new();
        while !bad_burts.is_empty() {
            let mut current = bad_burts.remove(0);

            // re-educate and mutate the current
            let teacher = sorted_burts.get(thread_rng().gen_range(0..sorted_burts.len())).unwrap();
            let new_mu = teacher.mu.clone();
            let new_sigma = teacher.sigma.clone();
            current.reeducate(new_mu, new_sigma);
            current.mutate(self.mutation_rate, self.range);

            let mut placed = false;
            for x in 0..new_burts.len() {
                let b = new_burts.get(x).unwrap();
                if current.id < b.id {
                    new_burts.insert(x, current.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                new_burts.push(current);
            }
        }

        while !sorted_burts.is_empty() {
            let current = sorted_burts.remove(0);
            // mutate the current
            //current.mutate(self.mutation_rate, self.range);

            let mut placed = false;
            for x in 0..new_burts.len() {
                let b = new_burts.get(x).unwrap();
                if current.id < b.id {
                    new_burts.insert(x, current.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                new_burts.push(current);
            }
        }
        self.burts = new_burts;
    }

    pub fn save_best_average(&self) {
        // todo(eric): Write the best of the average mu's and sigma's to a file along with the guess
        //  i.e. in "target:{target},mu:{average_mu},sigma:{average_sigma}"
    }

    pub fn av_guess_display(&self) -> String {
        format!("{}", if self.average_guess.is_some() {
            format!("{}", self.average_guess.unwrap())
        } else {
            format!("?")
        })
    }

    pub fn av_score_display(&self) -> String {
        format!("{}", if self.average_score.is_some() {
            format!("{}", self.average_score.unwrap())
        } else {
            format!("?")
        })
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
                         format!("Target:        {}", self.target),
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

pub fn populate_burts(burt_count: u32, range: u32) -> Vec<Burt> {
    let mut burts = Vec::new();
    // print the progress bar and begin populating Burts
    println!("Populating Burts...");
    hide_cursor();
    let mut pbar = PBar::new_at_cursor(BarType::Bar, true, true, 20)
        .expect("Failed to get cursor position: is this terminal supported?");
    // make each new burt with x being their id, and update the progress bar
    for x in 0..burt_count {
        // push the new burt into the vector
        burts.push(Burt::new(x, range.clone()));

        // get the percentage of completion
        let percent = x as f64 / (burt_count - 1) as f64;
        pbar.update((percent * 100.0) as u8);
        pbar.draw();
        println!();
        println!("Burts: {} / {}", x + 1, burt_count);
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

        // can't process with less than 1 burt
        if burt_count == 0 {
            println!("{}Warning: There has to be at least 1 burt!", Color::Yellow);
            flush_styles();
            continue;
        }

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

    BurtGang::new(populate_burts(burt_count, range.clone()), range, target, generations, survival_rate, mutation_rate)
}