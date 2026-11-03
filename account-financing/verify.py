#!/usr/bin/env python3
"""Check the released projection of a recorded local Solana run with Python 3.

The input contains original amount bytes, selected states, token balances and
fingerprints. Signed packets and full snapshots remain outside this exhibit.
The checks below verify the relationships among the released observations.
"""
from dataclasses import dataclass, replace
from hashlib import sha256
import json
from pathlib import Path
import struct
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def amounts(hex_string, count):
    return struct.unpack("<" + "Q" * count, bytes.fromhex(hex_string))


@dataclass(frozen=True)
class State:
    drawn: int
    facility_repaid: int
    supplier: str = ""
    creditor: str = ""
    revision: int = 0
    earned: int = 0
    paid: int = 0
    original: int = 0
    outstanding: int = 0
    repaid: int = 0
    extinguished: int = 0


def read_state(row, owners):
    facility = amounts(row["facility_drawn_repaid_le"], 2)
    if "supplier_sha256" not in row:
        return State(*facility)
    principal = amounts(row["originated_outstanding_repaid_extinguished_le"], 4)
    require(principal[0] == sum(principal[1:]), "Principal is not conserved")
    return State(*facility, owners[row["supplier_sha256"]], owners[row["creditor_sha256"]],
                 row["ownership_revision"], *amounts(row["earned_paid_le"], 2), *principal)


STEPS = (
    ("work-earned", 12), ("title-sold", 17), ("second-sale-refused", None),
    ("supplier-collected", 13), ("old-creditor-refused", None),
    ("interest-serviced", 16), ("company-repaid", 16),
    ("facility-interest-serviced", 18), ("issuer-repaid", 18),
)


