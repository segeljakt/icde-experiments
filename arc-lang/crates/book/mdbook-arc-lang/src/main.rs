use std::process;

use mdbook::preprocess::Preprocessor;
use mdbook_arc_lang::handle_preprocessing;
use mdbook_arc_lang::ArcLang;

fn main() {
    let preprocessor = ArcLang::new();

    let mut args = std::env::args();
    let _ = args.next().unwrap();
    match args.next().as_deref() {
        Some("run") => {
            let path = args.next().unwrap();
            let code = std::fs::read_to_string(&path).unwrap();
            let html = preprocessor.html(&code);
            println!("{}", html);
        }
        Some(_) => {
            if preprocessor.supports_renderer(&args.next().unwrap()) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        None => {
            if let Err(e) = handle_preprocessing(&preprocessor) {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}
