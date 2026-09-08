# rusty-toolbox (`rtb`)

A blazing fast, cross-platform developer Swiss Army knife written in Rust with zero JVM/heavy runtime dependencies.

Runs anywhere: **macOS**, **Linux**, and **Windows**.

---

## ⚡ Key Highlights

### 1. `rtb xlsx` / `rtb csv` — Multi-Workbook Catalogs, CSVs & Cross-File SQL Joins
Query, filter, and inspect `.xlsx`, `.xls`, `.ods`, `.xlsb`, `.csv`, and `.tsv` files directly in your terminal using standard SQL with zero Microsoft Office, Python, or JVM dependencies.

- **CSV & Excel Single Files or Directories**:
  - Single CSV: `rtb csv employees.csv "SELECT * FROM data WHERE active = 'true'"`
  - Single Excel: `rtb xlsx sales.xlsx "SELECT * FROM Orders"`
  - Folder Catalog: `rtb xlsx ./data_folder/`
- **Cross-File SQL Joins**: `JOIN` tables across `.xlsx`, `.xls`, and `.csv` files in a single SQL query.
- **Interactive TUI with Row Yanking & Multi-Highlight**:
  - `Space`: Toggle multi-selection on rows.
  - `y` / `Y`: Yank selected rows (or active row) directly into system clipboard in clean TSV format (ready to paste into Excel, Google Sheets, or Neovim).
  - `a`: Select/deselect all rows.
  - `c`: Clear selection.
  - `/`: Live interactive SQL query prompt.
- **SQLite Export**: Instant export of entire multi-workbook directories, spreadsheets, or CSVs to standalone `.sqlite` databases (`--dump-sqlite`).

#### Example Usage:

```bash
# 1. Single CSV Query
rtb csv examples/employees.csv "SELECT first_name, last_name, salary FROM employees WHERE active = 'true'"

# 2. Open Interactive TUI for a single file or entire folder
rtb xlsx ./financial_data_folder/
rtb csv ./employees.csv

# 3. Cross-File SQL JOIN across separate Excel and CSV files in a folder
rtb xlsx ./examples/sample_catalog/ "SELECT o.OrderID, c.Name as Customer, p.ProductName, s.PartnerName as Carrier
FROM sales.Orders o
JOIN sales.Customers c ON o.CustomerID = c.CustomerID
JOIN inventory.Products p ON o.ProductSKU = p.ProductSKU
JOIN shipping_partners s ON s.PartnerCode = 'SHIP-FEDEX'"

# 4. Export Multi-Workbook Folder to SQLite
rtb xlsx ./financial_data_folder/ --dump-sqlite company_data.db
```

---

### 2. `rtb hsql` — Native HyperSQL (HSQLDB) Client
Inspect and query HyperSQL (HSQLDB) database catalogs, files, and schemas without installing Java or JDBC drivers.

- **Zero JVM Dependency**: Pure Rust parser for HSQLDB `.properties` and `.script` (DDL & memory rows).
- **Flexible Path Input**: Pass a directory containing database files or a direct path to a `.script`/`.properties` file.
- **Interactive TUI**: Table browser with live SQL query runner (`/`), multi-row highlight (`Space`), and clipboard yank (`y`).

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

## ⌨️ TUI Keybindings (for `rtb xlsx`, `rtb csv`, and `rtb hsql`)
| Key | Action |
|---|---|
| `Tab` / `BackTab` | Switch focus between **Sidebar** and **Data Table** |
| `↑` / `↓` (`k` / `j`) | Navigate rows / schemas / sheets / tables |
| `PgUp` / `PgDn` | Fast scroll (15 rows) |
| `Space` | **Toggle Multi-Row Selection / Highlight** |
| `y` / `Y` | **Yank (Copy) Selected Row(s) to System Clipboard (TSV format)** |
| `a` | Select / Deselect All Rows |
| `c` | Clear Row Selection |
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
1. `rtb xlsx` / `rtb csv` — Multi-workbook Excel & CSV catalog with cross-file SQL and TUI clipboard yank
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
