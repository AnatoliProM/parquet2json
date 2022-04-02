# parquet2json

A command-line tool for converting [Parquet](https://parquet.apache.org) to [newline-delimited JSON](https://en.wikipedia.org/wiki/JSON_streaming#Line-delimited_JSON).

It uses the excellent [Apache Parquet Official Native Rust Implementation](https://github.com/apache/arrow-rs/tree/master/parquet).

## How to use it

Install from [crates.io](https://crates.io) and execute from the command line, e.g.:

```shell
$ cargo install parquet2json
$ parquet2json --help

USAGE:
    parquet2json [OPTIONS] <FILE>

ARGS:
    <FILE>    Location of Parquet input file (file path, HTTP or S3 URL)

OPTIONS:
    -o, --offset <OFFSET>                  Starts outputting from this row [default: 0]
    -l, --limit <LIMIT>                    Maximum number of rows to output
    -t, --timeout <TIMEOUT>                Request timeout in seconds [default: 60]
    -s, --schema-output <SCHEMA_OUTPUT>    Outputs thrift schema only
    -c, --columns <COLUMNS>                Select columns by name (comma,separated)
    -h, --help                             Print help information
```

### S3 Settings

Credentials are provided as per standard AWS toolchain, i.e. per environment variables (`AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`), AWS credentials file or IAM ECS container/instance profile.

The default AWS region must be set per environment variable (`AWS_DEFAULT_REGION`) o in AWS credentials file and must match region of the bucket the bucket is located in.

### Examples

Use it to stream output to files and other tools such as `grep` and [jq](https://stedolan.github.io/jq/).

#### Output to a file

```shell
$ parquet2json ./myfile.pq > output.ndjson
```

#### Filter with jq

```shell
$ parquet2json ./myfile.pq | jq 'select(.level==3) | .id'
```

#### From S3 or HTTP (S3)

```shell
$ parquet2json s3://amazon-reviews-pds/parquet/product_category=Gift_Card/part-00000-495c48e6-96d6-4650-aa65-3c36a3516ddd.c000.snappy.parquet
```

```shell
$ parquet2json https://amazon-reviews-pds.s3.us-east-1.amazonaws.com/parquet/product_category%3DGift_Card/part-00000-495c48e6-96d6-4650-aa65-3c36a3516ddd.c000.snappy.parquet
```

## License

[MIT](LICENSE.md)
