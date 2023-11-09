use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::SystemTime;

use clap::Parser;
use clap::ValueEnum;
use csv::WriterBuilder;
use nexmark::config::NexmarkConfig;
use nexmark::config::RateShape;
use nexmark::event::Event;
use nexmark::event::EventType;
use serde::Serialize;
use serde_jsonlines::JsonLinesWriter;

fn split(string: &str) -> Vec<String> {
    string.split(',').map(String::from).collect()
}

fn config() -> NexmarkConfig {
    NexmarkConfig {
        // Maximum number of people to consider as active for placing auctions or bids.
        // Default: 1000
        active_people: 1000,
        // Average number of auction which should be inflight at any time, per generator.
        // Default: 100
        in_flight_auctions: 100,
        // One out of X events will be emitted out of order
        // Default: 1
        out_of_order_group_size: 20,
        // Average idealized size of a 'new person' event, in bytes.
        // Default: 200
        avg_person_byte_size: 200,
        // Average idealized size of a 'new auction' event, in bytes.
        // Default: 500
        avg_auction_byte_size: 500,
        // Average idealized size of a 'new bid' event, in bytes.
        // Default: 100
        avg_bid_byte_size: 100,
        // Ratio of auctions for 'hot' sellers compared to all other people.
        // Default: 4
        hot_seller_ratio: 4,
        // Ratio of bids to 'hot' auctions compared to all other auctions.
        // Default: 2
        hot_auction_ratio: 2,
        // Ratio of bids for 'hot' bidders compared to all other people.
        // Default: 4
        hot_bidder_ratio: 4,
        // Ratio of bids for 'hot' channels compared to all other channels.
        // Default: 2
        hot_channel_ratio: 2,
        // Event id of first event to be generated.
        // Event ids are unique over all generators, and are used as a seed to
        // generate each event's data.
        // Default: 0
        first_event_id: 0,
        // First event number.
        // Generators running in parallel time may share the same event number, and
        // the event number is used to determine the event timestamp.
        // Default: 0
        first_event_number: 0,
        // Time for first event (ms since epoch).
        // Default: SystemTime::now() - SystemTime::UNIX_EPOCH
        base_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        // Auction categories.
        // Default: 5
        num_categories: 5,
        // Use to calculate the next auction id.
        // Default: 10
        auction_id_lead: 10,
        // Person Proportion.
        // Default: 1
        person_proportion: 1,
        // Auction Proportion.
        // Default: 3
        auction_proportion: 3,
        // Bid Proportion.
        // Default: 46
        bid_proportion: 46,
        // We start the ids at specific values to help ensure the queries find a
        // match even on small synthesized dataset sizes.
        // Default: 1000
        first_auction_id: 1000,
        // We start the ids at specific values to help ensure the queries find a
        // match even on small synthesized dataset sizes.
        // Default: 1000
        first_person_id: 1000,
        // We start the ids at specific values to help ensure the queries find a
        // match even on small synthesized dataset sizes.
        // Default: 1000
        first_category_id: 10,
        // Use to calculate the next id.
        // Default: 10
        person_id_lead: 10,
        // Use to calculate inter_event_delays for rate-shape sine.
        // Default: 10
        sine_approx_steps: 10,
        // The collection of U.S. statees
        // Default: az,ca,id,or,wa,wy
        us_states: split("az,ca,id,or,wa,wy"),
        // The collection of U.S. cities.
        // Default: phoenix,los angeles,san francisco,boise,portland,bend,redmond,seattle,kent,cheyenne
        us_cities: split(
            "phoenix,los angeles,san francisco,boise,portland,bend,redmond,seattle,kent,cheyenne",
        ),
        // The collection of hot_channels.
        // Default: Google,Facebook,Baidu,Apple
        hot_channels: split("Google,Facebook,Baidu,Apple"),
        // The collection of first names.
        // Default: peter,paul,luke,john,saul,vicky,kate,julie,sarah,deiter,walter
        first_names: split("peter,paul,luke,john,saul,vicky,kate,julie,sarah,deiter,walter"),
        // The collection of last names.
        // Default: shultz,abrams,spencer,white,bartels,walton,smith,jones,noris
        last_names: split("shultz,abrams,spencer,white,bartels,walton,smith,jones,noris"),
        // Number of event generators to use. Each generates events in its own timeline.
        // Default: 1
        num_event_generators: 1,
        rate_shape: RateShape::Sine,
        rate_period: 600,
        first_rate: 10_000,
        next_rate: 10_000,
        us_per_unit: 1_000_000,
        // The collection of hot urls.
        // hot_urls: (0..4).map(get_base_url).collect(),
        ..Default::default()
    }
}

