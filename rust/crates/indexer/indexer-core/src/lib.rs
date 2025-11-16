//! Core, WASM-friendly primitives for parsing viewing keys and decrypting shielded data.

use orchard::note_encryption::OrchardDomain;
use sapling::note_encryption::{PreparedIncomingViewingKey as PreparedSaplingIvk, SaplingDomain};
use serde::Serialize;
use thiserror::Error;
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_note_encryption::{try_note_decryption, try_output_recovery_with_ovk};
use zcash_primitives::transaction::components::sapling::zip212_enforcement;
use zcash_primitives::transaction::{Transaction, TxId};
use zcash_protocol::consensus::{BlockHeight, BranchId, Network as ConsensusNetwork, Parameters};
use zcash_protocol::memo::MemoBytes;
use zip32::Scope;

/// Supported Zcash networks.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

/// Shielded pool the note belongs to.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum ShieldedProtocol {
    Sapling,
    Orchard,
}

/// How the wallet relates to the decrypted note.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum TransferType {
    Incoming,
    Outgoing,
    Internal,
}

/// A decrypted shielded note with minimal metadata for UI consumption.
#[derive(Debug, Clone, Serialize)]
pub struct DecryptedNote {
    #[serde(serialize_with = "serialize_txid")]
    pub txid: TxId,
    pub index: usize,
    pub value: u64,
    pub memo: Vec<u8>,
    pub protocol: ShieldedProtocol,
    pub transfer_type: TransferType,
    pub height: u32,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("{0}")]
    Message(String),
}

impl From<String> for CoreError {
    fn from(value: String) -> Self {
        CoreError::Message(value)
    }
}

/// Returns consensus parameters for the requested network.
pub fn params_for(network: Network) -> ConsensusNetwork {
    match network {
        Network::Mainnet => ConsensusNetwork::MainNetwork,
        Network::Testnet => ConsensusNetwork::TestNetwork,
    }
}

/// Parses a Unified Full Viewing Key string for the provided network.
pub fn parse_viewing_key(
    encoded: &str,
    network: Network,
) -> Result<UnifiedFullViewingKey, CoreError> {
    let params = params_for(network);
    UnifiedFullViewingKey::decode(&params, encoded).map_err(CoreError::from)
}

/// Parses raw transaction bytes into a [`Transaction`].
///
/// Note: without context we assume NU5+ semantics (branch ID [`BranchId::Nu5`]).
pub fn parse_transaction(bytes: &[u8]) -> Result<Transaction, CoreError> {
    let branch_id = BranchId::Nu5;
    Transaction::read(std::io::Cursor::new(bytes), branch_id)
        .map_err(|e| CoreError::Message(format!("failed to parse transaction: {e}")))
}

/// Decrypts the Sapling and Orchard outputs of a transaction with a single UFVK.
pub fn decrypt_transaction_with_ufvk<P: Parameters>(
    params: &P,
    height: BlockHeight,
    ufvk: &UnifiedFullViewingKey,
    tx: &Transaction,
) -> Vec<DecryptedNote> {
    let mut notes = Vec::new();
    decrypt_sapling_outputs(params, height, ufvk, tx, &mut notes);
    decrypt_orchard_actions(height, ufvk, tx, &mut notes);
    notes
}

fn decrypt_sapling_outputs<P: Parameters>(
    params: &P,
    height: BlockHeight,
    ufvk: &UnifiedFullViewingKey,
    tx: &Transaction,
    notes: &mut Vec<DecryptedNote>,
) {
    let dfvk = match ufvk.sapling() {
        Some(dfvk) => dfvk,
        None => return,
    };

    let bundle = match tx.sapling_bundle() {
        Some(bundle) => bundle,
        None => return,
    };

    let zip212 = zip212_enforcement(params, height);
    let domain = SaplingDomain::new(zip212);
    let ivk_external = PreparedSaplingIvk::new(&dfvk.to_ivk(Scope::External));
    let ivk_internal = PreparedSaplingIvk::new(&dfvk.to_ivk(Scope::Internal));
    let ovk = dfvk.fvk().ovk;

    for (index, output) in bundle.shielded_outputs().iter().enumerate() {
        let decrypted = try_note_decryption(&domain, &ivk_external, output)
            .map(|ret| (ret, TransferType::Incoming))
            .or_else(|| {
                try_note_decryption(&domain, &ivk_internal, output)
                    .map(|ret| (ret, TransferType::Internal))
            })
            .or_else(|| {
                try_output_recovery_with_ovk(
                    &domain,
                    &ovk,
                    output,
                    output.cv(),
                    output.out_ciphertext(),
                )
                .map(|ret| (ret, TransferType::Outgoing))
            });

        if let Some(((note, _address, memo), transfer_type)) = decrypted {
            notes.push(DecryptedNote {
                txid: tx.txid(),
                index,
                value: note.value().inner(),
                memo: MemoBytes::from_bytes(&memo)
                    .expect("memo byte length is enforced by decryption")
                    .into_bytes()
                    .to_vec(),
                protocol: ShieldedProtocol::Sapling,
                transfer_type,
                height: height.into(),
            });
        }
    }
}

fn decrypt_orchard_actions(
    height: BlockHeight,
    ufvk: &UnifiedFullViewingKey,
    tx: &Transaction,
    notes: &mut Vec<DecryptedNote>,
) {
    let fvk = match ufvk.orchard() {
        Some(fvk) => fvk,
        None => return,
    };

    let bundle = match tx.orchard_bundle() {
        Some(bundle) => bundle,
        None => return,
    };

    let ivk_external = orchard::keys::PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::External));
    let ivk_internal = orchard::keys::PreparedIncomingViewingKey::new(&fvk.to_ivk(Scope::Internal));
    let ovk = fvk.to_ovk(Scope::External);

    for (index, action) in bundle.actions().iter().enumerate() {
        let domain = OrchardDomain::for_action(action);
        let decrypted = try_note_decryption(&domain, &ivk_external, action)
            .map(|ret| (ret, TransferType::Incoming))
            .or_else(|| {
                try_note_decryption(&domain, &ivk_internal, action)
                    .map(|ret| (ret, TransferType::Internal))
            })
            .or_else(|| {
                try_output_recovery_with_ovk(
                    &domain,
                    &ovk,
                    action,
                    action.cv_net(),
                    &action.encrypted_note().out_ciphertext,
                )
                .map(|ret| (ret, TransferType::Outgoing))
            });

        if let Some(((note, _address, memo), transfer_type)) = decrypted {
            notes.push(DecryptedNote {
                txid: tx.txid(),
                index,
                value: note.value().inner(),
                memo: MemoBytes::from_bytes(&memo)
                    .expect("memo byte length is enforced by decryption")
                    .into_bytes()
                    .to_vec(),
                protocol: ShieldedProtocol::Orchard,
                transfer_type,
                height: height.into(),
            });
        }
    }
}

fn serialize_txid<S>(txid: &TxId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&txid.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcash_protocol::consensus::NetworkType;

    #[test]
    fn params_for_maps_network() {
        assert_eq!(
            params_for(Network::Mainnet).network_type(),
            NetworkType::Main
        );
        assert_eq!(
            params_for(Network::Testnet).network_type(),
            NetworkType::Test
        );
    }

    #[test]
    fn parsing_invalid_transaction_fails() {
        let err = parse_transaction(&[0u8; 4]).unwrap_err();
        assert!(
            err.to_string().contains("failed to parse transaction"),
            "unexpected error: {err:?}"
        );
    }
}
