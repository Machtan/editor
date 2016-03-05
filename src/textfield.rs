
use cursor::Cursor;
use common::StringSliceExt;

/// A field of text
#[derive(Debug, Clone)]
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
        let cons = self.selection_marker.constrained(&self.lines);
        cons != self.cursor.constrained(&self.lines)
    }
    
    /// Deselects the current selected area
    pub fn clear_selection(&mut self) {
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
        self.clear_selection();
    }
    
    /// Movest the cursor right
    pub fn right(&mut self) {
        if self.has_selection() {
            self.cursor = self.cursor.clone_max(&self.selection_marker);
        } else {
            self.cursor.right(&self.lines);
        }
        self.clear_selection();
    }
    
    /// Moves the cursor up
    pub fn up(&mut self) {
        if self.has_selection() {
            let tmp = self.cursor.clone_min(&self.selection_marker);
            let mut cons = tmp.constrained(&self.lines);
            cons.up();
            self.cursor = cons;
        } else {
            self.cursor.up();
        }
        self.clear_selection();
    }
    
    /// Moves the cursor down
    pub fn down(&mut self) {
        if self.has_selection() {
            let tmp = self.cursor.clone_max(&self.selection_marker);
            let mut cons = tmp.constrained(&self.lines);
            cons.down(&self.lines);
            self.cursor = cons;
        } else {
            self.cursor.down(&self.lines);
        }
        self.clear_selection();
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
    
    /// Returns the current selected text.
    pub fn selected_text(&mut self) -> String {
        let mut text = String::new();
        if ! self.has_selection() {
            return text
        }
        let (first, last) = self.cursor.order(&self.selection_marker);
        if first.line == last.line {
            let ref line = self.lines[first.line];
            let left_index = line.slice_until(first.col).len();
            let right_index = line.len() - line.slice_after(last.col).len();
            text.push_str(&line[left_index .. right_index]);
        } else {
            text.push_str(self.lines[first.line].slice_after(first.col));
            text.push_str("\n");
            let middle_lines = last.line - first.line - 1;
            for i in 0 .. middle_lines {
                text.push_str(&self.lines[first.line + i + 1]);
                text.push_str("\n");
            }
            text.push_str(self.lines[last.line].slice_until(last.col));
        }
        
        text
    }
    
    /// Deletes the current selection of the text field.
    pub fn delete_selection(&mut self) {
        if ! self.has_selection() {
            return;
        }
        let (first, last) = self.cursor.order(&self.selection_marker);
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
            for _ in 0 .. middle_lines {
                self.lines.remove(first.line + 1);
            }
        }
        self.cursor = first.clone();
        self.clear_selection();
    }
    
    /// Delete a character bacward in the text, or the current selection.
    pub fn delete_previous(&mut self) {
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
                self.clear_selection();
            
            // Merge with previous (if any)
            } else {
                if self.cursor.line != 0 {
                    let line = self.lines.remove(self.cursor.line);
                    let prev_len = self.lines[self.cursor.line - 1]
                        .chars().count();
                    self.lines[self.cursor.line - 1].push_str(&line);
                    self.cursor = Cursor::new(self.cursor.line - 1, prev_len);
                    self.clear_selection();
                }
            }
        } else {
            self.delete_selection();
        }
    }
    
    /// Delete a character forward in the text, or the current selection.
    pub fn delete_next(&mut self) {
        if ! self.has_selection() {
            let cons = self.cons_cursor();
            let line_len = self.lines[cons.line].chars().count();
            // Delete within same line
            if cons.col != line_len {
                let text = {
                    let ref line = self.lines[self.cursor.line];
                    let mut new_line = String::new();
                    new_line.push_str(line.slice_until(cons.col));
                    new_line.push_str(line.slice_after(cons.col + 1));
                    new_line
                };
                self.lines[self.cursor.line] = text;
                self.cursor = cons;
                self.clear_selection();
            
            // Merge with next (if any)
            } else {
                if self.cursor.line != (self.lines.len() - 1) {
                    let line = self.lines.remove(self.cursor.line + 1);
                    self.lines[self.cursor.line].push_str(&line);
                    self.cursor = cons;
                    self.clear_selection();
                }
            }
        
        } else {
            self.delete_selection();
        }
    }
    
    pub fn insert(&mut self, text: &str) {
        self.delete_selection();
        let start = self.cursor.line;
        
        let left = String::from(self.lines[start].slice_until(self.cursor.col));
        let right = String::from(self.lines[start].slice_after(self.cursor.col));
        self.lines[start] = left;
        let mut num_lines = 0;
        for (num, line) in text.lines().enumerate() {
            if num == 0 {
                self.lines[start].push_str(line);
            } else {
                self.lines.insert(start + num + 1, String::from(line));
            }
            num_lines += 1;
        }
        if text.ends_with("\n") { // Insert the line that 'lines' ignores
            self.lines.insert(start + num_lines, String::new());
            num_lines += 1;
        }
        
        // Update the cursor
        if num_lines == 1 {
            self.cursor = self.cons_cursor();
            self.cursor.col += text.chars().count();
        } else {
            let line_chars = self.lines[start + num_lines - 1].chars().count();
            self.cursor = Cursor::new(start + num_lines - 1, line_chars);
        }
        self.clear_selection();
        
        // Add the last part to the last line
        self.lines[start + num_lines - 1].push_str(&right);
    }
}
