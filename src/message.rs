#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Start,
    Load(crate::percent::Percent),
}
