use std::fs::OpenOptions;
use std::io::{Write};

struct Cli {
    argument: String,
    value: String,
}

fn main(){
    // define argument and value
    let argument = std::env::args().nth(1).expect("Error");
    let value = std::env::args().nth(2).expect("Error");


    // Create an instance of the cli
    let arg = Cli {argument, value};

    // print value if argument is "notes"
    if arg.argument == "notes" {

        // Open the file in Append mode, create if missing

        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("notes.txt")
        .expect("No file named 'notes.txt' found!");

        // Format the text with a new line
        
        let log_entry = format!("{}\n", arg.value);

        // Write the note in notes.txt

        file.write_all(log_entry.as_bytes()).expect("Failed to save note");
        println!("Saved!")
    };
}

