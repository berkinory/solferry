# iroh-mdns-address-lookup 0.6.1, patched

Copied from crates.io (MIT OR Apache-2.0) with one change in `src/lib.rs`, marked "Solferry patch".

swarm-discovery sends mDNS only on the interface the default route uses unless it is given interfaces. A phone sharing its hotspot routes multicast to its cellular link, so devices on the hotspot never hear it. The patch hands swarm-discovery every private or link-local IPv4 address of the endpoint, and updates that list whenever the endpoint's addresses change.

Remove this folder and the `[patch.crates-io]` entry in the workspace `Cargo.toml` once upstream does the same.
