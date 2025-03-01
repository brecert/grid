/*!
# Two Dimensional Grid
Continuos growable 2D data structure.
The purpose of this crate is to provide an universal data structure that is faster,
uses less memory, and is easier to use than a naive `Vec<Vec<T>>` solution.

Similar to *C-like* arrays `grid` uses a flat 1D `Vec<T>` data structure to have a continuos
memory data layout. See also [this](https://stackoverflow.com/questions/17259877/1d-or-2d-array-whats-faster)
explanation of why you should probably use a one-dimensional array approach.

Note that this crate uses a [*row-major*](https://eli.thegreenplace.net/2015/memory-layout-of-multi-dimensional-arrays) memory layout.
Therefore, `grid.push_row()` is way faster then the `grid.push_col()` operation.

This crate will always provide a 2D data structure. If you need three or more dimensions take a look at the
[ndarray](https://docs.rs/ndarray/0.13.0/ndarray/) library. The `grid` create is a container for all kind of data.
If you need to perform matrix operations, you are better of with a linear algebra lib, such as
[cgmath](https://docs.rs/cgmath/0.17.0/cgmath/) or [nalgebra](https://docs.rs/nalgebra/0.21.0/nalgebra/).
No other dependencies except for the std lib are used.
Most of the functions `std::Vec<T>` offer are also implemented in `grid` and slightly modified for a 2D data object.
# Examples
```
use grid::*;

let mut grid = grid![
    [1,2,3]
    [4,5,6]
];

assert_eq!(grid, Grid::from_vec(vec![1,2,3,4,5,6],3));
assert_eq!(grid.get(0,2), Some(&3));
assert_eq!(grid[1][1], 5);
assert_eq!(grid.size(), (2,3));

grid.push_row(vec![7,8,9]);
assert_eq!(grid, grid![[1,2,3][4,5,6][7,8,9]])
 ```
*/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate no_std_compat as std;
#[cfg(not(feature = "std"))]
use std::prelude::v1::*;

use std::cmp::Eq;
use std::fmt;
use std::iter::StepBy;
use std::ops::Index;
use std::ops::IndexMut;
use std::slice::Iter;
use std::slice::IterMut;

#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

/// Init a grid with values.
///
/// Each array within `[]` represents a row starting from top to button.
///
/// # Examples
///
/// In this example a grid of numbers from 1 to 9 is created:
///
///  
/// ```
/// use grid::grid;
/// let grid = grid![[1, 2, 3]
/// [4, 5, 6]
/// [7, 8, 9]];
/// assert_eq!(grid.size(), (3, 3))
/// ```
///
/// # Examples
///
/// Not that each row must be of the same length. The following example will not compile:  
///  
/// ``` ignore
/// # use grid::grid;
/// let grid = grid![[1, 2, 3]
/// [4, 5] // This does not work!
/// [7, 8, 9]];
/// ```
#[macro_export]
macro_rules! grid {
    () => {
        $crate::Grid::from_vec(vec![], 0)
    };
    ( [$( $x:expr ),* ]) => { {
        let vec = vec![$($x),*];
        let len  = vec.len();
        $crate::Grid::from_vec(vec, len)
    } };
    ( [$( $x0:expr ),*] $([$( $x:expr ),*])* ) => {
        {
            let mut _assert_width0 = [(); $crate::count!($($x0)*)];
            let cols = $crate::count!($($x0)*);
            let rows = 1usize;

            $(
                let _assert_width = [(); $crate::count!($($x)*)];
                _assert_width0 = _assert_width;
                let rows = rows + 1usize;
            )*

            let mut vec = Vec::with_capacity(rows * cols);

            $( vec.push($x0); )*
            $( $( vec.push($x); )* )*

            $crate::Grid::from_vec(vec, cols)
        }
    };
}

#[doc(hidden)]
pub fn index_at(cols: usize, row: usize, col: usize) -> usize {
    cols * row + col
}

/// Stores elements of a certain type in a 2D grid structure.
///
/// Uses a rust [`Vec<T>`] type to reference the grid data on the heap.
/// Also the number of rows and columns are stored in the grid data structure.
///
/// The grid data is stored in a row-major memory layout.
///
/// # Examples
/// ```
/// use grid::Grid;
///
/// let mut grid: Grid<u8> = Grid::new(2, 3);
///
/// grid.insert_row(1, vec![1, 2, 3]);
///
/// assert_eq!(grid.pop_col(), Some(vec![0, 3, 0]));
/// assert_eq!(format!("{:?}", grid), "[[0, 0][1, 2][0, 0]]")
/// ```
///
/// The [`grid!`] macro is provided to make initialization more convenient:
///
/// ```
/// use grid::grid;
///
/// let mut grid = grid![[1, 2, 3][4, 5, 6]];
/// grid.push_row(vec![7, 8, 9]);
/// assert_eq!(format!("{:?}", grid), "[[1, 2, 3][4, 5, 6][7, 8, 9]]")
/// ```
#[derive(Hash)]
pub struct Grid<T> {
    #[doc(hidden)]
    pub data: Vec<T>,
    #[doc(hidden)]
    pub cols: usize,
    #[doc(hidden)]
    pub rows: usize,
}

