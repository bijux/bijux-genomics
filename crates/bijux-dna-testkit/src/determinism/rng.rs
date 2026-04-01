use rand::rngs::StdRng;
use rand::SeedableRng;

#[must_use]
pub fn fixed_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}
