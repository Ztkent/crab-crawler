use rusqlite::{params, Connection, Result, ToSql};
use std::error::Error;
use crate::config;
use crate::data;

// Connect to the sqlite database, and run any migrations
pub(crate) fn connect_sqlite_and_migrate(config: &config::Config) -> Result<Option<Connection>, Box<dyn Error>> {
    // Connect to sqlite
    let results_db;
    if !config.sqlite_enabled {
        results_db = Connection::open_in_memory()?;
    } else  {
        results_db = Connection::open(config.sqlite_path.clone())?;
    }

    // Handle any migrations to setup the database
    let migrations = get_sorted_migration_files()?;
    for migration in migrations {
        results_db.execute_batch(&migration)?;
    }
    Ok(Some(results_db))
}

pub(crate) fn insert_visited_site(conn: &Connection, visited_site: data::VisitedSite) -> Result<bool, Box<dyn Error>> {
    let visited_at = visited_site.visited_at().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute("
        INSERT INTO visited (url, referrer, last_visited_at, is_blocked) VALUES (?1, ?2, ?3, 0)
        ON CONFLICT(url) DO UPDATE SET referrer = ?2, last_visited_at = strftime('%Y-%m-%d %H:%M:%S', 'now'), is_blocked = 0;
        ", &[visited_site.url(), visited_site.referrer(), &visited_at])?;
    Ok(true)
}

pub(crate) fn insert_image(conn: &Connection, referrer: &String, url: &String, image: &Vec<u8>, name: &String, success: bool) -> Result<bool, Box<dyn Error>> {
    let success_as_string = if success { "1" } else { "0" };
    conn.execute("
        INSERT INTO images (referrer, url, image, name, success) VALUES (?1, ?2, ?3, ?4, ?5)
        ", params![&referrer, &url, image as &[u8], &name, success_as_string])?;
    Ok(true)
}

pub(crate) fn insert_html(conn: &Connection, url: &String, html: &String) -> Result<bool, Box<dyn Error>> {
    conn.execute("
        INSERT INTO html (url, html) VALUES (?1, ?2)
        ", &[url, html])?;
    Ok(true)
}

pub(crate) fn mark_url_complete(conn: &Connection, url: &String) -> Result<bool, Box<dyn Error>> {
    conn.execute("UPDATE visited SET is_complete = 1 WHERE url = ?1", &[url])?;
    Ok(true)
}

pub(crate) fn mark_url_blocked(conn: &Connection, url: &String, referrer: &String) -> Result<bool, Box<dyn Error>> {
    conn.execute("
        INSERT INTO visited (url, referrer, last_visited_at, is_blocked) VALUES (?1, ?2, strftime('%Y-%m-%d %H:%M:%S', 'now'), 1)
        ON CONFLICT(url) DO UPDATE SET referrer = ?2, last_visited_at = strftime('%Y-%m-%d %H:%M:%S', 'now'), is_blocked = 1;
        ", &[url, referrer])?;
    Ok(true)
}

#[allow(dead_code)] // Not in use right now.
pub(crate) fn is_previously_visited_url(conn: &Connection, url: &String) -> Result<Option<bool>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT 1 FROM visited WHERE url = ?1 LIMIT 1")?;
    let mut rows = stmt.query(&[url])?;
    let row = rows.next()?;
    match row {
        Some(_) => Ok(Some(true)),
        None => Ok(Some(false))
    }
}

pub(crate) fn is_previously_completed_url(conn: &Connection, url: &String) -> Result<Option<bool>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT 1 FROM visited WHERE url = ?1 AND is_complete = 1 LIMIT 1")?;
    let mut rows = stmt.query(&[url])?;
    let row = rows.next()?;
    match row {
        Some(_) => Ok(Some(true)),
        None => Ok(Some(false))
    }
}

pub(crate) fn connect_and_get_total_rows(config: &config::Config) -> Result<u64, Box<dyn Error>> {
    let results_db;
    if !config.sqlite_enabled {
        return Ok(0);
    } else  {
        results_db = Connection::open(config.sqlite_path.clone())?;
    }
    let mut stmt = results_db.prepare("SELECT COUNT(*) FROM visited")?;
    let mut rows = stmt.query(&[] as &[&dyn ToSql])?;

    match rows.next()? {
        Some(row) => {
            let count: i64 = row.get(0)?;
            Ok(count as u64)
        },
        None => Ok(0)
    }
}

pub(crate) fn connect_and_get_completed_rows(config: &config::Config) -> Result<u64, Box<dyn Error>> {
    let results_db;
    if !config.sqlite_enabled {
        return Ok(0);
    } else  {
        results_db = Connection::open(config.sqlite_path.clone())?;
    }
    let mut stmt = results_db.prepare("SELECT COUNT(*) FROM visited WHERE is_complete = 1")?;
    let mut rows = stmt.query(&[] as &[&dyn ToSql])?;

    match rows.next()? {
        Some(row) => {
            let count: i64 = row.get(0)?;
            Ok(count as u64)
        },
        None => Ok(0)
    }
}

// Get the sql migrations we've set. Include them in the binary, the user doesn't need to see them.
fn get_sorted_migration_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let migrations: Vec<String> = vec![
        include_str!("../db/migrations/012425_init.sql").to_string(),
    ];
    Ok(migrations)
}