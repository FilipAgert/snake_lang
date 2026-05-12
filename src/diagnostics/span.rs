#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: usize, // start (inclusive)
    pub end: usize,   // (exclusive)
}
impl Span {
    pub fn merge(s1: &Self, s2: &Self) -> Self {
        let start = s1.start.min(s2.start);
        let end = s1.end.max(s2.end);
        Self { start, end }
    }
    pub const fn min_info() -> Self {
        Self {
            start: usize::MAX,
            end: usize::MIN,
        }
    }
}
