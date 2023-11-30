use runtime::prelude::*;

use runtime::prelude::formats::csv;
use runtime::prelude::Data;

#[data]
struct Auction {
    id: usize,
    item_name: String,
    description: String,
    initial_bid: usize,
    reserve: usize,
    date_time: u64,
    expires: u64,
    seller: usize,
    category: usize,
    extra: String,
}

#[data]
struct Person {
    id: usize,
    name: String,
    email_address: String,
    credit_card: String,
    city: String,
    state: String,
    date_time: u64,
    extra: String,
}

#[data]
struct PrunedAuction {
    id: usize,
    seller: usize,
}

#[data]
struct PrunedPerson {
    id: usize,
    name: String,
    city: String,
    state: String,
}

#[data]
struct Query3 {
    name: String,
    city: String,
    state: String,
    id: usize,
}

fn q3(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
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

fn q3_opt(auctions: Stream<Auction>, persons: Stream<Person>, ctx: &mut Context) {
    let persons2 = persons.filter_map(ctx, |p| {
        if p.state == String::from("OR")
            || p.state == String::from("ID")
            || p.state == String::from("CA")
        {
            Option::some(PrunedPerson {
                id: p.id,
                name: p.name,
                city: p.city,
                state: p.state,
            })
        } else {
            Option::none()
        }
    });
    let auctions2 = auctions.filter_map(ctx, |a| {
        if a.category == 10 {
            Option::some(PrunedAuction {
                id: a.id,
                seller: a.seller,
            })
        } else {
            Option::none()
        }
    });
    auctions2
        .window_join(
            ctx,
            persons2,
            |a| a.seller,
            |p| p.id,
            Duration::from_seconds(10),
            |a, p| Query3 {
                name: p.name,
                city: p.city,
                state: p.state,
                id: a.id,
            },
        )
        .drain(ctx);
}

pub fn mmap_csv_source<T: Data>(ctx: &mut Context, path: &str, f: fn(T) -> Time) -> Stream<T> {
    let file = std::fs::File::open(path).expect("Unable to open file");
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .expect("Unable to map file")
    };
    let mut reader = csv::de::Reader::<1024>::new(',');
    let mut i = 0;
    let iter = std::iter::from_fn(move || {
        if i >= mmap.len() {
            return None;
        }
        let mut deserializer = csv::de::Deserializer::new(&mut reader, &mmap[i..]);
        let bid = T::deserialize(&mut deserializer).unwrap();
        i += deserializer.nread;
        Some(bid)
    });
    Stream::from_iter(ctx, iter, f, 1000, Duration::from_milliseconds(100))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("No `dir` specified");
    let query = args.next().expect("No `query` specified");

    let auctions = |ctx: &mut Context| {
        mmap_csv_source::<Auction>(ctx, &format!("{dir}/auctions.csv"), |a| {
            Time::from_milliseconds(a.date_time as i128)
        })
    };
    let persons = |ctx: &mut Context| {
        mmap_csv_source::<Person>(ctx, &format!("{dir}/persons.csv"), |p| {
            Time::from_milliseconds(p.date_time as i128)
        })
    };

    let time = std::time::Instant::now();
    CurrentThreadRunner::run(move |ctx| match query.as_str() {
        "q3" => q3(auctions(ctx), persons(ctx), ctx),
        "q3-opt" => q3_opt(auctions(ctx), persons(ctx), ctx),
        _ => panic!("unknown query"),
    });
    eprintln!("{}", time.elapsed().as_millis());
}
