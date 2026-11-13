# Releasing

1. Rename `## Unreleased` in `CHANGELOG.md` to `## X.Y.Z - YYYY-MM-DD`. That section becomes the release notes, and the workflow refuses a tag without one.
2. Set `version` in the root `Cargo.toml`, and `MARKETING_VERSION` in the iOS project, then merge to `main`. Android and the packages read the version from `Cargo.toml`.
3. Tag that commit and push the tag: `git tag v0.2.0 && git push origin v0.2.0`.

The `release` workflow builds every platform, signs and notarizes the macOS app, publishes the GitHub release and the relay image, and updates the Homebrew tap. Running the workflow by hand does the same builds and publishes nothing.

| Name | Kind | What it is |
| --- | --- | --- |
| `SOLFERRY_UPDATE_KEY` | variable | Hex Ed25519 public key built into the apps |
| `SOLFERRY_UPDATE_SIGNING_KEY` | secret | PEM private key that signs `latest.json` |
| `MACOS_CERTIFICATE` | secret | Base64 of the Developer ID Application `.p12` of team Z66C58Z3RC, whose ID prefixes the app group the app and its share extension share |
| `MACOS_CERTIFICATE_PASSWORD` | secret | Password of that `.p12` |
| `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD` | secrets | Notarization login, with an app-specific password |
| `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD` | secrets | Android signing key |
| `HOMEBREW_TAP_TOKEN` | secret | Token allowed to push to `berkinory/homebrew-brew` |

Losing the update signing key means shipped apps can no longer update themselves, so keep a copy offline. The first release publishes the relay image as private: make the `solferry-relay` package public once in its GitHub settings.
