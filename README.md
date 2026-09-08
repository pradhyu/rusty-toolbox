# rusty-toolbox (`rtb`)

A blazing fast, cross-platform developer Swiss Army knife written in Rust with zero JVM/heavy runtime dependencies.

Runs anywhere: **macOS**, **Linux**, and **Windows**.

---

## ⚡ Key Highlights

### 1. `rtb xlsx` / `rtb excel` — Multi-Workbook Folder Catalogs & Cross-File SQL Joins
Query, filter, and inspect `.xlsx`, `.xls`, `.ods`, `.xlsb`, and `.csv` spreadsheets directly in your terminal using standard SQL with zero Microsoft Office, Python, or JVM dependencies.

- **Folder / Multi-File Catalog**: Pass an entire directory containing multiple Excel spreadsheets. Each spreadsheet file becomes a **Schema namespace** (e.g. `sales.xlsx` -> `sales.Orders`, `inventory.xlsx` -> `inventory.Products`).
- **Cross-File SQL Joins**: Seamlessly `JOIN` worksheets across different Excel files in one query.
- **Dual Mode**:
  - **Scriptable CLI / ASCII Tables**: Run SQL queries with aggregate functions (`SUM`, `COUNT`, `AVG`), `GROUP BY`, and `JOIN`s.
  - **Interactive TUI**: Dual-pane Ratatui browser with a schema & sheet tree sidebar, scrollable data grid, and interactive SQL query prompt (`/`).
- **SQLite Export**: Instant export of entire multi-workbook directories or individual workbooks to `.sqlite` databases (`--dump-sqlite`).

#### Example Usage:

```bash
# 1. Open Interactive TUI for a single file or entire folder
rtb xlsx ./sales_report.xlsx
rtb xlsx ./financial_data_folder/

# 2. Cross-File SQL JOIN across separate Excel files
rtb xlsx ./examples/sample_catalog/ "SELECT o.OrderID, c.Name as Customer, p.ProductName, w.Location as Warehouse, o.TotalAmount
FROM sales.Orders o
JOIN sales.Customers c ON o.CustomerID = c.CustomerID
JOIN inventory.Products p ON o.ProductSKU = p.ProductSKU
JOIN inventory.Warehouses w ON p.WarehouseCode = w.WarehouseCode"

# 3. Aggregations & Grouping
rtb xlsx ./examples/sample_sales.xlsx "SELECT Region, COUNT(o.OrderID) as Orders, SUM(CAST(o.TotalAmount AS REAL)) as Revenue \
  FROM Orders o JOIN Customers c ON o.CustomerID = c.CustomerID GROUP BY Region"

# 4. Export Multi-Workbook Folder to SQLite
rtb xlsx ./financial_data_folder/ --dump-sqlite company_data.db
```

---

### 2. `rtb hsql` — Native HyperSQL (HSQLDB) Client
Inspect and query HyperSQL (HSQLDB) database catalogs, files, and schemas without installing Java or JDBC drivers.

- **Zero JVM Dependency**: Pure Rust parser for HSQLDB `.properties` and `.script` (DDL & memory rows).
- **Flexible Path Input**: Pass a directory containing database files or a direct path to a `.script`/`.properties` file.
- **Interactive TUI**: Table browser with live SQL query runner (`/`).

#### Example Usage:
```bash
# 1. Open interactive TUI explorer
rtb hsql ./data/sample_hsqldb/

# 2. Run direct SQL queries in your shell (scriptable)
rtb hsql ./data/sample_hsqldb/ "SELECT * FROM USERS WHERE ACTIVE = 1"
rtb hsql ./data/sample_hsqldb/ "SELECT u.USERNAME, o.ORDER_ID, o.AMOUNT FROM USERS u JOIN ORDERS o ON u.ID = o.USER_ID"

# 3. Export HSQLDB database to a standalone SQLite database file
rtb hsql ./data/sample_hsqldb/ --dump-sqlite exported_hsql.db
```

---

## ⌨️ TUI Keybindings (for both `rtb xlsx` & `rtb hsql`)
| Key | Action |
|---|---|
| `Tab` / `BackTab` | Switch focus between **Sidebar** and **Data Table** |
| `↑` / `↓` (`k` / `j`) | Navigate rows / schemas / sheets / tables |
| `PgUp` / `PgDn` | Fast scroll (15 rows) |
| `/` | Open custom interactive SQL query prompt |
| `Enter` | Execute SQL query |
| `Esc` | Cancel SQL query prompt / Restore full table view |
| `q` | Quit |

---

## 🛠️ Installation & Building

```bash
git clone https://github.com/pradhyu/rusty-toolbox.git
cd rusty-toolbox
cargo build --release
cargo install --path .
```

---

## 📋 Full Planned Toolset Overview (`rtb`)

See [`SPEC.md`](SPEC.md) for the complete specification of all tools:
1. `rtb xlsx` — Multi-workbook Excel catalog & cross-file SQL query engine
2. `rtb hsql` — HyperSQL native inspector & SQLite converter
3. `rtb jwt` — Offline JWT decoder & visualizer
4. `rtb time` — Epoch & multi-timezone converter
5. `rtb json` — High-speed JSON schema & path extractor
6. `rtb cert` — X.509 TLS/SSL certificate inspector
7. `rtb port` — Port listener & process identifier
8. `rtb b64` / `rtb url` / `rtb hash` — Fast encoders, decoders, and checksum tools
9. `rtb ip` / `rtb cidr` — IP subnet calculations and local interface scanner
10. `rtb diff` — Semantic AST & JSON diff tool
11. `rtb cron` — Cron expression parser and next execution schedule predictor

---

## 📜 License
MIT
