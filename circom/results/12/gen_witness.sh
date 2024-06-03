set -e

circom merkle_tree.circom --r1cs --wasm --sym --c
node ./merkle_tree_js/generate_witness.js ./merkle_tree_js/merkle_tree.wasm input.json witness.wtns
