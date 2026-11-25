#!/usr/bin/env python3
"""Check internal links in the mdBook sources.

Broken internal links (a relative link to a file that does not exist, or a
SUMMARY.md entry pointing at a missing chapter) are errors and make the script
exit with status 1. Problems that do not break the published book on their own
(missing heading anchors, links that leave `book/src` and therefore are not
published) are reported as warnings.

External links (http, https, mailto, ...) are not fetched here; CI checks them
separately with lychee as a non-blocking job so an outage elsewhere cannot fail
the build.

Usage:
    python3 scripts/check_book_links.py [--root book/src] [--strict-anchors]
        [--baseline book/linkcheck-baseline.txt] [--update-baseline]

A baseline file lists known broken links (`<file>|<target>` per line) that
are tracked for cleanup. They are reported as warnings so CI can fail on *new*
breakage today while the backlog is worked down. Entries that no longer match
anything are reported so the baseline only ever shrinks.

When run inside GitHub Actions the findings are also emitted as workflow
annotations so they show up inline on the pull request.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import unicodedata
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote

INLINE_LINK = re.compile(r"!?\[(?:[^\[\]]|\[[^\]]*\])*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REFERENCE_DEF = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*<?(\S+?)>?(?:\s+.*)?$")
HTML_LINK = re.compile(r"<(?:a|img)\s[^>]*?(?:href|src)=\"([^\"]+)\"", re.IGNORECASE)
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
EXPLICIT_ID = re.compile(r"\{#([\w-]+)\}\s*$")
HTML_ID = re.compile(r"\bid=\"([^\"]+)\"")
FENCE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE = re.compile(r"`+[^`]*`+")
SCHEME = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")


@dataclass
class Finding:
    level: str  # "error" or "warning"
    path: Path
    line: int
    message: str
    target: str = ""


def slugify(text: str) -> str:
    """Approximate mdBook's heading id generation."""
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = text.replace("`", "")
    out = []
    for ch in text.strip().lower():
        if ch.isalnum() or ch in "-_":
            out.append(ch)
        elif ch.isspace():
            out.append("-")
        elif unicodedata.category(ch).startswith("L"):
            out.append(ch)
    return "".join(out)


def iter_lines(text: str):
    """Yield (line_no, line) for lines outside fenced code blocks, with
    inline code removed."""
    in_fence = False
    fence_marker = ""
    for no, line in enumerate(text.splitlines(), start=1):
        m = FENCE.match(line)
        if m:
            if not in_fence:
                in_fence, fence_marker = True, m.group(1)
            elif m.group(1) == fence_marker:
                in_fence = False
            continue
        if not in_fence:
            yield no, INLINE_CODE.sub("", line)


def collect_anchors(path: Path, cache: dict[Path, set[str]]) -> set[str]:
    if path in cache:
        return cache[path]
    anchors: set[str] = set()
    seen: dict[str, int] = {}
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        cache[path] = anchors
        return anchors
    for _, line in iter_lines(text):
        for html_id in HTML_ID.findall(line):
            anchors.add(html_id)
        m = HEADING.match(line)
        if not m:
            continue
        title = m.group(2)
        explicit = EXPLICIT_ID.search(title)
        if explicit:
            anchors.add(explicit.group(1))
            continue
        slug = slugify(title)
        count = seen.get(slug, 0)
        anchors.add(slug if count == 0 else f"{slug}-{count}")
        seen[slug] = count + 1
    cache[path] = anchors
    return anchors


def extract_links(text: str):
    for no, line in iter_lines(text):
        for regex in (INLINE_LINK, HTML_LINK):
            for target in regex.findall(line):
                yield no, target
        m = REFERENCE_DEF.match(line)
        if m:
            yield no, m.group(1)


def resolve(source: Path, target: str, root: Path) -> Path:
    if target.startswith("/"):
        return root / target.lstrip("/")
    return (source.parent / target).resolve()


def check_file(
    source: Path, root: Path, anchors_cache: dict[Path, set[str]], strict_anchors: bool
) -> list[Finding]:
    findings: list[Finding] = []
    text = source.read_text(encoding="utf-8")
    anchor_level = "error" if strict_anchors else "warning"

    for line_no, raw in extract_links(text):
        target = unquote(raw.strip())
        if not target or SCHEME.match(target) or target.startswith("//"):
            continue  # external link or protocol-relative URL
        if "{{" in target:
            continue  # mdBook template / include syntax

        path_part, _, fragment = target.partition("#")
        path_part = path_part.split("?", 1)[0]

        if not path_part:
            dest = source
        else:
            dest = resolve(source, path_part, root)
            if dest.suffix == ".html":
                md = dest.with_suffix(".md")
                if md.exists():
                    dest = md
                elif dest.name == "index.html" and (dest.parent / "README.md").exists():
                    dest = dest.parent / "README.md"
            if dest.is_dir():
                for index in ("README.md", "index.md"):
                    if (dest / index).exists():
                        dest = dest / index
                        break
            if not dest.exists():
                findings.append(
                    Finding(
                        "error",
                        source,
                        line_no,
                        f"broken link: '{raw}' (no such file)",
                        target=raw,
                    )
                )
                continue
            try:
                dest.relative_to(root)
            except ValueError:
                findings.append(
                    Finding(
                        "warning",
                        source,
                        line_no,
                        f"link '{raw}' points outside {root.name}/ and is not published with the book",
                    )
                )
                continue

        if fragment and dest.suffix == ".md":
            if fragment not in collect_anchors(dest, anchors_cache):
                findings.append(
                    Finding(anchor_level, source, line_no, f"missing anchor '#{fragment}' in '{raw}'")
                )
    return findings


