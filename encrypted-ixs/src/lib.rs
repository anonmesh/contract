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

    pub struct RelayCount {
        count: u64,
    }

    #[instruction]
    pub fn relay_increment(
        current: Enc<Shared, RelayCount>,
    ) -> Enc<Shared, RelayCount> {
        let c = current.to_arcis();
        let next = RelayCount {
            count: c.count + 1,
        };
        current.owner.from_arcis(next)
    }
}
