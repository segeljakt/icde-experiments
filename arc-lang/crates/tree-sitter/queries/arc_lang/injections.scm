(stmt_code lang: "rust" code: (code) @injection.content (#set! injection.language "rust"))
(stmt_code lang: "python" code: (code) @injection.content (#set! injection.language "python"))
((line_comment) @injection.content
 (#lua-match? @injection.content "^#.")
 (#set! injection.language "markdown")
 (#offset! @injection.content 0 1 0 0))
