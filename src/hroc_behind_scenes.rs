use std::io::{stdin, stdout, Write};
use std::ops::Range;
use std::process::exit;

fn read_input<S: Into<String>>(prompt: S) -> String {
    print!("{}", prompt.into());
    let r = stdout().flush();
    if r.is_err() {
        panic!("Error flusing output: {}", r.unwrap_err());
    }
    let mut buffer = String::new();
    let r2 = stdin().read_line(&mut buffer);
    if r2.is_err() {
        panic!("Error in reading input: {}", r.unwrap_err());
    }
    println!();
    buffer.replace("\n", "").replace("\r", "")
}

pub fn prompt<S: Into<String>>(prompt: S) -> bool {
    let input = read_input(format!("{} (Y or N): ", prompt.into()));
    match input.to_ascii_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            eprintln!("Invalid input in Yes or No question: {}", input);
            exit(1);
        }
    }
}

pub fn get_num<S: Into<String>>(prompt: S) -> u32 {
    let input = read_input(format!("{}: ", prompt.into()));
    if input.is_empty() || !input.is_ascii() {
        eprintln!();
        exit(1);
    }
    for c in input.chars() {
        if !c.is_numeric() {
            eprintln!("Invalid input: Expected number but found '{}'", c);
            exit(1);
        }
    }
    let parsed = input.parse::<u32>();
    if parsed.is_err() {
        eprintln!("Invalid input: failed to get number from input!");
        exit(1);
    }
    let x = parsed.unwrap();
    x
}

pub fn get_in_range<S: Into<String>, R: Into<Range<u32>>>(prompt: S, r: R) -> Result<u32, String> {
    let range = r.into();
    let x = get_num(format!("{}: ", prompt.into()));
    if !range.contains(&x) {
        return Err(format!("Invalid number: Expected a number between {} and {}!", range.start, range.end - 1));
    }
    Ok(x)
}

pub fn get_decimal_in_range<S: Into<String>, R: Into<Range<f32>>>(prompt: S, r: R) -> Result<f32, String> {
    let range = r.into();
    let input = read_input(format!("{}: ", prompt.into()));
    if input.is_empty() || !input.is_ascii() {
        return Err(format!("Invalid input! Expected a number between {} and {}: ", range.start, range.end - 1.0))
    }
    for c in input.chars() {
        if !c.is_numeric() && c != '.' {
            return Err(format!("Invalid input: Expected number but found '{}'", c));
        }
    }
    let parsed = input.parse::<f32>();
    if parsed.is_err() {
        return Err(format!("Failed to get valid number from the input! Error: {}", parsed.unwrap_err()));
    }
    let x = parsed.unwrap();
    if !range.contains(&x) {
        return Err(format!("Invalid number: Expected a number between {} and {}!", range.start, range.end - 1.0));
    }
    Ok(x)
}

pub fn get_num_0_1<S: Into<String>>(prompt: S) -> f32 {
    let number_res = get_decimal_in_range(prompt.into(), 0.0..1.0);
    if number_res.is_err() {
        eprintln!("{}", number_res.unwrap_err());
        exit(1);
    }
    number_res.unwrap()
}