fn status(n: usize, i: usize) {
    let p = n / 100;
    if i % p == 0 {
        println!("{}%", (i / p) + 1);
    }
}

trait WriteLine: Sized {
    fn write_line(&mut self, row: &impl Serialize) -> Result<(), Box<dyn Error>>;
    fn create(path: PathBuf) -> Result<Self, Box<dyn Error>>;
}

impl WriteLine for csv::Writer<BufWriter<File>> {
    fn write_line(&mut self, row: &impl Serialize) -> Result<(), Box<dyn Error>> {
        Ok(self.serialize(row)?)
    }
    fn create(mut path: PathBuf) -> Result<Self, Box<dyn Error>> {
        path.set_extension("csv");
        Ok(WriterBuilder::new()
            .has_headers(false)
            .delimiter(b',')
            .from_writer(BufWriter::new(File::create(path)?)))
    }
}

impl WriteLine for JsonLinesWriter<BufWriter<File>> {
    fn write_line(&mut self, row: &impl Serialize) -> Result<(), Box<dyn Error>> {
        Ok(self.write(row)?)
    }
    fn create(mut path: PathBuf) -> Result<Self, Box<dyn Error>> {
        path.set_extension("jsonl");
        Ok(JsonLinesWriter::new(BufWriter::new(File::create(path)?)))
    }
}

fn write<T: WriteLine>(
    args: Args,
    iter: impl Iterator<Item = Event>,
) -> Result<(), Box<dyn Error>> {
    let mut persons = T::create(args.dir.join("persons"))?;
    let mut auctions = T::create(args.dir.join("auctions"))?;
    let mut bids = T::create(args.dir.join("bids"))?;

    iter.take(args.num_events)
        .enumerate()
        .try_for_each(|(i, event)| {
            status(args.num_events, i);
            match event {
                Event::Person(row) => persons.write_line(&row),
                Event::Auction(row) => auctions.write_line(&row),
                Event::Bid(row) => bids.write_line(&row),
            }
        })?;
    Ok(())
}

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Number of events to generate.
    #[clap(long, default_value = "100000000")]
    num_events: usize,
    /// Type of event to generate.
    #[clap(long, value_enum, default_value = "all")]
    event_type: Type,
    #[clap(long, default_value = "1")]
    person_proportion: usize,
    #[clap(long, default_value = "3")]
    auction_proportion: usize,
    #[clap(long, default_value = "46")]
    bid_proportion: usize,
    /// Encoding to use for output.
    #[clap(long, value_enum, default_value = "csv")]
    encoding: Encoding,
    /// Directory to write output to.
    #[clap(long, default_value = ".")]
    dir: PathBuf,
}

#[derive(ValueEnum, Clone, Debug)]
enum Encoding {
    Json,
    Csv,
}

#[derive(ValueEnum, Clone, Debug)]
enum Type {
    All,
    Person,
    Auction,
    Bid,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut conf = config();
    conf.person_proportion = args.person_proportion;
    conf.auction_proportion = args.auction_proportion;
    conf.bid_proportion = args.bid_proportion;
    let iter = nexmark::EventGenerator::new(conf);
    let iter = match &args.event_type {
        Type::All => iter,
        Type::Bid => iter.with_type_filter(EventType::Bid),
        Type::Person => iter.with_type_filter(EventType::Person),
        Type::Auction => iter.with_type_filter(EventType::Auction),
    };
    println!(
        "Generating {} events to {}",
        args.num_events,
        args.dir.display(),
    );
    std::fs::create_dir_all(&args.dir)?;
    match args.encoding {
        Encoding::Json => write::<JsonLinesWriter<_>>(args, iter),
        Encoding::Csv => write::<csv::Writer<_>>(args, iter),
    }
}
