use rusqlite::{Connection, params};
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA cache_size=-4000;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS memos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL,
                visibility TEXT NOT NULL DEFAULT 'private',
                password_hash TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
            CREATE TABLE IF NOT EXISTS memo_tags (
                memo_id INTEGER NOT NULL,
                tag TEXT NOT NULL COLLATE NOCASE,
                FOREIGN KEY (memo_id) REFERENCES memos(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS resources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                filename TEXT NOT NULL,
                original_name TEXT NOT NULL,
                mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                size INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
            CREATE TABLE IF NOT EXISTS rate_limits (
                key TEXT NOT NULL,
                action TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_rate_limits ON rate_limits (key, action, timestamp);"
        )?;
        conn.execute_batch("ALTER TABLE memos ADD COLUMN title TEXT NOT NULL DEFAULT ''").ok();
        conn.execute_batch("ALTER TABLE memos ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private'").ok();
        conn.execute_batch("ALTER TABLE memos ADD COLUMN password_hash TEXT").ok();
        Ok(Database { conn: Mutex::new(conn) })
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
            params![username, password_hash],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_user_by_username(&self, username: &str) -> rusqlite::Result<Option<(i64, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;
        match rows.next()? {
            Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?))),
            None => Ok(None),
        }
    }

    pub fn get_user_by_id(&self, id: i64) -> rusqlite::Result<Option<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, username FROM users WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some((row.get(0)?, row.get(1)?))),
            None => Ok(None),
        }
    }

    pub fn create_memo(&self, user_id: i64, title: &str, content: &str, visibility: &str, password_hash: Option<&str>) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO memos (user_id, title, content, visibility, password_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, title, content, visibility, password_hash],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_memos(&self, user_id: i64) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE user_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_memo_by_id(&self, memo_id: i64, user_id: i64) -> rusqlite::Result<Option<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE id = ?1 AND user_id = ?2"
        )?;
        let mut rows = stmt.query(params![memo_id, user_id])?;
        match rows.next()? {
            Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))),
            None => Ok(None),
        }
    }

    pub fn get_memo_public(&self, memo_id: i64) -> rusqlite::Result<Option<(i64, String, String, String, Option<String>, String, i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.title, m.content, m.visibility, m.password_hash, m.created_at, m.user_id, u.username
             FROM memos m
             JOIN users u ON m.user_id = u.id
             WHERE m.id = ?1"
        )?;
        let mut rows = stmt.query(params![memo_id])?;
        match rows.next()? {
            Some(row) => Ok(Some((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
                row.get(7)?,
            ))),
            None => Ok(None),
        }
    }

    pub fn get_sidebar_memos(&self, user_id: i64, limit: i64) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![user_id, limit], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_memos_paginated(&self, user_id: i64, limit: i64, offset: i64) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(params![user_id, limit, offset], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_resources_paginated(&self, user_id: i64, limit: i64, offset: i64) -> rusqlite::Result<Vec<(i64, String, String, String, i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, filename, original_name, mime_type, size, created_at FROM resources WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map(params![user_id, limit, offset], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        })?;
        let mut resources = Vec::new();
        for row in rows {
            resources.push(row?);
        }
        Ok(resources)
    }

    pub fn search_memos(&self, user_id: i64, query: &str) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE user_id = ?1 AND (title LIKE ?2 OR content LIKE ?2) ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![user_id, pattern], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_memos_by_date(&self, user_id: i64, date: &str) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, content, visibility, created_at FROM memos WHERE user_id = ?1 AND date(created_at) = ?2 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![user_id, date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_memos_by_tag(&self, user_id: i64, tag: &str) -> rusqlite::Result<Vec<(i64, String, String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.title, m.content, m.visibility, m.created_at FROM memos m
             JOIN memo_tags t ON m.id = t.memo_id
             WHERE m.user_id = ?1 AND t.tag = ?2
             ORDER BY m.created_at DESC"
        )?;
        let rows = stmt.query_map(params![user_id, tag], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })?;
        let mut memos = Vec::new();
        for row in rows {
            memos.push(row?);
        }
        Ok(memos)
    }

    pub fn get_memo_dates_in_month(&self, user_id: i64, year: i32, month: u32) -> rusqlite::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let month_start = format!("{}-{:02}-%", year, month);
        let mut stmt = conn.prepare(
            "SELECT DISTINCT date(created_at) FROM memos WHERE user_id = ?1 AND created_at LIKE ?2"
        )?;
        let rows = stmt.query_map(params![user_id, month_start], |row| {
            row.get::<_, String>(0)
        })?;
        let mut dates = Vec::new();
        for row in rows {
            dates.push(row?);
        }
        Ok(dates)
    }

    pub fn set_memo_tags(&self, memo_id: i64, tags: &[String]) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memo_tags WHERE memo_id = ?1", params![memo_id])?;
        for tag in tags {
            conn.execute(
                "INSERT OR IGNORE INTO memo_tags (memo_id, tag) VALUES (?1, ?2)",
                params![memo_id, tag],
            )?;
        }
        Ok(())
    }

    pub fn get_user_tags(&self, user_id: i64) -> rusqlite::Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.tag, COUNT(*) as cnt FROM memo_tags t
             JOIN memos m ON t.memo_id = m.id
             WHERE m.user_id = ?1
             GROUP BY t.tag ORDER BY cnt DESC, t.tag ASC"
        )?;
        let rows = stmt.query_map(params![user_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }
        Ok(tags)
    }

    pub fn update_memo(&self, memo_id: i64, user_id: i64, content: &str) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memos SET content = ?1 WHERE id = ?2 AND user_id = ?3",
            params![content, memo_id, user_id],
        )?;
        Ok(())
    }

    pub fn update_memo_visibility(&self, memo_id: i64, user_id: i64, visibility: &str, password_hash: Option<&str>) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE memos SET visibility = ?1, password_hash = ?2 WHERE id = ?3 AND user_id = ?4",
            params![visibility, password_hash, memo_id, user_id],
        )?;
        Ok(())
    }

    pub fn delete_memo(&self, memo_id: i64, user_id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM memos WHERE id = ?1 AND user_id = ?2", params![memo_id, user_id])?;
        Ok(())
    }

    pub fn create_resource(&self, user_id: i64, filename: &str, original_name: &str, mime_type: &str, size: i64) -> rusqlite::Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO resources (user_id, filename, original_name, mime_type, size) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, filename, original_name, mime_type, size],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_resource(&self, resource_id: i64, user_id: i64) -> rusqlite::Result<Option<(i64, String, String, String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, filename, original_name, mime_type, size FROM resources WHERE id = ?1 AND user_id = ?2"
        )?;
        let mut rows = stmt.query(params![resource_id, user_id])?;
        match rows.next()? {
            Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))),
            None => Ok(None),
        }
    }

    pub fn delete_resource(&self, resource_id: i64, user_id: i64) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM resources WHERE id = ?1 AND user_id = ?2", params![resource_id, user_id])?;
        Ok(())
    }

    pub fn get_resource_public(&self, resource_id: i64) -> rusqlite::Result<Option<(i64, String, String, String, i64, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, filename, original_name, mime_type, size, user_id FROM resources WHERE id = ?1"
        )?;
        let mut rows = stmt.query(params![resource_id])?;
        match rows.next()? {
            Some(row) => Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))),
            None => Ok(None),
        }
    }

    pub fn get_memos_referencing_resource(&self, ref_pattern: &str) -> rusqlite::Result<Vec<(i64, String, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, visibility, user_id FROM memos WHERE content LIKE ?1"
        )?;
        let rows = stmt.query_map(params![format!("%{}%", ref_pattern)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_memo_tags(&self, memo_id: i64) -> rusqlite::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT tag FROM memo_tags WHERE memo_id = ?1 ORDER BY tag ASC"
        )?;
        let rows = stmt.query_map(params![memo_id], |row| row.get(0))?;
        let mut tags = Vec::new();
        for r in rows {
            tags.push(r?);
        }
        Ok(tags)
    }

    pub fn check_and_record_rate_limit(
        &self,
        key: &str,
        action: &str,
        limit: i64,
        window_secs: i64,
    ) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        let window_start = now - window_secs;

        // Delete expired entries
        conn.execute(
            "DELETE FROM rate_limits WHERE timestamp < ?1",
            params![window_start],
        )?;

        // Count active entries in the window
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM rate_limits WHERE key = ?1 AND action = ?2 AND timestamp >= ?3"
        )?;
        let count: i64 = stmt.query_row(params![key, action, window_start], |row| row.get(0))?;

        if count >= limit {
            Ok(false)
        } else {
            conn.execute(
                "INSERT INTO rate_limits (key, action, timestamp) VALUES (?1, ?2, ?3)",
                params![key, action, now],
            )?;
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limits() {
        let db = Database::new(":memory:").unwrap();
        let key = "127.0.0.1";
        
        // Allowed 3 requests in 10 seconds
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        
        // 4th request should fail
        assert!(!db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        
        // Different action or different key should succeed
        assert!(db.check_and_record_rate_limit(key, "other_action", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit("192.168.1.1", "test", 3, 10).unwrap());
    }
}

