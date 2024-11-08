#![allow(dead_code, unused_imports, unused_variables, unused_must_use)]
pub mod internal;

use bitcoin::blockdata::opcodes::all as opcodes;
use bitcoin::hashes::ripemd160::Hash as Ripemd160;
use bitcoin::locktime::absolute::LockTime;
use bitcoin::script::ScriptBuf;
use bitcoin::secp256k1::ecdsa::Signature;
use bitcoin::transaction::Version;
use bitcoin::PubkeyHash;
use bitcoin::{Block, OutPoint, PublicKey, Transaction, TxIn, TxOut};
use internal::bitcoind_client::BitcoindClient;
use internal::builder::Builder;
use internal::channel_manager::ChannelManager;
use internal::helper::{pubkey_multiplication_tweak, sha256_hash};

fn p2pkh(pubkey: &PublicKey) -> ScriptBuf {
    Builder::new()
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_pubkey_hash(pubkey)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_opcode(opcodes::OP_CHECKSIG)
        .into_script()
}

fn two_of_two_multisig(alice_pubkey: &PublicKey, bob_pubkey: &PublicKey) -> ScriptBuf {
    Builder::new()
        .push_int(2)
        .push_key(alice_pubkey)
        .push_key(bob_pubkey)
        .push_int(2)
        .push_opcode(opcodes::OP_CHECKMULTISIG)
        .into_script()
}

fn two_of_three_multisig_redeem_script(
    pubkey: &PublicKey,
    pubkey2: &PublicKey,
    pubkey3: &PublicKey,
) -> ScriptBuf {
    Builder::new()
        .push_int(2)
        .push_key(pubkey)
        .push_key(pubkey2)
        .push_key(pubkey3)
        .push_int(3)
        .push_opcode(opcodes::OP_CHECKMULTISIG)
        .into_script()
}

fn p2sh(script_hash: ScriptHash) -> ScriptBuf {
    Builder::new()
        .push_opcode(opcodes::OP_HASH160)
        .push_slice(script_hash)
        .push_opcode(opcodes::OP_EQUAL)
        .into_script()
}

fn cltv_p2pkh(pubkey: &PublicKey, height_or_timestamp: i64) -> ScriptBuf {
    Builder::new()
        .push_int(height_or_timestamp)
        .push_opcode(opcodes::OP_CLTV)
        .push_opcode(opcodes::OP_DROP)
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_pubkey_hash(pubkey)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_opcode(opcodes::OP_CHECKSIG)
        .into_script()
}

fn csv_p2pkh(pubkey: &PublicKey, height_or_timestamp: i64) -> ScriptBuf {
    Builder::new()
        .push_int(height_or_timestamp)
        .push_opcode(opcodes::OP_CSV)
        .push_opcode(opcodes::OP_DROP)
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_pubkey_hash(pubkey)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_opcode(opcodes::OP_CHECKSIG)
        .into_script()
}

fn build_output(amount: Amount, output_script: ScriptBuf) -> TxOut {
    TxOut {
        value: amount,
        script_pubkey: output_script,
    }
}

fn build_timelocked_transaction(
    txins: Vec<TxIn>,
    pubkey: &PublicKey,
    block_height: u64,
    csv_delay: u32,
    amount: Amount,
) -> Transaction {
    
    let output_script = csv_p2pkh(pubkey, csv_delay);
    
    let txout = build_output(amount, output_script);

    Transaction {
        version: Version::ONE,
        lock_time: LockTime::from_height(block_height),
        input: txins,
        output: vec![txout],
    }
}

fn payment_channel_funding_output(
    alice_pubkey: &PublicKey,
    bob_pubkey: &PublicKey,
    height: i64,
) -> ScriptBuf {
    Builder::new()
        .push_opcode(opcodes::OP_IF)
        .push_script(two_of_two_multisig(alice_pubkey, bob_pubkey))
        .push_opcode(opcodes::OP_ELSE)
        .push_script(csv_p2pkh(alice_pubkey, height))
        .push_opcode(opcodes::OP_ENDIF)
        .into_script()
}

fn block_connected(funding_output: ScriptBuf, channel_amount_sats: u64, block: Block) -> bool {
    for tx in block.txdata {
        for output in tx.output {
            if output.script_pubkey == funding_output && output.value == channel_amount_sats {
                return true;
            }
        }
    }
    false
}

fn spend_multisig(alice_signature: Signature, bob_signature: Signature) -> ScriptBuf {
    Builder::new()
        .push_signature(alice_signature)
        .push_signature(bob_signature)
        .push_int(0)
        .into_script()
}

fn spend_refund(alice_pubkey: &PublicKey, alice_signature: Signature) -> ScriptBuf {
    Builder::new()
        .push_signature(alice_signature)
        .push_key(alice_pubkey)
        .push_int(1)
        .into_script()
}

