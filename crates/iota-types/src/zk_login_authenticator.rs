// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::hash::{Hash, Hasher};

use fastcrypto::{error::FastCryptoError, traits::ToFromBytes};
use fastcrypto_zkp::bn254::{
    zk_login::{OIDCProvider, ZkLoginInputs},
    zk_login_api::verify_zk_login,
};
use once_cell::sync::OnceCell;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared_crypto::intent::IntentMessage;

use crate::{
    base_types::{EpochId, IotaAddress},
    crypto::{DefaultHash, IotaSignature, PublicKey, Signature, SignatureScheme},
    digests::ZKLoginInputsDigest,
    error::{IotaError, IotaResult},
    signature::{AuthenticatorTrait, VerifyParams},
};

//#[cfg(any(test, feature = "test-utils"))]
#[cfg(test)]
#[path = "unit_tests/zk_login_authenticator_test.rs"]
mod zk_login_authenticator_test;

/// An zk login authenticator with all the necessary fields.
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZkLoginAuthenticator {
    pub inputs: ZkLoginInputs,
    max_epoch: EpochId,
    user_signature: Signature,
    #[serde(skip)]
    pub bytes: OnceCell<Vec<u8>>,
}

impl ZkLoginAuthenticator {
    pub fn hash_inputs(&self) -> ZKLoginInputsDigest {
        use fastcrypto::hash::HashFunction;
        let mut hasher = DefaultHash::default();
        hasher.update(bcs::to_bytes(&self.inputs).expect("serde should not fail"));
        ZKLoginInputsDigest::new(hasher.finalize().into())
    }

    /// Create a new [struct ZkLoginAuthenticator] with necessary fields.
    pub fn new(inputs: ZkLoginInputs, max_epoch: EpochId, user_signature: Signature) -> Self {
        Self {
            inputs,
            max_epoch,
            user_signature,
            bytes: OnceCell::new(),
        }
    }

    pub fn get_pk(&self) -> IotaResult<PublicKey> {
        PublicKey::from_zklogin_inputs(&self.inputs)
    }

    pub fn get_iss(&self) -> &str {
        self.inputs.get_iss()
    }

    pub fn get_max_epoch(&self) -> EpochId {
        self.max_epoch
    }

    #[cfg(feature = "test-utils")]
    pub fn user_signature_mut_for_testing(&mut self) -> &mut Signature {
        &mut self.user_signature
    }
}

/// Necessary trait for [struct SenderSignedData].
impl PartialEq for ZkLoginAuthenticator {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

/// Necessary trait for [struct SenderSignedData].
impl Eq for ZkLoginAuthenticator {}

/// Necessary trait for [struct SenderSignedData].
impl Hash for ZkLoginAuthenticator {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl AuthenticatorTrait for ZkLoginAuthenticator {
    fn verify_user_authenticator_epoch(&self, epoch: EpochId) -> IotaResult {
        // Verify the max epoch in aux inputs is <= the current epoch of authority.
        if epoch > self.get_max_epoch() {
            return Err(IotaError::InvalidSignature {
                error: format!("ZKLogin expired at epoch {}", self.get_max_epoch()),
            });
        }
        Ok(())
    }

