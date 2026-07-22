#!/usr/bin/env python3
"""C1-dry core path — SIMULATED / TEST payment only. NON-C2.

Exercises create → claim → precheck → fulfill(fake generate) → status(delivered)
against makereel-core with gateway/generate mocked. No real Stars, no real x402.

Usage (from makereel-core venv or with PYTHONPATH):

  cd /Users/howard/Projects/x402/MakeReel/makereel-core
  BUZZ_FUEL_ENABLED=1 INTERNAL_API_KEY=test-key \\
    uv run python /Users/howard/orca/projects/buzz402/buzz-fuel-poc/scripts/c1_dry_core_path.py

Writes redacted evidence under buzz-fuel-poc/evidence/<timestamp>/ when --write-evidence.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from unittest.mock import patch

# MakeReel core on path
CORE = Path("/Users/howard/Projects/x402/MakeReel/makereel-core")
sys.path.insert(0, str(CORE))

from fastapi import FastAPI
from fastapi.testclient import TestClient

from api import buzz_fuel, config


LABEL = "C1-DRY · SIMULATED PAYMENT · NON-C2 · TEST"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-evidence", action="store_true")
    args = parser.parse_args()

    config.BUZZ_FUEL_ENABLED = True
    config.INTERNAL_API_KEY = "c1-dry-test-key"
    config.BUZZ_FUEL_BOT_USERNAME = "MakeReel_xyz_bot"
    config.PUBLIC_BASE_URL = "https://example.test"
    config.MINIAPP_TEST_USER_IDS = set()
    buzz_fuel._reset_store_for_tests()

    app = FastAPI()
    app.include_router(buzz_fuel.router)
    client = TestClient(app)
    auth = {"Authorization": "Bearer c1-dry-test-key"}

    event_id = "c1dry" + "a" * 40
    print(f"[{LABEL}] create intent for event {event_id[:16]}…")
    r = client.post(
        "/internal/buzz-fuel/intents",
        json={"buzz_channel_id": "c1-dry-channel", "buzz_request_event_id": event_id},
        headers=auth,
    )
    assert r.status_code == 200, r.text
    created = r.json()
    assert created["status"] == "awaiting_claim"
    assert "t.me/MakeReel_xyz_bot?start=fuel_" in created["telegram_url"]

    def fake_quote(model_family, body, mode="custom"):
        return {"x402Version": 2, "accepts": [{"amount": "500000", "network": "base-test"}]}

    print(f"[{LABEL}] claim as tg_user=9001 (mock quote $0.50 → Stars)")
    with (
        patch("api.buzz_fuel.gateway.request_quote", side_effect=fake_quote),
        patch(
            "api.buzz_fuel.accounts.get_or_create_tg",
            return_value=({"account_id": "acct-c1dry"}, True),
        ),
    ):
        claim = client.post(
            f"/internal/buzz-fuel/intents/{created['start_token']}/claim",
            json={"tg_user_id": 9001, "tg_username": "c1dry"},
            headers=auth,
        )
    assert claim.status_code == 200, claim.text
    claim_body = claim.json()
    stars = claim_body["amount_stars"]
    order_id = claim_body["order_id"]
    assert claim_body["invoice_payload"] == f"bf:{order_id}"
    print(f"[{LABEL}] invoice amount_stars={stars} (from quote, not hard-coded)")

    pre = client.post(
        f"/internal/buzz-fuel/orders/{order_id}/precheck",
        json={"tg_user_id": 9001, "stars_amount": stars},
        headers=auth,
    )
    assert pre.json() == {"ok": True}

    print(f"[{LABEL}] fulfill with SIMULATED successful_payment (fake tg_generate)")
    with patch(
        "api.buzz_fuel.internal.tg_generate",
        return_value={
            "job_id": "job-c1dry-sim",
            "status": "pending",
            "price_usdc": "0.50",
            "tx": None,  # NON-C2 — no settlement
        },
    ):
        ful = client.post(
            f"/internal/buzz-fuel/orders/{order_id}/fulfill",
            json={
                "tg_user_id": 9001,
                "stars_charge_id": "SIMULATED-charge-c1dry",
                "stars_amount": stars,
            },
            headers=auth,
        )
    assert ful.status_code == 200, ful.text
    assert ful.json()["job_id"] == "job-c1dry-sim"
    assert ful.json()["status"] == "paid"
    print(f"[{LABEL}] intent paid/running (Fueled equivalent) job_id=job-c1dry-sim")

    with (
        patch(
            "api.buzz_fuel.internal.poll_and_download",
            return_value={
                "job_id": "job-c1dry-sim",
                "status": "succeeded",
                "video_ready": True,
                "share_token": "simshare",
            },
        ),
        patch(
            "api.buzz_fuel.storage.get_manifest_entry",
            return_value={"job_id": "job-c1dry-sim", "share_token": "simshare"},
        ),
    ):
        st = client.get(
            f"/internal/buzz-fuel/intents/{created['intent_id']}",
            headers=auth,
        )
    assert st.status_code == 200
    status = st.json()
    assert status["status"] == "delivered"
    assert status["share_url"].endswith("/s/simshare")
    assert "tg_user" not in str(status)
    assert "SIMULATED-charge" not in str(status)
    print(f"[{LABEL}] delivered share_url={status['share_url']}")

    evidence = {
        "label": LABEL,
        "claims": {
            "real_stars": False,
            "real_x402": False,
            "internal_token_bypass": False,
            "non_c2": True,
        },
        "correlation": {
            "buzz_request_event_id": event_id,
            "intent_id": created["intent_id"],
            "order_id": order_id,
            "job_id": "job-c1dry-sim",
            "share_url": status["share_url"],
            "amount_stars": stars,
        },
        "public_status": status,
        "telegram_url": created["telegram_url"],
    }
    print(json.dumps(evidence, indent=2))

    if args.write_evidence:
        ts = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
        out = Path(__file__).resolve().parents[1] / "evidence" / ts
        out.mkdir(parents=True, exist_ok=True)
        (out / "c1-dry-core-path.json").write_text(json.dumps(evidence, indent=2) + "\n")
        (out / "LABEL.txt").write_text(LABEL + "\n")
        print(f"wrote {out}")

    print(f"[{LABEL}] PASS — plumbing only; not real Stars; not x402 settlement")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
