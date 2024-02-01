-- PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS visited (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    referrer TEXT NOT NULL,
    last_visited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_complete BOOLEAN NOT NULL DEFAULT 0,
    is_blocked BOOLEAN NOT NULL DEFAULT 0,
    UNIQUE(url)
);

CREATE TABLE IF NOT EXISTS html (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    html TEXT,
    FOREIGN KEY(url) REFERENCES visited(url) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    referrer TEXT NOT NULL,
    url TEXT NOT NULL,
    image BLOB,
    name TEXT,
    success BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY(referrer) REFERENCES visited(url) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_visited_url ON visited(url);
CREATE INDEX IF NOT EXISTS idx_visited_referrer ON visited(referrer);
CREATE INDEX IF NOT EXISTS idx_images_url ON images(url);