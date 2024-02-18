### ErcStylus 

## Deploy

Deploy to Arbitrum Stylus testnet 
```
cargo stylus deploy \
  --private-key-path=./.secret/pk.key
```  

Can verify the deployment on the block explorer : https://stylus-testnet-explorer.arbitrum.io/

## Run scripts

Run rust script from examples directory:

`cargo run --example my_token --target=aarch64-apple-darwin`