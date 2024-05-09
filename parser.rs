extern crate swc_ecma_parser;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::VisitWith;

/// Extract gettext strings from a source file
pub fn parse_file(path: &PathBuf, pot: Arc<Mutex<crate::pot::POT>>) {
    let syntax = match path.extension() {
        Some(os_str) => match os_str.to_str() {
            Some("d.ts") => Syntax::Typescript(TsConfig {
                tsx: false,
                dts: true,
                ..Default::default()
            }),
            Some("ts") => Syntax::Typescript(TsConfig {
                tsx: false,
                dts: false,
                ..Default::default()
            }),
            Some("tsx") => Syntax::Typescript(TsConfig {
                tsx: true,
                dts: false,
                ..Default::default()
            }),
            Some("js") => Syntax::Es(Default::default()),
            Some("jsx") => Syntax::Es(EsConfig {
                jsx: true,
                ..Default::default()
            }),
            _ => panic!("Unknown extension"),
        },
        _ => panic!("Unknown extension"),
    };

    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let comments: swc_common::comments::SingleThreadedComments = Default::default();

    let fm = cm.load_file(path).expect("Failed to load file");
    let lexer = Lexer::new(
        syntax,
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );
    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    let mut visitor = crate::visitor::GettextVisitor {
        pot: pot,
        cm: Lrc::clone(&cm),
        comments: Some(&comments),
    };

    module.visit_with(&mut visitor);
}
