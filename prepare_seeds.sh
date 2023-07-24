#!/bin/bash

# Prepare seeds from glocier
mkdir -p seed-source
pushd seed-source
git clone https://github.com/rust-lang/glacier/tree/master
popd

# just make sure seeds directory exists
mkdir -p seeds

for file in seed-source/glacier/fixed/*.rs; do
    # Skip some problematic ones
    # still causing ICE on some version
    if [[ $file == *"79699.rs" ]]; then
        continue
    fi
    # still causing ICE on some version
    if [[ $file == *"89868.rs" ]]; then
        continue
    fi
    # copy as seed
    echo "Copying file $file as seed"
    cp $file seeds
done

echo "Finished setting up seeds directory."
