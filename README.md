# rusty-toolbox (`rtb`)

A blazing fast, cross-platform developer Swiss Army knife written in Rust with zero JVM/heavy runtime dependencies.

Runs anywhere: **macOS**, **Linux**, and **Windows**.

---

## ⚡ Key Highlights

### 1. `rtb xlsx` / `rtb excel` — Native Excel & Spreadsheet SQL Inspector
Query, filter, and inspect `.xlsx`, `.xls`, `.ods`, and `.csv` spreadsheets directly in your terminal using standard SQL with zero Microsoft Office, Python, or JVM dependencies.

- **Cross-Sheet SQL Joins**: Automatically maps each worksheet into a relational table.
- **Dual Mode**:
  - **Scriptable CLI / ASCII Tables**: Run SQL queries with aggregate functions (`SUM`, `COUNT`, `AVG`), `GROUP BY`, and `JOIN`s.
  - **Interactive TUI**: Dual-pane Ratatui browser with a sheet selector sidebar, scrollable data grid, and interactive SQL query prompt (`/`).
- **SQLite Export**: Instant export of entire multi-sheet workbooks to `.sqlite` databases (`--dump-sqlite`).

#### Example Usage:
```bash
# 1. Interactive TUI Browser
rtb xlsx ./sales_report.xlsx
# or alias
rtb excel ./sales_report.xlsx

# 2. Run Direct SQL Queries
rtb xlsx sales.xlsx "SELECT * FROM Customers WHERE Region = 'North America'"

# Multi-sheet JOIN
rtb xlsx sales.xlsx "SELECT o.OrderID, c.Name, o.TotalAmount FROM Orders o JOIN Customers c ON o.CustomerID = c.CustomerID"

# Aggregations & Grouping
rtb xlsx sales.xlsx "SELECT Region, COUNT(o.OrderID) as Orders, SUM(CAST(o.TotalAmount AS REAL)) as Revenue FROM Orders o JOIN Customers c ON o.CustomerID = c.CustomerID GROUP BY Region"

# 3. Export to SQLite
rtb xlsx sales.xlsx --dump-sqlite sales.db
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
| `↑` / `↓` (`k` / `j`) | Navigate rows / sheets / tables |
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
```

Add an alias to your `~/.zshrc` or `~/.bashrc`:
```bash
alias rtb="$HOME/git/rusty-toolbox/target/release/rtb"
alias tb="$HOME/git/rusty-toolbox/target/release/rtb"
```

---

## 📋 Full Planned Toolset Overview (`rtb`)

See [`SPEC.md`](SPEC.md) for the complete specification of all tools:
1. `rtb xlsx` — Native Excel/Spreadsheet SQL inspector & SQLite converter
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
