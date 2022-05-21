This is the code and scripts used to collect and process data for the timely vote credits proposal.

If you want to fetch and analyze data:

STEP 1 ---------------------------------------------------------------------

First, 'cargo build --release' to build all the programs.

STEP 2 ---------------------------------------------------------------------

Next, you must acquire Solana Foundation Google Bigtables access credentials.  Contact them for that.

Now, fetch the data for the epochs you are interested in:

./scripts/fetch_data.sh <path_to_your_credentials_file> <first_epoch> <last_epoch>

(the above will take a LONG time -- hours -- as downloading all of the data from Google Bigtables is really slow)
(and it will make a new directory "data" with the data)

The data is in compressed files, one per epoch, and whose contents are lines of the form:

SLOT VOTE_ID VOTE_SLOT [VOTE_SLOT...]

In other words, a SLOT, the VOTE_ID of the validator who voted, and a list of VOTE_SLOTs which are each a slot
cast by the validator that successfully landed in SLOT.

STEP 3 ---------------------------------------------------------------------

After that, ensure that you have a validators.app API query key.  See the validators.app website for details on
how to get that.

Now, query validators.app to get the data center + validator name/icon info from validators.app for all validators:

./scripts/fetch_validators_app_mainnet_beta.sh <secret_api_key>
(this will write a file validators_app_mainnet_beta.json)

STEP 4 ---------------------------------------------------------------------

Next, process the raw data.  This will turn it into a much smaller per-validator data set that is more easily
operated on by subsequent commands.

./scripts/process_data.sh

This will take some time - minutes - as it has to read and process all of that fetched data.

This will create a ./data_processed directory and put a file in it for every epoch.  These files are of the form:

DATA_CENTER VOTE_ID TOTAL_TRANSACTIONS TOTAL_VOTE_CREDITS LIST...

DATA_CENTER and VOTE_ID identify the validator.
TOTAL_TRANSCATIONS is the total number of vote transactions successfully landed by that validator in the epoch.
TOTAL_VOTE_CREDITS is the total number of vote credits successfully landed by that validator in the epoch.

The LIST is 64 integers in sequence.  Each one is the "number of votes landed at that latency".  So for example:

100 50 0 7 ...

Would mean that the validator landed votes on 100 slots at "latency 0" (i.e. in the slot immediately after the slot being voted on), and landed 50 slots at "latency 1" (i.e. with 1 slot latency), and landed 0 slots at "latency 2", etc.

STEP 5 ---------------------------------------------------------------------

Next, compute a set of "timely vote credits" results that give details of what the actual results of the
timely vote credits voting mechanism would be for different parameterizations of that algorithm.

Examples:

./scripts/calculate_results.sh 4 60 1
./scripts/calculate_results.sh 4 60 1.5
./scripts/calculate_results.sh 4 44 1
./scripts/calculate_results.sh 4 44 1.5

The results will be written into sub-directories under 'results', and within each of those subdirectories,
one file per epoch for the results for that epoch.


STEP 6 ---------------------------------------------------------------------

Turn the results into html pages that are easier to analyze than the raw data files.

./scripts/collate_results

The results will be written under the 'html' directory.
