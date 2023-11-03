use crate::{
    ClientError, Config, EventContext, EventUnsubscriber, Program, ProgramAccountsIterator,
    RequestBuilder,
};
use anchor_lang::{prelude::Pubkey, AccountDeserialize, Discriminator};
use std::ops::Deref;

#[cfg(target_arch = "wasm32")]
use solana_client_wasm::{
    solana_sdk::{
        commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
        transaction::Transaction,
    },
    utils::{rpc_config::RpcSendTransactionConfig, rpc_filter::RpcFilterType},
};

#[cfg(not(target_arch = "wasm32"))]
use {
    solana_client::{rpc_config::RpcSendTransactionConfig, rpc_filter::RpcFilterType},
    solana_sdk::{
        commitment_config::CommitmentConfig, signature::Signature, signer::Signer,
        transaction::Transaction,
    },
    std::{marker::PhantomData, sync::Arc},
    tokio::sync::RwLock,
};

#[cfg(not(target_arch = "wasm32"))]
impl<'a> EventUnsubscriber<'a> {
    /// Unsubscribe gracefully.
    pub async fn unsubscribe(self) {
        self.unsubscribe_internal().await
    }
}

#[cfg(target_arch = "wasm32")]
impl EventUnsubscriber {
    /// Unsubscribe gracefully.
    pub async fn unsubscribe(self) {
        self.unsubscribe_internal().await
    }
}

impl<C: Deref<Target = impl Signer> + Clone> Program<C> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(program_id: Pubkey, cfg: Config<C>) -> Result<Self, ClientError> {
        Ok(Self {
            program_id,
            cfg,
            sub_client: Arc::new(RwLock::new(None)),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(program_id: Pubkey, cfg: Config<C>) -> Result<Self, ClientError> {
        Ok(Self { program_id, cfg })
    }

    /// Returns the account at the given address.
    pub async fn account<T: AccountDeserialize>(&self, address: Pubkey) -> Result<T, ClientError> {
        self.account_internal(address).await
    }

    /// Returns all program accounts of the given type matching the given filters
    pub async fn accounts<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<Vec<(Pubkey, T)>, ClientError> {
        self.accounts_lazy(filters).await?.collect()
    }

    /// Returns all program accounts of the given type matching the given filters as an iterator
    /// Deserialization is executed lazily
    pub async fn accounts_lazy<T: AccountDeserialize + Discriminator>(
        &self,
        filters: Vec<RpcFilterType>,
    ) -> Result<ProgramAccountsIterator<T>, ClientError> {
        self.accounts_lazy_internal(filters).await
    }

    /// Subscribe to program logs.
    ///
    /// Returns an [`EventUnsubscriber`] to unsubscribe and close connection gracefully.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> Result<EventUnsubscriber, ClientError> {
        let (handle, rx) = self.on_internal(f).await?;

        Ok(EventUnsubscriber {
            handle,
            rx,
            _lifetime_marker: PhantomData,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn on<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
        f: impl Fn(&EventContext, T) + Send + 'static,
    ) -> EventUnsubscriber {
        self.on_internal(f).await
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> RequestBuilder<'a, C> {
    pub fn from(
        program_id: Pubkey,
        cluster: &str,
        payer: C,
        options: Option<CommitmentConfig>,
    ) -> Self {
        Self {
            program_id,
            payer,
            cluster: cluster.to_string(),
            accounts: Vec::new(),
            options: options.unwrap_or_default(),
            instructions: Vec::new(),
            instruction_data: None,
            signers: Vec::new(),
        }
    }

    pub async fn signed_transaction(&self) -> Result<Transaction, ClientError> {
        self.signed_transaction_internal().await
    }

    pub async fn send(self) -> Result<Signature, ClientError> {
        self.send_internal().await
    }

    pub async fn send_with_spinner_and_config(
        self,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, ClientError> {
        self.send_with_spinner_and_config_internal(config).await
    }
}