impl<T> Grid<T> {
    /// Init a grid of size rows x columns with default values of the given type.
    /// For example this will generate a 2x3 grid of zeros:
    /// ```
    /// # use grid::Grid;
    /// let grid: Grid<u8> = Grid::new(2,2);
    /// assert_eq!(grid[0][0], 0);
    /// ```
    pub fn new(rows: usize, cols: usize) -> Grid<T>
    where
        T: Default,
    {
        if rows < 1 || cols < 1 {
            panic!("Grid size of rows and columns must be greater than zero.");
        }

        let mut data = Vec::with_capacity(rows * cols);
        data.resize_with(rows * cols, Default::default);

        Grid { data, cols, rows }
    }

    /// Init a grid of size rows x columns with the given data element.
    /// For example this will generate a 2x3 grid of `'a'`:
    /// ```
    /// # use grid::Grid;
    /// let grid = Grid::init(2, 3, 'a');
    /// assert_eq!(format!("{:?}", grid), "[['a', 'a', 'a']['a', 'a', 'a']]")
    /// ```
    pub fn init(rows: usize, cols: usize, data: T) -> Grid<T>
    where
        T: Clone,
    {
        if rows < 1 || cols < 1 {
            panic!("Grid size of rows and columns must be greater than zero.");
        }

        Grid {
            data: vec![data; rows * cols],
            cols,
            rows,
        }
    }

    /// Returns a grid from a vector with a given column length.
    /// The length of `vec` must be a multiple of `cols`.
    ///
    /// For example:
    ///
    /// ```
    /// # use grid::Grid;
    /// let grid = Grid::from_vec(vec![1,2,3,4,5,6], 3);
    /// assert_eq!(grid.size(), (2, 3));
    /// ```
    ///
    /// will create a grid with the following layout:
    /// \[1,2,3\]
    /// \[4,5,6\]
    ///
    /// This example will fail, because `vec.len()` is not a multiple of `cols`:
    ///
    /// ``` should_panic
    /// # use grid::Grid;
    /// Grid::from_vec(vec![1,2,3,4,5], 3);
    /// ```
    pub fn from_vec(vec: Vec<T>, cols: usize) -> Grid<T> {
        let rows = vec.len();
        if rows == 0 {
            if cols == 0 {
                Grid {
                    data: vec![],
                    rows: 0,
                    cols: 0,
                }
            } else {
                panic!("Vector length is zero, but cols is {:?}", cols);
            }
        } else if rows % cols != 0 {
            panic!("Vector length must be a multiple of cols.");
        } else {
            Grid {
                data: vec,
                rows: rows / cols,
                cols,
            }
        }
    }

    /// Returns a reference to an element, without performing bound checks.
    /// Generally not recommended, use with caution!
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked(&self, row: usize, col: usize) -> &T {
        self.data.get_unchecked(row * self.cols + col)
    }

