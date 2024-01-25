use rusqlite::{Connection, Result};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::collections::HashMap;
use crate::constants as consts;

// Connect to the sqlite database, and run any migrations
pub(crate) fn connect_sqlite_and_migrate() -> Result<Option<Connection>, Box<dyn Error>> {
    if !consts::SQLITE_ENABLED {
        return Ok(None);
    }

    // Connect to sqlite
    let results_db = match Connection::open(consts::SQLITE_PATH) {
        Ok(connection) => connection,
        Err(e) => {
            eprintln!("Failed to connect to SQLite: {}", e);
            return Err(e.into())
        }
    };

    // Get all .sql files in the db directory
    let migrations = get_sorted_migration_files()?;

    // Handle any migrations to setup the database
    for migration in migrations {
        results_db.execute(&migration, [])?;
    }
    Ok(Some(results_db))
}

// Get the contents of the sql migrations from the /db folder
fn get_sorted_migration_files() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut migrations: HashMap<String, String> = HashMap::new();
    let paths = fs::read_dir("db")?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    for path in paths {
        if path.extension() == Some(std::ffi::OsStr::new("sql")) {
            // Read the SQL file
            let mut file = File::open(&path)?;
            let mut sql = String::new();
            file.read_to_string(&mut sql)?;

            // Use the filename (without extension) as the key
            let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
            migrations.insert(filename, sql);
        }
    }
    // Sort the migrations by key (filename)
    let mut sorted_migration_list = migrations.into_iter().collect::<Vec<_>>();
    sorted_migration_list.sort_by(|a, b| a.0.cmp(&b.0));
    // Get a vector of just the SQL strings (values)
    let sorted_migration_list = sorted_migration_list.into_iter().map(|(_, v)| v).collect::<Vec<_>>();
    Ok(sorted_migration_list)
}