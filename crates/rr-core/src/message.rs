use nostr::prelude::*;
use nostr_sdk::prelude::*;

pub async fn send_message(
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

pub async fn receive_message(
    client: &Client,
    gift_wrap: &Event,
) -> Result<UnwrappedGift, Box<dyn std::error::Error>> {
    let unwrapped = client.unwrap_gift_wrap(gift_wrap).await?;
    Ok(unwrapped)
}
