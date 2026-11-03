// Selected Rust source from the program build used in this local exercise.
// Lines between excerpts are omitted. This file is for reading.

// Account instructions require signatures from both owners.
#[derive(Accounts)]
#[instruction(args: AssignFundedRunArgs)]
pub struct AssignFundedRun<'info> {
    #[account(mut)] pub payer: Signer<'info>,
    pub seller_owner: Signer<'info>, pub buyer_owner: Signer<'info>,
    pub principal: Box<Account<'info, Principal>>,
    pub seller: Box<Account<'info, Principal>>, pub buyer: Box<Account<'info, Principal>>,

// A sale binds the parties, accounts and consideration.
pub fn assign_run<'info>(ctx: Context<'info, AssignFundedRun<'info>>, args: AssignFundedRunArgs) -> Result<()> {
    let a = ctx.accounts;
    let operation_id = crate::funding::operation_id(b"assign-run",a.seller_owner.key(), &args.request_id);
    let digest = financial_digest(b"DEXTER:FUNDED-RUN-ASSIGN:1", &(
        (a.principal.key(),a.claim.key(),a.run.key(),a.seller.key(),a.buyer.key()),
        (a.seller_owner.key(),a.buyer_owner.key(),args.seller_owner_generation,args.buyer_owner_generation),
        (a.seller_custody.key(),a.buyer_custody.key(),a.purchase_from.key(),a.seller_destination.key(),a.buyer_destination.key()),
        (a.mint.key(),args.consideration,args.acquisition_costs,args.acquisition_cost_evidence_hash,args.expected_ownership_revision),
        (a.cost_principal.as_ref().map(|x|x.key()),a.cost_custody.as_ref().map(|x|x.key()),a.cost_destination.as_ref().map(|x|x.key()))))?;
    if replay(&a.receipt,&digest)? { return Ok(()); }
    a.claim.check(a.claim.key())?; a.run.check(a.run.key(),a.claim.key(),&a.claim)?;

// Both owners authorize the sale and the seller must own the current title.
    valid_principal(&a.principal)?; owner(&a.seller,a.seller_owner.key(),args.seller_owner_generation)?;
    owner_cash(&a.buyer,a.buyer_owner.key(),args.buyer_owner_generation,&a.purchase_from,a.mint.key())?;
    require!(a.seller_owner.key().is_on_curve() && a.principal.key()==a.claim.terms.principal
        && a.principal.domain==a.claim.terms.domain && a.seller.domain==a.principal.domain
        && a.buyer.domain==a.principal.domain && a.buyer.key()!=a.seller.key()
        && a.buyer.key()!=a.principal.key(),CoreError::FundingAssignment);
    require!(a.claim.assignment_policy_hash==within_domain_assignment_policy(&a.principal.domain),CoreError::FundingAssignment);
    require!(a.claim.revision==args.expected_claim_revision && a.run.ownership_revision==args.expected_ownership_revision,
        CoreError::FundingRevision);
    require!(a.run.current_creditor==a.seller.key() && a.run.creditor_custody==a.seller_custody.key(),CoreError::FundingCreditor);
    custody_token(&a.seller_custody,&a.seller_destination,a.mint.key())?;
    custody_token(&a.buyer_custody,&a.buyer_destination,a.mint.key())?;
    require!(a.seller_custody.principal==a.seller.key() && a.buyer_custody.principal==a.buyer.key()
        && a.run.mint==a.mint.key(),CoreError::FundingTokenRouting);
    reservation_matches(&a.reservation,&a.claim)?;
    require!(args.consideration>0,CoreError::FundingAmount);
    let acquisition_basis=plus(args.consideration,args.acquisition_costs)?;

// The buyer pays before title changes hands.
    transfer_checked_exact(&a.purchase_from.to_account_info(),&a.seller_destination.to_account_info(),&a.buyer_owner.to_account_info(),
        &a.mint,a.token_program.key(),args.consideration,&[])?;
    a.seller_custody.credit(args.consideration)?;
    if args.acquisition_costs>0 {
        let destination=a.cost_destination.as_ref().ok_or_else(||error!(CoreError::FundingAssignment))?;
        transfer_checked_exact(&a.purchase_from.to_account_info(),&destination.to_account_info(),&a.buyer_owner.to_account_info(),
            &a.mint,a.token_program.key(),args.acquisition_costs,&[])?;
        credit_protocol(&mut cost_route,a.cost_custody.as_ref(),a.cost_destination.as_ref(),&mut a.seller_custody,args.acquisition_costs)?;
    }
    a.seller_destination.reload()?; a.seller_custody.check(a.seller_destination.amount)?;
    let old_creditor=a.run.current_creditor; let old_basis=a.run.acquisition_basis;
    let new_revision=plus(a.run.ownership_revision,1)?;
    // The whole residual includes the lender's uncollected interest entitlement.
    // The protocol allocation and all borrower/supplier terms stay frozen.
    let provenance=financial_digest(b"DEXTER:FUNDED-TITLE-TRANSFER:1", &(
        a.run.provenance_hash,digest,old_creditor,a.buyer.key(),new_revision,a.run.principal.outstanding,
        interest_due,old_basis,acquisition_basis,a.run.interest_terms.clone()))?;
    a.run.current_creditor=a.buyer.key(); a.run.creditor_custody=a.buyer_custody.key();
    a.run.ownership_revision=new_revision; a.run.acquisition_basis=acquisition_basis;
    a.run.provenance_hash=provenance; a.run.interest=interest;

// The books and receipt record the same change of creditor.
    let company=a.principal.key(); let buyer=a.buyer.key();
    let mut changes=vec![inc(company,INTEREST_OWED,new_interest),
        inc(old_creditor,INTEREST_OWNED,new_interest),
        dec(old_creditor,OWNED_PRINCIPAL,a.run.principal.outstanding),inc(buyer,OWNED_PRINCIPAL,a.run.principal.outstanding),
        dec(old_creditor,INTEREST_OWNED,interest_due),inc(buyer,INTEREST_OWNED,interest_due),
        inc(buyer,CASH_SENT,acquisition_basis),inc(old_creditor,CASH_RECEIVED,args.consideration)];
    if args.acquisition_costs>0 { changes.push(inc(cost_beneficiary,CASH_RECEIVED,args.acquisition_costs)); }
    let mut record=record_base(company,operation_id,digest,FINANCE_ASSIGN,a.mint.key(),a.claim.key(),a.run.key(),a.run.principal.outstanding);
    record.debit_principal=buyer;record.debit_authority=a.buyer_owner.key();record.debit_from=a.purchase_from.key();
    record.previous_creditor=old_creditor;record.current_creditor=buyer;record.ownership_revision=new_revision;
    record.creditor_destination=a.buyer_destination.key();record.assignment_consideration=args.consideration;
    record.assignment_acquisition_cost=args.acquisition_costs;record.previous_acquisition_basis=old_basis;
    record.new_acquisition_basis=acquisition_basis;record.assignment_cost_beneficiary=cost_beneficiary;
    record.assignment_cost_destination=cost_destination;
    journal(&mut a.receipt,record,&changes,ctx.remaining_accounts,&a.pool_program.to_account_info(),false,true)?;
    a.claim.check(a.claim.key())?;a.run.check(a.run.key(),a.claim.key(),&a.claim)?;
    Ok(())
}
