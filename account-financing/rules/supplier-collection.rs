// Selected Rust source from the program build used in this local exercise.
// Lines between excerpts are omitted. This file is for reading.

// The claim names the supplier and identifies the earned range.
pub fn collect<'info>(ctx: Context<'info, CollectFinancier<'info>>,
    operation_id: [u8; 32], paid_before: u64, amount: u64, expected_revision: u64,
    service_without_pool: bool, sponsorship: Option<SponsorQuote>) -> Result<()> {
    ctx.accounts.claim.check(ctx.accounts.claim.key())?;
    ctx.accounts.run.check(ctx.accounts.run.key(), ctx.accounts.claim.key(), &ctx.accounts.claim)?;
    let supplier_destination = associated_token::get_associated_token_address(
        &ctx.accounts.claim.terms.supplier, &ctx.accounts.claim.terms.asset);
    let collection = EarnedCollection { version: FINANCING_VERSION, domain: ctx.accounts.claim.terms.domain,
        claim: ctx.accounts.claim.key(), payable_lot: ctx.accounts.run.key(),
        accepted_service_grant_hash: ctx.accounts.claim.acceptance_hash,
        reserve_ledger: ctx.accounts.claim.custody, token_account: ctx.accounts.custody.token_account,
        asset: Asset { mint: ctx.accounts.claim.terms.asset, token_program: token::ID },
        supplier: ctx.accounts.claim.terms.supplier, destination: supplier_destination, paid_before, amount };
    require!(operation_id == math(collection.operation_id())?, CoreError::FundingReplay);
    let digest = financial_digest(b"DEXTER:COLLECT-FINANCIER:1", &collection)?;
    // A recorded collection is the same economic event regardless of caller,
    // nonce, transport, Pool choice or an attempted new sponsorship quote.
    // Historical success does not reimburse a sponsor or transfer cash twice.
    if replay(&ctx.accounts.receipt, &digest)? { return Ok(()); }

// The token destination and unpaid balance must match that claim.
    custody_token(&ctx.accounts.custody, &ctx.accounts.custody_token, ctx.accounts.mint.key())?;
    validate_custody_authority(&ctx.accounts.custody, &ctx.accounts.wallet_binding,
        &ctx.accounts.wallet_authority)?;
    require!(ctx.accounts.destination.key() == supplier_destination
        && ctx.accounts.destination.owner == ctx.accounts.claim.terms.supplier,
        CoreError::FundingTokenRouting);
    math(collection.validate_unpaid_range(
        minus(ctx.accounts.run.gross_earned, ctx.accounts.run.gross_paid)?, ctx.accounts.run.gross_paid))?;
    require!(amount <= ctx.accounts.claim.payable && amount <= ctx.accounts.custody.payable
        && amount <= ctx.accounts.facility.reserved, CoreError::FundingAmount);

// The external facility records its own draw and transfers the supplier's net payout.
    // Facility interest is on the existing external issuer debt, before this
    // draw changes principal. Issuer-owned capital creates no debt to itself.
    if ctx.accounts.facility.external {
        let (interest, accrued) = math(ctx.accounts.facility.interest.accrue(
            ctx.accounts.facility.drawn, Clock::get()?.unix_timestamp, &ctx.accounts.facility.interest_terms))?;
        ctx.accounts.facility.interest = interest;
        ctx.accounts.facility.drawn = plus(ctx.accounts.facility.drawn, amount)?;
        changes.push(inc(financier, FACILITY_ASSETS, amount));
        changes.push(inc(issuer, FACILITY_DEBT, amount));
        changes.push(inc(issuer, INTEREST_OWED, accrued));
        changes.push(inc(financier, INTEREST_OWNED, accrued));
    }

    transfer_from_custody(&ctx.accounts.custody, &ctx.accounts.wallet_binding,
        &ctx.accounts.wallet_authority, &ctx.accounts.custody_token.to_account_info(),
        &ctx.accounts.destination.to_account_info(), &ctx.accounts.mint, ctx.accounts.token_program.key(),
        settlement.supplier_net)?;

// The same collection reduces the unpaid obligation.
    ctx.accounts.custody_token.reload()?;
    ctx.accounts.custody.payable = minus(ctx.accounts.custody.payable, amount)?;
    ctx.accounts.custody.accounted_cash = minus(ctx.accounts.custody.accounted_cash, amount)?;
    ctx.accounts.custody.revision = plus(ctx.accounts.custody.revision, 1)?;
    ctx.accounts.custody.check(ctx.accounts.custody_token.amount)?;
    ctx.accounts.facility.reserved = minus(ctx.accounts.facility.reserved, amount)?;
    ctx.accounts.facility.revision = plus(ctx.accounts.facility.revision, 1)?;
    ctx.accounts.claim.payable = minus(ctx.accounts.claim.payable, amount)?;
    ctx.accounts.claim.paid = plus(ctx.accounts.claim.paid, amount)?;
    ctx.accounts.claim.seller_fee_state = fee_state;
    ctx.accounts.claim.revision = plus(ctx.accounts.claim.revision, 1)?;
    ctx.accounts.run.gross_paid = plus(ctx.accounts.run.gross_paid, amount)?;
    ctx.accounts.claim.check(ctx.accounts.claim.key())?;
