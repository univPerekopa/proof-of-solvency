use std::fs::File;
use std::io::{BufWriter, Write};
use web3::{transports::Http};
use web3::signing::{SecretKey, Key};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Http::new("https://sepolia.drpc.org")?;
    let provider = web3::Web3::new(provider);

    let private_keys = std::env::var("PRIVATE_KEYS")?
        .split(',')
        .map(|key| key.to_string())
        .collect::<Vec<String>>();
    let message = "Address is owned by exchange";

    let mut signatures = Vec::new();
    for key in private_keys {
        let key = SecretKey::from_str(&key).unwrap();
        dbg!((&key).address());
        let sign = provider.accounts().sign(message.as_bytes(), &key);

        signatures.push((sign.v, sign.r, sign.s))
    }

    let file = File::create(format!("signatures.json")).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &signatures).unwrap();
    writer.flush().unwrap();

    Ok(())
}
