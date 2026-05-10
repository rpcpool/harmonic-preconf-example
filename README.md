# preconf-example

Minimal Rust client for the preconf gRPC service. Authenticates with an
ed25519 keypair (challenge / signature → JWT) and streams `Preconf` messages.

## Layout

- `protos/preconf.proto` — service definition (auth + stream).
- `build.rs` — compiles the proto via `tonic-prost-build` at build time.
- `src/main.rs` — example client: load keypair → auth → subscribe → print.

## Run

```sh
cargo run --release -- \
    --url https://preconf.example.com:443 \
    --keypair ./keypair.json
```

The keypair must be authenticated to connect to the Preconf server.

## Protocol

1. `GenerateAuthChallenge(pubkey)` — server returns opaque challenge bytes.
2. `GenerateAuthTokens(pubkey, challenge, signature)` — server verifies the
   ed25519 signature over `challenge` and returns a short-lived JWT.
3. `SubscribePreconfs` with `Authorization: Bearer <JWT>` metadata — server
   streams `Preconf { slot, data }` where `data` is the raw serialized
   Solana transaction as the builder received it.
