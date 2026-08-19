use std::env;
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        show_usage();
        return Ok(());
    }

    // Isolate a dynamic tag flag if passed anywhere in the arguments (e.g., --idea)
    let mut search_tag: Option<String> = None;
    if let Some(flag_index) = args.iter().position(|a| a.starts_with("--")) {
        let clean_tag = args[flag_index].replacen("--", "", 1).trim().to_string();
        if !clean_tag.is_empty() {
            search_tag = Some(clean_tag);
        }
        args.remove(flag_index); 
    }

    let first_arg = args.first().map(|s| s.as_str()).unwrap_or("");

    match first_arg {
        "list" => list_notes(search_tag)?,
        "tags" => list_tags()?, // New command arm to view all user-defined types
        _ => {
            let content = args.join(" ").trim().to_string();

            if content.is_empty() {
                println!("Error: Missing note content.\n");
                show_usage();
                return Ok(());
            }

            let tag = search_tag.unwrap_or_else(|| "general".to_string());
            save_note(&content, &tag)?;
        }
    }

    Ok(())
}

fn show_usage() {
    println!("Usage:");
    println!("  tn \"your note text here\"     -> Append a new raw note");
    println!("  tn \"your note\" --<any-tag>  -> Append a note with a custom dynamic tag");
    println!("  tn list                     -> Stream all entries to stdout");
    println!("  tn list --<any-tag>         -> Filter results by a specific tag type");
    println!("  tn tags                     -> List all custom tags you have defined");
}

fn get_db_connection() -> Result<Connection> {
    let mut db_path = dirs::data_dir().expect("Unable to locate core data directory");
    db_path.push("terminote");
    std::fs::create_dir_all(&db_path).expect("Failed to build footprint");
    db_path.push("terminotes.db");

    Connection::open(db_path)
}

fn save_note(note: &str, tag: &str) -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            tag TEXT DEFAULT 'general'
        )",
        [],
    )?;

    conn.execute(
        "INSERT INTO notes (content, tag) VALUES (?1, ?2)",
        [note, tag],
    )?;

    println!("Saved under [{}]!", tag);
    Ok(())
}

fn list_notes(filter_tag: Option<String>) -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            tag TEXT DEFAULT 'general'
        )",
        [],
    )?;

    let (query, params) = match &filter_tag {
        Some(t) => ("SELECT id, content, tag FROM notes WHERE tag = ?1", vec![t.clone()]),
        None => ("SELECT id, content, tag FROM notes", vec![]),
    };

    let mut stmt = conn.prepare(query)?;
    let note_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
    })?;

    match &filter_tag {
        Some(t) => println!("--- Filtered Notes [{}] ---", t),
        None => println!("--- All Journal Entries ---"),
    }

    let mut count = 0;
    for note in note_iter {
        let (id, content, tag) = note?;
        count += 1;
        if tag == "general" {
            println!("{}. {}", id, content);
        } else {
            println!("{}. [{}] {}", id, tag, content);
        }
    }

    if count == 0 && filter_tag.is_some() {
        println!("(No notes found with that tag)");
    }
    
    Ok(())
}

// Scans database for all uniquely generated tags
fn list_tags() -> Result<()> {
    let conn = get_db_connection()?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            tag TEXT DEFAULT 'general'
        )",
        [],
    )?;

    // Uses SQL DISTINCT to filter out matching duplicate tag strings
    let mut stmt = conn.prepare("SELECT DISTINCT tag FROM notes WHERE tag != 'general' ORDER BY tag ASC")?;
    let tag_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

    println!("--- Defined Custom Tags ---");
    let mut count = 0;
    for tag in tag_iter {
        println!("  --{}", tag?);
        count += 1;
    }

    if count == 0 {
        println!("  (No custom tags created yet)");
    }

    Ok(())
}
