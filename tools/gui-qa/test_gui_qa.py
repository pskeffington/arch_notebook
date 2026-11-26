#!/usr/bin/env python3
"""Offline safety and packaging regressions for the GUI QA helpers."""

from __future__ import annotations

import importlib.util
import json
import os
import pathlib
import stat
import subprocess
import tempfile
from types import SimpleNamespace
import urllib.request
import unittest
from unittest.mock import patch


TOOLS = pathlib.Path(__file__).resolve().parent


def load_module(name: str, path: pathlib.Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load test module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


runner = load_module("gui_qa_runner", TOOLS / "run_gui_qa.py")
qwen = load_module("gui_qa_qwen", TOOLS / "qwen_review.py")


class GuiQaSafetyTests(unittest.TestCase):
    def test_fixture_has_expected_synthetic_chart_counts(self):
        fixture = runner.valid_fixture(TOOLS.parents[1] / "fixtures/chart/correctness.case.json")
        self.assertEqual(len(fixture["entities"]), 8)
        self.assertEqual(len(fixture["links"]), 8)

    def test_report_sanitizer_omits_paths_and_unbounded_text(self):
        sanitized = qwen.sanitize_report(
            {
                "schema": "arch-notebook-gui-qa-run/v1",
                "status": "PASS",
                "source_commit": "a" * 40,
                "checks": {
                    "fixture_structure": {"status": "PASS", "entity_count": 8},
                    "private": {"status": "FAIL", "log_tail": "/home/private/secret"},
                },
                "private_path": "/home/private/case.json",
            }
        )
        encoded = json.dumps(sanitized)
        self.assertNotIn("/home", encoded)
        self.assertNotIn("log_tail", encoded)
        self.assertIn("entity_count", encoded)

    def test_qwen_policy_requires_explicit_remote_and_cloud_opt_in(self):
        with self.assertRaises(ValueError):
            qwen.validate_policy("https://example.invalid/api/chat", "qwen3.5:9b", False, False)
        with self.assertRaises(ValueError):
            qwen.validate_policy("http://127.0.0.1:11434/api/chat", "qwen3-coder:480b-cloud", False, False)
        qwen.validate_policy("https://example.invalid/api/chat", "qwen3-coder:480b-cloud", True, True)

    def test_qwen_redirects_are_not_followed(self):
        handler = qwen.NoRedirectHandler()
        self.assertIsNone(handler.redirect_request(None, None, 302, "found", {}, "https://example.invalid/"))

    def test_wayland_runner_mounts_only_the_selected_display_socket(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            socket_path = root / "wayland-test"
            socket_path.write_text("socket path fixture")
            run_dir = root / "run"
            run_dir.mkdir()
            (run_dir / "runtime").mkdir()
            command = ["bwrap"]
            with (
                patch.dict(os.environ, {"WAYLAND_DISPLAY": str(socket_path)}, clear=True),
                patch.object(
                    pathlib.Path,
                    "stat",
                    return_value=SimpleNamespace(st_mode=stat.S_IFSOCK, st_uid=os.getuid()),
                ),
            ):
                environment = runner.add_display_mounts(command, run_dir)
            self.assertEqual(environment["WAYLAND_DISPLAY"], "wayland.sock")
            self.assertEqual(environment["XDG_RUNTIME_DIR"], "/tmp/qa-run/runtime")
            self.assertIn(str(socket_path), command)
            self.assertIn("/tmp/qa-run/runtime/wayland.sock", command)

    def test_explicit_x11_selection_binds_only_the_selected_socket(self):
        socket_path = pathlib.Path("/tmp/.X11-unix/X0")
        if not socket_path.exists() or not stat.S_ISSOCK(socket_path.stat().st_mode):
            self.skipTest("no local X11 display socket is available")
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = pathlib.Path(temporary) / "run"
            run_dir.mkdir()
            (run_dir / "runtime").mkdir()
            command = ["bwrap"]
            with patch.dict(os.environ, {"WAYLAND_DISPLAY": "wayland-1", "DISPLAY": ":0", "XAUTHORITY": ""}, clear=True):
                environment = runner.add_display_mounts(command, run_dir, "x11")
            self.assertEqual(environment["DISPLAY"], ":0")
            self.assertNotIn("WAYLAND_DISPLAY", environment)
            self.assertIn(str(socket_path), command)
            self.assertNotIn("/tmp/qa-run/runtime/wayland.sock", command)

    def test_bwrap_probe_records_network_namespace_unavailability(self):
        with patch.object(runner.subprocess, "run") as run:
            run.return_value = SimpleNamespace(returncode=0, stderr="")
            self.assertEqual(runner.self_test_bwrap("bwrap", True), ("PASS", "bubblewrap probes passed"))
            network_call = run.call_args_list[1].args[0]
            self.assertLess(network_call.index("--unshare-net"), network_call.index("--"))
            self.assertEqual(runner.self_test_bwrap("bwrap", False), ("PASS", "bubblewrap probes passed"))
        with patch.object(runner.subprocess, "run") as run:
            run.side_effect = [
                SimpleNamespace(returncode=0, stderr=""),
                SimpleNamespace(returncode=1, stderr="network namespace unavailable"),
            ]
            status, message = runner.self_test_bwrap("bwrap", True)
            self.assertEqual(status, "BLOCKED")
            self.assertIn("network namespace unavailable", message)

    def test_report_creation_is_private_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = pathlib.Path(temporary) / "report.json"
            report = {"schema": "arch-notebook-gui-qa-run/v1", "status": "PASS"}
            runner.write_report(output, report)
            self.assertEqual(output.stat().st_mode & 0o777, 0o600)
            with self.assertRaises(FileExistsError):
                runner.write_report(output, {"status": "FAIL"})
            self.assertEqual(json.loads(output.read_text()), report)

    def test_owned_report_can_be_atomically_updated_but_external_files_cannot(self):
        with tempfile.TemporaryDirectory() as temporary:
            run_dir = pathlib.Path(temporary) / "run"
            run_dir.mkdir(mode=0o700)
            output = run_dir / "run-report.json"
            runner.write_report(output, {"status": "PENDING"})
            runner.update_owned_report(output, {"status": "CLOSED"}, run_dir)
            self.assertEqual(json.loads(output.read_text()), {"status": "CLOSED"})
            with self.assertRaises(ValueError):
                runner.update_owned_report(pathlib.Path(temporary) / "other.json", {}, run_dir)

    def test_loopback_opener_ignores_proxy_environment(self):
        with patch.dict(os.environ, {"HTTP_PROXY": "http://127.0.0.1:7777", "http_proxy": "http://127.0.0.1:7777"}):
            opener = qwen.opener_for_endpoint("http://127.0.0.1:11434/api/chat")
        self.assertFalse(any(isinstance(handler, urllib.request.ProxyHandler) for handler in opener.handlers))

    def test_sanitizer_ignores_untrusted_non_string_status(self):
        sanitized = qwen.sanitize_report(
            {
                "schema": "arch-notebook-gui-qa-run/v1",
                "status": ["PASS"],
                "checks": {"fixture_structure": {"status": {"PASS": True}}},
                "gui": {"status": []},
            }
        )
        self.assertNotIn("status", sanitized)
        self.assertEqual(sanitized["checks"]["fixture_structure"], {})
        self.assertEqual(sanitized["gui"], {})

    def test_cloud_helper_rejects_symlinked_config_ancestor(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            real_config = root / "real-config"
            real_config.mkdir()
            link = root / "config-link"
            link.symlink_to(real_config, target_is_directory=True)
            environment = dict(os.environ, XDG_CONFIG_HOME=str(link))
            result = subprocess.run(
                ["bash", str(TOOLS / "ollama-cloud-mode.sh"), "status"],
                env=environment,
                capture_output=True,
                text=True,
                timeout=5,
            )
        self.assertEqual(result.returncode, 1)
        self.assertIn("symlinked Ollama drop-in path", result.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
