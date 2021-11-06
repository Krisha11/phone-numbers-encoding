use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::rc::Rc;
use std::usize;
use string_builder::Builder;

const DICT_FILE: &str = "dictionary.txt";
const IN_FILE: &str = "input.txt";
const OUT_FILE: &str = "output.txt";

const SIZE: usize = 10;

const EMPTY_GO: [Vertex; SIZE] = [None, None, None, None, None, None, None, None, None, None];

const EMPTY_NODE: Node = Node {
    flag: false,
    word_indexes: Vec::new(),
    go: EMPTY_GO,
};

fn read_file(s: &str) -> Vec<String> {
    let dict_path = Path::new(s);
    let dict_file = io::BufReader::new(File::open(&dict_path).unwrap());

    let mut vec = Vec::new();
    for line_iter in dict_file.lines() {
        vec.push(line_iter.unwrap());
    }
    vec
}

fn write_file(dict: &Vec<String>, results: &Vec<(String, Vec<Vec<i32>>)>, path: &str) {
    let file = File::create(&path).unwrap();
    let mut out_file = io::BufWriter::new(file);

    for (before, res) in results {
        for vec in res {
            out_file.write_fmt(format_args!("{} ", before)).unwrap();
            for elem in vec {
                if *elem < 0 {
                    out_file
                        .write_fmt(format_args!("{} ", -(elem + 1)))
                        .unwrap();
                } else {
                    out_file
                        .write_fmt(format_args!("{} ", dict[*elem as usize]))
                        .unwrap();
                }
            }
            out_file.write_fmt(format_args!("\n")).unwrap();
        }
    }
}

fn init_keyboard() -> HashMap<char, usize> {
    let book_reviews = Vec::from([
        (0, Vec::from(['e'])),
        (1, Vec::from(['j', 'n', 'q'])),
        (2, Vec::from(['r', 'w', 'x'])),
        (3, Vec::from(['d', 's', 'y'])),
        (4, Vec::from(['f', 't'])),
        (5, Vec::from(['a', 'm'])),
        (6, Vec::from(['c', 'i', 'v'])),
        (7, Vec::from(['b', 'k', 'u'])),
        (8, Vec::from(['l', 'o', 'p'])),
        (9, Vec::from(['g', 'h', 'z'])),
    ]);

    let mut numbers: HashMap<char, usize> = HashMap::new();
    for (n, v) in book_reviews {
        for ch in v {
            numbers.insert(ch, n);
        }
    }

    numbers
}

fn words_to_nums(words: &Vec<String>, numbers: &HashMap<char, usize>) -> Vec<Vec<usize>> {
    let mut res = Vec::new();
    for word in words {
        let mut builder = Vec::new();
        for ch in word.to_lowercase().chars() {
            if !(ch >= 'a' && ch <= 'z') {
                continue;
            }
            builder.push(numbers[&ch])
        }
        res.push(builder)
    }

    res
}

fn clean_nums(nums: &Vec<String>) -> Vec<(String, String)> {
    let mut res = Vec::new();
    for num in nums {
        let mut builder = Builder::default();
        for ch in num.chars() {
            if !(ch >= '0' && ch <= '9') {
                continue;
            }
            builder.append(ch)
        }
        res.push((builder.string().unwrap(), num.clone()))
    }

    res
}

type Vertex = Option<Rc<RefCell<Node>>>;

#[derive(Clone)]
struct Node {
    flag: bool,
    word_indexes: Vec<usize>,
    go: [Vertex; SIZE],
}

fn insert(v: &mut Vertex, word_index: usize, c: &Vec<usize>, cur: usize) {
    if cur == (*c).len() {
        if let Some(b) = v {
            let elem = &mut (*b).borrow_mut();
            (*elem).flag = true;
            (*elem).word_indexes.push(word_index);
        }

        return;
    }

    if let Some(b) = v {
        let index = c[cur];
        let elem = &mut (*b).borrow_mut().go[index];
        if let None = elem {
            (*elem) = Some(Rc::new(RefCell::new(EMPTY_NODE.clone())));
        }
        insert(&mut *elem, word_index, c, cur + 1)
    }
}

fn build(root: &mut Vertex, d: &Vec<Vec<usize>>) {
    for i in 0..d.len() {
        insert(root, i, &d[i], 0)
    }
}

fn char_to_int(a: char) -> i32 {
    match a.to_digit(10) {
        Some(x) => x as i32,
        None => -1,
    }
}

fn get_ans<'a>(
    root: &'a Vertex,
    word: &'a String,
    mut index: usize,
    can_digit: bool,
) -> Vec<Vec<i32>> {
    let start_index = index;
    let mut link = (*root).clone();
    let len = word.len();
    if index >= len {
        return vec![Vec::new()];
    }

    let mut res: Vec<Vec<i32>> = Vec::new();
    let mut has_word = false;
    while index < len {
        if let Some(b) = link {
            let ch = word.chars().nth(index).unwrap();
            link = (*b).borrow().go[char_to_int(ch) as usize].clone();

            if !(*b).borrow().flag {
                index += 1;
                continue;
            }

            has_word = true;
            let new_res = get_ans(root, word, index, true);
            for elem in &(*b).borrow().word_indexes {
                for i in 0..new_res.len() {
                    let mut start = vec![*elem as i32];
                    start.extend(new_res[i].iter());

                    res.push(start);
                }
            }
        } else {
            break;
        }

        index += 1;
    }

    if index == len {
        if let Some(b) = link {
            if (*b).borrow().flag {
                has_word = true;
                let indexes = (*b).borrow().word_indexes.clone();
                for elem in indexes {
                    res.push(vec![elem as i32]);
                }
            }
        }
    }

    if !has_word && can_digit {
        let new_res = get_ans(root, word, start_index + 1, false);

        let ch = word.chars().nth(start_index).unwrap();
        for i in 0..new_res.len() {
            let mut start = vec![-char_to_int(ch) - 1];
            start.extend(new_res[i].iter());
            res.push(start);
        }
    }

    res
}

fn main() {
    let numbers = init_keyboard();
    let dict = read_file(&DICT_FILE);
    let dict_nums = words_to_nums(&dict, &numbers);

    let mut root: Vertex = Some(Rc::new(RefCell::new(EMPTY_NODE.clone())));
    build(&mut root, &dict_nums);

    let nums = read_file(&IN_FILE);
    let nums_good = clean_nums(&nums);

    let mut res = Vec::new();
    for (good, before) in nums_good {
        res.push((before, get_ans(&mut root, &good, 0, true)));
    }
    write_file(&dict, &res, &OUT_FILE);
}
