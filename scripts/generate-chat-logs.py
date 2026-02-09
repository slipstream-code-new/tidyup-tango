#!/usr/bin/env python3
"""Generate human-readable markdown chat logs from Claude Code JSONL session transcripts.

Pure stdlib Python 3 — no dependencies. Incremental: only regenerates when the JSONL
file's mtime is newer than the corresponding chat-log file.

Output directory: .claude-sessions/chat-logs/
Filename pattern: {session_id}@{user}@{host}.md
"""

import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def git_user_email() -> str:
    try:
        result = subprocess.run(
            ["git", "config", "user.email"],
            capture_output=True, text=True, check=True,
        )
        return result.stdout.strip()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return "unknown"


def git_repo_root() -> Path:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, check=True,
        )
        return Path(result.stdout.strip())
    except (subprocess.CalledProcessError, FileNotFoundError):
        return Path.cwd()


def format_timestamp(ts_str: str) -> str:
    """Parse ISO 8601 timestamp and format as 'YYYY-MM-DD HH:MM UTC'."""
    try:
        dt = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        return dt.strftime("%Y-%m-%d %H:%M UTC")
    except (ValueError, AttributeError):
        return ts_str


def tool_summary(content_block: dict) -> str:
    """Return a one-line summary for a tool_use content block."""
    name = content_block.get("name", "Unknown")
    inp = content_block.get("input", {})

    if name == "Bash":
        desc = inp.get("description", "")
        if desc:
            return f"Bash: {desc}"
        cmd = inp.get("command", "")
        if cmd:
            short = cmd.split("\n")[0][:80]
            return f"Bash: `{short}`"
        return "Bash"
    elif name == "Read":
        fp = inp.get("file_path", "")
        if fp:
            return f"Read: {fp}"
        return "Read"
    elif name == "Write":
        fp = inp.get("file_path", "")
        if fp:
            return f"Write: {fp}"
        return "Write"
    elif name == "Edit":
        fp = inp.get("file_path", "")
        if fp:
            return f"Edit: {fp}"
        return "Edit"
    elif name == "Grep":
        pat = inp.get("pattern", "")
        if pat:
            return f"Grep: `{pat}`"
        return "Grep"
    elif name == "Glob":
        pat = inp.get("pattern", "")
        if pat:
            return f"Glob: `{pat}`"
        return "Glob"
    elif name == "Task":
        desc = inp.get("description", "")
        if desc:
            return f"Task: {desc}"
        return "Task"
    elif name == "WebSearch":
        q = inp.get("query", "")
        if q:
            return f"WebSearch: `{q}`"
        return "WebSearch"
    elif name == "WebFetch":
        url = inp.get("url", "")
        if url:
            return f"WebFetch: {url}"
        return "WebFetch"
    else:
        return name


