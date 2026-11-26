# Arch Notebook GUI QA package

This creates a Linux review bundle containing the release executable, the deterministic fictional chart fixture, an isolated launcher, the manual checklist, and a sanitized Qwen review adapter. It is a QA artifact, not an application release.

## Build the bundle

From the repository root:

```sh
tools/gui-qa/package.sh
```

The script runs formatting and locked offline Rust tests, builds the release binary, verifies its JSON version, validates the fixture, then creates `/tmp/arch-notebook-gui-qa-<source-sha>.tar.gz` and a SHA-256 sidecar. Build artifacts remain in `/tmp/arch-notebook-target`.

Extract and run:

```sh
tar -xzf /tmp/arch-notebook-gui-qa-<source-sha>.tar.gz -C /tmp
cd /tmp/arch-notebook-gui-qa-<source-sha>
./run.sh
```

The runner uses a new mode-0700 run directory under `/tmp`, copies the fixture there, and gives the application separate XDG data/config/cache directories. Bubblewrap is required. The default sandbox denies app network access, masks common user data locations and the user session bus, clears inherited environment variables, and mounts only the selected local X11 or Wayland socket; the rest of the root filesystem is read-only. X11 also requires a read-only Xauthority cookie mount when the host uses one. The app can interact with the selected display, so use this bundle for the repository's own review build, not arbitrary untrusted binaries. If network namespace creation is unavailable, the default runner fails closed. `--allow-app-network` permits host networking while retaining filesystem and process isolation; it does not create an isolated network sandbox. Use it only for this trusted review build on a controlled host. Qwen review runs separately after the app exits.

If both Wayland and X11 variables are present but one compositor is unavailable, choose a backend explicitly with `--display-backend wayland` or `--display-backend x11`; the default is `auto` and prefers Wayland.

Use `python3 tools/gui-qa/run_gui_qa.py --bundle . --preflight` to verify package and sandbox prerequisites without opening a window. `run-report.json` distinguishes checks from manual acceptance. Complete [review-checklist.md](review-checklist.md) and update the copied `manual-review.json` to assess the actual UI.

## Qwen review

Qwen review is an advisory analysis of the sanitized run report. Default model/endpoint are `qwen3.5:9b` and `http://127.0.0.1:11434/api/chat`; no request is sent unless you run the reviewer. It sends only an allowlisted report and status-only manual checklist; it excludes the fixture, app source, logs, local paths, and free-form manual notes. Redirects are rejected. A screenshot is included only with `--include-screenshot` and must be checked for private content first. Remote endpoints require `--allow-remote` and HTTPS. Cloud models require `--allow-cloud`; if hosted inference is configured behind the local Ollama service, the report leaves the machine even though the endpoint is loopback.

```sh
python3 tools/gui-qa/qwen_review.py /tmp/<run-dir>/run-report.json \
  --manual-review /tmp/<run-dir>/manual-review.json
```

For Ollama-hosted cloud inference, cloud requests require the explicit `--allow-cloud` flag. The helper below changes the Ollama user service policy and restarts the service; it does not sign in. Run it from a normal user session where `systemctl --user` can reach the user bus, then sign in separately:

```sh
python3 tools/gui-qa/ollama-cloud-mode.sh enable
ollama signin
python3 tools/gui-qa/qwen_review.py /tmp/<run-dir>/run-report.json \
  --model qwen3-coder:480b-cloud --allow-cloud
```

Ollama cloud models run on hosted compute; the report is sent off this machine and usage may consume account credits. See [Ollama Cloud](https://docs.ollama.com/cloud) and [Ollama privacy policy](https://ollama.com/privacy). `ollama-cloud-mode.sh disable` removes only a regular drop-in whose contents match this helper, then restarts the Ollama user service. The helper refuses symlinked or unrelated drop-ins and never signs in or stores credentials.

## Limits

The current developer sandbox may deny display access, network namespaces, user-systemd control, or loopback sockets. Record those as `BLOCKED`; do not treat package preflight or a Qwen response as native GUI acceptance. No VM/cloud guest is provisioned by this package.
