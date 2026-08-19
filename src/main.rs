use std::env;
use rusqlite::{Connection, Result};

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        show_usage();
        return Ok(());
    }

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
        "tags" => list_tags()?, 
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
    println!("  tn ltags                     -> List all custom tags you have defined");
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

    // Keep this minimal confirmation message standard
    println!("Saved!");
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

    // No headers. Just raw, tab-separated stdout values for easy stream parsing.
    for note in note_iter {
        let (id, content, tag) = note?;
        println!("{}\t{}\t{}", id, tag, content);
    }
    
    Ok(())
}

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

    let mut stmt = conn.prepare("SELECT DISTINCT tag FROM notes WHERE tag != 'general' ORDER BY tag ASC")?;
    let tag_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

    // Output raw clean values only
    for tag in tag_iter {
        println!("{}", tag?);
    }

    Ok(())
}