def process_session(jsonl_path: Path) -> list[dict]:
    """Parse a JSONL session file and return a list of chat entries.

    Each entry is a dict with keys: timestamp, role, text, tools.
    """
    entries = []

    with open(jsonl_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except json.JSONDecodeError:
                continue

            # Skip non-message types
            msg_type = msg.get("type")
            if msg_type not in ("user", "assistant"):
                continue

            # Skip sidechain messages
            if msg.get("isSidechain", False):
                continue

            # Skip tool results (user messages that are responses to tool calls)
            if msg.get("sourceToolAssistantUUID"):
                continue

            timestamp = msg.get("timestamp", "")
            inner = msg.get("message", {})
            role = inner.get("role", msg_type)
            content = inner.get("content", "")

            text_parts = []
            tool_uses = []

            if isinstance(content, str):
                # Skip command/system messages and internal tags
                if any(content.startswith(tag) for tag in (
                    "<local-command",
                    "<command-name>",
                    "<command-message>",
                    "<local-command-stdout>",
                    "<system-reminder>",
                )):
                    continue
                # Strip inline XML tags that may wrap content
                cleaned = re.sub(r"</?(?:command-name|command-message|command-args|local-command-caveat|system-reminder)[^>]*>", "", content).strip()
                if not cleaned:
                    continue
                text_parts.append(cleaned)
            elif isinstance(content, list):
                has_tool_result = False
                for block in content:
                    btype = block.get("type", "")
                    if btype == "text":
                        text = block.get("text", "")
                        # Strip system-reminder and similar tags from text blocks
                        text = re.sub(r"<system-reminder>[\s\S]*?</system-reminder>", "", text)
                        text = re.sub(r"</?(?:command-name|command-message|command-args|local-command-caveat)[^>]*>", "", text)
                        text = text.strip()
                        if text:
                            text_parts.append(text)
                    elif btype == "tool_use":
                        tool_uses.append(tool_summary(block))
                    elif btype == "tool_result":
                        has_tool_result = True

                # If this is a user message that only contains tool results, skip it
                if has_tool_result and not text_parts:
                    continue

            # Skip empty messages
            if not text_parts and not tool_uses:
                continue

            entries.append({
                "timestamp": timestamp,
                "role": "User" if role == "user" else "Assistant",
                "text": "\n\n".join(text_parts),
                "tools": tool_uses,
                "git_branch": msg.get("gitBranch", ""),
            })

    return entries


def generate_markdown(session_id: str, entries: list[dict]) -> str:
    """Generate a markdown chat log from parsed entries."""
    if not entries:
        return ""

    # Determine date range and git branch
    timestamps = [e["timestamp"] for e in entries if e["timestamp"]]
    first_ts = format_timestamp(timestamps[0]) if timestamps else "Unknown"
    last_ts = format_timestamp(timestamps[-1]) if timestamps else "Unknown"

    # Get git branch from the first entry that has one
    git_branch = ""
    for e in entries:
        if e.get("git_branch"):
            git_branch = e["git_branch"]
            break

    lines = []
    lines.append(f"# Chat Log: {session_id}")
    lines.append("")
    lines.append("| Field | Value |")
    lines.append("|-------|-------|")
    lines.append(f"| **Session ID** | `{session_id}` |")
    lines.append(f"| **Date Range** | {first_ts} -- {last_ts} |")
    if git_branch:
        lines.append(f"| **Git Branch** | `{git_branch}` |")
    lines.append("")
    lines.append("---")
    lines.append("")

    for entry in entries:
        ts = format_timestamp(entry["timestamp"]) if entry["timestamp"] else "Unknown"
        lines.append(f"## {ts} -- {entry['role']}")
        lines.append("")

        if entry["text"]:
            lines.append(entry["text"])
            lines.append("")

        if entry["tools"]:
            n = len(entry["tools"])
            lines.append("<details>")
            lines.append(f"<summary>Tool usage ({n} call{'s' if n != 1 else ''})</summary>")
            lines.append("")
            for t in entry["tools"]:
                lines.append(f"- {t}")
            lines.append("")
            lines.append("</details>")
            lines.append("")

        lines.append("---")
        lines.append("")

    return "\n".join(lines)


def main():
    repo_root = git_repo_root()
    sessions_dir = repo_root / ".claude-sessions"

    if not sessions_dir.is_dir():
        print("No .claude-sessions/ directory found.", file=sys.stderr)
        return

    output_dir = sessions_dir / "chat-logs"
    output_dir.mkdir(parents=True, exist_ok=True)

    user_email = git_user_email()
    hostname = os.uname().nodename

    # Auto-stage session transcripts
    subprocess.run(["git", "add", str(sessions_dir)], capture_output=True)

    generated = 0
    skipped = 0

    for jsonl_file in sorted(sessions_dir.glob("*.jsonl")):
        session_id = jsonl_file.stem
        output_file = output_dir / f"{session_id}@{user_email}@{hostname}.md"

        # Incremental: skip if chat log is newer than source
        if output_file.exists():
            src_mtime = jsonl_file.stat().st_mtime
            dst_mtime = output_file.stat().st_mtime
            if dst_mtime >= src_mtime:
                skipped += 1
                continue

        entries = process_session(jsonl_file)
        if not entries:
            skipped += 1
            continue

        md = generate_markdown(session_id, entries)
        if not md:
            skipped += 1
            continue

        output_file.write_text(md, encoding="utf-8")
        generated += 1

    # Auto-stage generated chat logs
    subprocess.run(["git", "add", str(output_dir)], capture_output=True)

    print(f"Chat logs: {generated} generated, {skipped} skipped (up-to-date or empty)")


if __name__ == "__main__":
    main()
