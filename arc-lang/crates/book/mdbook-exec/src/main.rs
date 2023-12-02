use std::process;

use mdbook::preprocess::Preprocessor;
use mdbook_exec::handle_preprocessing;
use mdbook_exec::Exec;

fn main() {
    let preprocessor = Exec::new();

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    if args.next().is_some() {
        if preprocessor.supports_renderer(&args.next().unwrap()) {
            process::exit(0);
        } else {
            process::exit(1);
        }
    } else {
        if let Err(e) = handle_preprocessing(&preprocessor) {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}
