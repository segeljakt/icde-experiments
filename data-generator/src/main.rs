use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use csv::WriterBuilder;
use nexmark::config::NexmarkConfig;
use nexmark::event::Event;
use nexmark::event::EventType;

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Number of events to generate.
    #[clap(long, default_value = "100000000")]
    num_events: usize,
    #[clap(long, default_value = "46")]
    bid_proportion: usize,
    #[clap(long, default_value = "3")]
    auction_proportion: usize,
    #[clap(long, default_value = "1")]
    person_proportion: usize,
    /// Directory to write output to.
    #[clap(long, default_value = ".")]
    dir: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let conf = NexmarkConfig {
        // Set to 1700000000 (Tue Nov 14 2023 22:13:20 GMT+0000) to make events reproducible.
        base_time: 1700000000,
        bid_proportion: args.bid_proportion,
        auction_proportion: args.auction_proportion,
        person_proportion: args.person_proportion,
        ..Default::default()
    };
    let iter = nexmark::EventGenerator::new(conf);
    println!(
        "Generating {} events of each category to {}",
        args.num_events,
        args.dir.display(),
    );
    std::fs::create_dir_all(&args.dir)?;

    let divisor = args.bid_proportion + args.auction_proportion + args.person_proportion;

    for (name, ty, proportion) in [
        ("persons", EventType::Person, args.person_proportion),
        ("auctions", EventType::Auction, args.auction_proportion),
        ("bids", EventType::Bid, args.bid_proportion),
    ] {
        if proportion == 0 {
            continue;
        }
        let n = args.num_events * proportion / divisor;

        let file = File::create(args.dir.join(name).with_extension("csv"))?;
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(BufWriter::new(file));
        iter.clone()
            .with_type_filter(ty)
            .take(n)
            .enumerate()
            .try_for_each(|(i, event)| {
                let p = n / 100;
                if i % p == 0 {
                    println!("{name}: {}%", (i / p) + 1);
                }
                match event {
                    Event::Person(row) if ty == EventType::Person => writer.serialize(&row),
                    Event::Auction(row) if ty == EventType::Auction => writer.serialize(&row),
                    Event::Bid(row) if ty == EventType::Bid => writer.serialize(&row),
                    _ => unreachable!(),
                }
            })?;
    }

    Ok(())
}
