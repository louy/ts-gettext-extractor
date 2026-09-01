use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn find_ts_files(
    path: PathBuf,
    exclude: Vec<String>,
) -> Result<impl Iterator<Item = walkdir::DirEntry>, walkdir::Error> {
    Ok(WalkDir::new(path)
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
                        entry.path().extension().map_or(false, |ext| {
                            ext == "ts"
                                || ext == "tsx"
                                || ext == "js"
                                || ext == "jsx"
                                || ext == "astro"
                        })
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
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::VisitWith;

/// Extract gettext strings from a source file
pub fn parse_file(path: &Path, pot: Arc<Mutex<crate::pot::POT>>, references_relative_to: &PathBuf) {
    // Astro files aren't valid TSX, so they are rewritten before being parsed
    let (syntax, rewritten) = match path.extension() {
        Some(os_str) => match os_str.to_str() {
            Some("d.ts") => (
                Syntax::Typescript(TsConfig {
                    tsx: false,
                    dts: true,
                    decorators: true,
                    ..Default::default()
                }),
                None,
            ),
            Some("ts") => (
                Syntax::Typescript(TsConfig {
                    tsx: false,
                    dts: false,
                    decorators: true,
                    ..Default::default()
                }),
                None,
            ),
            Some("tsx") => (
                Syntax::Typescript(TsConfig {
                    tsx: true,
                    dts: false,
                    decorators: true,
                    ..Default::default()
                }),
                None,
            ),
            Some("js") => (Syntax::Es(Default::default()), None),
            Some("jsx") => (
                Syntax::Es(EsConfig {
                    jsx: true,
                    ..Default::default()
                }),
                None,
            ),
            Some("astro") => {
                let source = std::fs::read_to_string(path).expect("Failed to load file");
                (
                    crate::astro::syntax(),
                    Some(crate::astro::transform(&source)),
                )
            }
            _ => panic!("Unknown extension"),
        },
        _ => panic!("Unknown extension"),
    };

    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let comments: swc_common::comments::SingleThreadedComments = Default::default();

    let fm = match rewritten {
        Some(source) => cm.new_source_file(FileName::Real(path.to_path_buf()), source),
        None => cm.load_file(path).expect("Failed to load file"),
    };
    let lexer = Lexer::new(
        syntax,
        // EsVersion defaults to es5
        swc_ecma_ast::EsVersion::EsNext,
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
        pot,
        cm: Lrc::clone(&cm),
        comments: Some(&comments),
        references_relative_to,
    };

    module.visit_with(&mut visitor);
}
