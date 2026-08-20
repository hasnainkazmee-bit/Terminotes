use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, Result};

struct Note {
    id: i64,
    content: String,
    tag: Option<String>,
    created_at: i64,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        show_usage();
        return Ok(());
    }

    match args[0].as_str() {
        "list" => {
            let tag = extract_tag(&args);
            list_notes(tag)?;
        }

        "tags" => {
            list_tags()?;
        }

        "search" => {
            search_notes(&args[1..])?;
        }

        "today" => {
            list_today()?;
        }

        _ => {
            create_note(&args)?;
        }
    }

    Ok(())
}

// ------------------------------------------------------------
// DATABASE
// ------------------------------------------------------------

fn get_db_connection() -> Result<Connection> {
    let mut db_path = dirs::data_dir()
        .expect("Unable to locate your data directory");

    db_path.push("terminote");

    std::fs::create_dir_all(&db_path)
        .expect("Failed to create Terminotes directory");

    db_path.push("terminotes.db");

    let conn = Connection::open(db_path)?;

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            tag TEXT,
            created_at INTEGER NOT NULL
        )
        ",
        [],
    )?;

    Ok(conn)
}

// ------------------------------------------------------------
// CREATE
// ------------------------------------------------------------

fn create_note(args: &[String]) -> Result<()> {
    let mut content_parts = Vec::new();
    let mut tag: Option<String> = None;

    for arg in args {
        if let Some(value) = arg.strip_prefix("--") {
            if !value.is_empty() {
                tag = Some(value.to_string());
            }
        } else {
            content_parts.push(arg.as_str());
        }
    }

    let content = content_parts.join(" ").trim().to_string();

    if content.is_empty() {
        println!("Nothing to Terminote.");
        return Ok(());
    }

    let timestamp = current_timestamp();

    let conn = get_db_connection()?;

    conn.execute(
        "
        INSERT INTO notes (content, tag, created_at)
        VALUES (?1, ?2, ?3)
        ",
        params![content, tag, timestamp],
    )?;

    println!("Terminoted.");

    Ok(())
}

// ------------------------------------------------------------
// LIST
// ------------------------------------------------------------

fn list_notes(tag: Option<String>) -> Result<()> {
    let notes = get_notes(tag)?;

    if notes.is_empty() {
        println!();
        println!("No Terminotes yet.");
        return Ok(());
    }

    println!();
    println!("Terminotes");
    println!("────────────────────────────────────────");

    for note in &notes {
        print_note(note);
    }

    println!();

    Ok(())
}

