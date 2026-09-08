# rusty-toolbox (`rtb`)

A blazing fast, cross-platform developer Swiss Army knife written in Rust with zero JVM/heavy runtime dependencies.

Runs anywhere: **macOS**, **Linux**, and **Windows**.

---

## ⚡ Highlights: `rrtb hsql` (Native HyperSQL / HSQLDB Client)

Inspect and query HyperSQL (HSQLDB) database catalogs, files, and schemas without installing Java or JDBC drivers.

### Features
- **Zero JVM Dependency**: Native Rust parser for HSQLDB `.properties` and `.script` (DDL & memory rows).
- **Dual Mode**:
  - **Terminal / CI Query Mode**: Run SQL directly on a database file or directory, formatted as clean ASCII tables.
  - **Interactive TUI Mode**: Ratatui-powered dual-pane explorer with a table sidebar, scrollable data grid, and interactive SQL query prompt (`/`).
- **SQLite Bridge & Conversion**: Fast export from HSQLDB to SQLite (`--dump-sqlite`).
- **Flexible Path Input**: Pass a directory containing database files (`mydb.properties`, `mydb.script`) or direct path to a `.script`/`.properties` file.

### Usage

```bash
# 1. Open interactive TUI explorer
rrtb hsql ./data/sample_hsqldb/
# or
rrtb hsql ./data/sample_hsqldb/mydb.script -i

# 2. Run direct SQL queries in your shell (scriptable)
rrtb hsql ./data/sample_hsqldb/ "SELECT * FROM USERS WHERE ACTIVE = 1"

rrtb hsql ./data/sample_hsqldb/ "SELECT u.USERNAME, o.ORDER_ID, o.AMOUNT FROM USERS u JOIN ORDERS o ON u.ID = o.USER_ID"

# 3. Export HSQLDB database to a standalone SQLite database file
rrtb hsql ./data/sample_hsqldb/ --dump-sqlite exported_hsql.db
```

### TUI Keybindings
| Key | Action |
|---|---|
| `Tab` / `BackTab` | Switch focus between **Tables Sidebar** and **Data Table** |
| `↑` / `↓` (`k` / `j`) | Navigate rows / tables |
| `/` | Open custom interactive SQL query prompt |
| `Enter` | Execute SQL query |
| `Esc` | Cancel SQL query prompt / Restore table view |
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
alias tb="/path/to/rusty-toolbox/target/release/rusty-toolbox"
```

---

## 📋 Full Toolset Overview (`rtb`)

See [`SPEC.md`](SPEC.md) for the complete specification of all 17 planned developer tools:
1. `rrtb hsql` — HyperSQL native inspector & SQLite converter
2. `rtb jwt` — Offline JWT decoder & visualizer
3. `rtb time` — Epoch & multi-timezone converter
4. `rtb json` — High-speed JSON schema & path extractor
5. `rtb cert` — X.509 TLS/SSL certificate inspector
6. `rtb port` — Port listener & process identifier
7. `rtb b64` / `rtb url` / `rtb hash` — Fast encoders, decoders, and checksum tools
8. `rtb ip` / `rtb cidr` — IP subnet calculations and local interface scanner
9. `rtb diff` — Semantic AST & JSON diff tool
10. `rtb cron` — Cron expression parser and next execution schedule predictor

---

## 📜 License
MIT
