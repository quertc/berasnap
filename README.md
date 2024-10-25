# bera-snap 🐻⛓️📦

Auto snapshots tool for [berachain-docker-node](https://github.com/upnodedev/berachain-docker-node) with configurable cron scheduling, REST API for snapshot distribution, and optional Google Cloud Storage (GCS) upload. Both API and GCS features can be enabled simultaneously.

## Getting Started

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed (>= 1.81.0) and [Application Default Credentials (ADC)](https://cloud.google.com/docs/authentication/provide-credentials-adc) configured if you have enabled uploading to Google Cloud Storage (`--gcs` flag).

## Installation

```bash
cargo install --path .
bera-snap -h
```

## Usage

### Start Automated Snapshots

#### Basic Usage

```bash
bera-snap start \
  --path "/root/berachain-docker-node" \
  --job-time "0 0 0 * * *"
```

#### With API Enabled

```bash
bera-snap start \
  --path "/root/berachain-docker-node" \
  --job-time "0 0 0 * * *" \
  --api
```

#### With GCS Upload

```bash
bera-snap start \
  --path "/root/berachain-docker-node" \
  --job-time "0 0 0 * * *" \
  --gcs \
  --gcs-bucket "mybucket" \
  --gcs-folder "berachain/snapshots"
```

#### With Both API and GCS

```bash
bera-snap start \
  --path "/root/berachain-docker-node" \
  --job-time "0 0 0 * * *" \
  --api \
  --gcs \
  --gcs-bucket "mybucket" \
  --gcs-folder "berachain/snapshots"
```

### Command-line Options

- `--path <PATH>`: Path to the Berachain Docker node (required)
- `--storage-path <STORAGE_PATH>`: Path to store local snapshots (default: "storage")
- `--job-time <JOB_TIME>`: Cron job schedule for taking snapshots (required). For pattern syntax, see [Croner-rust docs](https://github.com/Hexagon/croner-rust?tab=readme-ov-file#pattern)
- `--api`: Enable API for snapshot distribution
- `--api-port <API_PORT>`: API server port (default: 3050, required if `--api` is set)
- `--gcs`: Enable Google Cloud Storage upload
- `--gcs-bucket <GCS_BUCKET>`: GCS bucket name (required if `--gcs` is set)
- `--gcs-folder <GCS_FOLDER>`: GCS folder path (required if `--gcs` is set)
- `--keep <KEEP>`: Number of snapshots to keep (default: 1)

## Environment Variables

You can also set options using environment variables:

- `NODE_PATH`: Equivalent to `--path`
- `STORAGE_PATH`: Equivalent to `--storage-path`
- `CRON_JOB_TIME`: Equivalent to `--job-time`
- `API_PORT`: Equivalent to `--api-port`
- `GCS_BUCKET`: Equivalent to `--gcs-bucket`
- `GCS_FOLDER`: Equivalent to `--gcs-folder`
- `SNAPS_KEEP`: Equivalent to `--keep`

## API Endpoints

When API is enabled (`--api` flag), the following endpoints are available:

- `GET /snapshots`: List all available snapshots
- `GET /snapshots/:filename`: Download a specific snapshot

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
