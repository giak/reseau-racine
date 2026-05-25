use nostr::Keys;
use nostr_sdk::Client;
use rr_core::CellTransport;

#[tokio::test]
#[ignore]
async fn test_cell_transport_create() {
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "ws://172.20.0.2:8080".to_string());
    let alice = Keys::generate();
    let bob = Keys::generate();
    let charlie = Keys::generate();

    let client = Client::new(alice.clone());
    client.add_relay(&relay).await.unwrap();
    client.connect().await;

    let transport = CellTransport::new(client, alice.clone());

    let cell = transport
        .create_cell("test-cell", &[bob.public_key(), charlie.public_key()])
        .await
        .expect("create_cell failed");

    assert_eq!(cell.label, "test-cell");
    assert_eq!(cell.members.len(), 3);

    let dave = Keys::generate();
    transport
        .invite_member(&cell.id, &dave.public_key())
        .await
        .unwrap();

    let store = rr_core::CellStore::load();
    let loaded = store.find(&cell.id).unwrap();
    assert_eq!(loaded.members.len(), 4);
}
