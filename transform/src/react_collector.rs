use swc_core::ecma::{
    ast::*,
    atoms::Atom,
    visit::{noop_visit_mut_type, VisitMut},
};
use tracing::debug;

use crate::constants::{CREATE_ELEMENT, INTEROP_REQUIRE_DEFAULT, REACT_PACKAGE, REQUIRE};

#[derive(Debug)]
pub enum ModuleType {
    Esm,
    Cjs,
}

#[derive(Debug)]
pub enum ImportTarget {
    Base,
    CreateElement,
}

#[derive(Debug)]
pub struct ReactImport {
    pub module_type: ModuleType,
    pub target: ImportTarget,
    pub ident: Ident,
}

impl ReactImport {
    fn esm(ident: Ident, target: ImportTarget) -> Self {
        ReactImport {
            module_type: ModuleType::Esm,
            target,
            ident,
        }
    }

    fn cjs(ident: Ident, target: ImportTarget) -> Self {
        ReactImport {
            module_type: ModuleType::Cjs,
            target,
            ident,
        }
    }
}

pub struct ReactCollector {
    pub react_imports: Vec<ReactImport>,
}

impl ReactCollector {
    fn is_react(&mut self, str: &str) -> bool {
        str == REACT_PACKAGE
    }

    fn is_create_element(&mut self, str: &str) -> bool {
        str == CREATE_ELEMENT
    }

    fn is_react_require_expr(&mut self, expr: &Expr) -> bool {
        let require_source = self.get_require_source(expr);
        let require_interop_source = self.get_interop_require_default_source(expr);
        if let Some(source) = require_source.or(require_interop_source) {
            debug!("is_react_require_expr: {:#?}", source);
            return source.as_str() == REACT_PACKAGE;
        }
        false
    }

    /// Get `source` in `require(source)`
    fn get_require_source(&mut self, expr: &Expr) -> Option<Atom> {
        if let Some(call_expr) = expr.as_call() {
            if let Some(expr) = call_expr
                .callee
                .as_expr()
                .and_then(|callee_expr| callee_expr.as_ident())
            {
                if expr.sym == REQUIRE {
                    debug!("get_require_source: {:#?}", expr.sym);
                    if let Some(lit) = call_expr
                        .args
                        .get(0)
                        .and_then(|expr_or_spread| expr_or_spread.expr.as_lit())
                    {
                        return match lit {
                            Lit::Str(Str { value, .. }) => return Some(value.clone()),
                            _ => None,
                        };
                    }
                }
            }
        }
        None
    }

    /// Get `source` in `_interopRequireDefault(require(source))`
    fn get_interop_require_default_source(&mut self, expr: &Expr) -> Option<Atom> {
        if let Some(callee_expr) = expr
            .as_call()
            .and_then(|call_expr| call_expr.callee.as_expr())
        {
            if callee_expr
                .as_ident()
                .map_or(false, |ident| ident.sym == INTEROP_REQUIRE_DEFAULT)
            {
                if let Some(expr_or_spread) =
                    expr.as_call().and_then(|call_expr| call_expr.args.get(0))
                {
                    debug!(
                        "get_interop_require_default_source: {:#?}",
                        callee_expr.as_ident().unwrap().sym
                    );
                    return self.get_require_source(expr_or_spread.expr.as_ref());
                }
                return None;
            }
        }
        None
    }

    /// Collect React from variable declare statements.
    fn collect_from_var_decl(&mut self, var_decl: &VarDecl) {
        if var_decl.decls.len() != 1 || var_decl.declare {
            return;
        }

        if let Some(var_declarator) = var_decl.decls.get(0) {
            if let Some(expr) = var_declarator.init.as_ref() {
                // `const React = require('react')`
                // `const React = _interopRequireDefault(require('react'))`
                // `const { createElement } = require('react')`
                // `const { createElement } = _interopRequireDefault(require('react'))`
                // `const { createElement as c } = require('react')`
                // `const { createElement as c } = _interopRequireDefault(require('react'))`
                if self.is_react_require_expr(expr) {
                    if let Some(binding_ident) = var_declarator.name.as_ident() {
                        self.react_imports.push(ReactImport::cjs(
                            binding_ident.id.clone(),
                            ImportTarget::Base,
                        ));
                    } else if let Some(ObjectPat {
                        optional: false,
                        props,
                        ..
                    }) = var_declarator.name.as_object()
                    {
                        for prop in props.iter() {
                            match prop {
                                ObjectPatProp::Assign(assign_pat_prop) => {
                                    self.react_imports.push(ReactImport::cjs(
                                        assign_pat_prop.key.clone(),
                                        ImportTarget::CreateElement,
                                    ));
                                    return;
                                }
                                ObjectPatProp::KeyValue(KeyValuePatProp {
                                    key: PropName::Ident(key_ident),
                                    value: value_pat,
                                }) => {
                                    if self.is_create_element(key_ident.sym.as_str()) {
                                        if let Some(value_ident) = value_pat.as_ident() {
                                            self.react_imports.push(ReactImport::cjs(
                                                value_ident.id.clone(),
                                                ImportTarget::CreateElement,
                                            ));
                                            return;
                                        }
                                    }
                                }
                                ObjectPatProp::KeyValue(KeyValuePatProp {
                                    key: PropName::Str(str),
                                    value: value_pat,
                                }) => {
                                    if self.is_create_element(str.value.as_str()) {
                                        if let Some(value_ident) = value_pat.as_ident() {
                                            self.react_imports.push(ReactImport::cjs(
                                                value_ident.id.clone(),
                                                ImportTarget::CreateElement,
                                            ));
                                            return;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    /// Collect React from import statements.
    fn collect_from_import_specifiers(&mut self, specifiers: &Vec<ImportSpecifier>) {
        for specifier in specifiers {
            match specifier {
                // `import React from 'react'`
                ImportSpecifier::Default(default) => {
                    self.react_imports
                        .push(ReactImport::esm(default.local.clone(), ImportTarget::Base));
                }
                // `import * as React from 'react'`
                ImportSpecifier::Namespace(namespace) => {
                    self.react_imports.push(ReactImport::esm(
                        namespace.local.clone(),
                        ImportTarget::Base,
                    ));
                }
                // `import { createElement } from 'react'`
                // `import { createElement as other } from 'react'`
                ImportSpecifier::Named(named)
                    if self.is_create_element(named.local.sym.as_str()) =>
                {
                    if let Some(ModuleExportName::Ident(import_ident)) = &named.imported {
                        self.react_imports.push(ReactImport::esm(
                            import_ident.clone(),
                            ImportTarget::CreateElement,
                        ));
                    } else {
                        self.react_imports.push(ReactImport::esm(
                            named.local.clone(),
                            ImportTarget::CreateElement,
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

impl Default for ReactCollector {
    fn default() -> Self {
        ReactCollector {
            react_imports: Vec::new(),
        }
    }
}

impl VisitMut for ReactCollector {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        for module_item in &module.body {
            match module_item {
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(var_decl))) => {
                    self.collect_from_var_decl(var_decl.as_ref());
                }
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    type_only: false,
                    specifiers,
                    src,
                    ..
                })) if self.is_react(src.value.as_str()) => {
                    self.collect_from_import_specifiers(specifiers);
                }
                _ => {}
            }
        }
    }
}
