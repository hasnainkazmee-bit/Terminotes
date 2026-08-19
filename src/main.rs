use std::{
    env,
    fs::OpenOptions,
    io::Write,
};

struct Cli {
    argument: String,
    value: String,
}

fn main() {
    let cli = parse_args();

    match cli.argument.as_str() {
        "notes" => save_note(&cli.value),
        _ => println!("Unknown argument: {}", cli.argument),
    }
}

fn parse_args() -> Cli {
    let mut args = env::args().skip(1);

    let argument = args.next().expect("Missing argument");
    let value = args.next().expect("Missing value");

    Cli { argument, value }
}

fn save_note(note: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("notes.txt")
        .expect("Failed to open notes.txt");

    writeln!(file, "{note}")
        .expect("Failed to save note");

    println!("Saved!");
}