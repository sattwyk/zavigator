use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use indexer_core::{
    decrypt_transaction_with_ufvk, params_for, parse_transaction_at_height, parse_viewing_key,
    DecryptedNote, Network as CoreNetwork,
};
use serde::Deserialize;
use wasm_bindgen::prelude::*;
use zcash_protocol::consensus::BlockHeight;

#[derive(Deserialize)]
struct TxInput {
    raw_tx: String,
    height: u32,
}

fn map_network(network: &str) -> Result<CoreNetwork, String> {
    match network.to_ascii_lowercase().as_str() {
        "mainnet" => Ok(CoreNetwork::Mainnet),
        "testnet" => Ok(CoreNetwork::Testnet),
        other => Err(format!(
            "unsupported network '{other}', expected 'mainnet' or 'testnet'"
        )),
    }
}

fn decode_txs(txs_json: &str) -> Result<Vec<TxInput>, String> {
    serde_json::from_str::<Vec<TxInput>>(txs_json).map_err(|e| format!("invalid txs json: {e}"))
}

fn decode_raw_tx(raw: &str) -> Result<Vec<u8>, String> {
    STANDARD
        .decode(raw.as_bytes())
        .map_err(|e| format!("failed to decode transaction: {e}"))
}

/// Decrypts a set of transactions with a UFVK and returns the decrypted notes as JSON.
#[wasm_bindgen]
pub fn decrypt_history(
    viewing_key_str: &str,
    txs_json: &str,
    network: &str,
) -> Result<String, JsValue> {
    let network = map_network(network).map_err(|e| JsValue::from_str(&e))?;
    let ufvk = parse_viewing_key(viewing_key_str, network)
        .map_err(|e| JsValue::from_str(&format!("invalid viewing key: {e}")))?;
    let params = params_for(network);
    let inputs = decode_txs(txs_json).map_err(|e| JsValue::from_str(&e))?;

    let mut notes: Vec<DecryptedNote> = Vec::new();
    for tx in inputs {
        let bytes = decode_raw_tx(&tx.raw_tx).map_err(|e| JsValue::from_str(&e))?;
        let height = BlockHeight::from(tx.height);
        let parsed_tx = parse_transaction_at_height(&params, height, &bytes)
            .map_err(|e| JsValue::from_str(&format!("failed to parse tx: {e}")))?;
        notes.extend(decrypt_transaction_with_ufvk(
            &params,
            height,
            &ufvk,
            &parsed_tx,
        ));
    }

    serde_json::to_string(&notes).map_err(|e| JsValue::from_str(&format!("serialize error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_network() {
        let err = map_network("regtest").unwrap_err();
        assert!(err.contains("unsupported network"));
    }

    #[test]
    fn decodes_txs_json() {
        let json = r#"[{"raw_tx":"", "height":5}]"#;
        let txs = decode_txs(json).unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(txs[0].height, 5);
    }
}
