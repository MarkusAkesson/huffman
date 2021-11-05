use std::fmt;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::time::Instant;

const FILE: &str = "/home/markus/dev/skolarbete/eitn45/HandIn2/alice29.txt";
const BYTES: usize = 152_089;

#[derive(Debug, Eq, PartialEq)]
enum Type {
    Byte,
    Node,
}

struct Leaf {
    ch: u8,
    freq: u32,
    leaf_type: Type,
    left: Option<Box<Leaf>>,
    right: Option<Box<Leaf>>,
}

impl Leaf {
    pub fn new(ch: u8, freq: u32, t: Type) -> Leaf {
        Leaf {
            ch,
            freq,
            leaf_type: t,
            left: None,
            right: None,
        }
    }

    fn assign_encoding(&self, encoding: String, vec: &mut [String]) {
        match self.leaf_type {
            Type::Byte => {
                vec[self.ch as usize] = encoding.trim_start_matches(&"0".to_string()).to_string()
            }
            Type::Node => {
                if let Some(ref l) = self.left {
                    l.assign_encoding(encoding.clone() + "0", vec);
                }
                if let Some(ref r) = self.right {
                    r.assign_encoding(encoding.clone() + "1", vec);
                }
            }
        }
    }
}

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.ch, self.freq)
    }
}

fn build_tree(map: &[u32]) -> Leaf {
    let mut nodes: Vec<Leaf> = map
        .iter()
        .enumerate()
        .map(|(i, freq)| Leaf::new(i as u8, *freq, Type::Byte))
        .filter(|l| l.freq > 0)
        .collect();

    while nodes.len() > 1 {
        nodes.sort_by(|a, b| b.freq.cmp(&a.freq));

        let left = nodes.pop().unwrap();
        let right = nodes.pop().unwrap();

        let mut top = Leaf::new(0, left.freq + right.freq, Type::Node);

        top.left = Some(Box::new(left));
        top.right = Some(Box::new(right));

        nodes.push(top);
    }
    nodes.pop().unwrap()
}

fn frequency(content: &str) -> Vec<u32> {
    let mut vec = vec![0; 256];
    content.bytes().for_each(|b| {
        vec[b as usize] += 1;
    });
    vec
}

fn print_frequency(map: &[u32]) {
    let mut s = String::new();
    let bytes: u32 = map.iter().map(|i| i).sum();
    map.iter().enumerate().for_each(|(byte, freq)| {
        if *freq != 0 {
            s.push_str(
                format!(
                    "{} {}: {:.5}\n",
                    byte,
                    match byte as u8 as char {
                        '\n' => "\\n".to_owned(),
                        '\t' => "\\t".to_owned(),
                        '\r' => "\\r".to_owned(),
                        ch => ch.to_string(),
                    },
                    *freq as f64 / bytes as f64
                )
                .as_str(),
            )
        }
    });
    println!("{}", s);
}

fn print_encoding(vec: &[String]) {
    let mut s = String::new();
    vec.iter().enumerate().for_each(|(byte, encoding)| {
        if encoding != "" {
            s.push_str(
                format!(
                    "{} {}: {}\n",
                    byte,
                    match byte as u8 as char {
                        '\n' => "\\n".to_owned(),
                        '\t' => "\\t".to_owned(),
                        '\r' => "\\r".to_owned(),
                        ch => ch.to_string(),
                    },
                    encoding,
                )
                .as_str(),
            );
        }
    });
    println!("{}", s);
}

fn calc_new_size(freq_vec: &[u32], enc_vec: &[String]) -> u32 {
    let size: u32 = freq_vec
        .iter()
        .enumerate()
        .map(|(ch, freq)| {
            let len = enc_vec[ch].len();
            freq * (len as u32)
        })
        .sum();
    size / 8 + if size % 8 == 0 { 0 } else { 1 }
}

fn ones(freq_vec: &[u32], enc_vec: &[String]) -> u32 {
    freq_vec
        .iter()
        .enumerate()
        .map(|(byte, freq)| {
            let ones: u32 = enc_vec[byte]
                .chars()
                .map(|ch| if ch == '1' { 1 } else { 0 })
                .sum();
            freq * ones
        })
        .sum()
}

fn average_len(enc_vec: &[String]) -> f64 {
    let code_len: usize = enc_vec.iter().map(|s| s.len()).sum();
    let len: u32 = enc_vec.iter().map(|s| if s == "" { 0 } else { 1 }).sum();

    code_len as f64 / len as f64
}

fn main() {
    let start = Instant::now();

    let f = File::open(FILE).unwrap();
    let mut reader = BufReader::new(f);

    let mut content = String::with_capacity(BYTES);
    reader.read_to_string(&mut content).unwrap();

    let freq_vec = frequency(&content);
    print_frequency(&freq_vec);

    let root = build_tree(&freq_vec);

    let mut enc_vec = vec![String::new(); 256];
    root.assign_encoding("".to_string(), &mut enc_vec);
    print_encoding(&enc_vec);

    let new_size = calc_new_size(&freq_vec, &enc_vec);
    let ones = ones(&freq_vec, &enc_vec);
    let avg_len = average_len(&enc_vec);
    let words: u32 = freq_vec.iter().map(|f| f).sum();

    let mut s = String::new();
    s.push_str(format!("Mean code length: {}\n", avg_len).as_str());
    s.push_str(format!("Old Size: {} bytes\n", content.len()).as_str());
    s.push_str(format!("New Size: {} bytes\n", new_size).as_str());
    s.push_str(
        format!(
            "Compression ratio: {}\n",
            content.len() as f32 / new_size as f32
        )
        .as_str(),
    );
    s.push_str(
        format!(
            "Percentage of ones: {}%\n",
            100.0 * ones as f32 / (new_size * 8) as f32
        )
        .as_str(),
    );
    s.push_str(
        format!(
            "Average code length: {}\n",
            new_size as f64 / (words / 8) as f64
        )
        .as_str(),
    );

    let elapsed = start.elapsed();
    let sec = (elapsed.as_secs() as f64) + (elapsed.as_nanos() as f64 / 1_000_000_000.0);
    s.push_str(format!("\nExecution time: {}s", sec).as_str());

    println!("{}", s);
}
