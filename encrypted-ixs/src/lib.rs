use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct PaymentStats {
        total_payments: u64,
        total_volume: u64,
    }

    pub struct PaymentInput {
        amount: u64,
    }

    #[instruction]
    pub fn payment_stats(
        payment_input: Enc<Shared, PaymentInput>,
        stats_ctxt: Enc<Mxe, PaymentStats>,
    ) -> Enc<Mxe, PaymentStats> {
        let payment = payment_input.to_arcis();
        let mut stats = stats_ctxt.to_arcis();

        stats.total_payments += 1;
        stats.total_volume += payment.amount;

        stats_ctxt.owner.from_arcis(stats)
    }

    // Initialize stats
    #[instruction]
    pub fn init_payment_stats(mxe: Mxe) -> Enc<Mxe, PaymentStats> {
        let stats = PaymentStats {
            total_payments: 0,
            total_volume: 0,
        };
        mxe.from_arcis(stats)
    }
}
