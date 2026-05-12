# preconf-example

Minimal Rust client for the preconf gRPC service. Authenticates with an
ed25519 keypair (challenge / signature → JWT) and streams `Preconf` messages.

## IMPORTANT!

For a given Preconf server, Preconfs will _only be sent_ during the leader window of a validator connected to _that Harmonic region_.  This may make single-region connections appear to have sparse traffic.

The Preconf Protocol is currently in alpha testing, and breaking changes may be pushed at the sole discretion of the Harmonic team.

Join the [Harmonic Discord](https://discord.gg/sCWBcNHHeN) for API update announcements.

## Layout

- `protos/preconf.proto` — service definition (auth + stream).
- `build.rs` — compiles the proto via `tonic-prost-build` at build time.
- `src/main.rs` — example client: load keypair → auth → subscribe → print.

## Run

```sh
cargo run --release -- \
    --url http://ams.be.harmonic.gg:12349 \
    --keypair ./keypair.json
```

The keypair must be authenticated to connect to the Preconf server.

## Endpoints

ams - `http://ams.be.harmonic.gg:12349`
fra - `http://fra.be.harmonic.gg:12349`
lon - `http://lon.be.harmonic.gg:12349`
ewr - `http://ewr.be.harmonic.gg:12349`
sgp - `http://sgp.be.harmonic.gg:12349`
tyo - `http://tyo.be.harmonic.gg:12349`

## Protocol

1. `GenerateAuthChallenge(pubkey)` — server returns opaque challenge bytes.
2. `GenerateAuthTokens(pubkey, challenge, signature)` — server verifies the
   ed25519 signature over `challenge` and returns a short-lived JWT.
3. `SubscribePreconfs` with `Authorization: Bearer <JWT>` metadata — server
   streams `Preconf { slot, data }` where `data` is the raw serialized
   Solana transaction as the builder received it.
