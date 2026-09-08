use rtb::modules::xlsx::loader::{query_to_table, ExcelCatalog};
use rusqlite::Connection;
use std::fs;

#[test]
fn test_load_catalog_directory() {
    let catalog = ExcelCatalog::load_from_path("examples/sample_catalog/").expect("Failed to load catalog directory");
    assert!(catalog.is_directory);
    assert!(catalog.workbooks.len() >= 3); // inventory.xlsx, sales.xlsx, shipping_partners.csv
}

#[test]
fn test_cross_file_and_cross_format_joins() {
    let catalog = ExcelCatalog::load_from_path("examples/sample_catalog/").unwrap();
    let conn = catalog.create_in_memory_sqlite().unwrap();

    // Query joining 2 Excel files (sales.xlsx, inventory.xlsx) and 1 CSV file (shipping_partners.csv)
    let sql = "SELECT o.OrderID, c.Name as Customer, p.ProductName, w.Location as Warehouse, s.PartnerName as Carrier
FROM sales.Orders o
JOIN sales.Customers c ON o.CustomerID = c.CustomerID
JOIN inventory.Products p ON o.ProductSKU = p.ProductSKU
JOIN inventory.Warehouses w ON p.WarehouseCode = w.WarehouseCode
JOIN shipping_partners s ON s.PartnerCode = 'SHIP-FEDEX'
WHERE o.OrderID = 'ORD-101'";

    let (cols, rows) = query_to_table(&conn, sql).expect("Cross-file JOIN query failed");
    assert_eq!(cols, vec!["OrderID", "Customer", "ProductName", "Warehouse", "Carrier"]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0][0], "ORD-101");
    assert_eq!(rows[0][1], "Acme Industries");
    assert_eq!(rows[0][2], "UltraWide 4K IPS Monitor");
    assert_eq!(rows[0][3], "New York, USA");
    assert_eq!(rows[0][4], "FedEx Express");
}

#[test]
fn test_export_catalog_to_sqlite() {
    let out_db = "examples/test_catalog_export.db";
    let _ = fs::remove_file(out_db);

    let catalog = ExcelCatalog::load_from_path("examples/sample_catalog/").unwrap();
    catalog.export_to_sqlite(out_db).expect("Failed to export catalog to SQLite");

    let conn = Connection::open(out_db).unwrap();
    
    // Check tables from separate workbooks were created
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name ASC").unwrap();
    let table_names: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect();

    assert!(table_names.contains(&"inventory_Products".to_string()) || table_names.contains(&"inventory.Products".to_string()) || table_names.contains(&"shipping_partners".to_string()));

    let _ = fs::remove_file(out_db);
}
