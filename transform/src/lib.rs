mod constants;
mod react_collector;

use crate::constants::{
    CREATE_ELEMENT, CREATE_ELEMENT_AND_CHECK_CSS_INTEROP, REACT_NATIVE_CSS_INTEROP_PACKAGE,
};
use constants::DENIED_FILE_REGEX;
use react_collector::{ImportTarget, ReactCollector, ReactImport};
use regex::Regex;
use swc_core::{
    atoms::{js_word, Atom},
    common::DUMMY_SP,
    ecma::{
        ast::*,
        utils::private_ident,
        visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith},
    },
};
use tracing::debug;

pub struct NativeWindVisitor {
    filename: String,
    interop_ident: Ident,
    react_imports: Vec<ReactImport>,
    replaced_create_element_cnt: i32,
}

impl NativeWindVisitor {
    fn default(filename: String) -> Self {
        NativeWindVisitor {
            filename,
            interop_ident: private_ident!("__c"),
            react_imports: Vec::new(),
            replaced_create_element_cnt: 0,
        }
    }

    fn is_denied_file(&mut self) -> bool {
        let allowed_filename_regex = Regex::new(DENIED_FILE_REGEX).unwrap();
        allowed_filename_regex.is_match(&self.filename)
    }

    fn is_react(&mut self, target_sym: &Atom) -> bool {
        self.react_imports.iter().any(|import| match import.target {
            ImportTarget::Base => *target_sym == import.ident.sym,
            _ => false,
        })
    }

    fn is_create_element(&mut self, target_sym: &Atom) -> bool {
        self.react_imports.iter().any(|import| match import.target {
            ImportTarget::CreateElement => *target_sym == import.ident.sym,
            _ => false,
        })
    }

    fn is_react_create_element_member_expr(
        &mut self,
        member_expr: &MemberExpr,
        check_create_element: bool,
    ) -> bool {
        if let Some(prop_ident) = member_expr.prop.as_ident() {
            if check_create_element && prop_ident.sym != CREATE_ELEMENT {
                return false;
            }

            if let Some(obj_ident) = member_expr.obj.as_ident() {
                debug!("is_react_create_element_member_expr");
                return self.is_react(&obj_ident.sym);
            } else if let Some(inner_member_expr) = member_expr.obj.as_member() {
                debug!(
                    "is_react_create_element_member_expr inner: {:#?}",
                    inner_member_expr
                );
                return self.is_react_create_element_member_expr(inner_member_expr, false);
            }
        }
        false
    }

    fn get_create_element_interop_call_expr(&mut self, orig_call_expr: CallExpr) -> CallExpr {
        CallExpr {
            callee: Callee::Expr(Box::new(Expr::Ident(self.interop_ident.clone()))),
            args: orig_call_expr.args.clone(),
            span: orig_call_expr.span,
            type_args: None,
        }
    }

    /// `import { createElementAndCheckCssInterop as __c } from 'react-native-css-interop'`
    fn get_import_create_element_and_check_css_interop(&mut self) -> ModuleItem {
        ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
            span: DUMMY_SP,
            src: Box::new(Str::from(REACT_NATIVE_CSS_INTEROP_PACKAGE)),
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                span: DUMMY_SP,
                local: self.interop_ident.clone(),
                imported: Some(ModuleExportName::Ident(Ident::new(
                    js_word!(CREATE_ELEMENT_AND_CHECK_CSS_INTEROP),
                    DUMMY_SP,
                ))),
                is_type_only: false,
            })],
            type_only: false,
            with: None,
        }))
    }
}

impl VisitMut for NativeWindVisitor {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        if self.is_denied_file() {
            debug!("this file is denied: {:#?}", self.filename);
            return;
        }

        // Collect all of React(import or require) from module.
        let mut collector = ReactCollector::default();
        module.visit_mut_with(&mut collector);

        self.react_imports.append(&mut collector.react_imports);
        debug!("react_imports: {:#?}", self.react_imports);

        // After collect, visit children.
        module.visit_mut_children_with(self);

        if self.react_imports.len() > 0 && self.replaced_create_element_cnt > 0 {
            module
                .body
                .insert(0, self.get_import_create_element_and_check_css_interop());
        }
    }

    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        debug!("visit_mut_call_expr: {:#?}", call_expr);
        if call_expr.type_args.is_some() {
            return;
        }

        if let Some(callee_expr) = call_expr.callee.as_expr() {
            match callee_expr.as_ref() {
                // `createElement(...)`
                Expr::Ident(Ident {
                    optional: false,
                    sym,
                    ..
                }) if self.is_create_element(sym) => {
                    self.replaced_create_element_cnt += 1;
                    *call_expr = self.get_create_element_interop_call_expr(call_expr.clone());
                }
                // A: `<react_ident>.createElement(...)`
                // B: `<react_ident>.default.createElement(...)`
                Expr::Member(member_expr)
                    if self.is_react_create_element_member_expr(member_expr, true) =>
                {
                    self.replaced_create_element_cnt += 1;
                    *call_expr = self.get_create_element_interop_call_expr(call_expr.clone());
                }
                _ => {}
            }
        }
    }
}

pub fn nativewind(filename: String) -> impl VisitMut + Fold {
    as_folder(NativeWindVisitor::default(filename))
}
