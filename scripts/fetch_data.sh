#!/bin/sh

# Run this first to the data from google bigtables

# Arguments are: credentials_path first_epoch last_epoch

# Writes data into a directory 'data'

CREDENTIALS_PATH=$1
FIRST_EPOCH=$2
LAST_EPOCH=$3

if [ -z "$CREDENTIALS_PATH" -o -z "$FIRST_EPOCH" -o -z "$LAST_EPOCH" ]; then
    echo "Usage: fetch_data.sh <credendials_path> <first_epoch> <last_epoch>"
    exit -1
fi

mkdir -p data

C=0
for epoch in `seq $FIRST_EPOCH $LAST_EPOCH`; do
    (./target/release/fetch_data "$CREDENTIALS_PATH" $(($epoch*432000)) $(($(($(($epoch+1))*432000))-1)) | gzip -c > data/$epoch.gz) &
    pids[${C}]=$!
done

for pid in ${pids[*]}; do
    wait $pid
done
