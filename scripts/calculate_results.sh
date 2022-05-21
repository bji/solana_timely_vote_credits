#!/bin/sh

# Run this after process_data has written the epoch data into data_processed

# Arguments are GRACE MAX_CREDITS MULTIPLIER

# Writes results into "results"

GRACE_PERIOD=$1
MAX_CREDITS=$2
REDUCTION_FACTOR=$3

if [ -z "$GRACE_PERIOD" -o -z "$MAX_CREDITS" -o -z "$REDUCTION_FACTOR" ]; then
    echo "Usage: calculate_results.sh <grace_period> <max_credits> <reduction_factor>"
    exit -1
fi

EPOCHS=$(for i in $(cd data_processed; ls); do echo -n "$i "; done)

mkdir -p results

DIR="${GRACE_PERIOD}_${MAX_CREDITS}_${REDUCTION_FACTOR}"

mkdir -p results/$DIR

for epoch in $EPOCHS; do

    ./target/release/calculate_results v $GRACE_PERIOD $MAX_CREDITS $REDUCTION_FACTOR < data_processed/$epoch > results/$DIR/v_$epoch
    
    ./target/release/calculate_results d $GRACE_PERIOD $MAX_CREDITS $REDUCTION_FACTOR < data_processed/$epoch > results/$DIR/d_$epoch
    
done
