# SPL Compiler in Rust – Bedienung & Installation

## Voraussetzungen

Der SPL-Compiler ist in Rust geschrieben und wird
über die Kommandozeile gesteuert. Für eine reibungslose
Nutzung empfiehlt es sich, die aktuelle Version von Rust zu verwenden.

### Rust installieren oder aktualisieren

Falls Rust noch nicht installiert ist,
kann es unter Linux mit folgendem Befehl eingerichtet werden:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Für Windows- oder macOS-Nutzer\:innen steht ein grafischer Installer zur Verfügung:

[https://rustup.rs](https://rustup.rs)

Falls Rust bereits installiert ist, kann es mit folgendem Befehl aktualisiert werden:

```bash
rustup update
```

Nach erfolgreicher Installation sollte folgender Befehl
die installierte Rust-Version anzeigen:

```bash
rustc --version
```

## Cargo – Das Build-System von Rust

Rust verwendet das integrierte Tool Cargo zur Paketverwaltung,
zum Kompilieren und zum Ausführen von Projekten.

### Projektstruktur

```text
.
├── Cargo.toml         # Projekt-Konfiguration
├── src/
│   └── main.rs        # Einstiegspunkt
│   └── ...            # Module und das restliche Projekt
├── target/            # Build-Ausgabe
```

### Build-Optionen

* Debug-Build (schnell, keine Optimierungen):

  ```bash
  cargo build
  ```

  → Ausgabe liegt unter `target/debug/`

* Release-Build (optimiert, performant):

  ```bash
  cargo build --release
  ```

  → Ausgabe liegt unter `target/release/`

* Direkt ausführen (mit Argumenten):

  ```bash
  cargo run -- <argumente>
  ```

## Bedienung des SPL-Compilers

Nach dem Kompilieren kann der Compiler entweder über
Cargo oder direkt als Binary gestartet werden:

```bash
cargo run -- <optionen> <datei.spl>
```

Oder direkt aus dem target-Ordner:

```bash
./target/debug/spl-rust --help
```

### Verfügbare Optionen

```bash
Usage: spl-rust [OPTIONS] <file>

Arguments:
  <file>  Path to SPL code

Options:
  -p, --parse             Parse input file, returns the abstract syntax tree
  -t, --tables            Fills symbol tables and prints them
  -s, --semant            Semantic analysis
  -3, --tac               Generates three address code
  -P, --proc <name>       Name of the procedure to be examined
  -O, --optis <optis>...  Optimizations to apply: [cse, rch, lv, dead, gcp, scc, licm]
  -d, --dot[=<output>]    Generates block graph
  -h, --help              Print help
  -V, --version           Print version

```

### Beispiele

* Nur Parsen und AST anzeigen:

  ```bash
  cargo run -- -p examples/beispiel1.spl
  ```

* Symboltabellen anzeigen:

  ```bash
  cargo run -- -t examples/beispiel1.spl
  ```

* Optimierungen durchführen und Blockgraph exportieren:

  ```bash
  cargo run -- -O licm -d graph.dot examples/beispiel1.spl
  ```

## Beispiele & Tests

Im Ordner `spl-testfiles/` befinden sich
SPL-Beispielprogramme zur Demonstration und zum Testen.

Unit- oder Integrationstests können sie mit folgendem Befehl ausgeführt werden:

```bash
cargo test
```
