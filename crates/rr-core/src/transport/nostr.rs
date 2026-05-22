use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct NostrTransport {
    client: Client,
}

impl NostrTransport {
    pub async fn new(relay_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let keys = Keys::generate();
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        Ok(Self { client })
    }

    pub async fn with_keys(relay_url: &str, keys: Keys) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        Ok(Self { client })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
