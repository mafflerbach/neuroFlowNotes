#!/usr/bin/env python3
"""
Transcript Summarizer - NeuroFlow Integration

Processes markdown transcript files using a local LLM (LM Studio).
Outputs JSON status messages to stdout for integration with NeuroFlow.

Usage:
  python transcript_summarizer.py --input-dir <path> --output-dir <path>
"""

import argparse
import json
import os
import re
import sys
import time
from pathlib import Path
import requests

# Defaults
LMSTUDIO_BASE_URL = os.getenv("LMSTUDIO_BASE_URL", "http://localhost:1234/v1").rstrip("/")
LMSTUDIO_CHAT_URL = f"{LMSTUDIO_BASE_URL}/chat/completions"
LLM_MODEL = os.getenv("LLM_MODEL", "gpt-oss-20b-uncensored")

# Tuning knobs
SUMMARY_WORDS = (320, 500)
BULLET_COUNT = 12


def emit(msg_type: str, **data):
    """Output JSON status message for NeuroFlow integration."""
    print(json.dumps({"type": msg_type, **data}), flush=True)


def wait_for_stable(path: Path, stable_checks=3, interval=1.0, timeout=300) -> bool:
    """Wait until file size stops changing."""
    last = -1
    steady = 0
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            size = path.stat().st_size
        except FileNotFoundError:
            time.sleep(interval)
            continue
        if size == last and size > 0:
            steady += 1
            if steady >= stable_checks:
                return True
        else:
            steady = 0
            last = size
        time.sleep(interval)
    return False


def list_pending_files(input_dir: Path) -> list[Path]:
    """List all .md files in the input directory."""
    return sorted([p for p in input_dir.glob("*.md") if p.is_file()])


_YT_ID = r"([A-Za-z0-9_-]{11})"


def extract_video_id(text: str) -> str | None:
    """Extract YouTube video ID from text."""
    # YAML field
    m = re.search(r"^\s*video_id\s*:\s*['\"]?" + _YT_ID + r"['\"]?\s*$", text, re.M)
    if m:
        return m.group(1)

    # URL fields
    for field in ("url", "source_url", "original_url"):
        m = re.search(rf"^\s*{field}\s*:\s*['\"]?(.+?)['\"]?\s*$", text, re.M)
        if m:
            vid = _extract_from_url(m.group(1))
            if vid:
                return vid

    # Any YouTube URL
    urls = re.findall(r"(https?://[^\s)>\]]+)", text)
    for u in urls:
        vid = _extract_from_url(u)
        if vid:
            return vid

    return None


def _extract_from_url(u: str) -> str | None:
    m = re.search(r"[?&]v=" + _YT_ID + r"(\b|&|#|/)", u)
    if m:
        return m.group(1)
    m = re.search(r"youtu\.be/" + _YT_ID + r"(\b|[?&#/])", u)
    if m:
        return m.group(1)
    m = re.search(r"youtube\.com/shorts/" + _YT_ID + r"(\b|[?&#/])", u)
    if m:
        return m.group(1)
    return None


def download_thumbnail(video_id: str, asset_template: str) -> Path | None:
    """Download YouTube thumbnail."""
    url = f"https://img.youtube.com/vi/{video_id}/hqdefault.jpg"
    out_path = Path(os.path.expanduser(asset_template.format(videoId=video_id)))
    out_path.parent.mkdir(parents=True, exist_ok=True)

    if out_path.exists() and out_path.stat().st_size > 0:
        return out_path

    try:
        r = requests.get(url, timeout=30)
        if r.status_code != 200:
            return None
        out_path.write_bytes(r.content)
        return out_path
    except Exception:
        return None


