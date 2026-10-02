# integrity

how terobit knows a pack hasn't been tampered with, who it came from, and who may play it.

**everything here works offline.** no check needs a network connection. the network is
only used, when it's available anyway, to pick up revocations and new signatures.

## chain of trust

```
root key (public half built into every client and server)
 └─ certificate ─ pack signing key
                   └─ .tbsig ─ manifest.tbman
                                └─ index hash ─ pack index
                                                 └─ chunk hashes ─ file data
```

change any byte anywhere and the link above it stops matching. opening a pack costs one
signature check and one index hash. reading costs one hash per chunk read.

## hashing

blake3, 32 bytes, always in derive-key mode with a context string that says what is being
hashed:

```
hash(context, bytes) = blake3::Hasher::new_derive_key(context).update(bytes).finalize()
```

the context makes hashes of different things impossible to confuse. for example, a chunk
can't be crafted so that its hash also passes as an index hash.

| name      | context string                          | hashes                                |
|-----------|-----------------------------------------|---------------------------------------|
| `CHUNK`   | `terobit 2026-10-01 chunk`              | a chunk's stored bytes                |
| `INDEX`   | `terobit 2026-10-01 pack index`         | a pack index's bytes                  |
| `KEY_ID`  | `terobit 2026-10-01 key id`             | a public key, to make a key id        |
| `CERT`    | `terobit 2026-10-01 certificate`        | what a root key signs                 |
| `PACK`    | `terobit 2026-10-01 pack signature`     | what a pack signing key signs         |
| `LICENSE` | `terobit 2026-10-01 license signature`  | what a license signing key signs      |

context strings never change once released. a new meaning gets a new string.

chunks are hashed **as stored** (after compression), so every chunk is verified before
the zstd decoder sees it, and a malicious pack can never feed crafted input to the
decompressor. to keep hashes stable, the zstd version and level are pinned per manifest
version (see [tbman v1](../tbman/v1.md#compression)).

always verify the exact bytes you're about to use, in memory. never verify a file on disk
and then read it again, because it can change in between.

## keys

two levels of ed25519 key:

| key         | lives                                        | signs                                        | if it leaks |
|-------------|----------------------------------------------|----------------------------------------------|-------------|
| root key    | offline, never on a networked machine. public half built into clients and servers | certificates, the revocation list | client update |
| signing key | platform publish and store servers           | packs or licenses, never both                | revoke it, no client update |

the root key is used rarely: to certify a new signing key, and to sign the revocation list.
rotating signing keys is routine. rotating a root key needs a client update, so clients
ship with more than one root key to rotate between.

### certificates

a signing key is trusted because a root key signed a certificate for it. the certificate
travels inside every `.tbsig` and `.tblic`, so checking it needs nothing but the root keys
already built in.

```rust
struct Certificate {
    /// what this key may sign
    purpose: Purpose,
    /// the signing key's ed25519 public key
    public_key: [u8; 32],
    /// first 8 bytes of hash(KEY_ID, root public key). picks the root key, not a security check
    root_key_id: [u8; 8],
    /// root key's ed25519 signature over hash(CERT, [purpose as u8] ++ public_key)
    root_signature: [u8; 64],
}

enum Purpose {
    Pack = 0,
    License = 1,
}
```

checking a certificate:

1. find the built-in root key matching `root_key_id`. none: reject
2. verify `root_signature` over `hash(CERT, ...)` of the 33 bytes: the purpose as one byte,
   then the 32 public key bytes
3. `purpose` must be what the caller expects. a license key can't sign packs, and a pack
   key can't sign licenses
4. `public_key` must not be in the cached revocation list

### no expiry dates

certificates, signatures and licenses carry no dates. an offline machine's clock can't be
trusted, and someone holding a stolen key could put any date they like on a signature
anyway. revocation is what stops a stolen key.

## revocation

if a signing key leaks:

1. the root key signs a new revocation list containing that key's full public key
2. the platform re-signs every legitimate pack and license that used it, with a new key
3. online clients receive the new list **and** the new `.tbsig` and `.tblic` files for what
   they have installed in the same update, so nothing legitimate stops working

rules:

- the list is fetched only when the client is online anyway (login, store, updates) and
  cached on disk
- a missing or old list **never blocks play**. offline play uses whatever was cached last
- the trade-off: a machine that stays offline keeps trusting a leaked key until it next
  connects. that is inherent to offline play, and an attacker still has to get a forged
  pack onto that specific machine

the revocation list will get its own format doc when it's defined.

## who signs and checks what

| file          | signed with  | checked by                                                  | invalid or missing |
|---------------|--------------|-------------------------------------------------------------|--------------------|
| client pack   | pack key     | the client                                                  | refuse to run      |
| server pack   | pack key     | whoever hosts it: official, developer, community, or a player (p2p, offline) | refuse to run |
| license       | license key  | the client, offline (see [tblic](../tblic/REF.md))          | refuse to run a `Licensed` realm |
| dev directory | nothing      | nothing                                                     | only loads in `dev` feature builds with `--dev`, or in the editor later |

the platform validates every upload (file types, asset limits, paths) before signing it. a
signature means the platform accepted the pack.

community and p2p hosts control their own machines, so they can modify their server or skip
its checks. signing protects hosts from tampered packs. it can't protect players from a
tampered host, so a community server is only as trustworthy as whoever runs it.
