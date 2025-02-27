// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This analysis flags situations when instances of a iota::table::Table or
//! iota::table_vec::TableVec or iota::bag::Bag are being compared for
//! (in)equality at this type of comparison is not very useful and DOES NOT take
//! into consideration structural (in)equality.

use super::{
    base_type, LinterDiagCategory, BAG_MOD_NAME, BAG_STRUCT_NAME, IOTA_PKG_NAME,
    LINKED_TABLE_MOD_NAME, LINKED_TABLE_STRUCT_NAME, LINTER_DEFAULT_DIAG_CODE, LINT_WARNING_PREFIX,
    OBJECT_BAG_MOD_NAME, OBJECT_BAG_STRUCT_NAME, OBJECT_TABLE_MOD_NAME, OBJECT_TABLE_STRUCT_NAME,
    TABLE_MOD_NAME, TABLE_STRUCT_NAME, TABLE_VEC_MOD_NAME, TABLE_VEC_STRUCT_NAME, VEC_MAP_MOD_NAME,
    VEC_MAP_STRUCT_NAME, VEC_SET_MOD_NAME, VEC_SET_STRUCT_NAME,
};
use crate::{
    diag,
    diagnostics::{
        codes::{custom, DiagnosticInfo, Severity},
        WarningFilters,
    },
    naming::ast as N,
    parser::ast as P,
    shared::{program_info::TypingProgramInfo, CompilationEnv, Identifier},
    typing::{
        ast as T,
        visitor::{TypingVisitorConstructor, TypingVisitorContext},
    },
};

const COLLECTIONS_EQUALITY_DIAG: DiagnosticInfo = custom(
    LINT_WARNING_PREFIX,
    Severity::Warning,
    LinterDiagCategory::CollectionEquality as u8,
    LINTER_DEFAULT_DIAG_CODE,
    "possibly useless collections compare",
);

const COLLECTION_TYPES: &[(&str, &str, &str)] = &[
    (IOTA_PKG_NAME, BAG_MOD_NAME, BAG_STRUCT_NAME),
    (IOTA_PKG_NAME, OBJECT_BAG_MOD_NAME, OBJECT_BAG_STRUCT_NAME),
    (IOTA_PKG_NAME, TABLE_MOD_NAME, TABLE_STRUCT_NAME),
    (
        IOTA_PKG_NAME,
        OBJECT_TABLE_MOD_NAME,
        OBJECT_TABLE_STRUCT_NAME,
    ),
    (
        IOTA_PKG_NAME,
        LINKED_TABLE_MOD_NAME,
        LINKED_TABLE_STRUCT_NAME,
    ),
    (IOTA_PKG_NAME, TABLE_VEC_MOD_NAME, TABLE_VEC_STRUCT_NAME),
    (IOTA_PKG_NAME, VEC_MAP_MOD_NAME, VEC_MAP_STRUCT_NAME),
    (IOTA_PKG_NAME, VEC_SET_MOD_NAME, VEC_SET_STRUCT_NAME),
];

pub struct CollectionEqualityVisitor;
pub struct Context<'a> {
    env: &'a mut CompilationEnv,
}

impl TypingVisitorConstructor for CollectionEqualityVisitor {
    type Context<'a> = Context<'a>;

    fn context<'a>(
        env: &'a mut CompilationEnv,
        _program_info: &'a TypingProgramInfo,
        _program: &T::Program_,
    ) -> Self::Context<'a> {
        Context { env }
    }
}

impl TypingVisitorContext for Context<'_> {
    fn visit_exp_custom(&mut self, exp: &mut T::Exp) -> bool {
        use T::UnannotatedExp_ as E;
        if let E::BinopExp(_, op, t, _) = &exp.exp.value {
            if op.value != P::BinOp_::Eq && op.value != P::BinOp_::Neq {
                // not a comparison
                return false;
            }
            let Some(bt) = base_type(t) else {
                return false;
            };
            let N::Type_::Apply(_, tname, _) = &bt.value else {
                return false;
            };
            let N::TypeName_::ModuleType(mident, sname) = tname.value else {
                return false;
            };

            if let Some((caddr, cmodule, cname)) =
                COLLECTION_TYPES.iter().find(|(caddr, cmodule, cname)| {
                    mident.value.is(*caddr, *cmodule) && sname.value().as_str() == *cname
                })
            {
                let msg = format!(
                    "Comparing collections of type '{caddr}::{cmodule}::{cname}' may yield unexpected result."
                );
                let note_msg = format!(
                    "Equality for collections of type '{caddr}::{cmodule}::{cname}' IS NOT a structural check based on content"
                );
                let mut d = diag!(COLLECTIONS_EQUALITY_DIAG, (op.loc, msg),);
                d.add_note(note_msg);
                self.env.add_diag(d);
                return true;
            }
        }
        false
    }

    fn add_warning_filter_scope(&mut self, filter: WarningFilters) {
        self.env.add_warning_filter_scope(filter)
    }

    fn pop_warning_filter_scope(&mut self) {
        self.env.pop_warning_filter_scope()
    }
}
