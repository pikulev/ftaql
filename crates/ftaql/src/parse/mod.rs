use std::cell::Cell;
#[cfg(feature = "project-analysis")]
use std::path::PathBuf;
use crate::structs::{ExportInfo, ImportInfo};
use swc_common::comments::Comment;
use swc_common::sync::Lrc;
use swc_common::{comments::Comments, input::SourceFileInput, BytePos, SourceMap};
use swc_ecma_ast::{EsVersion, Module};
#[cfg(feature = "project-analysis")]
use log::debug;
#[cfg(feature = "project-analysis")]
use rspack_resolver::Resolver;
#[cfg(feature = "project-analysis")]
use crate::structs::ExportKind;
#[cfg(feature = "project-analysis")]
use swc_ecma_ast::{ExportDecl, ExportDefaultDecl, ExportDefaultExpr, ImportSpecifier};
use swc_ecma_parser::{error::Error, lexer::Lexer, Parser, Syntax, TsConfig};
#[cfg(feature = "project-analysis")]
use swc_ecma_visit::{Visit, VisitWith};
#[cfg(feature = "project-analysis")]
use tokio::runtime::Runtime;

mod tests;

#[cfg(feature = "project-analysis")]
#[derive(Clone)]
struct AstVisitor<'a> {
    imports: Vec<ImportInfo>,
    exports: Vec<ExportInfo>,
    source_file_dir: PathBuf,
    resolver: &'a Resolver,
    runtime: &'a Runtime,
    project_root: &'a str,
}

#[cfg(feature = "project-analysis")]
impl<'a> AstVisitor<'a> {
    fn resolve_dependency(&self, specifier: &str) -> Option<String> {
        if specifier.starts_with("http://") || specifier.starts_with("https://") {
            return None;
        }

        debug!("Resolving dependency: {}", specifier);
        let result = self
            .runtime
            .block_on(self.resolver.resolve(&self.source_file_dir, specifier));
        match result {
            Ok(resolution) => {
                let resolved_path = resolution.path();
                let abs_path = resolved_path.to_str().unwrap();
                let rel_path = std::path::Path::new(abs_path)
                    .strip_prefix(self.project_root)
                    .unwrap_or_else(|_| std::path::Path::new(abs_path))
                    .to_str()
                    .unwrap()
                    .to_string();
                debug!("Resolved dependency: {} -> {}", specifier, rel_path);
                Some(rel_path)
            }
            _ => {
                debug!("Failed to resolve dependency: {}", specifier);
                None
            }
        }
    }
}

