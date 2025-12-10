use chess::BitBoard;

/// An iterator over the indices of the 1s in a BitBoard.
pub struct BitBoardIter {
    bb: BitBoard,
}

impl BitBoardIter {
    pub fn new(bb: BitBoard) -> Self {
        Self { bb }
    }
}

impl Iterator for BitBoardIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.bb.0 == 0 {
            None
        } else {
            let index = self.bb.0.trailing_zeros() as usize;
            self.bb.0 &= self.bb.0 - 1;
            Some(index)
        }
    }
}
