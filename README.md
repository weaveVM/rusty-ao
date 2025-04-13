<p align="center">
  <a href="https://wvm.dev">
    <img src="./assets/banner.png">
  </a>
</p>

## About
A blazingly fast client for [AO](https://ao.arweave.dev) written in Rust.

## Install

```bash
cargo add rusty_ao
```

Alternatively, in your `Cargo.toml`, add:

```Cargo.toml
[dependencies]
rusty_ao = { git = "https://github.com/weaveVM/rusty-ao.git", branch = "main" }
```

## Usage Examples: HyperBEAM 

### Init HyperBEAM client

```rust
use rusty_ao::hyperbeam::Hyperbeam;

pub async fn init_hb() {
    let hb = Hyperbeam::new(
        "https://tee-1.forward.computer".to_string(),
        SignerTypes::Arweave("test_key.json".to_string()),
    )
    .unwrap();
}
```

### Get a process last computed message state

Returns the `/Results` key of the latest computed message -- `~process@1.0`

```rust
let process_id = "oQZQd1-MztVOxODecwrxFR9UGUnsrX5wGseMJ9iSH38";
let state = hb.process_now(process_id.to_string()).await.unwrap();
```


## Usage Examples: Legacy 

### Init an AO client

```rust
// import the crate
use rusty_ao::ao::Legacy;
// Initialize an AO client 
let ao = Legacy::new(
  "https://mu.ao-testnet.xyz".to_string(),
  "https://cu.ao-testnet.xyz".to_string(),
  SignerTypes::Arweave("test_key.json".to_string()),
  )
  .unwrap();
```

or using the `default_init` method

```rust
let ao = Legacy::default_init(SignerTypes::Arweave("test_key.json".to_string()))
  .unwrap();
```
### Dry run an AO process message call

```rust
// let ao = ...init AO...

let res = ao
    .dry_run(
        "xU9zFkq3X2ZQ6olwNVvr1vUWIjc3kXTWr7xKQD6dh10".to_string(),
        "".to_string(),
        vec![Tag {
            name: "Action".to_string(),
            value: "Info".to_string(),
        }],
    )
    .await;

assert!(res.is_ok());
println!("{}", serde_json::to_string(&res.unwrap()).unwrap());
```

### Spawn a new process

```rust
// let ao = ...init AO...

let res = ao
    .spawn(
        "test1".to_string(),
        "rusty-ao".to_string(),
        DEFAULT_MODULE.to_string(),
        DEFAULT_SCHEDULER.to_string(),
        vec![],
    )
    .await;

println!("{:?}", res);
assert!(res.is_ok());
println!("{}", serde_json::to_string(&res.unwrap()).unwrap());
```
### Request CU get process result

```rust
// let ao = ...init AO...

let res = ao
    .get(
        "ya9XinY0qXeYyf7HXANqzOiKns8yiXZoDtFqUMXkX0Q".to_string(),
        "5JtjkYy1hk0Zce5mP6gDWIOdt9rCSQAFX-K9jZnqniw".to_string(),
    )
    .await;

println!("{:?}", res);
assert!(res.is_ok());
println!("{}", serde_json::to_string(&res.unwrap()).unwrap());
```

## Credits
- goao: Golang SDK for interacting with ao processes. [link](https://github.com/permadao/goao)

## License
This project is licensed under the [MIT License](./LICENSE)
