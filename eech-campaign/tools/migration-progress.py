#!/usr/bin/env python3
"""Validate the active extraction decision; never builds, mutates a card or approves delivery."""
import argparse
import json
from pathlib import Path
import sys


def check(card, action, blocker):
    for name in ("task", "epic", "next_production_change", "next_action", "runtime_checkpoint"):
        if not isinstance(card.get(name), str) or not card[name].strip():
            raise ValueError(f"Missing concrete {name}")
    if card.get("schema") != 1 or not card.get("authority") or not card.get("acceptance"):
        raise ValueError("Missing schema, live authority or acceptance gates")
    findings = {item["id"]: item for item in card["blockers"]}
    if len(findings) != len(card["blockers"]):
        raise ValueError("Duplicate blocker IDs")
    for item in findings.values():
        if item["status"] not in ("open", "closed"):
            raise ValueError("Invalid blocker status")
        for name in ("evidence", "missing_fact", "next_fix"):
            if not isinstance(item.get(name), str) or not item[name].strip():
                raise ValueError(f"Blocker {item['id']} needs {name}")
    batches, limit = card["preparation_batches_without_cutover"], card["preparation_limit"]
    if type(batches) is not int or type(limit) is not int or batches < 0 or limit < 1:
        raise ValueError("Invalid preparation counts")
    selected = findings.get(blocker)
    if action == "fix-blocker" and (not selected or selected["status"] != "open"):
        raise ValueError("Fix a named OPEN blocker; do not invent another preparation goal")
    if action == "characterize" and batches >= limit:
        if not selected or selected["status"] != "open" or not selected["blocks_cutover"]:
            raise ValueError("Preparation limit reached: make the production change or name its actual correctness blocker")
    if action in ("production-cutover", "deliver"):
        if any(x["status"] == "open" and (action == "deliver" or x["blocks_cutover"]) for x in findings.values()):
            raise ValueError("Resolve required findings before this action; the preparation limit never permits unsafe work")
    if action == "deliver" and not all(x["complete"] is True for x in card["acceptance"]):
        raise ValueError("Delivery gates remain incomplete; preparation is not acceptance")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--card", type=Path, default=Path(__file__).resolve().parents[2] / "docs/handover/ACTIVE-EXTRACTION.json")
    parser.add_argument("--action", choices=("check", "fix-blocker", "characterize", "production-cutover", "validate", "review", "deliver"), default="check")
    parser.add_argument("--blocker")
    args = parser.parse_args()
    try:
        card = json.loads(args.card.read_text(encoding="utf-8"))
        check(card, args.action, args.blocker)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"STOP: {error}", file=sys.stderr)
        return 1
    print(json.dumps({"task": card["task"], "action": args.action, "next_action": card["next_action"], "next_production_change": card["next_production_change"]}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