    /// Returns a mutable reference to an element, without performing bound checks.
    /// Generally not recommended, use with caution!
    ///
    /// # Safety
    ///
    /// Calling this method with an out-of-bounds index is undefined behavior even if the resulting reference is not used.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, row: usize, col: usize) -> &mut T {
        let cols = self.cols;
        self.data.get_unchecked_mut(row * cols + col)
    }

    /// Access a certain element in the grid.
    /// Returns None if an element beyond the grid bounds is tried to be accessed.
    pub fn get(&self, row: usize, col: usize) -> Option<&T> {
        if row < self.rows && col < self.cols {
            unsafe { Some(self.get_unchecked(row, col)) }
        } else {
            None
        }
    }

    /// Mutable access to a certain element in the grid.
    /// Returns None if an element beyond the grid bounds is tried to be accessed.
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut T> {
        if row < self.rows && col < self.cols {
            unsafe { Some(self.get_unchecked_mut(row, col)) }
        } else {
            None
        }
    }

    /// Returns the size of the gird as a two element tuple.
    /// First element are the number of rows and the second the columns.
    pub fn size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Returns the number of rows of the grid.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the number of columns of the grid.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Returns true if the grid contains no elements.
    /// For example:
    /// ```
    /// # use grid::*;
    /// let grid : Grid<u8> = grid![];
    /// assert!(grid.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.cols == 0 && self.rows == 0
    }

    /// Clears the grid.
    pub fn clear(&mut self) {
        self.rows = 0;
        self.cols = 0;
        self.data.clear();
    }

    /// Returns an iterator over the whole grid, starting from the first row and column.
    /// ```
    /// # use grid::*;
    /// let grid: Grid<u8> = grid![[1,2][3,4]];
    /// let mut iter = grid.iter();
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), Some(&4));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    /// Returns an mutable iterator over the whole grid that allows modifying each value.
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1,2][3,4]];
    /// let mut iter = grid.iter_mut();
    /// let next = iter.next();
    /// assert_eq!(next, Some(&mut 1));
    /// *next.unwrap() = 10;
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }

    // TODO: name this properly, maybe have it be a trait?
    /// Returns an mutable iterator over the whole grid with index positions.
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1,2][3,4]];
    /// let mut iter = grid.iter_with_index();
    /// assert_eq!(iter.next(), Some(((0, 0), &1)));
    /// assert_eq!(iter.next(), Some(((0, 1), &2)));
    /// assert_eq!(iter.next(), Some(((1, 0), &3)));
    /// assert_eq!(iter.next(), Some(((1, 1), &4)));
    /// ```
    pub fn iter_with_index(&self) -> impl DoubleEndedIterator<Item = ((usize, usize), &T)> {
        (0..self.rows)
            .flat_map(move |row| (0..self.cols).map(move |col| ((row, col), &self[row][col])))
    }

    /// Returns an iterator over a column.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let mut col_iter = grid.iter_col(1);
    /// assert_eq!(col_iter.next(), Some(&2));
    /// assert_eq!(col_iter.next(), Some(&4));
    /// assert_eq!(col_iter.next(), None);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the col index is out of bounds.
    pub fn iter_col(&self, col: usize) -> StepBy<Iter<T>> {
        if col < self.cols {
            return self.data[col..].iter().step_by(self.cols);
        } else {
            panic!(
                "out of bounds. Column must be less than {:?}, but is {:?}.",
                self.cols, col
            )
        }
    }

    /// Returns a mutable iterator over a column.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let mut col_iter = grid.iter_col_mut(1);
    /// let next = col_iter.next();
    /// assert_eq!(next, Some(&mut 2));
    /// *next.unwrap() = 10;
    /// assert_eq!(grid[0][1], 10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the col index is out of bounds.
    pub fn iter_col_mut(&mut self, col: usize) -> StepBy<IterMut<T>> {
        let cols = self.cols;
        if col < cols {
            return self.data[col..].iter_mut().step_by(cols);
        } else {
            panic!(
                "out of bounds. Column must be less than {:?}, but is {:?}.",
                self.cols, col
            )
        }
    }

    /// Returns an iterator over a row.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let mut col_iter = grid.iter_row(1);
    /// assert_eq!(col_iter.next(), Some(&3));
    /// assert_eq!(col_iter.next(), Some(&4));
    /// assert_eq!(col_iter.next(), Some(&5));
    /// assert_eq!(col_iter.next(), None);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the row index is out of bounds.
    pub fn iter_row(&self, row: usize) -> Iter<T> {
        if row < self.rows {
            let start = row * self.cols;
            self.data[start..(start + self.cols)].iter()
        } else {
            panic!(
                "out of bounds. Row must be less than {:?}, but is {:?}.",
                self.rows, row
            )
        }
    }

    /// Returns a mutable iterator over a row.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let mut col_iter = grid.iter_row_mut(1);
    /// let next = col_iter.next();
    /// *next.unwrap() = 10;
    /// assert_eq!(grid[1][0], 10);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the row index is out of bounds.
    pub fn iter_row_mut(&mut self, row: usize) -> IterMut<T> {
        if row < self.rows {
            let cols = self.cols;
            let start = row * cols;
            self.data[start..(start + cols)].iter_mut()
        } else {
            panic!(
                "out of bounds. Row must be less than {:?}, but is {:?}.",
                self.rows, row
            )
        }
    }

    /// Add a new row to the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let row = vec![6,7,8];
    /// grid.push_row(row);
    /// assert_eq!(grid.rows(), 3);
    /// assert_eq!(grid[2][0], 6);
    /// assert_eq!(grid[2][1], 7);
    /// assert_eq!(grid[2][2], 8);
    /// ```
    ///
    /// Can also be used to init an empty grid:
    ///
    /// ```
    /// use grid::*;
    /// let mut grid: Grid<u8> = grid![];
    /// let row = vec![1,2,3];
    /// grid.push_row(row);
    /// assert_eq!(grid.size(), (1, 3));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the grid is not empty and `row.len() != grid.cols()`.
    pub fn push_row(&mut self, row: Vec<T>) {
        let input_row_len = row.len();
        if self.rows > 0 && input_row_len != self.cols {
            panic!(
                "pushed row does not match. Length must be {:?}, but was {:?}.",
                self.cols, input_row_len
            )
        }
        self.data.extend(row);
        self.rows += 1;
        self.cols = input_row_len;
    }

    /// Add a new column to the grid.
    ///
    /// *Important:*
    /// Please note that `Grid` uses a Row-Major memory layout. Therefore, the `push_col()`
    /// operation requires quite a lot of memory shifting and will be significantly slower compared
    /// to a `push_row()` operation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1, 2, 3][3, 4, 5]];
    /// let col = vec![4,6];
    /// grid.push_col(col);
    /// assert_eq!(grid.cols(), 4);
    /// assert_eq!(grid[0][3], 4);
    /// assert_eq!(grid[1][3], 6);
    /// ```
    ///
    /// Can also be used to init an empty grid:
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![];
    /// let col = vec![1,2,3];
    /// grid.push_col(col);
    /// assert_eq!(grid.size(), (3, 1));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the grid is not empty and `col.len() != grid.rows()`.
    pub fn push_col(&mut self, col: Vec<T>) {
        let input_col_len = col.len();
        if self.cols > 0 && input_col_len != self.rows {
            panic!(
                "pushed column does not match. Length must be {:?}, but was {:?}.",
                self.rows, input_col_len
            )
        }
        self.data.reserve(col.len());
        for (idx, d) in col.into_iter().enumerate() {
            let vec_idx = (idx + 1) * self.cols + idx;
            self.data.insert(vec_idx, d);
        }
        self.cols += 1;
        self.rows = input_col_len;
    }

    /// Insert a new row at the index and shifts all rows after down.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// grid.insert_row(1, vec![7,8,9]);
    /// assert_eq!(grid[0], [1,2,3]);
    /// assert_eq!(grid[1], [7,8,9]);
    /// assert_eq!(grid[2], [4,5,6]);
    /// assert_eq!(grid.size(), (3,3))
    /// ```
    pub fn insert_row(&mut self, index: usize, row: Vec<T>) {
        if row.len() != self.cols {
            panic!(
                "Inserted row must be of length {}, but was {}.",
                self.cols,
                row.len()
            );
        }
        if index > self.rows {
            panic!(
                "Out of range. Index was {}, but must be less or equal to {}.",
                index, self.cols
            );
        }
        self.rows += 1;
        let data_idx = index * self.cols;
        self.data.splice(data_idx..data_idx, row);
    }

    /// Insert a new column at the index.
    ///
    /// Important! Insertion of columns is a lot slower than the lines insertion.
    /// This is because of the memory layout of the grid data structure.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// grid.insert_col(1, vec![9,9]);
    /// assert_eq!(grid[0], [1,9,2,3]);
    /// assert_eq!(grid[1], [4,9,5,6]);
    /// assert_eq!(grid.size(), (2,4))
    /// ```
    pub fn insert_col(&mut self, index: usize, col: Vec<T>) {
        if col.len() != self.rows {
            panic!(
                "Inserted col must be of length {}, but was {}.",
                self.rows,
                col.len()
            );
        }
        if index > self.cols {
            panic!(
                "Out of range. Index was {}, but must be less or equal to {}.",
                index, self.rows
            );
        }

        self.data.reserve(self.rows);

        let cols = self.cols + 1;

        let indices = (0..self.rows).map(|row_index| index_at(cols, row_index, index));

        for (elem, idx) in col.into_iter().zip(indices) {
            self.data.insert(idx, elem)
        }

        self.cols += 1;
    }

    /// Replace an existing row at the index.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// grid.replace_row(0, vec![7,8,9]);
    /// assert_eq!(grid[0], [7,8,9]);
    /// assert_eq!(grid[1], [4,5,6]);
    /// assert_eq!(grid.size(), (2,3))
    /// ```
    pub fn replace_row(&mut self, index: usize, row: Vec<T>) {
        if row.len() != self.cols {
            panic!(
                "Inserted row must be of length {}, but was {}.",
                self.cols,
                row.len()
            );
        }
        if index > self.rows {
            panic!(
                "Out of range. Index was {}, but must be less or equal to {}.",
                index, self.cols
            );
        }
        let data_idx = index * self.cols;
        self.data.splice(data_idx..data_idx + self.cols, row);
    }

    /// Replace a new column at the index.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// grid.replace_col(1, vec![9,9]);
    /// assert_eq!(grid[0], [1,9,3]);
    /// assert_eq!(grid[1], [4,9,6]);
    /// assert_eq!(grid.size(), (2,3))
    /// ```
    pub fn replace_col(&mut self, index: usize, col: Vec<T>) {
        if col.len() != self.rows {
            panic!(
                "Inserted col must be of length {}, but was {}.",
                self.rows,
                col.len()
            );
        }
        if index > self.cols {
            panic!(
                "Out of range. Index was {}, but must be less or equal to {}.",
                index, self.rows
            );
        }

        // todo: use a `for` loop here to match the style of the rest of the code.

        self.data
            .iter_mut()
            .skip(index)
            .step_by(self.cols)
            .zip(col)
            .for_each(|(old, new)| *old = new);
    }

    /// Removes the last row from a grid and returns it, or None if it is empty.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// assert_eq![grid.pop_row(), Some(vec![4,5,6])];
    /// assert_eq![grid.pop_row(), Some(vec![1,2,3])];
    /// assert_eq![grid.pop_row(), None];
    /// ```
    pub fn pop_row(&mut self) -> Option<Vec<T>> {
        if self.rows > 0 {
            let row = self.data.split_off((self.rows - 1) * self.cols);
            self.rows -= 1;
            if self.rows == 0 {
                self.cols = 0;
            }
            Some(row)
        } else {
            None
        }
    }

    /// Removes the last column from a grid and returns it, or None if it is empty.
    ///
    /// Note that this operation is much slower than the `pop_row()` because the memory layout
    /// of `Grid` is row-major and removing a column requires a lot of move operations.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let mut grid = grid![[1,2,3][4,5,6]];
    /// assert_eq![grid.pop_col(), Some(vec![3,6])];
    /// assert_eq![grid.pop_col(), Some(vec![2,5])];
    /// assert_eq![grid.pop_col(), Some(vec![1,4])];
    /// assert_eq![grid.pop_col(), None];
    /// ```
    pub fn pop_col(&mut self) -> Option<Vec<T>> {
        if self.cols > 0 {
            let mut col = Vec::with_capacity(self.rows);
            for i in 0..self.rows {
                let idx = i * self.cols + self.cols - 1 - i;
                col.push(self.data.remove(idx));
            }
            self.cols -= 1;
            if self.cols == 0 {
                self.rows = 0;
            }
            Some(col)
        } else {
            None
        }
    }

    /// Swap the values of two elements in the grid.
    ///
    /// # Arguments
    ///
    /// * a - The index of the first element
    /// * b - The index of the second element
    ///
    /// # Panics
    ///
    /// Panics if `a` or `b` are out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1, 2, 3][4, 5, 6]];
    /// grid.swap((0, 2), (1, 1));
    /// assert_eq![format!["{:?}", grid], "[[1, 2, 5][4, 3, 6]]"]
    /// ```
    pub fn swap(&mut self, a: (usize, usize), b: (usize, usize)) {
        let a = index_at(self.cols, a.0, a.1);
        let b = index_at(self.cols, b.0, b.1);
        self.data.swap(a, b);
    }

    /// Get adjacent elements in the grid a specicific index.
    ///
    /// Adjacent results are currently in the pattern of `[top, right, bottom, left]`
    ///
    /// # Returns
    ///
    /// Returns [`None`] when the element index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use grid::*;
    /// let grid: Grid<u8> = grid![[1, 2, 3][4, 5, 6][7, 8, 9]];
    /// assert_eq!(grid.adjacent(1, 1), [Some(&2), Some(&6), Some(&8), Some(&4)]);
    /// assert_eq!(grid.adjacent(0, 0), [None, Some(&2), Some(&4), None]);
    /// ```
    pub fn adjacent(&self, row: usize, col: usize) -> [Option<&T>; 4] {
        [
            row.checked_sub(1).and_then(|row| self.get(row, col)),
            col.checked_add(1).and_then(|col| self.get(row, col)),
            row.checked_add(1).and_then(|row| self.get(row, col)),
            col.checked_sub(1).and_then(|col| self.get(row, col)),
        ]
    }

    /// Transpose the grid so that columns become rows in a new grid.
    ///
    /// ```
    /// # use grid::*;
    /// let mut grid: Grid<u8> = grid![[1,2,3][4,5,6]];
    /// assert_eq!(format!("{:?}", grid.transpose()), "[[1, 4][2, 5][3, 6]]");
    /// ```
    pub fn transpose(&self) -> Grid<T>
    where
        T: Clone,
    {
        let mut data = Vec::with_capacity(self.data.len());
        for c in 0..self.cols {
            for r in 0..self.rows {
                data.push(self[r][c].clone());
            }
        }
        Grid {
            data,
            cols: self.rows,
            rows: self.cols,
        }
    }

    /// Returns a reference to the internal data structure of the grid.
    ///
    /// Grid uses a row major layout.
    /// All rows are placed right after each other in the vector data structure.
    ///
    /// # Examples
    /// ```
    /// # use grid::*;
    /// let grid = grid![[1,2,3][4,5,6]];
    /// let flat = grid.flatten();
    /// assert_eq!(flat, &vec![1,2,3,4,5,6]);
    /// ```
    pub fn flatten(&self) -> &Vec<T> {
        &self.data
    }

    /// Converts self into a vector without clones or allocation.
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }
}

