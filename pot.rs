use indexmap::IndexMap;
// See https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html for details about a POT file format

/// An individual message in a POT file
#[derive(Debug, Eq, Hash, PartialEq)]
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
impl POTMessageID {
    fn convert_to_string(&self) -> String {
        let mut result = String::new();
        match self {
            POTMessageID::Singular(None, msg) => {
                result.push_str(&format!("msgid {}\nmsgstr \"\"", format_message(msg)));
            }
            POTMessageID::Plural(None, msg1, msg2) => {
                result.push_str(&format!(
                    "msgid {}\nmsgid_plural {}\nmsgstr[0] \"\"\nmsgstr[1] \"\"",
                    format_message(msg1),
                    format_message(msg2)
                ));
            }
            POTMessageID::Singular(Some(ctx), msg) => {
                result.push_str(&format!(
                    "msgctxt \"{}\"\nmsgid {}\nmsgstr \"\"",
                    ctx,
                    format_message(msg)
                ));
            }
            POTMessageID::Plural(Some(ctx), msg1, msg2) => {
                result.push_str(&format!(
                    "msgctxt \"{}\"\nmsgid {}\nmsgid_plural {}\nmsgstr[0] \"\"\nmsgstr[1] \"\"",
                    ctx,
                    format_message(msg1),
                    format_message(msg2)
                ));
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
                result.push_str(&format!("#  {}\n", comment));
            }
            for comment in extracted_comments {
                result.push_str(&format!("#. {}\n", comment));
            }
            for reference in references {
                result.push_str(&format!("#: {}\n", reference));
            }
            for flag in flags {
                result.push_str(&format!("#, {}\n", flag));
            }
        }
        result
    }
}

#[derive(Debug)]
pub struct POTFile {
    messages: IndexMap<POTMessageID, POTMessageMeta>,
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
            messages: IndexMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct POT {
    default_domain: String,
    pub domains: IndexMap<String, POTFile>,
}
impl POT {
    pub fn new(default_domain: impl Into<Option<String>>) -> Self {
        Self {
            default_domain: default_domain.into().unwrap_or("default".to_string()),
            domains: IndexMap::new(),
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

fn format_message(msg: &str) -> std::string::String {
    if msg.len() > 80 {
        let mut result = String::new();
        result.push_str("\"\"\n");
        let mut line = String::new();
        for word in msg.split_whitespace() {
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
        format!("\"{}\"", msg)
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
msgid "File"
msgstr ""

#: src/main.rs
msgctxt "menu"
msgid "%d file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""
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
}
