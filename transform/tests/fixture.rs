use std::path::PathBuf;

use swc_core::ecma::parser::{Syntax, TsConfig};
use swc_ecma_transforms_testing::test_fixture;
use swc_nativewind::nativewind;

#[testing::fixture("tests/fixture/default/**/input.js")]
fn default_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");

    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| nativewind(String::from("/someFile.js")),
        &input,
        &output,
        Default::default(),
    );
}

#[testing::fixture("tests/fixture/denied_file/**/input.js")]
fn denied_file_fixture(input: PathBuf) {
    let filename = input.to_string_lossy();
    let output = input.with_file_name("output.js");

    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: filename.ends_with(".tsx"),
            ..Default::default()
        }),
        &|_| {
            nativewind(String::from(
                "/node_modules/react-native-css-interop/someFile.js",
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}
