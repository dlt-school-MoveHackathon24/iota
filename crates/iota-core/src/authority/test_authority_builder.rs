// Copyright (c) Mysten Labs, Inc.
// Modifications Copyright (c) 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{path::PathBuf, sync::Arc};

use fastcrypto::traits::KeyPair;
use iota_archival::reader::ArchiveReaderBalancer;
use iota_config::{
    certificate_deny_config::CertificateDenyConfig,
    genesis::Genesis,
    node::{
        AuthorityOverloadConfig, AuthorityStorePruningConfig, DBCheckpointConfig,
        ExpensiveSafetyCheckConfig, StateDebugDumpConfig,
    },
    transaction_deny_config::TransactionDenyConfig,
};
use iota_macros::nondeterministic;
use iota_protocol_config::{ProtocolConfig, SupportedProtocolVersions};
use iota_storage::IndexStore;
use iota_swarm_config::{genesis_config::AccountConfig, network_config::NetworkConfig};
use iota_types::{
    base_types::{AuthorityName, ObjectID},
    crypto::AuthorityKeyPair,
    digests::ChainIdentifier,
    executable_transaction::VerifiedExecutableTransaction,
    iota_system_state::IotaSystemStateTrait,
    object::Object,
    transaction::VerifiedTransaction,
};
use prometheus::Registry;
use tempfile::tempdir;

use crate::{
    authority::{
        authority_per_epoch_store::AuthorityPerEpochStore,
        authority_store_tables::AuthorityPerpetualTables,
        epoch_start_configuration::EpochStartConfiguration, AuthorityState, AuthorityStore,
    },
    checkpoints::CheckpointStore,
    epoch::{committee_store::CommitteeStore, epoch_metrics::EpochMetrics},
    execution_cache::ExecutionCache,
    module_cache_metrics::ResolverMetrics,
    signature_verifier::SignatureVerifierMetrics,
};

#[derive(Default, Clone)]
pub struct TestAuthorityBuilder<'a> {
    store_base_path: Option<PathBuf>,
    store: Option<Arc<AuthorityStore>>,
    transaction_deny_config: Option<TransactionDenyConfig>,
    certificate_deny_config: Option<CertificateDenyConfig>,
    protocol_config: Option<ProtocolConfig>,
    reference_gas_price: Option<u64>,
    node_keypair: Option<&'a AuthorityKeyPair>,
    genesis: Option<&'a Genesis>,
    starting_objects: Option<&'a [Object]>,
    expensive_safety_checks: Option<ExpensiveSafetyCheckConfig>,
    disable_indexer: bool,
    accounts: Vec<AccountConfig>,
    /// By default, we don't insert the genesis checkpoint, which isn't needed
    /// by most tests.
    insert_genesis_checkpoint: bool,
    authority_overload_config: Option<AuthorityOverloadConfig>,
}

