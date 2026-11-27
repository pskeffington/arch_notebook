#!/usr/bin/env python3
"""Verify the per-user Linux archive install in an isolated temporary home."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
ARCHIVE_NAME = "arch-notebook-linux-x86_64"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path, help="Linux x86_64 Arch Notebook executable")
    parser.add_argument("--report", required=True, type=Path, help="write the JSON acceptance report here")
    args = parser.parse_args()

    binary = args.binary.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise SystemExit(f"binary is not executable: {binary}")

    report: dict[str, object] = {
        "schema_version": 1,
        "suite": "arch_notebook_linux_user_install_acceptance",
        "binary_sha256": sha256(binary),
        "result": "PASS",
        "checks": {},
    }
    checks: dict[str, object] = report["checks"]  # type: ignore[assignment]
    archive_files = {
        "arch-notebook": binary,
        "arch-notebook.desktop": ROOT / "packaging/arch/arch-notebook.desktop",
        "arch-notebook.svg": ROOT / "assets/icons/arch-notebook.svg",
        "arch-notebook.png": ROOT / "assets/icons/arch-notebook.png",
        "arch-notebook-case.xml": ROOT / "packaging/arch/arch-notebook-case.xml",
        "install-linux-automation.sh": ROOT / "scripts/install-linux-automation.sh",
    }

    with tempfile.TemporaryDirectory(prefix="arch-notebook-user-install-") as temporary:
        temp_root = Path(temporary)
        home = temp_root / "home"
        desktop = home / "Desktop"
        config_home = home / ".config"
        data_home = home / ".local/share"
        archive = temp_root / ARCHIVE_NAME
        desktop.mkdir(parents=True)
        config_home.mkdir(parents=True)
        archive.mkdir()
        (config_home / "user-dirs.dirs").write_text(
            'XDG_DESKTOP_DIR="$HOME/Desktop"\n', encoding="utf-8"
        )
        for filename, source in archive_files.items():
            shutil.copy2(source, archive / filename)

        environment = os.environ.copy()
        environment.update(
            {
                "HOME": str(home),
                "XDG_CONFIG_HOME": str(config_home),
                "XDG_DATA_HOME": str(data_home),
            }
        )
        installer = subprocess.run(
            [str(ROOT / "scripts/install-linux-user.sh"), str(archive)],
            cwd=ROOT,
            env=environment,
            check=False,
            capture_output=True,
            text=True,
        )
        if installer.returncode != 0:
            raise SystemExit(
                f"installer exited {installer.returncode}:\n{installer.stdout}{installer.stderr}"
            )

        menu_entry = data_home / "applications/arch-notebook.desktop"
        desktop_shortcut = desktop / "Arch Notebook.desktop"
        installed_binary = home / ".local/bin/arch-notebook"
        installed_svg = data_home / "icons/hicolor/scalable/apps/arch-notebook.svg"
        installed_png = data_home / "icons/hicolor/1024x1024/apps/arch-notebook.png"
        installed_automation = home / ".local/bin/arch-notebook-install-automation"
        installed_mime = data_home / "mime/packages/arch-notebook-case.xml"

        if menu_entry.read_bytes() != desktop_shortcut.read_bytes():
            raise SystemExit("Desktop shortcut differs from the application-menu entry")
        if stat.S_IMODE(desktop_shortcut.stat().st_mode) != 0o755:
            raise SystemExit("Desktop shortcut is not executable")
        for installed in (menu_entry, desktop_shortcut):
            desktop_text = installed.read_text(encoding="utf-8")
            if "Icon=arch-notebook" not in desktop_text:
                raise SystemExit(f"missing synchronized application icon reference: {installed}")
            if f'Exec="{installed_binary}" %f' not in desktop_text:
                raise SystemExit(f"desktop entry does not target the installed executable: {installed}")
        if sha256(installed_binary) != report["binary_sha256"]:
            raise SystemExit("installed executable differs from the release binary")
        if sha256(installed_svg) != sha256(archive / "arch-notebook.svg"):
            raise SystemExit("installed SVG icon differs from the package artwork")
        if sha256(installed_png) != sha256(archive / "arch-notebook.png"):
            raise SystemExit("installed PNG icon differs from the package artwork")
        if sha256(installed_automation) != sha256(archive / "install-linux-automation.sh"):
            raise SystemExit("installed SQL automation helper differs from the package helper")
        if installed_mime.read_bytes() != (archive / "arch-notebook-case.xml").read_bytes():
            raise SystemExit("installed case MIME definition differs from the package file")

        desktop_validation = "NOT AVAILABLE"
        validator = shutil.which("desktop-file-validate")
        if validator:
            validation = subprocess.run(
                [validator, str(menu_entry), str(desktop_shortcut)],
                check=False,
                capture_output=True,
                text=True,
            )
            if validation.returncode != 0:
                raise SystemExit(validation.stderr or validation.stdout)
            desktop_validation = "PASS"

        version = subprocess.run(
            [str(installed_binary), "--version", "--json"],
            check=False,
            capture_output=True,
            text=True,
        )
        if version.returncode != 0:
            raise SystemExit(f"installed --version exited {version.returncode}: {version.stderr}")
        version_json = json.loads(version.stdout)
        checks.update(
            {
                "installer_exit_code": installer.returncode,
                "desktop_menu_and_shortcut_identical": True,
                "desktop_shortcut_mode": oct(stat.S_IMODE(desktop_shortcut.stat().st_mode)),
                "desktop_file_validation": desktop_validation,
                "installed_binary_matches_release": True,
                "svg_icon_matches_package": True,
                "png_icon_matches_package": True,
                "sql_automation_helper_matches_package": True,
                "case_mime_definition_matches_package": True,
                "version_json": version_json,
                "desktop_path": str(desktop_shortcut),
            }
        )

    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"result": report["result"], "report": str(args.report), "checks": checks}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
