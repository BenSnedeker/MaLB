use std::io::{stdin, stdout, Write};
use std::process::exit;
use better_term::{Color, flush_styles};

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
    buffer.replace("\n", "").replace("\r", "")
}

pub fn prompt<S: Into<String>>(prompt: S) -> bool {
    let p = prompt.into();
    loop {
        let input = read_input(format!("{} (Y or N): ", p));
        match input.to_ascii_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            _ => {
                println!("{}Warning: The input can only be Y or N!", Color::Yellow);
                flush_styles();
            }
        }
    }
}

pub fn get_num<S: Into<String>>(prompt: S) -> Option<u32> {
    let input = read_input(format!("{}: ", prompt.into()));
    if input.is_empty() || !input.is_ascii() {
        println!("{}Warning: The input can only be a number!", Color::Yellow);
        flush_styles();
        return None;
    }
    for c in input.chars() {
        if !c.is_numeric() {
            println!("{}Warning: The input can only be a number!", Color::Yellow);
            flush_styles();
            return None;
        }
    }
    let parsed = input.parse::<u32>();
    if parsed.is_err() {
        println!("{}Warning: The input can only be a number!", Color::Yellow);
        flush_styles();
        return None;
    }
    let x = parsed.unwrap();
    Some(x)
}

pub fn get_decimal<S: Into<String>>(prompt: S) -> Option<f32> {
    let input = read_input(format!("{}: ", prompt.into()));
    if input.is_empty() || !input.is_ascii() {
        println!("{}Warning: The input can only be a number!", Color::Yellow);
        flush_styles();
        return None;
    }
    for c in input.chars() {
        if !c.is_numeric() && c != '.' {
            println!("{}Warning: The input can only be a number!", Color::Yellow);
            flush_styles();
            return None;
        }
    }
    let parsed = input.parse::<f32>();
    if parsed.is_err() {
        eprintln!("Invalid input: failed to get decimal from input!");
        exit(1);
    }
    Some(parsed.unwrap())
}