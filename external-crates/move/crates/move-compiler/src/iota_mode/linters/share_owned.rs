// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This analysis flags making objects passed as function parameters or
//! resulting from unpacking (likely already owned) shareable which would lead
//! to an abort. A typical patterns is to create a fresh object and share it
//! within the same function

use std::collections::BTreeMap;

use move_ir_types::location::*;

use super::{
    type_abilities, LinterDiagCategory, IOTA_PKG_NAME, LINTER_DEFAULT_DIAG_CODE,
    LINT_WARNING_PREFIX, PUBLIC_SHARE_FUN, SHARE_FUN, TRANSFER_MOD_NAME,
};
use crate::{
    cfgir::{
        absint::JoinResult,
        ast::Program,
        visitor::{
            LocalState, SimpleAbsInt, SimpleAbsIntConstructor, SimpleDomain, SimpleExecutionContext,
        },
        CFGContext,
    },
    diag,
    diagnostics::{
        codes::{custom, DiagnosticInfo, Severity},
        Diagnostic, Diagnostics,
    },
    hlir::ast::{
        Exp, LValue, LValue_, Label, ModuleCall, SingleType, Type, Type_, UnannotatedExp_, Var,
    },
    parser::ast::Ability_,
    shared::{CompilationEnv, Identifier},
};

const SHARE_FUNCTIONS: &[(&str, &str, &str)] = &[
    (IOTA_PKG_NAME, TRANSFER_MOD_NAME, PUBLIC_SHARE_FUN),
    (IOTA_PKG_NAME, TRANSFER_MOD_NAME, SHARE_FUN),
];

const SHARE_OWNED_DIAG: DiagnosticInfo = custom(
    LINT_WARNING_PREFIX,
    Severity::Warning,
    LinterDiagCategory::ShareOwned as u8,
    LINTER_DEFAULT_DIAG_CODE,
    "possible owned object share",
);

//**************************************************************************************************
// types
//**************************************************************************************************

pub struct ShareOwnedVerifier;
pub struct ShareOwnedVerifierAI;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Value {
    /// a fresh object resulting from packing
    FreshObj,
    /// a most likely non-fresh object coming from unpacking or a function
    /// argument
    NotFreshObj(Loc),
    #[default]
    Other,
}

pub struct ExecutionContext {
    diags: Diagnostics,
}

#[derive(Clone, Debug)]
pub struct State {
    locals: BTreeMap<Var, LocalState<Value>>,
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl SimpleAbsIntConstructor for ShareOwnedVerifier {
    type AI<'a> = ShareOwnedVerifierAI;

    fn new<'a>(
        _env: &CompilationEnv,
        program: &'a Program,
        context: &'a CFGContext<'a>,
        _init_state: &mut <Self::AI<'a> as SimpleAbsInt>::State,
    ) -> Option<Self::AI<'a>> {
        if context.attributes.is_test_or_test_only()
            || program
                .modules
                .get(&context.module)
                .unwrap()
                .attributes
                .is_test_or_test_only()
        {
            return None;
        }
        Some(ShareOwnedVerifierAI)
    }
}

impl SimpleAbsInt for ShareOwnedVerifierAI {
    type State = State;
    type ExecutionContext = ExecutionContext;

    fn finish(&mut self, _final_states: BTreeMap<Label, State>, diags: Diagnostics) -> Diagnostics {
        diags
    }

    fn start_command(&self, _: &mut State) -> ExecutionContext {
        ExecutionContext {
            diags: Diagnostics::new(),
        }
    }

    fn finish_command(&self, context: ExecutionContext, _state: &mut State) -> Diagnostics {
        let ExecutionContext { diags } = context;
        diags
    }

    fn exp_custom(
        &self,
        context: &mut ExecutionContext,
        state: &mut State,
        e: &Exp,
    ) -> Option<Vec<Value>> {
        use UnannotatedExp_ as E;

        if let E::Pack(_, _, fields) = &e.exp.value {
            for (_, _, inner) in fields.iter() {
                self.exp(context, state, inner);
            }
            return Some(vec![Value::FreshObj]);
        };

        None
    }

