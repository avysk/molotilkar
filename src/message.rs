#[derive(PartialEq)]
pub enum Message {
    Start,
    Load(crate::percent::Percent),
}
