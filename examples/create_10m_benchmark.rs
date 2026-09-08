use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let path = "examples/benchmark_10m.csv";
    println!("🚀 Generating 10,000,000 row benchmark dataset at '{}'...", path);
    let start = Instant::now();

    let file = File::create(path)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024 * 8, file); // 8MB write buffer

    // 10 columns per row (10M rows = 100M cells)
    writeln!(writer, "id,transaction_id,customer_id,product_sku,quantity,unit_price,total_amount,status,region,created_at")?;

    let statuses = ["COMPLETED", "PROCESSING", "SHIPPED", "CANCELLED", "REFUNDED"];
    let regions = ["North America", "Europe", "Asia-Pacific", "Latin America", "Middle East"];
    let skus = ["SKU-MON-4K", "SKU-KB-MEC", "SKU-DOCK-TB", "SKU-MOUSE-WL", "SKU-CABLE-HD"];

    for i in 1..=10_000_000 {
        let status = statuses[i % 5];
        let region = regions[i % 5];
        let sku = skus[i % 5];
        let qty = (i % 10) + 1;
        let price = ((i % 500) + 50) as f64 + 0.99;
        let total = (qty as f64) * price;

        writeln!(
            writer,
            "{},TXN-{:08},CUST-{:06},{},{},{:.2},{:.2},{},{},2026-09-01T{:02}:00:00Z",
            i,
            i,
            (i % 100000) + 1,
            sku,
            qty,
            price,
            total,
            status,
            region,
            (i % 24)
        )?;

        if i % 2_000_000 == 0 {
            println!("  ⏳ Generated {} / 10,000,000 rows ({:.1?})...", i, start.elapsed());
        }
    }

    writer.flush()?;
    println!("✔ Finished generating 10,000,000 rows in {:.2?}! File: {}", start.elapsed(), path);
    Ok(())
}
