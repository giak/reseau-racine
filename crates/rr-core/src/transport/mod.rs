use nostr_sdk::prelude::*;

pub mod nostr;

pub trait TransportProvider: Send + Sync {
    fn client(&self) -> &Client;
    fn kind(&self) -> &'static str;
}

impl TransportProvider for nostr::NostrTransport {
    fn client(&self) -> &Client {
        self.client()
    }

    fn kind(&self) -> &'static str {
        "nostr"
    }
}
