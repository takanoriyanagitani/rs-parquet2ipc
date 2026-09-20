use std::io;
use std::process::ExitCode;

use rs_parquet2ipc::PqConfig;

use rs_parquet2ipc::BATCH_SIZE_DEFAULT;
use rs_parquet2ipc::OFFSET_DEFAULT;

fn io_envkey2str(key: &'static str) -> impl Fn() -> String {
    move || std::env::var(key).unwrap_or_default()
}

fn io_envkey2usize(key: &'static str) -> impl Fn() -> Option<usize> {
    move || {
        let val: String = io_envkey2str(key)();
        str::parse(&val).ok()
    }
}

fn io_envkey2usize_alt(key: &'static str, alt: usize) -> impl Fn() -> usize {
    move || {
        let ouz: Option<usize> = io_envkey2usize(key)();
        ouz.unwrap_or(alt)
    }
}

fn io_batch_size() -> impl Fn() -> usize {
    io_envkey2usize_alt("ENV_BATCH_SIZE", BATCH_SIZE_DEFAULT)
}

fn io_limit() -> impl Fn() -> Option<usize> {
    io_envkey2usize("ENV_LIMIT")
}

fn io_offset() -> impl Fn() -> usize {
    io_envkey2usize_alt("ENV_OFFSET", OFFSET_DEFAULT)
}

fn io_pq_filename() -> impl Fn() -> String {
    io_envkey2str("ENV_PQ_FILENAME")
}

fn io_config() -> impl Fn() -> PqConfig {
    move || {
        let batch_size: usize = io_batch_size()();
        let limit: Option<usize> = io_limit()();
        let offset: usize = io_offset()();
        let pqfilename: String = io_pq_filename()();

        PqConfig {
            batch_size,
            limit,
            offset,
            pqfilename,
        }
    }
}

fn io_main() -> impl FnMut() -> Result<(), io::Error> {
    move || {
        let cfg: PqConfig = io_config()();
        cfg.reader2stdout()
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let cfg: PqConfig = io_config()();

        let batch_size: usize = cfg.batch_size;
        let limit: Option<usize> = cfg.limit;
        let offset: usize = cfg.offset;
        let pqfilename: String = cfg.pqfilename;

        eprintln!("ENV_BATCH_SIZE: {batch_size}");
        eprintln!("ENV_LIMIT: {limit:#?}");
        eprintln!("ENV_OFFSET: {offset}");
        eprintln!("ENV_PQ_FILENAME: {pqfilename}");

        ExitCode::FAILURE
    })
}
