use std::error::Error;
use std::process;

fn example() -> Result<(), Box<dyn Error>> {
    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_path("../data/bids.csv").unwrap();
    for result in rdr.records() {
        // The iterator yields Result<StringRecord, Error>, so we check the error here.
        let _ = result?;
    }
    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