fn get_notes(tag: Option<String>) -> Result<Vec<Note>> {
    let conn = get_db_connection()?;

    let mut notes = Vec::new();

    match tag {
        Some(tag) => {
            let mut stmt = conn.prepare(
                "
                SELECT id, content, tag, created_at
                FROM notes
                WHERE tag = ?1
                ORDER BY created_at DESC
                ",
            )?;

            let rows = stmt.query_map([tag], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    tag: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?;

            for row in rows {
                notes.push(row?);
            }
        }

        None => {
            let mut stmt = conn.prepare(
                "
                SELECT id, content, tag, created_at
                FROM notes
                ORDER BY created_at DESC
                ",
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    tag: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?;

            for row in rows {
                notes.push(row?);
            }
        }
    }

    Ok(notes)
}

// ------------------------------------------------------------
// SEARCH
// ------------------------------------------------------------

fn search_notes(words: &[String]) -> Result<()> {
    if words.is_empty() {
        println!("What do you want to search for?");
        return Ok(());
    }

    let search = words.join(" ");
    let pattern = format!("%{}%", search);

    let conn = get_db_connection()?;

    let mut stmt = conn.prepare(
        "
        SELECT id, content, tag, created_at
        FROM notes
        WHERE content LIKE ?1
        ORDER BY created_at DESC
        ",
    )?;

    let rows = stmt.query_map([pattern], |row| {
        Ok(Note {
            id: row.get(0)?,
            content: row.get(1)?,
            tag: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    let mut found = false;

    println!();
    println!("Search: {}", search);
    println!("────────────────────────────────────────");

    for row in rows {
        let note = row?;

        found = true;

        print_note(&note);
    }

    if !found {
        println!();
        println!("Nothing found.");
    }

    println!();

    Ok(())
}

// ------------------------------------------------------------
// TAGS
// ------------------------------------------------------------

fn list_tags() -> Result<()> {
    let conn = get_db_connection()?;

    let mut stmt = conn.prepare(
        "
        SELECT tag, COUNT(*)
        FROM notes
        WHERE tag IS NOT NULL
        GROUP BY tag
        ORDER BY COUNT(*) DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
        ))
    })?;

    println!();
    println!("Tags");
    println!("────────────────────────────────────────");

    let mut found = false;

    for row in rows {
        let (tag, count) = row?;

        found = true;

        println!("{:<15} {}", tag, count);
    }

    if !found {
        println!("No tags yet.");
    }

    println!();

    Ok(())
}

// ------------------------------------------------------------
// TODAY
// ------------------------------------------------------------

fn list_today() -> Result<()> {
    let now = current_timestamp();

    let day = 60 * 60 * 24;

    let start_of_day = now - (now % day);

    let conn = get_db_connection()?;

    let mut stmt = conn.prepare(
        "
        SELECT id, content, tag, created_at
        FROM notes
        WHERE created_at >= ?1
        ORDER BY created_at DESC
        ",
    )?;

    let rows = stmt.query_map([start_of_day], |row| {
        Ok(Note {
            id: row.get(0)?,
            content: row.get(1)?,
            tag: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;

    println!();
    println!("Today's Terminotes");
    println!("────────────────────────────────────────");

    let mut found = false;

    for row in rows {
        let note = row?;

        found = true;

        print_note(&note);
    }

    if !found {
        println!();
        println!("Nothing Terminoted today.");
    }

    println!();

    Ok(())
}

// ------------------------------------------------------------
// OUTPUT
// ------------------------------------------------------------

fn print_note(note: &Note) {
    let tag = match &note.tag {
        Some(tag) => tag.as_str(),
        None => "general",
    };

    println!();
    println!("#{}  {}", note.id, note.content);
    println!("     {} · {}", tag, format_date(note.created_at));
}

// ------------------------------------------------------------
// DATE / TIME
// ------------------------------------------------------------

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is before UNIX epoch")
        .as_secs() as i64
}

fn format_date(timestamp: i64) -> String {
    let now = current_timestamp();

    let difference = now - timestamp;

    if difference < 60 {
        return "just now".to_string();
    }

    if difference < 60 * 60 {
        let minutes = difference / 60;

        return format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        );
    }

    if difference < 60 * 60 * 24 {
        let hours = difference / (60 * 60);

        return format!(
            "{} hour{} ago",
            hours,
            if hours == 1 { "" } else { "s" }
        );
    }

    if difference < 60 * 60 * 48 {
        return "yesterday".to_string();
    }

    let days = difference / (60 * 60 * 24);

    format!(
        "{} day{} ago",
        days,
        if days == 1 { "" } else { "s" }
    )
}

// ------------------------------------------------------------
// ARGUMENTS
// ------------------------------------------------------------

fn extract_tag(args: &[String]) -> Option<String> {
    args.iter()
        .find_map(|arg| {
            arg.strip_prefix("--")
                .filter(|value| !value.is_empty())
                .map(String::from)
        })
}

// ------------------------------------------------------------
// HELP
// ------------------------------------------------------------

fn show_usage() {
    println!();
    println!("Terminotes");
    println!("A tiny memory for your terminal.");
    println!();

    println!("Usage:");
    println!();
    println!("  terminote \"buy milk\"");
    println!("  terminote \"buy milk\" --todo");
    println!();
    println!("  terminote list");
    println!("  terminote list --todo");
    println!("  terminote --todo");
    println!();
    println!("  terminote tags");
    println!("  terminote search rust");
    println!("  terminote today");
    println!();
}