use rust_xlsxwriter::*;
use std::fs;

fn main() -> Result<(), XlsxError> {
    fs::create_dir_all("examples/sample_catalog").ok();
    let format = Format::new();

    // 1. sales.xlsx
    let mut sales_wb = Workbook::new("examples/sample_catalog/sales.xlsx");
    let orders_sheet = sales_wb.add_worksheet();
    orders_sheet.set_name("Orders")?;
    let orders = [
        vec!["OrderID", "CustomerID", "ProductSKU", "Quantity", "TotalAmount", "Status"],
        vec!["ORD-101", "CUST-A", "SKU-MONITOR", "2", "1398.00", "SHIPPED"],
        vec!["ORD-102", "CUST-B", "SKU-KEYBOARD", "5", "749.95", "DELIVERED"],
        vec!["ORD-103", "CUST-A", "SKU-DOCK", "1", "189.50", "PROCESSING"],
        vec!["ORD-104", "CUST-C", "SKU-MONITOR", "4", "2796.00", "DELIVERED"],
    ];
    for (r, row) in orders.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            orders_sheet.write_string(r as u32, c as u16, *val, &format)?;
        }
    }

    let cust_sheet = sales_wb.add_worksheet();
    cust_sheet.set_name("Customers")?;
    let customers = [
        vec!["CustomerID", "Name", "Country", "AccountTier"],
        vec!["CUST-A", "Acme Industries", "USA", "Platinum"],
        vec!["CUST-B", "Global Logistics", "Germany", "Gold"],
        vec!["CUST-C", "Apex Tech Labs", "Japan", "Platinum"],
    ];
    for (r, row) in customers.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            cust_sheet.write_string(r as u32, c as u16, *val, &format)?;
        }
    }
    sales_wb.close()?;

    // 2. inventory.xlsx
    let mut inv_wb = Workbook::new("examples/sample_catalog/inventory.xlsx");
    let prod_sheet = inv_wb.add_worksheet();
    prod_sheet.set_name("Products")?;
    let products = [
        vec!["ProductSKU", "ProductName", "Category", "UnitPrice", "WarehouseCode"],
        vec!["SKU-MONITOR", "UltraWide 4K IPS Monitor", "Displays", "699.00", "WH-EAST"],
        vec!["SKU-KEYBOARD", "Mechanical RGB Keyboard", "Peripherals", "149.99", "WH-WEST"],
        vec!["SKU-DOCK", "USB-C Thunderbolt Dock", "Accessories", "189.50", "WH-EAST"],
    ];
    for (r, row) in products.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            prod_sheet.write_string(r as u32, c as u16, *val, &format)?;
        }
    }

    let wh_sheet = inv_wb.add_worksheet();
    wh_sheet.set_name("Warehouses")?;
    let warehouses = [
        vec!["WarehouseCode", "Location", "CapacityUnits"],
        vec!["WH-EAST", "New York, USA", "50000"],
        vec!["WH-WEST", "San Francisco, USA", "35000"],
    ];
    for (r, row) in warehouses.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            wh_sheet.write_string(r as u32, c as u16, *val, &format)?;
        }
    }
    inv_wb.close()?;

    println!("Generated multi-workbook catalog in examples/sample_catalog/");
    Ok(())
}
