#!/bin/sh

# Fetch the validators_app_mainnet_beta.json file that contains validator data center and display info.  This should
# be fetched before running process_data.sh or collate_data.sh

# Argument: validators.app "secret API key"
# To get a secret API key, register with validators.app as a user

curl -H "Token: $1" 'https://www.validators.app/api/v1/validators/mainnet.json' > validators_app_mainnet_beta.json
