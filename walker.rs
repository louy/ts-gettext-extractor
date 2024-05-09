use std::path::PathBuf;
use walkdir::WalkDir;

pub fn find_ts_files(
    path: PathBuf,
    exclude: Vec<String>,
) -> Result<impl Iterator<Item = walkdir::DirEntry>, walkdir::Error> {
    Ok(WalkDir::new(&path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .metadata()
                .ok()
                .map(|metadata| metadata.is_file())
                .unwrap_or(false)
        })
        .filter(move |entry| {
            entry
                .path()
                .to_str()
                .map(|path| {
                    // Remove any excluded paths
                    if exclude.iter().any(|exclude| path.contains(exclude)) {
                        false
                    } else {
                        // Filter out all files with extensions other than `ts` or `tsx` or `js` or `jsx`
                        if entry.path().extension().map_or(false, |ext| {
                            ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx"
                        }) {
                            true
                        } else {
                            false
                        }
                    }
                })
                .unwrap_or(false)
        }))
}

extern crate swc_ecma_parser;
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
