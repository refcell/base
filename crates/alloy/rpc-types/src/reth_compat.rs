//! Reth RPC conversion trait implementations for base-alloy-rpc-types types.

use crate::{OpTransactionRequest, Transaction};
use alloy_consensus::{SignableTransaction, error::ValueError, transaction::Recovered};
use alloy_evm::{EvmEnv, env::BlockEnvironment, rpc::TryIntoTxEnv};
use alloy_network::TxSigner;
use alloy_primitives::{Address, Bytes};
use alloy_signer::Signature;
use base_alloy_consensus::{
    OpTransaction as OpTransactionTrait, OpTxEnvelope, transaction::OpTransactionInfo,
};
use core::convert::Infallible;
use op_revm::OpTransaction;
use reth_rpc_convert::{
    SignTxRequestError, SignableTxRequest, TryIntoSimTx, transaction::FromConsensusTx,
};
use revm::context::TxEnv;

impl<T: OpTransactionTrait + alloy_consensus::Transaction> FromConsensusTx<T> for Transaction<T> {
    type TxInfo = OpTransactionInfo;
    type Err = Infallible;

    fn from_consensus_tx(
        tx: T,
        signer: Address,
        tx_info: OpTransactionInfo,
    ) -> Result<Self, Infallible> {
        Ok(Self::from_transaction(Recovered::new_unchecked(tx, signer), tx_info))
    }
}

impl TryIntoSimTx<OpTxEnvelope> for OpTransactionRequest {
    fn try_into_sim_tx(self) -> Result<OpTxEnvelope, ValueError<Self>> {
        let tx = self
            .build_typed_tx()
            .map_err(|request| ValueError::new(*request, "Required fields missing"))?;

        let signature = Signature::new(Default::default(), Default::default(), false);

        Ok(tx.into_signed(signature).into())
    }
}

impl<Block: BlockEnvironment> TryIntoTxEnv<OpTransaction<TxEnv>, Block> for OpTransactionRequest {
    type Err = alloy_evm::rpc::EthTxEnvError;

    fn try_into_tx_env<Spec>(
        self,
        evm_env: &EvmEnv<Spec, Block>,
    ) -> Result<OpTransaction<TxEnv>, Self::Err> {
        Ok(OpTransaction {
            base: self.as_ref().clone().try_into_tx_env(evm_env)?,
            enveloped_tx: Some(Bytes::new()),
            deposit: Default::default(),
        })
    }
}

impl SignableTxRequest<OpTxEnvelope> for OpTransactionRequest {
    async fn try_build_and_sign(
        self,
        signer: impl TxSigner<Signature> + Send,
    ) -> Result<OpTxEnvelope, SignTxRequestError> {
        let mut tx = self
            .build_typed_tx()
            .map_err(|_| SignTxRequestError::InvalidTransactionRequest)?;

        if tx.is_deposit() {
            return Err(SignTxRequestError::InvalidTransactionRequest);
        }

        let signature = signer.sign_transaction(&mut tx).await?;

        Ok(tx.into_signed(signature).into())
    }
}
