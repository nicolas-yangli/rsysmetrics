# rsysmetrics

`rsysmetrics` is a system metrics collection agent written in Rust. It gathers system metrics (currently CPU, memory, disk, GPU, network, system, and temperature) and exports them to InfluxDB. The agent is designed to be lightweight and configurable.

![Grafana Dashboard](/contrib/grafana-dashboard.png)

## Building

### Generic

To build the project, use the following command:

```bash
cargo build --release
```

### Arch Linux

This project includes a `PKGBUILD` file for creating an Arch Linux package. To build and install the package, run the following commands from the project root directory:

```bash
makepkg -si
```

## Running

To run the agent, you need a configuration file. You can specify the path to the configuration file using the `--config` or `-c` flag. If not specified, it will look for `rsysmetrics.toml` in the same directory. See `rsysmetrics.toml` for an example configuration.

```bash
./target/release/rsysmetrics
```

### Oneshot Mode

For testing purposes, you can run the agent in "oneshot" mode. This will collect metrics twice, print them to the console, and then exit.

```bash
./target/release/rsysmetrics --oneshot
```

### Systemd Service

The included `rsysmetrics.service` file allows you to run `rsysmetrics` as a systemd service. This is the recommended way to run the agent in production.

To enable and start the service, run the following commands:

```bash
systemctl enable rsysmetrics.service
systemctl start rsysmetrics.service
```
