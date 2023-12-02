use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor, PreprocessorContext};
use std::io;

pub fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

pub struct Exec;

impl Exec {
    pub fn new() -> Exec {
        Exec
    }
}

impl Preprocessor for Exec {
    fn name(&self) -> &str {
        "exec-preprocessor"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let exec_regex = regex::Regex::new(r#"\{\{\s*#exec\s*(.*?)\}\}"#).unwrap();
        book.for_each_mut(|item| {
            if let mdbook::BookItem::Chapter(ch) = item {
                ch.content = exec_regex
                    .replace_all(&ch.content, |caps: &regex::Captures<'_>| {
                        let s = caps.get(1).unwrap().as_str();
                        let s = std::process::Command::new("/bin/sh")
                            .arg("-c")
                            .arg(s)
                            .output()
                            .unwrap()
                            .stdout;
                        std::str::from_utf8(s.as_ref()).unwrap().to_string()
                    })
                    .into_owned();
            }
        });
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}