    /// This verifies the address derivation and ephemeral signature.
    /// It does not verify the zkLogin inputs (that includes the expensive zk
    /// proof verify).
    fn verify_uncached_checks<T>(
        &self,
        intent_msg: &IntentMessage<T>,
        author: IotaAddress,
        aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize,
    {
        // Always evaluate the unpadded address derivation.
        if author != IotaAddress::try_from_unpadded(&self.inputs)? {
            // If the verify_legacy_zklogin_address flag is set, also evaluate the padded
            // address derivation.
            if !aux_verify_data.verify_legacy_zklogin_address
                || author != IotaAddress::try_from_padded(&self.inputs)?
            {
                return Err(IotaError::InvalidAddress);
            }
        }

        // Only when supported_providers list is not empty, we check if the provider is
        // supported. Otherwise, we just use the JWK map to check if its
        // supported.
        if !aux_verify_data.supported_providers.is_empty()
            && !aux_verify_data.supported_providers.contains(
                &OIDCProvider::from_iss(self.inputs.get_iss()).map_err(|_| {
                    IotaError::InvalidSignature {
                        error: "Unknown provider".to_string(),
                    }
                })?,
            )
        {
            return Err(IotaError::InvalidSignature {
                error: format!("OIDC provider not supported: {}", self.inputs.get_iss()),
            });
        }

        // Verify the ephemeral signature over the intent message of the transaction
        // data.
        self.user_signature
            .verify_secure(intent_msg, author, SignatureScheme::ZkLoginAuthenticator)
    }

    /// Verify an intent message of a transaction with an zk login
    /// authenticator.
    fn verify_claims<T>(
        &self,
        intent_msg: &IntentMessage<T>,
        author: IotaAddress,
        aux_verify_data: &VerifyParams,
    ) -> IotaResult
    where
        T: Serialize,
    {
        self.verify_uncached_checks(intent_msg, author, aux_verify_data)?;

        // Use flag || pk_bytes.
        let mut extended_pk_bytes = vec![self.user_signature.scheme().flag()];
        extended_pk_bytes.extend(self.user_signature.public_key_bytes());
        verify_zk_login(
            &self.inputs,
            self.max_epoch,
            &extended_pk_bytes,
            &aux_verify_data.oidc_provider_jwks,
            &aux_verify_data.zk_login_env,
        )
        .map_err(|e| IotaError::InvalidSignature {
            error: e.to_string(),
        })
    }
}

impl ToFromBytes for ZkLoginAuthenticator {
    fn from_bytes(bytes: &[u8]) -> Result<Self, FastCryptoError> {
        // The first byte matches the flag of MultiSig.
        if bytes.first().ok_or(FastCryptoError::InvalidInput)?
            != &SignatureScheme::ZkLoginAuthenticator.flag()
        {
            return Err(FastCryptoError::InvalidInput);
        }
        let mut zk_login: ZkLoginAuthenticator =
            bcs::from_bytes(&bytes[1..]).map_err(|_| FastCryptoError::InvalidSignature)?;
        zk_login.inputs.init()?;
        Ok(zk_login)
    }
}

impl AsRef<[u8]> for ZkLoginAuthenticator {
    fn as_ref(&self) -> &[u8] {
        self.bytes
            .get_or_try_init::<_, eyre::Report>(|| {
                let as_bytes = bcs::to_bytes(self).expect("BCS serialization should not fail");
                let mut bytes = Vec::with_capacity(1 + as_bytes.len());
                bytes.push(SignatureScheme::ZkLoginAuthenticator.flag());
                bytes.extend_from_slice(as_bytes.as_slice());
                Ok(bytes)
            })
            .expect("OnceCell invariant violated")
    }
}

#[derive(Debug, Clone)]
pub struct AddressSeed([u8; 32]);

impl AddressSeed {
    pub fn unpadded(&self) -> &[u8] {
        let mut buf = self.0.as_slice();

        while !buf.is_empty() && buf[0] == 0 {
            buf = &buf[1..];
        }

        // If the value is '0' then just return a slice of length 1 of the final byte
        if buf.is_empty() { &self.0[31..] } else { buf }
    }

    pub fn padded(&self) -> &[u8] {
        &self.0
    }
}

impl std::fmt::Display for AddressSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let big_int = num_bigint::BigUint::from_bytes_be(&self.0);
        let radix10 = big_int.to_str_radix(10);
        f.write_str(&radix10)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AddressSeedParseError {
    #[error("unable to parse radix10 encoded value `{0}`")]
    Parse(#[from] num_bigint::ParseBigIntError),
    #[error("larger than 32 bytes")]
    TooBig,
}

impl std::str::FromStr for AddressSeed {
    type Err = AddressSeedParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let big_int = <num_bigint::BigUint as num_traits::Num>::from_str_radix(s, 10)?;
        let be_bytes = big_int.to_bytes_be();
        let len = be_bytes.len();
        let mut buf = [0; 32];

        if len > 32 {
            return Err(AddressSeedParseError::TooBig);
        }

        buf[32 - len..].copy_from_slice(&be_bytes);
        Ok(Self(buf))
    }
}

// AddressSeed's serialized format is as a radix10 encoded string
impl Serialize for AddressSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AddressSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = std::borrow::Cow::<'de, str>::deserialize(deserializer)?;
        std::str::FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use num_bigint::BigUint;
    use proptest::prelude::*;

    use super::AddressSeed;

    #[test]
    fn unpadded_slice() {
        let seed = AddressSeed([0; 32]);
        let zero: [u8; 1] = [0];
        assert_eq!(seed.unpadded(), zero.as_slice());

        let mut seed = AddressSeed([1; 32]);
        seed.0[0] = 0;
        assert_eq!(seed.unpadded(), [1; 31].as_slice());
    }

    proptest! {
        #[test]
        fn dont_crash_on_large_inputs(
            bytes in proptest::collection::vec(any::<u8>(), 33..1024)
        ) {
            let big_int = BigUint::from_bytes_be(&bytes);
            let radix10 = big_int.to_str_radix(10);

            // doesn't crash
            let _ = AddressSeed::from_str(&radix10);
        }

        #[test]
        fn valid_address_seeds(
            bytes in proptest::collection::vec(any::<u8>(), 1..=32)
        ) {
            let big_int = BigUint::from_bytes_be(&bytes);
            let radix10 = big_int.to_str_radix(10);

            let seed = AddressSeed::from_str(&radix10).unwrap();
            assert_eq!(radix10, seed.to_string());
            // Ensure unpadded doesn't crash
            seed.unpadded();
        }
    }
}
