# Financing a job

A company orders work on credit. By the time the supplier finishes, the company owes 60,000. Its lender would rather have cash today and agrees to sell the debt. That leaves two different rights to account for. The supplier earned payment for doing the work. The new lender bought the right to be repaid by the company.

Dexter ran this case on November 1, 2026, using its compiled Account programs on a local Solana runtime. All amounts below are test-token units. The supplier, company and lenders were test participants.

## The sale

Both lenders sign the transfer. Here is where each party stands immediately afterward.

| Party | Position after the sale |
| --- | --- |
| Company | Still owes 60,000, now to the new lender. |
| Supplier | Can collect the same 60,000 gross under the terms accepted for the job. |
| Original lender | Received 57,000 for selling the claim. |
| New lender | Paid 57,000 plus 123 in costs to acquire the claim. |

When the original lender tries to sell it again, the program checks who owns the claim and refuses the sale. [Ownership rules](rules/claim-terms.rs) · [Transfer code](rules/transfer-title.rs)

## Supplier payout

The supplier collects against the funds set aside for the job. Its agreed seller fee is 829. A separate charge of 550 covers the cost and margin for sponsoring the transaction. The supplier receives **58,621**. Paying it leaves the company's 60,000 principal outstanding. [Supplier collection code](rules/supplier-collection.rs)

## Repayment

A repayment naming the former lender fails. The company then pays the remaining principal and interest to the current creditor under the agreed terms. The original lender had borrowed its own funding for the job. It still owes that financier and later repays it separately. [Repayment code](rules/repayment.rs)

## Check the result

The [recorded outcomes](evidence.json) contain selected balances, changes in claim ownership and two refused operations. The linked Rust files are excerpts from the Account implementation used in the run.

Run the offline check from the repository root.

```sh
python3 account-financing/verify.py
```

The check needs only Python 3. It follows the amounts through nine recorded steps, compares the payer and recipient at each transfer, and checks that the refused transactions left the recorded protected accounts untouched. The original signed transactions were verified separately before publication.

[All Dexter demos](../)
