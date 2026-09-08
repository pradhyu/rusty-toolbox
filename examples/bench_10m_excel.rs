use rtb::modules::xlsx::loader::{query_to_table, ExcelCatalog};
use rust_xlsxwriter::*;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("══════════════════════════════════════════════════════════════════════════");
    println!(" 📊 RUSTY TOOLBOX (`rtb`) — 10-MILLION ROW EXCEL BENCHMARK");
    println!("══════════════════════════════════════════════════════════════════════════\n");

    let excel_path = "examples/benchmark_10m.xlsx";

    // 1. Generate 10 Million row Excel file (10 sheets x 1,000,000 rows each)
    println!("📦 [Phase 1] Generating 10-Million Row Excel Workbook (10 sheets x 1,000,000 rows)...");
    let gen_start = Instant::now();

    let mut workbook = Workbook::new(excel_path);
    let format = Format::new();

    let columns = ["ID", "TXN_CODE", "USER_ID", "SKU", "QTY", "PRICE", "TOTAL", "STATUS", "REGION"];
    let statuses = ["COMPLETED", "PROCESSING", "SHIPPED", "CANCELLED", "REFUNDED"];
    let regions = ["North America", "Europe", "Asia-Pacific", "Latin America", "Middle East"];
    let skus = ["SKU-MON-4K", "SKU-KB-MEC", "SKU-DOCK-TB", "SKU-MOUSE-WL", "SKU-CABLE-HD"];

    for sheet_idx in 1..=10 {
        let sheet_start = Instant::now();
        let sheet_name = format!("Batch_{:02}", sheet_idx);
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&sheet_name)?;

        // Header
        for (c, col_name) in columns.iter().enumerate() {
            worksheet.write_string(0, c as u16, *col_name, &format)?;
        }

        // 1,000,000 rows per sheet
        for r in 1..=1_000_000 {
            let id = (sheet_idx - 1) * 1_000_000 + r;
            let status = statuses[r as usize % 5];
            let region = regions[r as usize % 5];
            let sku = skus[r as usize % 5];
            let qty = (r % 10) + 1;
            let price = ((r % 500) + 50) as f64 + 0.99;
            let total = (qty as f64) * price;

            worksheet.write_number(r as u32, 0, id as f64, &format)?;
            worksheet.write_string(r as u32, 1, &format!("TXN-{:08}", id), &format)?;
            worksheet.write_string(r as u32, 2, &format!("CUST-{:06}", (id % 50000) + 1), &format)?;
            worksheet.write_string(r as u32, 3, sku, &format)?;
            worksheet.write_number(r as u32, 4, qty as f64, &format)?;
            worksheet.write_number(r as u32, 5, price, &format)?;
            worksheet.write_number(r as u32, 6, total, &format)?;
            worksheet.write_string(r as u32, 7, status, &format)?;
            worksheet.write_string(r as u32, 8, region, &format)?;
        }

        println!("  ✔ Sheet {} (1,000,000 rows) generated in {:.2?}", sheet_name, sheet_start.elapsed());
    }

    println!("  💾 Compressing and writing '{}' to disk...", excel_path);
    let save_start = Instant::now();
    workbook.close()?;
    println!("  ✔ Saved 10,000,000 rows in {:.2?} (Total generation time: {:.2?})\n", save_start.elapsed(), gen_start.elapsed());

    // 2. Benchmark Loading with rtb
    println!("⚡ [Phase 2] Benchmarking `rtb xlsx` Engine Loading & SQLite In-Memory Indexing...");
    let load_start = Instant::now();
    let catalog = ExcelCatalog::load_from_path(excel_path)?;
    println!("  ✔ Excel Parsing Time: {:.2?} (Read 10 sheets, 10M rows)", load_start.elapsed());

    let sql_bridge_start = Instant::now();
    let conn = catalog.create_in_memory_sqlite()?;
    println!("  ✔ In-Memory SQLite Indexing Time: {:.2?}", sql_bridge_start.elapsed());
    println!("  ⚡ Total Load + Index Throughput: {:.2?} (~{:.2} million rows/sec)\n",
        load_start.elapsed(),
        10.0 / load_start.elapsed().as_secs_f64()
    );

    // 3. Benchmark Queries
    println!("🔍 [Phase 3] Running SQL Query Benchmarks...");

    // Query 1: Single Sheet Filter
    let q1_start = Instant::now();
    let (_, rows1) = query_to_table(&conn, "SELECT TXN_CODE, USER_ID, TOTAL FROM Batch_01 WHERE STATUS = 'COMPLETED' AND TOTAL > 2000 LIMIT 10")?;
    println!("  Query 1 (Single 1M Sheet Filter): {:.2?} ({} rows returned)", q1_start.elapsed(), rows1.len());

    // Query 2: Cross-Sheet Multi-Million Aggregate
    let q2_start = Instant::now();
    let sql_multi = "
        SELECT REGION, COUNT(*) as tx_count, SUM(CAST(TOTAL AS REAL)) as total_rev
        FROM (
            SELECT * FROM Batch_01 UNION ALL
            SELECT * FROM Batch_02 UNION ALL
            SELECT * FROM Batch_03 UNION ALL
            SELECT * FROM Batch_04 UNION ALL
            SELECT * FROM Batch_05
        )
        GROUP BY REGION
    ";
    let (_, rows2) = query_to_table(&conn, sql_multi)?;
    println!("  Query 2 (5-Million Row Aggregation & Group By): {:.2?} ({} regions aggregated)", q2_start.elapsed(), rows2.len());

    println!("\n══════════════════════════════════════════════════════════════════════════");
    println!(" 🏆 BENCHMARK SUMMARY FOR 10-MILLION ROW EXCEL WORKBOOK");
    println!("══════════════════════════════════════════════════════════════════════════");
    println!(" • Dataset: 10,000,000 rows across 10 sheets (90,000,000 cells)");
    println!(" • Format: Native OpenXML Excel (.xlsx)");
    println!(" • Engine: rtb (Rust + Calamine + In-Memory SQLite)");
    println!("══════════════════════════════════════════════════════════════════════════\n");

    Ok(())
}
