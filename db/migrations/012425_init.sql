CREATE TABLE IF NOT EXISTS visited (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    referrer TEXT NOT NULL,
    last_visited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_complete BOOLEAN NOT NULL DEFAULT 0,
    is_blocked BOOLEAN NOT NULL DEFAULT 0,
    UNIQUE(url)
);

CREATE INDEX IF NOT EXISTS idx_visited_url ON visited(url);
CREATE INDEX IF NOT EXISTS idx_visited_referrer ON visited(referrer);
CREATE INDEX IF NOT EXISTS idx_visited_visited_at ON visited(last_visited_at);
CREATE INDEX IF NOT EXISTS idx_visited_is_complete ON visited(is_complete);
CREATE INDEX IF NOT EXISTS idx_visited_is_blocked ON visited(is_blocked);