#!/usr/bin/env python3
"""Verify and launch a fictional Arch Notebook GUI QA package."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def valid_fixture(path: pathlib.Path) -> dict:
    fixture = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(fixture.get("entities"), list) or not isinstance(fixture.get("links"), list):
        raise ValueError("fixture must contain entities and links arrays")
    if len(fixture["entities"]) != 8 or len(fixture["links"]) != 8:
        raise ValueError("fixture does not match the expected 8-entity/8-link synthetic chart")
    return fixture


def write_report(path: pathlib.Path, report: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if os.path.lexists(path):
        raise FileExistsError(f"refusing to overwrite report: {path}")
    temporary = _write_private_temp(path.parent, report)
    try:
        os.link(temporary, path, follow_symlinks=False)
    finally:
        temporary.unlink(missing_ok=True)


def update_owned_report(path: pathlib.Path, report: dict, run_dir: pathlib.Path) -> None:
    """Atomically update only the report owned by a private QA run directory."""
    directory_info = run_dir.lstat()
    if not stat.S_ISDIR(directory_info.st_mode) or directory_info.st_uid != os.getuid() or directory_info.st_mode & 0o077:
        raise PermissionError("QA run directory is not an owned private directory")
    if path.parent != run_dir or path.name != "run-report.json":
        raise ValueError("refusing to update a report outside the owned run directory")
    current = path.lstat()
    if not stat.S_ISREG(current.st_mode) or current.st_uid != os.getuid() or current.st_mode & 0o077 or current.st_nlink != 1:
        raise PermissionError("refusing to replace a report that is not an owned private regular file")
    temporary = _write_private_temp(run_dir, report)
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _write_private_temp(directory: pathlib.Path, report: dict) -> pathlib.Path:
    descriptor, temp_name = tempfile.mkstemp(prefix=".gui-qa-report-", suffix=".tmp", dir=directory)
    temp_path = pathlib.Path(temp_name)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(json.dumps(report, indent=2, sort_keys=True) + "\n")
    except Exception:
        temp_path.unlink(missing_ok=True)
        raise
    return temp_path


def self_test_bwrap(bwrap: str, isolate_network: bool) -> tuple[str, str]:
    base = [bwrap, "--die-with-parent", "--new-session", "--unshare-pid"]
    mounts = [
        "--ro-bind", "/", "/", "--dev", "/dev", "--tmpfs", "/tmp",
        "--dir", "/tmp/qa-run", "--dir", "/tmp/qa-package",
        "--tmpfs", "/run", "--proc", "/proc",
    ]
    for protected_path in ("/home", "/root", "/mnt", "/media", "/opt", "/srv"):
        if pathlib.Path(protected_path).is_dir():
            mounts.extend(["--tmpfs", protected_path])
    probe = "/usr/bin/sh"
    checks = "test -d /tmp/qa-run && test -d /tmp/qa-package && test ! -e /run/user"
    common = [*base, *mounts, "--clearenv", "--setenv", "PATH", "/usr/bin:/bin", "--", probe, "-c", checks]
    basic = subprocess.run(
        common,
        capture_output=True,
        text=True,
        timeout=10,
    )
    if basic.returncode != 0:
        return "FAIL", (basic.stderr.strip() or "bubblewrap filesystem sandbox probe failed")[:500]
    if isolate_network:
        network = subprocess.run(
            [*base, "--unshare-net", *mounts, "--clearenv", "--setenv", "PATH", "/usr/bin:/bin", "--", probe, "-c", checks],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if network.returncode != 0:
            return "BLOCKED", (network.stderr.strip() or "network namespace isolation is unavailable")[:500]
    return "PASS", "bubblewrap probes passed"


def add_display_mounts(command: list[str], run_dir: pathlib.Path, backend: str = "auto") -> dict[str, str]:
    """Expose only the local display socket needed by the review window."""
    environment: dict[str, str] = {}
    runtime_dir = run_dir / "runtime"
    runtime_dir.mkdir(mode=0o700, exist_ok=True)

    wayland_display = os.environ.get("WAYLAND_DISPLAY", "")
    host_runtime = os.environ.get("XDG_RUNTIME_DIR", "")
    wayland_socket = pathlib.Path(wayland_display)
    if wayland_display and not wayland_socket.is_absolute() and host_runtime:
        wayland_socket = pathlib.Path(host_runtime) / wayland_display
    if backend in {"auto", "wayland"} and wayland_display and not wayland_socket.is_symlink() and wayland_socket.exists():
        socket_info = wayland_socket.stat()
    else:
        socket_info = None
    if socket_info and stat.S_ISSOCK(socket_info.st_mode) and socket_info.st_uid == os.getuid():
        target = runtime_dir / "wayland.sock"
        target.touch(mode=0o600, exist_ok=True)
        command.extend(["--ro-bind", str(wayland_socket), "/tmp/qa-run/runtime/wayland.sock"])
        environment["XDG_RUNTIME_DIR"] = "/tmp/qa-run/runtime"
        environment["WAYLAND_DISPLAY"] = "wayland.sock"
        return environment

    display = os.environ.get("DISPLAY", "") if backend in {"auto", "x11"} else ""
    if display:
        host, _, display_number = display.rpartition(":")
        host = host.removesuffix("/")
        number = display_number.split(".", 1)[0]
        if host in {"", "unix", "localhost"} and number.isdigit():
            socket_dir = pathlib.Path("/tmp/.X11-unix")
            socket_path = socket_dir / f"X{number}"
            if socket_dir.is_dir() and not socket_dir.is_symlink() and socket_path.exists() and not socket_path.is_symlink() and stat.S_ISSOCK(socket_path.stat().st_mode):
                x11_target_dir = runtime_dir / "X11"
                x11_target_dir.mkdir(mode=0o700, exist_ok=True)
                x11_target = x11_target_dir / f"X{number}"
                x11_target.touch(mode=0o600, exist_ok=True)
                command.extend(["--ro-bind", str(socket_path), f"/tmp/qa-run/runtime/X11/X{number}"])
                environment["DISPLAY"] = f":{number}"
                xauthority = os.environ.get("XAUTHORITY")
                if not xauthority:
                    default_authority = pathlib.Path.home() / ".Xauthority"
                    xauthority = str(default_authority) if default_authority.is_file() else ""
                if xauthority:
                    authority_path = pathlib.Path(xauthority)
                    if authority_path.is_symlink() or not authority_path.is_file():
                        raise ValueError("Xauthority must be a regular non-symlink file")
                    descriptor = os.open(authority_path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
                    authority_info = os.fstat(descriptor)
                    if not stat.S_ISREG(authority_info.st_mode) or authority_info.st_uid != os.getuid() or authority_info.st_size > 1024 * 1024:
                        os.close(descriptor)
                        raise ValueError("Xauthority must be owned by the current user and no larger than 1 MiB")
                    target = runtime_dir / "Xauthority"
                    output_descriptor = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600)
                    with os.fdopen(descriptor, "rb") as source, os.fdopen(output_descriptor, "wb") as destination:
                        shutil.copyfileobj(source, destination)
                    command.extend(["--ro-bind", str(target), "/tmp/qa-run/runtime/Xauthority"])
                    environment["XAUTHORITY"] = "/tmp/qa-run/runtime/Xauthority"
                environment["XDG_RUNTIME_DIR"] = "/tmp/qa-run/runtime"

    return environment


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bundle", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2])
    parser.add_argument("--preflight", action="store_true", help="verify the package without launching the GUI")
    parser.add_argument("--launch", action="store_true", help="launch the fixture UI inside bubblewrap")
    parser.add_argument("--allow-app-network", action="store_true", help="allow host networking when isolated networking is unavailable")
    parser.add_argument("--display-backend", choices=("auto", "wayland", "x11"), default="auto", help="select a local display socket when both display protocols are present")
    parser.add_argument("--startup-seconds", type=int, default=5)
    parser.add_argument("--output", type=pathlib.Path, help="new report path for preflight, or new run directory for launch")
    args = parser.parse_args()
    if args.preflight == args.launch:
        parser.error("choose exactly one of --preflight or --launch")
    if not 1 <= args.startup_seconds <= 30:
        parser.error("--startup-seconds must be between 1 and 30")

    bundle = args.bundle.resolve()
    manifest_path = bundle / "manifest.json"
    binary = bundle / "bin" / "arch-notebook"
    fixture_path = bundle / "fixtures" / "correctness.case.json"
    checks: dict[str, dict] = {}
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema") != "arch-notebook-gui-qa-package/v1":
        raise SystemExit("unsupported QA package manifest schema")

    checks["binary_digest"] = {
        "status": "PASS" if sha256(binary) == manifest["binary"]["sha256"] else "FAIL",
        "sha256": sha256(binary),
    }
    checks["fixture_digest"] = {
        "status": "PASS" if sha256(fixture_path) == manifest["fixture"]["sha256"] else "FAIL",
        "sha256": sha256(fixture_path),
    }
    fixture = valid_fixture(fixture_path)
    checks["fixture_structure"] = {
        "status": "PASS",
        "entity_count": len(fixture["entities"]),
        "relationship_count": len(fixture["links"]),
    }
    tool_manifest = manifest.get("qa_tools_sha256")
    tools_valid = isinstance(tool_manifest, dict) and 1 <= len(tool_manifest) <= 32
    tools_checked = 0
    if tools_valid:
        for relative, expected_digest in tool_manifest.items():
            relative_path = pathlib.PurePosixPath(relative) if isinstance(relative, str) else pathlib.PurePosixPath("..")
            if (
                relative_path.is_absolute()
                or ".." in relative_path.parts
                or "\\" in str(relative)
                or not isinstance(expected_digest, str)
                or len(expected_digest) != 64
            ):
                tools_valid = False
                break
            tool_path = bundle.joinpath(*relative_path.parts)
            try:
                resolved_tool = tool_path.resolve(strict=True)
            except OSError:
                tools_valid = False
                break
            if tool_path.is_symlink() or not resolved_tool.is_relative_to(bundle) or not resolved_tool.is_file():
                tools_valid = False
                break
            tools_checked += 1
            if sha256(resolved_tool) != expected_digest:
                tools_valid = False
                break
    checks["qa_tool_integrity"] = {
        "status": "PASS" if tools_valid else "FAIL",
        "files_checked": tools_checked,
    }
    source_files = manifest.get("source_files", {})
    app_source_clean = isinstance(source_files, dict) and source_files.get("app_or_dependency_worktree_dirty") is False
    checks["candidate_source"] = {"status": "PASS" if app_source_clean else "BLOCKED", "source_inputs_dirty": not app_source_clean}
    version_result = subprocess.run([str(binary), "--version", "--json"], capture_output=True, text=True, timeout=10)
    version_data = json.loads(version_result.stdout) if version_result.returncode == 0 else {}
    checks["application_version"] = {
        "status": "PASS" if version_data == manifest["application"] else "FAIL",
        "version": version_data,
    }
    bwrap = shutil.which("bwrap")
    if bwrap:
        sandbox_status, sandbox_message = self_test_bwrap(bwrap, not args.allow_app_network)
        checks["sandbox"] = {
            "status": sandbox_status,
            "network_isolated": not args.allow_app_network,
            "message": sandbox_message,
        }
    else:
        checks["sandbox"] = {"status": "BLOCKED", "message": "bubblewrap is not installed"}

    report = {
        "schema": "arch-notebook-gui-qa-run/v1",
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "source_commit": manifest.get("source_commit"),
        "application": version_data,
        "package_sha256": None,
        "checks": checks,
        "gui": {"status": "NOT_RUN", "native_review": "PENDING"},
        "qwen_review": {"status": "NOT_RUN"},
    }
    if any(item["status"] == "FAIL" for item in checks.values()):
        report["status"] = "FAIL"
    elif any(item["status"] == "BLOCKED" for item in checks.values()):
        report["status"] = "BLOCKED"
    else:
        report["status"] = "PASS"

    if args.preflight:
        output = args.output or pathlib.Path(tempfile_dir()) / f"arch-notebook-gui-qa-preflight-{time.strftime('%Y%m%d-%H%M%S')}-{os.getpid()}.json"
        if os.path.lexists(output):
            raise SystemExit(f"refusing to overwrite report: {output}")
        write_report(output, report)
        print(json.dumps({"status": report["status"], "report": str(output), "checks": checks}, indent=2))
        return 1 if report["status"] == "FAIL" else 0

    if report["status"] == "FAIL":
        raise SystemExit("package integrity check failed; refusing to launch")

    run_dir = args.output or pathlib.Path(tempfile_dir()) / f"arch-notebook-gui-qa-run-{time.strftime('%Y%m%d-%H%M%S')}-{os.getpid()}"
    if os.path.lexists(run_dir):
        raise SystemExit(f"refusing to reuse existing run directory: {run_dir}")
    run_dir.mkdir(mode=0o700, parents=True)
    for name in ("home", "tmp", "xdg-data", "xdg-config", "xdg-cache"):
        (run_dir / name).mkdir(mode=0o700)
    run_fixture = run_dir / "correctness.case.json"
    shutil.copyfile(fixture_path, run_fixture)
    run_fixture.chmod(0o600)
    template = bundle / "manual-review.template.json"
    if template.is_file():
        shutil.copyfile(template, run_dir / "manual-review.json")
        (run_dir / "manual-review.json").chmod(0o600)
    report_path = run_dir / "run-report.json"
    if not bwrap:
        report["gui"] = {"status": "BLOCKED", "reason": "bubblewrap is required to launch the package"}
        report["status"] = "BLOCKED"
        write_report(report_path, report)
        print(json.dumps({"status": "BLOCKED", "report": str(report_path)}, indent=2))
        return 3

    sandbox_status, sandbox_message = self_test_bwrap(bwrap, not args.allow_app_network)
    if sandbox_status != "PASS":
        report["gui"] = {"status": "BLOCKED", "reason": sandbox_message}
        report["status"] = "BLOCKED"
        write_report(report_path, report)
        print(json.dumps({"status": "BLOCKED", "report": str(report_path), "reason": sandbox_message}, indent=2))
        return 3

    command = [
        bwrap,
        "--die-with-parent",
        "--new-session",
        "--unshare-pid",
        "--ro-bind", "/", "/",
        "--dev", "/dev",
        "--proc", "/proc",
        "--tmpfs", "/tmp",
        "--dir", "/tmp/qa-run",
        "--bind", str(run_dir), "/tmp/qa-run",
        "--dir", "/tmp/qa-package",
        "--ro-bind", str(bundle), "/tmp/qa-package",
    ]
    display_environment = add_display_mounts(command, run_dir, args.display_backend)
    if not display_environment:
        report["gui"] = {"status": "BLOCKED", "reason": "no bindable local Wayland or X11 display socket is available"}
        report["status"] = "BLOCKED"
        write_report(report_path, report)
        print(json.dumps({"status": "BLOCKED", "report": str(report_path), "reason": report["gui"]["reason"]}, indent=2))
        return 3
    if "DISPLAY" in display_environment:
        command.extend(["--symlink", "/tmp/qa-run/runtime/X11", "/tmp/.X11-unix"])
    command.extend([
        "--tmpfs", "/run",
        "--chdir", "/tmp/qa-run",
        "--clearenv",
        "--setenv", "HOME", "/tmp/qa-run/home",
        "--setenv", "TMPDIR", "/tmp/qa-run/tmp",
        "--setenv", "XDG_DATA_HOME", "/tmp/qa-run/xdg-data",
        "--setenv", "XDG_CONFIG_HOME", "/tmp/qa-run/xdg-config",
        "--setenv", "XDG_CACHE_HOME", "/tmp/qa-run/xdg-cache",
        "--setenv", "PATH", "/usr/bin:/bin",
        "--setenv", "LANG", os.environ.get("LANG", "C.UTF-8"),
        "--setenv", "LC_ALL", os.environ.get("LC_ALL", "C.UTF-8"),
    ])
    var_run = pathlib.Path("/var/run")
    if not (var_run.is_symlink() and var_run.resolve() == pathlib.Path("/run")):
        command.extend(["--tmpfs", "/var/run"])
    for protected_path in ("/home", "/root", "/mnt", "/media", "/opt", "/srv"):
        if pathlib.Path(protected_path).is_dir():
            command.extend(["--tmpfs", protected_path])
    for variable, value in display_environment.items():
        command.extend(["--setenv", variable, value])
    if not args.allow_app_network:
        command.append("--unshare-net")
    command.extend(["--", "/tmp/qa-package/bin/arch-notebook", "--open-case", "/tmp/qa-run/correctness.case.json"])

    log_path = run_dir / "app.log"
    process = None
    display_auth = run_dir / "runtime" / "Xauthority"
    try:
        with log_path.open("w", encoding="utf-8") as log:
            process = subprocess.Popen(command, stdout=log, stderr=subprocess.STDOUT, text=True)
            time.sleep(args.startup_seconds)
            if process.poll() is not None:
                log.flush()
                log_text = log_path.read_text(encoding="utf-8", errors="replace")[-1200:]
                report["gui"] = {
                    "status": "BLOCKED" if "display" in log_text.lower() else "FAIL",
                    "reason": "application exited during startup",
                    "exit_code": process.returncode,
                    "log_tail": log_text,
                }
                report["status"] = report["gui"]["status"]
                write_report(report_path, report)
                print(json.dumps({"status": report["status"], "report": str(report_path)}, indent=2))
                return 3 if report["status"] == "BLOCKED" else 1
            report["gui"] = {"status": "STARTED_FOR_MANUAL_REVIEW", "startup_seconds": args.startup_seconds}
            report["status"] = "PENDING_HUMAN_REVIEW"
            write_report(report_path, report)
            print(f"Application started with the fictional fixture. Review the window, then close it: {run_dir}", flush=True)
            exit_code = process.wait()
        report["gui"]["exit_code_after_review"] = exit_code
        report["gui"]["status"] = "CLOSED_AFTER_REVIEW"
        report["status"] = "PENDING_HUMAN_REVIEW"
        update_owned_report(report_path, report, run_dir)
        print(f"Review report: {report_path}")
        return 0
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        if os.path.lexists(display_auth):
            display_auth.unlink()


def tempfile_dir() -> str:
    return os.environ.get("TMPDIR", "/tmp")


if __name__ == "__main__":
    raise SystemExit(main())
