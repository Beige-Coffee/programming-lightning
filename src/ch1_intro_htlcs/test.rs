#![allow(dead_code, unused_imports, unused_variables, unused_must_use)]
use crate::ch1_intro_htlcs::exercises::{
    build_commitment_transaction, build_funding_transaction, build_htlc_commitment_transaction,
    build_htlc_timeout_transaction, build_refund_transaction, generate_revocation_pubkey, to_local, two_of_two_multisig_witness_script,
};
use crate::internal::helper::{
    bitcoin_pubkey_from_private_key, pubkey_from_private_key, secp256k1_private_key,
};
use bitcoin::hash_types::Txid;
use bitcoin::script::ScriptBuf;
use bitcoin::secp256k1::PublicKey;
use bitcoin::secp256k1::{self, Secp256k1};
use bitcoin::PublicKey as BitcoinPublicKey;
use bitcoin::{OutPoint, Sequence, Transaction, TxIn, Witness};

/// hash160 of the empty string
const HASH160_DUMMY: [u8; 20] = [
    0xb4, 0x72, 0xa2, 0x66, 0xd0, 0xbd, 0x89, 0xc1, 0x37, 0x06, 0xa4, 0x13, 0x2c, 0xcf, 0xb1, 0x6f,
    0x7c, 0x3b, 0x9f, 0xcb,
];

#[test]
fn test_two_of_two_multisig_witness_script() {
    let alice_pubkey = pubkey_from_private_key(&[0x01; 32]);
    let bob_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let result = two_of_two_multisig_witness_script(&alice_pubkey, &bob_pubkey);

    let their_solution = format!("{}", result.script_hash());
    println!("their solution: {}", their_solution);
    let acceptable_solutions = [
        "d8fecda80c30e89a9e7f0964ee79ce055288bc1c".to_string(),
        "e5c7b40d1f14542cc2c7a15819de088a33c8f7ba".to_string(),
    ];

    assert!(acceptable_solutions.contains(&their_solution))
}

#[test]
fn test_build_funding_transaction() {
    let outpoint = OutPoint::new(
        "d9334caed6503ebc710d13a5f663f03bec531026d2bc786befdfdb8ef5aad721"
            .parse::<Txid>()
            .unwrap(),
        1,
    );

    let txin = vec![TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    }];

    let pubkey1 = pubkey_from_private_key(&[0x01; 32]);
    let pubkey2 = pubkey_from_private_key(&[0x02; 32]);

    let amount: u64 = 100000;

    let transaction = build_funding_transaction(txin, &pubkey2, &pubkey1, amount);

    let their_solution = transaction.compute_txid().to_string();

    println!("their solution: {}", their_solution);

    let acceptable_solutions = [
        "8f28cec85c8d986559c7bf5760d57d57446e26f27ac3ed623d591e4579b7bc9c".to_string(),
        "90c0c517ef964f7082837865a12702b5c8bc4f45500b59a9ee09d1e8f681bfd4".to_string(),
    ];

    assert!(acceptable_solutions.contains(&their_solution));
}

#[test]
fn test_build_refund_transaction() {
    let outpoint = OutPoint::new(
        "d9334caed6503ebc710d13a5f663f03bec531026d2bc786befdfdb8ef5aad721"
            .parse::<Txid>()
            .unwrap(),
        1,
    );

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let pubkey1 = pubkey_from_private_key(&[0x01; 32]);
    let pubkey2 = pubkey_from_private_key(&[0x02; 32]);

    let alice_amount: u64 = 100000;
    let bob_amount: u64 = 100000;

    let transaction = build_refund_transaction(txin, pubkey1, pubkey2, alice_amount, bob_amount);

    let their_solution = transaction.compute_txid().to_string();

    println!("their solution: {}", their_solution);

    let acceptable_solutions = [
        "5964f26f1734b139b30b71a9d37bb10c56422d2cd81f9684d7a9d27bd504ff01".to_string(),
        "52c346e97b9847178eaccd11341cc8a1266ba83f78c60fef282cd7b951c3ab4f".to_string(),
    ];

    assert!(acceptable_solutions.contains(&their_solution));
}

#[test]
fn test_generate_revocation_pubkey() {
    let countersignatory_basepoint = pubkey_from_private_key(&[0x01; 32]);
    let per_commitment_point = pubkey_from_private_key(&[0x02; 32]);

    let revocation_pubkey =
        generate_revocation_pubkey(countersignatory_basepoint, per_commitment_point);

    let actual = revocation_pubkey.to_string();

    let expected = "02f406b2b2ef2372c08d003e5062e4b34929b86f107a36bfbd406f5644419e9ff6";

    assert_eq!(
        actual, expected,
        "Revocation pubkey doesn't match expected value"
    );
}

