CREATE TABLE IF NOT EXISTS visited (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    referrer TEXT NOT NULL,
    visited_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_visited_url ON visited(url);
CREATE INDEX IF NOT EXISTS idx_visited_referrer ON visited(referrer);
CREATE INDEX IF NOT EXISTS idx_visited_visited_at ON visited(visited_at);