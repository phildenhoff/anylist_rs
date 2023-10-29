# anylist_rs

A Rust crate for interacting with [AnyList](https://www.anylist.com/)'s undocumented API.

## running scripts

to run the (Swift) login script,

```fish
swift login.swift <email> "<password>"
```

that will return the Signed user ID, which you can use for authentication, and whether your account is premium or not.

To run the Rust version (compiling & executing in one step):

```fish
cargo run --bin anylist -- login --email <email> --password "<password>"
```
