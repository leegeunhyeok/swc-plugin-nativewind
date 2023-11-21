use regex::Regex;
use swc_core::{
    ecma::{ast::Program, visit::FoldWith},
    plugin::{
        metadata::TransformPluginMetadataContextKind, plugin_transform,
        proxies::TransformPluginProgramMetadata,
    },
};
use swc_nativewind::nativewind;

const ALLOWED_FILE_REGEX: &str =
    r"^[^\/\\]*(react|react-native|react-native-web|react-native-css-interop)[^\/\\]*$";

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let allowed_filename_regex = Regex::new(ALLOWED_FILE_REGEX).unwrap();
    let filename = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap();

    if allowed_filename_regex.is_match(filename.as_str()) {
        program.fold_with(&mut nativewind())
    } else {
        program
    }
}
