
/// A field of text
struct TextField {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub selection_marker: Cursor,
}

impl TextField {
    /// Creates a new text field
    pub fn new(text: &str) -> TextField {
        let cursor = Cursor::new(0, 0);
        let lines = text.lines().map(|s| s.to_string()).collect();
        TextField { lines: lines, cursor: cursor,
            selection_marker: cursor.clone()
        }
    }
    
    /// Returns whether the text-field has text selected or not
    pub fn has_selection(&self) -> bool {
        self.selection_marker.constrained(&self.lines) != self.cursor.constrained(&self.lines)
    }
    
    /// Deselects the current selected area
    fn deselect(&mut self) {
        self.selection_marker = self.cursor.clone();
    }
    
    /// Moves the cursor left
    pub fn left(&mut self) {
        if self.has_selection() {
            self.cursor = self.cursor.clone_min(self.selection_marker);
        } else {
            self.cursor.left();
        }
        self.deselect();
    }
    
    /// Movest the cursor right
    pub fn right(&mut self) {
        if self.has_selection() {
            self.cursor = self.cursor.clone_max(self.selection_marker);
        } else {
            self.cursor.right();
        }
        self.deselect();
    }
    
    /// Moves the cursor up
    pub fn up(&mut self) {
        if self.has_selection() {
            let mut tmp = self.cursor.clone_min(self.selection_marker).up();
            tmp.col = self.cursor.col;
            self.cursor = tmp;
        } else {
            self.cursor.up();
        }
        self.deselect();
    }
    
    /// Moves the cursor down
    pub fn down(&mut self) {
        if self.has_selection() {
            let mut tmp = self.cursor.clone_max(self.selection_marker).down();
            tmp.col = self.cursor.col;
            self.cursor = tmp;
        } else {
            self.cursor.down();
        }
        self.deselect();
    }
}
