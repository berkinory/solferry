# Security

Please report vulnerabilities privately through GitHub security advisories, not in public issues.

## What beam protects

- File contents and names are end-to-end encrypted and authenticated with TLS 1.3.
- Devices are identified by public keys, so names alone are never trusted.
- Peer-supplied paths are validated before anything touches the disk.
- Every delivered file is verified against the sender's BLAKE3 hash.

## What it does not protect

- Relays and discovery services can see IP addresses, device ids, timing and traffic volume.
- `device.key` is a plain file protected by file permissions. OS keychain storage is planned.
- beam has not had an independent security audit yet.
