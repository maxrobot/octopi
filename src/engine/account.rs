use rust_decimal::Decimal;

#[derive(Clone)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        let expected_total = self.available + self.held;

        self.total == expected_total
    }

    pub fn is_available(&self) -> bool {
        !self.locked
    }
}
