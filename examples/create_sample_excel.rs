use rust_xlsxwriter::*;

fn main() -> Result<(), XlsxError> {
    let mut workbook = Workbook::new("examples/sample_sales.xlsx");
    let format = Format::new();

    // Sheet 1: Customers
    let sheet1 = workbook.add_worksheet();
    sheet1.set_name("Customers")?;
    let customers = [
        vec!["CustomerID", "Name", "Email", "Region", "Tier"],
        vec!["C101", "Acme Corp", "orders@acme.com", "North America", "Enterprise"],
        vec!["C102", "Globex Inc", "info@globex.org", "Europe", "Mid-Market"],
        vec!["C103", "Soylent Tech", "sales@soylent.io", "Asia-Pacific", "Starter"],
        vec!["C104", "Initech", "peter@initech.com", "North America", "Enterprise"],
    ];
    for (r, row) in customers.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            sheet1.write_string(r as u32, c as u16, *val, &format)?;
        }
    }

    // Sheet 2: Orders
    let sheet2 = workbook.add_worksheet();
    sheet2.set_name("Orders")?;
    let orders = [
        vec!["OrderID", "CustomerID", "ProductSKU", "Quantity", "TotalAmount", "Status"],
        vec!["ORD-501", "C101", "SKU-MON-4K", "3", "2097.00", "DELIVERED"],
        vec!["ORD-502", "C101", "SKU-KB-MEC", "10", "1499.90", "DELIVERED"],
        vec!["ORD-503", "C102", "SKU-DOCK-TB", "5", "947.50", "PROCESSING"],
        vec!["ORD-504", "C103", "SKU-MON-4K", "1", "699.00", "SHIPPED"],
        vec!["ORD-505", "C104", "SKU-KB-MEC", "2", "299.98", "CANCELLED"],
    ];
    for (r, row) in orders.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            sheet2.write_string(r as u32, c as u16, *val, &format)?;
        }
    }

    // Sheet 3: Products
    let sheet3 = workbook.add_worksheet();
    sheet3.set_name("Products")?;
    let products = [
        vec!["ProductSKU", "Title", "UnitPrice", "Stock"],
        vec!["SKU-MON-4K", "UltraWide 4K IPS Monitor", "699.00", "18"],
        vec!["SKU-KB-MEC", "Ergonomic Mechanical Keyboard", "149.99", "64"],
        vec!["SKU-DOCK-TB", "USB-C Thunderbolt 4 Dock", "189.50", "42"],
    ];
    for (r, row) in products.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            sheet3.write_string(r as u32, c as u16, *val, &format)?;
        }
    }

    workbook.close()?;
    println!("Successfully generated examples/sample_sales.xlsx");
    Ok(())
}
