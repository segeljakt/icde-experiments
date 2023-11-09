use runtime::prelude::tokio::task::JoinSet;
use runtime::prelude::*;

#[data]
pub struct Auction {
    pub id: u64,
    pub item_name: String,
    pub description: String,
    pub initial_bid: u64,
    pub reserve: u64,
    pub date_time: u64,
    pub expires: u64,
    pub seller: u64,
    pub category: u64,
    pub extra: String,
}

#[data]
pub struct Bid {
    auction: u64,
    bidder: u64,
    price: u64,
    channel: String,
    url: String,
    date_time: u64,
    extra: String,
}

#[data]
pub struct Person {
    pub id: u64,
    pub name: String,
    pub email_address: String,
    pub credit_card: String,
    pub city: String,
    pub state: String,
    pub date_time: u64,
    pub extra: String,
}

#[data]
pub struct Query1 {
    auction: u64,
    price: u64,
    bidder: u64,
    date_time: u64,
}

#[data]
pub struct Query2 {
    auction: u64,
    bidder: u64,
}

#[data]
pub struct Query3 {
    name: String,
    city: String,
    state: String,
    id: u64,
}

fn bids(join_set: &mut JoinSet<()>) -> Stream<Bid> {
    Stream::<Bid>::source(
        join_set,
        Reader::file(Path::new("../data/bids.csv"), false),
        Encoding::csv(','),
        TimeSource::event(
            |bid| Time::from_milliseconds(bid.date_time as i128),
            Duration::from_seconds(20),
            Duration::from_seconds(5),
        ),
    )
}

fn persons(join_set: &mut JoinSet<()>) -> Stream<Person> {
    Stream::<Person>::source(
        join_set,
        Reader::file(Path::new("../data/people.csv"), false),
        Encoding::csv(','),
        TimeSource::event(
            |person| Time::from_milliseconds(person.date_time as i128),
            Duration::from_seconds(20),
            Duration::from_seconds(5),
        ),
    )
}

fn auctions(join_set: &mut JoinSet<()>) -> Stream<Auction> {
    Stream::<Auction>::source(
        join_set,
        Reader::file(Path::new("../data/auctions.csv"), false),
        Encoding::csv(','),
        TimeSource::event(
            |auction| Time::from_milliseconds(auction.date_time as i128),
            Duration::from_seconds(20),
            Duration::from_seconds(5),
        ),
    )
}

fn query1(join_set: &mut JoinSet<()>) {
    bids(join_set)
        .map(join_set, |bid| Query1 {
            auction: bid.auction,
            price: bid.price * 100 / 85,
            bidder: bid.bidder,
            date_time: bid.date_time,
        })
        .sink(
            join_set,
            Writer::file(Path::new("../data/rust-opt-query1.csv")),
            Encoding::csv(','),
        );
}

fn query2(join_set: &mut JoinSet<()>) {
    bids(join_set)
        .filter(join_set, |bid| {
            bid.auction == 1007
                || bid.auction == 1020
                || bid.auction == 2001
                || bid.auction == 2019
                || bid.auction == 2087
        })
        .map(join_set, |bid| Query2 {
            auction: bid.auction,
            bidder: bid.bidder,
        })
        .sink(
            join_set,
            Writer::file(Path::new("../data/rust-opt-query2.csv")),
            Encoding::csv(','),
        );
}

fn query3(join_set: &mut JoinSet<()>) {
    let filtered = persons(join_set).filter(join_set, |person| {
        person.state == String::from("OR")
            || person.state == String::from("ID")
            || person.state == String::from("CA")
    });
    auctions(join_set)
        .filter(join_set, |auction| auction.category == 10)
        .window_join(
            join_set,
            filtered,
            |auction| auction.seller,
            |person| person.id,
            Duration::from_minutes(1),
            |auction, person| (auction, person),
        )
        .map(join_set, |(auction, person)| Query3 {
            name: person.name,
            city: person.city,
            state: person.state,
            id: auction.id,
        })
        .sink(
            join_set,
            Writer::file(Path::new("../data/rust-opt-query3.csv")),
            Encoding::csv(','),
        );
}

fn query3_opt(join_set: &mut JoinSet<()>) {
    let persons = persons(join_set);
    auctions(join_set)
        .window_join(
            join_set,
            persons,
            |auction| auction.seller,
            |person| person.id,
            Duration::from_minutes(1),
            |auction, person| (auction, person),
        )
        .filter(join_set, |(auction, person)| {
            (person.state == String::from("OR")
                || person.state == String::from("ID")
                || person.state == String::from("CA"))
                && auction.category == 10
        })
        .map(join_set, |(auction, person)| Query3 {
            name: person.name,
            city: person.city,
            state: person.state,
            id: auction.id,
        })
        .sink(
            join_set,
            Writer::file(Path::new("../data/rust-opt-query3.csv")),
            Encoding::csv(','),
        );
}

fn query4(join_set: &mut JoinSet<()>) {
    let bids = bids(join_set);
    auctions(join_set)
        .window_join(
            join_set,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            Duration::from_minutes(1),
            |auction, bid| (auction, bid),
        )
        .filter(join_set, |(auction, bid)| bid.date_time < auction.expires)
        .map(join_set, |(auction, bid)| {
            (auction.id, auction.category, bid.price)
        })
        .keyby(join_set, |(id, _category, _price)| id)
        .window(
            join_set,
            Assigner::tumbling(Duration::from_minutes(1)),
            Aggregator::incremental(
                |(_, category, price)| (category, price),
                |(category0, price0), (category1, price1)| {
                    if price0 > price1 {
                        (category0, price0)
                    } else {
                        (category1, price1)
                    }
                },
                |(category, price)| (category, price),
            ),
        )
        .keyby(join_set, |(category, _)| category)
        .window(
            join_set,
            Assigner::tumbling(Duration::from_minutes(1)),
            Aggregator::incremental(
                |(_, price)| (price, 1),
                |(sum0, count0), (sum1, count1)| (sum0 + sum1, count0 + count1),
                |(price, count)| price / count,
            ),
        )
        .unkey(join_set)
        .sink(
            join_set,
            Writer::file(Path::new("../data/rust-opt-query4.csv")),
            Encoding::csv(','),
        );
}

fn main() {
    std::env::set_current_dir(std::env!("CARGO_MANIFEST_DIR")).unwrap();
    let log = Logger::file("log");
    let mut args = std::env::args().skip(1);
    let query = args.next().expect("no query specified");
    let mut runner = TaskParallelRunner::new(log);
    let time = std::time::Instant::now();
    match query.as_str() {
        "query1" => runner.spawn(query1),
        "query2" => runner.spawn(query2),
        "query3" => runner.spawn(query3),
        "query3-opt" => runner.spawn(query3_opt),
        "query4" => runner.spawn(query4),
        _ => panic!("unknown query"),
    };
    runner.await_termination();
    let duration = time.elapsed();
    println!("Job Runtime: {} ms", duration.as_millis());
}