impl<'a> TestAuthorityBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store_base_path(mut self, path: PathBuf) -> Self {
        assert!(self.store_base_path.replace(path).is_none());
        self
    }

    pub fn with_starting_objects(mut self, objects: &'a [Object]) -> Self {
        assert!(self.starting_objects.replace(objects).is_none());
        self
    }

    pub fn with_store(mut self, store: Arc<AuthorityStore>) -> Self {
        assert!(self.store.replace(store).is_none());
        self
    }

    pub fn with_transaction_deny_config(mut self, config: TransactionDenyConfig) -> Self {
        assert!(self.transaction_deny_config.replace(config).is_none());
        self
    }

    pub fn with_certificate_deny_config(mut self, config: CertificateDenyConfig) -> Self {
        assert!(self.certificate_deny_config.replace(config).is_none());
        self
    }

    pub fn with_protocol_config(mut self, config: ProtocolConfig) -> Self {
        assert!(self.protocol_config.replace(config).is_none());
        self
    }

    pub fn with_reference_gas_price(mut self, reference_gas_price: u64) -> Self {
        // If genesis is already set then setting rgp is meaningless since it will be
        // overwritten.
        assert!(self.genesis.is_none());
        assert!(
            self.reference_gas_price
                .replace(reference_gas_price)
                .is_none()
        );
        self
    }

    pub fn with_genesis_and_keypair(
        mut self,
        genesis: &'a Genesis,
        keypair: &'a AuthorityKeyPair,
    ) -> Self {
        assert!(self.genesis.replace(genesis).is_none());
        assert!(self.node_keypair.replace(keypair).is_none());
        self
    }

    pub fn with_keypair(mut self, keypair: &'a AuthorityKeyPair) -> Self {
        assert!(self.node_keypair.replace(keypair).is_none());
        self
    }

    /// When providing a network config, we will use the first validator's
    /// key as the keypair for the new node.
    pub fn with_network_config(self, config: &'a NetworkConfig) -> Self {
        self.with_genesis_and_keypair(
            &config.genesis,
            config.validator_configs()[0].protocol_key_pair(),
        )
    }

    pub fn disable_indexer(mut self) -> Self {
        self.disable_indexer = true;
        self
    }

    pub fn insert_genesis_checkpoint(mut self) -> Self {
        self.insert_genesis_checkpoint = true;
        self
    }

    pub fn with_expensive_safety_checks(mut self, config: ExpensiveSafetyCheckConfig) -> Self {
        assert!(self.expensive_safety_checks.replace(config).is_none());
        self
    }

    pub fn with_accounts(mut self, accounts: Vec<AccountConfig>) -> Self {
        self.accounts = accounts;
        self
    }

    pub fn with_authority_overload_config(mut self, config: AuthorityOverloadConfig) -> Self {
        assert!(self.authority_overload_config.replace(config).is_none());
        self
    }

    pub async fn build(self) -> Arc<AuthorityState> {
        let mut local_network_config_builder =
            iota_swarm_config::network_config_builder::ConfigBuilder::new_with_temp_dir()
                .with_accounts(self.accounts)
                .with_reference_gas_price(self.reference_gas_price.unwrap_or(500));
        if let Some(protocol_config) = &self.protocol_config {
            local_network_config_builder =
                local_network_config_builder.with_protocol_version(protocol_config.version);
        }
        let local_network_config = local_network_config_builder.build();
        let genesis = &self.genesis.unwrap_or(&local_network_config.genesis);
        let genesis_committee = genesis.committee().unwrap();
        let path = self.store_base_path.unwrap_or_else(|| {
            let dir = std::env::temp_dir();
            let store_base_path =
                dir.join(format!("DB_{:?}", nondeterministic!(ObjectID::random())));
            std::fs::create_dir(&store_base_path).unwrap();
            store_base_path
        });
        let authority_store = match self.store {
            Some(store) => store,
            None => {
                let perpetual_tables =
                    Arc::new(AuthorityPerpetualTables::open(&path.join("store"), None));
                // unwrap ok - for testing only.
                AuthorityStore::open_with_committee_for_testing(
                    perpetual_tables,
                    &genesis_committee,
                    genesis,
                    0,
                )
                .await
                .unwrap()
            }
        };
        let keypair = self
            .node_keypair
            .unwrap_or_else(|| local_network_config.validator_configs()[0].protocol_key_pair());
        let secret = Arc::pin(keypair.copy());
        let name: AuthorityName = secret.public().into();
        let registry = Registry::new();
        let cache_metrics = Arc::new(ResolverMetrics::new(&registry));
        let signature_verifier_metrics = SignatureVerifierMetrics::new(&registry);
        // `_guard` must be declared here so it is not dropped before
        // `AuthorityPerEpochStore::new` is called
        // Force disable random beacon for tests using this builder, because it doesn't
        // set up the RandomnessManager.
        let _guard = if let Some(mut config) = self.protocol_config {
            config.set_random_beacon_for_testing(false);
            ProtocolConfig::apply_overrides_for_testing(move |_, _| config.clone())
        } else {
            ProtocolConfig::apply_overrides_for_testing(|_, mut config| {
                config.set_random_beacon_for_testing(false);
                config
            })
        };
        let epoch_start_configuration = EpochStartConfiguration::new(
            genesis.iota_system_object().into_epoch_start_state(),
            *genesis.checkpoint().digest(),
            &genesis.objects(),
            None,
        )
        .unwrap();
        let expensive_safety_checks = self.expensive_safety_checks.unwrap_or_default();
        let cache = Arc::new(ExecutionCache::new_for_tests(
            authority_store.clone(),
            &registry,
        ));
        let epoch_store = AuthorityPerEpochStore::new(
            name,
            Arc::new(genesis_committee.clone()),
            &path.join("store"),
            None,
            EpochMetrics::new(&registry),
            epoch_start_configuration,
            cache.clone(),
            cache_metrics,
            signature_verifier_metrics,
            &expensive_safety_checks,
            ChainIdentifier::from(*genesis.checkpoint().digest()),
        );
        let committee_store = Arc::new(CommitteeStore::new(
            path.join("epochs"),
            &genesis_committee,
            None,
        ));

        let checkpoint_store = CheckpointStore::new(&path.join("checkpoints"));
        if self.insert_genesis_checkpoint {
            checkpoint_store.insert_genesis_checkpoint(
                genesis.checkpoint(),
                genesis.checkpoint_contents().clone(),
                &epoch_store,
            );
        }
        let index_store = if self.disable_indexer {
            None
        } else {
            Some(Arc::new(IndexStore::new(
                path.join("indexes"),
                &registry,
                epoch_store
                    .protocol_config()
                    .max_move_identifier_len_as_option(),
            )))
        };
        let transaction_deny_config = self.transaction_deny_config.unwrap_or_default();
        let certificate_deny_config = self.certificate_deny_config.unwrap_or_default();
        let authority_overload_config = self.authority_overload_config.unwrap_or_default();
        let mut pruning_config = AuthorityStorePruningConfig::default();
        if !epoch_store
            .protocol_config()
            .simplified_unwrap_then_delete()
        {
            // We cannot prune tombstones if simplified_unwrap_then_delete is not enabled.
            pruning_config.set_killswitch_tombstone_pruning(true);
        }
        let state = AuthorityState::new(
            name,
            secret,
            SupportedProtocolVersions::SYSTEM_DEFAULT,
            authority_store,
            cache,
            epoch_store,
            committee_store,
            index_store,
            checkpoint_store,
            &registry,
            pruning_config,
            genesis.objects(),
            &DBCheckpointConfig::default(),
            ExpensiveSafetyCheckConfig::new_enable_all(),
            transaction_deny_config,
            certificate_deny_config,
            usize::MAX,
            StateDebugDumpConfig {
                dump_file_directory: Some(tempdir().unwrap().into_path()),
            },
            authority_overload_config,
            ArchiveReaderBalancer::default(),
        )
        .await;
        // For any type of local testing that does not actually spawn a node, the
        // checkpoint executor won't be started, which means we won't actually
        // execute the genesis transaction. In that case, the genesis objects
        // (e.g. all the genesis test coins) won't be accessible. Executing it
        // explicitly makes sure all genesis objects are ready for use.
        state
            .try_execute_immediately(
                &VerifiedExecutableTransaction::new_from_checkpoint(
                    VerifiedTransaction::new_unchecked(genesis.transaction().clone()),
                    genesis.epoch(),
                    genesis.checkpoint().sequence_number,
                ),
                None,
                &state.epoch_store_for_testing(),
            )
            .await
            .unwrap();

        // We want to insert these objects directly instead of relying on genesis
        // because genesis process would set the previous transaction field for
        // these objects, which would change their object digest. This makes it
        // difficult to write tests that want to use these objects directly.
        // TODO: we should probably have a better way to do this.
        if let Some(starting_objects) = self.starting_objects {
            state
                .insert_objects_unsafe_for_testing_only(starting_objects)
                .await
                .unwrap();
        };
        state
    }
}
