pub trait GridCounters<T> {
    fn notify_change(&mut self, from: &T, to: &T);
}

pub struct Grid<T, I, C> {
    pub counters: C,
    data: Vec<T>,
    w: I,
    h: I
}

pub struct RowsIter<'a, T, I, C> {
    grid: &'a Grid<T, I, C>,
    idx: usize
}

impl<'a, T, I: Into<usize> + Copy, C> Iterator for RowsIter<'a, T, I, C> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.idx >= self.grid.data.len() {
            return None;
        }

        let prev = self.idx;
        self.idx += self.grid.w.into();

        Some(&self.grid.data[prev..self.idx])
    }

    fn size_hint(&self) -> (usize, Option<usize>)
    {
        let len = self.grid.data.len();
        (len,Some(len))
    }
}

impl<T, M, I, C> Grid<T, I, C> where
    T: Copy,
    M: Into<usize>,
    I: Copy + Into<usize> + std::ops::Mul<Output = M>,
    C: GridCounters<T>
{
    pub fn new(width: I, height: I, counters: C, default_value: T) -> Self
    {
        Self {
            data: vec![default_value; width.into() * height.into()],
            counters, w: width, h: height
        }
    }

    pub fn from_vec(width: I, height: I, counters: C, data: Vec<T>) -> Option<Self> {
        if width.into() * height.into() != data.len() {
            return None;
        }

        Some(Self {
            data, counters, w: width, h: height
        })
    }

    pub fn width(&self) -> I {
        self.w
    }

    pub fn height(&self) -> I {
        self.h
    }

    pub fn rows(&self) -> RowsIter<T, I, C> {
        RowsIter {
            grid: self,
            idx: 0
        }
    }

    fn to_idx(&self, row: I, col: I) -> usize
    {
        assert!(row.into() < self.h.into());
        assert!(col.into() < self.w.into());

        row.into() * self.w.into() + col.into()
    }

    pub fn get(&self, row: I, col: I) -> &T
    {
        // Use unsafe here because values are already checked by to_idx()
        unsafe {
            self.data.get_unchecked(self.to_idx(row, col))
        }
    }

    pub fn set(&mut self, row: I, col: I, val: T)
    {
        let idx = self.to_idx(row, col);
        // Use unsafe here because values are already checked by to_idx()
        let ptr = unsafe {
            self.data.get_unchecked_mut(idx)
        };

        self.counters.notify_change(ptr, &val);

        *ptr = val;
    }
}
