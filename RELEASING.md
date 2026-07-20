# Releasing

Releases are published from GitHub Actions. The workflow publishes the crate
first and creates the corresponding `vX.Y.Z` tag and GitHub release only after
crates.io serves the uploaded archive.

## One-time setup for 0.1.0

Trusted Publishing cannot publish a crate name that does not exist yet, so the
first release requires a scoped crates.io token.

1. In the GitHub repository settings, create an environment named `crates-io`.
   Add a required reviewer if repository policy requires release approval.
2. On crates.io, create an API token with permission to publish a new crate.
3. Add that token to the `crates-io` GitHub environment as the secret
   `CARGO_REGISTRY_TOKEN`.
4. Merge the release-preparation PR into `main`.
5. Open **Actions → Publish to crates.io → Run workflow**, select `main`, enter
   `0.1.0`, and run the workflow.

The workflow checks that the input matches `Cargo.toml`, runs all release
checks, publishes the crate, verifies that the uploaded archive records the
current Git commit, and then creates `v0.1.0` and the GitHub release.

## Switch to Trusted Publishing

After 0.1.0 appears on crates.io:

1. Open the crate's **Settings → Trusted Publishing** page on crates.io.
2. Add a GitHub Actions publisher with these values:
   - repository owner: `milyin`
   - repository: `project-root-patch`
   - workflow: `publish.yml`
   - environment: `crates-io`
3. In the GitHub `crates-io` environment, add the variable
   `CRATES_IO_TRUSTED_PUBLISHING` with the value `true`.
4. Delete the `CARGO_REGISTRY_TOKEN` environment secret.
5. Optionally enable Trusted-Publishing-Only mode in the crate settings after
   one OIDC-authenticated release succeeds.

The workflow then exchanges GitHub's OIDC identity for a short-lived crates.io
token. No long-lived publishing credential is stored in GitHub.

## Publish a later version

1. In a PR, update the version in `Cargo.toml`, update `CHANGELOG.md`, and run
   `cargo check` so `Cargo.lock` records the new package version.
2. Merge the PR into `main` after CI succeeds.
3. Run **Publish to crates.io** from `main` with the exact manifest version.

Do not create or push the version tag manually. If publication succeeds but a
later workflow step fails, rerun the workflow with the same version. It verifies
the published archive's Git commit before completing the missing tag or release.
