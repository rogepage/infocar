use crate::domain::debt::money::Money;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixOption {
    pub total_with_discount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardInstallment {
    pub quantity: u8,
    pub installment_amount: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditCardOption {
    pub installments: Vec<CreditCardInstallment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentOption {
    pub option_type: String,
    pub base_amount: Money,
    pub pix: PixOption,
    pub credit_card: CreditCardOption,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentSimulationResult {
    pub options: Vec<PaymentOption>,
}
