use runtime::prelude::*;

use runtime::prelude::formats::csv;

#[data]
struct Auction {
    id: u64,
    item_name: String,
    description: String,
    initial_bid: u64,
    reserve: u64,
    date_time: u64,
    expires: u64,
    seller: u64,
    category: u64,
    extra: String,
}

#[data]
struct Person {
    id: u64,
    name: String,
    email_address: String,
    credit_card: String,
    city: String,
    state: String,
    date_time: u64,
    extra: String,
}

#[data]
struct Bid {
    auction: u64,
    bidder: u64,
    price: u64,
    channel: String,
    url: String,
    date_time: u64,
    extra: String,
}

#[data]
struct PrunedAuction {
    id: u64,
    seller: u64,
}

#[data]
struct PrunedPerson {
    id: u64,
    name: String,
    city: String,
    state: String,
}

#[data]
struct Query1 {
    auction: u64,
    price: u64,
    bidder: u64,
    date_time: u64,
}

#[data]
struct Query3 {
    name: String,
    city: String,
    state: String,
    id: u64,
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
            (person.state == "OR" || person.state == "ID" || person.state == "CA")
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
        if p.state == "or" || p.state == "id" || p.state == "ca" {
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

pub fn iter<T: Data>(path: &str) -> impl Iterator<Item = T> {
    let file = std::fs::File::open(path).expect("Unable to open file");
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .expect("Unable to map file")
    };
    mmap.advise(memmap2::Advice::Sequential).unwrap();
    let mut reader = csv::de::Reader::<1024>::new(',');
    let mut i = 0;
    std::iter::from_fn(move || {
        if i >= mmap.len() {
            return None;
        }
        let mut deserializer = csv::de::Deserializer::new(&mut reader, &mmap[i..]);
        let bid = T::deserialize(&mut deserializer).unwrap();
        i += deserializer.nread;
        Some(bid)
    })
}

pub fn stream<T: Data>(
    ctx: &mut Context,
    iter: impl Iterator<Item = T> + Send + 'static,
    f: impl Fn(&T) -> Time + Send + 'static,
) -> Stream<T> {
    Stream::from_iter(ctx, iter, f, 1000, Duration::from_milliseconds(100))
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("No `dir` specified");
    let query = args.next().expect("No `query` specified");

    match query.as_str() {
        "q1" => {
            let bids_iter = iter::<Bid>(&format!("{dir}/bids.csv"));
            let time = std::time::Instant::now();
            CurrentThreadRunner::run(move |ctx| {
                let bids = stream(ctx, bids_iter, |b| Time::from_millis(b.date_time as i128));
                query1(bids, ctx)
            });
            eprintln!("{}", time.elapsed().as_millis());
        }
        "q1-io" => {
            let bids_iter = iter::<Bid>(&format!("{dir}/bids.csv"));
            let time = std::time::Instant::now();
            CurrentThreadRunner::run(move |ctx| {
                let bids = stream(ctx, bids_iter, |b| Time::from_millis(b.date_time as i128));
                bids.drain(ctx);
            });
            eprintln!("{}", time.elapsed().as_millis());
        }
        "q3" => {
            let auctions_iter = iter::<Auction>(&format!("{dir}/auctions.csv"));
            let persons_iter = iter::<Person>(&format!("{dir}/persons.csv"));
            let time = std::time::Instant::now();
            CurrentThreadRunner::run(move |ctx| {
                let persons = stream(ctx, persons_iter, |p| {
                    Time::from_millis(p.date_time as i128)
                });
                let auctions = stream(ctx, auctions_iter, |a| {
                    Time::from_millis(a.date_time as i128)
                });
                q3(auctions, persons, ctx)
            });
            eprintln!("{}", time.elapsed().as_millis());
        }
        "q3-opt" => {
            let auctions_iter = iter::<Auction>(&format!("{dir}/auctions.csv"));
            let persons_iter = iter::<Person>(&format!("{dir}/persons.csv"));
            let time = std::time::Instant::now();
            CurrentThreadRunner::run(move |ctx| {
                let persons = stream(ctx, persons_iter, |p| {
                    Time::from_millis(p.date_time as i128)
                });
                let auctions = stream(ctx, auctions_iter, |a| {
                    Time::from_millis(a.date_time as i128)
                });
                q3_opt(auctions, persons, ctx)
            });
            eprintln!("{}", time.elapsed().as_millis());
        }
        "q3-io" => {
            let auctions_iter = iter::<Auction>(&format!("{dir}/auctions.csv"));
            let persons_iter = iter::<Person>(&format!("{dir}/persons.csv"));
            let time = std::time::Instant::now();
            CurrentThreadRunner::run(move |ctx| {
                let persons = stream(ctx, persons_iter, |p| {
                    Time::from_millis(p.date_time as i128)
                });
                let auctions = stream(ctx, auctions_iter, |a| {
                    Time::from_millis(a.date_time as i128)
                });
                persons.drain(ctx);
                auctions.drain(ctx);
            });
            eprintln!("{}", time.elapsed().as_millis());
        }
        _ => panic!("unknown query"),
    }
}
