#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::io::BufRead;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    assert!(args.len() == 3);

    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut reader = std::io::BufReader::new(file);

    let mut lines = HashSet::<String>::new();

    for line in reader.lines() {
        lines.insert(line.unwrap());
    }

    file = std::fs::File::open(&args[2]).unwrap();
    reader = std::io::BufReader::new(file);

    for line in reader.lines() {
        let unwraped_line = line.unwrap();
        if lines.contains(&unwraped_line) {
            lines.remove(&unwraped_line);
            println!("{unwraped_line}");
        }
    }
}
