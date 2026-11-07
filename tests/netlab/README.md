# netlab

Runs real `beam` binaries inside Linux network namespaces. Two homes sit behind separate NAT routers joined by a fake internet with two TLS relays. Routers drop unsolicited inbound traffic, as home routers do.

| Scenario | Checks |
|---|---|
| `nat-cone` | Hole punching gives a direct connection |
| `nat-symmetric-own-relay` | Hole punching fails, data uses your own relay |
| `nat-no-relay` | Two NATs with no relay: pairing fails with an error that says what to do |
| `relay-from-pairing` | Only one side set up a relay, the other gets it and its token from the pairing code |
| `relay-wrong-token` | A relay refuses devices without its token |
| `direct-throughput` | Speed on a clean direct path |
| `offline-lan` | No internet and no servers: discovery by name and transfer |
| `wifi-link`, `lossy-link`, `jitter-link` | Throughput under loss, delay and jitter |
| `outage-10s` | Transfer survives a short network outage |
| `outage-45s` | Longer than the idle timeout: reconnects and resumes |
| `receiver-restart` | Receiver is killed mid-transfer, then resumes |
| `address-change` | Sender changes IP address mid-transfer |

Every scenario that delivers a file compares hashes end to end.

```sh
cargo build --release -p beam-cli
python3 tests/netlab/scenarios.py /path/to/iroh-relay
python3 tests/netlab/scenarios.py /path/to/iroh-relay nat-cone offline-lan
```

Needs Linux, passwordless sudo, `iproute2`, `iptables`, `tc`, `openssl` and `iroh-relay` built with the `server` feature.
