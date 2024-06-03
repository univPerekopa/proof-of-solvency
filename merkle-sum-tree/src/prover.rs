use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;
use crate::merkle_tree::{MerkleTree, UserInfo};
use crate::traits::VectorCommitment;
use rand::Rng;

fn generate_vector(n: usize) -> Vec<UserInfo> {
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(n);

    for i in 0..n {
        let balance = rng.gen_range(0..100000);
        vec.push(UserInfo {
            email: format!("user_{i}@gmail.com"),
            balance,
        });
    }

    vec
}

mod merkle_tree;
mod traits;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args[1].clone().parse().unwrap();
    let users = generate_vector(n);

    let started_at = Instant::now();

    let tree = MerkleTree::init(users);
    let commitment = tree.get_commitment();
    dbg!(commitment);

    let file = File::create("commitment.json").unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &commitment).unwrap();
    writer.flush().unwrap();

    let mut proofs = Vec::new();
    for i in 0..n {
        let proof = tree.get_proof(i);
        proofs.push(proof);
    }
    let file = File::create(format!("proofs.json")).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &proofs).unwrap();
    writer.flush().unwrap();

    println!("Proof generation took {:?}", started_at.elapsed());
}

/*
2^10 -- commit 5ms    proof 9µs  verify 15µs, size 1450 B
2^11 -- commit 10ms   proof 10µs verify 16µs, size 1600 B
2^12 -- commit 19ms   proof 11µs verify 17µs, size 1750 B
2^15 -- commit 150ms  proof 12µs verify 19µs, size 2170 B
2^20 -- commit 5750ms proof 14µs verify 26µs, size 2887 B
 */
