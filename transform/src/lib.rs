mod constants;
mod react_collector;

use crate::{
    constants::{CREATE_ELEMENT_AND_CHECK_CSS_INTEROP, REACT_NATIVE_CSS_INTEROP_PACKAGE, REQUIRE},
    react_collector::ModuleType,
};
use react_collector::ReactCollector;
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
}

impl NativeWindVisitor {
    fn default() -> Self {
        NativeWindVisitor {
            interop_ident: private_ident!("__c"),
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
        let mut collector = ReactCollector::default();
        module.visit_mut_with(&mut collector);

        let ReactCollector { react_imports } = collector;
        debug!("react_imports: {:#?}", react_imports);

        if react_imports.len() == 0 {
            return;
        }

        module.body.insert(
            0,
            if react_imports
                .into_iter()
                .any(|import| match import.module_type {
                    ModuleType::Esm => true,
                    ModuleType::Cjs => false,
                })
            {
                self.get_import_create_element_and_check_css_interop()
            } else {
                self.get_require_create_element_and_check_css_interop()
            },
        );
    }
}

pub fn nativewind() -> impl VisitMut + Fold {
    as_folder(NativeWindVisitor::default())
}