#[test]
fn test_to_local() {
    let revocation_key = pubkey_from_private_key(&[0x01; 32]);
    let to_local_delayed_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let to_self_delay: i64 = 144;
    let result = to_local(&revocation_key, &to_local_delayed_pubkey, to_self_delay);

    let their_solution = format!("{}", result.script_hash());
    println!("their solution: {}", their_solution);
    let acceptable_solutions = ["6c61fd62fe96fc6aa5485a820ce87d662329f4ab".to_string()];

    assert!(acceptable_solutions.contains(&their_solution))
}

#[test]
fn test_build_commitment_transaction() {
    let outpoint = OutPoint::new(
        "d9334caed6503ebc710d13a5f663f03bec531026d2bc786befdfdb8ef5aad721"
            .parse::<Txid>()
            .unwrap(),
        1,
    );

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let revocation_pubkey = pubkey_from_private_key(&[0x01; 32]);
    let to_local_delayed_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let remote_pubkey = pubkey_from_private_key(&[0x03; 32]);

    let to_self_delay: i64 = 144;
    let alice_amount: u64 = 100000;
    let bob_amount: u64 = 100000;

    let transaction = build_commitment_transaction(
        txin,
        &revocation_pubkey,
        &to_local_delayed_pubkey,
        remote_pubkey,
        to_self_delay,
        alice_amount,
        bob_amount,
    );

    let their_solution = transaction.compute_txid().to_string();

    println!("their solution: {}", their_solution);

    let acceptable_solutions =
        ["e0da6b5c753ab8a22a65b6f9bcc0460f2724f244121f62ddc05c35dbf42719cd".to_string()];

    assert!(acceptable_solutions.contains(&their_solution));
}

#[test]
fn test_build_htlc_commitment_transaction() {
    let outpoint = OutPoint::new(
        "d9334caed6503ebc710d13a5f663f03bec531026d2bc786befdfdb8ef5aad721"
            .parse::<Txid>()
            .unwrap(),
        1,
    );

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let revocation_pubkey = pubkey_from_private_key(&[0x01; 32]);
    let remote_htlc_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let local_htlc_pubkey = pubkey_from_private_key(&[0x03; 32]);
    let to_local_delayed_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let remote_pubkey = pubkey_from_private_key(&[0x02; 32]);

    let to_self_delay: i64 = 144;
    let payment_hash160 = HASH160_DUMMY;
    let htlc_amount: u64 = 100000;
    let local_amount: u64 = 100000;
    let remote_amount: u64 = 100000;

    let transaction = build_htlc_commitment_transaction(
        txin,
        &revocation_pubkey,
        &remote_htlc_pubkey,
        &local_htlc_pubkey,
        &to_local_delayed_pubkey,
        remote_pubkey,
        to_self_delay,
        &payment_hash160,
        htlc_amount,
        local_amount,
        remote_amount,
    );

    let their_solution = transaction.compute_txid().to_string();

    println!("their solution: {}", their_solution);

    let acceptable_solutions =
        ["d8509e0413e1d72f7ff35201a4b62308dfca839613568b62c0274ba721a2da3b".to_string()];

    assert!(acceptable_solutions.contains(&their_solution));
}

#[test]
fn test_build_htlc_timeout_transaction() {
    let outpoint = OutPoint::new(
        "d9334caed6503ebc710d13a5f663f03bec531026d2bc786befdfdb8ef5aad721"
            .parse::<Txid>()
            .unwrap(),
        1,
    );

    let txin = TxIn {
        previous_output: outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
    };

    let revocation_pubkey = pubkey_from_private_key(&[0x01; 32]);
    let broadcaster_delayed_payment_key = pubkey_from_private_key(&[0x02; 32]);
    let local_htlc_pubkey = pubkey_from_private_key(&[0x03; 32]);
    let to_local_delayed_pubkey = pubkey_from_private_key(&[0x02; 32]);
    let remote_pubkey = pubkey_from_private_key(&[0x02; 32]);

    let contest_delay: i64 = 144;
    let cltv_expiry: u32 = 300;
    let htlc_amount: u64 = 100000;

    let transaction = build_htlc_timeout_transaction(
        txin,
        &revocation_pubkey,
        &broadcaster_delayed_payment_key,
        contest_delay,
        cltv_expiry,
        htlc_amount,
    );

    let their_solution = transaction.compute_txid().to_string();

    println!("their solution: {}", their_solution);

    let acceptable_solutions =
        ["f590bc5b3b48fd28e0c8f79dd2cad685dc2c0034cb12ebfaa9c933ffbe433fb7".to_string()];

    assert!(acceptable_solutions.contains(&their_solution));
}
