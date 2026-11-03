// Selected Rust source from the program build used in this local exercise.
// Lines between excerpts are omitted. This file is for reading.

// Company debt can only be paid to the current creditor's custody.
    require!(a.claim.revision == args.expected_claim_revision && a.run.ownership_revision == args.expected_ownership_revision,
        CoreError::FundingRevision);
    reservation_matches(&a.reservation, &a.claim)?;
    owner_cash(&a.repayment_principal, a.repayment_authority.key(), args.repayment_owner_generation, &a.from, a.mint.key())?;
    custody_token(&a.creditor_custody, &a.destination, a.mint.key())?;
    require!(a.creditor_custody.key() == a.run.creditor_custody
        && a.creditor_custody.principal == a.run.current_creditor, CoreError::FundingCreditor);

// Transfer principal and the agreed share of collected interest.
    let credited = plus(split.principal, allocation.lender)?;
    transfer_checked_exact(&a.from.to_account_info(), &a.destination.to_account_info(), &a.repayment_authority.to_account_info(),
        &a.mint, a.token_program.key(), credited, &[])?;
    if allocation.protocol > 0 {
        let dest = a.protocol_destination.as_ref().ok_or_else(||error!(CoreError::FundingTokenRouting))?;
        transfer_checked_exact(&a.from.to_account_info(), &dest.to_account_info(), &a.repayment_authority.to_account_info(),
            &a.mint, a.token_program.key(), allocation.protocol, &[])?;
    }
    if credited > 0 { a.creditor_custody.credit(credited)?; }

// Advance the company debt by the principal that was paid.
    a.destination.reload()?; a.creditor_custody.check(a.destination.amount)?;
    a.run.principal = principal_next; a.run.interest = paid_interest;
    a.run.fee_authority_released = plus(a.run.fee_authority_released, unused_fee)?;
    a.claim.debt = minus(a.claim.debt, split.principal)?; a.claim.repaid = plus(a.claim.repaid, split.principal)?;

// The issuer's debt names a separate financier and custody.
    valid_principal(&a.issuer)?; a.facility.check(a.facility.key())?;
    require!(a.facility.external && a.facility.issuer==a.issuer.key() && a.facility.mint==a.mint.key()
        && a.facility.custody==a.financier_custody.key() && a.facility.financier==a.financier_custody.principal,
        CoreError::FundingFacility);
    require!(a.issuer.domain==a.financier_custody.domain,CoreError::FundingFacility);
    require!(a.facility.revision==args.expected_facility_revision,CoreError::FundingRevision);
    owner_cash(&a.repayment_principal,a.repayment_authority.key(),args.repayment_owner_generation,&a.from,a.mint.key())?;
    custody_token(&a.financier_custody,&a.destination,a.mint.key())?;

// Pay the financier before reducing the separate facility draw.
    let credited=plus(split.principal,allocation.lender)?;
    transfer_checked_exact(&a.from.to_account_info(),&a.destination.to_account_info(),&a.repayment_authority.to_account_info(),
        &a.mint,a.token_program.key(),credited,&[])?;
    if allocation.protocol>0 {
        let destination=a.protocol_destination.as_ref().ok_or_else(||error!(CoreError::FundingTokenRouting))?;
        transfer_checked_exact(&a.from.to_account_info(),&destination.to_account_info(),&a.repayment_authority.to_account_info(),
            &a.mint,a.token_program.key(),allocation.protocol,&[])?;
    }
    if credited>0 { a.financier_custody.credit(credited)?; }
    credit_protocol(&mut route,a.protocol_custody.as_ref(),a.protocol_destination.as_ref(),&mut a.financier_custody,allocation.protocol)?;
    a.destination.reload()?; a.financier_custody.check(a.destination.amount)?;
    a.facility.drawn=minus(a.facility.drawn,split.principal)?;a.facility.repaid=plus(a.facility.repaid,split.principal)?;
