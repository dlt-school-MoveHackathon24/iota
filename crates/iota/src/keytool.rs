// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    fmt::{Debug, Display, Formatter},
    fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use aws_sdk_kms::{
    primitives::Blob,
    types::{MessageType, SigningAlgorithmSpec},
    Client as KmsClient,
};
use bip32::DerivationPath;
use clap::*;
use fastcrypto::{
    ed25519::Ed25519KeyPair,
    encoding::{Base64, Encoding, Hex},
    hash::HashFunction,
    secp256k1::recoverable::Secp256k1Sig,
    traits::{KeyPair, ToFromBytes},
};
use iota_keys::{
    key_derive::generate_new_key,
    keypair_file::{
        read_authority_keypair_from_file, read_keypair_from_file, write_authority_keypair_to_file,
        write_keypair_to_file,
    },
    keystore::{AccountKeystore, Keystore},
};
use iota_types::{
    base_types::IotaAddress,
    crypto::{
        get_authority_key_pair, DefaultHash, EncodeDecodeBase64, IotaKeyPair, PublicKey,
        SignatureScheme,
    },
    error::IotaResult,
    multisig::{MultiSig, MultiSigPublicKey, ThresholdUnit, WeightUnit},
    signature::{AuthenticatorTrait, GenericSignature, VerifyParams},
    transaction::{TransactionData, TransactionDataAPI},
};
use json_to_table::{json_to_table, Orientation};
use serde::Serialize;
use serde_json::json;
use shared_crypto::intent::{Intent, IntentMessage};
use tabled::{
    builder::Builder,
    settings::{object::Rows, Modify, Rotate, Width},
};
use tracing::info;

use crate::key_identity::{get_identity_address_from_keystore, KeyIdentity};
#[cfg(test)]
#[path = "unit_tests/keytool_tests.rs"]
mod keytool_tests;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
#[clap(rename_all = "kebab-case")]
pub enum KeyToolCommand {
    /// Update an old alias to a new one.
    /// If a new alias is not provided, a random one will be generated.
    #[clap(name = "update-alias")]
    Alias {
        old_alias: String,
        /// The alias must start with a letter and can contain only letters,
        /// digits, dots, hyphens (-), or underscores (_).
        new_alias: Option<String>,
    },
    /// Convert private key in Hex or Base64 to new format (Bech32
    /// encoded 33 byte flag || private key starting with "iotaprivkey").
    /// Hex private key format import and export are both deprecated in
    /// Iota Wallet and Iota CLI Keystore. Use `iota keytool import` if you
    /// wish to import a key to Iota Keystore.
    Convert { value: String },
    /// Given a Base64 encoded transaction bytes, decode its components. If a
    /// signature is provided, verify the signature against the transaction
    /// and output the result.
    DecodeOrVerifyTx {
        #[clap(long)]
        tx_bytes: String,
        #[clap(long)]
        sig: Option<GenericSignature>,
    },
    /// Given a Base64 encoded MultiSig signature, decode its components.
    /// If tx_bytes is passed in, verify the multisig.
    DecodeMultiSig {
        #[clap(long)]
        multisig: MultiSig,
        #[clap(long)]
        tx_bytes: Option<String>,
    },
    /// Generate a new keypair with key scheme flag {ed25519 | secp256k1 |
    /// secp256r1} with optional derivation path, default to
    /// m/44'/4218'/0'/0'/0' for ed25519 or m/54'/4218'/0'/0/0 for secp256k1
    /// or m/74'/4218'/0'/0/0 for secp256r1. Word length can be { word12 |
    /// word15 | word18 | word21 | word24} default to word12
    /// if not specified.
    ///
    /// The keypair file is output to the current directory. The content of the
    /// file is a Base64 encoded string of 33-byte `flag || privkey`.
    ///
    /// Use `iota client new-address` if you want to generate and save the key
    /// into iota.keystore.
    Generate {
        key_scheme: SignatureScheme,
        derivation_path: Option<DerivationPath>,
        word_length: Option<String>,
    },

    /// Add a new key to Iota CLI Keystore using either the input mnemonic
    /// phrase or a Bech32 encoded 33-byte `flag || privkey` starting with
    /// "iotaprivkey", the key scheme flag {ed25519 | secp256k1 | secp256r1}
    /// and an optional derivation path, default to m/44'/4218'/0'/0'/0' for
    /// ed25519 or m/54'/4218'/0'/0/0 for secp256k1 or m/74'/4218'/0'/0/0
    /// for secp256r1. Supports mnemonic phrase of word length 12, 15,
    /// 18, 21, 24. Set an alias for the key with the --alias flag. If no alias
    /// is provided, the tool will automatically generate one.
    Import {
        /// Sets an alias for this address. The alias must start with a letter
        /// and can contain only letters, digits, hyphens (-), or underscores
        /// (_).
        #[clap(long)]
        alias: Option<String>,
        input_string: String,
        key_scheme: SignatureScheme,
        derivation_path: Option<DerivationPath>,
    },
    /// Output the private key of the given key identity in Iota CLI Keystore as
    /// Bech32 encoded string starting with `iotaprivkey`.
    Export {
        #[clap(long)]
        key_identity: KeyIdentity,
    },
    /// List all keys by its Iota address, Base64 encoded public key, key scheme
    /// name in iota.keystore.
    List {
        /// Sort by alias
        #[clap(long, short = 's')]
        sort_by_alias: bool,
    },
    /// This reads the content at the provided file path. The accepted format
    /// can be [enum IotaKeyPair] (Base64 encoded of 33-byte `flag ||
    /// privkey`) or `type AuthorityKeyPair` (Base64 encoded `privkey`).
    /// This prints out the account keypair as Base64 encoded `flag || privkey`,
    /// the network keypair, worker keypair, protocol keypair as Base64 encoded
    /// `privkey`.
    LoadKeypair { file: PathBuf },
    /// To MultiSig Iota Address. Pass in a list of all public keys `flag || pk`
    /// in Base64. See `keytool list` for example public keys.
    MultiSigAddress {
        #[clap(long)]
        threshold: ThresholdUnit,
        #[clap(long, num_args(1..))]
        pks: Vec<PublicKey>,
        #[clap(long, num_args(1..))]
        weights: Vec<WeightUnit>,
    },
    /// Provides a list of participating signatures (`flag || sig || pk` encoded
    /// in Base64), threshold, a list of all public keys and a list of their
    /// weights that define the MultiSig address. Returns a valid MultiSig
    /// signature and its sender address. The result can be used as
    /// signature field for `iota client execute-signed-tx`. The sum
    /// of weights of all signatures must be >= the threshold.
    ///
    /// The order of `sigs` must be the same as the order of `pks`.
    /// e.g. for [pk1, pk2, pk3, pk4, pk5], [sig1, sig2, sig5] is valid, but
    /// [sig2, sig1, sig5] is invalid.
    MultiSigCombinePartialSig {
        #[clap(long, num_args(1..))]
        sigs: Vec<GenericSignature>,
        #[clap(long, num_args(1..))]
        pks: Vec<PublicKey>,
        #[clap(long, num_args(1..))]
        weights: Vec<WeightUnit>,
        #[clap(long)]
        threshold: ThresholdUnit,
    },