impl<T: Clone> Clone for Grid<T> {
    fn clone(&self) -> Self {
        Grid {
            rows: self.rows,
            cols: self.cols,
            data: self.data.clone(),
        }
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = [T];

    fn index(&self, idx: usize) -> &Self::Output {
        if idx < self.rows {
            let start_idx = idx * self.cols;
            &self.data[start_idx..start_idx + self.cols]
        } else {
            panic!(
                "index {:?} out of bounds. Grid has {:?} rows.",
                self.rows, idx
            );
        }
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.data[(idx * self.cols)..]
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (col, row): (usize, usize)) -> &Self::Output {
        &self[col][row]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (col, row): (usize, usize)) -> &mut Self::Output {
        &mut self[col][row]
    }
}

impl<T: fmt::Debug> fmt::Debug for Grid<T> {
    #[allow(unused_must_use)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[");
        if self.cols > 0 {
            for (i, _) in self.data.iter().enumerate().step_by(self.cols) {
                write!(f, "{:?}", &self.data[i..(i + self.cols)]);
            }
        }
        write!(f, "]")
    }
}

impl<T: Eq> PartialEq for Grid<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols && self.data == other.data
    }
}

impl<T: Eq> Eq for Grid<T> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index() {
        let grid = grid![[1, 2, 3][4, 5, 6][7, 8, 9]];
        assert_eq!(grid[1], [4, 5, 6]);
        assert_eq!(grid[2][0], 7);
    }

    #[test]
    fn insert_col_at_end() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        grid.insert_col(2, vec![5, 6]);
        assert_eq!(grid[0], [1, 2, 5]);
        assert_eq!(grid[1], [3, 4, 6]);
    }

    #[test]
    #[should_panic]
    fn insert_col_out_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        grid.insert_col(3, vec![4, 5]);
    }

    #[test]
    fn insert_row_at_end() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        grid.insert_row(2, vec![5, 6]);
        assert_eq!(grid[0], [1, 2]);
        assert_eq!(grid[1], [3, 4]);
        assert_eq!(grid[2], [5, 6]);
    }

    #[test]
    #[should_panic]
    fn insert_row_out_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        grid.insert_row(3, vec![4, 5]);
    }

    #[test]
    #[should_panic]
    fn insert_row_wrong_size_of_idx() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        grid.insert_row(1, vec![4, 5, 4]);
    }

    #[test]
    fn insert_row_start() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        let new_row = [5, 6];
        grid.insert_row(1, new_row.to_vec());
        assert_eq!(grid[1], new_row);
    }

    #[test]
    fn pop_col() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_eq!(grid.pop_col(), Some(vec![2, 4]));
        assert_eq!(grid.size(), (2, 1));
        assert_eq!(grid.pop_col(), Some(vec![1, 3]));
        assert_eq!(grid.size(), (0, 0));
        assert_eq!(grid.pop_col(), None);
    }

    #[test]
    fn pop_col_empty() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![], 0);
        assert_eq!(grid.pop_row(), None);
    }

    #[test]
    fn pop_row() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_eq!(grid.pop_row(), Some(vec![3, 4]));
        assert_ne!(grid.size(), (1, 4));
        assert_eq!(grid.pop_row(), Some(vec![1, 2]));
        assert_eq!(grid.size(), (0, 0));
        assert_eq!(grid.pop_row(), None);
    }

    #[test]
    fn pop_row_empty() {
        let mut grid: Grid<u8> = Grid::from_vec(vec![], 0);
        assert_eq!(grid.pop_row(), None);
    }

    #[test]
    fn ne_full_empty() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        let g2: Grid<u8> = grid![];
        assert_ne!(g1, g2);
    }

    #[test]
    fn ne() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 5], 2);
        let g2 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        assert_ne!(g1, g2);
    }

    #[test]
    fn ne_dif_rows() {
        let g1 = Grid::from_vec(vec![1, 2, 3, 4], 2);
        let g2 = Grid::from_vec(vec![1, 2, 3, 4], 1);
        assert_ne!(g1, g2);
    }

    #[test]
    fn equal_empty() {
        let grid: Grid<char> = grid![];
        let grid2: Grid<char> = grid![];
        assert_eq!(grid, grid2);
    }
    #[test]
    fn equal() {
        let grid: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        let grid2: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        assert_eq!(grid, grid2);
    }

    #[test]
    #[should_panic]
    fn idx_out_of_col_bounds() {
        let grid: Grid<char> = grid![['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']['a', 'b', 'c', 'd']];
        let _ = grid[0][5];
    }

    #[test]
    fn push_col_small() {
        let mut grid: Grid<u8> = grid![  
                    [0, 1, 2]
                    [10, 11, 12]];
        grid.push_col(vec![3, 13]);
        assert_eq!(grid.size(), (2, 4));
        assert_eq!(
            grid.iter_row(0).copied().collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            grid.iter_row(1).copied().collect::<Vec<_>>(),
            vec![10, 11, 12, 13]
        );
    }

    #[test]
    fn push_col() {
        let mut grid: Grid<char> = grid![  
                    ['a', 'b', 'c', 'd']
                    ['a', 'b', 'c', 'd']
                    ['a', 'b', 'c', 'd']];
        grid.push_col(vec!['x', 'y', 'z']);
        assert_eq!(grid.size(), (3, 5));
        assert_eq!(
            grid.iter_row(0).copied().collect::<Vec<_>>(),
            vec!['a', 'b', 'c', 'd', 'x']
        );
        assert_eq!(
            grid.iter_row(1).copied().collect::<Vec<_>>(),
            vec!['a', 'b', 'c', 'd', 'y']
        );
        assert_eq!(
            grid.iter_row(2).copied().collect::<Vec<_>>(),
            vec!['a', 'b', 'c', 'd', 'z']
        );
    }

    #[test]
    fn push_col_single() {
        let mut grid: Grid<char> = grid![['a', 'b', 'c']];
        grid.push_col(vec!['d']);
        assert_eq!(grid.size(), (1, 4));
        assert_eq!(grid[0][3], 'd');
    }

    #[test]
    fn push_col_empty() {
        let mut grid: Grid<char> = grid![];
        grid.push_col(vec!['b', 'b', 'b', 'b']);
        assert_eq!(grid.size(), (4, 1));
        assert_eq!(grid[0][0], 'b');
    }

    #[test]
    #[should_panic]
    fn push_col_wrong_size() {
        let mut grid: Grid<char> = grid![['a','a','a']['a','a','a']];
        grid.push_col(vec!['b']);
        grid.push_col(vec!['b', 'b']);
    }

    #[test]
    fn push_row_empty() {
        let mut grid: Grid<char> = grid![];
        grid.push_row(vec!['b', 'b', 'b', 'b']);
        assert_eq!(grid.size(), (1, 4));
        assert_eq!(grid[0][0], 'b');
    }

    #[test]
    #[should_panic]
    fn push_row_wrong_size() {
        let mut grid: Grid<char> = grid![['a','a','a']['a','a','a']];
        grid.push_row(vec!['b']);
        grid.push_row(vec!['b', 'b', 'b', 'b']);
    }

    #[test]
    fn iter_row() {
        let grid: Grid<u8> = grid![[1,2,3][1,2,3]];
        let mut iter = grid.iter_row(0);
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic]
    fn iter_row_empty() {
        let grid: Grid<u8> = grid![];
        let _ = grid.iter_row(0);
    }

    #[test]
    #[should_panic]
    fn iter_row_out_of_bound() {
        let grid: Grid<u8> = grid![[1,2,3][1,2,3]];
        let _ = grid.iter_row(2);
    }

    #[test]
    #[should_panic]
    fn iter_col_out_of_bound() {
        let grid: Grid<u8> = grid![[1,2,3][1,2,3]];
        let _ = grid.iter_col(3);
    }

    #[test]
    #[should_panic]
    fn iter_col_zero() {
        let grid: Grid<u8> = grid![];
        let _ = grid.iter_col(0);
    }

    #[test]
    fn iter() {
        let grid: Grid<u8> = grid![[1,2][3,4]];
        let mut iter = grid.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn clear() {
        let mut grid: Grid<u8> = grid![[1, 2, 3]];
        grid.clear();
        assert!(grid.is_empty());
    }

    #[test]
    fn is_empty_false() {
        let grid: Grid<u8> = grid![[1, 2, 3]];
        assert!(!grid.is_empty());
    }

    #[test]
    fn is_empty_true() {
        let grid: Grid<u8> = grid![];
        assert!(grid.is_empty());
    }

    #[test]
    fn fmt_empty() {
        let grid: Grid<u8> = grid![];
        assert_eq!(format!("{:?}", grid), "[]");
    }

    #[test]
    fn fmt_row() {
        let grid: Grid<u8> = grid![[1, 2, 3]];
        assert_eq!(format!("{:?}", grid), "[[1, 2, 3]]");
    }

    #[test]
    fn fmt_grid() {
        let grid: Grid<u8> = grid![[1,2,3][4,5,6][7,8,9]];
        assert_eq!(format!("{:?}", grid), "[[1, 2, 3][4, 5, 6][7, 8, 9]]");
    }

    #[test]
    fn clone() {
        let grid = grid![[1, 2, 3][4, 5, 6]];
        let mut clone = grid.clone();
        clone[0][2] = 10;
        assert_eq!(grid[0][2], 3);
        assert_eq!(clone[0][2], 10);
    }

    #[test]
    fn macro_init() {
        let grid = grid![[1, 2, 3][4, 5, 6]];
        assert_eq!(grid[0][0], 1);
        assert_eq!(grid[0][1], 2);
        assert_eq!(grid[0][2], 3);
        assert_eq!(grid[1][0], 4);
        assert_eq!(grid[1][1], 5);
        assert_eq!(grid[1][2], 6);
    }

    #[test]
    fn macro_init_2() {
        let grid = grid![[1, 2, 3]
                         [4, 5, 6]
                         [7, 8, 9]];
        assert_eq!(grid.size(), (3, 3))
    }

    #[test]
    fn macro_init_char() {
        let grid = grid![['a', 'b', 'c']
                         ['a', 'b', 'c']
                         ['a', 'b', 'c']];
        assert_eq!(grid.size(), (3, 3));
        assert_eq!(grid[1][1], 'b');
    }

    #[test]
    fn macro_one_row() {
        let grid: Grid<usize> = grid![[1, 2, 3, 4]];
        assert_eq!(grid.size(), (1, 4));
        assert_eq!(grid[0][0], 1);
        assert_eq!(grid[0][1], 2);
        assert_eq!(grid[0][2], 3);
    }

    #[test]
    fn macro_init_empty() {
        let grid: Grid<usize> = grid![];
        assert_eq!(grid.size(), (0, 0));
    }

    #[test]
    fn from_vec_zero() {
        let grid: Grid<u8> = Grid::from_vec(vec![], 0);
        assert_eq!(grid.size(), (0, 0));
    }

    #[test]
    #[should_panic]
    fn from_vec_panics_1() {
        let _: Grid<u8> = Grid::from_vec(vec![1, 2, 3], 0);
    }

    #[test]
    #[should_panic]
    fn from_vec_panics_2() {
        let _: Grid<u8> = Grid::from_vec(vec![1, 2, 3], 2);
    }

    #[test]
    #[should_panic]
    fn from_vec_panics_3() {
        let _: Grid<u8> = Grid::from_vec(vec![], 1);
    }

    #[test]
    fn init() {
        Grid::init(1, 2, 3);
        Grid::init(1, 2, 1.2);
        Grid::init(1, 2, 'a');
    }

    #[test]
    fn new() {
        let grid: Grid<u8> = Grid::new(1, 2);
        assert_eq!(grid[0][0], 0);
    }

    #[test]
    #[should_panic]
    fn init_panics() {
        Grid::init(0, 2, 3);
    }

    #[test]
    #[should_panic]
    fn ctr_panics_2() {
        Grid::init(1, 0, 3);
    }

    #[test]
    fn get() {
        let grid = Grid::init(1, 2, 3);
        assert_eq!(grid.get(0, 0), Some(&3));
    }
    #[test]
    fn get_none() {
        let grid = Grid::init(1, 2, 3);
        assert_eq!(grid.get(1, 0), None);
    }

    #[test]
    fn get_mut() {
        let mut grid = Grid::init(1, 2, 3);
        let mut_ref = grid.get_mut(0, 0).unwrap();
        *mut_ref = 5;
        assert_eq!(grid[0][0], 5);
    }

    #[test]
    fn get_mut_none() {
        let mut grid = Grid::init(1, 2, 3);
        let mut_ref = grid.get_mut(1, 4);
        assert_eq!(mut_ref, None);
    }

    #[test]
    fn idx() {
        let grid = Grid::init(1, 2, 3);
        assert_eq!(grid[0][0], 3);
    }

    #[test]
    #[should_panic]
    fn idx_panic_1() {
        let grid = Grid::init(1, 2, 3);
        grid[20][0];
    }

    #[test]
    #[should_panic]
    fn idx_panic_2() {
        let grid = Grid::init(1, 2, 3);
        grid[0][20];
    }

    #[test]
    fn idx_set() {
        let mut grid = Grid::init(1, 2, 3);
        grid[0][0] = 4;
        assert_eq!(grid[0][0], 4);
    }

    #[test]
    fn size() {
        let grid = Grid::init(1, 2, 3);
        assert_eq!(grid.size(), (1, 2));
    }
}