def check_summary(root: Path) -> list[Finding]:
    """mdBook silently creates empty files for missing SUMMARY entries, so a
    typo there ships a blank page instead of failing. Treat it as an error."""
    summary = root / "SUMMARY.md"
    findings: list[Finding] = []
    if not summary.exists():
        return [Finding("error", summary, 0, "SUMMARY.md not found")]
    for line_no, target in extract_links(summary.read_text(encoding="utf-8")):
        if SCHEME.match(target) or not target.strip():
            continue
        path_part = unquote(target.split("#", 1)[0])
        if path_part and not (root / path_part).exists():
            findings.append(
                Finding("error", summary, line_no, f"SUMMARY entry '{target}' points at a missing chapter")
            )
    return findings


def baseline_key(f: Finding, repo_root: Path) -> str:
    try:
        shown = f.path.relative_to(repo_root)
    except ValueError:
        shown = f.path
    return f"{shown.as_posix()}|{f.target}"


def load_baseline(path: Path) -> set[str]:
    if not path.exists():
        return set()
    entries = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            entries.add(line)
    return entries


def write_baseline(path: Path, keys: list[str]) -> None:
    header = (
        "# Known broken internal links in book/src, tracked for cleanup.\n"
        "# Format: <file>|<link target>. Generated by\n"
        "#   python3 scripts/check_book_links.py --update-baseline\n"
        "# Remove entries as links are fixed. Never add new ones by hand:\n"
        "# fix the link instead.\n"
    )
    path.write_text(header + "".join(f"{k}\n" for k in sorted(set(keys))), encoding="utf-8")


def apply_baseline(findings: list[Finding], baseline: set[str], repo_root: Path) -> list[str]:
    """Downgrade baselined errors to warnings. Returns stale baseline keys."""
    used = set()
    for f in findings:
        if f.level != "error" or not f.target:
            continue
        key = baseline_key(f, repo_root)
        if key in baseline:
            used.add(key)
            f.level = "warning"
            f.message += " [known, listed in baseline]"
    return sorted(baseline - used)


def report(findings: list[Finding], repo_root: Path) -> None:
    in_actions = os.environ.get("GITHUB_ACTIONS") == "true"
    for f in findings:
        try:
            shown = f.path.relative_to(repo_root)
        except ValueError:
            shown = f.path
        print(f"{f.level.upper()}: {shown}:{f.line}: {f.message}")
        if in_actions:
            print(f"::{f.level} file={shown},line={max(f.line, 1)}::{f.message}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", default="book/src", help="mdBook source directory")
    parser.add_argument(
        "--strict-anchors",
        action="store_true",
        help="treat missing heading anchors as errors instead of warnings",
    )
    parser.add_argument("--baseline", help="file listing known broken links to tolerate")
    parser.add_argument(
        "--update-baseline",
        action="store_true",
        help="rewrite --baseline with the broken links found now and exit 0",
    )
    args = parser.parse_args()
    if args.update_baseline and not args.baseline:
        parser.error("--update-baseline requires --baseline")

    root = Path(args.root).resolve()
    if not root.is_dir():
        print(f"error: {root} is not a directory", file=sys.stderr)
        return 2
    repo_root = Path.cwd().resolve()

    anchors_cache: dict[Path, set[str]] = {}
    findings = check_summary(root)
    files = sorted(root.rglob("*.md"))
    for md in files:
        findings.extend(check_file(md.resolve(), root, anchors_cache, args.strict_anchors))

    if args.baseline:
        baseline_path = Path(args.baseline)
        if args.update_baseline:
            keys = [baseline_key(f, repo_root) for f in findings if f.level == "error" and f.target]
            write_baseline(baseline_path, keys)
            print(f"Wrote {len(set(keys))} entries to {baseline_path}")
            return 0
        stale = apply_baseline(findings, load_baseline(baseline_path), repo_root)
        for key in stale:
            print(f"NOTE: baseline entry no longer broken, please remove it: {key}")

    findings.sort(key=lambda f: (f.level != "error", str(f.path), f.line))
    report(findings, repo_root)

    errors = sum(1 for f in findings if f.level == "error")
    warnings = len(findings) - errors
    print(f"\nChecked {len(files)} files: {errors} error(s), {warnings} warning(s).")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