    /// Read the content at the provided file path. The accepted format can be
    /// [enum IotaKeyPair] (Base64 encoded of 33-byte `flag || privkey`) or
    /// `type AuthorityKeyPair` (Base64 encoded `privkey`). It prints its
    /// Base64 encoded public key and the key scheme flag.
    Show { file: PathBuf },
    /// Create signature using the private key for for the given address (or its
    /// alias) in iota keystore. Any signature commits to a [struct
    /// IntentMessage] consisting of the Base64 encoded of the BCS
    /// serialized transaction bytes itself and its intent. If intent is absent,
    /// default will be used.
    Sign {
        #[clap(long)]
        address: KeyIdentity,
        #[clap(long)]
        data: String,
        #[clap(long)]
        intent: Option<Intent>,
    },
    /// Creates a signature by leveraging AWS KMS. Pass in a key-id to leverage
    /// Amazon KMS to sign a message and the base64 pubkey.
    /// Generate PubKey from pem using iotaledger/base64pemkey
    /// Any signature commits to a [struct IntentMessage] consisting of the
    /// Base64 encoded of the BCS serialized transaction bytes itself and
    /// its intent. If intent is absent, default will be used.
    SignKMS {
        #[clap(long)]
        data: String,
        #[clap(long)]
        keyid: String,
        #[clap(long)]
        intent: Option<Intent>,
        #[clap(long)]
        base64pk: String,
    },
    /// This takes [enum IotaKeyPair] of Base64 encoded of 33-byte `flag ||
    /// privkey`). It outputs the keypair into a file at the current
    /// directory where the address is the filename, and prints out its Iota
    /// address, Base64 encoded public key, the key scheme, and the key scheme
    /// flag.
    Unpack { keypair: String },
    // Commented for now: https://github.com/iotaledger/iota/issues/1777
    // /// Given the max_epoch, generate an OAuth url, ask user to paste the
    // /// redirect with id_token, call salt server, then call the prover server,
    // /// create a test transaction, use the ephemeral key to sign and execute it
    // /// by assembling to a serialized zkLogin signature.
    // ZkLoginSignAndExecuteTx {
    //     #[clap(long)]
    //     max_epoch: EpochId,
    //     #[clap(long, default_value = "devnet")]
    //     network: String,
    //     #[clap(long, default_value = "true")]
    //     fixed: bool, // if true, use a fixed kp generated from [0; 32] seed.
    //     #[clap(long, default_value = "true")]
    //     test_multisig: bool, // if true, use a multisig address with zklogin and a traditional
    // kp.     #[clap(long, default_value = "false")]
    //     sign_with_sk: bool, /* if true, execute tx with the traditional sig (in the multisig),
    //                          * otherwise with the zklogin sig. */
    // },

    // /// A workaround to the above command because sometimes token pasting does
    // /// not work (for Facebook). All the inputs required here are printed from
    // /// the command above.
    // ZkLoginEnterToken {
    //     #[clap(long)]
    //     parsed_token: String,
    //     #[clap(long)]
    //     max_epoch: EpochId,
    //     #[clap(long)]
    //     jwt_randomness: String,
    //     #[clap(long)]
    //     kp_bigint: String,
    //     #[clap(long)]
    //     ephemeral_key_identifier: IotaAddress,
    //     #[clap(long, default_value = "devnet")]
    //     network: String,
    //     #[clap(long, default_value = "true")]
    //     test_multisig: bool,
    //     #[clap(long, default_value = "false")]
    //     sign_with_sk: bool,
    // },

    // /// Given a zkLogin signature, parse it if valid. If `bytes` provided,
    // /// parse it as either as TransactionData or PersonalMessage based on
    // /// `intent_scope`. It verifies the zkLogin signature based its latest
    // /// JWK fetched. Example request: iota keytool zk-login-sig-verify --sig
    // /// $SERIALIZED_ZKLOGIN_SIG --bytes $BYTES --intent-scope 0 --network devnet
    // /// --curr-epoch 10
    // ZkLoginSigVerify {
    //     /// The Base64 of the serialized zkLogin signature.
    //     #[clap(long)]
    //     sig: String,
    //     /// The Base64 of the BCS encoded TransactionData or PersonalMessage.
    //     #[clap(long)]
    //     bytes: Option<String>,
    //     /// Either 0 for TransactionData or 3 for PersonalMessage.
    //     #[clap(long)]
    //     intent_scope: u8,
    //     /// The current epoch for the network to verify the signature's
    //     /// max_epoch against.
    //     #[clap(long)]
    //     curr_epoch: Option<EpochId>,
    //     /// The network to verify the signature for, determines ZkLoginEnv.
    //     #[clap(long, default_value = "devnet")]
    //     network: String,
    // },

