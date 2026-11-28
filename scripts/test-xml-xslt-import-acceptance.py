#!/usr/bin/env python3
"""Exercise trusted XSLT-to-ANX CLI ingestion and failure atomicity."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def run(binary: Path, *args: str, success: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run([str(binary), *args], text=True, capture_output=True, check=False)
    if (result.returncode == 0) != success:
        raise RuntimeError(f"unexpected exit {result.returncode}: {result.stderr or result.stdout}")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    fixture = args.fixture.resolve()
    work = args.work_dir.resolve()
    if not binary.is_file() or not fixture.is_file():
        parser.error("binary and fixture must exist")
    work.mkdir(parents=True, exist_ok=False)

    source_case = work / "source.case.json"
    source_case.write_bytes(fixture.read_bytes())
    anx = work / "source.anx"
    run(binary, "--export-anx", str(source_case), str(anx), "--json")

    input_xml = work / "source.xml"
    input_xml.write_bytes(anx.read_bytes())
    stylesheet = work / "identity.xsl"
    stylesheet.write_text(
        """<?xml version="1.0"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="xml" encoding="UTF-16"/>
  <xsl:template match="@*|node()"><xsl:copy><xsl:apply-templates select="@*|node()"/></xsl:copy></xsl:template>
</xsl:stylesheet>
""",
        encoding="utf-8",
    )
    destination = work / "destination.case.json"
    destination.write_text(
        json.dumps({"format_version": 19, "title": "XSLT acceptance", "entities": [], "links": []}),
        encoding="utf-8",
    )
    imported = run(
        binary,
        "--import-xml-xslt",
        str(destination),
        str(input_xml),
        str(stylesheet),
        "--json",
    )
    report = json.loads(imported.stdout)
    case = json.loads(destination.read_text(encoding="utf-8"))
    if not report.get("ok") or not report.get("saved") or report.get("input") != str(input_xml):
        raise RuntimeError(f"successful import report is unexpected: {report}")
    if len(case["entities"]) != 8 or len(case["links"]) != 8:
        raise RuntimeError("XSLT-transformed ANX did not preserve the fixture chart")

    before = destination.read_bytes()
    for encoding in ("utf-8", "utf-16"):
        doctype_input = work / f"doctype-{encoding}.xml"
        doctype_input.write_bytes(
            '<!DOCTYPE Chart [<!ENTITY x "unsafe">]><Chart>&x;</Chart>'.encode(encoding)
        )
        rejected_input = run(
            binary,
            "--import-xml-xslt",
            str(destination),
            str(doctype_input),
            str(stylesheet),
            "--json",
            success=False,
        )
        if "document type declaration" not in json.loads(rejected_input.stdout).get("error", ""):
            raise RuntimeError(f"DOCTYPE {encoding} input was rejected for an unexpected reason")
        if destination.read_bytes() != before:
            raise RuntimeError(f"DOCTYPE {encoding} input rejection modified the case")

    doctype_stylesheet = work / "doctype.xsl"
    doctype_stylesheet.write_bytes(
        b'<!DOCTYPE xsl:stylesheet [<!ENTITY x "unsafe">]>' + stylesheet.read_bytes()
    )
    rejected_stylesheet = run(
        binary,
        "--import-xml-xslt",
        str(destination),
        str(input_xml),
        str(doctype_stylesheet),
        "--json",
        success=False,
    )
    if "document type declaration" not in json.loads(rejected_stylesheet.stdout).get("error", ""):
        raise RuntimeError("DOCTYPE stylesheet was rejected for an unexpected reason")
    if destination.read_bytes() != before:
        raise RuntimeError("DOCTYPE stylesheet rejection modified the case")

    print(json.dumps({"result": "PASS", "entities": 8, "relationships": 8,
                      "source_doctype_rejected": True, "stylesheet_doctype_rejected": True,
                      "failed_import_preserved_case": True}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
