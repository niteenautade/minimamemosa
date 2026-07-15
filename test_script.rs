use rusqlite::Connection;
fn main() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memo_tags (
            memo_id INTEGER NOT NULL,
            tag TEXT NOT NULL COLLATE NOCASE,
            UNIQUE(tag)
        );"
    ).unwrap();
}
