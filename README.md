# bera-snap 🐻⛓️📦

Auto snapshots tool for [berachain-docker-node](https://github.com/upnodedev/berachain-docker-node) with configurable cron scheduling and optional Google Cloud Storage (GCS) upload.

## Getting Started

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed (>= 1.81.0) and [Application Default Credentials (ADC)](https://cloud.google.com/docs/authentication/provide-credentials-adc) configured if you have enabled uploading to Google Cloud Storage (`--gcs` flag).

## Installation

```bash
cargo install --path .
bera-snap -h
```

## Usage

### Start Automated Snapshots

#### Local Storage Only

```bash
bera-snap start \
  --path "/root/berachain-docker-node" \
  --job-time "0 0 0 * * *"
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

### Command-line Options

- `--path <PATH>`: Path to the Berachain Docker node (required)
- `--job-time <JOB_TIME>`: Cron job schedule for taking snapshots (required). For pattern syntax, see [Croner-rust docs](https://github.com/Hexagon/croner-rust?tab=readme-ov-file#pattern)
- `--gcs`: Enable Google Cloud Storage upload
- `--gcs-bucket <GCS_BUCKET>`: GCS bucket name (required if `--gcs` is set)
- `--gcs-folder <GCS_FOLDER>`: GCS folder path (required if `--gcs` is set)

## Environment Variables

You can also set options using environment variables:

- `NODE_PATH`: Equivalent to `--path`
- `CRON_JOB_TIME`: Equivalent to `--job-time`
- `GCS_BUCKET`: Equivalent to `--gcs-bucket`
- `GCS_FOLDER`: Equivalent to `--gcs-folder`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
