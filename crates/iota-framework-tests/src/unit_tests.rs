// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{fs, io, path::PathBuf};

use iota_move::unit_test::run_move_unit_tests;
use iota_move_build::BuildConfig;
use move_cli::base::test::UnitTestResult;
use move_package::LintFlag;
use move_unit_test::UnitTestingConfig;

const FILTER_ENV: &str = "FILTER";

#[test]
#[cfg_attr(msim, ignore)]
fn run_iota_framework_tests() {
    check_move_unit_tests({
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.extend(["..", "iota-framework", "packages", "iota-framework"]);
        buf
    });
}

#[test]
#[cfg_attr(msim, ignore)]
fn run_iota_system_tests() {
    check_move_unit_tests({
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.extend(["..", "iota-framework", "packages", "iota-system"]);
        buf
    });
}

#[test]
#[cfg_attr(msim, ignore)]
fn run_deepbook_tests() {
    check_move_unit_tests({
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.extend(["..", "iota-framework", "packages", "deepbook"]);
        buf
    });
}

#[test]
#[cfg_attr(msim, ignore)]
fn run_stardust_tests() {
    check_move_unit_tests({
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.extend(["..", "iota-framework", "packages", "stardust"]);
        buf
    });
}

#[test]
#[cfg_attr(msim, ignore)]
fn run_examples_move_unit_tests() {
    for example in [
        "basics",
        "capy",
        "crypto",
        "defi",
        "fungible_tokens",
        "games",
        "move_tutorial",
        "nfts",
        "objects_tutorial",
    ] {
        let path = {
            let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            buf.extend(["..", "..", "iota_programmability", "examples", example]);
            buf
        };

        check_package_builds(path.clone());
        check_move_unit_tests(path);
    }
}

#[test]
#[cfg_attr(msim, ignore)]
fn run_docs_examples_move_unit_tests() -> io::Result<()> {
    let examples = {
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.extend(["..", "..", "examples", "move"]);
        buf
    };

    for entry in fs::read_dir(examples)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() && entry.path().join("Move.toml").exists() {
            check_package_builds(entry.path());
            check_move_unit_tests(entry.path());
        }
    }

    Ok(())
}

/// Ensure packages build outside of test mode.
fn check_package_builds(path: PathBuf) {
    let mut config = BuildConfig::new_for_testing();
    config.config.dev_mode = true;
    config.run_bytecode_verifier = true;
    config.print_diags_to_stderr = true;
    config.config.warnings_are_errors = true;
    config.config.silence_warnings = false;
    config.config.lint_flag = LintFlag::LEVEL_DEFAULT;
    config
        .build(path.clone())
        .unwrap_or_else(|e| panic!("Building package {}.\nWith error {e}", path.display()));
}

fn check_move_unit_tests(path: PathBuf) {
    let mut config = BuildConfig::new_for_testing();
    // Make sure to verify tests
    config.config.dev_mode = true;
    config.config.test_mode = true;
    config.run_bytecode_verifier = true;
    config.print_diags_to_stderr = true;
    config.config.warnings_are_errors = true;
    config.config.silence_warnings = false;
    config.config.lint_flag = LintFlag::LEVEL_DEFAULT;
    let move_config = config.config.clone();
    let mut testing_config = UnitTestingConfig::default_with_bound(Some(3_000_000));
    testing_config.filter = std::env::var(FILTER_ENV).ok().map(|s| s.to_string());

    assert_eq!(
        run_move_unit_tests(path, move_config, Some(testing_config), false).unwrap(),
        UnitTestResult::Success
    );
}
