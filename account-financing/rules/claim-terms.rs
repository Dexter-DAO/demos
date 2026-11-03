// Selected Rust source from the program build used in this local exercise.
// Lines between excerpts are omitted. This file is for reading.

// A title and the principal amounts it governs.
/// Principal title is independent from the original supplier payable/backing.
/// Original basis and adjustments need durable book effects; impairment alone
/// does not forgive the company's debt or change these principal amounts.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq, InitSpace)]
pub struct ReceivableTitle {
    pub lot: Pubkey,
    pub economic_origin: [u8; 32],
    pub accepted_guarantee: Pubkey,
    pub debtor_principal: Pubkey,
    pub asset: Asset,
    pub current_creditor: Pubkey,
    pub repayment_destination: Pubkey,
    pub creditor_book: Pubkey,
    pub ownership_revision: u64,
    pub acquisition_basis: u64,
    pub provenance_hash: [u8; 32],
    pub assignment_and_disclosure_policy_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Debug, PartialEq, Eq, InitSpace)]
pub struct PrincipalAmounts {
    pub originated: u64,
    pub outstanding: u64,
    pub repaid: u64,
    /// Only legally/economically extinguishing, authenticated corrections.
    pub extinguished: u64,
}
impl PrincipalAmounts {
    pub fn validate(&self) -> FinanceResult<()> {
        if add(add(self.outstanding, self.repaid)?, self.extinguished)? != self.originated {
            return Err(FinanceError::InvalidAmounts);
        }
        Ok(())
    }
    pub fn repay(self, amount: u64) -> FinanceResult<Self> {
        self.validate()?; positive(amount)?;
        let next = Self { outstanding: sub(self.outstanding, amount)?, repaid: add(self.repaid, amount)?, ..self };
        next.validate()?; Ok(next)
    }
}

// The supplier's earned collection identifies its own destination and unpaid range.
/// Canonical payable range, authenticated by an existing earned entitlement.
/// There is no arbitrary global caller-owned ID to pre-allocate. Replay checks
/// the receipt first; otherwise paid_before must equal the lot's paid counter
/// and amount must not exceed its current collectible payable after adjustments.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq, InitSpace)]
pub struct EarnedCollection {
    pub version: u16,
    pub domain: [u8; 32],
    pub claim: Pubkey,
    pub payable_lot: Pubkey,
    pub accepted_service_grant_hash: [u8; 32],
    pub reserve_ledger: Pubkey,
    pub token_account: Pubkey,
    pub asset: Asset,
    pub supplier: Pubkey,
    pub destination: Pubkey,
    pub paid_before: u64,
    pub amount: u64,
}
impl EarnedCollection {
    /// Read the authenticated residual after corrections/offsets, not historical
    /// gross earnings or the shared custody balance for unrelated suppliers.
    pub fn validate_unpaid_range(&self, remaining_payable: u64, already_paid: u64) -> FinanceResult<()> {
        self.operation_id()?;
        if self.paid_before != already_paid { return Err(FinanceError::WrongRevision); }
        if self.amount > remaining_payable { return Err(FinanceError::LimitExceeded); }
        Ok(())
    }
    /// Fixed economic identity across relayers, lost responses, attempt nonces,
    /// blockhashes and optional Pool projection. New domain; no old receipt reuse.
    pub fn operation_id(&self) -> FinanceResult<[u8; 32]> {
        if self.version != FINANCING_VERSION { return Err(FinanceError::SchemaVersion); }
        positive(self.amount)?; add(self.paid_before, self.amount)?;
        if self.reserve_ledger != custody_reserve_address(&self.token_account) { return Err(FinanceError::WrongBinding); }
        digest(b"DEXTER:EARNED-COLLECTION:1", self)
    }
}