#[cfg(feature = "project-analysis")]
impl Visit for AstVisitor<'_> {
    fn visit_import_decl(&mut self, n: &swc_ecma_ast::ImportDecl) {
        if let Some(resolved_path) = self.resolve_dependency(&n.src.value) {
            let specifiers = n
                .specifiers
                .iter()
                .map(|s| match s {
                    ImportSpecifier::Named(n) => n.local.sym.to_string(),
                    ImportSpecifier::Default(d) => d.local.sym.to_string(),
                    ImportSpecifier::Namespace(ns) => ns.local.sym.to_string(),
                })
                .collect();

            self.imports.push(ImportInfo {
                path: resolved_path,
                specifiers,
                is_type_only: n.type_only,
            });
        }
    }

    fn visit_named_export(&mut self, n: &swc_ecma_ast::NamedExport) {
        if let Some(src) = &n.src {
            if let Some(resolved_path) = self.resolve_dependency(&src.value) {
                let specifiers = n
                    .specifiers
                    .iter()
                    .filter_map(|s| match s {
                        swc_ecma_ast::ExportSpecifier::Named(n) => match &n.orig {
                            swc_ecma_ast::ModuleExportName::Ident(ident) => {
                                Some(ident.sym.to_string())
                            }
                            swc_ecma_ast::ModuleExportName::Str(s) => Some(s.value.to_string()),
                        },
                        _ => None,
                    })
                    .collect();
                self.imports.push(ImportInfo {
                    path: resolved_path,
                    specifiers,
                    is_type_only: n.type_only,
                });
            }
        }
    }

    fn visit_export_all(&mut self, n: &swc_ecma_ast::ExportAll) {
        if let Some(resolved_path) = self.resolve_dependency(&n.src.value) {
            self.imports.push(ImportInfo {
                path: resolved_path,
                specifiers: vec!["*".to_string()],
                is_type_only: n.type_only,
            });
        }
    }

    fn visit_call_expr(&mut self, n: &swc_ecma_ast::CallExpr) {
        if let swc_ecma_ast::Callee::Import(..) = n.callee {
            if let Some(arg) = n.args.get(0) {
                if let swc_ecma_ast::Expr::Lit(lit) = &*arg.expr {
                    if let swc_ecma_ast::Lit::Str(s) = lit {
                        if let Some(resolved_path) = self.resolve_dependency(&s.value) {
                            self.imports.push(ImportInfo {
                                path: resolved_path,
                                specifiers: vec!["dynamic".to_string()],
                                is_type_only: false,
                            });
                        }
                    }
                }
            }
        }
        n.visit_children_with(self);
    }

    fn visit_export_decl(&mut self, n: &ExportDecl) {
        match &n.decl {
            swc_ecma_ast::Decl::Class(class_decl) => {
                self.exports.push(ExportInfo {
                    name: class_decl.ident.sym.to_string(),
                    kind: ExportKind::Class,
                });
            }
            swc_ecma_ast::Decl::Fn(fn_decl) => {
                self.exports.push(ExportInfo {
                    name: fn_decl.ident.sym.to_string(),
                    kind: ExportKind::Value,
                });
            }
            swc_ecma_ast::Decl::Var(var_decl) => {
                for decl in &var_decl.decls {
                    if let Some(ident) = decl.name.as_ident() {
                        self.exports.push(ExportInfo {
                            name: ident.sym.to_string(),
                            kind: ExportKind::Value,
                        });
                    }
                }
            }
            swc_ecma_ast::Decl::TsInterface(iface_decl) => {
                self.exports.push(ExportInfo {
                    name: iface_decl.id.sym.to_string(),
                    kind: ExportKind::Type,
                });
            }
            swc_ecma_ast::Decl::TsTypeAlias(type_alias) => {
                self.exports.push(ExportInfo {
                    name: type_alias.id.sym.to_string(),
                    kind: ExportKind::Type,
                });
            }
            swc_ecma_ast::Decl::TsEnum(e) => {
                self.exports.push(ExportInfo {
                    name: e.id.sym.to_string(),
                    kind: ExportKind::Value,
                });
            }
            swc_ecma_ast::Decl::TsModule(_) => {}
            _ => {}
        }
        n.visit_children_with(self);
    }

    fn visit_export_default_decl(&mut self, n: &ExportDefaultDecl) {
        let name = match &n.decl {
            swc_ecma_ast::DefaultDecl::Class(c) => {
                Some(c.ident.as_ref().map(|i| i.sym.to_string()))
            }
            swc_ecma_ast::DefaultDecl::Fn(f) => Some(f.ident.as_ref().map(|i| i.sym.to_string())),
            _ => None,
        }
        .flatten()
        .unwrap_or_else(|| "default".to_string());

        let kind = match n.decl {
            swc_ecma_ast::DefaultDecl::Class(_) => ExportKind::Class,
            _ => ExportKind::Value,
        };

        self.exports.push(ExportInfo { name, kind });
        n.visit_children_with(self);
    }

    fn visit_export_default_expr(&mut self, n: &ExportDefaultExpr) {
        self.exports.push(ExportInfo {
            name: "default".to_string(),
            kind: ExportKind::Value,
        });
        n.visit_children_with(self);
    }
}

pub struct ParseResult {
    pub module: Result<Module, Error>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub line_count: usize,
    pub source_code: String,
}

fn _parse(
    fm: &swc_common::SourceFile,
    comments: &CountingComments,
    use_tsx: bool,
) -> Result<Module, Error> {
    let ts_config = TsConfig {
        tsx: use_tsx,
        decorators: false,
        dts: false,
        no_early_errors: false,
        disallow_ambiguous_jsx_like: true,
    };

    let lexer = Lexer::new(
        Syntax::Typescript(ts_config),
        EsVersion::Es2020,
        SourceFileInput::from(fm),
        Some(comments),
    );

    let mut parser = Parser::new_from(lexer);
    parser.parse_module()
}

fn count_lines(source: &str, comments_count: usize, include_comments: bool) -> usize {
    let mut line_count = source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();

    if !include_comments {
        line_count -= comments_count;
    }

    line_count
}

