# Project Overview

`rsysmetrics` is a system metrics collection agent written in Rust. It gathers system metrics (currently CPU and memory) and exports them to InfluxDB. The agent is designed to be lightweight and configurable.

## Key Technologies

*   **Language:** Rust
*   **Metrics Collection:** `sysinfo` crate for basic system information.
*   **HTTP Client:** `reqwest` for sending metrics to InfluxDB.
*   **Configuration:** `toml` for parsing the configuration file.
*   **Async Runtime:** `tokio` for asynchronous operations.

## Architecture

The application consists of three main components:

1.  **Collectors:** Responsible for gathering specific system metrics. Each collector implements the `Collector` trait.
2.  **Exporters:** Responsible for sending the collected metrics to a time-series database. Currently, only InfluxDB is supported.
3.  **Main Loop:** The main loop orchestrates the collection and export process at a configurable interval.

## Development Conventions

### Code Style

The project follows the standard Rust formatting guidelines. Use `cargo fmt` to format the code.

### Testing

Unit tests are located within the modules they are testing. To run the tests, use:

```bash
cargo test
```

### Collector Implementation

The `sysinfo` crate provides OS-independent implementations, but they often provide too few useful metrics. Therefore, we will always add a Linux-specific collector implementation if it provides more valuable metrics.

# Building and Running

## Building

To build the project, use the following command:

```bash
cargo build --release
```

## Running

To run the agent, you need a configuration file named `rsysmetrics.toml` in the same directory. See `rsysmetrics.toml` for an example configuration.

```bash
./target/release/rsysmetrics
```

### Oneshot Mode

For testing purposes, you can run the agent in "oneshot" mode. This will collect metrics twice, print them to the console, and then exit.

```bash
./target/release/rsysmetrics --oneshot
```
