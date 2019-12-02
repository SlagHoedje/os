pub fn align_down(x: usize, log2: usize) -> usize {
    x & (!0 << log2)
}

pub fn align_up(x: usize, log2: usize) -> usize {
    (x + (1 << log2) - 1) & (!0 << log2)
}