/// Parses a TypeScript/JavaScript module from a file path, resolves its dependencies,
/// and calculates line count metrics.
///
/// This function is intended for a full project analysis where understanding the
/// relationships between files is crucial. It loads a file from disk, parses it,
/// and then walks the AST to find all `import` and `export` statements. It uses
/// the provided `Resolver` to trace these dependencies to other files in the project.
///
/// # Arguments
///
/// * `file_path` - The path to the source code file to parse.
/// * `resolver` - A reference to the `rspack_resolver::Resolver` used to resolve module specifiers.
/// * `runtime` - A reference to the `tokio::runtime::Runtime` needed to execute the async resolver.
/// * `project_root` - The root directory of the project.
/// * `include_comments` - A boolean flag indicating whether to include comments in the line count.
///
/// # Returns
///
/// A tuple containing:
/// 1. `Result<Module, Error>` - The parsed SWC module or a parsing error.
/// 2. `Vec<ImportInfo>` - A vector of resolved import/export dependencies.
/// 3. `usize` - The calculated number of lines in the file (excluding empty lines and optionally comments).
/// 4. `String` - The full source code of the file.
#[cfg(feature = "project-analysis")]
pub fn parse_module(
    file_path: &str,
    resolver: &Resolver,
    runtime: &Runtime,
    project_root: &str,
    include_comments: bool,
) -> ParseResult {
    let cm: Lrc<SourceMap> = Default::default();
    let comments = CountingComments::new();

    let fm = cm
        .load_file(std::path::Path::new(file_path))
        .expect("failed to load file");

    let use_tsx = file_path.ends_with(".tsx") || file_path.ends_with(".jsx");

    let parsed_module_result = _parse(&fm, &comments, use_tsx);

    let source_code = fm.src.to_string();
    let line_count = count_lines(&source_code, comments.count(), include_comments);

    let (imports, exports) = if let Ok(ref module) = parsed_module_result {
        let source_file_path = PathBuf::from(file_path);
        let source_file_dir = source_file_path.parent().unwrap().to_path_buf();
        let mut visitor = AstVisitor {
            imports: Vec::new(),
            exports: Vec::new(),
            source_file_dir,
            resolver,
            runtime,
            project_root,
        };
        module.visit_with(&mut visitor);
        (visitor.imports, visitor.exports)
    } else {
        (Vec::new(), Vec::new())
    };

    ParseResult {
        module: parsed_module_result,
        imports,
        exports,
        line_count,
        source_code,
    }
}

/// Parses a TypeScript/JavaScript module from a string of source code.
///
/// This function is a lightweight version of `parse_module`. It does NOT resolve
/// any dependencies, as it operates on a string without the context of a project's
/// file system. It's primarily useful for analyzing isolated code snippets,
/// such as in unit tests or in environments like WASM where file system access
/// is not available.
///
/// # Arguments
///
/// * `source` - A string slice containing the source code to parse.
/// * `use_tsx` - A boolean flag indicating whether to parse the code as TSX.
/// * `include_comments` - A boolean flag indicating whether to include comments in the line count.
///
/// # Returns
///
/// A tuple containing:
/// 1. `Result<Module, Error>` - The parsed SWC module or a parsing error.
/// 2. `usize` - The calculated number of lines in the code (excluding empty lines and optionally comments).
pub fn parse_module_from_string(
    source: &str,
    use_tsx: bool,
    include_comments: bool,
) -> (Result<Module, Error>, usize) {
    let cm: Lrc<SourceMap> = Default::default();
    let comments = CountingComments::new();
    let code: String = source.lines().collect::<Vec<_>>().join("\n");

    let fm = cm.new_source_file(
        swc_common::FileName::Custom("input.ts".to_string()),
        code.clone(),
    );

    let parsed = _parse(&fm, &comments, use_tsx);

    let line_count = count_lines(&code, comments.count(), include_comments);

    (parsed, line_count)
}

struct CountingComments {
    count: Cell<usize>,
}

impl Comments for CountingComments {
    fn add_leading(&self, _pos: BytePos, comment: Comment) {
        if comment.text.trim().starts_with("//") {
            self.count.set(self.count.get() + 1);
        } else {
            self.count
                .set(self.count.get() + 1 + comment.text.matches('\n').count());
        }
    }

    fn add_leading_comments(&self, _pos: BytePos, comments: Vec<Comment>) {
        for comment in comments {
            self.add_leading(_pos, comment);
        }
    }

    fn add_trailing(&self, _pos: BytePos, comment: Comment) {
        if comment.text.trim().starts_with("//") {
            self.count.set(self.count.get() + 1);
        } else {
            self.count
                .set(self.count.get() + 1 + comment.text.matches('\n').count());
        }
    }

    fn add_trailing_comments(&self, _pos: BytePos, comments: Vec<Comment>) {
        for comment in comments {
            self.add_trailing(_pos, comment);
        }
    }

    fn has_leading(&self, _pos: BytePos) -> bool {
        false
    }

    fn has_trailing(&self, _pos: BytePos) -> bool {
        false
    }

    fn take_leading(&self, _pos: BytePos) -> Option<Vec<Comment>> {
        None
    }

    fn take_trailing(&self, _pos: BytePos) -> Option<Vec<Comment>> {
        None
    }

    fn move_leading(&self, _from: swc_common::BytePos, _to: swc_common::BytePos) {
        ()
    }

    fn get_leading(&self, _pos: swc_common::BytePos) -> Option<Vec<swc_common::comments::Comment>> {
        None
    }

    fn move_trailing(&self, _from: swc_common::BytePos, _to: swc_common::BytePos) {}

    fn get_trailing(
        &self,
        _pos: swc_common::BytePos,
    ) -> Option<Vec<swc_common::comments::Comment>> {
        None
    }

    fn add_pure_comment(&self, _pos: swc_common::BytePos) {}
}

impl CountingComments {
    fn new() -> Self {
        Self {
            count: Cell::new(0),
        }
    }

    fn count(&self) -> usize {
        self.count.get()
    }
}
