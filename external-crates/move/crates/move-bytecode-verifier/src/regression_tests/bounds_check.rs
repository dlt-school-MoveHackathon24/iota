// Copyright (c) The Move Contributors
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use hex_literal::hex;
use move_binary_format::file_format::CompiledScript;

#[test]
fn script_with_too_many_locals() {
    let bad_script = hex!(
        "a11ceb0b05000000010500d1018701010101010705050601070101010101070505060107050505050505060107010505050601070101010101070505060107050a0101010101070505060107010101010107050506010705050505050506010701050505060107010101010107010101010c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c050c0c0c0c0c0c0c0c0c0c0c012d0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c030602000000070b010c000a20010b000c010500"
    );
    let _ = CompiledScript::deserialize(&bad_script).unwrap_err();
}