def verify(data, root):
    require(data["schema"] == "dexter/account-financing/1", "Unexpected evidence schema")
    files = {"rules/" + name + ".rs" for name in ("claim-terms", "repayment", "supplier-collection", "transfer-title")}
    require(set(data["source_files_sha256"]) == files, "Unexpected or missing source excerpts")
    for name in files:
        expected = data["source_files_sha256"][name]
        require(sha256((root / name).read_bytes()).hexdigest() == expected, "Excerpt changed: " + name)
    roles = data["roles"]
    require(set(roles) == {"supplier", "company", "issuer", "buyer_of_debt", "original_funder"},
            "Missing or unexpected party")
    owners = {fingerprint: name for name, fingerprint in roles.items()}
    require(len(owners) == len(roles) and all(len(bytes.fromhex(key)) == 32 for key in owners),
            "Parties must have distinct fingerprints")
    states = [read_state(row, owners) for row in data["states"]]
    events = data["events"]
    require([(row["step"], row.get("kind")) for row in events] == list(STEPS), "Events differ from the case")
    packets = {row["packet_sha256"] for row in events}
    require(len(packets) == len(events) and all(len(bytes.fromhex(key)) == 32 for key in packets),
            "Missing or duplicate packet fingerprint")
    before = State(0, 0)
    results = []
    for event in events:
        step = event["step"]
        i, j = event["before"], event["after"]
        require(0 <= i < len(states) and 0 <= j < len(states), "Invalid state reference in " + step)
        require(states[i] == before, "Discontinuous account state in " + step)
        balances = {role: amounts(pair, 2) for role, pair in event["token_balances_le"].items()}
        cash = {role: end - start for role, (start, end) in balances.items()}
        require(sum(cash.values()) == 0, "Token balance changes do not balance in " + step)
        after, expected_cash = before, {}
        if "instruction_error" in event:
            require(event.get("kind") is None and event["instruction_error"] == [0, {"Custom": 6045}],
                    "Expected a refusal for the wrong creditor in " + step)
            require(event["attempted_creditor"] == "issuer" and before.creditor == "buyer_of_debt",
                    "The refused party must be the former creditor")
            require(event["protected_accounts"] > 0 and len(bytes.fromhex(event["before_digest"])) == 32
                    and event["before_digest"] == event["after_digest"], "Protected state changed in " + step)
            description = "former creditor refused, recorded protected state unchanged"
        else:
            require(event.get("kind") is not None, "Missing refusal result in " + step)
            gross, principal, interest = amounts(event["gross_principal_interest_le"], 3)
            require(gross > 0, "Zero receipt in " + step)
            if step == "work-earned":
                require(principal == interest == 0, "Unexpected repayment at delivery")
                after = replace(before, supplier="supplier", creditor="issuer", revision=1,
                                original=gross, earned=gross, outstanding=gross)
                description = f"supplier earned {gross:,} units"
            elif step == "title-sold":
                price, costs, old_basis, new_basis = amounts(event["consideration_cost_old_basis_new_basis_le"], 4)
                require(event["seller"] == before.creditor and event["buyer"] == "buyer_of_debt",
                        "Wrong parties to the sale")
                require(principal == interest == 0 and gross == before.outstanding == old_basis
                        and 0 < price < gross and new_basis == price + costs, "Sale does not reconcile")
                expected_cash = {"buyer_cash": -new_basis, "issuer_sale_proceeds": price, "fees": costs}
                after = replace(before, creditor=event["buyer"], revision=before.revision + 1)
                description = f"{gross:,} of debt changed hands for {price:,} plus {costs:,} in costs"
            elif step == "supplier-collected":
                net, fee, reimbursement, margin = amounts(event["net_fee_reimbursement_margin_le"], 4)
                require(event["kind"] == 13 and principal == interest == 0 and gross == net + fee
                        and gross == before.earned - before.paid, "Supplier entitlement does not reconcile")
                charge = reimbursement + margin
                expected_cash = {"funding_cash": -gross, "supplier_cash": net - charge, "fees": fee, "sponsor": charge}
                after = replace(before, paid=before.paid + gross, drawn=before.drawn + gross)
                description = f"supplier received {net-charge:,}, company still owed {after.outstanding:,}"
            else:
                require(gross == principal + interest, "Repayment exceeds its principal and interest")
                fee, = amounts(event["interest_fee_le"], 1)
                require(fee <= interest, "Fee exceeds interest collected")
                if fee:
                    expected_cash["interest_fees"] = fee
                if event["kind"] == 16:
                    payee = before.creditor
                    payer = "company" if step == "company-repaid" else "issuer"
                    require(principal == (before.outstanding if payer == "company" else 0),
                            "Wrong company principal allocation")
                    expected_cash.update({payer + "_cash": -gross, "buyer_repayment": gross - fee})
                    after = replace(before, outstanding=before.outstanding-principal, repaid=before.repaid+principal)
                    description = f"{payer} paid {gross:,} toward company debt, facility principal {after.drawn:,}"
                else:
                    payer, payee = "issuer", "original_funder"
                    require(before.outstanding == 0 and principal == (before.drawn if step == "issuer-repaid" else 0),
                            "Wrong facility principal allocation")
                    expected_cash.update(issuer_cash=-gross, funding_cash=gross-fee)
                    after = replace(before, drawn=before.drawn-principal, facility_repaid=before.facility_repaid+principal)
                    description = f"issuer paid {gross:,} toward its own facility, principal left {after.drawn:,}"
                require(event["payer"] == payer and event["payee"] == payee, "Wrong repayment parties")
        require(cash == expected_cash, "Token destinations or quantities differ in " + step)
        require(states[j] == after, "Unexpected supplier, creditor, debt or funding balance in " + step)
        results.append(step + "  " + description)
        before = after
    require(before.earned == before.paid == before.repaid == before.facility_repaid > 0
            and before.outstanding == before.drawn == 0, "Ending obligations do not reconcile")
    return results


if __name__ == "__main__":
    directory = Path(__file__).resolve().parent
    try:
        output = verify(json.loads((directory / "evidence.json").read_text()), directory)
    except (KeyError, IndexError, ValueError, TypeError, struct.error, OSError) as exc:
        sys.exit("Verification failed: " + str(exc))
    print("\n".join(output))
    print("Passed. Selected receipt fields, account changes, token movements and refusal fingerprints agree.")
    print("The original packets are withheld. This check does not verify signatures or execute the programs.")
