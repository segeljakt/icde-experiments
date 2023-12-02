use colors::*;
use highlighter::SyntaxHighlighter;
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::CmdPreprocessor;
use mdbook::preprocess::Preprocessor;
use mdbook::preprocess::PreprocessorContext;
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

pub struct ArcLang;

impl ArcLang {
    pub fn new() -> ArcLang {
        ArcLang
    }

    pub fn html(&self, code: &str) -> String {
        let highlighter = highlighter::SyntaxHighlighter::new();
        Self::_html(&highlighter, &code)
    }

    fn _html(highlighter: &SyntaxHighlighter, code: &str) -> String {
        let s = highlighter
            .regex
            .replace_all(code, |caps: &regex::Captures| {
                if let Some(s) = caps.name("keyword") {
                    format!(r#"<span class="keyword">{}</span>"#, s.as_str())
                } else if let Some(s) = caps.name("numeric") {
                    format!(r#"<span class="numeric">{}</span>"#, s.as_str())
                } else if let Some(s) = caps.name("string") {
                    format!(r#"<span class="string">{}</span>"#, s.as_str())
                } else if let Some(s) = caps.name("builtin") {
                    format!(r#"<span class="builtin">{}</span>"#, s.as_str())
                } else if let Some(s) = caps.name("comment") {
                    format!(r#"<span class="comment">{}</span>"#, s.as_str())
                } else {
                    unreachable!()
                }
            });
        indoc::formatdoc! {"
             <pre><code>
             {}
             </code></pre>
             <style>
                .keyword {{ color: {keyword}; font-weight:bold }}
                .numeric {{ color: {numeric}; }}
                .string {{ color: {string}; }}
                .builtin {{ color: {builtin}; }}
                .comment {{ color: {comment}; font-style=italic }}
             </style>",
            s.trim(),
            keyword = html(KEYWORD_COLOR),
            numeric = html(NUMERIC_COLOR),
            string = html(STRING_COLOR),
            builtin = html(BUILTIN_COLOR),
            comment = html(COMMENT_COLOR)
        }
    }
}

impl Preprocessor for ArcLang {
    fn name(&self) -> &str {
        "arc-lang-preprocessor"
    }

    fn run(&self, _: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let grammar_regex = regex::Regex::new(r"(?s)```arc-lang\n(.*?)```").unwrap();
        let highlighter = highlighter::SyntaxHighlighter::new();
        book.for_each_mut(|item| {
            if let mdbook::BookItem::Chapter(ch) = item {
                ch.content = grammar_regex
                    .replace_all(&ch.content, |caps: &regex::Captures<'_>| {
                        let s = caps.get(1).unwrap().as_str();
                        Self::_html(&highlighter, s)
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
