use std::{
    ops::Deref,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use swc_common::{
    comments::{Comment, Comments},
    sync::Lrc,
};
use swc_common::{SourceMap, Span};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

use crate::pot::{POTMessageID, POTMessageMeta};

pub struct GettextVisitor<'a> {
    pub pot: Arc<Mutex<crate::pot::POT>>,
    pub cm: Lrc<SourceMap>,
    pub comments: Option<&'a dyn Comments>,
    pub references_relative_to: &'a PathBuf,
}
impl GettextVisitor<'_> {
    fn add_message_meta(&self, span: &Span, meta: &mut POTMessageMeta) {
        meta.references.push(format_reference(
            &self.cm,
            span,
            self.references_relative_to,
        ));

        let mut comments = Vec::<Comment>::new();

        match self.comments.get_leading(span.lo) {
            Some(leading) => comments.extend(leading),
            None => {}
        }
        match self.comments.get_trailing(span.hi) {
            Some(trailing) => comments.extend(trailing),
            None => {}
        }

        for comment in comments {
            meta.extracted_comments
                .push(comment.text.trim().to_string());
        }
    }
}
impl Visit for GettextVisitor<'_> {
    noop_visit_type!();

    fn visit_call_expr(&mut self, call: &CallExpr) {
        match &call {
            CallExpr {
                callee: Callee::Expr(expr),
                args,
                span,
                ..
            } => match &expr.deref() {
                Expr::Ident(Ident { sym, .. }) => match sym.as_str() {
                    "__" | "gettext" => match &args[..1] {
                        [ExprOrSpread { expr: expr1, .. }] => {
                            match (&extract_string_from_expr(expr1),) {
                                (Some(value1),) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        None,
                                        POTMessageID::Singular(None, value1.to_string()),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__n" | "ngettext" => match &args[..2] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                            ) {
                                (Some(value1), Some(value2)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        None,
                                        POTMessageID::Plural(
                                            None,
                                            value1.to_string(),
                                            value2.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__p" | "pgettext" => match &args[..2] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                            ) {
                                (Some(value1), Some(value2)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        None,
                                        POTMessageID::Singular(
                                            Some(value1.to_string()),
                                            value2.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__np" | "npgettext" => match &args[..3] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                                &extract_string_from_expr(expr3),
                            ) {
                                (Some(value1), Some(value2), Some(value3)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        None,
                                        POTMessageID::Plural(
                                            Some(value1.to_string()),
                                            value2.to_string(),
                                            value3.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__d" | "dgettext" => match &args[..2] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                            ) {
                                (Some(value1), Some(value2)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        Some(value1.to_string()),
                                        POTMessageID::Singular(None, value2.to_string()),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__dn" | "dngettext" => match &args[..3] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                                &extract_string_from_expr(expr3),
                            ) {
                                (Some(value1), Some(value2), Some(value3)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        Some(value1.to_string()),
                                        POTMessageID::Plural(
                                            None,
                                            value2.to_string(),
                                            value3.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__dp" | "dpgettext" => match &args[..3] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                                &extract_string_from_expr(expr3),
                            ) {
                                (Some(value1), Some(value2), Some(value3)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        Some(value1.to_string()),
                                        POTMessageID::Singular(
                                            Some(value2.to_string()),
                                            value3.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    "__dnp" | "dnpgettext" => match &args[..4] {
                        [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }, ExprOrSpread { expr: expr4, .. }] => {
                            match (
                                &extract_string_from_expr(expr1),
                                &extract_string_from_expr(expr2),
                                &extract_string_from_expr(expr3),
                                &extract_string_from_expr(expr4),
                            ) {
                                (Some(value1), Some(value2), Some(value3), Some(value4)) => {
                                    let pot = &mut self.pot.lock().unwrap();
                                    let meta = pot.add_message(
                                        Some(value1.to_string()),
                                        POTMessageID::Plural(
                                            Some(value2.to_string()),
                                            value3.to_string(),
                                            value4.to_string(),
                                        ),
                                    );
                                    self.add_message_meta(span, meta)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        call.visit_children_with(self);
    }
}

fn format_reference(cm: &Lrc<SourceMap>, span: &Span, references_relative_to: &PathBuf) -> String {
    let loc = cm.lookup_char_pos(span.lo);
    let file = match pathdiff::diff_paths(loc.file.name.to_string(), references_relative_to) {
        Some(relative) => relative.as_os_str().to_str().unwrap().to_string(),
        None => loc.file.name.to_string(),
    };

    format!("{}:{}", file, loc.line.to_string())
}

fn extract_string_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(Lit::Str(Str { value, .. })) => Some(value.to_string()),
        Expr::Tpl(Tpl { quasis, .. }) => match &quasis[..] {
            [TplElement { cooked, .. }] => cooked.as_ref().map(|s| s.to_string()),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use swc_common::FileName;
    use swc_ecma_parser::{lexer::Lexer, Parser, StringInput};

    use super::*;

    #[test]
    fn detects_singular_message_with_no_context() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse("test.js", r#"__("Hello, world!");"#, Arc::clone(&pot));
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: test.js:1
msgid "Hello, world!"
msgstr ""
"#
        );
    }

    #[test]
    fn detects_plural_message_with_no_context() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse("test.js", r#"__n("1 file", "%d files");"#, Arc::clone(&pot));
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: test.js:1
msgid "1 file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""
"#
        );
    }

    #[test]
    fn detects_singular_message_with_context() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse(
            "test.js",
            r#"__p("menu", "Hello, world!");"#,
            Arc::clone(&pot),
        );
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: test.js:1
msgctxt "menu"
msgid "Hello, world!"
msgstr ""
"#
        );
    }

    #[test]
    fn detects_plural_message_with_context() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse(
            "test.js",
            r#"__np("menu", "1 file", "%d files");"#,
            Arc::clone(&pot),
        );
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#: test.js:1
msgctxt "menu"
msgid "1 file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""
"#
        );
    }

    #[test]
    fn detects_message_with_comment() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse(
            "test.js",
            r#"/* Test comment */ __("Test message");"#,
            Arc::clone(&pot),
        );
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#. Test comment
#: test.js:1
msgid "Test message"
msgstr ""
"#
        );
    }

    // Comments seem to behave differently in tsx, so this is a regression test
    #[test]
    fn detects_message_with_comment_in_jsx() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse(
            "file.tsx",
            r#"
<App>{
    /* Test comment */ __n("1 object", "%d objects", 3)
}</App>;"#,
            Arc::clone(&pot),
        );
        assert_eq!(
            pot.lock().unwrap().to_string(None).unwrap(),
            r#"msgid ""
msgstr ""
"Content-Type: text/plain; charset=utf-8\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

#. Test comment
#: file.tsx:3
msgid "1 object"
msgid_plural "%d objects"
msgstr[0] ""
msgstr[1] ""
"#
        );
    }

    use swc_ecma_visit::VisitWith;

    fn parse(filename: &str, source: &str, pot: Arc<Mutex<crate::pot::POT>>) {
        let cm: Lrc<SourceMap> = Default::default();
        let comments: swc_common::comments::SingleThreadedComments = Default::default();
        let mut visitor = GettextVisitor {
            pot,
            cm: Lrc::clone(&cm),
            comments: Some(&comments),
            references_relative_to: &PathBuf::from("."),
        };
        let fm = cm.new_source_file(FileName::Custom(filename.into()), source.into());
        let lexer = Lexer::new(
            swc_ecma_parser::Syntax::Typescript(swc_ecma_parser::TsConfig {
                tsx: true,
                decorators: true,
                dts: false,
                no_early_errors: true,
                disallow_ambiguous_jsx_like: false,
            }),
            Default::default(),
            StringInput::from(&*fm),
            Some(&comments),
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();
        module.visit_with(&mut visitor);
    }
}
