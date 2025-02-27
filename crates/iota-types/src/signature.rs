// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::hash::Hash;

pub use enum_dispatch::enum_dispatch;
use fastcrypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    error::FastCryptoError,
    secp256k1::{Secp256k1PublicKey, Secp256k1Signature},
    secp256r1::{Secp256r1PublicKey, Secp256r1Signature},
    traits::{EncodeDecodeBase64, ToFromBytes},
};
use fastcrypto_zkp::bn254::{
    zk_login::{JwkId, OIDCProvider, JWK},
    zk_login_api::ZkLoginEnv,
};
use im::hashmap::HashMap as ImHashMap;
use schemars::JsonSchema;
use serde::Serialize;
use shared_crypto::intent::IntentMessage;

use crate::{
    base_types::IotaAddress,
    committee::EpochId,
    crypto::{
        CompressedSignature, IotaSignature, PublicKey, Signature, SignatureScheme,
        ZkLoginAuthenticatorAsBytes,
    },
    error::{IotaError, IotaResult},
    multisig::MultiSig,
    multisig_legacy::MultiSigLegacy,
    zk_login_authenticator::ZkLoginAuthenticator,
};
#[derive(Default, Debug, Clone)]
pub struct VerifyParams {
    // map from JwkId (iss, kid) => JWK
    pub oidc_provider_jwks: ImHashMap<JwkId, JWK>,
    pub supported_providers: Vec<OIDCProvider>,
    pub zk_login_env: ZkLoginEnv,
    pub verify_legacy_zklogin_address: bool,
    pub accept_zklogin_in_multisig: bool,
}

impl VerifyParams {
    pub fn new(
        oidc_provider_jwks: ImHashMap<JwkId, JWK>,
        supported_providers: Vec<OIDCProvider>,
        zk_login_env: ZkLoginEnv,
        verify_legacy_zklogin_address: bool,
        accept_zklogin_in_multisig: bool,
    ) -> Self {
        Self {
            oidc_provider_jwks,
            supported_providers,
            zk_login_env,
            verify_legacy_zklogin_address,
            accept_zklogin_in_multisig,
        }
    }
}

/// A lightweight trait that all members of [enum GenericSignature] implement.
#[enum_dispatch]
pub trait AuthenticatorTrait {
    fn verify_user_authenticator_epoch(&self, epoch: EpochId) -> IotaResult;

    fn verify_claims<T>(
        &self,
        value: &IntentMessage<T>,
        author: IotaAddress,
        aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize;

    fn verify_authenticator<T>(
        &self,
        value: &IntentMessage<T>,
        author: IotaAddress,
        epoch: Option<EpochId>,
        aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize,
    {
        if let Some(epoch) = epoch {
            self.verify_user_authenticator_epoch(epoch)?;
        }
        self.verify_claims(value, author, aux_verify_data)
    }

    fn verify_uncached_checks<T>(
        &self,
        value: &IntentMessage<T>,
        author: IotaAddress,
        aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize;
}

/// Due to the incompatibility of [enum Signature] (which dispatches a trait
/// that assumes signature and pubkey bytes for verification), here we add a
/// wrapper enum where member can just implement a lightweight [trait
/// AuthenticatorTrait]. This way MultiSig (and future Authenticators) can
/// implement its own `verify`.
#[enum_dispatch(AuthenticatorTrait)]
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Hash)]
pub enum GenericSignature {
    MultiSig,
    MultiSigLegacy,
    Signature,
    ZkLoginAuthenticator,
}

impl GenericSignature {
    pub fn is_zklogin(&self) -> bool {
        matches!(self, GenericSignature::ZkLoginAuthenticator(_))
    }

    pub fn is_upgraded_multisig(&self) -> bool {
        matches!(self, GenericSignature::MultiSig(_))
    }

    /// Parse [enum CompressedSignature] from trait IotaSignature `flag || sig
    /// || pk`. This is useful for the MultiSig to combine partial signature
    /// into a MultiSig public key.
    pub fn to_compressed(&self) -> Result<CompressedSignature, IotaError> {
        match self {
            GenericSignature::Signature(s) => {
                let bytes = s.signature_bytes();
                match s.scheme() {
                    SignatureScheme::ED25519 => Ok(CompressedSignature::Ed25519(
                        (&Ed25519Signature::from_bytes(bytes).map_err(|_| {
                            IotaError::InvalidSignature {
                                error: "Cannot parse ed25519 sig".to_string(),
                            }
                        })?)
                            .into(),
                    )),
                    SignatureScheme::Secp256k1 => Ok(CompressedSignature::Secp256k1(
                        (&Secp256k1Signature::from_bytes(bytes).map_err(|_| {
                            IotaError::InvalidSignature {
                                error: "Cannot parse secp256k1 sig".to_string(),
                            }
                        })?)
                            .into(),
                    )),
                    SignatureScheme::Secp256r1 => Ok(CompressedSignature::Secp256r1(
                        (&Secp256r1Signature::from_bytes(bytes).map_err(|_| {
                            IotaError::InvalidSignature {
                                error: "Cannot parse secp256r1 sig".to_string(),
                            }
                        })?)
                            .into(),
                    )),
                    _ => Err(IotaError::UnsupportedFeature {
                        error: "Unsupported signature scheme".to_string(),
                    }),
                }
            }
            GenericSignature::ZkLoginAuthenticator(s) => Ok(CompressedSignature::ZkLogin(
                ZkLoginAuthenticatorAsBytes(s.as_ref().to_vec()),
            )),
            _ => Err(IotaError::UnsupportedFeature {
                error: "Unsupported signature scheme".to_string(),
            }),
        }
    }

