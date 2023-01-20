# crisp-liquidation-bot
Bot which automatically liquidates borrow position on crisp-exchange contract. Notice that you have to own enough tokens on your exchange balance!

## Requirements

- NEAR 3.4.2
- Rust 1.64.0

## Setup

Install near-cli using instructions found [here](https://docs.near.org/tools/near-cli). 

Install rust using [this](https://www.rust-lang.org/tools/install).

## Usage

Write your account_id and secret_key in main.rs:
```
pub const ACCOUNT_ID: &str = "abobac.testnet";
pub const SECRET_KEY: &str = "ed25519:4SSA3XVDM8Z8YaajAMQ8zFomDJbWNsuZc7gJmgAoKPphJHxbieUJ4Weieu6k8g5wDcybZTuGLwT83gcvoikdgSzo";
```
Run bot
```
cargo run
```
