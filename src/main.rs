use std::env;
use rusqlite::{Connection, Result};

struct Cli {
    argument: String,
    value: String,
}

fn main() -> Result<()> {
    // 1. If they don't provide any arguments at all
    if env::args().len() < 2 {
        show_usage();
        return Ok(());
    }

    let cli = parse_args();

    match cli.argument.as_str() {
        "tn" => {
            // 2. If they type 'tn' but forgot the note text
            if cli.value.trim().is_empty() {
                println!("Error: Missing note text!\n");
                show_usage();
                return Ok(());
            }
            save_note(&cli.value)?;
        }
        "list" => list_notes()?,
        _ => {
            println!("Unknown argument: {}\n", cli.argument);
            show_usage();
        }
    }

    Ok(())
}

fn show_usage() {
    println!("Usage: terminotes <command> [value]");
    println!("Commands:");
    println!("  tn \"your note text here\"  -> Saves a new note");
    println!("  list                    -> Lists all saved notes");
}

fn parse_args() -> Cli {
    let mut args = env::args().skip(1);

    let argument = args.next().expect("Missing argument");
    let value = args.next().unwrap_or_default();

    Cli { argument, value }
}

fn get_db_connection() -> Result<Connection> {
    let mut db_path = dirs::data_dir().expect("Could not find data directory");
    db_path.push("terminote");
    std::fs::create_dir_all(&db_path).expect("Failed to create app directory");
    db_path.push("terminotes.db");

    Connection::open(db_path)
}

fn save_note(note: &str) -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO notes (content) VALUES (?1)",
        [note],
    )?;

    println!("Saved to database!");
    Ok(())
}

fn list_notes() -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL
        )",
        [],
    )?;

    let mut stmt = conn.prepare("SELECT id, content FROM notes")?;
    let note_iter = stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    })?;

    println!("--- Your Saved Notes ---");
    for note in note_iter {
        let (id, content) = note?;
        println!("{}. {}", id, content);
    }
    
    Ok(())
}
