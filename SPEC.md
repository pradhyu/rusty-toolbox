# 🧰 Rusty Toolbox (`tb`) — System Architecture & Specification

A cross-platform (macOS, Linux, Windows), zero-dependency, ultra-fast CLI & TUI developer productivity toolkit written in Rust.

---

## 1. Project Objectives & Design Goals

1. **100% Cross-Platform Parity**:
   - Compiles to native single binaries on **macOS** (Apple Silicon + Intel), **Linux** (x86_64, aarch64, musl), and **Windows** (`.exe`).
   - Zero runtime dependencies: No Node.js, Python, or shell wrappers required.
2. **Instant Startup (< 10ms)**:
   - Optimized startup time with lazy initialization per subcommand.
3. **Dual Mode Interface**:
   - **Scriptable CLI / Stdout mode**: Pipe-friendly (`tb port 3000 -k`, `tb conv jwt <token>`).
   - **Interactive TUI mode**: Rich keyboard-driven visual navigation (`ratatui` + `crossterm`).

---

## 2. Architecture & Module Structure

```
rusty-toolbox/
├── Cargo.toml                # Workspace & dependencies configuration
├── SPEC.md                   # Technical specification & architecture design
├── src/
│   ├── main.rs               # Root CLI dispatcher (`clap`)
│   ├── modules/
│   │   ├── port/             # Module 1: Port & Process Inspector / Killer (`portx`)
│   │   │   ├── mod.rs
│   │   │   ├── scanner.rs    # Cross-platform socket/process scanner (`sysinfo` / `netstat2`)
│   │   │   ├── killer.rs     # Safe process termination (SIGTERM -> SIGKILL / TerminateProcess)
│   │   │   └── tui.rs        # Interactive listening ports table & process manager
│   │   ├── clean/            # Module 2: Dev Artifact & Disk Reclaimer (`disk-vac`)
│   │   │   ├── mod.rs
│   │   │   ├── scanner.rs    # Multithreaded folder scanner (`rayon` + `walkdir`)
│   │   │   ├── cleaner.rs    # Safe artifact removal (`target`, `node_modules`, `.venv`)
│   │   │   └── tui.rs        # Interactive disk space visualizer & cleaner
│   │   ├── conv/             # Module 3: Universal Dev Converter (`epochx`)
│   │   │   ├── mod.rs
│   │   │   ├── epoch.rs      # Timestamps (Unix, UTC, Local, ISO8601, Relative)
│   │   │   ├── jwt.rs        # JWT Decoder & expiry validator
│   │   │   └── hash.rs       # SHA256, SHA512, MD5, Base64, UUID inspector
│   │   └── git_sweep/        # Module 4: Git Branch Sweeper (`git-sweep`)
│   │       ├── mod.rs
│   │       ├── git.rs        # Branch status & upstream tracking (`git2`)
│   │       └── tui.rs        # Multi-select branch deletion UI
│   └── ui/                   # Shared TUI widgets & theme styling
│       ├── theme.rs          # Nord, Catppuccin, ANSI truecolor palettes
│       └── widgets.rs        # Reusable modal, prompt, and table components
```

---

## 3. Subcommand Specifications

### 3.1 `tb port` — Port & Process Inspector / Killer

#### Capabilities
- **Direct Query**: `tb port 3000`
  - Output: Process Name, PID, Listening Port, Protocol (TCP/UDP), Memory usage, Working Directory, Command line invocation.
- **Batch Kill**: `tb port -k 3000,8080,5432`
  - Sends graceful termination (`SIGTERM` on Unix / `WM_CLOSE` on Windows), falling back to force kill (`SIGKILL` / `TerminateProcess`).
- **Interactive TUI**: `tb port -i` (or `tb port` without arguments)
  - Live table of all listening ports.
  - Filter by port number or process name (`/`).
  - Press `k` or `x` to kill selected process, `r` to refresh.

#### Cross-Platform Strategy
- **macOS & Linux**: Native socket scanning via `/proc/net` and `sysinfo` process tables.
- **Windows**: `netstat2` / Win32 `GetExtendedTcpTable` via `windows-sys`.

