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
