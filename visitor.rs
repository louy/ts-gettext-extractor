use std::{ops::Deref, sync::Arc};

use swc_ecma_ast::*;
use swc_ecma_visit::{noop_fold_type, Fold, FoldWith};

pub struct GettextVisitor<'a> {
    pub pot: &'a mut Arc<crate::pot::POT>,
}
impl Fold for GettextVisitor<'_> {
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
        let mut pot = Arc::new(crate::pot::POT::new());
        parse("test.js", r#"__("Hello, world!");"#, &mut pot);
        assert_eq!(
            pot.to_string("default"),
            r#"#: src/main.js
msgid "Hello, world!"
msgstr ""

"#
        );
    }

    fn parse(filename: &str, source: &str, pot: &mut Arc<crate::pot::POT>) {
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
