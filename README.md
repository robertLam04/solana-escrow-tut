### Environment Setup
1. Install Rust from https://rustup.rs/
2. Install Solana from https://docs.solana.com/cli/install-solana-cli-tools#use-solanas-install-tool

### Build and test for program compiled natively
```
$ cargo build
$ cargo test
```

### Build and test the program compiled for BPF
```
$ cargo build-bpf
$ cargo test-bpf
```

### Setup local net (in a new terminal)
```
$ solana-test-validator
```

### Deploy program
#### Ensure config is set to local host:
```
$ solana config get
$ solana config set --url https://127.0.0.1:8899
```
#### If you don't already have a keypair, make one:
```
$ solana-keygen new
```
#### Build and test the program compiled for BPF:
```
$ cargo build-bpf
$ cargo test-bpf
```
#### Deploy
```
$ solana program deploy <PATH_TO_.SO_FILE>
```
