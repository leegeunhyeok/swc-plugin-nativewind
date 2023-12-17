use std::path::PathBuf;

use swc_core::ecma::parser::{Syntax, TsConfig};
use swc_ecma_transforms_testing::test_fixture;
use swc_nativewind::nativewind;

#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");

    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| {
            nativewind(String::from(if filename.contains("denied_modules") {
                // denied file name
                "/node_modules/react-native-css-interop/someFile.js"
            } else {
                // allowed file name
                "/someFile.js"
            }))
        },
        &input,
        &output,
        Default::default(),
    );
}
