use rtb::modules::hsql::parser::HsqlDatabase;
use rtb::modules::hsql::sqlite_bridge::SqliteBridge;
use rusqlite::Connection;
use std::fs;

#[test]
fn test_hsql_load_directory() {
    let db = HsqlDatabase::load_from_path("examples/sample_hsqldb/").expect("Failed to load HSQLDB dir");
    assert_eq!(db.name, "mydb");
    assert!(db.tables.contains_key("USERS"));
    assert!(db.tables.contains_key("ORDERS"));
    assert!(db.tables.contains_key("PRODUCTS"));

    let users = db.tables.get("USERS").unwrap();
    assert_eq!(users.rows.len(), 3);
    assert_eq!(users.columns.len(), 5);
}

#[test]
fn test_hsql_load_single_file() {
    let db = HsqlDatabase::load_from_path("examples/sample_hsqldb/mydb.script").expect("Failed to load HSQLDB script");
    assert_eq!(db.name, "mydb");
    assert!(db.tables.contains_key("USERS"));
}

#[test]
fn test_hsql_query_execution() {
    let db = HsqlDatabase::load_from_path("examples/sample_hsqldb/").expect("Failed to load HSQLDB");
    let conn = SqliteBridge::create_in_memory_from_hsql(&db).expect("Failed to create SQLite bridge");

    // 1. Simple query
    let (cols, rows) = SqliteBridge::query_to_table(&conn, "SELECT USERNAME, EMAIL FROM USERS WHERE ROLE = 'ADMIN'").unwrap();
    assert_eq!(cols, vec!["USERNAME", "EMAIL"]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "alice");
    assert_eq!(rows[0][1], "alice@company.com");

    // 2. JOIN query
    let (cols, rows) = SqliteBridge::query_to_table(
        &conn,
        "SELECT u.USERNAME, o.ORDER_ID, o.AMOUNT FROM USERS u JOIN ORDERS o ON u.ID = o.USER_ID ORDER BY o.ORDER_ID ASC"
    ).unwrap();
    assert_eq!(cols, vec!["USERNAME", "ORDER_ID", "AMOUNT"]);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0], vec!["alice", "101", "499.99"]);
    assert_eq!(rows[1], vec!["alice", "102", "129.5"]);
    assert_eq!(rows[2], vec!["bob_dev", "103", "89"]);
}

#[test]
fn test_hsql_sqlite_export() {
    let test_db_path = "examples/sample_hsqldb/test_export.db";
    let _ = fs::remove_file(test_db_path);

    let db = HsqlDatabase::load_from_path("examples/sample_hsqldb/").unwrap();
    SqliteBridge::export_to_file(&db, test_db_path).expect("Failed to export HSQLDB to SQLite");

    // Verify written file
    let conn = Connection::open(test_db_path).expect("Failed to open exported SQLite file");
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM USERS").unwrap();
    let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
    assert_eq!(count, 3);

    let _ = fs::remove_file(test_db_path);
}
