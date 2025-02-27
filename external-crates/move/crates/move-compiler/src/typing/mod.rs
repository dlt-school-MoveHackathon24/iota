// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod ast;
pub mod core;
mod dependency_ordering;
mod expand;
mod infinite_instantiations;
mod macro_expand;
mod recursive_structs;
mod syntax_methods;
pub(crate) mod translate;
pub mod visitor;
