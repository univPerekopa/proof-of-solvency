use std::fs::File;
use std::io::{BufReader};
use std::time::Instant;
use crate::merkle_tree::{MerkleTree, Proof};
use crate::traits::VectorCommitment;

mod merkle_tree;
mod traits;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let i: usize = args[1].clone().parse().unwrap();

    let file = File::open("commitment.json").unwrap();
    let reader = BufReader::new(file);
    let commitment = serde_json::from_reader(reader).unwrap();

    let file = File::open("proofs.json").unwrap();
    let reader = BufReader::new(file);
    let proofs: Vec<Proof> = serde_json::from_reader(reader).unwrap();

    let started_at = Instant::now();
    let result = MerkleTree::verify(proofs[i].clone(), commitment);
    assert!(result);
    println!("Verification took {:?}", started_at.elapsed());
}

/*
2^11 -- 16µs, 1600 B
2^12 -- 17µs, 1750 B
2^15 -- 19µs, 2170 B
2^20 -- 26.375µs, 2887 B
 */