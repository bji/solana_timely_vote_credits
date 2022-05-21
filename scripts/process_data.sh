#!/bin/sh

# Run this after the data has been pulled from Google Bigtables by the 'fetch_data.sh' script

mkdir -p data_processed

ALL_EPOCHS=$(cd data; for i in $(ls); do echo $i | cut -d '.' -f 1; done)

C=0
for epoch in $ALL_EPOCHS; do
    (gunzip -c data/$epoch.gz | ./target/release/process_data validators_app_mainnet_beta.json > data_processed/$epoch) &
    pids[${C}]=$!
done

for pid in ${pids[*]}; do
    wait $pid
done