    // /// TESTING ONLY: Given a string of data, sign with the fixed dev-only
    // /// ephemeral key and output a zkLogin signature with a fixed dev-only
    // /// proof with fixed max epoch 10.
    // ZkLoginInsecureSignPersonalMessage {
    //     /// The string of data to sign.
    //     #[clap(long)]
    //     data: String,
    // },
}

// Command Output types
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AliasUpdate {
    old_alias: String,
    new_alias: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMultiSig {
    public_base64_key: String,
    sig_base64: String,
    weight: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedMultiSigOutput {
    multisig_address: IotaAddress,
    participating_keys_signatures: Vec<DecodedMultiSig>,
    pub_keys: Vec<MultiSigOutput>,
    threshold: usize,
    transaction_result: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeOrVerifyTxOutput {
    tx: TransactionData,
    result: Option<IotaResult>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    alias: Option<String>,
    iota_address: IotaAddress,
    public_base64_key: String,
    key_scheme: String,
    flag: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    mnemonic: Option<String>,
    peer_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedKey {
    exported_private_key: String,
    key: Key,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeypairData {
    account_keypair: String,
    network_keypair: Option<String>,
    worker_keypair: Option<String>,
    key_scheme: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigAddress {
    multisig_address: String,
    multisig: Vec<MultiSigOutput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigCombinePartialSig {
    multisig_address: IotaAddress,
    multisig_parsed: GenericSignature,
    multisig_serialized: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSigOutput {
    address: IotaAddress,
    public_base64_key: String,
    weight: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertOutput {
    bech32_with_flag: String, // latest Iota Keystore and Iota Wallet import/export format
    base64_with_flag: String, // Iota Keystore storage format
    hex_without_flag: String, // Legacy Iota Wallet format
    scheme: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivateKeyBase64 {
    base64: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedSig {
    serialized_sig_base64: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignData {
    iota_address: IotaAddress,
    // Base64 encoded string of serialized transaction data.
    raw_tx_data: String,
    // Intent struct used, see [struct Intent] for field definitions.
    intent: Intent,
    // Base64 encoded [struct IntentMessage] consisting of (intent || message)
    // where message can be `TransactionData` etc.
    raw_intent_msg: String,
    // Base64 encoded blake2b hash of the intent message, this is what the signature commits to.
    digest: String,
    // Base64 encoded `flag || signature || pubkey` for a complete
    // serialized Iota signature to be send for executing the transaction.
    iota_signature: String,
}

// #[derive(Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ZkLoginSignAndExecuteTx {
//     tx_digest: String,
// }

// #[derive(Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ZkLoginSigVerifyResponse {
//     data: Option<String>,
//     parsed: Option<String>,
//     jwks: Option<String>,
//     res: Option<IotaResult>,
// }

// #[derive(Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ZkLoginInsecureSignPersonalMessage {
//     sig: String,
//     bytes: String,
// }

#[derive(Serialize)]
#[serde(untagged)]
pub enum CommandOutput {
    Alias(AliasUpdate),
    Convert(ConvertOutput),
    DecodeMultiSig(DecodedMultiSigOutput),
    DecodeOrVerifyTx(DecodeOrVerifyTxOutput),
    Error(String),
    Generate(Key),
    Import(Key),
    Export(ExportedKey),
    List(Vec<Key>),
    LoadKeypair(KeypairData),
    MultiSigAddress(MultiSigAddress),
    MultiSigCombinePartialSig(MultiSigCombinePartialSig),
    PrivateKeyBase64(PrivateKeyBase64),
    Show(Key),
    Sign(SignData),
    SignKMS(SerializedSig),
    // ZkLoginSignAndExecuteTx(ZkLoginSignAndExecuteTx),
    // ZkLoginInsecureSignPersonalMessage(ZkLoginInsecureSignPersonalMessage),
    // ZkLoginSigVerify(ZkLoginSigVerifyResponse),
}

impl KeyToolCommand {
    pub async fn execute(self, keystore: &mut Keystore) -> Result<CommandOutput, anyhow::Error> {
        let cmd_result = Ok(match self {
            KeyToolCommand::Alias {
                old_alias,
                new_alias,
            } => {
                let new_alias = keystore.update_alias(&old_alias, new_alias.as_deref())?;
                CommandOutput::Alias(AliasUpdate {
                    old_alias,
                    new_alias,
                })
            }
            KeyToolCommand::Convert { value } => {
                let result = convert_private_key_to_bech32(value)?;
                CommandOutput::Convert(result)
            }

            KeyToolCommand::DecodeMultiSig { multisig, tx_bytes } => {
                let pks = multisig.get_pk().pubkeys();
                let sigs = multisig.get_sigs();
                let bitmap = multisig.get_indices()?;
                let address = IotaAddress::from(multisig.get_pk());

                let pub_keys = pks
                    .iter()
                    .map(|(pk, w)| MultiSigOutput {
                        address: (pk).into(),
                        public_base64_key: pk.encode_base64(),
                        weight: *w,
                    })
                    .collect::<Vec<MultiSigOutput>>();

                let threshold = *multisig.get_pk().threshold() as usize;

                let mut output = DecodedMultiSigOutput {
                    multisig_address: address,
                    participating_keys_signatures: vec![],
                    pub_keys,
                    threshold,
                    transaction_result: "".to_string(),
                };

                for (sig, i) in sigs.iter().zip(bitmap) {
                    let (pk, w) = pks
                        .get(i as usize)
                        .ok_or(anyhow!("Invalid public keys index".to_string()))?;
                    output.participating_keys_signatures.push(DecodedMultiSig {
                        public_base64_key: pk.encode_base64().clone(),
                        sig_base64: Base64::encode(sig.as_ref()),
                        weight: w.to_string(),
                    })
                }

                if tx_bytes.is_some() {
                    let tx_bytes = Base64::decode(&tx_bytes.unwrap())
                        .map_err(|e| anyhow!("Invalid base64 tx bytes: {:?}", e))?;
                    let tx_data: TransactionData = bcs::from_bytes(&tx_bytes)?;
                    let s = GenericSignature::MultiSig(multisig);
                    let res = s.verify_authenticator(
                        &IntentMessage::new(Intent::iota_transaction(), tx_data),
                        address,
                        None,
                        &VerifyParams::default(),
                    );
                    output.transaction_result = format!("{:?}", res);
                };

                CommandOutput::DecodeMultiSig(output)
            }

            KeyToolCommand::DecodeOrVerifyTx { tx_bytes, sig } => {
                let tx_bytes = Base64::decode(&tx_bytes)
                    .map_err(|e| anyhow!("Invalid base64 key: {:?}", e))?;
                let tx_data: TransactionData = bcs::from_bytes(&tx_bytes)?;
                match sig {
                    None => CommandOutput::DecodeOrVerifyTx(DecodeOrVerifyTxOutput {
                        tx: tx_data,
                        result: None,
                    }),
                    Some(s) => {
                        let res = s.verify_authenticator(
                            &IntentMessage::new(Intent::iota_transaction(), tx_data.clone()),
                            tx_data.sender(),
                            None,
                            &VerifyParams::default(),
                        );
                        CommandOutput::DecodeOrVerifyTx(DecodeOrVerifyTxOutput {
                            tx: tx_data,
                            result: Some(res),
                        })
                    }
                }
            }
            KeyToolCommand::Generate {
                key_scheme,
                derivation_path,
                word_length,
            } => match key_scheme {
                SignatureScheme::BLS12381 => {
                    let (iota_address, kp) = get_authority_key_pair();
                    let file_name = format!("bls-{iota_address}.key");
                    write_authority_keypair_to_file(&kp, file_name)?;
                    CommandOutput::Generate(Key {
                        alias: None,
                        iota_address,
                        public_base64_key: kp.public().encode_base64(),
                        key_scheme: key_scheme.to_string(),
                        flag: SignatureScheme::BLS12381.flag(),
                        mnemonic: None,
                        peer_id: None,
                    })
                }
                _ => {
                    let (iota_address, ikp, _scheme, phrase) =
                        generate_new_key(key_scheme, derivation_path, word_length)?;
                    let file = format!("{iota_address}.key");
                    write_keypair_to_file(&ikp, file)?;
                    let mut key = Key::from(&ikp);
                    key.mnemonic = Some(phrase);
                    CommandOutput::Generate(key)
                }
            },

            KeyToolCommand::Import {
                alias,
                input_string,
                key_scheme,
                derivation_path,
            } => {
                if Hex::decode(&input_string).is_ok() {
                    return Err(anyhow!(
                        "Iota Keystore and Iota Wallet no longer support importing 
                    private key as Hex, if you are sure your private key is encoded in Hex, use 
                    `iota keytool convert $HEX` to convert first then import the Bech32 encoded 
                    private key starting with `iotaprivkey`."
                    ));
                }

                match IotaKeyPair::decode(&input_string) {
                    Ok(ikp) => {
                        info!("Importing Bech32 encoded private key to keystore");
                        let key = Key::from(&ikp);
                        keystore.add_key(alias, ikp)?;
                        CommandOutput::Import(key)
                    }
                    Err(_) => {
                        info!("Importing mneomonics to keystore");
                        let iota_address = keystore.import_from_mnemonic(
                            &input_string,
                            key_scheme,
                            None,
                            derivation_path,
                        )?;
                        let ikp = keystore.get_key(&iota_address)?;
                        let key = Key::from(ikp);
                        CommandOutput::Import(key)
                    }
                }
            }
            KeyToolCommand::Export { key_identity } => {
                let address = get_identity_address_from_keystore(key_identity, keystore)?;
                let ikp = keystore.get_key(&address)?;
                let key = ExportedKey {
                    exported_private_key: ikp
                        .encode()
                        .map_err(|_| anyhow!("Cannot decode keypair"))?,
                    key: Key::from(ikp),
                };
                CommandOutput::Export(key)
            }
            KeyToolCommand::List { sort_by_alias } => {
                let mut keys = keystore
                    .keys()
                    .into_iter()
                    .map(|pk| {
                        let mut key = Key::from(pk);
                        key.alias = keystore.get_alias_by_address(&key.iota_address).ok();
                        key
                    })
                    .collect::<Vec<Key>>();
                if sort_by_alias {
                    keys.sort_unstable();
                }
                CommandOutput::List(keys)
            }

            KeyToolCommand::LoadKeypair { file } => {
                let output = match read_keypair_from_file(&file) {
                    Ok(keypair) => {
                        // Account keypair is encoded with the key scheme flag {},
                        // and network and worker keypair are not.
                        let network_worker_keypair = match &keypair {
                            IotaKeyPair::Ed25519(kp) => kp.encode_base64(),
                            IotaKeyPair::Secp256k1(kp) => kp.encode_base64(),
                            IotaKeyPair::Secp256r1(kp) => kp.encode_base64(),
                        };
                        KeypairData {
                            account_keypair: keypair.encode_base64(),
                            network_keypair: Some(network_worker_keypair.clone()),
                            worker_keypair: Some(network_worker_keypair),
                            key_scheme: keypair.public().scheme().to_string(),
                        }
                    }
                    Err(_) => {
                        // Authority keypair file is not stored with the flag, it will try read as
                        // BLS keypair..
                        match read_authority_keypair_from_file(&file) {
                            Ok(keypair) => KeypairData {
                                account_keypair: keypair.encode_base64(),
                                network_keypair: None,
                                worker_keypair: None,
                                key_scheme: SignatureScheme::BLS12381.to_string(),
                            },
                            Err(e) => {
                                return Err(anyhow!(format!(
                                    "Failed to read keypair at path {:?} err: {:?}",
                                    file, e
                                )));
                            }
                        }
                    }
                };
                CommandOutput::LoadKeypair(output)
            }

            KeyToolCommand::MultiSigAddress {
                threshold,
                pks,
                weights,
            } => {
                let multisig_pk = MultiSigPublicKey::new(pks.clone(), weights.clone(), threshold)?;
                let address: IotaAddress = (&multisig_pk).into();
                let mut output = MultiSigAddress {
                    multisig_address: address.to_string(),
                    multisig: vec![],
                };

                for (pk, w) in pks.into_iter().zip(weights.into_iter()) {
                    output.multisig.push(MultiSigOutput {
                        address: Into::<IotaAddress>::into(&pk),
                        public_base64_key: pk.encode_base64(),
                        weight: w,
                    });
                }
                CommandOutput::MultiSigAddress(output)
            }

            KeyToolCommand::MultiSigCombinePartialSig {
                sigs,
                pks,
                weights,
                threshold,
            } => {
                let multisig_pk = MultiSigPublicKey::new(pks, weights, threshold)?;
                let address: IotaAddress = (&multisig_pk).into();
                let multisig = MultiSig::combine(sigs, multisig_pk)?;
                let generic_sig: GenericSignature = multisig.into();
                let multisig_serialized = generic_sig.encode_base64();
                CommandOutput::MultiSigCombinePartialSig(MultiSigCombinePartialSig {
                    multisig_address: address,
                    multisig_parsed: generic_sig,
                    multisig_serialized,
                })
            }

            KeyToolCommand::Show { file } => {
                let res = read_keypair_from_file(&file);
                match res {
                    Ok(ikp) => {
                        let key = Key::from(&ikp);
                        CommandOutput::Show(key)
                    }
                    Err(_) => match read_authority_keypair_from_file(&file) {
                        Ok(keypair) => {
                            let public_base64_key = keypair.public().encode_base64();
                            CommandOutput::Show(Key {
                                alias: None, // alias does not get stored in key files
                                iota_address: (keypair.public()).into(),
                                public_base64_key,
                                key_scheme: SignatureScheme::BLS12381.to_string(),
                                flag: SignatureScheme::BLS12381.flag(),
                                peer_id: None,
                                mnemonic: None,
                            })
                        }
                        Err(e) => CommandOutput::Error(format!(
                            "Failed to read keypair at path {:?}, err: {e}",
                            file
                        )),
                    },
                }
            }

            KeyToolCommand::Sign {
                address,
                data,
                intent,
            } => {
                let address = get_identity_address_from_keystore(address, keystore)?;
                let intent = intent.unwrap_or_else(Intent::iota_transaction);
                let intent_clone = intent.clone();
                let msg: TransactionData =
                    bcs::from_bytes(&Base64::decode(&data).map_err(|e| {
                        anyhow!("Cannot deserialize data as TransactionData {:?}", e)
                    })?)?;
                let intent_msg = IntentMessage::new(intent, msg);
                let raw_intent_msg: String = Base64::encode(bcs::to_bytes(&intent_msg)?);
                let mut hasher = DefaultHash::default();
                hasher.update(bcs::to_bytes(&intent_msg)?);
                let digest = hasher.finalize().digest;
                let iota_signature =
                    keystore.sign_secure(&address, &intent_msg.value, intent_msg.intent)?;
                CommandOutput::Sign(SignData {
                    iota_address: address,
                    raw_tx_data: data,
                    intent: intent_clone,
                    raw_intent_msg,
                    digest: Base64::encode(digest),
                    iota_signature: iota_signature.encode_base64(),
                })
            }

            KeyToolCommand::SignKMS {
                data,
                keyid,
                intent,
                base64pk,
            } => {
                // Currently only supports secp256k1 keys
                let pk_owner = PublicKey::decode_base64(&base64pk)
                    .map_err(|e| anyhow!("Invalid base64 key: {:?}", e))?;
                let address_owner = IotaAddress::from(&pk_owner);
                info!("Address For Corresponding KMS Key: {}", address_owner);
                info!("Raw tx_bytes to execute: {}", data);
                let intent = intent.unwrap_or_else(Intent::iota_transaction);
                info!("Intent: {:?}", intent);
                let msg: TransactionData =
                    bcs::from_bytes(&Base64::decode(&data).map_err(|e| {
                        anyhow!("Cannot deserialize data as TransactionData {:?}", e)
                    })?)?;
                let intent_msg = IntentMessage::new(intent, msg);
                info!(
                    "Raw intent message: {:?}",
                    Base64::encode(bcs::to_bytes(&intent_msg)?)
                );
                let mut hasher = DefaultHash::default();
                hasher.update(bcs::to_bytes(&intent_msg)?);
                let digest = hasher.finalize().digest;
                info!("Digest to sign: {:?}", Base64::encode(digest));

                // Set up the KMS client in default region.
                let config = aws_config::from_env().load().await;
                let kms = KmsClient::new(&config);

                // Sign the message, normalize the signature and then compacts it
                // serialize_compact is loaded as bytes for Secp256k1Sinaturere
                let response = kms
                    .sign()
                    .key_id(keyid)
                    .message_type(MessageType::Raw)
                    .message(Blob::new(digest))
                    .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
                    .send()
                    .await?;
                let sig_bytes_der = response
                    .signature
                    .expect("Requires Asymmetric Key Generated in KMS");

                let mut external_sig = Secp256k1Sig::from_der(sig_bytes_der.as_ref())?;
                external_sig.normalize_s();
                let sig_compact = external_sig.serialize_compact();

                let mut serialized_sig = vec![SignatureScheme::Secp256k1.flag()];
                serialized_sig.extend_from_slice(&sig_compact);
                serialized_sig.extend_from_slice(pk_owner.as_ref());
                let serialized_sig = Base64::encode(&serialized_sig);
                CommandOutput::SignKMS(SerializedSig {
                    serialized_sig_base64: serialized_sig,
                })
            }

            KeyToolCommand::Unpack { keypair } => {
                let keypair = IotaKeyPair::decode_base64(&keypair)
                    .map_err(|_| anyhow!("Invalid Base64 encode keypair"))?;

                let key = Key::from(&keypair);
                let path_str = format!("{}.key", key.iota_address).to_lowercase();
                let path = Path::new(&path_str);
                let out_str = format!(
                    "address: {}\nkeypair: {}\nflag: {}",
                    key.iota_address,
                    keypair.encode_base64(),
                    key.flag
                );
                fs::write(path, out_str).unwrap();
                CommandOutput::Show(key)
            } /* KeyToolCommand::ZkLoginInsecureSignPersonalMessage { data } => {
               *     let msg = PersonalMessage {
               *         message: data.as_bytes().to_vec(),
               *     };
               *     let intent_msg = IntentMessage::new(Intent::personal_message(),
               * msg.clone()); */

              /*     let ikp =
               *         IotaKeyPair::Ed25519(Ed25519KeyPair::generate(&mut StdRng::from_seed([0;
               * 32])));     let s = Signature::new_secure(&intent_msg, &ikp); */

              /*     let sig = GenericSignature::ZkLoginAuthenticator(ZkLoginAuthenticator::new(
               *         get_zklogin_inputs(), // this is for the fixed keypair
               *         10,
               *         s,
               *     ));
               *     CommandOutput::ZkLoginInsecureSignPersonalMessage(
               *         ZkLoginInsecureSignPersonalMessage {
               *             sig: Base64::encode(sig.as_bytes()),
               *             bytes: Base64::encode(bcs::to_bytes(&msg).unwrap()),
               *         },
               *     )
               * }
               * KeyToolCommand::ZkLoginSignAndExecuteTx {
               *     max_epoch,
               *     network,
               *     fixed,
               *     test_multisig,
               *     sign_with_sk,
               * } => {
               *     let ikp = if fixed {
               *         IotaKeyPair::Ed25519(Ed25519KeyPair::generate(&mut StdRng::from_seed([0;
               * 32])))     } else {
               *         IotaKeyPair::Ed25519(Ed25519KeyPair::generate(&mut rand::thread_rng()))
               *     };
               *     println!("Ephemeral keypair: {:?}", ikp.encode());
               *     let pk = ikp.public();
               *     let ephemeral_key_identifier: IotaAddress = (&ikp.public()).into();
               *     println!("Ephemeral key identifier: {ephemeral_key_identifier}");
               *     keystore.add_key(None, ikp)?; */

              /*     let mut eph_pk_bytes = vec![pk.flag()];
               *     eph_pk_bytes.extend(pk.as_ref());
               *     let kp_bigint = BigUint::from_bytes_be(&eph_pk_bytes);
               *     println!("Ephemeral pubkey (BigInt): {:?}", kp_bigint); */

              /*     let jwt_randomness = if fixed {
               *         "100681567828351849884072155819400689117".to_string()
               *     } else {
               *         let random_bytes = rand::thread_rng().gen::<[u8; 16]>();
               *         let jwt_random_bytes = BigUint::from_bytes_be(&random_bytes);
               *         jwt_random_bytes.to_string()
               *     };
               *     println!("Jwt randomness: {jwt_randomness}");
               *     let url = get_oidc_url(
               *         OIDCProvider::Google,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "25769832374-famecqrhe2gkebt5fvqms2263046lj96.apps.googleusercontent.
               * com",         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_2 = get_oidc_url(
               *         OIDCProvider::Twitch,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "rs1bh065i9ya4ydvifixl4kss0uhpt",
               *         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_3 = get_oidc_url(
               *         OIDCProvider::Facebook,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "233307156352917",
               *         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_4 = get_oidc_url(
               *         OIDCProvider::Kakao,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "aa6bddf393b54d4e0d42ae0014edfd2f",
               *         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_5 = get_token_exchange_url(
               *         OIDCProvider::Kakao,
               *         "aa6bddf393b54d4e0d42ae0014edfd2f",
               *         "https://iota.io/",
               *         "$YOUR_AUTH_CODE",
               *         "", // not needed
               *     )?;
               *     let url_6 = get_oidc_url(
               *         OIDCProvider::Apple,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "nl.digkas.wallet.client",
               *         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_7 = get_oidc_url(
               *         OIDCProvider::Slack,
               *         &eph_pk_bytes,
               *         max_epoch,
               *         "2426087588661.5742457039348",
               *         "https://iota.io/",
               *         &jwt_randomness,
               *     )?;
               *     let url_8 = get_token_exchange_url(
               *         OIDCProvider::Slack,
               *         "2426087588661.5742457039348",
               *         "https://iota.io/",
               *         "$YOUR_AUTH_CODE",
               *         "39b955a118f2f21110939bf3dff1de90",
               *     )?;
               *     println!("Visit URL (Google): {url}");
               *     println!("Visit URL (Twitch): {url_2}");
               *     println!("Visit URL (Facebook): {url_3}");
               *     println!("Visit URL (Kakao): {url_4}");
               *     println!("Token exchange URL (Kakao): {url_5}");
               *     println!("Visit URL (Apple): {url_6}");
               *     println!("Visit URL (Slack): {url_7}");
               *     println!("Token exchange URL (Slack): {url_8}"); */

              /*     println!(
               *         "Finish login and paste the entire URL here (e.g. https://iota.io/#id_token=...):"
               *     ); */

              /*     let parsed_token = read_cli_line()?;
               *     let tx_digest = perform_zk_login_test_tx(
               *         &parsed_token,
               *         max_epoch,
               *         &jwt_randomness,
               *         &kp_bigint.to_string(),
               *         ephemeral_key_identifier,
               *         keystore,
               *         &network,
               *         test_multisig,
               *         sign_with_sk,
               *     )
               *     .await?;
               *     CommandOutput::ZkLoginSignAndExecuteTx(ZkLoginSignAndExecuteTx { tx_digest
               * }) }
               * KeyToolCommand::ZkLoginEnterToken {
               *     parsed_token,
               *     max_epoch,
               *     jwt_randomness,
               *     kp_bigint,
               *     ephemeral_key_identifier,
               *     network,
               *     test_multisig,
               *     sign_with_sk,
               * } => {
               *     let tx_digest = perform_zk_login_test_tx(
               *         &parsed_token,
               *         max_epoch,
               *         &jwt_randomness,
               *         &kp_bigint,
               *         ephemeral_key_identifier,
               *         keystore,
               *         &network,
               *         test_multisig,
               *         sign_with_sk,
               *     )
               *     .await?;
               *     CommandOutput::ZkLoginSignAndExecuteTx(ZkLoginSignAndExecuteTx { tx_digest
               * }) } */

              /* KeyToolCommand::ZkLoginSigVerify {
               *     sig,
               *     bytes,
               *     intent_scope,
               *     curr_epoch,
               *     network,
               * } => {
               *     match GenericSignature::from_bytes(
               *         &Base64::decode(&sig).map_err(|e| anyhow!("Invalid base64 sig: {:?}",
               * e))?,     )? {
               *         GenericSignature::ZkLoginAuthenticator(zk) => {
               *             if bytes.is_none() || curr_epoch.is_none() {
               *                 return
               * Ok(CommandOutput::ZkLoginSigVerify(ZkLoginSigVerifyResponse {
               *                     data: None,
               *                     parsed: Some(serde_json::to_string(&zk)?),
               *                     res: None,
               *                     jwks: None,
               *                 }));
               *             } */

              /*             let client = reqwest::Client::new();
               *             let provider = OIDCProvider::from_iss(zk.get_iss())
               *                 .map_err(|_| anyhow!("Invalid iss"))?;
               *             let jwks = fetch_jwks(&provider, &client).await?;
               *             let parsed: ImHashMap<JwkId, JWK> =
               * jwks.clone().into_iter().collect();             let env = match
               * network.as_str() {                 "devnet" | "localnet" =>
               * ZkLoginEnv::Test,                 "mainnet" | "testnet" =>
               * ZkLoginEnv::Prod,                 _ => return Err(anyhow!("Invalid
               * network")),             };
               *             let aux_verify_data = VerifyParams::new(parsed, vec![], env, true,
               * true); */

              /*             let (serialized, res) = match IntentScope::try_from(intent_scope)
               *                 .map_err(|_| anyhow!("Invalid scope"))?
               *             {
               *                 IntentScope::TransactionData => {
               *                     let tx_data: TransactionData = bcs::from_bytes(
               *                         &Base64::decode(&bytes.unwrap())
               *                             .map_err(|e| anyhow!("Invalid base64 tx data: {:?}",
               * e))?,                     )?; */

              /*                     let res = zk.verify_authenticator(
               *                         &IntentMessage::new(
               *                             Intent::iota_transaction(),
               *                             tx_data.clone(),
               *                         ),
               *                         tx_data.execution_parts().1,
               *                         Some(curr_epoch.unwrap()),
               *                         &aux_verify_data,
               *                     );
               *                     (serde_json::to_string(&tx_data)?, res)
               *                 }
               *                 IntentScope::PersonalMessage => {
               *                     let data: PersonalMessage = bcs::from_bytes(
               *                         &Base64::decode(&bytes.unwrap()).map_err(|e| {
               *                             anyhow!("Invalid base64 personal message data:
               * {:?}", e)                         })?,
               *                     )?; */

              /*                     let res = zk.verify_authenticator(
               *                         &IntentMessage::new(Intent::personal_message(),
               * data.clone()),                         (&zk).try_into()?,
               *                         Some(curr_epoch.unwrap()),
               *                         &aux_verify_data,
               *                     );
               *                     (serde_json::to_string(&data)?, res)
               *                 }
               *                 _ => return Err(anyhow!("Invalid intent scope")),
               *             };
               *             CommandOutput::ZkLoginSigVerify(ZkLoginSigVerifyResponse {
               *                 data: Some(serialized),
               *                 parsed: Some(serde_json::to_string(&zk)?),
               *                 jwks: Some(serde_json::to_string(&jwks)?),
               *                 res: Some(res),
               *             })
               *         }
               *         _ => CommandOutput::Error("Not a zkLogin signature".to_string()),
               *     }
               * } */
        });

        cmd_result
    }
}

// pub async fn fetch_jwks(
//     provider: &OIDCProvider,
//     client: &reqwest::Client,
// ) -> Result<Vec<(JwkId, JWK)>, FastCryptoError> {
//     let response = client
//         .get(provider.get_config().jwk_endpoint)
//         .send()
//         .await
//         .map_err(|e| {
//             FastCryptoError::GeneralError(format!(
//                 "Failed to get JWK {:?} {:?}",
//                 e.to_string(),
//                 provider
//             ))
//         })?;
//     let bytes = response.bytes().await.map_err(|e| {
//         FastCryptoError::GeneralError(format!(
//             "Failed to get bytes {:?} {:?}",
//             e.to_string(),
//             provider
//         ))
//     })?;
//     fastcrypto_zkp::bn254::zk_login::parse_jwks(&bytes, provider)
// }

impl From<&IotaKeyPair> for Key {
    fn from(ikp: &IotaKeyPair) -> Self {
        Key::from(ikp.public())
    }
}

impl From<PublicKey> for Key {
    fn from(pk: PublicKey) -> Self {
        Key {
            alias: None, // this is retrieved later
            iota_address: IotaAddress::from(&pk),
            public_base64_key: pk.encode_base64(),
            key_scheme: pk.scheme().to_string(),
            mnemonic: None,
            flag: pk.flag(),
            peer_id: anemo_styling(&pk),
        }
    }
}

impl Display for CommandOutput {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandOutput::Alias(update) => {
                write!(
                    formatter,
                    "Old alias {} was updated to {}",
                    update.old_alias, update.new_alias
                )
            }
            // Sign needs to be manually built because we need to wrap the very long
            // rawTxData string and rawIntentMsg strings into multiple rows due to
            // their lengths, which we cannot do with a JsonTable
            CommandOutput::Sign(data) => {
                let intent_table = json_to_table(&json!(&data.intent))
                    .with(tabled::settings::Style::rounded().horizontals([]))
                    .to_string();

                let mut builder = Builder::default();
                builder
                    .set_header([
                        "iotaSignature",
                        "digest",
                        "rawIntentMsg",
                        "intent",
                        "rawTxData",
                        "iotaAddress",
                    ])
                    .push_record([
                        &data.iota_signature,
                        &data.digest,
                        &data.raw_intent_msg,
                        &intent_table,
                        &data.raw_tx_data,
                        &data.iota_address.to_string(),
                    ]);
                let mut table = builder.build();
                table.with(Rotate::Left);
                table.with(tabled::settings::Style::rounded().horizontals([]));
                table.with(Modify::new(Rows::new(0..)).with(Width::wrap(160).keep_words()));
                write!(formatter, "{}", table)
            }
            _ => {
                let json_obj = json![self];
                let mut table = json_to_table(&json_obj);
                let style = tabled::settings::Style::rounded().horizontals([]);
                table.with(style);
                table.array_orientation(Orientation::Column);
                write!(formatter, "{}", table)
            }
        }
    }
}

impl CommandOutput {
    pub fn print(&self, pretty: bool) {
        let line = if pretty {
            format!("{self}")
        } else {
            format!("{:?}", self)
        };
        // Log line by line
        for line in line.lines() {
            // Logs write to a file on the side.  Print to stdout and also log to file, for
            // tests to pass.
            println!("{line}");
            info!("{line}")
        }
    }
}

// when --json flag is used, any output result is transformed into a JSON pretty
// string and sent to std output
impl Debug for CommandOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string_pretty(self) {
            Ok(json) => write!(f, "{json}"),
            Err(err) => write!(f, "Error serializing JSON: {err}"),
        }
    }
}

/// Converts legacy formatted private key to 33 bytes bech32 encoded private key
/// or vice versa. It can handle:
/// 1) Hex encoded 32 byte private key (assumes scheme is Ed25519), this is the
///    legacy wallet format
/// 2) Base64 encoded 32 bytes private key (assumes scheme is Ed25519)
/// 3) Base64 encoded 33 bytes private key with flag.
/// 4) Bech32 encoded 33 bytes private key with flag.
fn convert_private_key_to_bech32(value: String) -> Result<ConvertOutput, anyhow::Error> {
    let ikp = match IotaKeyPair::decode(&value) {
        Ok(s) => s,
        Err(_) => match Hex::decode(&value) {
            Ok(decoded) => {
                if decoded.len() != 32 {
                    return Err(anyhow!(format!(
                        "Invalid private key length, expected 32 but got {}",
                        decoded.len()
                    )));
                }
                IotaKeyPair::Ed25519(Ed25519KeyPair::from_bytes(&decoded)?)
            }
            Err(_) => match IotaKeyPair::decode_base64(&value) {
                Ok(ikp) => ikp,
                Err(_) => match Ed25519KeyPair::decode_base64(&value) {
                    Ok(kp) => IotaKeyPair::Ed25519(kp),
                    Err(_) => return Err(anyhow!("Invalid private key encoding")),
                },
            },
        },
    };

    Ok(ConvertOutput {
        bech32_with_flag: ikp.encode().map_err(|_| anyhow!("Cannot encode keypair"))?,
        base64_with_flag: ikp.encode_base64(),
        hex_without_flag: Hex::encode(&ikp.to_bytes()[1..]),
        scheme: ikp.public().scheme().to_string(),
    })
}

fn anemo_styling(pk: &PublicKey) -> Option<String> {
    if let PublicKey::Ed25519(public_key) = pk {
        Some(anemo::PeerId(public_key.0).to_string())
    } else {
        None
    }
}
