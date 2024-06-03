filename=$1
size=$2

set -e

circom $filename.circom --r1cs --wasm --sym --c
node ./"$filename"_js/generate_witness.js ./"$filename"_js/"$filename".wasm input.json witness.wtns
#
snarkjs powersoftau prepare phase2 pot"$size"_0000.ptau pot"$size"_final.ptau -v
snarkjs groth16 setup "$filename".r1cs pot"$size"_final.ptau "$filename"_0000.zkey
snarkjs zkey contribute "$filename"_0000.zkey "$filename"_0001.zkey --name="1st Contributor Name" -v
snarkjs zkey export verificationkey "$filename"_0001.zkey verification_key.json

snarkjs groth16 prove "$filename"_0001.zkey witness.wtns proof.json public.json
