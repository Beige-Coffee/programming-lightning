#![allow(dead_code, unused_imports, unused_variables, unused_must_use)]
use crate::internal;
use bitcoin::script::ScriptBuf;
use internal::builder::Builder;
use internal::helper::{tweak_pubkey, hash_pubkeys,
                      build_output, build_transaction, p2wpkh_output_script,
                      build_htlc_offerer_witness_script, add_pubkeys};
use bitcoin::blockdata::opcodes::all as opcodes;
use bitcoin::secp256k1::PublicKey;
use bitcoin::PublicKey as BitcoinPublicKey;
use bitcoin::hashes::Hash;
use bitcoin::{Block, OutPoint, PubkeyHash, Sequence, Transaction, TxIn, TxOut, Witness};
use bitcoin::transaction::Version;
use bitcoin::locktime::absolute::LockTime;

pub fn two_of_two_multisig_witness_script(
    pubkey1: &PublicKey,
    pubkey2: &PublicKey,
) -> ScriptBuf {
    
    Builder::new()
        .push_int(2)
        .push_key(pubkey1)
        .push_key(pubkey2)
        .push_int(2)
        .push_opcode(opcodes::OP_CHECKMULTISIG)
    .into_script()
}

pub fn timelocked_p2pkh(pubkey: &PublicKey, blocks_or_seconds: i64) -> ScriptBuf {
    
    Builder::new()
        .push_int(blocks_or_seconds)
        .push_opcode(opcodes::OP_CSV)
        .push_opcode(opcodes::OP_DROP)
        .push_opcode(opcodes::OP_DUP)
        .push_opcode(opcodes::OP_HASH160)
        .push_pubkey_hash(pubkey)
        .push_opcode(opcodes::OP_EQUALVERIFY)
        .push_opcode(opcodes::OP_CHECKSIG)
        .into_script()
    
}

pub fn build_funding_transaction(
    txins: Vec<TxIn>,
    alice_pubkey: &PublicKey,
    bob_pubkey: &PublicKey,
    amount: u64,
) -> Transaction {

    let output_script = two_of_two_multisig_witness_script(alice_pubkey, bob_pubkey);

    let output = build_output(amount, output_script.to_p2wsh());

    let version = Version::TWO;
    let locktime = LockTime::ZERO;

    let transaction = build_transaction(version, locktime, txins, vec![output]);

    transaction
}

pub fn build_refund_transaction(
    funding_txin: TxIn,
    alice_pubkey: PublicKey,
    bob_pubkey: PublicKey,
    alice_balance: u64,
    bob_balance: u64
) -> Transaction {
    
    let alice_output_script = p2wpkh_output_script(alice_pubkey);

    let bob_output_script = p2wpkh_output_script(bob_pubkey);

    
    let alice_output = build_output(alice_balance, alice_output_script);

    let bob_output = build_output(bob_balance, bob_output_script);

    let version = Version::TWO;
    let locktime = LockTime::ZERO;
    
    let transaction = build_transaction(version, locktime, vec![funding_txin],
                                       vec![alice_output, bob_output]);

    transaction

}

pub fn generate_revocation_pubkey(
    countersignatory_basepoint: PublicKey,
    per_commitment_point: PublicKey,
) -> PublicKey {
    
    todo!()
    
}

pub fn to_local(
    revocation_key: &PublicKey,
    to_local_delayed_pubkey: &PublicKey,
    to_self_delay: i64,
) -> ScriptBuf {

    todo!()
    
}

pub fn build_commitment_transaction(
    funding_txin: TxIn,
    revocation_pubkey: &PublicKey,
    to_local_delayed_pubkey: &PublicKey,
    remote_pubkey: PublicKey,
    to_self_delay: i64,
    local_amount: u64,
    remote_amount: u64,
) -> Transaction {
    
    todo!()
    
}

pub fn build_htlc_commitment_transaction(
    funding_txin: TxIn,
    revocation_pubkey: &PublicKey,
    remote_htlc_pubkey: &PublicKey,
    local_htlc_pubkey: &PublicKey,
    to_local_delayed_pubkey: &PublicKey,
    remote_pubkey: PublicKey,
    to_self_delay: i64,
    payment_hash160: &[u8; 20],
    htlc_amount: u64,
    local_amount: u64,
    remote_amount: u64,
) -> Transaction {

    todo!()
  
}

pub fn build_htlc_timeout_transaction(
    htlc_txin: TxIn,
    revocation_pubkey: &PublicKey,
    to_local_delayed_pubkey: &PublicKey,
    to_self_delay: i64,
    cltv_expiry: u32,
    htlc_amount: u64,
) -> Transaction {
    todo!()
}
