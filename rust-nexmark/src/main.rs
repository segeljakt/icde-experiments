mod utils;

use nexmark::config::NexmarkConfig;
use runtime::prelude::*;

#[data]
pub struct Auction {
    pub id: usize,
    pub item_name: String,
    pub description: String,
    pub initial_bid: usize,
    pub reserve: usize,
    pub date_time: u64,
    pub expires: u64,
    pub seller: usize,
    pub category: usize,
    pub extra: String,
}

#[data]
pub struct Bid {
    pub auction: usize,
    pub bidder: usize,
    pub price: usize,
    pub channel: String,
    pub url: String,
    pub date_time: u64,
    pub extra: String,
}

#[data]
pub struct Person {
    pub id: usize,
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
    auction: usize,
    price: usize,
    bidder: usize,
    date_time: u64,
}

#[data]
pub struct Query2 {
    auction: usize,
    bidder: usize,
}

#[data]
pub struct Query3 {
    name: String,
    city: String,
    state: String,
    id: usize,
}

fn query1(bids: Stream<Bid>, ctx: &mut Context) {
    bids.map(ctx, |bid| Query1 {
        auction: bid.auction,
        price: bid.price * 100 / 85,
        bidder: bid.bidder,
        date_time: bid.date_time,
    })
    .drain(ctx);
}

fn query2(bids: Stream<Bid>, ctx: &mut Context) {
    bids.filter(ctx, |bid| {
        bid.auction == 1007
            || bid.auction == 1020
            || bid.auction == 2001
            || bid.auction == 2019
            || bid.auction == 2087
    })
    .map(ctx, |bid| Query2 {
        auction: bid.auction,
        bidder: bid.bidder,
    })
    .drain(ctx);
}

fn query3(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    auctions
        .window_join(
            ctx,
            persons,
            |auction| auction.seller,
            |person| person.id,
            Duration::from_seconds(10),
            |auction, person| (auction, person),
        )
        .filter(ctx, |(auction, person)| {
            (person.state == String::from("OR")
                || person.state == String::from("ID")
                || person.state == String::from("CA"))
                && auction.category == 10
        })
        .map(ctx, |(auction, person)| Query3 {
            name: person.name,
            city: person.city,
            state: person.state,
            id: auction.id,
        })
        .drain(ctx);
}

fn query3_opt(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    let filtered = persons.filter(ctx, |person| {
        person.state == String::from("OR")
            || person.state == String::from("ID")
            || person.state == String::from("CA")
    });
    auctions
        .filter(ctx, |auction| auction.category == 10)
        .window_join(
            ctx,
            filtered,
            |auction| auction.seller,
            |person| person.id,
            Duration::from_seconds(10),
            |auction, person| (auction, person),
        )
        .map(ctx, |(auction, person)| Query3 {
            name: person.name,
            city: person.city,
            state: person.state,
            id: auction.id,
        })
        .drain(ctx);
}

