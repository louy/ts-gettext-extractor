//! Support for `.astro` single file components.
//!
//! An astro file is TypeScript frontmatter fenced by `---`, followed by HTML
//! markup in which JavaScript only ever appears inside `{...}` expressions.
//! Neither half parses as TSX on its own, so the file is rewritten before it
//! reaches the visitor: the frontmatter is kept verbatim, every template
//! expression is kept as a block statement, and everything else is replaced
//! with spaces. Newlines are always preserved, so line numbers in the rewritten
//! source still match the original file.

use swc_common::BytePos;
use swc_ecma_ast::{EsVersion, Stmt};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};

/// The syntax the rewritten source has to be parsed with.
pub fn syntax() -> Syntax {
    Syntax::Typescript(TsConfig {
        tsx: true,
        dts: false,
        decorators: true,
        ..Default::default()
    })
}

/// Rewrite an astro source file into TSX of the same shape.
pub fn transform(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let template = write_frontmatter(source, &mut out);
    write_template(source, template, &mut out);
    out
}

/// Write the frontmatter, returning the offset the template starts at.
fn write_frontmatter(source: &str, out: &mut String) -> usize {
    // Only whitespace may precede the opening fence
    let open = match source.find("---") {
        Some(open) if source[..open].trim().is_empty() => open,
        _ => return 0,
    };
    let fence_end = line_end(source, open + 3);
    if !source[open + 3..fence_end].trim().is_empty() {
        return 0;
    }
    let (close, close_end) = match find_closing_fence(source, fence_end) {
        Some(fence) => fence,
        // Without a closing fence there is no frontmatter, only markup
        None => return 0,
    };

    blank(&source[..fence_end], out);
    out.push_str(&source[fence_end..close]);
    blank(&source[close..close_end], out);
    close_end
}

/// Find the closing `---` fence, as `(line start, line end)`.
fn find_closing_fence(source: &str, from: usize) -> Option<(usize, usize)> {
    let mut pos = from;
    while pos < source.len() {
        let start = pos + 1; // Skip the newline the previous line ended with
        let end = line_end(source, start);
        if source[start..end].trim() == "---" {
            return Some((start, end));
        }
        pos = end;
    }
    None
}

fn write_template(source: &str, from: usize, out: &mut String) {
    let bytes = source.as_bytes();
    let mut i = from;
    while i < source.len() {
        if bytes[i] == b'<' {
            if source[i..].starts_with("<!--") {
                let end = match source[i..].find("-->") {
                    Some(offset) => i + offset + 3,
                    None => source.len(),
                };
                blank(&source[i..end], out);
                i = end;
            } else if let Some(tag) = raw_text_tag(&source[i..]) {
                i = write_raw_text_element(source, i, tag, out);
            } else {
                i = write_tag(source, i, out);
            }
        } else if bytes[i] == b'{' {
            i = write_expression(source, i, out);
        } else {
            i = blank_char(source, i, out);
        }
    }
}

/// `script` and `style` hold text rather than markup, so their contents can't be
/// scanned for expressions. Style contents are dropped; script contents are kept
/// inside a block, which both parses as TSX and keeps their scope separate.
fn write_raw_text_element(source: &str, start: usize, tag: &str, out: &mut String) -> usize {
    let open_end = write_tag(source, start, out);
    let (close, close_end) = find_close_tag(source, open_end, tag);
    let keep = tag == "script"
        && holds_javascript(&source[start..open_end])
        && source[..open_end].ends_with('>')
        && close < source.len();

    if keep {
        // Turn the `>` that was just written into the opening brace of a block
        out.pop();
        out.push('{');
        out.push_str(&source[open_end..close]);
        out.push('}');
        blank(&source[close + 1..close_end], out);
    } else {
        blank(&source[open_end..close_end], out);
    }
    close_end
}

/// Whether a `script` tag holds JavaScript that can be extracted from.
fn holds_javascript(tag: &str) -> bool {
    let tag = tag.to_ascii_lowercase();
    if tag.contains("is:raw") {
        return false;
    }
    match tag
        .split_whitespace()
        .find_map(|attribute| attribute.strip_prefix("type="))
    {
        Some(kind) => ["module", "javascript", "typescript"]
            .iter()
            .any(|js| kind.contains(js)),
        None => true,
    }
}

