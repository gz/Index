//! This main is temporary, and is just meant to test the Index
//! The index lib will be used in a larger project.

use index::*;

use std::fs::File;
use std::io::{BufRead, BufReader};

fn test_index() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("\nUsage: {} <query>\n", args[0]);
        std::process::exit(1);
    }

    let start = std::time::SystemTime::now();

    let mut index: Index<String, Vec<(usize, String)>> = Index::new(); // to see if growing correctly

    let filename = "lear.txt";
    let file = File::open(filename)
        .unwrap_or_else(|_| panic!("Error while opening file: `{}`", filename));
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line: String = line
            .unwrap_or_else(|_| panic!("Error while reading file: `{}` at line: {}", filename, i+1));

        let split = line.split(|c: char| !c.is_alphanumeric());

        for word in split {
            if !word.is_empty() {
                let word = word.to_lowercase();
                let location = (i + 1, filename.to_string());

                let res = index.get_mut(&word);
                match res {
                    Some(mut v) => {
                        v.push(location);
                    }
                    None => {
                        drop(res);
                        index.insert(word, vec![location]);
                    }
                }
            }
        }
    }

    let time = start.elapsed().unwrap();

    println!("\n=====================================================================================");
    println!("Index loaded {} elements in {:?}\n", index.len(), time);

    let query = &args[1];
    println!("QUERY: {:?}", query);

    if let Some(v) = index.get(query) {
        println!("RESPONSE: the word {:?} appears {} times in \"lear.txt\"", query, v.len());
    } else {
        println!("RESPONSE: the word {:?} doesn't appear in \"lear.txt\"", query);
    }

    println!("=====================================================================================\n");
}

fn main() {
    test_index();
}

/* control: 4031 elements */
