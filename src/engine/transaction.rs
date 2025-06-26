use rust_decimal::Decimal;

pub struct Transaction {
    pub client: u16,
    pub tx_id: u32,
    pub kind: TransactionType,
    pub amount: Option<Decimal>,
}

pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute { referenced_tx_id: u32 },
    Resolve { referenced_tx_id: u32 },
    Chargeback { referenced_tx_id: u32 },
}

impl Transaction {
    pub fn is_valid(&self) -> bool {
        match self.kind {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                self.amount.is_some() && self.amount.unwrap() > Decimal::ZERO
            }
            TransactionType::Dispute { .. }
            | TransactionType::Resolve { .. }
            | TransactionType::Chargeback { .. } => self.amount.is_none(),
        }
    }

    pub fn new_deposit(client: u16, tx_id: u32, amount: Decimal) -> Self {
        if amount <= Decimal::ZERO {
            panic!("Deposit amount must be positive");
        }
        Self {
            client,
            tx_id,
            kind: TransactionType::Deposit,
            amount: Some(amount),
        }
    }

    pub fn new_withdrawal(client: u16, tx_id: u32, amount: Decimal) -> Self {
        if amount <= Decimal::ZERO {
            panic!("Withdrawal amount must be positive");
        }
        Self {
            client,
            tx_id,
            kind: TransactionType::Withdrawal,
            amount: Some(amount),
        }
    }

    pub fn new_dispute(client: u16, tx_id: u32, referenced_tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Dispute { referenced_tx_id },
            amount: None,
        }
    }

    pub fn new_resolve(client: u16, tx_id: u32, referenced_tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Resolve { referenced_tx_id },
            amount: None,
        }
    }

    pub fn new_chargeback(client: u16, tx_id: u32, referenced_tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Chargeback { referenced_tx_id },
            amount: None,
        }
    }
}
