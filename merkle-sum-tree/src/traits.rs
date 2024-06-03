pub trait VectorCommitment<T, C, P> {
    #[allow(dead_code)]
    fn init(vector: Vec<T>) -> Self;

    #[allow(dead_code)]
    fn get_commitment(&self) -> C;

    #[allow(dead_code)]
    fn get_proof(&self, idx: usize) -> P;

    #[allow(dead_code)]
    fn verify(proof: P, commitment: C) -> bool;
}