    fn call_custom(
        &self,
        context: &mut ExecutionContext,
        _state: &mut State,
        loc: &Loc,
        return_ty: &Type,
        f: &ModuleCall,
        args: Vec<Value>,
    ) -> Option<Vec<Value>> {
        if SHARE_FUNCTIONS
            .iter()
            .any(|(addr, module, fun)| f.is(addr, module, fun))
            && args[0] != Value::FreshObj
        {
            let msg = "Potential abort from a (potentially) owned object created by a different transaction.";
            let uid_msg = "Creating a fresh object and sharing it within the same function will ensure this does not abort.";
            let mut d = diag!(
                SHARE_OWNED_DIAG,
                (*loc, msg),
                (f.arguments[0].exp.loc, uid_msg)
            );
            if let Value::NotFreshObj(l) = args[0] {
                d.add_secondary_label((l, "A potentially owned object coming from here"))
            }
            context.add_diag(d)
        }
        Some(match &return_ty.value {
            Type_::Unit => vec![],
            Type_::Single(t) => {
                let v = if is_obj_type(t) {
                    Value::NotFreshObj(t.loc)
                } else {
                    Value::Other
                };
                vec![v]
            }
            Type_::Multiple(types) => types
                .iter()
                .map(|t| {
                    if is_obj_type(t) {
                        Value::NotFreshObj(t.loc)
                    } else {
                        Value::Other
                    }
                })
                .collect(),
        })
    }

    fn lvalue_custom(
        &self,
        context: &mut ExecutionContext,
        state: &mut State,
        l: &LValue,
        _value: &Value,
    ) -> bool {
        use LValue_ as L;

        let sp!(_, l_) = l;
        if let L::Unpack(_, _, fields) = l_ {
            for (f, l) in fields {
                let v = if is_obj(l) {
                    Value::NotFreshObj(f.loc())
                } else {
                    Value::default()
                };
                self.lvalue(context, state, l, v)
            }
            return true;
        }
        false
    }
}

fn is_obj(sp!(_, l_): &LValue) -> bool {
    if let LValue_::Var { ty: st, .. } = l_ {
        return is_obj_type(st);
    }
    false
}

fn is_obj_type(st_: &SingleType) -> bool {
    let Some(abilities) = type_abilities(st_) else {
        return false;
    };
    abilities.has_ability_(Ability_::Key)
}

impl SimpleDomain for State {
    type Value = Value;

    fn new(context: &CFGContext, mut locals: BTreeMap<Var, LocalState<Value>>) -> Self {
        for (_mut, v, st) in &context.signature.parameters {
            if is_obj_type(st) {
                let local_state = locals.get_mut(v).unwrap();
                if let LocalState::Available(loc, _) = local_state {
                    *local_state = LocalState::Available(*loc, Value::NotFreshObj(*loc));
                }
            }
        }
        State { locals }
    }

    fn locals_mut(&mut self) -> &mut BTreeMap<Var, LocalState<Value>> {
        &mut self.locals
    }

    fn locals(&self) -> &BTreeMap<Var, LocalState<Value>> {
        &self.locals
    }

    fn join_value(v1: &Value, v2: &Value) -> Value {
        match (v1, v2) {
            (Value::FreshObj, Value::FreshObj) => Value::FreshObj,
            (stale @ Value::NotFreshObj(_), _) | (_, stale @ Value::NotFreshObj(_)) => *stale,
            (Value::Other, _) | (_, Value::Other) => Value::Other,
        }
    }

    fn join_impl(&mut self, _: &Self, _: &mut JoinResult) {}
}

impl SimpleExecutionContext for ExecutionContext {
    fn add_diag(&mut self, diag: Diagnostic) {
        self.diags.add(diag)
    }
}
