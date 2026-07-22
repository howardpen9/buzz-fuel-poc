#!/usr/bin/env python3
"""Export Buzz channel events for evidence bundles.

Preserves the complete signed Nostr event when available (including ``sig``).
Never reads or writes private keys.

Usage:
  # From a buzz CLI messages get JSON file:
  python scripts/export_buzz_evidence.py --from-messages-json path.json \\
      --relay ws://localhost:3000 --channel-id <uuid> --channel-name "…" \\
      --out evidence/<ts>/buzz-events.json

  # Optional: try HTTP event fetch (relay-dependent; records reason on failure)
  python scripts/export_buzz_evidence.py ... --try-fetch-signed

Signature policy:
  - If input includes a non-empty ``sig`` (or full event object), keep it.
  - Else if ``--try-fetch-signed`` succeeds, use the fetched signed event.
  - Else write ``sig_status`` explaining why ``sig`` is absent — never silent null.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


# Keys that must never appear in exported evidence.
_FORBIDDEN_KEY_NAMES = {
    "private_key",
    "privkey",
    "secret",
    "nsec",
    "bot_private_key",
    "owner_private_key",
    "BUZZ_BOT_PRIVATE_KEY",
    "BUZZ_PRIVATE_KEY",
    "BUZZ_OWNER_PRIVATE_KEY",
}


def _assert_no_secrets(obj: Any, path: str = "$") -> None:
    if isinstance(obj, dict):
        for k, v in obj.items():
            if k in _FORBIDDEN_KEY_NAMES or any(
                s in k.lower() for s in ("private_key", "privkey", "nsec")
            ):
                raise SystemExit(f"refusing to export secret field {path}.{k}")
            _assert_no_secrets(v, f"{path}.{k}")
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            _assert_no_secrets(v, f"{path}[{i}]")


def _normalize_event(raw: dict) -> dict:
    """Normalize a message/event dict into evidence shape."""
    # Full signed event may be nested under "event"
    if isinstance(raw.get("event"), dict):
        raw = {**raw, **raw["event"]}

    eid = raw.get("id") or raw.get("event_id")
    out: dict[str, Any] = {
        "id": eid,
        "kind": raw.get("kind"),
        "pubkey": raw.get("pubkey"),
        "created_at": raw.get("created_at"),
        "content": raw.get("content"),
        "tags": raw.get("tags") or [],
    }

    sig = raw.get("sig") or raw.get("signature")
    if isinstance(sig, str) and sig.strip():
        out["sig"] = sig.strip()
        out["sig_status"] = "present"
        # Preserve full signed event envelope for verifier tools when fields complete
        if all(out.get(k) is not None for k in ("id", "kind", "pubkey", "created_at", "content")):
            out["signed_event"] = {
                "id": out["id"],
                "pubkey": out["pubkey"],
                "created_at": out["created_at"],
                "kind": out["kind"],
                "tags": out["tags"],
                "content": out["content"],
                "sig": out["sig"],
            }
    else:
        # Explicit absence — never leave a bare null without explanation
        out["sig_status"] = (
            raw.get("sig_status")
            or "unavailable: source message list did not include Nostr sig "
            "(buzz messages get REST projection is content-oriented; "
            "re-export with --try-fetch-signed against a relay that serves full events, "
            "or capture the signed EVENT frame at publish time)"
        )
        # Do not emit sig:null

    return out


def _try_fetch_signed(relay_http: str, event_id: str) -> dict | None:
    """Best-effort fetch of a full signed event. Relay API shapes vary; failures are OK."""
    candidates = [
        f"{relay_http.rstrip('/')}/events/{event_id}",
        f"{relay_http.rstrip('/')}/api/events/{event_id}",
        f"{relay_http.rstrip('/')}/event/{event_id}",
    ]
    for url in candidates:
        try:
            req = urllib.request.Request(url, headers={"Accept": "application/json"})
            with urllib.request.urlopen(req, timeout=5) as resp:
                data = json.loads(resp.read().decode())
            if isinstance(data, dict) and (data.get("sig") or data.get("signature")):
                return data
            if isinstance(data, dict) and isinstance(data.get("event"), dict):
                ev = data["event"]
                if ev.get("sig") or ev.get("signature"):
                    return ev
        except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, json.JSONDecodeError):
            continue
    return None


def export_events(
    messages: list[dict],
    *,
    relay_url: str,
    channel_id: str,
    channel_name: str,
    bot_pubkey: str | None,
    label: str,
    try_fetch: bool,
) -> dict:
    relay_http = relay_url.replace("ws://", "http://").replace("wss://", "https://")
    events: list[dict] = []
    for raw in messages:
        if not isinstance(raw, dict):
            continue
        if try_fetch and raw.get("id"):
            fetched = _try_fetch_signed(relay_http, str(raw["id"]))
            if fetched:
                raw = {**raw, **fetched, "sig_status": "present (fetched)"}
            elif not (raw.get("sig") or raw.get("signature")):
                raw = {
                    **raw,
                    "sig_status": (
                        "unavailable: REST message list omitted sig; "
                        f"HTTP full-event fetch against {relay_http} also failed "
                        f"for id={raw.get('id')}"
                    ),
                }
        events.append(_normalize_event(raw))

    payload = {
        "label": label,
        "relay_url": relay_url,
        "channel_name": channel_name,
        "channel_uuid": channel_id,
        "bot_pubkey": bot_pubkey,
        "sig_export_policy": (
            "Preserve complete signed Nostr events when available. "
            "Never export private keys. "
            "If sig cannot be exported, record sig_status reason and omit sig field."
        ),
        "events": events,
    }
    _assert_no_secrets(payload)
    return payload


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--from-messages-json", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--relay", default="ws://localhost:3000")
    ap.add_argument("--channel-id", required=True)
    ap.add_argument("--channel-name", default="")
    ap.add_argument("--bot-pubkey", default=None)
    ap.add_argument("--label", default="C1-DRY · TEST")
    ap.add_argument(
        "--try-fetch-signed",
        action="store_true",
        help="Attempt HTTP full-event fetch to recover sig when list API omits it",
    )
    args = ap.parse_args()

    raw = json.loads(args.from_messages_json.read_text())
    if isinstance(raw, dict) and "events" in raw:
        messages = raw["events"]
    elif isinstance(raw, list):
        messages = raw
    else:
        raise SystemExit("messages json must be a list or {events:[...]}")

    payload = export_events(
        messages,
        relay_url=args.relay,
        channel_id=args.channel_id,
        channel_name=args.channel_name,
        bot_pubkey=args.bot_pubkey,
        label=args.label,
        try_fetch=args.try_fetch_signed,
    )
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2) + "\n")
    present = sum(1 for e in payload["events"] if e.get("sig_status", "").startswith("present"))
    missing = len(payload["events"]) - present
    print(
        f"wrote {args.out} events={len(payload['events'])} "
        f"sig_present={present} sig_unavailable={missing}",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
