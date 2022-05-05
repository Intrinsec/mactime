# Mactime

[License](./LICENSE)

Rust implementation of [mactime.pl](https://github.com/sleuthkit/sleuthkit/blob/master/tools/timeline/mactime.base)

Generate a MACB timeline in CSV format from a bodyfile.

## Build

`cargo build --release`

## Usage

```text
USAGE:
    mactime.exe [OPTIONS] --bodyfile <bodyfile>

OPTIONS:
    -b, --bodyfile <bodyfile>
    -f, --filter <filter>        Date filter format: YYYY-MM-DD..YYYY-MM-DD (time not handled yet)
    -h, --help                   Print help information
    -o, --output <output>        CSV output to file (stdout if not specified)
    -s, --sort                   Sort timeline by datetime
```

## Debug

`cargo run -- --bodyfile <bodyfile>`
