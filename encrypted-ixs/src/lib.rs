use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    pub struct BindingInput {
        rns_dest_hash: u128,
        region_code: u32,
    }

    #[instruction]
    pub fn beacon_bind(input: Enc<Shared, BindingInput>) -> Enc<Shared, u128> {
        let binding = input.to_arcis();
        let rns_bytes = binding.rns_dest_hash.to_le_bytes();
        let region_bytes = binding.region_code.to_le_bytes();

        let mut msg = [0u8; 20];
        for i in 0..16 {
            msg[i] = rns_bytes[i];
        }
        for i in 0..4 {
            msg[16 + i] = region_bytes[i];
        }

        let hash = SHA3_256::new().digest(&msg);
        let mut commitment: u128 = 0;
        let mut shift: u128 = 1;
        for i in 0..16 {
            commitment = commitment + (hash[i] as u128) * shift;
            shift = shift * 256;
        }

        input.owner.from_arcis(commitment)
    }

    pub struct RelayCount {
        count: u64,
    }

    #[instruction]
    pub fn relay_increment(current: Enc<Shared, RelayCount>) -> Enc<Shared, RelayCount> {
        let current_count = current.to_arcis();
        let next = RelayCount {
            count: current_count.count + 1,
        };
        current.owner.from_arcis(next)
    }
}
