use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct NostrTransport {
    client: Client,
    relay_url: String,
}

impl NostrTransport {
    pub async fn new(relay_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let keys = Keys::generate();
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        client
            .wait_for_connection(std::time::Duration::from_secs(10))
            .await;
        Ok(Self {
            client,
            relay_url: relay_url.to_string(),
        })
    }

    pub async fn with_keys(
        relay_url: &str,
        keys: Keys,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::new(keys);
        client.add_relay(relay_url).await?;
        client.connect().await;
        client
            .wait_for_connection(std::time::Duration::from_secs(10))
            .await;
        Ok(Self {
            client,
            relay_url: relay_url.to_string(),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn relay_url(&self) -> &str {
        &self.relay_url
    }
}
