use swc_core::{
    ecma::{ast::Program, visit::FoldWith},
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_nativewind::nativewind;

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut nativewind(
        metadata
            .get_context(&TransformPluginMetadataContextKind::Filename)
            .unwrap(),
    ))
}
