use std::{fs::OpenOptions, io::Write, num::NonZeroUsize, path::PathBuf};

use anyhow::Result;
use clap::{error::ErrorKind, CommandFactory, Parser, ValueEnum};
use mac_address::get_mac_address;
use rand::{thread_rng, Rng};
use uuid::{timestamp::context::Context, Timestamp, Uuid};

#[derive(Clone, Copy, ValueEnum)]
enum Version {
    #[value(name = "1")]
    V1,
    #[value(name = "3")]
    V3,
    #[value(name = "4")]
    V4,
    #[value(name = "5")]
    V5,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    #[value(name = "BIN")]
    Binary,
    #[value(name = "STR")]
    String,
    #[value(name = "SIV")]
    Siv,
}

#[derive(Parser)]
struct Cli {
    #[clap(short = 'v')]
    version: Version,
    #[clap(short = 'm', default_value_t = false)]
    random_mac: bool,
    #[clap(short = 'n', default_value_t = NonZeroUsize::new(1).unwrap())]
    count: NonZeroUsize,
    #[clap(short = '1')]
    reset_context: bool,
    #[clap(short = 'F', default_value = "STR")]
    format: Format,
    #[clap(short = 'o')]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.count.get() == 1 && cli.reset_context {
        let mut cmd = Cli::command();
        cmd.error(
            ErrorKind::ArgumentConflict,
            "'-1' used while '-n' is set to 1. Invalid input",
        )
        .exit();
    }

    let mut output = match cli.output {
        Some(path) => Box::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)?,
        ) as Box<dyn Write>,
        None => Box::new(std::io::stdout().lock()),
    };

    let node_id = if cli.random_mac {
        let mut rng = thread_rng();
        rng.gen()
    } else {
        let mac = get_mac_address()?;
        match mac {
            Some(mac) => mac.bytes(),
            None => panic!("No MAC address found"),
        }
    };

    let mut context = Context::new_random();

    for _ in 0..cli.count.get() {
        let uuid = match cli.version {
            Version::V1 => Uuid::new_v1(Timestamp::now(&context), &node_id),
            Version::V3 => Uuid::new_v3(&Uuid::NAMESPACE_URL, b"http://www.ossp.org/"),
            Version::V4 => Uuid::new_v4(),
            Version::V5 => Uuid::new_v5(&Uuid::NAMESPACE_URL, b"http://www.ossp.org/"),
        };

        if cli.reset_context {
            context = Context::new_random();
        }

        match cli.format {
            Format::String => writeln!(output, "{uuid}").unwrap(),
            Format::Binary => output.write_all(uuid.as_bytes()).unwrap(),
            Format::Siv => writeln!(output, "{}", uuid.as_u128()).unwrap(),
        }
    }

    Ok(())
}
