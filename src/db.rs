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

    pub fn get_memo_has_password(&self, memo_id: i64, user_id: i64) -> bool {
        let conn = self.conn.lock().unwrap();
        let stmt = conn.prepare(
            "SELECT password_hash IS NOT NULL AND password_hash != '' FROM memos WHERE id = ?1 AND user_id = ?2"
        ).ok();
        match stmt {
            Some(mut s) => s.query(params![memo_id, user_id]).ok().and_then(|mut rows| {
                rows.next().ok().flatten().and_then(|row| row.get::<_, bool>(0).ok())
            }).unwrap_or(false),
            None => false,
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
    use chrono::Datelike;

    fn setup_db() -> Database {
        Database::new(":memory:").unwrap()
    }

    fn create_user(db: &Database) -> i64 {
        let hash = "$2b$12$LJ3m4ys3Lk0TSwHnbfOMiOXPmZQmQmQmQmQmQmQmQmQmQmQm".to_string();
        db.create_user("testuser", &hash).unwrap()
    }

    // ── User Tests ──

    #[test]
    fn test_create_and_get_user_by_username() {
        let db = setup_db();
        let id = create_user(&db);
        let user = db.get_user_by_username("testuser").unwrap().unwrap();
        assert_eq!(user.0, id);
        assert_eq!(user.1, "testuser");
    }

    #[test]
    fn test_get_user_by_username_not_found() {
        let db = setup_db();
        assert!(db.get_user_by_username("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_get_user_by_id() {
        let db = setup_db();
        let id = create_user(&db);
        let user = db.get_user_by_id(id).unwrap().unwrap();
        assert_eq!(user.0, id);
        assert_eq!(user.1, "testuser");
    }

    #[test]
    fn test_get_user_by_id_not_found() {
        let db = setup_db();
        assert!(db.get_user_by_id(999).unwrap().is_none());
    }

    #[test]
    fn test_create_duplicate_username_fails() {
        let db = setup_db();
        let hash = "hash".to_string();
        db.create_user("testuser", &hash).unwrap();
        let result = db.create_user("testuser", &hash);
        assert!(result.is_err());
    }

    // ── Memo CRUD Tests ──

    #[test]
    fn test_create_memo() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "Test Title", "Hello world", "private", None).unwrap();
        assert!(id > 0);
    }

    #[test]
    fn test_get_memos_empty() {
        let db = setup_db();
        let user_id = create_user(&db);
        let memos = db.get_memos(user_id).unwrap();
        assert!(memos.is_empty());
    }

    #[test]
    fn test_get_memos() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "Title1", "Content1", "private", None).unwrap();
        db.create_memo(user_id, "Title2", "Content2", "public", None).unwrap();
        let memos = db.get_memos(user_id).unwrap();
        assert_eq!(memos.len(), 2);
    }

    #[test]
    fn test_get_memos_other_user_not_visible() {
        let db = setup_db();
        let user1 = create_user(&db);
        let hash = "hash".to_string();
        let user2 = db.create_user("user2", &hash).unwrap();
        db.create_memo(user1, "T1", "C1", "private", None).unwrap();
        let memos = db.get_memos(user2).unwrap();
        assert!(memos.is_empty());
    }

    #[test]
    fn test_get_memo_by_id() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "Title", "Content", "private", None).unwrap();
        let memo = db.get_memo_by_id(id, user_id).unwrap().unwrap();
        assert_eq!(memo.0, id);
        assert_eq!(memo.1, "Title");
        assert_eq!(memo.2, "Content");
        assert_eq!(memo.3, "private");
    }

    #[test]
    fn test_get_memo_by_id_not_found() {
        let db = setup_db();
        assert!(db.get_memo_by_id(999, 1).unwrap().is_none());
    }

    #[test]
    fn test_get_memo_by_id_wrong_user() {
        let db = setup_db();
        let user1 = create_user(&db);
        let hash = "hash".to_string();
        let user2 = db.create_user("user2", &hash).unwrap();
        let id = db.create_memo(user1, "T", "C", "private", None).unwrap();
        assert!(db.get_memo_by_id(id, user2).unwrap().is_none());
    }

    #[test]
    fn test_update_memo() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "Old", "Old content", "private", None).unwrap();
        db.update_memo(id, user_id, "Updated content").unwrap();
        let memo = db.get_memo_by_id(id, user_id).unwrap().unwrap();
        assert_eq!(memo.2, "Updated content");
    }

    #[test]
    fn test_update_memo_wrong_user_noop() {
        let db = setup_db();
        let user1 = create_user(&db);
        let hash = "hash".to_string();
        let user2 = db.create_user("user2", &hash).unwrap();
        let id = db.create_memo(user1, "T", "C", "private", None).unwrap();
        db.update_memo(id, user2, "new").unwrap();
        let memo = db.get_memo_by_id(id, user1).unwrap().unwrap();
        assert_eq!(memo.2, "C");
    }

    #[test]
    fn test_delete_memo() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        db.delete_memo(id, user_id).unwrap();
        assert!(db.get_memo_by_id(id, user_id).unwrap().is_none());
    }

    #[test]
    fn test_delete_memo_wrong_user_fails() {
        let db = setup_db();
        let user1 = create_user(&db);
        let hash = "hash".to_string();
        let user2 = db.create_user("user2", &hash).unwrap();
        let id = db.create_memo(user1, "T", "C", "private", None).unwrap();
        let result = db.delete_memo(id, user2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_memos_paginated() {
        let db = setup_db();
        let user_id = create_user(&db);
        for i in 0..5 {
            db.create_memo(user_id, &format!("T{}", i), &format!("C{}", i), "private", None).unwrap();
        }
        let page1 = db.get_memos_paginated(user_id, 3, 0).unwrap();
        assert_eq!(page1.len(), 3);
        let page2 = db.get_memos_paginated(user_id, 3, 3).unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_sidebar_memos_limit() {
        let db = setup_db();
        let user_id = create_user(&db);
        for i in 0..10 {
            db.create_memo(user_id, &format!("T{}", i), &format!("C{}", i), "private", None).unwrap();
        }
        let memos = db.get_sidebar_memos(user_id, 5).unwrap();
        assert_eq!(memos.len(), 5);
    }

    // ── Visibility Tests ──

    #[test]
    fn test_update_memo_visibility() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        db.update_memo_visibility(id, user_id, "public", None).unwrap();
        let memo = db.get_memo_by_id(id, user_id).unwrap().unwrap();
        assert_eq!(memo.3, "public");
    }

    #[test]
    fn test_get_memo_has_password() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "protected", Some("hash")).unwrap();
        assert!(db.get_memo_has_password(id, user_id));
    }

    #[test]
    fn test_get_memo_has_no_password() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        assert!(!db.get_memo_has_password(id, user_id));
    }

    #[test]
    fn test_get_memo_has_password_not_found() {
        let db = setup_db();
        assert!(!db.get_memo_has_password(999, 1));
    }

    #[test]
    fn test_get_memo_public() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "public", None).unwrap();
        let memo = db.get_memo_public(id).unwrap().unwrap();
        assert_eq!(memo.0, id);
        assert_eq!(memo.7, "testuser");
    }

    #[test]
    fn test_get_memo_public_not_found() {
        let db = setup_db();
        assert!(db.get_memo_public(999).unwrap().is_none());
    }

    // ── Tag Tests ──

    #[test]
    fn test_set_and_get_memo_tags() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        db.set_memo_tags(id, &["rust".to_string(), "test".to_string()]).unwrap();
        let tags = db.get_memo_tags(id).unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust".to_string()));
        assert!(tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_set_memo_tags_replaces_old() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        db.set_memo_tags(id, &["old".to_string()]).unwrap();
        db.set_memo_tags(id, &["new".to_string()]).unwrap();
        let tags = db.get_memo_tags(id).unwrap();
        assert_eq!(tags, vec!["new".to_string()]);
    }

    #[test]
    fn test_get_user_tags() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id1 = db.create_memo(user_id, "T1", "C1 #rust", "private", None).unwrap();
        let id2 = db.create_memo(user_id, "T2", "C2 #rust #web", "private", None).unwrap();
        db.set_memo_tags(id1, &["rust".to_string()]).unwrap();
        db.set_memo_tags(id2, &["rust".to_string(), "web".to_string()]).unwrap();
        let tags = db.get_user_tags(user_id).unwrap();
        assert_eq!(tags.len(), 2);
        let rust_count = tags.iter().find(|(n, _)| n == "rust").map(|(_, c)| *c).unwrap();
        assert_eq!(rust_count, 2);
    }

    #[test]
    fn test_get_memos_by_tag() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        db.set_memo_tags(id, &["rust".to_string()]).unwrap();
        let memos = db.get_memos_by_tag(user_id, "rust").unwrap();
        assert_eq!(memos.len(), 1);
        let memos_empty = db.get_memos_by_tag(user_id, "nonexistent").unwrap();
        assert!(memos_empty.is_empty());
    }

    #[test]
    fn test_get_memo_tags_empty() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let tags = db.get_memo_tags(id).unwrap();
        assert!(tags.is_empty());
    }

    // ── Search Tests ──

    #[test]
    fn test_search_memos() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "Shopping", "Buy milk and eggs", "private", None).unwrap();
        db.create_memo(user_id, "Rust", "Learn Rust programming", "private", None).unwrap();
        let results = db.search_memos(user_id, "Rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, "Rust");
    }

    #[test]
    fn test_search_memos_no_results() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let results = db.search_memos(user_id, "zzzzz").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_memos_empty_query() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let results = db.search_memos(user_id, "").unwrap();
        assert_eq!(results.len(), 1);
    }

    // ── Date Query Tests ──

    #[test]
    fn test_get_memos_by_date() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let memo = db.get_memo_by_id(id, user_id).unwrap().unwrap();
        let date = memo.4[..10].to_string();
        let memos = db.get_memos_by_date(user_id, &date).unwrap();
        assert_eq!(memos.len(), 1);
    }

    #[test]
    fn test_get_memos_by_date_no_match() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let memos = db.get_memos_by_date(user_id, "2099-01-01").unwrap();
        assert!(memos.is_empty());
    }

    #[test]
    fn test_get_memo_dates_in_month() {
        let db = setup_db();
        let user_id = create_user(&db);
        db.create_memo(user_id, "T", "C", "private", None).unwrap();
        let now = chrono::Utc::now();
        let dates = db.get_memo_dates_in_month(user_id, now.year(), now.month()).unwrap();
        assert_eq!(dates.len(), 1);
    }

    #[test]
    fn test_get_memo_dates_in_month_empty() {
        let db = setup_db();
        let user_id = create_user(&db);
        let dates = db.get_memo_dates_in_month(user_id, 2000, 1).unwrap();
        assert!(dates.is_empty());
    }

    // ── Resource Tests ──

    #[test]
    fn test_create_and_get_resource() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_resource(user_id, "file.txt", "original.txt", "text/plain", 1024).unwrap();
        let res = db.get_resource(id, user_id).unwrap().unwrap();
        assert_eq!(res.0, id);
        assert_eq!(res.1, "file.txt");
        assert_eq!(res.2, "original.txt");
        assert_eq!(res.3, "text/plain");
        assert_eq!(res.4, 1024);
    }

    #[test]
    fn test_get_resource_not_found() {
        let db = setup_db();
        assert!(db.get_resource(999, 1).unwrap().is_none());
    }

    #[test]
    fn test_get_resource_wrong_user() {
        let db = setup_db();
        let user1 = create_user(&db);
        let hash = "hash".to_string();
        let user2 = db.create_user("user2", &hash).unwrap();
        let id = db.create_resource(user1, "f.txt", "o.txt", "text/plain", 100).unwrap();
        assert!(db.get_resource(id, user2).unwrap().is_none());
    }

    #[test]
    fn test_get_resource_public() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_resource(user_id, "f.txt", "o.txt", "text/plain", 100).unwrap();
        let res = db.get_resource_public(id).unwrap().unwrap();
        assert_eq!(res.0, id);
        assert_eq!(res.5, user_id);
    }

    #[test]
    fn test_get_resource_public_not_found() {
        let db = setup_db();
        assert!(db.get_resource_public(999).unwrap().is_none());
    }

    #[test]
    fn test_get_resources_paginated() {
        let db = setup_db();
        let user_id = create_user(&db);
        for i in 0..5 {
            db.create_resource(user_id, &format!("f{}.txt", i), "o.txt", "text/plain", i).unwrap();
        }
        let page1 = db.get_resources_paginated(user_id, 3, 0).unwrap();
        assert_eq!(page1.len(), 3);
        let page2 = db.get_resources_paginated(user_id, 3, 3).unwrap();
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_delete_resource() {
        let db = setup_db();
        let user_id = create_user(&db);
        let id = db.create_resource(user_id, "f.txt", "o.txt", "text/plain", 100).unwrap();
        db.delete_resource(id, user_id).unwrap();
        assert!(db.get_resource(id, user_id).unwrap().is_none());
    }

    #[test]
    fn test_get_memos_referencing_resource() {
        let db = setup_db();
        let user_id = create_user(&db);
        let res_id = db.create_resource(user_id, "f.txt", "o.txt", "text/plain", 100).unwrap();
        db.create_memo(user_id, "T", &format!("See [file](/resources/{})", res_id), "private", None).unwrap();
        let memos = db.get_memos_referencing_resource(&format!("/resources/{}", res_id)).unwrap();
        assert_eq!(memos.len(), 1);
    }

    #[test]
    fn test_get_memos_referencing_resource_empty() {
        let db = setup_db();
        let memos = db.get_memos_referencing_resource("/resources/999").unwrap();
        assert!(memos.is_empty());
    }

    // ── Rate Limiting Tests ──

    #[test]
    fn test_rate_limits_basic() {
        let db = setup_db();
        let key = "127.0.0.1";
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(!db.check_and_record_rate_limit(key, "test", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "other_action", 3, 10).unwrap());
        assert!(db.check_and_record_rate_limit("192.168.1.1", "test", 3, 10).unwrap());
    }

    #[test]
    fn test_rate_limits_limit_of_one() {
        let db = setup_db();
        assert!(db.check_and_record_rate_limit("ip", "login", 1, 10).unwrap());
        assert!(!db.check_and_record_rate_limit("ip", "login", 1, 10).unwrap());
    }

    #[test]
    fn test_rate_limits_zero_limit() {
        let db = setup_db();
        assert!(!db.check_and_record_rate_limit("ip", "test", 0, 10).unwrap());
    }

    #[test]
    fn test_rate_limits_expiry_allows_again() {
        let db = setup_db();
        assert!(db.check_and_record_rate_limit("ip", "test", 3, 1).unwrap());
        assert!(db.check_and_record_rate_limit("ip", "test", 3, 1).unwrap());
        assert!(db.check_and_record_rate_limit("ip", "test", 3, 1).unwrap());
        assert!(!db.check_and_record_rate_limit("ip", "test", 3, 1).unwrap());
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(db.check_and_record_rate_limit("ip", "test", 3, 1).unwrap());
    }

    #[test]
    fn test_rate_limits_multiple_actions_independent() {
        let db = setup_db();
        let key = "127.0.0.1";
        assert!(db.check_and_record_rate_limit(key, "login", 1, 10).unwrap());
        assert!(!db.check_and_record_rate_limit(key, "login", 1, 10).unwrap());
        assert!(db.check_and_record_rate_limit(key, "signup", 1, 10).unwrap());
        assert!(!db.check_and_record_rate_limit(key, "signup", 1, 10).unwrap());
    }

    #[test]
    fn test_rate_limits_different_keys_independent() {
        let db = setup_db();
        assert!(db.check_and_record_rate_limit("ip1", "test", 1, 10).unwrap());
        assert!(!db.check_and_record_rate_limit("ip1", "test", 1, 10).unwrap());
        assert!(db.check_and_record_rate_limit("ip2", "test", 1, 10).unwrap());
    }

    #[test]
    fn test_rate_limits_negative_window() {
        let db = setup_db();
        assert!(db.check_and_record_rate_limit("ip", "test", 1, -1).unwrap());
        assert!(db.check_and_record_rate_limit("ip", "test", 1, -1).unwrap());
    }
}

