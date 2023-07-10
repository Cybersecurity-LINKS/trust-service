# Trust service

<div align="center">
  [![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
  ![Iota](https://img.shields.io/badge/iota-29334C?style=for-the-badge&logo=iota&logoColor=white)
</div>

## Description

<!-- Provide a short description explaining the what, why, and how of your project. Use the following questions as a guide:

- What was your motivation?
- Why did you build this project? (Note: the answer is not "Because it was a homework assignment.")
- What problem does it solve?
- What did you learn? --> 

This project is a microservice for creating and verifying proofs of data authenticity and data integrity. The application store the proofs on the [IOTA Tangle](https://wiki.iota.org/shimmer/). To answer the requirement that a user should not own crypto tokens, a centralized approach has been used where the service handles the identity keys of the user.

## Installation
<!-- What are the steps required to install your project? Provide a step-by-step description of how to get the development environment running. -->

### Requirements

- `Rust` and `Cargo`. Follow the [documentation](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install them. 
- `Docker`


### Running locally


For testing the application with MongoDB, follow these steps:

- Run `docker compose up -d` to start the MongoDB container.
- Create a database called `MODERATE`.
- Create a collection called `Users`.
- Use [MongoDB Compass](https://www.mongodb.com/products/compass) to view the database content.
Note: MongoDB Compass is a tool that can be used to interact with MongoDB databases and inspect their content.

For launching the application: 
```
cd actix-server
cargo run
```

Beware of the configuration of the environment variables.

### Running via docker

Commands for building the app’s container image and starting the app container:

```shell
docker build -t ts .
docker run -dp 8080:8080 ts  
```

## Usage

<!-- Provide instructions and examples for use. Include screenshots as needed. -->
- [API Reference](./actix-server/api/specifications.yaml)


## License

[Apache-2.0](http://www.apache.org/licenses/LICENSE-2.0)