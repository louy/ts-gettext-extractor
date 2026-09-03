// See https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html for details about a POT file format

use itertools::Itertools;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// An individual message in a POT file
#[derive(Debug, PartialEq, Eq, Hash, Default, Ord, PartialOrd)]
pub struct POTMessageID {
    pub msgctx: Option<String>,
    pub msgid: String,
    pub msgid_plural: Option<String>,
}
impl POTMessageID {
    fn convert_to_string(&self) -> String {
        let mut result = String::new();

        if let Some(ctx) = &self.msgctx {
            result.push_str(&format_po_message("msgctxt", ctx));
            result.push('\n');
        }
        result.push_str(&format_po_message("msgid", &self.msgid));
        result.push('\n');

        if let Some(msgid_plural) = &self.msgid_plural {
            result.push_str(&format_po_message("msgid_plural", msgid_plural));
            result.push('\n');
            result.push_str(&format_po_message("msgstr[0]", ""));
            result.push('\n');
            result.push_str(&format_po_message("msgstr[1]", ""));
        } else {
            result.push_str(&format_po_message("msgstr", ""));
        }

        result
    }
}

/// Metadata about a message in a POT file that doesn't affect it's uniqueness
#[derive(Debug)]
pub struct POTMessageMeta {
    pub references: BTreeSet<String>,
    pub translator_comments: BTreeSet<String>,
    pub extracted_comments: BTreeSet<String>,
    pub flags: BTreeSet<String>,
}
impl POTMessageMeta {
    fn new() -> Self {
        Self {
            references: BTreeSet::new(),
            translator_comments: BTreeSet::new(),
            extracted_comments: BTreeSet::new(),
            flags: BTreeSet::new(),
        }
    }

