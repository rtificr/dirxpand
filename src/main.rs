use std::{collections::BTreeSet, env::args, fs, io::BufRead, path::Path};

fn main() {
    let filename = args().nth(1).expect("No input file provided");
    let path = Path::new(&filename).with_extension("dir");

    let file = fs::File::open(&path).expect("Failed to open input file");
    let reader = std::io::BufReader::new(file);
    let mut lines = reader
        .lines()
        .filter_map(|line| {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let name = line.trim().to_string();
            Some(IndentMeasuredString { indent, name })
        })
        .collect::<Vec<_>>();

    normalize_indentation(&mut lines);

    let mut root = Node::Directory(Directory {
        name: filename.to_string(),
        children: Vec::new(),
    });
    let mut stack  = vec![&mut root as *mut Node];
    for line in &mut lines {
        let level = line.indent;
        stack.truncate(level + 1);

        let new_node = if line.name.ends_with("/") {
            let dir_name = line.name.trim_end_matches('/');
            Node::Directory(Directory {
                name: dir_name.to_string(),
                children: Vec::new(),
            })
        } else {
            Node::File(line.name.clone())
        };

        unsafe {
            match &mut *stack[level] {
                Node::Directory(dir) => {
                    dir.children.push(new_node);
                    let child_ref = dir.children.last_mut().unwrap() as *mut Node;
                    stack.push(child_ref);
                }
                Node::File(_) => {
                    panic!("Unexpected child node at level {}. Files cannot have children", level);
                }
            }
        }
    }

    if let Err(e) = create_node(&root, Path::new(".")) {
        println!("Error creating directories: {}", e);
    }
}

fn create_node(root: &Node, path: &Path) -> Result<(), std::io::Error> {
    match root {
        Node::Directory(dir) => {
            let dir_path = path.join(&dir.name);
            if dir_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Path '{}' already exists", dir_path.display()),
                ));
            }
            fs::create_dir_all(&dir_path)?;
            for child in &dir.children {
                create_node(child, &dir_path)?;
            }
        }
        Node::File(file_name) => {
            let file_path = path.join(file_name);
            fs::File::create(file_path)?;
        }
    }
    Ok(())
}

fn normalize_indentation(lines: &mut Vec<IndentMeasuredString>) {
    let mut indent_levels = BTreeSet::new();
    let mut leading_spaces = Vec::new();

    for line in &mut *lines {
        indent_levels.insert(line.indent);
        leading_spaces.push(line.indent);
    }

    let indent_map = indent_levels
        .iter()
        .enumerate()
        .map(|(index, &indent)| (indent, index))
        .collect::<std::collections::HashMap<_, _>>();

    for line in lines {
        if let Some(&index) = indent_map.get(&line.indent) {
            line.indent = index;
        }
    }
}

struct IndentMeasuredString {
    indent: usize,
    name: String,
}

#[derive(Clone)]
struct Directory {
    name: String,
    children: Vec<Node>,
}
#[derive(Clone)]
enum Node {
    Directory(Directory),
    File(String),
}
impl Node {
    fn name(&self) -> &str {
        match self {
            Node::Directory(dir) => &dir.name,
            Node::File(name) => name,
        }
    }

    fn children(&self) -> &[Node] {
        match self {
            Node::Directory(dir) => &dir.children,
            Node::File(_) => &[],
        }
    }
}