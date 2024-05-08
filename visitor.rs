use std::{
    borrow::BorrowMut,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use swc_ecma_ast::*;
use swc_ecma_visit::{noop_fold_type, Fold, FoldWith};

use crate::pot::POTMessageID;

pub struct GettextVisitor {
    pub pot: Arc<Mutex<crate::pot::POT>>,
}
impl Fold for GettextVisitor {
    noop_fold_type!();

    fn fold_call_expr(&mut self, call: CallExpr) -> CallExpr {
        let call = call.fold_children_with(self);
        println!("{:?}", call);

        match &call {
            CallExpr {
                callee: Callee::Expr(expr),
                args,
                ..
            } => {
                println!("{:?}", args);
                match &expr.deref() {
                    Expr::Ident(Ident {
                        span,
                        sym,
                        optional,
                    }) => match sym.as_str() {
                        "__" => {
                            println!("Found call to: {:?}", sym);
                            match args.first() {
                                Some(ExprOrSpread { expr, .. }) => match &expr.deref() {
                                    Expr::Lit(Lit::Str(Str { value, .. })) => {
                                        println!("Found string: {:?}", value);
                                        self.pot.lock().unwrap().add_message(
                                            None,
                                            POTMessageID::Singular(value.to_string()),
                                            "src/main.js",
                                        );
                                    }
                                    _ => {}
                                },
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

        call
    }

    fn fold_fn_expr(&mut self, f: FnExpr) -> FnExpr {
        let f = f.fold_children_with(self);

        if let Some(id) = &f.ident {
            println!("Found function: {}", id.sym);
        }

        f
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
            r#"#: src/main.js
msgid "Hello, world!"
msgstr ""

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
        module.fold_with(&mut visitor);
    }
}
