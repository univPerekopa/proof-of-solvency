pragma circom 2.0.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";

template MerkleTree(H) {
    // N = 2^H
    var N = 1;
    for (var i = 0; i < H; i++) {
        N = N * 2;
    }

    // Input/Output.
    signal input leafs[N];
    signal output rootHash;
    signal output sum;

    signal nodes[N * 2];
    for (var i = N; i < N * 2; i++) {
        nodes[i] <== leafs[i - N];
    }

    component poseidon[N];
    for (var i = N - 1; i >= 1; i--) {
        poseidon[i] = Poseidon(2);

        poseidon[i].inputs[0] <== nodes[i * 2];
        poseidon[i].inputs[1] <== nodes[i * 2 + 1];

        nodes[i] <== poseidon[i].out;
    }

    signal prefixSums[N];
    prefixSums[0] <== leafs[0];
    for (var i = 1; i < N; i++) {
        prefixSums[i] <== prefixSums[i - 1] + leafs[i];
    }

    rootHash <== nodes[1];
    sum <== prefixSums[N - 1];
}

component main = MerkleTree(11);