def call_lm_studio_markdown(input_text: str, endpoint: str, model: str) -> str:
    """Send note to LM Studio for summarization."""
    min_w, max_w = SUMMARY_WORDS

    system_prompt = (
        "You are generating an Obsidian-compatible Markdown note.\n"
        "OUTPUT REQUIREMENTS:\n"
        "1) Output a COMPLETE Markdown document that STARTS with YAML frontmatter delimited by '---'.\n"
        "   There should no whitespaces after the '---' \n"
        "2) If the INPUT already has YAML frontmatter, COPY all existing fields and values unless updated below. "
        "   Preserve unknown keys. Keep YAML lists as lists (not comma strings).\n"
        "3) FRONTMATTER UPDATES (MANDATORY):\n"
        "   - Overwrite 'status' with 'summarized'.\n"
        "   - Ensure 'tags' is a YAML list; remove 'unprocessed' and 'ready-for-processing' if present; add 'summarized' if missing.\n"
        "   - Generate 6–12 domain-relevant, lowercase, hyphenated content tags (no '#', no spaces, no punctuation) "
        "     based on the content. EXCLUDE generic media tags like 'youtube', 'video', 'transcript' from 'content_tags'.\n"
        "   - Append those content-derived tags to 'tags' (dedupe). Also write them to a separate 'content_tags' YAML list.\n"
        "   - If absent, add 'processed_date' as an ISO 8601 timestamp.\n"
        "   - Preserve fields like 'title', 'url', 'video_id', 'channel', 'duration' exactly if they exist.\n"
        "4) BODY SECTIONS (after frontmatter):\n"
        "   - '# {title}' (reuse existing title; if missing, create a concise one)\n"
        f"   - '## Summary' (~{min_w}–{max_w} words, plain text)\n"
        f"   - '## Key Points' (exactly {BULLET_COUNT} concise bullets)\n"
        "   - '## Source' listing url/channel/duration if present\n"
        "   - Do NOT include the raw transcript text.\n"
        "5) STYLE:\n"
        "   - This is a Markdown file for **Obsidian**.\n"
        "   - Use lowercase, hyphenated tags; YAML lists; no trailing spaces.\n"
        "   - NO code fences, NO backticks, NO extra commentary, NO reasoning—only the final note."
    )

    user_prompt = (
        "INPUT NOTE (may include YAML frontmatter and transcript/body):\n"
        "-----BEGIN NOTE-----\n"
        f"{input_text}\n"
        "-----END NOTE-----\n\n"
        "Now produce the final Obsidian Markdown per the OUTPUT REQUIREMENTS above.\n"
        "Do not add any other sections."
    )

    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ],
        "temperature": 0.3,
        "max_tokens": 3000,
    }

    resp = requests.post(endpoint, json=payload, timeout=1000)
    if resp.status_code >= 400:
        raise RuntimeError(f"LM Studio error {resp.status_code}: {resp.text}")

    data = resp.json()
    msg = data.get("choices", [{}])[0].get("message", {})
    out = (msg.get("content") or msg.get("reasoning") or "").strip()

    if not out:
        raise RuntimeError(f"Empty assistant message. Raw: {resp.text[:800]}")

    # Strip accidental code fences
    if out.startswith("```"):
        out = re.sub(r"^```(?:markdown|md)?\s*|\s*```$", "", out, flags=re.S).strip()

    return out + ("\n" if not out.endswith("\n") else "")


def process_file(
    src_path: Path,
    out_dir: Path,
    endpoint: str,
    model: str,
    asset_template: str | None = None
) -> dict:
    """Process a single transcript file."""
    emit("progress", file=str(src_path), stage="reading")

    if not wait_for_stable(src_path, stable_checks=3, interval=1.0, timeout=60):
        raise RuntimeError("File did not become stable in time.")

    text = src_path.read_text(encoding="utf-8")
    video_id = extract_video_id(text)

    emit("progress", file=str(src_path), stage="summarizing")
    summary_md = call_lm_studio_markdown(text, endpoint, model)

    # Try to extract video_id from LLM output if not found earlier
    if not video_id:
        video_id = extract_video_id(summary_md)

    out_dir.mkdir(parents=True, exist_ok=True)
    out_name = src_path.name
    out_path = out_dir / out_name

    # Write atomically
    tmp = out_path.with_suffix(out_path.suffix + ".tmp")
    tmp.write_text(summary_md, encoding="utf-8")
    tmp.replace(out_path)

    # Download thumbnail if possible
    thumb_path = None
    if asset_template and video_id:
        try:
            thumb_path = download_thumbnail(video_id, asset_template)
        except Exception:
            pass

    # Remove source after successful write
    src_path.unlink(missing_ok=True)

    return {
        "source": str(src_path),
        "output": str(out_path),
        "video_id": video_id,
        "thumbnail": str(thumb_path) if thumb_path else None
    }


def main():
    ap = argparse.ArgumentParser(
        description="Transcript Summarizer - NeuroFlow Integration"
    )
    ap.add_argument(
        "--input-dir",
        required=True,
        help="Directory containing transcript .md files"
    )
    ap.add_argument(
        "--output-dir",
        required=True,
        help="Directory to write summarized notes"
    )
    ap.add_argument(
        "--asset-template",
        default=None,
        help="Path template for thumbnails with {videoId} placeholder"
    )
    ap.add_argument(
        "--endpoint",
        default=LMSTUDIO_CHAT_URL,
        help=f"LM Studio chat endpoint (default: {LMSTUDIO_CHAT_URL})"
    )
    ap.add_argument(
        "--model",
        default=LLM_MODEL,
        help=f"Model name (default: {LLM_MODEL})"
    )
    args = ap.parse_args()

    in_dir = Path(os.path.expanduser(args.input_dir))
    out_dir = Path(os.path.expanduser(args.output_dir))

    if not in_dir.exists():
        emit("error", error=f"Input directory does not exist: {in_dir}")
        sys.exit(1)

    in_dir.mkdir(parents=True, exist_ok=True)
    out_dir.mkdir(parents=True, exist_ok=True)

    pending = list_pending_files(in_dir)
    emit("status", message=f"Found {len(pending)} pending transcripts", input_dir=str(in_dir))

    if not pending:
        emit("complete", processed=0, failed=0, results=[])
        return

    results = []
    failed = 0

    for i, src_path in enumerate(pending):
        emit("progress", file=str(src_path), index=i, total=len(pending), stage="starting")
        try:
            result = process_file(
                src_path,
                out_dir,
                args.endpoint,
                args.model,
                args.asset_template
            )
            result["success"] = True
            results.append(result)
            emit("success", **result)
        except Exception as e:
            failed += 1
            result = {
                "source": str(src_path),
                "success": False,
                "error": str(e)
            }
            results.append(result)
            emit("error", file=str(src_path), error=str(e))

    emit("complete", processed=len(pending) - failed, failed=failed, results=results)

    if failed > 0:
        sys.exit(1)


if __name__ == "__main__":
    main()