---

### 3.2 `tb clean` — Dev Artifact & Disk Space Reclaimer

#### Capabilities
- **Fast Multithreaded Scan**: Scans designated directory (default `~/git` or current folder).
- **Target Patterns**:
  - Rust: `target/`
  - Node.js: `node_modules/`, `.next/`, `.nuxt/`
  - Python: `.venv/`, `venv/`, `__pycache__/`, `.pytest_cache/`
  - Terraform: `.terraform/`
  - Java/Gradle: `build/`, `.gradle/`
- **Interactive TUI**:
  - Displays total space reclaimable (e.g. `48.2 GB reclaimable across 24 projects`).
  - Checkbox multi-select (`Space`) to select projects to clean.
  - `Enter` to vacuum selected artifacts.

---

### 3.3 `tb conv` — Universal Developer Converter

#### Capabilities
- **Epoch / Timestamps**:
  - `tb conv ts 1725764000` -> ISO 8601, UTC, Local Time, Humanized relative ("3 hours ago").
  - `tb conv ts now` -> Current timestamp in seconds & milliseconds.
- **JWT Inspector**:
  - `tb conv jwt <token>` -> Pretty prints decoded header & claims, color-codes token expiration (Active in Green vs Expired in Red).
- **Hashes & Encoding**:
  - `tb conv hash "secret"` -> Outputs MD5, SHA256, SHA512, Base64 encode/decode, Hex format.

---

### 3.4 `tb git` — Interactive Git Branch Sweeper

#### Capabilities
- **Branch Analysis**: Identifies local branches merged into `main`/`master` or deleted on origin remote.
- **Interactive Sweeper**:
  - Lists branch names, author, last commit date, and commit message.
  - Batch delete stale branches safely with confirmation modal.

---

### 3.5 `tb env` — Interactive `.env` Diff, Validator & Secret Vault
- **Problem**: Missing environment variables between `.env.example`, `.env.local`, and staging/production configs cause runtime crashes.
- **Capabilities**:
  - `tb env diff .env .env.example` -> Visual side-by-side comparison highlighting missing keys, extra keys, and type mismatches.
  - `tb env check` -> Validates all required variables exist without leaking secret values on screen (masks values by default).
  - `tb env gen` -> Auto-generates template `.env.example` from existing codebase references (`process.env.*`, `os.environ[...]`, `std::env::var`).

---

### 3.6 `tb bench` / `tb time` — Microbenchmark & Memory Peak Profiler
- **Problem**: Basic `time` command only shows total wall time and lacks memory footprint (RSS peak), warm-up iterations, and statistical outliers.
- **Capabilities**:
  - `tb bench "cargo check" "cargo build"` -> Side-by-side execution comparison with statistical variance (p50, p95, p99), mean duration, and peak memory consumption (RAM RSS).
  - Generates sparkline graphs and colorized terminal histograms.
  - Exports benchmarks directly to Markdown tables for PR descriptions.

---

### 3.7 `tb json` — High-Speed JSON / YAML / TOML Query & Converter
- **Problem**: Switching between `jq`, `yq`, `dasel`, and online converters for config format translation and schema validation.
- **Capabilities**:
  - `tb json convert config.yaml -t toml` -> Instant conversion between JSON, YAML, TOML, and CSV.
  - `tb json query data.json ".users[].email"` -> Fast SIMD-accelerated key extraction.
  - `tb json flatten` / `tb json unflatten` -> Converts deeply nested structures into flat dot-notation keys for easy grepping.

---

### 3.8 `tb cert` — SSL/TLS Certificate & DNS Diagnostic Inspector
- **Problem**: Debugging expired SSL certs, TLS cipher handshake failures, and DNS propagation across global resolvers (`8.8.8.8`, `1.1.1.1`).
- **Capabilities**:
  - `tb cert example.com` -> Checks SANs, Issuer, Expiry Countdown (e.g. `Expires in 14 days ⚠️`), TLS protocol versions, and cipher suite support.
  - `tb cert dns example.com` -> Multi-resolver concurrent query comparing A, AAAA, CNAME, MX, and TXT records across major DNS providers simultaneously.