    fn convert_to_string(&self) -> String {
        let mut result = String::new();
        let POTMessageMeta {
            references,
            translator_comments,
            extracted_comments,
            flags,
        } = self;
        {
            for comment in translator_comments {
                result.push_str(&format_po_comment(&' ', comment));
            }
            for comment in extracted_comments {
                result.push_str(&format_po_comment(&'.', comment));
            }
            for reference in references {
                result.push_str(&format!("#: {}\n", reference));
            }
            for flag in flags {
                result.push_str(&format_po_comment(&',', flag));
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct POTFile {
    messages: HashMap<POTMessageID, POTMessageMeta>,
}
impl POTFile {
    pub fn convert_to_string(&self) -> String {
        let mut result = String::new();

        // Add headers
        result.push_str(
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"
"#,
        );

        for message in self.messages.keys().sorted() {
            let meta = self.messages.get(message).unwrap();
            result.push('\n');
            result.push_str(&meta.convert_to_string());
            result.push_str(&message.convert_to_string());
            result.push('\n');
        }
        result
    }
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct POT {
    default_domain: String,
    pub domains: BTreeMap<String, POTFile>,
}
impl POT {
    pub fn new(default_domain: impl Into<Option<String>>) -> Self {
        Self {
            default_domain: default_domain.into().unwrap_or("default".to_string()),
            domains: BTreeMap::new(),
        }
    }

    pub fn add_message(
        &mut self,
        domain: Option<String>,
        message: POTMessageID,
    ) -> &mut POTMessageMeta {
        let file = self
            .domains
            .entry(domain.unwrap_or(self.default_domain.clone()).to_string())
            .or_insert_with(POTFile::new);
        file.messages
            .entry(message)
            .or_insert_with(POTMessageMeta::new)
    }

    #[allow(dead_code)]
    pub fn to_string(&self, domain: Option<&str>) -> Option<String> {
        self.domains
            .get(domain.unwrap_or(&self.default_domain))
            .map(|file| file.convert_to_string())
    }
}

const MAX_LINE_LENGTH: usize = 80;

fn format_po_message(key: &str, msg: &str) -> std::string::String {
    let lines = escape_po_lines(msg);

    // A message without line breaks stays on one line as long as it fits
    // (including the key, quotes & space)
    if let [line] = &lines[..] {
        if line.len() <= MAX_LINE_LENGTH - key.len() - 3 {
            return format!("{} \"{}\"", key, line);
        }
    }

    let mut result = format!("{} \"\"\n", key);
    result.push_str(
        &lines
            .iter()
            .flat_map(|line| wrap_words(line, KeepSpaces::Yes))
            .map(|chunk| format!("\"{}\"", chunk))
            .join("\n"),
    );
    result
}

fn format_po_comment(prefix: &char, msg: &str) -> std::string::String {
    // Comments can't span lines, so every line gets its own comment marker
    let line_prefix = format!("#{} ", prefix);
    let mut result = String::new();
    for line in normalise_line_endings(msg).split('\n') {
        let chunks = wrap_words(line, KeepSpaces::No);
        if chunks.is_empty() {
            result.push_str(&format!("#{}\n", prefix));
        }
        for chunk in chunks {
            result.push_str(&format!("{}{}\n", line_prefix, chunk));
        }
    }
    result
}

/// Escapes a message for a PO file and splits it after each line break, so that
/// each line can be written as its own PO string. Gettext strings are always
/// single-line, with `\n` standing in for the line break.
fn escape_po_lines(msg: &str) -> Vec<String> {
    // Backslashes are escaped first, so that a literal `\n` in the source can't
    // be read back as a line break
    let escaped = normalise_line_endings(msg)
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let lines: Vec<String> = escaped
        .split_inclusive('\n')
        .map(|line| match line.strip_suffix('\n') {
            Some(line) => format!("{}\\n", line),
            None => line.to_string(),
        })
        .collect();
    if lines.is_empty() {
        return vec![String::new()];
    }
    lines
}

fn normalise_line_endings(msg: &str) -> String {
    msg.replace("\r\n", "\n").replace('\r', "\n")
}

/// Whether runs of whitespace within a line are significant. Messages keep them
/// verbatim; comments are free to reflow.
enum KeepSpaces {
    Yes,
    No,
}

/// The number of characters a wrapped chunk can hold, leaving room for the
/// quotes or comment marker around it and the space it is split on.
const MAX_CHUNK_LENGTH: usize = MAX_LINE_LENGTH - 3 - 1;

/// Breaks a single line into chunks short enough to fit on a line, splitting on
/// spaces. When spaces are kept, every chunk but the last ends with the space it
/// was split on, so that joining the chunks back together reproduces the line.
fn wrap_words(line: &str, keep_spaces: KeepSpaces) -> Vec<String> {
    let keep_spaces = matches!(keep_spaces, KeepSpaces::Yes);
    let words: Vec<&str> = if keep_spaces {
        line.split(' ').collect()
    } else {
        line.split_whitespace().collect()
    };
    let mut chunks: Vec<String> = Vec::new();
    let mut chunk = String::new();
    // Tracked separately from the chunk, which can be empty because a run of
    // spaces yields empty words
    let mut chunk_started = false;
    for word in words {
        if chunk_started {
            if (chunk.len() + 1 + word.len()) > MAX_CHUNK_LENGTH {
                chunks.push(if keep_spaces {
                    format!("{} ", chunk)
                } else {
                    chunk
                });
                chunk = String::new();
            } else {
                chunk.push(' ');
            }
        }
        chunk.push_str(word);
        chunk_started = true;
    }
    if chunk_started {
        chunks.push(chunk);
    }
    chunks
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    use pretty_assertions::assert_eq;

    use super::*;

    fn add_message_reference(
        pot: &mut POT,
        domain: Option<String>,
        message: POTMessageID,
        reference: String,
    ) {
        let meta = pot.add_message(domain, message);
        meta.references.insert(reference.to_string());
    }

    #[test]
    fn generates_file_with_singular_message() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgid: "Hello, world!".to_string(),
                ..Default::default()
            },
            "src/main.rs".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs
msgid "Hello, world!"
msgstr ""
"#
        );
    }
    #[test]
    fn generates_file_with_plural_message() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgid: "%d person".to_string(),
                msgid_plural: Some("%d people".to_string()),
                ..Default::default()
            },
            "src/main.rs".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs
msgid "%d person"
msgid_plural "%d people"
msgstr[0] ""
msgstr[1] ""
"#
        );
    }

    #[test]
    fn generates_file_with_context_messages() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("menu".to_string()),
                msgid: "File".to_string(),
                ..Default::default()
            },
            "src/main.rs".to_string(),
        );
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("menu".to_string()),
                msgid: "%d file".to_string(),
                msgid_plural: Some("%d files".to_string()),
            },
            "src/main.rs".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs
msgctxt "menu"
msgid "%d file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""

#: src/main.rs
msgctxt "menu"
msgid "File"
msgstr ""
"#
        );
    }

    #[test]
    fn it_breaks_long_ids_into_multiple_lines() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID
            {
                msgid: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),..Default::default()
            },
            "src/main.rs".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs
msgid ""
"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod "
"tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, "
"quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo "
"consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse "
"cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat "
"non proident, sunt in culpa qui officia deserunt mollit anim id est laborum."
msgstr ""
"#
        );
    }

    #[test]
    fn it_keeps_line_breaks_in_comments() {
        let mut pot = POT::new(None);
        let meta = pot.add_message(
            None,
            POTMessageID {
                msgid: "Hi friend".to_string(),
                ..Default::default()
            },
        );
        meta.extracted_comments.insert(String::from(
            r#"
This is a not so long comment.
However, it has a line break in it.
Lines that are too long to fit on a single comment line are still broken up into several lines.
"#,
        ));
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#.
#. This is a not so long comment.
#. However, it has a line break in it.
#. Lines that are too long to fit on a single comment line are still broken up
#. into several lines.
#.
msgid "Hi friend"
msgstr ""
"#
        );
    }

    #[test]
    fn it_doesnt_break_on_very_long_reference_filename() {
        let mut pot = POT::new(None);
        let meta = pot.add_message(
            None,
            POTMessageID {
                msgid: "Hi friend".to_string(),
                ..Default::default()
            },
        );
        meta.references.insert(
            "path/to/very/long/filename/that/shouldnt/be/broken/here/we/go/really/this/time/my_super_special_file_v3_FINAL_FINAL_NO_EDIT.tsx:246912631923213"
                .to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: path/to/very/long/filename/that/shouldnt/be/broken/here/we/go/really/this/time/my_super_special_file_v3_FINAL_FINAL_NO_EDIT.tsx:246912631923213
msgid "Hi friend"
msgstr ""
"#
        );
    }

    #[test]
    fn it_handles_special_whitespaces_correctly() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID {
                msgid: r#"A string with a new line
should keep the line break"#
                    .to_string(),
                ..Default::default()
            },
        );
        pot.add_message(
            None,
            POTMessageID {
                msgid: "A string double  whitespace".to_string(),
                ..Default::default()
            },
        );
        pot.add_message(
            None,
            POTMessageID {
                msgid: "Special\u{a0}space".to_string(),
                ..Default::default()
            },
        );

        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid "A string double  whitespace"
