use core::time::Duration;
use std::fs::File;
use std::path::Path;

use clap::{AppSettings, Parser};
use parquet::file::reader::{ChunkReader, FileReader, SerializedFileReader};
use parquet::record::reader::RowIter;
use parquet::schema::printer::print_schema;
use rusoto_core::Region;
use url::Url;

mod http_reader;
use http_reader::HttpChunkReader;
mod s3_reader;
use s3_reader::S3ChunkReader;

enum Source {
    File(String),
    Http(String),
    S3(String),
}

fn output_rows(iter: RowIter, offset: u32, limit: i32) {
    let mut input_rows_count = 0;
    let mut output_rows_count = 0;
    for record in iter {
        input_rows_count += 1;
        if input_rows_count < offset {
            continue;
        }

        output_rows_count += 1;
        if limit > -1 && output_rows_count > limit {
            return;
        }

        println!("{}", record.to_json_value());
    }
}

fn output_thrift_schema<R: 'static + ChunkReader>(file_reader: &SerializedFileReader<R>) {
    let parquet_metadata = file_reader.metadata();
    print_schema(
        &mut std::io::stdout(),
        parquet_metadata.file_metadata().schema(),
    );
}

async fn print_json_from(
    source: Source,
    offset: u32,
    limit: i32,
    should_output_schema: bool,
    timeout: Duration,
) {
    match source {
        Source::File(path) => {
            let file = File::open(&Path::new(&path)).unwrap();
            let file_reader = SerializedFileReader::new(file).unwrap();

            if should_output_schema {
                output_thrift_schema(&file_reader);
            } else {
                output_rows(file_reader.get_row_iter(None).unwrap(), offset, limit);
            }
        }
        Source::Http(url_str) => {
            let mut reader = HttpChunkReader::new_unknown_size(url_str).await;
            reader.start(timeout);

            let blocking_task = tokio::task::spawn_blocking(move || {
                let file_reader = SerializedFileReader::new(reader).unwrap();

                if should_output_schema {
                    output_thrift_schema(&file_reader);
                } else {
                    output_rows(file_reader.get_row_iter(None).unwrap(), offset, limit);
                }
            });
            blocking_task.await.unwrap();
        }
        Source::S3(url_str) => {
            let url = Url::parse(&url_str).unwrap();
            let host_str = url.host_str().unwrap();
            let key = &url.path()[1..];

            let mut reader = S3ChunkReader::new_unknown_size(
                (String::from(host_str), String::from(key)),
                Region::default(),
            )
            .await;
            reader.start(Region::default(), timeout).await;

            let blocking_task = tokio::task::spawn_blocking(move || {
                let file_reader = SerializedFileReader::new(reader).unwrap();

                if should_output_schema {
                    output_thrift_schema(&file_reader);
                } else {
                    output_rows(file_reader.get_row_iter(None).unwrap(), offset, limit);
                }
            });
            blocking_task.await.unwrap();
        }
    };
}

#[derive(Parser)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(version, about, long_about = None)]
struct Cli {
    /// Location of Parquet input file (file path, HTTP or S3 URL)
    file: String,

    /// Starts outputting from this row
    #[clap(default_value_t = 0, short, long, parse(try_from_str))]
    offset: u32,

    /// Maximum number of rows to output
    #[clap(short, long, parse(try_from_str))]
    limit: Option<i32>,

    /// Request timeout in seconds
    #[clap(default_value_t = 60, short, long, parse(try_from_str))]
    timeout: u16,

    /// Outputs thrift schema first
    #[clap(short, long)]
    schema_output: Option<bool>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let output_thrift_schema = cli.schema_output.unwrap_or(false);
    let offset = cli.offset;
    let limit: i32 = cli.limit.unwrap_or(-1);
    let timeout = Duration::from_secs(cli.timeout.into());
    let file = cli.file;

    if file.as_str().starts_with("s3://") {
        print_json_from(
            Source::S3(file),
            offset,
            limit,
            output_thrift_schema,
            timeout,
        )
        .await;
    } else if file.as_str().starts_with("http") {
        print_json_from(
            Source::Http(file),
            offset,
            limit,
            output_thrift_schema,
            timeout,
        )
        .await;
    } else {
        print_json_from(
            Source::File(file),
            offset,
            limit,
            output_thrift_schema,
            timeout,
        )
        .await;
    }
}
