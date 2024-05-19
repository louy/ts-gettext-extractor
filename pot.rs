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
    // If line will exceed max length (including quotes & space)
    let msg_escaped = msg
        .replace('"', "\\\"")
        .replace("\r\n", " ")
        .replace(['\r', '\n'], " ");
    if msg_escaped.len() > MAX_LINE_LENGTH - key.len() - 3 {
        let mut result = String::new();
        result.push_str(&format!("{} \"\"\n", key));
        let mut line = String::new();
        for word in msg_escaped.split(' ') {
            // minus 3 for the quotes and trailing space
            if (line.len() + word.len() + 1) > (MAX_LINE_LENGTH - 3) {
                result.push_str(&format!("\"{}\"\n", line));
                line = String::new();
            }
            line.push_str(&format!("{} ", word));
        }
        if !line.is_empty() {
            result.push_str(&format!("\"{}\"", &line[..line.len() - 1]));
        }
        result
    } else {
        format!("{} \"{}\"", key, msg_escaped)
    }
}

fn format_po_comment(prefix: &char, msg: &str) -> std::string::String {
    // If line will exceed max length (including prefix, hash and space)
    let line_prefix = format!("#{} ", prefix);
    if msg.len() > MAX_LINE_LENGTH - line_prefix.len() {
        let mut result = String::new();
        let mut line = String::new();
        line.push_str(&line_prefix);
        for word in msg.split_whitespace() {
            if (line.len() + word.len() + 1) > MAX_LINE_LENGTH {
                result.push_str(line.trim());
                result.push('\n');
                line = String::new();
                line.push_str(&line_prefix);
            }
            line.push_str(&format!("{} ", word));
        }
        result.push_str(&line);
        result.push('\n');
        result
    } else {
        format!("{}{}\n", line_prefix, msg)
    }
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
    fn it_doesnt_break_on_multiline_comment() {
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
This might tip your tool off.
"#,
        ));
        assert_eq!(
            pot.to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#. This is a not so long comment. However, it has a line break in it. This
#. might tip your tool off. 
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
should be replaced with a space"#
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

msgid "A string with a new line should be replaced with a space"
msgstr ""

msgid "Special space"
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
