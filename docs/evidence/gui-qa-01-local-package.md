# GUI QA package and local runtime evidence

**Task:** Build and run a reviewable GUI QA package for Arch Notebook, exercise local Qwen/cloud support, and check whether a network-capable local sandbox can launch.

**Owner:** Engineering orchestrator
**Baseline:** `b50269b99b61346cd5f154f421b9a11ca7529e53`
**Platform:** Linux x86_64, 2026-11-26

## Candidate artifact

- Package: `/tmp/arch-notebook-gui-qa-final5/arch-notebook-gui-qa-b50269b99b61.tar.gz`
- SHA-256: `3d719c3cda098c3bf184fdabf84d6420d3709f766c9a529d5f30276408164a57`
- Sidecar verification: `sha256sum -c ...tar.gz.sha256` — **PASS**
- Included application: `arch-notebook 0.1.0`
- Fixture: synthetic only, 8 entities and 8 relationships
- Binary SHA-256: `1ef36e782956eaf1196a06d074ad8edf02591b45095d9acb805879b000cfe941`
- Fixture SHA-256: `3170844fdc9db636dcb8dc9505075e486ec060790692bf6dc76cc9b32afe7f5f`

## Gates

| Gate | Result | Evidence |
|---|---|---|
| Formatting and shell syntax | PASS | `cargo fmt --all -- --check`; `bash -n` on package, launch, and cloud helpers |
| QA helper regressions | PASS | `python3 -B tools/gui-qa/test_gui_qa.py`; 12 passed, 0 failed |
| Qwen privacy self-test | PASS | Offline self-test; no network request |
| Rust test suite | PASS | `cargo test --locked --offline`; 34 passed, 0 failed |
| Release build and package integrity | PASS | Locked offline release build; binary, fixture, version, and QA-tool digests verified |
| Bubblewrap network namespace | BLOCKED | `bwrap: loopback: Failed to create NETLINK_ROUTE socket: Operation not permitted` |
| Isolated network sandbox | BLOCKED | Bubblewrap cannot configure its network namespace (`NETLINK_ROUTE` permission denied). |
| GUI launch with host network, Wayland | BLOCKED | `./run.sh --allow-app-network --startup-seconds 2`; Wayland socket reported `NoCompositor`; report: `/tmp/arch-notebook-gui-qa-run-20261126-041136-2/run-report.json` |
| GUI launch with host network, X11 | BLOCKED | `./run.sh --allow-app-network --display-backend x11 --startup-seconds 2`; XOpenDisplay failed; report: `/tmp/arch-notebook-gui-qa-run-20261126-041246-2/run-report.json` |
| Local Qwen request | BLOCKED | `qwen_review.py` returned 3 with `URLError` from `127.0.0.1`; report: `/tmp/arch-notebook-gui-qa-run-20261126-040811-2/qwen-review.json` |
| Ollama cloud activation | BLOCKED | `ollama-cloud-mode.sh enable --yes` could not reach the user systemd bus. Helper removed its opt-in drop-in; rollback/restart status could not be verified. No sign-in or cloud inference occurred. |
| Native GUI review | BLOCKED | No application window launched; manual checklist remains pending. |

## Network and cloud boundaries

The normal package launch requires a network namespace and fails closed if the host cannot create one. `--allow-app-network` retains bubblewrap filesystem/process restrictions but uses host networking; it is not a network-isolated sandbox. The current environment has no Docker/Podman executable, no `/dev/kvm`, and denies systemd user-bus and loopback access to this process.

Qwen review defaults to the local `qwen3.5:9b` model. Cloud models require an explicit `--allow-cloud`; remote endpoints also require `--allow-remote` and HTTPS. The local cloud helper can only change the Ollama service when run from a session with user-systemd access. Cloud inference sends the sanitized QA report off this machine and may consume account credits.

## Next actions

1. From the user's normal interactive session, run `tools/gui-qa/ollama-cloud-mode.sh status`; enable cloud only if the user's Ollama account is signed in and the service is reachable.
2. Run the package on a host that supports bubblewrap network namespaces, then complete the manual GUI checklist. If network access is required, explicitly use `--allow-app-network` and acknowledge host networking.
3. Run the packaged UI from a regular desktop session where a compositor and display socket are accessible, complete the manual checklist, and record the result.
