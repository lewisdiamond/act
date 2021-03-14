use act::parse::parse;
use act::process::process;
use act::stores::MemActStore;
use act::types::Transaction;
use clap::{App, Arg};
use std::collections::HashMap;
use std::fs;
use std::io::{stdin, BufRead, BufReader};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let matches = App::new("act")
        .version("0.1")
        .about("Merges transactions into final account state")
        .author("Lewis Diamond")
        .arg(
            Arg::with_name("input")
                .required(false)
                .index(1)
                .help("Input file, stdin if omitted or -"),
        )
        .get_matches();

    let input: Box<dyn BufRead> = match matches.value_of("input") {
        Some("-") | Some("") | None => Box::new(BufReader::new(stdin())),
        Some(f) => Box::new(BufReader::new(fs::File::open(f).unwrap())),
    };

    let mut act_store = MemActStore::new();
    let mut tx_store: HashMap<u32, Transaction> = HashMap::new();
    let s = parse(input);
    tokio::pin!(s);
    while let Some(v) = s.next().await {
        process(v, &mut act_store, &mut tx_store);
    }

    let mut writer = csv::WriterBuilder::new().from_writer(std::io::stdout());

    for act in act_store.into_iter() {
        writer.serialize(act.1).unwrap();
    }
}
