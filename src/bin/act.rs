use act::parse::parse;
use act::process::{InternalTransaction, process};
use act::stores::MemActStore;
use clap::{Arg, ArgAction::Count, command};
use log::{LevelFilter, warn};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, stdin};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(
            Arg::new("input")
                .required(false)
                .index(1)
                .help("Input file, stdin if omitted or -"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .required(false)
                .action(Count),
        )
        .get_matches();

    let level = match matches.get_count("debug") {
        1 => Some(LevelFilter::Warn),
        2 => Some(LevelFilter::Info),
        3 => Some(LevelFilter::Debug),
        4 => Some(LevelFilter::Trace),
        _ => None,
    };
    let mut logger = env_logger::Builder::new();
    logger.parse_env(env_logger::DEFAULT_FILTER_ENV);
    if let Some(l) = level {
        logger.filter_level(l);
    };
    logger.init();

    let input_arg = matches
        .get_one::<String>("input")
        .map(|s| s.as_str())
        .unwrap_or("-");
    let input: Box<dyn BufRead> = match input_arg {
        "-" | "" => Box::new(BufReader::new(stdin())),
        f => Box::new(BufReader::new(fs::File::open(f).unwrap())),
    };

    let mut act_store = MemActStore::new();
    let mut tx_store: HashMap<u32, InternalTransaction> = HashMap::new();
    let s = parse(input);
    tokio::pin!(s);
    while let Some(v) = s.next().await {
        if let Err(e) = process(v.clone(), &mut act_store, &mut tx_store) {
            warn!("Invalid transaction: {:?} {}", v, e);
        }
    }

    let mut writer = csv::WriterBuilder::new().from_writer(std::io::stdout());

    for act in act_store.into_iter() {
        writer.serialize(act.1).unwrap();
    }
}
