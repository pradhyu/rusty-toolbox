use rust_xlsxwriter::*;
use std::time::Instant;

fn main() -> Result<(), XlsxError> {
    let path = "examples/sample_large_100k.xlsx";
    println!("Generating 100,000 row Excel file at '{}'...", path);
    let start = Instant::now();

    let mut workbook = Workbook::new(path);
    let format = Format::new();

    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Transactions")?;

    let columns = ["ID", "TXN_CODE", "USER_ID", "SKU", "QTY", "PRICE", "TOTAL", "STATUS", "REGION"];
    for (c, col_name) in columns.iter().enumerate() {
        worksheet.write_string(0, c as u16, *col_name, &format)?;
    }

    let statuses = ["COMPLETED", "PROCESSING", "SHIPPED", "CANCELLED", "REFUNDED"];
    let regions = ["North America", "Europe", "Asia-Pacific", "Latin America", "Middle East"];
    let skus = ["SKU-MON-4K", "SKU-KB-MEC", "SKU-DOCK-TB", "SKU-MOUSE-WL", "SKU-CABLE-HD"];

    for r in 1..=100_000 {
        let status = statuses[r as usize % 5];
        let region = regions[r as usize % 5];
        let sku = skus[r as usize % 5];
        let qty = (r % 10) + 1;
        let price = ((r % 500) + 50) as f64 + 0.99;
        let total = (qty as f64) * price;

        worksheet.write_number(r as u32, 0, r as f64, &format)?;
        worksheet.write_string(r as u32, 1, &format!("TXN-{:08}", r), &format)?;
        worksheet.write_string(r as u32, 2, &format!("CUST-{:06}", (r % 5000) + 1), &format)?;
        worksheet.write_string(r as u32, 3, sku, &format)?;
        worksheet.write_number(r as u32, 4, qty as f64, &format)?;
        worksheet.write_number(r as u32, 5, price, &format)?;
        worksheet.write_number(r as u32, 6, total, &format)?;
        worksheet.write_string(r as u32, 7, status, &format)?;
        worksheet.write_string(r as u32, 8, region, &format)?;
    }

    workbook.close()?;
    println!("✔ Generated {} in {:.2?}", path, start.elapsed());
    Ok(())
}