    /// Parse [struct PublicKey] from trait IotaSignature `flag || sig || pk`.
    /// This is useful for the MultiSig to construct the bitmap in [struct
    /// MultiPublicKey].
    pub fn to_public_key(&self) -> Result<PublicKey, IotaError> {
        match self {
            GenericSignature::Signature(s) => {
                let bytes = s.public_key_bytes();
                match s.scheme() {
                    SignatureScheme::ED25519 => Ok(PublicKey::Ed25519(
                        (&Ed25519PublicKey::from_bytes(bytes).map_err(|_| {
                            IotaError::KeyConversion("Cannot parse ed25519 pk".to_string())
                        })?)
                            .into(),
                    )),
                    SignatureScheme::Secp256k1 => Ok(PublicKey::Secp256k1(
                        (&Secp256k1PublicKey::from_bytes(bytes).map_err(|_| {
                            IotaError::KeyConversion("Cannot parse secp256k1 pk".to_string())
                        })?)
                            .into(),
                    )),
                    SignatureScheme::Secp256r1 => Ok(PublicKey::Secp256r1(
                        (&Secp256r1PublicKey::from_bytes(bytes).map_err(|_| {
                            IotaError::KeyConversion("Cannot parse secp256r1 pk".to_string())
                        })?)
                            .into(),
                    )),
                    _ => Err(IotaError::UnsupportedFeature {
                        error: "Unsupported signature scheme in MultiSig".to_string(),
                    }),
                }
            }
            GenericSignature::ZkLoginAuthenticator(s) => s.get_pk(),
            _ => Err(IotaError::UnsupportedFeature {
                error: "Unsupported signature scheme".to_string(),
            }),
        }
    }
}

/// GenericSignature encodes a single signature [enum Signature] as is `flag ||
/// signature || pubkey`. It encodes [struct MultiSigLegacy] as the MultiSig
/// flag (0x03) concat with the bcs serializedbytes of [struct MultiSigLegacy]
/// i.e. `flag || bcs_bytes(MultiSigLegacy)`. [struct Multisig] is encodede as
/// the MultiSig flag (0x03) concat with the bcs serializedbytes of [struct
/// Multisig] i.e. `flag || bcs_bytes(Multisig)`.
impl ToFromBytes for GenericSignature {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        match SignatureScheme::from_flag_byte(
            bytes.first().ok_or(FastCryptoError::InputTooShort(0))?,
        ) {
            Ok(x) => match x {
                SignatureScheme::ED25519
                | SignatureScheme::Secp256k1
                | SignatureScheme::Secp256r1 => Ok(GenericSignature::Signature(
                    Signature::from_bytes(bytes).map_err(|_| FastCryptoError::InvalidSignature)?,
                )),
                SignatureScheme::MultiSig => match MultiSig::from_bytes(bytes) {
                    Ok(multisig) => Ok(GenericSignature::MultiSig(multisig)),
                    Err(_) => {
                        let multisig = MultiSigLegacy::from_bytes(bytes)?;
                        Ok(GenericSignature::MultiSigLegacy(multisig))
                    }
                },
                SignatureScheme::ZkLoginAuthenticator => {
                    let zk_login = ZkLoginAuthenticator::from_bytes(bytes)?;
                    Ok(GenericSignature::ZkLoginAuthenticator(zk_login))
                }
                _ => Err(FastCryptoError::InvalidInput),
            },
            Err(_) => Err(FastCryptoError::InvalidInput),
        }
    }
}

/// Trait useful to get the bytes reference for [enum GenericSignature].
impl AsRef<[u8]> for GenericSignature {
    fn as_ref(&self) -> &[u8] {
        match self {
            GenericSignature::MultiSig(s) => s.as_ref(),
            GenericSignature::MultiSigLegacy(s) => s.as_ref(),
            GenericSignature::Signature(s) => s.as_ref(),
            GenericSignature::ZkLoginAuthenticator(s) => s.as_ref(),
        }
    }
}

impl ::serde::Serialize for GenericSignature {
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            #[derive(serde::Serialize)]
            struct GenericSignature(String);
            GenericSignature(self.encode_base64()).serialize(serializer)
        } else {
            #[derive(serde::Serialize)]
            struct GenericSignature<'a>(&'a [u8]);
            GenericSignature(self.as_ref()).serialize(serializer)
        }
    }
}

impl<'de> ::serde::Deserialize<'de> for GenericSignature {
    fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        if deserializer.is_human_readable() {
            #[derive(serde::Deserialize)]
            struct GenericSignature(String);
            let s = GenericSignature::deserialize(deserializer)?;
            Self::decode_base64(&s.0).map_err(::serde::de::Error::custom)
        } else {
            #[derive(serde::Deserialize)]
            struct GenericSignature(Vec<u8>);

            let data = GenericSignature::deserialize(deserializer)?;
            Self::from_bytes(&data.0).map_err(|e| Error::custom(e.to_string()))
        }
    }
}

/// This ports the wrapper trait to the verify_secure defined on [enum
/// Signature].
impl AuthenticatorTrait for Signature {
    fn verify_user_authenticator_epoch(&self, _: EpochId) -> IotaResult {
        Ok(())
    }
    fn verify_uncached_checks<T>(
        &self,
        _value: &IntentMessage<T>,
        _author: IotaAddress,
        _aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize,
    {
        Ok(())
    }

    fn verify_claims<T>(
        &self,
        value: &IntentMessage<T>,
        author: IotaAddress,
        _aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize,
    {
        self.verify_secure(value, author, self.scheme())
    }
}
