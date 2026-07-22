"""Unit tests for export_buzz_evidence (no network)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from export_buzz_evidence import _assert_no_secrets, export_events


def test_preserves_sig_and_signed_event():
    msgs = [
        {
            "id": "ab" * 32,
            "kind": 9,
            "pubkey": "cd" * 32,
            "created_at": 1,
            "content": "Fueled ✅",
            "tags": [["h", "ch"]],
            "sig": "ef" * 32,
        }
    ]
    out = export_events(
        msgs,
        relay_url="ws://localhost:3000",
        channel_id="ch",
        channel_name="t",
        bot_pubkey="cd" * 32,
        label="TEST",
        try_fetch=False,
    )
    ev = out["events"][0]
    assert ev["sig"] == "ef" * 32
    assert ev["sig_status"] == "present"
    assert ev["signed_event"]["sig"] == "ef" * 32


def test_missing_sig_records_reason_not_null():
    msgs = [{"id": "ab" * 32, "kind": 9, "pubkey": "cd" * 32, "created_at": 1, "content": "x", "tags": []}]
    out = export_events(
        msgs,
        relay_url="ws://localhost:3000",
        channel_id="ch",
        channel_name="t",
        bot_pubkey=None,
        label="TEST",
        try_fetch=False,
    )
    ev = out["events"][0]
    assert "sig" not in ev
    assert "sig_status" in ev
    assert "unavailable" in ev["sig_status"]
    dumped = json.dumps(ev)
    assert '"sig": null' not in dumped


def test_refuses_private_key_fields():
    try:
        _assert_no_secrets({"nsec": "nsec1secret"})
        assert False, "should have exited"
    except SystemExit:
        pass


if __name__ == "__main__":
    test_preserves_sig_and_signed_event()
    test_missing_sig_records_reason_not_null()
    test_refuses_private_key_fields()
    print("ok")