fn query4(auctions: Stream<Auction>, bids: Stream<Bid>, ctx: &mut Context) {
    auctions
        .window_join(
            ctx,
            bids,
            |auction| auction.id,
            |bid| bid.auction,
            Duration::from_seconds(10),
            |auction, bid| (auction, bid),
        )
        .filter(ctx, |(auction, bid)| bid.date_time < auction.expires)
        .map(ctx, |(auction, bid)| {
            (auction.id, auction.category, bid.price)
        })
        .keyby(ctx, |(id, _category, _price)| id)
        .window(
            ctx,
            Assigner::tumbling(Duration::from_seconds(10)),
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
        .keyby(ctx, |(category, _)| category)
        .window(
            ctx,
            Assigner::tumbling(Duration::from_seconds(10)),
            Aggregator::incremental(
                |(_, price)| (price, 1),
                |(sum0, count0), (sum1, count1)| (sum0 + sum1, count0 + count1),
                |(price, count)| price / count,
            ),
        )
        .drain(ctx);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let query = args.next().expect("No `query` specified");
    let n: usize = args.next().expect("No `n` specified").parse().unwrap();
    let dir = args.next().expect("No `dir` specified");
    let mode = args.next().unwrap_or("generate".to_string());

    match mode.as_str() {
        "memory" => {
            let b = utils::csv_to_vec(&format!("{dir}/bids.csv")).into_iter();
            let a = utils::csv_to_vec(&format!("{dir}/auctions.csv")).into_iter();
            let p = utils::csv_to_vec(&format!("{dir}/persons.csv")).into_iter();
            run(n, query.as_str(), b, a, p);
        }
        "mmap" => {
            let b = utils::mmap_csv(&format!("{dir}/bids.csv")).into_iter();
            let a = utils::mmap_csv(&format!("{dir}/auctions.csv")).into_iter();
            let p = utils::mmap_csv(&format!("{dir}/persons.csv")).into_iter();
            run(n, query.as_str(), b, a, p);
        }
        "generate" => {
            let config = NexmarkConfig::default();
            let generator = nexmark::EventGenerator::new(config);
            let b = generator
                .clone()
                .with_type_filter(nexmark::event::EventType::Bid)
                .filter_map(utils::to_bid);
            let a = generator
                .clone()
                .with_type_filter(nexmark::event::EventType::Auction)
                .filter_map(utils::to_auction);
            let p = generator
                .with_type_filter(nexmark::event::EventType::Person)
                .filter_map(utils::to_person);
            run(n, query.as_str(), b, a, p);
        }
        _ => panic!("unknown mode"),
    }
}

pub fn iter_to_stream<T: Data>(
    ctx: &mut Context,
    iter: impl Iterator<Item = T> + Send + 'static,
    f: fn(T) -> Time,
) -> Stream<T> {
    Stream::from_iter(ctx, iter, f, 1000, Duration::from_milliseconds(100))
}

fn run(
    n: usize,
    query: &str,
    b: impl Iterator<Item = Bid> + Send + 'static,
    a: impl Iterator<Item = Auction> + Send + 'static,
    p: impl Iterator<Item = Person> + Send + 'static,
) {
    let nb = ((n as f64) * 0.92) as usize;
    let na = ((n as f64) * 0.06) as usize;
    let np = ((n as f64) * 0.02) as usize;

    println!("n: {}, nb: {}, na: {}, np: {}", n, nb, na, np);

    let b = b.take(nb);
    let a = a.take(na);
    let p = p.take(np);

    let time = std::time::Instant::now();
    CurrentThreadRunner::run(move |ctx| match query {
        "query1" => {
            let b = iter_to_stream(ctx, b, |b| Time::from_milliseconds(b.date_time as i128));
            query1(b, ctx)
        }
        "query2" => {
            let b = iter_to_stream(ctx, b, |b| Time::from_milliseconds(b.date_time as i128));
            query2(b, ctx)
        }
        "query3" => {
            let a = iter_to_stream(ctx, a, |a| Time::from_milliseconds(a.date_time as i128));
            let p = iter_to_stream(ctx, p, |p| Time::from_milliseconds(p.date_time as i128));
            query3(a, p, ctx)
        }
        "query3-opt" => {
            let a = iter_to_stream(ctx, a, |a| Time::from_milliseconds(a.date_time as i128));
            let p = iter_to_stream(ctx, p, |p| Time::from_milliseconds(p.date_time as i128));
            query3_opt(a, p, ctx)
        }
        "query4" => {
            let a = iter_to_stream(ctx, a, |a| Time::from_milliseconds(a.date_time as i128));
            let p = iter_to_stream(ctx, b, |b| Time::from_milliseconds(b.date_time as i128));
            query4(a, p, ctx)
        }
        _ => panic!("unknown query"),
    });
    eprintln!("{}", time.elapsed().as_millis());
}
