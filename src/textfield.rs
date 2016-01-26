
use cursor::Cursor;
use common::StringSliceExt;

/// A field of text
pub struct Textfield {
    pub lines: Vec<String>,
    pub cursor: Cursor,
    pub selection_marker: Cursor,
}

impl Textfield {
    /// Creates a new text field
    pub fn new(text: &str) -> Textfield {
        let cursor = Cursor::new(0, 0);
        let marker = cursor.clone();
        let lines = text.lines().map(|s| s.to_string()).collect();
        Textfield { lines: lines, cursor: cursor,
            selection_marker: marker
        }
    }
    
    /// Returns whether the text-field has text selected or not
    pub fn has_selection(&self) -> bool {
        self.selection_marker.constrained(&self.lines) != self.cursor.constrained(&self.lines)
    }
    
    /// Deselects the current selected area
    pub fn deselect(&mut self) {
        self.selection_marker = self.cursor.clone();
    }
    
    /// Returns the cursor constrained to the text.
    pub fn cons_cursor(&self) -> Cursor {
        self.cursor.constrained(&self.lines)
    }
    
    /// Returns the selection marker constrained to the text.
    pub fn cons_marker(&self) -> Cursor {
        self.selection_marker.constrained(&self.lines)
    }
    
    /// Moves the cursor left
    pub fn left(&mut self) {
        if self.has_selection() {
            self.cursor = self.cursor.clone_min(&self.selection_marker);
        } else {
            self.cursor.left(&self.lines);
        }
        self.deselect();
    }
    
    /// Movest the cursor right
    pub fn right(&mut self) {
        if self.has_selection() {
            self.cursor = self.cursor.clone_max(&self.selection_marker);
        } else {
            self.cursor.right(&self.lines);
        }
        self.deselect();
    }
    
    /// Moves the cursor up
    pub fn up(&mut self) {
        if self.has_selection() {
            let mut tmp = self.cursor.clone_min(&self.selection_marker);
            tmp.up();
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
            let mut tmp = self.cursor.clone_max(&self.selection_marker);
            tmp.down(&self.lines);
            tmp.col = self.cursor.col;
            self.cursor = tmp;
        } else {
            self.cursor.down(&self.lines);
        }
        self.deselect();
    }
    
    pub fn select_left(&mut self) {
        if self.has_selection() {
            self.selection_marker.left(&self.lines);
        } else {
            self.selection_marker = self.cursor.constrained(&self.lines);
            self.selection_marker.left(&self.lines);
        }
    }
    
    pub fn select_right(&mut self) {
        if self.has_selection() {
            self.selection_marker.right(&self.lines);
        } else {
            self.selection_marker = self.cursor.constrained(&self.lines);
            self.selection_marker.right(&self.lines);
        }
    }
    
    pub fn select_up(&mut self) {
        if self.has_selection() {
            self.selection_marker.up();
        } else {
            self.selection_marker = self.cursor.constrained(&self.lines);
            self.selection_marker.up();
        }
    }
    
    pub fn select_down(&mut self) {
        if self.has_selection() {
            self.selection_marker.down(&self.lines);
        } else {
            self.selection_marker = self.cursor.constrained(&self.lines);
            self.selection_marker.down(&self.lines);
        }
    }
    
    pub fn selected_text(&mut self) -> String {
        String::new()
    }
    
    pub fn delete_selection(&mut self) {
        if ! self.has_selection() {
            return;
        }
        let (first, last) = if self.cursor < self.selection_marker {
            (self.cursor.clone(), self.selection_marker.clone())
        } else {
            (self.selection_marker.clone(), self.cursor.clone())
        };
        // Same line
        if first.line == last.line {
            let text = {
                let ref line = self.lines[self.cursor.line];
                let mut new_line = String::new();
                new_line.push_str(line.slice_until(first.col));
                new_line.push_str(line.slice_after(last.col));
                new_line
            };
            self.lines[first.line] = text;
        } else {
            let text = {
                let ref first_line = self.lines[first.line];
                let ref last_line = self.lines[last.line];
                let mut new_line = String::new();
                new_line.push_str(first_line.slice_until(first.col));
                new_line.push_str(last_line.slice_after(last.col));
                new_line
            };
            self.lines[first.line] = text;
            
            let middle_lines = last.line - first.line;
            for i in 0 .. middle_lines {
                self.lines.remove(first.line + 1);
            }
        }
        self.cursor = first.clone();
        self.selection_marker = first.clone();
    }
    
    pub fn delete(&mut self) {
        if ! self.has_selection() {
            let cons = self.cons_cursor();
            // Delete within same line
            if cons.col > 0 {
                let text = {
                    let ref line = self.lines[self.cursor.line];
                    let mut new_line = String::new();
                    new_line.push_str(line.slice_until(cons.col - 1));
                    new_line.push_str(line.slice_after(cons.col));
                    new_line
                };
                self.lines[self.cursor.line] = text;
                self.cursor = cons;
                self.cursor.col -= 1;
                self.deselect();
            // Merge with previous (if any)
            } else {
                if self.cursor.line != 0 {
                    let line = self.lines.remove(self.cursor.line);
                    self.lines[self.cursor.line - 1].push_str(&line);
                    self.cursor = Cursor::new(self.cursor.line - 1, line.len());
                    self.deselect();
                }
                
            }
        } else {
            self.delete_selection();
        }
    }
    
    pub fn delete_forward(&mut self) {
        if ! self.has_selection() {
        
        } else {
            self.delete_selection();
        }
    }
}