---

### 3.9 `tb share` — Instant Localhost Tunnel & P2P File Drop
- **Problem**: Exposing a local dev server (port 3000) for mobile testing or sending a quick 500MB database dump to a colleague without uploading to third-party cloud buckets.
- **Capabilities**:
  - `tb share http 3000` -> Quick secure tunnel or local QR code generator for instant mobile browser testing on the same Wi-Fi.
  - `tb share send dump.sql.gz` -> Ephemeral, end-to-end encrypted direct peer-to-peer transfer via WebRTC / TCP hole punching with a 6-digit one-time code.

---

### 3.10 `tb docker` / `tb ctl` — Lightweight Container & Pod Quick-Cleaner
- **Problem**: `docker system prune` is all-or-nothing, while Docker Desktop is heavy and slow.
- **Capabilities**:
  - `tb docker` -> TUI dashboard showing running containers, dangling images, untagged volumes, and build cache with real-time CPU/RAM meters.
  - Quick actions: `s` to stop, `r` to restart, `l` to tail logs, `x` to force purge unused layers.

---

### 3.11 `tb mock` — Zero-Config Instant Mock API & Webhook Reflector
- **Problem**: Waiting for backend endpoints to be built, or needing an immediate local webhook receiver (Stripe, GitHub, Slack) without setting up an entire mock server project.
- **Capabilities**:
  - `tb mock 8080 --data '{"status": "ok", "items": [1, 2, 3]}'` -> Spawns an instant local HTTP API server with simulated latency (`--delay 200ms`) and dynamic status codes (`--status 201`).
  - `tb mock webhook 8080` -> Prints full incoming HTTP requests (headers, payload, query params, cookies) formatted and syntax-highlighted in real time.
  - Generates realistic mock schemas on the fly (UUIDs, names, emails, timestamps).

---

### 3.12 `tb regex` — Interactive Terminal Regex Tester & Explainer
- **Problem**: Jumping out of the terminal into regex101.com to debug complex regular expressions with sample text.
- **Capabilities**:
  - `tb regex` -> Interactive split-pane TUI: Top pane for regular expression pattern with flag toggles (`i`, `m`, `s`), bottom pane for test string.
  - Live real-time capture group highlighting with distinct colors per match group.
  - Explains regex AST breakdown in plain English.

---

### 3.13 `tb watch` — Smart Incremental Command Runner (Modern `entr`/`nodemon`)
- **Problem**: `entr` or `nodemon` often trigger redundant re-runs on temporary files (`.git`, `target/`, `.swp`, logs), leading to CPU spikes and broken compilation loops.
- **Capabilities**:
  - `tb watch "cargo test"` or `tb watch --ext rs,toml "cargo run"` -> Debounced, smart gitignore-aware incremental file watcher.
  - Auto-clears terminal screen (`--clear`), tracks execution duration per cycle, and beeps or sends desktop notifications on build failure.

---

### 3.14 `tb secret` — Git Pre-Commit & Codebase Secret Leak Scanner
- **Problem**: Accidentally committing AWS keys, GitHub PATs, OpenAI API keys, or private SSH keys into public/private git repositories.
- **Capabilities**:
  - `tb secret scan` -> High-speed multithreaded regex & entropy scan across uncommitted staged diffs or full git history.
  - Built-in detection for 50+ secret signatures (AWS, GCP, Stripe, GitHub, Slack, RSA/Ed25519 private keys, JWT secrets).
  - Can be installed as a 0ms overhead Git pre-commit hook (`tb secret hook install`).

---

