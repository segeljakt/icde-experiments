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

pub struct Grammar;

impl Grammar {
    pub fn new() -> Grammar {
        Grammar
    }
}

impl Preprocessor for Grammar {
    fn name(&self) -> &str {
        "grammar-preprocessor"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let grammar_regex = regex::Regex::new(r"(?s)```grammar\n(.*?)```").unwrap();
        let head_regex = regex::Regex::new(r"([A-Z][A-Za-z]*)( ::=.*)").unwrap();
        let keyword_regex = regex::Regex::new(r#""([^ ]+?)""#).unwrap();
        let keyword_subst = r"<b>${1}</b>";
        let nonterm_regex = regex::Regex::new(r"\[([A-Z][A-Za-z]+)\]").unwrap();
        let nonterm_subst = r#"<a href="${1}.html#${1}">${1}</a>"#;
        let subterm_regex = regex::Regex::new(r"\[\[([A-Z][A-Za-z]+)\]\]").unwrap();
        let comment_regex = regex::Regex::new(r"( *(?:\||::=).*)#[^{](.*)").unwrap();
        let comment_subst = r#"$1<i style="color:gray">${2}</i>"#;
        book.for_each_mut(|item| {
            if let mdbook::BookItem::Chapter(ch) = item {
                let path = ch.path.as_ref().unwrap();
                let name = path.file_stem().unwrap().to_str().unwrap();
                ch.content = grammar_regex
                    .replace_all(&ch.content, |caps: &regex::Captures<'_>| {
                        let subterm_subst = format!(r#"<a href="{}.html#${{1}}">${{1}}</a>"#, name);
                        let head_subst =
                            format!(r#"<a id="${{1}}" href="{}.html#${{1}}">${{1}}</a>$2"#, name);
                        let s = caps.get(1).unwrap().as_str();
                        let s = keyword_regex.replace_all(&s, keyword_subst);
                        let s = comment_regex.replace_all(&s, comment_subst);
                        let s = subterm_regex.replace_all(&s, subterm_subst);
                        let s = nonterm_regex.replace_all(&s, nonterm_subst);
                        let s = head_regex.replace_all(&s, head_subst);
                        let s = s.trim();
                        format!("<pre><code>{}</code></pre>", s)
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
