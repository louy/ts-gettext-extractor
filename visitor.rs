use std::{
    borrow::BorrowMut,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

use crate::pot::POTMessageID;

pub struct GettextVisitor {
    pub pot: Arc<Mutex<crate::pot::POT>>,
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
            } => {
                match &expr.deref() {
                    Expr::Ident(Ident {
                        span,
                        sym,
                        optional,
                    }) => match sym.as_str() {
                        "__" | "gettext" => {
                            println!("Found call to: {:?}", sym);
                            match &args[..1] {
                                [ExprOrSpread { expr, .. }] => match &expr.deref() {
                                    Expr::Lit(Lit::Str(Str { value, .. })) => {
                                        self.pot.lock().unwrap().add_message(
                                            None,
                                            POTMessageID::Singular(value.to_string()),
                                            "src/main.js", // FIXME - Loc
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
                                [ExprOrSpread { expr: expr1, .. }, ExprOrSpread { expr: expr2, .. }] =>
                                {
                                    match (&expr1.deref(), &expr2.deref()) {
                                        (
                                            Expr::Lit(Lit::Str(Str { value: value1, .. })),
                                            Expr::Lit(Lit::Str(Str { value: value2, .. })),
                                        ) => {
                                            self.pot.lock().unwrap().add_message(
                                                None,
                                                POTMessageID::Plural(
                                                    value1.to_string(),
                                                    value2.to_string(),
                                                ),
                                                "src/main.js", // FIXME - Loc
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
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use swc_common::SourceMap;
    use swc_common::{sync::Lrc, FileName};
    use swc_ecma_parser::{lexer::Lexer, Parser, StringInput};

    use super::*;

    #[test]
    fn detects_singular_message_with_no_context() {
        let pot = Arc::new(Mutex::new(crate::pot::POT::new(None)));
        parse("test.js", r#"__("Hello, world!");"#, Arc::clone(&pot));
        assert_eq!(
            pot.lock().unwrap().to_string(None),
            r#"#: test.js
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
            pot.lock().unwrap().to_string(None),
            r#"#: test.js
msgid "1 file"
msgid_plural "%d files"
msgstr[0] ""
msgstr[1] ""

"#
        );
    }

    fn parse(filename: &str, source: &str, pot: Arc<Mutex<crate::pot::POT>>) {
        let mut visitor = GettextVisitor { pot };
        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Custom(filename.into()), source.into());
        let lexer = Lexer::new(
            Default::default(),
            // EsVersion defaults to es5
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();
        module.visit_with(&mut visitor);
    }
}
