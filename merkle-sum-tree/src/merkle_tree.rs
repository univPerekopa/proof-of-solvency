use crate::traits::VectorCommitment;
use keccak_hash::keccak;
use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub balance: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub element: UserInfo,
    pub idx: usize,
    pub path: Vec<Node>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub sum: u128,
    pub hash: Hash,
}

#[derive(Debug)]
pub struct MerkleTree {
    leafs: Vec<UserInfo>,
    nodes: Vec<Node>,
}

impl MerkleTree {
    pub fn hash_leaf(leaf: UserInfo) -> Hash {
        let hash = keccak(leaf.email.as_bytes());
        let mut concat = [0u8; 48];
        concat[0..32].copy_from_slice(&hash.0);
        concat[32..48].copy_from_slice(&leaf.balance.to_be_bytes());

        keccak(&concat).0
    }

    pub fn hash_nodes(left: Node, right: Node) -> Hash {
        let mut concat = [0u8; 96];
        let mut offset = 0;
        concat[offset..offset + 32].copy_from_slice(&left.hash);
        offset += 32;

        concat[offset..offset + 16].copy_from_slice(&left.sum.to_be_bytes());
        offset += 16;

        concat[offset..offset + 32].copy_from_slice(&right.hash);
        offset += 32;

        concat[offset..offset + 16].copy_from_slice(&right.sum.to_be_bytes());

        keccak(&concat).0
    }

    pub fn combine_nodes(left: Node, right: Node) -> Node {
        let sum = left.sum + right.sum;
        let hash = Self::hash_nodes(left, right);
        Node {
            sum,
            hash,
        }
    }

    pub fn new(mut leafs: Vec<UserInfo>) -> Self {
        assert!(leafs.len() > 0);

        let number_of_nodes = leafs.len().next_power_of_two() * 2;
        leafs.resize(number_of_nodes / 2, Default::default());
        let mut nodes = vec![Default::default(); number_of_nodes];

        for i in (number_of_nodes / 2)..number_of_nodes {
            let leaf = leafs[i - number_of_nodes / 2].clone();
            nodes[i] = Node {
                hash: Self::hash_leaf(leaf),
                sum: leafs[i - number_of_nodes / 2].balance,
            };
        }
        for i in (1..(number_of_nodes / 2)).rev() {
            nodes[i] = Self::combine_nodes(nodes[i << 1], nodes[(i << 1) + 1])
        }

        Self { leafs, nodes }
    }

    pub fn root(&self) -> Node {
        self.nodes[1]
    }

    pub fn height(&self) -> u32 {
        self.nodes.len().ilog2() - 1
    }

    pub fn get_elem(&self, idx: usize) -> UserInfo {
        self.leafs[idx].clone()
    }
}

impl VectorCommitment<UserInfo, Node, Proof> for MerkleTree {
    fn init(vector: Vec<UserInfo>) -> Self {
        Self::new(vector)
    }

    fn get_commitment(&self) -> Node {
        self.root()
    }

    fn get_proof(&self, idx: usize) -> Proof {
        let element = self.get_elem(idx);
        let mut path = vec![Default::default(); self.height() as usize];

        let mut curr_idx = idx + self.nodes.len() / 2;
        for i in 0..path.len() {
            path[i] = self.nodes[curr_idx ^ 1];
            curr_idx >>= 1;
        }

        Proof { element, idx, path }
    }

    fn verify(proof: Proof, commitment: Node) -> bool {
        let mut curr_node = Node {
            sum: proof.element.balance,
            hash: Self::hash_leaf(proof.element)
        };

        let height = proof.path.len();
        let mut curr_idx = proof.idx + (1usize << height);
        for neighbor in proof.path {
            curr_node = if curr_idx % 2 == 1 {
                Self::combine_nodes(neighbor, curr_node)
            } else {
                Self::combine_nodes(curr_node, neighbor)
            };
            curr_idx >>= 1;
        }

        curr_node == commitment
    }
}