pub fn generate_revocation_pubkey(
    countersignatory_basepoint: PublicKey,
    per_commitment_point: PublicKey,
) -> PublicKey {
    let rev_append_commit_hash_key =
        sha256_hash(&countersignatory_basepoint, &per_commitment_point);

    let commit_append_rev_hash_key =
        sha256_hash(&per_commitment_point, &countersignatory_basepoint);

    let countersignatory_contrib =
        pubkey_multiplication_tweak(countersignatory_basepoint, rev_append_commit_hash_key);

    let broadcaster_contrib =
        pubkey_multiplication_tweak(per_commitment_point, commit_append_rev_hash_key);

    let pk = countersignatory_contrib.combine(&broadcaster_contrib)
        .expect("Addition only fails if the tweak is the inverse of the key. This is not possible when the tweak commits to the key.");

    pk
}

fn channel_closed(funding_outpoint: OutPoint, block: Block) -> bool {
    for tx in block.txdata {
        for input in tx.input {
            if input.previous_output == funding_outpoint {
                return true;
            }
        }
    }
    false
}

fn build_htlc_offerer_witness_script(
    revocation_pubkey160: &PubkeyHash,
    remote_htlc_pubkey: &PublicKey,
    local_htlc_pubkey: &PublicKey,
    payment_hash160: &[u8; 20],
) -> ScriptBuf {
    Builder::new()
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_slice(revocation_pubkey160)
        .push_opcode(opcodes::OP_EQUAL)
        .push_opcode(opcodes::OP_IF)
        .push_opcode(opcodes::OP_CHECKSIG)
        .push_opcode(opcodes::OP_ELSE)
        .push_key(&remote_htlc_pubkey)
        .push_opcode(opcodes::OP_SWAP)
        .push_opcode(opcodes::OP_SIZE)
        .push_int(32)
        .push_opcode(opcodes::OP_EQUAL)
        .push_opcode(opcodes::OP_NOTIF)
        .push_opcode(opcodes::OP_DROP)
        .push_int(2)
        .push_opcode(opcodes::OP_SWAP)
        .push_key(&local_htlc_pubkey)
        .push_int(2)
        .push_opcode(opcodes::OP_CHECKMULTISIG)
        .push_opcode(opcodes::OP_ELSE)
        .push_opcode(opcodes::OP_HASH160)
        .push_slice(payment_hash160)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_opcode(opcodes::OP_CHECKSIG)
        .push_opcode(opcodes::OP_ENDIF)
        .into_script()
}

fn build_htlc_receiver_witness_script(
    revocation_pubkey160: &PubkeyHash,
    remote_htlc_pubkey: &PublicKey,
    local_htlc_pubkey: &PublicKey,
    payment_hash160: &[u8; 20],
    cltv_expiry: i64,
) -> ScriptBuf {
    Builder::new()
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_slice(&revocation_pubkey160)
        .push_opcode(opcodes::OP_EQUAL)
        .push_opcode(opcodes::OP_IF)
        .push_opcode(opcodes::OP_CHECKSIG)
        .push_opcode(opcodes::OP_ELSE)
        .push_key(&remote_htlc_pubkey)
        .push_opcode(opcodes::OP_SWAP)
        .push_opcode(opcodes::OP_SIZE)
        .push_int(32)
        .push_opcode(opcodes::OP_EQUAL)
        .push_opcode(opcodes::OP_IF)
        .push_opcode(opcodes::OP_HASH160)
        .push_slice(&payment_hash160)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_int(2)
        .push_opcode(opcodes::OP_SWAP)
        .push_key(&local_htlc_pubkey)
        .push_int(2)
        .push_opcode(opcodes::OP_CHECKMULTISIG)
        .push_opcode(opcodes::OP_ELSE)
        .push_opcode(opcodes::OP_DROP)
        .push_int(cltv_expiry)
        .push_opcode(opcodes::OP_CLTV)
        .push_opcode(opcodes::OP_DROP)
        .push_opcode(opcodes::OP_CHECKSIG)
        .push_opcode(opcodes::OP_ENDIF)
        .into_script()
}

fn handle_funding_generation_ready(
    channel_manager: &ChannelManager,
    bitcoind_client: &BitcoindClient,
    temporary_channel_id: &[u8; 32],
    counterparty_node_id: &PublicKey,
    channel_value_satoshis: Amount,
    output_script: ScriptBuf,
    user_channel_id: u128,
) {
    let raw_tx = bitcoind_client.create_raw_transaction(vec![TxOut {
        value: channel_value_satoshis,
        script_pubkey: output_script,
    }]);

    let funded_tx = bitcoind_client.fund_raw_transaction(raw_tx);

    let signed_tx = bitcoind_client.sign_raw_transaction_with_wallet(funded_tx);

    channel_manager.funding_transaction_generated(
        temporary_channel_id,
        counterparty_node_id,
        signed_tx,
    );
}

#[cfg(test)]
mod test;
