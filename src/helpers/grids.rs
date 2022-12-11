use std::{fmt::{Debug, Write}, ops::{Deref, DerefMut, Add, Mul}};

pub trait ToGridChar {
    fn to_grid_char(&self) -> char;
}

impl ToGridChar for bool {
    fn to_grid_char(&self) -> char {
        if *self {'#'} else {'.'}
    }
}

impl ToGridChar for u8 {
    fn to_grid_char(&self) -> char {
        if (0..=9).contains(self) {
            (self + b'0') as char
        } else {
            '#'
        }
    }
}

impl ToGridChar for u16 {
    fn to_grid_char(&self) -> char {
        if (0..=9).contains(self) {
            (*self as u8 + b'0') as char
        } else {
            '#'
        }
    }
}

impl ToGridChar for u32 {
    fn to_grid_char(&self) -> char {
        if (0..=9).contains(self) {
            (*self as u8 + b'0') as char
        } else {
            '#'
        }
    }
}

pub struct SquareGrid<T, const S: usize>([[T; S]; S]);

impl<T, const S: usize> Deref for SquareGrid<T, S> {
    type Target = [[T; S]; S];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const S: usize> DerefMut for SquareGrid<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const S: usize> SquareGrid<T, S> {
    pub fn from_array(array: [[T; S]; S]) -> Self {
        SquareGrid(array)
    }
    
    pub fn new_with(val: T) -> Self where T: Copy {
        Self::from_array([[val; S]; S])
    }
}

impl<T: Add + Copy, const S: usize> Add for SquareGrid<T, S> where <T as Add>::Output: Into<T> {
    type Output = SquareGrid<T, S>;
    
    fn add(mut self, rhs: Self) -> Self::Output {
        for r in 0..S {
            for c in 0..S {
                self.0[r][c] = (self.0[r][c] + rhs.0[r][c]).into()
            }
        }
        self
    }
}

impl<T: Mul + Copy, const S: usize> Mul for SquareGrid<T, S> where <T as Mul>::Output: Into<T> {
    type Output = SquareGrid<T, S>;
    
    fn mul(mut self, rhs: Self) -> Self::Output {
        for r in 0..S {
            for c in 0..S {
                self.0[r][c] = (self.0[r][c] * rhs.0[r][c]).into()
            }
        }
        self
    }
}

impl<T: ToGridChar, const S: usize> Debug for SquareGrid<T, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..S {
            for c in 0..S {
                f.write_char(self.0[r][c].to_grid_char())?;
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct LineOrientation {
    col_step: isize,
    row_step: isize,
}

const LINE_ORIENTATION_RIGHT_TO_LEFT: LineOrientation = LineOrientation {
    col_step: -1,
    row_step: 0,
};
const LINE_ORIENTATION_LEFT_TO_RIGHT: LineOrientation = LineOrientation {
    col_step: 1,
    row_step: 0,
};
const LINE_ORIENTATION_TOP_TO_BOTTOM: LineOrientation = LineOrientation {
    col_step: 0,
    row_step: 1,
};
const LINE_ORIENTATION_BOTTOM_TO_TOP: LineOrientation = LineOrientation {
    col_step: 0,
    row_step: -1,
};

pub struct LineIterator<const S: usize> {
    orientation: LineOrientation,
    row: isize,
    col: isize,
}

impl<const S: usize> LineIterator<S> {
    pub fn get_line_from_top<'a>(offset: usize) -> LineIterator<S> {
        LineIterator {
            orientation: LINE_ORIENTATION_TOP_TO_BOTTOM,
            col: isize::try_from(offset).expect("Offset is too large"),
            row: 0,
        }
    }

    pub fn get_line_from_left(offset: usize) -> LineIterator<S> {
        LineIterator {
            orientation: LINE_ORIENTATION_LEFT_TO_RIGHT,
            col: 0,
            row: isize::try_from(offset).expect("Offset is too large"),
        }
    }

    pub fn get_line_from_bottom(offset: usize) -> LineIterator<S> {
        LineIterator {
            orientation: LINE_ORIENTATION_BOTTOM_TO_TOP,
            col: isize::try_from(offset).expect("Offset is too large"),
            row: isize::try_from(S).expect("Grid size is too large") - 1,
        }
    }

    pub fn get_line_from_right(offset: usize) -> LineIterator<S> {
        LineIterator {
            orientation: LINE_ORIENTATION_RIGHT_TO_LEFT,
            col: isize::try_from(S).expect("Grid size is too large") - 1,
            row: isize::try_from(offset).expect("Offset is too large"),
        }
    }
}

impl<'a, const S: usize> Iterator for LineIterator<S> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let max_size = isize::try_from(S).expect("Grid size is too large");
        if !(0..max_size).contains(&self.col) || !(0..max_size).contains(&self.row) {
            return None;
        }
        let ret = (
            usize::try_from(self.row).expect("Row size is too large"),
            usize::try_from(self.col).expect("Column size is too large"),
        );
        self.col = self.col + self.orientation.col_step;
        self.row = self.row + self.orientation.row_step;
        Some(ret)
    }
}
