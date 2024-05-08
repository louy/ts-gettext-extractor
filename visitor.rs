use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use swc_common::sync::Lrc;
use swc_common::{SourceMap, Span};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit};

use crate::pot::POTMessageID;

pub struct GettextVisitor {
    pub pot: Arc<Mutex<crate::pot::POT>>,
    pub cm: Lrc<SourceMap>,
}
impl Visit for GettextVisitor {
    noop_visit_type!();

    fn visit_call_expr(&mut self, call: &CallExpr) {
        // let call = call.visit_children_with(self);
        println!("{:?}", call);

        match &call {
            CallExpr {
                callee: Callee::Expr(expr),
                args,
                ..
            } => match &expr.deref() {
                Expr::Ident(Ident {
                    span,
                    sym,
                    optional: _,
                }) => match sym.as_str() {
                    "__" | "gettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..1] {
                            [ExprOrSpread { expr, .. }] => match &expr.deref() {
                                Expr::Lit(Lit::Str(Str { value, .. })) => {
                                    self.pot.lock().unwrap().add_message(
                                        None,
                                        POTMessageID::Singular(None, value.to_string()),
                                        format_reference(&self.cm, span),
                                    );
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    "__n" | "ngettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..2] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                                match (&expr1.deref(), &expr2.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            None,
                                            POTMessageID::Plural(
                                                None,
                                                value1.to_string(),
                                                value2.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__p" | "pgettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..2] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                                match (&expr1.deref(), &expr2.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            None,
                                            POTMessageID::Singular(
                                                Some(value1.to_string()),
                                                value2.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__np" | "npgettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..3] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                                match (&expr1.deref(), &expr2.deref(), &expr3.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value3, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            None,
                                            POTMessageID::Plural(
                                                Some(value1.to_string()),
                                                value2.to_string(),
                                                value3.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__d" | "dgettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..2] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] => {
                                match (&expr1.deref(), &expr2.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            Some(value1.to_string()),
                                            POTMessageID::Singular(None, value2.to_string()),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__dn" | "dngettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..3] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                                match (&expr1.deref(), &expr2.deref(), &expr3.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value3, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            Some(value1.to_string()),
                                            POTMessageID::Plural(
                                                None,
                                                value2.to_string(),
                                                value3.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__dp" | "dpgettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..3] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }] => {
                                match (&expr1.deref(), &expr2.deref(), &expr3.deref()) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value3, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            Some(value1.to_string()),
                                            POTMessageID::Singular(
                                                Some(value2.to_string()),
                                                value3.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    "__dnp" | "dnpgettext" => {
                        println!("Found call to: {:?}", sym);
                        match &args[..4] {
                            [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }, ExprOrSpread { expr: expr3, .. }, ExprOrSpread { expr: expr4, .. }] => {
                                match (
                                    &expr1.deref(),
                                    &expr2.deref(),
                                    &expr3.deref(),
                                    &expr4.deref(),
                                ) {
                                    (
                                        Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value3, .. })),
                                        Expr::Lit(Lit::Str(Str { value: value4, .. })),
                                    ) => {
                                        self.pot.lock().unwrap().add_message(
                                            Some(value1.to_string()),
                                            POTMessageID::Plural(
                                                Some(value2.to_string()),
                                                value3.to_string(),
                                                value4.to_string(),
                                            ),
                                            format_reference(&self.cm, span),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
    }
}

fn format_reference(cm: &Lrc<SourceMap>, span: &Span) -> String {
    let loc = cm.lookup_char_pos(span.lo);
    format!("{}:{}", &loc.file.name.to_string(), loc.line.to_string())
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
            r#"#: test.js:1
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
            r#"#: test.js:1
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
            r#"#: test.js:1
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
            r#"#: test.js:1
msgctxt "menu"
msgid "1 file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""

"#
        );
    }

    use swc_ecma_visit::VisitWith;

    fn parse(filename: &str, source: &str, pot: Arc<Mutex<crate::pot::POT>>) {
        let cm: Lrc<SourceMap> = Default::default();
        let mut visitor = GettextVisitor {
            pot,
            cm: Lrc::clone(&cm),
        };
        let fm = cm.new_source_file(FileName::Custom(filename.into()), source.into());
        let lexer = Lexer::new(
            Default::default(),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();
        module.visit_with(&mut visitor);
    }
}
