mod constants;
mod react_collector;

use crate::{
    constants::{
        CREATE_ELEMENT, CREATE_ELEMENT_AND_CHECK_CSS_INTEROP, REACT_NATIVE_CSS_INTEROP_PACKAGE,
        REQUIRE,
    },
    react_collector::ModuleType,
};
use react_collector::{ReactCollector, ReactImport};
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
    interop_ident: Ident,
    react_imports: Vec<ReactImport>,
    replaced: i32,
}

impl NativeWindVisitor {
    fn default() -> Self {
        NativeWindVisitor {
            interop_ident: private_ident!("__c"),
            react_imports: Vec::new(),
            replaced: 0,
        }
    }

    fn is_collected_react(&mut self, target_sym: &Atom) -> bool {
        for import in &self.react_imports {
            if *target_sym == import.ident.sym {
                return true;
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

    /// `const { createElementAndCheckCssInterop: __c } = require('react-native-css-interop')`
    fn get_require_create_element_and_check_css_interop(&mut self) -> ModuleItem {
        ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                definite: false,
                name: Pat::Object(ObjectPat {
                    span: DUMMY_SP,
                    props: vec![ObjectPatProp::KeyValue(KeyValuePatProp {
                        key: PropName::Ident(Ident::new(
                            js_word!(CREATE_ELEMENT_AND_CHECK_CSS_INTEROP),
                            DUMMY_SP,
                        )),
                        value: Box::new(Pat::Ident(BindingIdent {
                            id: self.interop_ident.clone(),
                            type_ann: None,
                        })),
                    })],
                    optional: false,
                    type_ann: None,
                }),
                init: Some(Box::new(Expr::Call(CallExpr {
                    span: DUMMY_SP,
                    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(
                        js_word!(REQUIRE),
                        DUMMY_SP,
                    )))),
                    args: vec![ExprOrSpread {
                        expr: Box::new(Expr::Lit(Lit::Str(Str {
                            span: DUMMY_SP,
                            value: Atom::from(REACT_NATIVE_CSS_INTEROP_PACKAGE),
                            raw: None,
                        }))),
                        spread: None,
                    }],
                    type_args: None,
                }))),
            }],
            declare: false,
        }))))
    }
}

impl VisitMut for NativeWindVisitor {
    noop_visit_mut_type!();

    fn visit_mut_module(&mut self, module: &mut Module) {
        // Collect all of React(import or require) from module.
        let mut collector = ReactCollector::default();
        module.visit_mut_with(&mut collector);

        let is_esm = collector
            .react_imports
            .iter()
            .any(|import| match import.module_type {
                ModuleType::Esm => true,
                ModuleType::Cjs => false,
            });

        self.react_imports.append(&mut collector.react_imports);
        debug!("react_imports: {:#?}", self.react_imports);

        // After collect, visit children.
        module.visit_mut_children_with(self);

        if self.react_imports.len() > 0 && self.replaced > 0 {
            module.body.insert(
                0,
                if is_esm {
                    self.get_import_create_element_and_check_css_interop()
                } else {
                    self.get_require_create_element_and_check_css_interop()
                },
            );
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
                }) => {
                    if self.is_collected_react(sym) {
                        self.replaced += 1;
                        *call_expr = self.get_create_element_interop_call_expr(call_expr.clone());
                    }
                }
                // `React.createElement(...)`
                Expr::Member(MemberExpr { obj, prop, .. }) => {
                    if let (Some(ident), Some(prop)) = (obj.as_ident(), prop.as_ident()) {
                        if prop.sym == CREATE_ELEMENT && self.is_collected_react(&ident.sym) {
                            self.replaced += 1;
                            *call_expr =
                                self.get_create_element_interop_call_expr(call_expr.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn nativewind() -> impl VisitMut + Fold {
    as_folder(NativeWindVisitor::default())
}
