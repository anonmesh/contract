use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct PaymentInput {
        amount: u64,
    }

    #[instruction]
    pub fn payment_v3(
        payment_input: Enc<Shared, PaymentInput>,
    ) -> Enc<Shared, u64> {
        let payment = payment_input.to_arcis();
        payment_input.owner.from_arcis(payment.amount)
    }
}
