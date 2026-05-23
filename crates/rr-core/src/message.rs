use nostr::prelude::*;
use nostr_sdk::prelude::*;

#[derive(Debug, Clone)]
pub struct MessageService;

impl MessageService {
    pub fn new() -> Self {
        Self
    }

    pub async fn send(
        &self,
        client: &Client,
        receiver_pubkey: PublicKey,
        content: &str,
    ) -> Result<EventId, Box<dyn std::error::Error>> {
        let output = client
            .send_private_msg(receiver_pubkey, content, vec![])
            .await?;
        if output.success.is_empty() {
            let errors: Vec<String> = output
                .failed
                .iter()
                .map(|(url, err)| format!("{url}: {err}"))
                .collect();
            return Err(format!("Échec d'envoi: {}", errors.join("; ")).into());
        }
        Ok(*output)
    }

    pub async fn receive(
        &self,
        client: &Client,
        gift_wrap: &Event,
    ) -> Result<UnwrappedGift, Box<dyn std::error::Error>> {
        let unwrapped = client.unwrap_gift_wrap(gift_wrap).await?;
        Ok(unwrapped)
    }
}

impl Default for MessageService {
    fn default() -> Self {
        Self::new()
    }
}
