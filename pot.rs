// See https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html for details about a POT file format

use std::collections::BTreeMap;

/// An individual message in a POT file
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum POTMessageID {
    Singular(
        /// msgctx
        Option<String>,
        /// msgid
        String,
    ),
    Plural(
        /// msgctx
        Option<String>,
        /// msgid
        String,
        /// msgid_plural
        String,
    ),
}
impl PartialOrd for POTMessageID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for POTMessageID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare msgctx
        if let (
            POTMessageID::Singular(Some(ctx1), _) | POTMessageID::Plural(Some(ctx1), _, _),
            POTMessageID::Singular(Some(ctx2), _) | POTMessageID::Plural(Some(ctx2), _, _),
        ) = (self, other)
        {
            match ctx1.cmp(ctx2) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            };
        }
        if let POTMessageID::Singular(Some(_), _) | POTMessageID::Plural(Some(_), _, _) = self {
            return std::cmp::Ordering::Less;
        }
        if let POTMessageID::Singular(Some(_), _) | POTMessageID::Plural(Some(_), _, _) = other {
            return std::cmp::Ordering::Greater;
        }

        // Compare msgid
        if let (
            POTMessageID::Singular(None, msgid1) | POTMessageID::Plural(None, msgid1, _),
            POTMessageID::Singular(None, msgid2) | POTMessageID::Plural(None, msgid2, _),
        ) = (self, other)
        {
            match msgid1.cmp(msgid2) {
                std::cmp::Ordering::Equal => {}
                other => return other,
            };
        }

        // Compare msgid_plural
        if let (POTMessageID::Plural(None, _, _), POTMessageID::Singular(None, _)) = (self, other) {
            return std::cmp::Ordering::Greater;
        }
        if let (POTMessageID::Singular(None, _), POTMessageID::Plural(None, _, _)) = (self, other) {
            return std::cmp::Ordering::Less;
        }

        if let (POTMessageID::Plural(None, _, msgid1), POTMessageID::Plural(None, _, msgid2)) =
            (self, other)
        {
            return msgid1.cmp(msgid2);
        }

        std::cmp::Ordering::Equal
    }
}
impl POTMessageID {
    fn convert_to_string(&self) -> String {
        let mut result = String::new();
        match self {
            POTMessageID::Singular(None, msg) => {
                result.push_str(&format_po_message("msgid", msg));
                result.push('\n');
                result.push_str(&format_po_message("msgstr", ""));
            }
            POTMessageID::Plural(None, msg1, msg2) => {
                result.push_str(&format_po_message("msgid", msg1));
                result.push('\n');
                result.push_str(&format_po_message("msgid_plural", msg2));
                result.push('\n');
                result.push_str(&format_po_message("msgstr[0]", ""));
                result.push('\n');
                result.push_str(&format_po_message("msgstr[1]", ""));
            }
            POTMessageID::Singular(Some(ctx), msg) => {
                result.push_str(&format_po_message("msgctxt", ctx));
                result.push('\n');
                result.push_str(&format_po_message("msgid", msg));
                result.push('\n');
                result.push_str(&format_po_message("msgstr", ""));
            }
            POTMessageID::Plural(Some(ctx), msg1, msg2) => {
                result.push_str(&format_po_message("msgctxt", ctx));
                result.push('\n');
                result.push_str(&format_po_message("msgid", msg1));
                result.push('\n');
                result.push_str(&format_po_message("msgid_plural", msg2));
                result.push('\n');
                result.push_str(&format_po_message("msgstr[0]", ""));
                result.push('\n');
                result.push_str(&format_po_message("msgstr[1]", ""));
            }
        }
        result
    }
}

/// Metadata about a message in a POT file that doesn't affect it's uniqueness
#[derive(Debug)]
pub struct POTMessageMeta {
    pub references: Vec<String>,
    pub translator_comments: Vec<String>,
    pub extracted_comments: Vec<String>,
    pub flags: Vec<String>,
}
impl POTMessageMeta {
    fn new() -> Self {
        Self {
            references: Vec::new(),
            translator_comments: Vec::new(),
            extracted_comments: Vec::new(),
            flags: Vec::new(),
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
    messages: BTreeMap<POTMessageID, POTMessageMeta>,
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

        for (message, meta) in &self.messages {
            result.push('\n');
            result.push_str(&meta.convert_to_string());
            result.push_str(&message.convert_to_string());
            result.push('\n');
        }
        result
    }
    pub fn new() -> Self {
        Self {
            messages: BTreeMap::new(),
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
        result.push_str(&format!("\"{}\"", line.trim()));
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
        meta.references.push(reference.to_string());
    }

    #[test]
    fn generates_file_with_singular_message() {
        let mut pot = POT::new(None);
        add_message_reference(
            &mut pot,
            None,
            POTMessageID::Singular(None, "Hello, world!".to_string()),
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
            POTMessageID::Plural(None, "%d person".to_string(), "%d people".to_string()),
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
            POTMessageID::Singular(Some("menu".to_string()), "File".to_string()),
            "src/main.rs".to_string(),
        );
        add_message_reference(
            &mut pot,
            None,
            POTMessageID::Plural(
                Some("menu".to_string()),
                "%d file".to_string(),
                "%d files".to_string(),
            ),
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
            POTMessageID::Singular(None, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string()),
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
        let meta = pot.add_message(None, POTMessageID::Singular(None, "Hi friend".to_string()));
        meta.extracted_comments.push(String::from(
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
        let meta = pot.add_message(None, POTMessageID::Singular(None, "Hi friend".to_string()));
        meta.references.push(
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
            POTMessageID::Singular(
                None,
                r#"A string with a new line
should be replaced with a space"#
                    .to_string(),
            ),
        );
        pot.add_message(
            None,
            POTMessageID::Singular(None, "A string double  whitespace".to_string()),
        );
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Special\u{a0}space".to_string()),
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
}
