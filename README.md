# crypto_api

An HTTP API that encrypts, decrypts, signs and verifies JSON payloads, written in Rust with [axum](https://github.com/tokio-rs/axum).

| Endpoint        | Purpose                                                       |
| --------------- | ------------------------------------------------------------- |
| `POST /encrypt` | Encrypts every property at depth 1 of a JSON object           |
| `POST /decrypt` | Decrypts encrypted properties, leaves the others unchanged    |
| `POST /sign`    | Returns an HMAC-SHA256 signature of any JSON value            |
| `POST /verify`  | `204` if the signature matches the data, `400` otherwise      |
| `GET /health`   | Liveness probe                                                |

## Quick start

### With Cargo

Requires a recent stable Rust toolchain (edition 2024; CI and the Docker image use the latest stable, 1.96).

```sh
cp .env.example .env
# Replace the placeholder with a real secret (at least 32 bytes):
sed -i '' "s/change-me/$(openssl rand -hex 32)/" .env   # macOS; drop the '' on Linux

cargo run
```

### With Docker

```sh
docker build -t crypto-api .
docker run --rm -p 3000:3000 -e HMAC_SECRET="$(openssl rand -hex 32)" crypto-api
```

### Configuration

| Variable      | Default     | Description                                                         |
| ------------- | ----------- | ------------------------------------------------------------------- |
| `HMAC_SECRET` | _required_  | Key used to sign and verify. Must be at least 32 bytes long.        |
| `HOST`        | `127.0.0.1` | Address to bind (`0.0.0.0` in the Docker image).                    |
| `PORT`        | `3000`      | Port to listen on.                                                  |
| `RUST_LOG`    | `info`      | Log filter, e.g. `debug` or `crypto_api=debug,tower_http=info`.     |

## API

All `POST` endpoints expect a `Content-Type: application/json` header.

### `POST /encrypt`

```sh
curl -X POST localhost:3000/encrypt \
  -H 'Content-Type: application/json' \
  -d '{"name":"John Doe","age":30,"contact":{"email":"john@example.com","phone":"123-456-7890"}}'
```

```json
{
  "age": "enc:MzA=",
  "contact": "enc:eyJlbWFpbCI6ImpvaG5AZXhhbXBsZS5jb20iLCJwaG9uZSI6IjEyMy00NTYtNzg5MCJ9",
  "name": "enc:IkpvaG4gRG9lIg=="
}
```

### `POST /decrypt`

Properties that were not encrypted are returned unchanged (`birth_date` below):

```sh
curl -X POST localhost:3000/decrypt \
  -H 'Content-Type: application/json' \
  -d '{"name":"enc:IkpvaG4gRG9lIg==","age":"enc:MzA=","contact":"enc:eyJlbWFpbCI6ImpvaG5AZXhhbXBsZS5jb20iLCJwaG9uZSI6IjEyMy00NTYtNzg5MCJ9","birth_date":"1998-11-19"}'
```

```json
{
  "age": 30,
  "birth_date": "1998-11-19",
  "contact": { "email": "john@example.com", "phone": "123-456-7890" },
  "name": "John Doe"
}
```

### `POST /sign`

```sh
curl -X POST localhost:3000/sign \
  -H 'Content-Type: application/json' \
  -d '{"message":"Hello World","timestamp":1616161616}'
```

```json
{ "signature": "0c093a542fc5ce513c576613447559682ee46c57aaccce2a5c6201e1fe9adfd3" }
```

The signature depends on `HMAC_SECRET`, so yours will differ. Sending the same properties in another order returns the same signature.

### `POST /verify`

```sh
curl -i -X POST localhost:3000/verify \
  -H 'Content-Type: application/json' \
  -d '{"signature":"<signature from /sign>","data":{"timestamp":1616161616,"message":"Hello World"}}'
# HTTP/1.1 204 No Content
```

If `data` or `signature` is tampered with, the response is `400 Bad Request`.

### Status codes

| Status | When                                                                                 |
| ------ | ------------------------------------------------------------------------------------ |
| `200`  | `/encrypt`, `/decrypt`, `/sign` succeeded                                            |
| `204`  | `/verify`: the signature is valid                                                    |
| `400`  | `/verify`: the signature is invalid (wrong value or not hex) · body is not valid JSON |
| `405`  | Wrong HTTP method                                                                    |
| `415`  | Missing `Content-Type: application/json` header                                      |
| `422`  | Valid JSON with the wrong shape: `/encrypt` or `/decrypt` with a non-object body, `/verify` without `signature` (string) or `data` |

`400` versus `422` follows axum's built-in distinction between a body that cannot be parsed and a body that parses but does not match the expected type.

## Design

Both algorithms sit behind traits that only handle bytes: `Encryptor` and `Signer`. JSON handling stays outside the implementations, so replacing Base64 with AES-GCM, or HMAC with Ed25519, means writing one new `impl` and changing one line in `main.rs`.

Key decisions:

- **Encrypted values are prefixed with `enc:`.** Many plain strings are valid Base64 (`"test"`, `"abcd"`), so the prefix is what makes detection reliable. A prefixed value that cannot be decrypted is returned unchanged.
- **Values are serialized to JSON before being encoded**, so `/decrypt` restores their type: `30` stays a number, `contact` stays an object.
- **Payloads are canonicalized explicitly before signing**: object keys sorted recursively, arrays kept in order. This does not rely on `serde_json`'s default key order, which any dependency enabling its `preserve_order` feature would silently change.
- **Signatures are HMAC-SHA256 in hex, compared in constant time** (`Mac::verify_slice`) to prevent timing attacks.

## Tests

```sh
cargo test
```

Unit tests live next to the code in `src/` (including the RFC 4231 test vectors for HMAC-SHA256). Integration tests in `tests/` replay every example from the specification against the real router.

## Limitations and next steps

- **AES-256-GCM `Encryptor`** selected through configuration, to back the abstraction with a real cipher.
- **Uniform JSON error bodies** (`{"error": "..."}`) instead of axum's plain-text messages.
