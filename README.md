# Trust service

This project is a microservice for creating and verifying proofs of data authenticity and data integrity. The application store the proofs on the [IOTA Tangle](https://wiki.iota.org/shimmer/). To answer the requirement that a user should not own crypto tokens, a centralized approach has been used where the service handles the identity keys of the user. The microservice also expose the API to mint an NFT representing a dataset.

## Getting started  

### Requirements

- `Rust` and `Cargo`. Follow the [documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install them. 
- `Docker` and `Docker compose`
-  Smart contracts already deployed on the same network that the trust-service will interact with. More information [here](https://github.com/MODERATE-Project/ipr-management).


## Run

Beware of the configuration of the environment variables.
Note: Modify `.env` and `.mongo.env` reasonably. (`ADDR`, `MONGO_ENDPOINT`, `ASSET_FACTORY_ADDR`,`L2_PRIVATE_KEY`)

### Locally

For testing the application with MongoDB, follow these steps:
- Run `docker compose --profile dev up -d` to start the MongoDB container.
- Create a database called `MODERATE`.
- Create a collection called `Users`.
- Use [MongoDB Compass](https://www.mongodb.com/products/compass) to view the database content.
Note: MongoDB Compass is a tool that can be used to interact with MongoDB databases and inspect their content.

Create the smart contract Rust bindings (mandatory the first time or if the smart contracts change)
```shell
cd abigen
# assuming the ipr-management folder is located in the same root folder of the trust-service
cargo run -- --contract AssetFactory --abi-source "../../ipr-management/artifacts/contracts/AssetFactory.sol/AssetFactory.json"
cargo run -- --contract Asset --abi-source "../../ipr-management/artifacts/contracts/Asset.sol/Asset.json"
```

Then, launch the application: 
```shell
cd actix-server
cargo run --release --bin actix-trust-service
```

### Via docker

Copy the smart contract json files to create the Rust bindings (mandatory the first time or if the smart contracts change)
```shell
    mkdir smart-contracts
    # assuming the ipr-management folder is located in the same root folder of the trust-service
    cp ../ipr-management/artifacts/contracts/AssetFactory.sol/AssetFactory.json ./smart-contracts
    cp ../ipr-management/artifacts/contracts/Asset.sol/Asset.json ./smart-contracts

```

Commands for building the app’s container image and starting the app container:
```shell
docker compose --profile deploy up -d
```

## Usage

<!-- Provide instructions and examples for use. Include screenshots as needed. -->
- [API Reference](./actix-server/api/specifications.yaml)
- [Postman Collection](./actix-server/api/Trust-service.postman_collection.json)


## License

[Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)