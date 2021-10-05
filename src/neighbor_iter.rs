pub struct NeighborIter
{
    width: u8,
    height: u8,
    row: u8,
    col: u8,
    i: u8
}

impl NeighborIter
{
    const DELTAS: [(i16, i16); 8] = [
        (-1, -1), (-1, 0), (-1, 1),
        ( 0, -1),          ( 0, 1),
        ( 1, -1), ( 1, 0), ( 1, 1)
    ];

    fn new(width: u8, height: u8, row: u8, col: u8) -> Self
    {
        Self{width, height, row, col, i: 0}
    }
}

impl Iterator for NeighborIter
{
    type Item = (u8, u8);

    fn next(&mut self) -> Option<Self::Item>
    {
        while self.i < 8 {
            let (dr, dc) = Self::DELTAS[self.i as usize];
            self.i += 1;

            let row = dr + i16::from(self.row);
            if row < 0 || row >= i16::from(self.height) {
                continue;
            }

            let col = dc + i16::from(self.col);
            if col < 0 || col >= i16::from(self.width) {
                continue;
            }

            return Some((row as u8, col as u8));
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        (3, Some(8))
    }
}

pub trait NeighborIterable
{
    fn width(&self) -> u8;
    fn height(&self) -> u8;

    fn neighbors_of(&self, row: u8, col: u8) -> NeighborIter
    {
        NeighborIter::new(self.width(), self.height(), row, col)
    }
}