/// Blank out a tag, keeping the expressions used in its attributes.
fn write_tag(source: &str, start: usize, out: &mut String) -> usize {
    let bytes = source.as_bytes();
    let mut i = start;
    let mut quote: Option<u8> = None;
    while i < source.len() {
        match bytes[i] {
            byte if quote == Some(byte) => {
                quote = None;
                i = blank_char(source, i, out);
            }
            _ if quote.is_some() => i = blank_char(source, i, out),
            b'"' | b'\'' => {
                quote = Some(bytes[i]);
                i = blank_char(source, i, out);
            }
            b'{' => i = write_expression(source, i, out),
            b'>' => {
                out.push(' ');
                return i + 1;
            }
            _ => i = blank_char(source, i, out),
        }
    }
    i
}

/// Keep a `{...}` expression verbatim, as a block statement.
fn write_expression(source: &str, start: usize, out: &mut String) -> usize {
    let end = match expression_end(source, start) {
        Some(end) => end,
        // Not an expression after all, e.g. a stray brace in text
        None => return blank_char(source, start, out),
    };

    let expression = &source[start..end];
    match spread_offset(expression) {
        // `{...props}` is only an expression once the spread is dropped
        Some(offset) => {
            out.push_str(&expression[..offset]);
            out.push_str("   ");
            out.push_str(&expression[offset + 3..]);
        }
        None => out.push_str(expression),
    }
    end
}

/// The offset of the `...` of a spread expression, if it is one.
fn spread_offset(expression: &str) -> Option<usize> {
    let offset = expression[1..]
        .find(|c: char| !c.is_whitespace())
        .map(|offset| offset + 1)?;
    if expression[offset..].starts_with("...") {
        Some(offset)
    } else {
        None
    }
}

/// Find the `}` matching the `{` at `start`, using the parser itself so that
/// strings, comments and nested markup are all accounted for.
fn expression_end(source: &str, start: usize) -> Option<usize> {
    let tail = &source[start..];
    let lexer = Lexer::new(
        syntax(),
        EsVersion::EsNext,
        StringInput::new(tail, BytePos(1), BytePos(tail.len() as u32 + 1)),
        None,
    );
    if let Ok(Stmt::Block(block)) = Parser::new_from(lexer).parse_stmt(false) {
        let end = block.span.hi.0 as usize - 1;
        if tail[..end].ends_with('}') {
            return Some(start + end);
        }
    }
    // The parser rejects some valid template expressions, spreads among them
    match_braces(tail).map(|end| start + end)
}

