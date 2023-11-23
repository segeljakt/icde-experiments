use rust_barebones::Bid;
use rust_barebones::Deserializer;
use rust_barebones::Reader;

use std::fs::File;

use memmap2::Mmap;
use memmap2::MmapOptions;
use rayon::prelude::ParallelIterator;
use rayon::str::ParallelString;
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;

fn main() {
    let mut args = std::env::args().skip(1);
    let time = std::time::Instant::now();

    match args.next().as_deref() {
        Some("basic") => basic(),
        Some("mmap") => mmap(),
        Some("mmap_deser") => mmap_deser(),
        Some("mmap_rayon") => mmap_rayon(),
        Some("mmap_tokio") => mmap_tokio(),
        _ => panic!("Invalid argument. Expected: basic, mmap, mmap_deser, mmap_rayon, mmap_tokio"),
    }

    let duration = time.elapsed();
    println!("Job Execution Time: {:?}", duration);
}

fn basic() {
    let time = std::time::Instant::now();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let f = tokio::fs::File::open("../data/bids.csv").await.unwrap();
            let mut input = tokio::io::BufReader::new(f);
            let mut buf = Vec::with_capacity(1024 * 30);
            let mut reader = Reader::<1024>::new(',');
            loop {
                match input.read_until(b'\n', &mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut deserializer = Deserializer::new(&mut reader, &buf[0..n]);
                        if let Ok(_) = Bid::deserialize(&mut deserializer) {
                            // tx.send(data).await.unwrap();
                        }
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        });
    println!("Job Execution Time: {:?}", time.elapsed());
}

fn mmap() {
    let time = std::time::Instant::now();
    let file = File::open("../data/bids.csv").expect("Unable to open file");
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).expect("Unable to map file") };
    let mut reader = Reader::<1024>::new(',');
    let mut n = 0;
    while n < mmap.len() {
        let mut deserializer = Deserializer::new(&mut reader, &mmap[n..]);
        if let Ok(_) = Bid::deserialize(&mut deserializer) {}
        n += deserializer.nread;
    }
    println!("Job Execution Time: {:?}", time.elapsed());
}

fn mmap_deser() {
    let time = std::time::Instant::now();
    let file = File::open("../data/bids.csv").expect("Unable to open file");
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).expect("Unable to map file") };
    let mut rdr = csv::Reader::from_reader(mmap.as_ref());
    for result in rdr.deserialize::<Bid>() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        if let Ok(bid) = result {
            println!("{:?}", bid);
        }
    }
    println!("Job Execution Time: {:?}", time.elapsed());
}

fn mmap_rayon() {
    let time = std::time::Instant::now();
    let file = File::open("../data/bids.csv").expect("Unable to open file");
    let mmap: Mmap = unsafe { MmapOptions::new().map(&file).expect("Unable to map file") };
    unsafe {
        std::str::from_utf8_unchecked(&mmap)
            .par_lines()
            .map_with(Reader::<1024>::new(','), |reader, line| {
                let mut deserializer = Deserializer::new(reader, &line.as_bytes());
                if let Ok(_) = Bid::deserialize(&mut deserializer) {}
            })
            .for_each(|_| {});
    };
    println!("Job Execution Time: {:?}", time.elapsed());
}

fn mmap_tokio() {
    let time = std::time::Instant::now();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let file = File::open("../data/bids.csv").expect("Unable to open file");
            let mmap: Mmap = unsafe { MmapOptions::new().map(&file).expect("Unable to map file") };
            unsafe {
                std::str::from_utf8_unchecked(&mmap)
                    .par_lines()
                    .map_with(Reader::<1024>::new(','), |reader, line| {
                        let mut deserializer = Deserializer::new(reader, &line.as_bytes());
                        if let Ok(_) = Bid::deserialize(&mut deserializer) {}
                    })
                    .for_each(|_| {});
            };
        });
    println!("Job Execution Time: {:?}", time.elapsed());
}
