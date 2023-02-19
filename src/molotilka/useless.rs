use rand::{thread_rng, Rng};

pub fn work(num: u64) {
    let mut pointless = [0u64; 2500];
    if thread_rng().try_fill(&mut pointless[..]).is_err() {
        panic!("WORKER {num}: Failed to fill array with random numbers.");
    }
    pointless.sort();
}
