#!/bin/sh

mkdir -p timely_voting_proposal

mkdir -p timely_voting_proposal/avg
cp sorttable.js timely_voting_proposal/avg

EPOCHS=$(for i in $(cd data_processed; ls); do echo -n "$i "; done)

PARAM_DIRS=$(for i in $(cd results; ls); do echo -n "$i "; done)

for n in $PARAM_DIRS; do

    PARAMS=$(echo -n $n | tr '_' ' ')

    for i in $EPOCHS; do
        mkdir -p timely_voting_proposal/$i
        cp sorttable.js timely_voting_proposal/$i

        TITLE="Epoch $i Params $PARAMS Validators"
        (echo "<html><head><title>$TITLE</title><script src=\"sorttable.js\"></script></head><body><h1>$TITLE</h1><p>Note: sort by column by clicking on the column header<p>"; ./target/release/collate_results v ./validators_app_mainnet_beta.json < results/$n/v_$i; echo "</body></html>") > timely_voting_proposal/$i/v_$n.html

        TITLE="Epoch $i Params $PARAMS Data Centers"
        (echo "<html><head><title>$TITLE</title><script src=\"sorttable.js\"></script></head><body><h1>$TITLE</h1><p><p>Note: sort by column by clicking on the column header<p>"; ./target/release/collate_results d ./validators_app_mainnet_beta.json < results/$n/d_$i; echo "</body></html>") > timely_voting_proposal/$i/d_$n.html
    done

    TITLE="Average for epochs ($EPOCHS) Params $PARAMS Validators"

    (echo "<html><head><title>$TITLE</title><script src=\"sorttable.js\"></script></head><body><h1>$TITLE</h1><p><p>Note: sort by column by clicking on the column header<p>"; (for i in $EPOCHS; do cat results/$n/v_$i; done) | ./target/release/collate_results v ./validators_app_mainnet_beta.json; echo "</body></html>") > timely_voting_proposal/avg/v_$n.html
    
    TITLE="Average for epochs ($EPOCHS) Params $PARAMS Data Centers"

    (echo "<html><head><title>$TITLE</title><script src=\"sorttable.js\"></script></head><body><h1>$TITLE</h1><p><p>Note: sort by column by clicking on the column header<p>"; (for i in $EPOCHS; do cat results/$n/d_$i; done) | ./target/release/collate_results d ./validators_app_mainnet_beta.json; echo "</body></html>") > timely_voting_proposal/avg/d_$n.html
    
done
