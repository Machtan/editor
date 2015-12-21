use std::fmt::{Display, Formatter, Error};

pub type Navigable = Vec<String>;

/*pub trait Lengthy {
	fn len(&self) -> usize;
}

impl <'a> Lengthy for &'a str {
	fn len(&self) -> usize {
		self.len()
	}
}*/

/// A position within a text document
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Cursor {
	pub line: usize,
	pub col: usize,
}

impl Cursor {
	/// Creates a new cursor.
	pub fn new(line: usize, col: usize) -> Cursor {
		Cursor { line: line, col: col }
	}

	/// Moves the cursor left and returns it.
	pub fn left(&mut self, source: &Navigable) -> &mut Cursor {
		if self.col == 0 {
			if self.line != 0 {
				self.line -= 1;
				self.col = source[self.line].len();
			}
		} else {
            self.constrain_col(source);
			self.col -= 1;
		}
		self
	}

	/// Moves the cursor right and returns it.
	pub fn right(&mut self, source: &Navigable) -> &mut Cursor {
		if self.col >= source[self.line].len() {
			if self.line != (source.len() - 1) {
				self.line += 1;
				self.col = 0;
			}
		} else {
			self.col += 1;
		}
		self
	}

	/// Moves the cursor up and returns it.
	pub fn up(&mut self) -> &mut Cursor {
		if self.line == 0 {
			self.col = 0;
		} else {
			self.line -= 1;
		}
		self
	}

	/// Moves the cursor down and returns it.
	pub fn down(&mut self, source: &Navigable) -> &mut Cursor {
		if self.line == source.len() - 1 {
			self.col = source[self.line].len();
		} else {
			self.line += 1;
		}
		self
	}
    
    /// Returns a new copy of the smaller of the two cursors.
    pub fn clone_min(&self, other: &Cursor) -> Cursor {
        if self < other {
            self.clone()
        } else {
            other.clone()
        }
    }
    
    /// Returns a new copy of the larger of the two cursors.
    pub fn clone_max(&self, other: &Cursor) -> Cursor {
        if self > other {
            self.clone()
        } else {
            other.clone()
        }
    }

	/// Prints the position of this cursor in the given text source.
	pub fn debug(&self, source: &Navigable) {
		for (lineno, line) in source.iter().enumerate() {
			if lineno == self.line {
				if self.col > line.len() {
					println!("{}|", line);
				} else {
					println!("{}|{}", &line[..self.col], &line[self.col..]);
				}
			} else {
				println!("{}", line);
			}
		}
	}
    
    /// Constrains the column of the cursor to be within the current line.
    fn constrain_col(&mut self, source: &Navigable) {
        let mut copy = self.clone();
        copy.constrain_line(source);
        let len = source[copy.line].len();
        if self.col > len {
            self.col = len;
        }
    }
    
    /// Constrains the line of the cursor to be within the length of the source.
    fn constrain_line(&mut self, source: &Navigable) {
        let len = source.len();
        if self.line > len {
            self.line = len;
        }
    }
    
    /// Returns the position the cursor should be shown in in the source.
    pub fn constrained(&self, source: &Navigable) -> Cursor {
        let mut cursor = self.clone();
        cursor.constrain(source);
        cursor
    }
    
    /// Constrains the cursor to fit within the given text
    pub fn constrain(&mut self, source: &Navigable) {
        self.constrain_col(source);
        self.constrain_line(source);
    }
}

impl Display for Cursor {
	fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
		fmt.write_str(&format!("({}, {})", self.line, self.col))
	}
}
