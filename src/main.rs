
mod cursor;

fn main() {
    println!("Hello, world!");
    let text = "\
    Hello world!\n\
    What's up?\n\
    This is a line!";
    let source: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let mut cursor = cursor::Cursor::new(1, 5);
    println!("{}", cursor);
    for _ in 0..20 {
        cursor.left(&source);
        //cursor.debug(&source);
    }
    println!("========================");
    for _ in 0..20 {
        cursor.right(&source);
        //cursor.debug(&source);
    }
    let mut copy = cursor.clone();
    copy.right(&source);
    println!("Cursor: {}, Copy: {}", cursor, copy);

    cursor.debug(&source);
    cursor.down(&source);
    cursor.debug(&source);
    cursor.down(&source);
    cursor.debug(&source);
    cursor.up();
    cursor.debug(&source);

}
