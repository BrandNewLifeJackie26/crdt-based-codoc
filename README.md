# crdt-based-codoc

to run this web-assembly:

## Server
### Start Zookeeper
Under local zookeeper directory
1. run ```bin/zkServer.sh start conf/zoo_sample.cfg```

### Start wasm Server
Under root directory
1. run ```cargo run```

## Client
### Compile wasm From Rust
Under `./wasm_client`
1. run ```wasm-pack build```. Only need to run this if you modify the rust source code
2. if ./wasm_client/www not exist, run  ```npm init wasm-app www``` 

### Start Front End
Under `./wasm_client/www`
1. run ```npm install```
2. run ```npm run start```
3. visit `http://localhost:9000/`