### 3.15 `tb sql` / `tb db` — Instant SQLite & Database Scratchpad
- **Problem**: Needing to inspect a `.sqlite` database, CSV file, or JSON array with SQL queries without opening DBeaver/DataGrip.
- **Capabilities**:
  - `tb sql data.sqlite` or `tb sql users.csv` or `tb sql log.json` -> Opens an interactive SQL REPL or runs one-liner queries (`tb sql users.csv "SELECT department, COUNT(*) FROM users GROUP BY department"`).
  - Clean formatted ASCII tables with export to JSON, CSV, or Markdown.

---

### 3.16 `tb log` — Streaming JSON Log Formatter & Live Filter
- **Problem**: Production structured JSON logs (Zap, Pino, Winston, Serilog, tracing-subscriber) scrolling as unreadable single-line blobs in `kubectl logs` or local Docker stdout.
- **Capabilities**:
  - `kubectl logs -f pod | tb log` or `cat app.log | tb log` -> Formats single-line JSON logs into human-readable colored log lines with timestamps, level badges (`INFO`, `WARN`, `ERROR`), and formatted stack traces.
  - Live filter by log level (`tb log --level error`) or key/value matching (`tb log --where "service=auth AND status>=400"`).

---

### 3.17 `tb hsql` — HyperSQL (HSQLDB) Inspector & Query Runner
- **Problem**: Opening HSQLDB files (`.script`, `.properties`, `.data`, `.log`) or folders containing standalone HSQLDB instances usually requires launching heavyweight Java GUI tools (`DatabaseManagerSwing` / `SqlTool`) or complex JDBC connection strings.
- **Capabilities**:
  - `tb hsql /path/to/db/` or `tb hsql mydb.script`:
    - **Zero-Java Schema & Script Inspector (Native Pure Rust)**:
      - Instantly parses `.script` & `.properties` metadata (table DDL, columns, memory records, version, isolation level).
      - Outputs a clean schema tree (tables, columns, types, primary keys, foreign keys).
    - **Interactive REPL & Table Browser (TUI)**:
      - Browse tables and rows in an interactive `ratatui` table viewer.
      - Execute read-only SQL queries directly against memory table dumps.
    - **Automated Java/JDBC Bridge Runner (Optional for `.data` cached tables)**:
      - If cached binary tables (`.data`) or active transaction logs (`.log`) are present, `tb hsql` auto-detects system `java`, transparently launches an embedded lightweight headless `hsqldb.jar` process in the background, connects via pipe/stdio, and streams SQL results directly to the terminal without requiring manual JDBC setup.
  - `tb hsql dump mydb.script --to sqlite` -> Converts an HSQLDB script dump directly into an SQLite database (`mydb.db`) for instant querying with modern tools.

---

## 4. Dependencies & Technology Stack

| Crate | Version | Purpose |
| :--- | :--- | :--- |
| `clap` | `4.x` (derive) | CLI argument parser & subcommands |
| `ratatui` | `0.30+` | Modern terminal UI rendering |
| `crossterm` | `0.29+` | Cross-platform terminal event & raw mode handling |
| `sysinfo` / `netstat2` | Latest | Cross-platform socket & process management |
| `rayon` | `1.x` | High-performance multithreaded directory scanning |
| `walkdir` | `2.x` | Cross-platform directory traversal |
| `chrono` | `0.4+` | Timestamp & timezone parsing |
| `jsonwebtoken` | `9.x` | JWT decoding |
| `sha2` / `md-5` / `base64` | Latest | Cryptographic hashing & encoding |
| `git2` | `0.20+` | Native libgit2 bindings for Git analysis |

---

## 5. Phased Roadmap

| Phase | Milestone | Deliverables |
| :--- | :--- | :--- |
| **Phase 1** | Project Setup & `tb port` | Initialize Cargo workspace, implement cross-platform port scanner, CLI query & kill, and interactive TUI. |
| **Phase 2** | `tb conv` | Implement universal converters (Epoch/Timestamp, JWT token decoder, Hash/Base64 inspector). |
| **Phase 3** | `tb clean` | Implement multithreaded disk artifact scanner (`target`, `node_modules`, `.venv`) + interactive cleaner TUI. |
| **Phase 4** | `tb git` | Implement Git branch sweep and upstream merge detector. |
