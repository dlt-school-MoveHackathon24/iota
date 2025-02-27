// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import type { BcsType, BcsTypeOptions } from '@iota/bcs';
import {
    bcs,
    BCS as BcsRegistry,
    fromB58,
    fromB64,
    fromHEX,
    getIotaMoveConfig,
    toB58,
    toB64,
    toHEX,
} from '@iota/bcs';

import type { MoveCallTransaction } from '../transactions/Transactions.js';
import { normalizeIotaAddress, IOTA_ADDRESS_LENGTH } from '../utils/iota-types.js';
import { TypeTagSerializer } from './type-tag-serializer.js';

export { TypeTagSerializer } from './type-tag-serializer.js';

/**
 * A reference to a shared object.
 */
export type SharedObjectRef = {
    /** Hex code as string representing the object id */
    objectId: string;

    /** The version the object was shared at */
    initialSharedVersion: number | string;

    /** Whether reference is mutable */
    mutable: boolean;
};

export type IotaObjectRef = {
    /** Base64 string representing the object digest */
    objectId: string;
    /** Object version */
    version: number | string | bigint;
    /** Hex code as string representing the object id */
    digest: string;
};

/**
 * An object argument.
 */
export type ObjectArg =
    | { ImmOrOwned: IotaObjectRef }
    | { Shared: SharedObjectRef }
    | { Receiving: IotaObjectRef };

/**
 * A pure argument.
 */
export type PureArg = { Pure: ArrayLike<number> };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function isPureArg(arg: any): arg is PureArg {
    return (arg as PureArg).Pure !== undefined;
}

/**
 * An argument for the transaction. It is a 'meant' enum which expects to have
 * one of the optional properties. If not, the BCS error will be thrown while
 * attempting to form a transaction.
 *
 * Example:
 * ```js
 * let arg1: CallArg = { Object: { Shared: {
 *   objectId: '5460cf92b5e3e7067aaace60d88324095fd22944',
 *   initialSharedVersion: 1,
 *   mutable: true,
 * } } };
 * let arg2: CallArg = { Pure: bcs.ser(BCS.STRING, 100000).toBytes() };
 * let arg3: CallArg = { Object: { ImmOrOwned: {
 *   objectId: '4047d2e25211d87922b6650233bd0503a6734279',
 *   version: 1,
 *   digest: 'bCiANCht4O9MEUhuYjdRCqRPZjr2rJ8MfqNiwyhmRgA='
 * } } };
 * ```
 *
 * For `Pure` arguments BCS is required. You must encode the values with BCS according
 * to the type required by the called function. Pure accepts only serialized values
 */
export type CallArg = PureArg | { Object: ObjectArg };

/**
 * Kind of a TypeTag which is represented by a Move type identifier.
 */
export type StructTag = {
    address: string;
    module: string;
    name: string;
    typeParams: TypeTag[];
};

/**
 * Iota TypeTag object. A decoupled `0x...::module::Type<???>` parameter.
 */
export type TypeTag =
    | { bool: null | true }
    | { u8: null | true }
    | { u64: null | true }
    | { u128: null | true }
    | { address: null | true }
    | { signer: null | true }
    | { vector: TypeTag }
    | { struct: StructTag }
    | { u16: null | true }
    | { u32: null | true }
    | { u256: null | true };

// ========== TransactionData ===========

/**
 * The GasData to be used in the transaction.
 */
export type GasData = {
    payment: IotaObjectRef[];
    owner: string; // Gas Object's owner
    price: number;
    budget: number;
};

/**
 * TransactionExpiration
 *
 * Indications the expiration time for a transaction.
 */
export type TransactionExpiration = { None: null } | { Epoch: number };

const bcsRegistry = new BcsRegistry({
    ...getIotaMoveConfig(),
    types: {
        enums: {
            'Option<T>': {
                None: null,
                Some: 'T',
            },
        },
    },
});

