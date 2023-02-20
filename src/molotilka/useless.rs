use rand::{thread_rng, Rng};

pub fn work(num: usize) {
    let mut pointless = [0u64; 10000];
    if thread_rng().try_fill(&mut pointless[..]).is_err() {
        panic!("WORKER {num}: Failed to fill array with random numbers.");
    }
    pointless.sort();
}