msgstr ""

msgid ""
"A string with a new line\n"
"should keep the line break"
msgstr ""

msgid "Special space"
msgstr ""
"#
        );
    }

    #[test]
    fn it_keeps_line_breaks_in_context_and_ids() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("A context\nover two lines".to_string()),
                msgid: "%d line\nbreak".to_string(),
                msgid_plural: Some("%d line\nbreaks".to_string()),
            },
            "src/main.rs".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs
msgctxt ""
"A context\n"
"over two lines"
msgid ""
"%d line\n"
"break"
msgid_plural ""
"%d line\n"
"breaks"
msgstr[0] ""
msgstr[1] ""
"#
        );
    }

    #[test]
    fn it_keeps_a_message_ending_in_a_line_break_on_one_line() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID {
                msgid: "A trailing line break\n".to_string(),
                ..Default::default()
            },
        );
        pot.add_message(
            None,
            POTMessageID {
                msgid: "Blank lines\n\nare kept\n".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid "A trailing line break\n"
msgstr ""

msgid ""
"Blank lines\n"
"\n"
"are kept\n"
msgstr ""
"#
        );
    }

    #[test]
    fn it_breaks_long_lines_of_a_multiline_message() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID {
                msgid: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.\nUt enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid ""
"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod "
"tempor incididunt ut labore et dolore magna aliqua.\n"
"Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut "
"aliquip ex ea commodo consequat."
msgstr ""
"#
        );
    }

    #[test]
    fn it_escapes_backslashes_so_they_arent_read_as_line_breaks() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID {
                msgid: r"A literal \n is not a line break".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

msgid "A literal \\n is not a line break"
msgstr ""
"#
        );
    }

    #[test]
    fn handles_duplicate_messages() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("Ctxt".to_string().clone()),
                msgid: "Hello, world!".to_string(),
                ..Default::default()
            },
            "src/main.rs:1".to_string(),
        );
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("Ctxt".to_string().clone()),
                msgid: "Hello, world!".to_string().clone(),
                ..Default::default()
            },
            "src/main.rs:2".to_string(),
        );
        add_message_reference(
            &mut pot,
            None,
            POTMessageID {
                msgctx: Some("Ctxt".to_string().clone()),
                msgid: "Hello, world!".to_string().clone(),
                ..Default::default()
            },
            "src/main.rs:3".to_string(),
        );
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: src/main.rs:1
#: src/main.rs:2
#: src/main.rs:3
msgctxt "Ctxt"
msgid "Hello, world!"
msgstr ""
"#
        );
    }
    #[test]
    fn it_has_correct_equality_check() {
        assert_eq!(
            POTMessageID {
                msgid: "Hello, world!".to_string(),
                ..Default::default()
            },
            POTMessageID {
                msgid: "Hello, world!".to_string(),
                ..Default::default()
            }
        );
        assert_eq!(
            POTMessageID {
                msgid: "1 file".to_string().clone(),
                msgid_plural: Some("%d files".to_string().clone()),
                ..Default::default()
            },
            POTMessageID {
                msgid: "1 file".to_string().clone(),
                msgid_plural: Some("%d files".to_string().clone()),
                ..Default::default()
            }
        );
        assert_eq!(
            POTMessageID {
                msgctx: Some("ctxt".to_string().clone()),
                msgid: "Hello, world!".to_string().clone(),
                ..Default::default()
            },
            POTMessageID {
                msgctx: Some("ctxt".to_string().clone()),
                msgid: "Hello, world!".to_string().clone(),
                ..Default::default()
            }
        );
        assert_eq!(
            POTMessageID {
                msgctx: Some("ctxt".to_string().clone()),
                msgid: "1 file".to_string().clone(),
                msgid_plural: Some("%d files".to_string().clone())
            },
            POTMessageID {
                msgctx: Some("ctxt".to_string().clone()),
                msgid: "1 file".to_string().clone(),
                msgid_plural: Some("%d files".to_string().clone())
            }
        );
    }
}
