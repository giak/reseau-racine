use nostr::PublicKey;
use rr_core::{Cell, CellMember, CellStore};
use std::str::FromStr;
use uuid::Uuid;

fn dummy_pk() -> PublicKey {
    PublicKey::from_str("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798").unwrap()
}

#[test]
fn test_cell_roundtrip() {
    let cell = Cell::new(
        "test-cell",
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        Vec::new(),
        vec![CellMember::new(dummy_pk(), Some("Alice".to_string()))],
    );
    let mut store = CellStore::default();
    store.add(cell);
    let json = serde_json::to_string_pretty(&store).unwrap();
    let parsed: CellStore = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.all().len(), 1);
    assert_eq!(parsed.all().first().unwrap().label, "test-cell");
}

#[test]
fn test_cell_store_find() {
    let cell = Cell::new(
        "find-me",
        "deadbeef".to_string(),
        Vec::new(),
        vec![CellMember::new(dummy_pk(), None)],
    );
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    assert!(store.find(&id).is_some());
    assert!(store.find(&Uuid::new_v4()).is_none());
}

#[test]
fn test_cell_store_add_remove() {
    let cell = Cell::new(
        "tmp",
        "key".to_string(),
        Vec::new(),
        vec![CellMember::new(dummy_pk(), None)],
    );
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    assert_eq!(store.all().len(), 1);
    store.remove(&id);
    assert_eq!(store.all().len(), 0);
}

#[test]
fn test_cell_store_update_members() {
    let cell = Cell::new(
        "growing",
        "key".to_string(),
        Vec::new(),
        vec![CellMember::new(dummy_pk(), None)],
    );
    let id = cell.id;
    let mut store = CellStore::default();
    store.add(cell);
    let new_member = CellMember::new(dummy_pk(), Some("Bob".to_string()));
    assert!(store.update_members(&id, vec![new_member]));
    assert_eq!(store.find(&id).unwrap().members.len(), 1);
    assert_eq!(
        store.find(&id).unwrap().members[0].label.as_deref(),
        Some("Bob")
    );
}

#[test]
fn test_cell_with_sender_keys_roundtrip() {
    use rr_core::SenderKey;
    let sk = SenderKey {
        member_pubkey: dummy_pk(),
        chain_key_hex: "ab".repeat(32),
        msg_count: 5,
        created_at_secs: 1700000000,
    };
    let cell = Cell::new(
        "sk-cell",
        "legacy_key_hex".to_string(),
        vec![sk],
        vec![CellMember::new(dummy_pk(), Some("Alice".to_string()))],
    );
    let mut store = CellStore::default();
    store.add(cell);
    let json = serde_json::to_string_pretty(&store).unwrap();
    let parsed: CellStore = serde_json::from_str(&json).unwrap();
    let cell = parsed.all().first().unwrap();
    assert_eq!(cell.sender_keys.len(), 1);
    assert_eq!(cell.sender_keys[0].msg_count, 5);
    assert_eq!(cell.sender_keys[0].chain_key_hex, "ab".repeat(32));
}

#[test]
fn test_cell_with_multiple_sender_keys() {
    use nostr::Keys;
    use rr_core::SenderKey;
    let pk2 = Keys::generate().public_key();
    let keys = vec![
        SenderKey {
            member_pubkey: dummy_pk(),
            chain_key_hex: "aa".repeat(32),
            msg_count: 0,
            created_at_secs: 1700000000,
        },
        SenderKey {
            member_pubkey: pk2,
            chain_key_hex: "bb".repeat(32),
            msg_count: 3,
            created_at_secs: 1700000001,
        },
    ];
    let cell = Cell::new("multi-sk", "key".to_string(), keys, vec![CellMember::new(dummy_pk(), None)]);
    let mut store = CellStore::default();
    store.add(cell);
    let json = serde_json::to_string_pretty(&store).unwrap();
    let parsed: CellStore = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.all().first().unwrap().sender_keys.len(), 2);
}

#[test]
fn test_legacy_cell_without_sender_keys() {
    let json = r#"{
        "cells": [{
            "id": "00000000-0000-0000-0000-000000000001",
            "label": "legacy",
            "cell_key_hex": "deadbeef",
            "sender_keys": [],
            "members": [{"pubkey": "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798", "label": null, "added_at_secs": 1700000000}],
            "created_at_secs": 1700000000
        }]
    }"#;
    let parsed: CellStore = serde_json::from_str(json).unwrap();
    let cell = parsed.all().first().unwrap();
    assert_eq!(cell.label, "legacy");
    assert!(cell.sender_keys.is_empty());
}
