# Code Review Guidelines

## Always check

- New or modified logic has accompanying tests
- Changes are minimal and isolated from upstream code to keep future rebases clean
- Do not modify upstream files unless strictly necessary; prefer adding new files or extending via hooks/traits
- PR must not patch upstream dependencies unless there is a strong justification in the PR description
- Any change to a public API or interface must have a clear reason documented in an inline comment or the PR description
- Do not modify upstream test fixtures unless they conflict with our changes
- Do not modify upstream RPC endpoints or test infrastructure without comparing against upstream first
- Changes to `revm`, `op-revm`, `alloy-evm`, `alloy`, or `[patch.crates-io]` must explain the fork-specific need, impact, and test coverage

## Fork-specific

- This is a fork of [foundry-rs/foundry](https://github.com/foundry-rs/foundry) pinned at v1.3.0
- When reviewing, distinguish between upstream code and fork additions
  Unmodified upstream code can usually be skipped
  Modified hunks in upstream files still need normal review
- If a PR touches upstream files, the reviewer must verify the change is necessary and cannot be achieved by other means
- Prefer fork-side extension points over broad edits to upstream files

## Foundry conventions

- Tests that use forking must contain "fork" in their name
- Bug fixes and new features must include tests
- Prefer incremental improvement over perfection; follow-up PRs can iterate
- Keep commits logically grouped; squash checkpoint commits that don't represent a single logical change
- Verify these pass before approving:
  ```sh
  make pr
  ```
  Keep the `make test` target in sync with `.github/workflows/build-and-test.yml`.

## Previous comments

- Before writing new comments, check all previous review threads on this PR
- If tooling permissions allow it and a previous comment has been addressed by the latest changes, resolve that thread using:
  `gh api graphql -f query='mutation { resolveReviewThread(input:{threadId:"THREAD_ID"}) { thread { id } } }'`
- To find thread IDs, query:
  `gh api graphql -f query='{ repository(owner:"OWNER", name:"REPO") { pullRequest(number:NUMBER) { reviewThreads(first:50) { nodes { id isResolved comments(first:1) { nodes { body path } } } } } } }'`
- Do not repeat feedback that has already been addressed

## Skip

- Formatting-only changes already enforced by `cargo fmt`, unless they create broad churn in upstream files
- Lint issues already caught by `cargo clippy`
- Upstream code that has not been modified by the fork
