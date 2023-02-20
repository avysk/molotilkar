mod useless;

use std::sync::mpsc::{Receiver, TryRecvError};

use log::debug;

use crate::{message::Message, percent::Percent};

pub fn start(num: usize, rx: Receiver<Message>) {
    debug!("WORKER {}: Created.", num);
    let res = rx.recv();
    let msg = res.expect("CHannel was closed before the thread managed to start.");
    if !(msg == Message::Start) {
        panic!("Programming error. Received non-Start message.");
    }
    debug!("WORKER {}: Started.", num);

    debug!("WORKER {}: Waiting for load message", num);
    let got = rx.recv();
    debug!("WORKER {0}: Got new message {1:?}.", num, got);
    if got.is_ok() {
        let mut load = got.unwrap();
        loop {
            match load {
                Message::Load(p) => {
                    if p == Percent(0) {
                        debug!("WORKER {}: Asked to stop.", num);
                        debug!("WORKER {}: Waiting for load message", num);
                        // The worker should be stopped. Block and wait for the next message.
                        match rx.recv() {
                            Err(_) => {
                                // "The recv operation can only fail if the sending half of a
                                // channel (or sync_channel) is disconnected, implying that no
                                // further messages will ever be received."
                                break;
                            }
                            Ok(l) => {
                                load = l;
                                continue;
                            }
                        }
                    }
                    let start_t = std::time::Instant::now();
                    useless::work(num);
                    let elapsed = start_t.elapsed().as_secs_f32();
                    let decimal_p = p.decimal();
                    let sleeping_time = elapsed * (1.0 - decimal_p) / decimal_p;
                    // We assume that useless work takes 100% of CPU and sleep takes 0%.
                    std::thread::sleep(std::time::Duration::from_secs_f32(sleeping_time));
                }
                _ => {
                    panic!("WORKER {}: Programming error. The started process recived non-Load message", num)
                }
            }
            match rx.try_recv() {
                Err(TryRecvError::Empty) => {
                    // No new message
                    continue;
                }
                Err(TryRecvError::Disconnected) => {
                    // Channel is closed
                    break;
                }
                Ok(l) => {
                    load = l;
                    debug!("WORKER {0}: Got new message Ok({1:?}).", num, l);
                }
            }
        }
        debug!("WORKER {}: Channel was closed. Exiting.", num);
    }
}
