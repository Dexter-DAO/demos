<p align="center">
  <img src="https://raw.githubusercontent.com/Dexter-DAO/dexter-x402-sdk/main/assets/dexter-wordmark.svg" alt="Dexter" width="360">
</p>

<h1 align="center">Demos</h1>

<p align="center">
  <strong>Dexter builds accounts for businesses run by agents.</strong><br>
  Each case pairs a transaction history with the code that explains the outcome.
</p>

<p align="center">
  <a href="#financing-a-job">Financing a job</a> ·
  <a href="#world-id-on-solana">World ID on Solana</a>
</p>

## Financing a job

A supplier finishes a job. While it waits to collect, a lender sells the company's debt. Dexter Account pays the supplier under the original terms and refuses a later repayment addressed to the former lender.

The case uses test assets and Dexter's compiled programs on a local Solana runtime.

[Read the case](account-financing/README.md) · [Rust source](account-financing/rules/transfer-title.rs) · [Recorded data](account-financing/evidence.json)

<a id="verifying-a-world-id-enrollment-on-solana"></a>

## World ID on Solana

Dexter's bridge let Solana check the World ID registry on World Chain. Anyone could submit a proof linking Ethereum finality, the winning rollup claim and the registry's contents. A Solana credit program used that evidence to verify a person's original World ID proof and upgrade their existing account without another attestation from Dexter.

The archive includes the Rust code and two July 1 mainnet proofs you can verify offline.

[Read the bridge case](https://github.com/Dexter-DAO/world-chain-bridge/blob/main/README.md) · [World Chain proof code](https://github.com/Dexter-DAO/world-chain-bridge/blob/main/source/world-chain-anchor.rs) · [Offline verifier](https://github.com/Dexter-DAO/world-chain-bridge/blob/main/verify.mjs)

<p align="center">
  <img src="assets/dexter-demos-cover.webp" alt="Dexter demos cover with an illustrated payment voucher" width="100%">
</p>

[Dexter](https://dexter.cash/) · [Developer documentation](https://docs.dexter.cash/docs/)

<!-- Each demonstration links to its own README, selected source and recorded result. Add a linked entry here when a new case is ready. -->
