use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum POTMessageID {
    Singular(
        /// Context
        Option<String>,
        /// msgid
        String,
    ),
    Plural(
        /// Context
        Option<String>,
        /// msgid
        String,
        /// msgid_plural
        String,
    ),
}

#[derive()]
pub struct POTFile {
    messages: IndexMap<POTMessageID, IndexSet<String>>,
}
impl POTFile {
    pub fn new() -> Self {
        Self {
            messages: IndexMap::new(),
        }
    }
}

#[derive()]
pub struct POT {
    pub default_domain: String,
    domains: IndexMap<String, POTFile>,
}
impl POT {
    pub fn new(default_domain: impl Into<Option<String>>) -> Self {
        Self {
            default_domain: default_domain.into().unwrap_or("default".to_string()),
            domains: IndexMap::new(),
        }
    }

    pub fn add_message(&mut self, domain: Option<&str>, message: POTMessageID, reference: &str) {
        let file = self
            .domains
            .entry(domain.unwrap_or(&self.default_domain).to_string())
            .or_insert_with(POTFile::new);
        file.messages
            .entry(message)
            .or_insert_with(IndexSet::new)
            .insert(reference.to_string());
    }

    pub fn to_string(&self, domain: Option<&str>) -> String {
        let mut result = String::new();
        if let Some(file) = self.domains.get(domain.unwrap_or(&self.default_domain)) {
            for (message, references) in &file.messages {
                for reference in references {
                    result.push_str(&format!("#: {}\n", reference));
                }
                match message {
                    POTMessageID::Singular(None, msg) => {
                        result.push_str(&format!("msgid {}\nmsgstr \"\"\n\n", format_message(msg)));
                    }
                    POTMessageID::Plural(None, msg1, msg2) => {
                        result.push_str(&format!(
                            "msgid {}\nmsgid_plural {}\nmsgstr[0] \"\"\nmsgstr[1] \"\"\n\n",
                            format_message(msg1),
                            format_message(msg2)
                        ));
                    }
                    POTMessageID::Singular(Some(ctx), msg) => {
                        result.push_str(&format!(
                            "msgctxt \"{}\"\nmsgid {}\nmsgstr \"\"\n\n",
                            ctx,
                            format_message(msg)
                        ));
                    }
                    POTMessageID::Plural(Some(ctx), msg1, msg2) => {
                        result.push_str(&format!(
                            "msgctxt \"{}\"\nmsgid {}\nmsgid_plural {}\nmsgstr[0] \"\"\nmsgstr[1] \"\"\n\n", 
                            ctx, 
                            format_message(msg1),
                            format_message(msg2)
                        ));
                    }
                }
            }
        }
        result
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

    #[test]
    fn generates_file_with_singular_message() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Hello, world!".to_string()),
            "src/main.rs",
        );
        assert_eq!(
            pot.to_string(None),
            r#"#: src/main.rs
msgid "Hello, world!"
msgstr ""

"#
        );
    }
    #[test]
    fn generates_file_with_plural_message() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID::Plural(None, "%d person".to_string(), "%d people".to_string()),
            "src/main.rs",
        );
        assert_eq!(
            pot.to_string(None),
            r#"#: src/main.rs
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
        pot.add_message(
            None,
            POTMessageID::Singular(Some("menu".to_string()), "File".to_string()),
            "src/main.rs",
        );
        pot.add_message(
            None,
            POTMessageID::Plural(
                Some("menu".to_string()),
                "%d file".to_string(),
                "%d files".to_string(),
            ),
            "src/main.rs",
        );
        assert_eq!(
            pot.to_string(None),
            r#"#: src/main.rs
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
    fn it_deduplicates_references() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Hello, world!".to_string()),
            "src/main.rs:1",
        );
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Hello, world!".to_string()),
            "src/main.rs:10",
        );
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Hello, world!".to_string()),
            "src/main.rs:10",
        );
        assert_eq!(
            pot.to_string(None),
            r#"#: src/main.rs:1
#: src/main.rs:10
msgid "Hello, world!"
msgstr ""

"#
        );
    }

    #[test]
    fn it_breaks_long_ids_into_multiple_lines() {
        let mut pot = POT::new(None);
        pot.add_message(
            None,
            POTMessageID::Singular(None, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string()),
            "src/main.rs",
        );
        assert_eq!(
            pot.to_string(None),
            r#"#: src/main.rs
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
