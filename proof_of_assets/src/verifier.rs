use std::fs::File;
use std::io::BufReader;
use web3::{transports::Http};
use web3::signing::{hash_message, recover};
use web3::types::{U256, Address, Recovery, H256};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("signatures.json").unwrap();
    let reader = BufReader::new(file);
    let signatures: Vec<(u64, H256, H256)> = serde_json::from_reader(reader).unwrap();

    // Create a provider
    let provider = Http::new("https://sepolia.drpc.org")?;
    let provider = web3::Web3::new(provider);

    // Sum up the balances
    let mut total_balance = U256::zero();

    for signature in signatures {
        // Recover the address
        let address = recover_address(signature)?;
        dbg!(address);

        // Get the balance
        let balance = provider.eth().balance(address, None).await?;

        // Add to the total balance
        total_balance += balance;
    }

    println!("Total balance: {}", total_balance);
    Ok(())
}

fn recover_address(signature: (u64, H256, H256)) -> Result<Address, Box<dyn std::error::Error>> {
    // Hash the message
    let message = "Address is owned by exchange";
    let hash = hash_message(message.as_bytes());

    let recovery = Recovery::new(hash, signature.0, signature.1, signature.2);
    let (signature, recovery_id) = recovery
        .as_signature().unwrap();

    // Recover the address
    let address = recover(hash.as_bytes(), &signature, recovery_id).unwrap();

    Ok(address)
}