/// Find the `}` matching the leading `{` by counting braces.
fn match_braces(source: &str) -> Option<usize> {
    /// Nesting that changes what a brace means
    enum Scope {
        /// Code, holding the number of braces left open
        Code(usize),
        Quoted(u8),
        Template,
    }

    let bytes = source.as_bytes();
    let mut scopes = vec![Scope::Code(0)];
    let mut i = 0;
    while i < bytes.len() {
        match scopes.last_mut()? {
            Scope::Code(depth) => match bytes[i] {
                b'{' => *depth += 1,
                b'}' => {
                    if *depth == 0 {
                        // Closes the interpolation this code is nested in
                        scopes.pop();
                    } else {
                        *depth -= 1;
                        if *depth == 0 && scopes.len() == 1 {
                            return Some(i + 1);
                        }
                    }
                }
                b'"' | b'\'' => scopes.push(Scope::Quoted(bytes[i])),
                b'`' => scopes.push(Scope::Template),
                b'/' if bytes.get(i + 1) == Some(&b'/') => {
                    i = line_end(source, i);
                    continue;
                }
                b'/' if bytes.get(i + 1) == Some(&b'*') => i += source[i..].find("*/")? + 2,
                _ => {}
            },
            Scope::Quoted(quote) => match bytes[i] {
                b'\\' => i += 1,
                // An unterminated string means the braces can't be trusted
                b'\n' => return None,
                byte if byte == *quote => {
                    scopes.pop();
                }
                _ => {}
            },
            Scope::Template => match bytes[i] {
                b'\\' => i += 1,
                b'`' => {
                    scopes.pop();
                }
                b'$' if bytes.get(i + 1) == Some(&b'{') => {
                    scopes.push(Scope::Code(0));
                    i += 1;
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

/// The name of the raw text element a tag opens, if it opens one.
fn raw_text_tag(source: &str) -> Option<&'static str> {
    let bytes = source.as_bytes();
    ["script", "style"].into_iter().find(|tag| {
        bytes.len() > tag.len() + 1
            && bytes[1..tag.len() + 1].eq_ignore_ascii_case(tag.as_bytes())
            && matches!(
                bytes[tag.len() + 1],
                b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r'
            )
    })
}

/// Find the tag closing a raw text element, as `(tag start, tag end)`.
fn find_close_tag(source: &str, from: usize, tag: &str) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut i = from;
    while let Some(offset) = source[i..].find('<') {
        let start = i + offset;
        let name = start + 2;
        if bytes.get(start + 1) == Some(&b'/')
            && bytes.len() >= name + tag.len()
            && bytes[name..name + tag.len()].eq_ignore_ascii_case(tag.as_bytes())
        {
            let end = match source[start..].find('>') {
                Some(offset) => start + offset + 1,
                None => source.len(),
            };
            return (start, end);
        }
        i = start + 1;
    }
    (source.len(), source.len())
}

/// Replace everything but newlines with spaces, so lines still line up.
fn blank(source: &str, out: &mut String) {
    out.extend(source.chars().map(|c| match c {
        '\n' | '\r' => c,
        _ => ' ',
    }));
}

fn blank_char(source: &str, at: usize, out: &mut String) -> usize {
    let c = source[at..]
        .chars()
        .next()
        .expect("offset inside the source");
    blank(&source[at..at + c.len_utf8()], out);
    at + c.len_utf8()
}

/// The offset of the newline ending the line `from` is on, or the end of source.
fn line_end(source: &str, from: usize) -> usize {
    match source[from..].find('\n') {
        Some(offset) => from + offset,
        None => source.len(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn keeps_frontmatter_and_template_expressions() {
        assert_eq!(
            transform("---\nconst a = __('Hi');\n---\n<p>{a}</p>\n"),
            "   \nconst a = __('Hi');\n   \n   {a}    \n"
        );
    }

    #[test]
    fn treats_a_file_without_frontmatter_as_markup() {
        assert_eq!(transform("<p>{__('Hi')}</p>\n"), "   {__('Hi')}    \n");
    }

    #[test]
    fn ignores_braces_in_quoted_attributes() {
        assert_eq!(
            transform("<p title=\"a { b\">x</p>\n"),
            format!("{}\n", " ".repeat(22))
        );
    }

    #[test]
    fn ignores_braces_in_markup_comments() {
        assert_eq!(transform("<!-- { a -->\n"), format!("{}\n", " ".repeat(12)));
    }

    #[test]
    fn drops_style_contents_and_scopes_script_contents() {
        let transformed =
            transform("<style>\n  h1 { color: red }\n</style>\n<script>\n  __('Hi');\n</script>\n");
        assert_eq!(
            transformed,
            "       \n                   \n        \n       {\n  __('Hi');\n}        \n"
        );
        assert_parses(&transformed);
    }

    #[test]
    fn drops_contents_of_scripts_that_arent_javascript() {
        let transformed = transform("<script type=\"application/json\">\n{\"a\": 1}\n</script>\n");
        assert_eq!(
            transformed,
            format!("{}\n{}\n{}\n", " ".repeat(32), " ".repeat(8), " ".repeat(9))
        );
        assert_parses(&transformed);
    }

    #[test]
    fn keeps_spread_attributes() {
        let transformed = transform("<Card {...props} title={__('Hi')} />\n");
        assert_eq!(transformed, "      {   props}       {__('Hi')}   \n");
        assert_parses(&transformed);
    }

    #[test]
    fn keeps_markup_nested_in_expressions() {
        let transformed =
            transform("<ul>{items.map((i) => <li>{__('It\u{2019}s here')}</li>)}</ul>\n");
        assert_eq!(
            transformed,
            "    {items.map((i) => <li>{__('It\u{2019}s here')}</li>)}     \n"
        );
        assert_parses(&transformed);
    }

    #[test]
    fn preserves_line_numbers() {
        let source = "---\n\nconst a = 1;\n---\n\n<p>\n  {__('Hi')}\n</p>\n";
        let transformed = transform(source);
        assert_eq!(source.lines().count(), transformed.lines().count());
        assert_eq!(transformed.lines().nth(6), Some("  {__('Hi')}"));
    }

    #[test]
    fn rewrites_the_example_into_parseable_tsx() {
        let source = std::fs::read_to_string("./tests/src/pages/[locale]/example.astro")
            .expect("the astro example");
        assert_parses(&transform(&source));
    }

    fn assert_parses(source: &str) {
        let lexer = Lexer::new(
            syntax(),
            EsVersion::EsNext,
            StringInput::new(source, BytePos(1), BytePos(source.len() as u32 + 1)),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module();
        let errors = parser.take_errors();
        assert!(module.is_ok(), "failed to parse: {:?}", module.err());
        assert!(errors.is_empty(), "failed to parse: {:?}", errors);
    }
}