function unsafe_u64(options?: BcsTypeOptions<number>) {
    return bcs
        .u64({
            name: 'unsafe_u64',
            ...(options as object),
        })
        .transform({
            input: (val: number) => val,
            output: (val) => Number(val),
        });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function optionEnum<T extends BcsType<any, any>>(type: T) {
    return bcs.enum('Option', {
        None: null,
        Some: type,
    });
}

/**
 * Wrapper around Enum, which transforms any `T` into an object with `kind` property:
 * @example
 * ```
 * let bcsEnum = { TransferObjects: { objects: [], address: ... } }
 * // becomes
 * let translatedEnum = { kind: 'TransferObjects', objects: [], address: ... };
 * ```
 */
function enumKind<T extends object, Input extends object>(type: BcsType<T, Input>) {
    type Merge<T> = T extends infer U ? { [K in keyof U]: U[K] } : never;
    type EnumKindTransform<T> = T extends infer U
        ? Merge<(U[keyof U] extends null | boolean ? object : U[keyof U]) & { kind: keyof U }>
        : never;

    return type.transform({
        input: (val: EnumKindTransform<Input>) =>
            ({
                [val.kind]: val,
            }) as Input,
        output: (val) => {
            const key = Object.keys(val)[0] as keyof T;

            return { kind: key, ...val[key] } as EnumKindTransform<T>;
        },
    });
}

const Address = bcs.bytes(IOTA_ADDRESS_LENGTH).transform({
    input: (val: string | Uint8Array) =>
        typeof val === 'string' ? fromHEX(normalizeIotaAddress(val)) : val,
    output: (val) => normalizeIotaAddress(toHEX(val)),
});

const ObjectDigest = bcs.vector(bcs.u8()).transform({
    name: 'ObjectDigest',
    input: (value: string) => fromB58(value),
    output: (value) => toB58(new Uint8Array(value)),
});

const IotaObjectRef = bcs.struct('IotaObjectRef', {
    objectId: Address,
    version: bcs.u64(),
    digest: ObjectDigest,
});

const SharedObjectRef = bcs.struct('SharedObjectRef', {
    objectId: Address,
    initialSharedVersion: bcs.u64(),
    mutable: bcs.bool(),
});

const ObjectArg = bcs.enum('ObjectArg', {
    ImmOrOwned: IotaObjectRef,
    Shared: SharedObjectRef,
    Receiving: IotaObjectRef,
});

const CallArg = bcs.enum('CallArg', {
    Pure: bcs.vector(bcs.u8()),
    Object: ObjectArg,
    ObjVec: bcs.vector(ObjectArg),
});

const TypeTag: BcsType<TypeTag> = bcs.enum('TypeTag', {
    bool: null,
    u8: null,
    u64: null,
    u128: null,
    address: null,
    signer: null,
    vector: bcs.lazy(() => TypeTag),
    struct: bcs.lazy(() => StructTag),
    u16: null,
    u32: null,
    u256: null,
}) as never;

const Argument = enumKind(
    bcs.enum('Argument', {
        GasCoin: null,
        Input: bcs.struct('Input', { index: bcs.u16() }),
        Result: bcs.struct('Result', { index: bcs.u16() }),
        NestedResult: bcs.struct('NestedResult', { index: bcs.u16(), resultIndex: bcs.u16() }),
    }),
);

/** Custom serializer for decoding package, module, function easier */
const ProgrammableMoveCall = bcs
    .struct('ProgrammableMoveCall', {
        package: Address,
        module: bcs.string(),
        function: bcs.string(),
        type_arguments: bcs.vector(TypeTag),
        arguments: bcs.vector(Argument),
    })
    .transform({
        input: (data: MoveCallTransaction) => {
            const [pkg, module, fun] = data.target.split('::');
            const type_arguments = data.typeArguments.map((tag) =>
                TypeTagSerializer.parseFromStr(tag, true),
            );

            return {
                package: normalizeIotaAddress(pkg),
                module,
                function: fun,
                type_arguments,
                arguments: data.arguments,
            };
        },
        output: (data) => {
            return {
                target: [data.package, data.module, data.function].join(
                    '::',
                ) as `${string}::${string}::${string}`,
                arguments: data.arguments,
                typeArguments: data.type_arguments.map(TypeTagSerializer.tagToString),
            };
        },
    });

const Transaction = enumKind(
    bcs.enum('Transaction', {
        /**
         * A Move Call - any public Move function can be called via
         * this transaction. The results can be used that instant to pass
         * into the next transaction.
         */
        MoveCall: ProgrammableMoveCall,
        /**
         * Transfer vector of objects to a receiver.
         */
        TransferObjects: bcs.struct('TransferObjects', {
            objects: bcs.vector(Argument),
            address: Argument,
        }),
        /**
         * Split `amount` from a `coin`.
         */
        SplitCoins: bcs.struct('SplitCoins', { coin: Argument, amounts: bcs.vector(Argument) }),
        /**
         * Merge Vector of Coins (`sources`) into a `destination`.
         */
        MergeCoins: bcs.struct('MergeCoins', {
            destination: Argument,
            sources: bcs.vector(Argument),
        }),
        /**
         * Publish a Move module.
         */
        Publish: bcs.struct('Publish', {
            modules: bcs.vector(bcs.vector(bcs.u8())),
            dependencies: bcs.vector(Address),
        }),
        /**
         * Build a vector of objects using the input arguments.
         * It is impossible to construct a `vector<T: key>` otherwise,
         * so this call serves a utility function.
         */
        MakeMoveVec: bcs.struct('MakeMoveVec', {
            type: optionEnum(TypeTag),
            objects: bcs.vector(Argument),
        }),
        /**  */
        Upgrade: bcs.struct('Upgrade', {
            modules: bcs.vector(bcs.vector(bcs.u8())),
            dependencies: bcs.vector(Address),
            packageId: Address,
            ticket: Argument,
        }),
    }),
);

const ProgrammableTransaction = bcs.struct('ProgrammableTransaction', {
    inputs: bcs.vector(CallArg),
    transactions: bcs.vector(Transaction),
});

const TransactionKind = bcs.enum('TransactionKind', {
    ProgrammableTransaction: ProgrammableTransaction,
    ChangeEpoch: null,
    Genesis: null,
    ConsensusCommitPrologue: null,
});

const TransactionExpiration = bcs.enum('TransactionExpiration', {
    None: null,
    Epoch: unsafe_u64(),
});

const StructTag = bcs.struct('StructTag', {
    address: Address,
    module: bcs.string(),
    name: bcs.string(),
    typeParams: bcs.vector(TypeTag),
});

const GasData = bcs.struct('GasData', {
    payment: bcs.vector(IotaObjectRef),
    owner: Address,
    price: bcs.u64(),
    budget: bcs.u64(),
});

const TransactionDataV1 = bcs.struct('TransactionDataV1', {
    kind: TransactionKind,
    sender: Address,
    gasData: GasData,
    expiration: TransactionExpiration,
});

const TransactionData = bcs.enum('TransactionData', {
    V1: TransactionDataV1,
});

const IntentScope = bcs.enum('IntentScope', {
    TransactionData: null,
    TransactionEffects: null,
    CheckpointSummary: null,
    PersonalMessage: null,
});

const IntentVersion = bcs.enum('IntentVersion', {
    V0: null,
});

const AppId = bcs.enum('AppId', {
    Iota: null,
});

const Intent = bcs.struct('Intent', {
    scope: IntentScope,
    version: IntentVersion,
    appId: AppId,
});

const IntentMessage = bcs.generic(['T'], (T) =>
    bcs.struct('IntentMessage<T>', {
        intent: Intent,
        value: T,
    }),
);

const CompressedSignature = bcs.enum('CompressedSignature', {
    ED25519: bcs.fixedArray(64, bcs.u8()),
    Secp256k1: bcs.fixedArray(64, bcs.u8()),
    Secp256r1: bcs.fixedArray(64, bcs.u8()),
    ZkLogin: bcs.vector(bcs.u8()),
});

const PublicKey = bcs.enum('PublicKey', {
    ED25519: bcs.fixedArray(32, bcs.u8()),
    Secp256k1: bcs.fixedArray(33, bcs.u8()),
    Secp256r1: bcs.fixedArray(33, bcs.u8()),
    ZkLogin: bcs.vector(bcs.u8()),
});

const MultiSigPkMap = bcs.struct('MultiSigPkMap', {
    pubKey: PublicKey,
    weight: bcs.u8(),
});

const MultiSigPublicKey = bcs.struct('MultiSigPublicKey', {
    pk_map: bcs.vector(MultiSigPkMap),
    threshold: bcs.u16(),
});

const MultiSig = bcs.struct('MultiSig', {
    sigs: bcs.vector(CompressedSignature),
    bitmap: bcs.u16(),
    multisig_pk: MultiSigPublicKey,
});

const base64String = bcs.vector(bcs.u8()).transform({
    input: (val: string | Uint8Array) => (typeof val === 'string' ? fromB64(val) : val),
    output: (val) => toB64(new Uint8Array(val)),
});

const SenderSignedTransaction = bcs.struct('SenderSignedTransaction', {
    intentMessage: IntentMessage(TransactionData),
    txSignatures: bcs.vector(base64String),
});

const SenderSignedData = bcs.vector(SenderSignedTransaction, {
    name: 'SenderSignedData',
});

const iotaBcs = {
    ...bcs,
    U8: bcs.u8(),
    U16: bcs.u16(),
    U32: bcs.u32(),
    U64: bcs.u64(),
    U128: bcs.u128(),
    U256: bcs.u256(),
    ULEB128: bcs.uleb128(),
    Bool: bcs.bool(),
    String: bcs.string(),
    Address,
    Argument,
    CallArg,
    CompressedSignature,
    GasData,
    MultiSig,
    MultiSigPkMap,
    MultiSigPublicKey,
    ObjectArg,
    ObjectDigest,
    ProgrammableMoveCall,
    ProgrammableTransaction,
    PublicKey,
    SenderSignedData,
    SenderSignedTransaction,
    SharedObjectRef,
    StructTag,
    IotaObjectRef,
    Transaction,
    TransactionData,
    TransactionDataV1,
    TransactionExpiration,
    TransactionKind,
    TypeTag,

    // preserve backwards compatibility with old bcs export
    ser: bcsRegistry.ser.bind(bcsRegistry),
    de: bcsRegistry.de.bind(bcsRegistry),
    getTypeInterface: bcsRegistry.getTypeInterface.bind(bcsRegistry),
    hasType: bcsRegistry.hasType.bind(bcsRegistry),
    parseTypeName: bcsRegistry.parseTypeName.bind(bcsRegistry),
    registerAddressType: bcsRegistry.registerAddressType.bind(bcsRegistry),
    registerAlias: bcsRegistry.registerAlias.bind(bcsRegistry),
    registerBcsType: bcsRegistry.registerBcsType.bind(bcsRegistry),
    registerEnumType: bcsRegistry.registerEnumType.bind(bcsRegistry),
    registerStructType: bcsRegistry.registerStructType.bind(bcsRegistry),
    registerType: bcsRegistry.registerType.bind(bcsRegistry),
    types: bcsRegistry.types,
};

bcsRegistry.registerBcsType('utf8string', () => bcs.string({ name: 'utf8string' }));
bcsRegistry.registerBcsType('unsafe_u64', () => unsafe_u64());
bcsRegistry.registerBcsType('enumKind', (T) => enumKind(T));

[
    Address,
    Argument,
    CallArg,
    CompressedSignature,
    GasData,
    MultiSig,
    MultiSigPkMap,
    MultiSigPublicKey,
    ObjectArg,
    ObjectDigest,
    ProgrammableMoveCall,
    ProgrammableTransaction,
    PublicKey,
    SenderSignedData,
    SharedObjectRef,
    StructTag,
    IotaObjectRef,
    Transaction,
    TransactionData,
    TransactionDataV1,
    TransactionExpiration,
    TransactionKind,
    TypeTag,
].forEach((type) => {
    bcsRegistry.registerBcsType(type.name, () => type);
});

export { iotaBcs as bcs, bcsRegistry };
