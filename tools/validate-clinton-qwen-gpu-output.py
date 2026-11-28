#!/usr/bin/env python3
"""Validate bounded page records and quote traceability for the private GPU demo."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys


def normalize(text: str) -> str:
    return " ".join(text.split())


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--pages-dir", required=True, type=Path)
    parser.add_argument("pages", nargs=10, type=int)
    args = parser.parse_args()

    try:
        payload = json.loads(args.output.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        print(f"invalid JSON output: {error}", file=sys.stderr)
        return 1
    if not isinstance(payload, dict) or set(payload) != {"records", "limitation"}:
        print("output must contain exactly records and limitation", file=sys.stderr)
        return 1
    if not isinstance(payload["limitation"], str) or not payload["limitation"].strip():
        print("limitation must be a nonempty string", file=sys.stderr)
        return 1
    records = payload["records"]
    if not isinstance(records, list) or len(records) != len(args.pages):
        print("records must contain exactly one item per sampled page", file=sys.stderr)
        return 1

    expected_fields = {
        "page", "email_like", "header_fields_present", "body_text_present",
        "quote_check", "qa_note",
    }
    seen: set[int] = set()
    for record in records:
        if not isinstance(record, dict) or set(record) != expected_fields:
            print("record has missing or unexpected fields", file=sys.stderr)
            return 1
        page = record["page"]
        if type(page) is not int or page not in args.pages or page in seen:
            print("record page is missing, unexpected, or duplicated", file=sys.stderr)
            return 1
        seen.add(page)
        if type(record["email_like"]) is not bool or type(record["body_text_present"]) is not bool:
            print(f"page {page}: boolean fields have invalid types", file=sys.stderr)
            return 1
        headers = record["header_fields_present"]
        if not isinstance(headers, list) or any(
            not isinstance(item, str) or item not in {"from", "to", "date", "subject"}
            for item in headers
        ) or len(set(headers)) != len(headers):
            print(f"page {page}: invalid header field list", file=sys.stderr)
            return 1
        note = record["qa_note"]
        if note is not None and (not isinstance(note, str) or len(note) > 240):
            print(f"page {page}: QA note is not bounded text or null", file=sys.stderr)
            return 1
        quote = record["quote_check"]
        if quote is None:
            continue
        if not isinstance(quote, str) or not 3 <= len(quote.split()) <= 8:
            print(f"page {page}: quote must contain 3 to 8 words or be null", file=sys.stderr)
            return 1
        page_path = args.pages_dir / f"page-{page:04d}.txt"
        try:
            source = normalize(page_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError) as error:
            print(f"page {page}: sampled source unavailable: {error}", file=sys.stderr)
            return 1
        if normalize(quote) not in source:
            print(f"page {page}: quote does not trace to its sampled source", file=sys.stderr)
            return 1

    if seen != set(args.pages):
        print("output does not cover the exact sampled page set", file=sys.stderr)
        return 1
    print(f"PASS: {len(seen)} page records; quote strings trace to sampled source where present")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
