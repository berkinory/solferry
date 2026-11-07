# Security

Please report vulnerabilities privately through GitHub security advisories, not in public issues.

## What solferry protects

- File contents and names are end-to-end encrypted and authenticated with TLS 1.3.
- Devices are identified by public keys, so names alone are never trusted.
- Peer-supplied paths are validated before anything touches the disk.
- Every delivered file is verified against the sender's BLAKE3 hash.

## What it does not protect

- solferry contacts no server by default. A relay you or a paired device run can see IP addresses, device ids, timing and traffic volume, never file contents.
- A pairing code includes the inviter's relay and its access token, so joining connects you to it. Treat codes as secrets and only use codes from people you trust.
- `device.key` is a plain file protected by file permissions. OS keychain storage is planned.
- solferry has not had an independent security audit yet.
- Received files carry the system's download marker on macOS and Windows, so they are checked before first run.
- Partial files belong to the device that started them, so another device can't continue or poison them.
