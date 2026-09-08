use rtb::modules::xlsx::loader::{query_to_table, ExcelCatalog};
use rusqlite::Connection;
use std::fs;

#[test]
fn test_load_excel_single_file() {
    let catalog = ExcelCatalog::load_from_path("examples/sample_sales.xlsx").expect("Failed to load Excel file");
    assert_eq!(catalog.workbooks.len(), 1);

    let wb = &catalog.workbooks[0];
    assert!(wb.sheets.contains_key("Customers"));
    assert!(wb.sheets.contains_key("Orders"));
    assert!(wb.sheets.contains_key("Products"));

    let customers = wb.sheets.get("Customers").unwrap();
    assert_eq!(customers.columns, vec!["CustomerID", "Name", "Email", "Region", "Tier"]);
    assert_eq!(customers.rows.len(), 4);
}

#[test]
fn test_query_excel_sheets_and_joins() {
    let catalog = ExcelCatalog::load_from_path("examples/sample_sales.xlsx").unwrap();
    let conn = catalog.create_in_memory_sqlite().unwrap();

    // 1. Single sheet query
    let (cols, rows) = query_to_table(&conn, "SELECT Name, Region FROM Customers WHERE Tier = 'Enterprise' ORDER BY Name ASC").unwrap();
    assert_eq!(cols, vec!["Name", "Region"]);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], vec!["Acme Corp", "North America"]);
    assert_eq!(rows[1], vec!["Initech", "North America"]);

    // 2. Cross-sheet join
    let (cols, rows) = query_to_table(
        &conn,
        "SELECT o.OrderID, c.Name, p.Title FROM Orders o JOIN Customers c ON o.CustomerID = c.CustomerID JOIN Products p ON o.ProductSKU = p.ProductSKU WHERE o.OrderID = 'ORD-501'"
    ).unwrap();
    assert_eq!(cols, vec!["OrderID", "Name", "Title"]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec!["ORD-501", "Acme Corp", "UltraWide 4K IPS Monitor"]);
}

#[test]
fn test_load_and_query_csv() {
    let catalog = ExcelCatalog::load_from_path("examples/employees.csv").expect("Failed to load CSV file");
    assert_eq!(catalog.workbooks.len(), 1);

    let conn = catalog.create_in_memory_sqlite().unwrap();

    // 1. Query by table name
    let (cols, rows) = query_to_table(&conn, "SELECT first_name, last_name, department FROM employees WHERE id = '101'").unwrap();
    assert_eq!(cols, vec!["first_name", "last_name", "department"]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec!["John", "Doe", "Engineering"]);

    // 2. Query by generic 'data' view alias
    let (cols, rows) = query_to_table(&conn, "SELECT COUNT(*) FROM data WHERE active = 'true'").unwrap();
    assert_eq!(cols, vec!["COUNT(*)"]);
    assert_eq!(rows[0][0], "4");
}

#[test]
fn test_export_excel_to_sqlite() {
    let out_db = "examples/test_sales_export.db";
    let _ = fs::remove_file(out_db);

    let catalog = ExcelCatalog::load_from_path("examples/sample_sales.xlsx").unwrap();
    catalog.export_to_sqlite(out_db).expect("Failed to export Excel to SQLite");

    let conn = Connection::open(out_db).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM Customers", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 4);

    let _ = fs::remove_file(out_db);
}
