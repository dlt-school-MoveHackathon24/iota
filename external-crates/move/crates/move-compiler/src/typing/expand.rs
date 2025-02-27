// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use move_core_types::u256::U256;
use move_ir_types::location::*;
use move_proc_macros::growing_stack;

use super::core::{self, Context};
use crate::{
    debug_display, diag,
    editions::FeatureGate,
    expansion::ast::Value_,
    ice,
    naming::ast::{BuiltinTypeName_, FunctionSignature, Type, TypeName_, Type_},
    parser::ast::Ability_,
    typing::ast as T,
};

//**************************************************************************************************
// Functions
//**************************************************************************************************

pub fn function_body_(context: &mut Context, b_: &mut T::FunctionBody_) {
    match b_ {
        T::FunctionBody_::Native | T::FunctionBody_::Macro => (),
        T::FunctionBody_::Defined(es) => sequence(context, es),
    }
}

pub fn function_signature(context: &mut Context, sig: &mut FunctionSignature) {
    for (_, _, st) in &mut sig.parameters {
        type_(context, st);
    }
    type_(context, &mut sig.return_type);
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn expected_types(context: &mut Context, ss: &mut [Option<Type>]) {
    for st_opt in ss.iter_mut().flatten() {
        type_(context, st_opt);
    }
}

fn types(context: &mut Context, ss: &mut Vec<Type>) {
    for st in ss {
        type_(context, st);
    }
}

pub fn type_(context: &mut Context, ty: &mut Type) {
    use Type_::*;
    match &mut ty.value {
        Anything | UnresolvedError | Param(_) | Unit => (),
        Ref(_, b) => type_(context, b),
        Var(tvar) => {
            let ty_tvar = sp(ty.loc, Var(*tvar));
            let replacement = core::unfold_type(&context.subst, ty_tvar);
            let replacement = match replacement {
                sp!(loc, Var(_)) => {
                    let diag = ice!((
                        ty.loc,
                        "ICE unfold_type_base failed to expand type inf. var"
                    ));
                    context.env.add_diag(diag);
                    sp(loc, UnresolvedError)
                }
                sp!(loc, Anything) => {
                    let msg = "Could not infer this type. Try adding an annotation";
                    context
                        .env
                        .add_diag(diag!(TypeSafety::UninferredType, (ty.loc, msg)));
                    sp(loc, UnresolvedError)
                }
                sp!(loc, Fun(_, _)) if !context.in_macro_function => {
                    // catch this here for better location infomration (the tvar instead of the fun)
                    unexpected_lambda_type(context, ty.loc);
                    sp(loc, UnresolvedError)
                }
                t => t,
            };
            *ty = replacement;
            type_(context, ty);
        }
        Apply(Some(_), sp!(_, TypeName_::Builtin(_)), tys) => types(context, tys),
        aty @ Apply(Some(_), _, _) => {
            let diag = ice!((
                ty.loc,
                format!("ICE expanding pre-expanded type {}", debug_display!(aty))
            ));
            context.env.add_diag(diag);
            *ty = sp(ty.loc, UnresolvedError)
        }
        Apply(None, _, _) => {
            let abilities = core::infer_abilities(&context.modules, &context.subst, ty.clone());
            match &mut ty.value {
                Apply(abilities_opt, _, tys) => {
                    *abilities_opt = Some(abilities);
                    types(context, tys);
                }
                _ => {
                    let diag = ice!((ty.loc, "ICE type-apply switched to non-apply"));
                    context.env.add_diag(diag);
                    *ty = sp(ty.loc, UnresolvedError)
                }
            }
        }
        Fun(args, result) => {
            if context.in_macro_function {
                types(context, args);
                type_(context, result);
            } else {
                unexpected_lambda_type(context, ty.loc);
                *ty = sp(ty.loc, UnresolvedError)
            }
        }
    }
}

fn unexpected_lambda_type(context: &mut Context, loc: Loc) {
    if context
        .env
        .check_feature(context.current_package, FeatureGate::MacroFuns, loc)
    {
        let msg = "Unexpected lambda type. \
            Lambdas can only be used with 'macro' functions, as parameters or direct arguments";
        context
            .env
            .add_diag(diag!(TypeSafety::UnexpectedFunctionType, (loc, msg)));
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[growing_stack]
fn sequence(context: &mut Context, (_, seq): &mut T::Sequence) {
    for item in seq {
        sequence_item(context, item)
    }
}

#[growing_stack]
fn sequence_item(context: &mut Context, item: &mut T::SequenceItem) {
    use T::SequenceItem_ as S;
    match &mut item.value {
        S::Seq(te) => exp(context, te),

        S::Declare(tbind) => lvalues(context, tbind),
        S::Bind(tbind, tys, te) => {
            lvalues(context, tbind);
            expected_types(context, tys);
            exp(context, te)
        }
    }
}

#[growing_stack]
pub fn exp(context: &mut Context, e: &mut T::Exp) {
    use T::UnannotatedExp_ as E;
    match &e.exp.value {
        // dont expand the type for return, abort, break, or continue
        E::Give(_, _) | E::Continue(_) | E::Return(_) | E::Abort(_) => {
            let t = e.ty.clone();
            match core::unfold_type(&context.subst, t) {
                sp!(_, Type_::Anything) => (),
                mut t => {
                    // report errors if there is an uninferred type argument somewhere
                    type_(context, &mut t);
                }
            }
            e.ty = sp(e.ty.loc, Type_::Anything)
        }
        E::Loop {
            has_break: false, ..
        } => {
            let t = e.ty.clone();
            match core::unfold_type(&context.subst, t) {
                sp!(_, Type_::Anything) => (),
                mut t => {
                    // report errors if there is an uninferred type argument somewhere
                    type_(context, &mut t);
                }
            }
            e.ty = sp(e.ty.loc, Type_::Anything)
        }
        _ => type_(context, &mut e.ty),
    }
    match &mut e.exp.value {
        E::Use(v) => {
            let from_user = false;
            let var = *v;
            let abs = core::infer_abilities(&context.modules, &context.subst, e.ty.clone());
            e.exp.value = if abs.has_ability_(Ability_::Copy) {
                E::Copy { from_user, var }
            } else {
                E::Move { from_user, var }
            }
        }
        E::Value(sp!(vloc, Value_::InferredNum(v))) => {
            use BuiltinTypeName_ as BT;
            let bt = match e.ty.value.builtin_name() {
                Some(sp!(_, bt)) if bt.is_numeric() => bt,
                _ => {
                    let diag = ice!((
                        e.exp.loc,
                        format!("ICE failed to infer number type for {}", debug_display!(e))
                    ));
                    context.env.add_diag(diag);
                    let _ = std::mem::replace(&mut e.ty.value, Type_::UnresolvedError);
                    let _ = std::mem::replace(&mut e.exp.value, E::UnresolvedError);
                    return;
                }
            };
            let v = *v;
            let u8_max = U256::from(u8::MAX);
            let u16_max = U256::from(u16::MAX);
            let u32_max = U256::from(u32::MAX);
            let u64_max = U256::from(u64::MAX);
            let u128_max = U256::from(u128::MAX);
            let u256_max = U256::max_value();
            let max = match bt {
                BT::U8 => u8_max,
                BT::U16 => u16_max,
                BT::U32 => u32_max,
                BT::U64 => u64_max,
                BT::U128 => u128_max,
                BT::U256 => u256_max,
                BT::Address | BT::Signer | BT::Vector | BT::Bool => unreachable!(),
            };
            let new_exp = if v > max {
                let msg = format!(
                    "Expected a literal of type '{}', but the value is too large.",
                    bt
                );
                let fix_bt = if v > u128_max {
                    BT::U256
                } else if v > u64_max {
                    BT::U128
                } else if v > u32_max {
                    BT::U64
                } else if v > u16_max {
                    BT::U32
                } else {
                    assert!(v > u8_max);
                    BT::U16
                };

                let fix = format!(
                    "Annotating the literal might help inference: '{value}{type}'",
                    value=v,
                    type=fix_bt,
                );
                context.env.add_diag(diag!(
                    TypeSafety::InvalidNum,
                    (e.exp.loc, "Invalid numerical literal"),
                    (e.ty.loc, msg),
                    (e.exp.loc, fix),
                ));
                E::UnresolvedError
            } else {
                let value_ = match bt {
                    BT::U8 => Value_::U8(v.down_cast_lossy()),
                    BT::U16 => Value_::U16(v.down_cast_lossy()),
                    BT::U32 => Value_::U32(v.down_cast_lossy()),
                    BT::U64 => Value_::U64(v.down_cast_lossy()),
                    BT::U128 => Value_::U128(v.down_cast_lossy()),
                    BT::U256 => Value_::U256(v),
                    BT::Address | BT::Signer | BT::Vector | BT::Bool => unreachable!(),
                };
                E::Value(sp(*vloc, value_))
            };
            e.exp.value = new_exp;
        }

        E::Unit { .. }
        | E::Value(_)
        | E::Constant(_, _)
        | E::Move { .. }
        | E::Copy { .. }
        | E::BorrowLocal(_, _)
        | E::Continue(_)
        | E::ErrorConstant(_)
        | E::UnresolvedError => (),

        E::ModuleCall(call) => module_call(context, call),
        E::Builtin(b, args) => {
            builtin_function(context, b);
            exp(context, args);
        }
        E::Vector(_vec_loc, _n, ty_arg, args) => {
            type_(context, ty_arg);
            exp(context, args);
        }

        E::IfElse(eb, et, ef) => {
            exp(context, eb);
            exp(context, et);
            exp(context, ef);
        }
        E::While(_, eb, eloop) => {
            exp(context, eb);
            exp(context, eloop);
        }
        E::Loop { body: eloop, .. } => exp(context, eloop),
        E::NamedBlock(_, seq) => sequence(context, seq),
        E::Block(seq) => sequence(context, seq),
        E::Assign(assigns, tys, er) => {
            lvalues(context, assigns);
            expected_types(context, tys);
            exp(context, er);
        }

        E::Return(er)
        | E::Abort(er)
        | E::Give(_, er)
        | E::Dereference(er)
        | E::UnaryExp(_, er)
        | E::Borrow(_, er, _)
        | E::TempBorrow(_, er) => exp(context, er),
        E::Mutate(el, er) => {
            exp(context, el);
            exp(context, er)
        }
        E::BinopExp(el, _, operand_ty, er) => {
            exp(context, el);
            exp(context, er);
            type_(context, operand_ty);
        }

        E::Pack(_, _, bs, fields) => {
            types(context, bs);
            for (_, _, (_, (bt, fe))) in fields.iter_mut() {
                type_(context, bt);
                exp(context, fe)
            }
        }
        E::ExpList(el) => exp_list(context, el),
        E::Cast(el, rhs_ty) | E::Annotate(el, rhs_ty) => {
            exp(context, el);
            type_(context, rhs_ty);
        }
    }
}

fn lvalues(context: &mut Context, binds: &mut T::LValueList) {
    for b in &mut binds.value {
        lvalue(context, b)
    }
}

fn lvalue(context: &mut Context, b: &mut T::LValue) {
    use T::LValue_ as L;
    match &mut b.value {
        L::Ignore => (),
        L::Var {
            ty,
            unused_binding: true,
            ..
        } => {
            // silence type inference error for unused bindings
            if let Type_::Var(tvar) = &ty.value {
                let ty_tvar = sp(ty.loc, Type_::Var(*tvar));
                let replacement = core::unfold_type(&context.subst, ty_tvar);
                if let sp!(_, Type_::Anything) = replacement {
                    b.value = L::Ignore;
                    return;
                }
            }
            type_(context, ty);
        }
        L::Var { ty, .. } => {
            type_(context, ty);
        }
        L::BorrowUnpack(_, _, _, bts, fields) | L::Unpack(_, _, bts, fields) => {
            types(context, bts);
            for (_, _, (_, (bt, innerb))) in fields.iter_mut() {
                type_(context, bt);
                lvalue(context, innerb)
            }
        }
    }
}

fn module_call(context: &mut Context, call: &mut T::ModuleCall) {
    types(context, &mut call.type_arguments);
    exp(context, &mut call.arguments);
    types(context, &mut call.parameter_types)
}

fn builtin_function(context: &mut Context, b: &mut T::BuiltinFunction) {
    use T::BuiltinFunction_ as B;
    match &mut b.value {
        B::Freeze(bt) => {
            type_(context, bt);
        }
        B::Assert(_) => (),
    }
}

fn exp_list(context: &mut Context, items: &mut Vec<T::ExpListItem>) {
    for item in items {
        exp_list_item(context, item)
    }
}

fn exp_list_item(context: &mut Context, item: &mut T::ExpListItem) {
    use T::ExpListItem as I;
    match item {
        I::Single(e, st) => {
            exp(context, e);
            type_(context, st);
        }
        I::Splat(_, e, ss) => {
            exp(context, e);
            types(context, ss);
        }
    }
}
