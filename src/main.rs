use rfd::MessageDialog;
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    io::{BufRead, LineWriter, Write},
    path::{Path, PathBuf},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let exe_dir = if cfg!(target_os = "macos") {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    } else {
        std::env::current_exe().unwrap().to_path_buf()
    };

    let raw_paths = match args.len() {
        2.. => args
            .iter()
            .skip(1)
            .map(|s| PathBuf::from(s))
            .collect::<Vec<_>>(),
        _ => rfd::FileDialog::new()
            .set_title("Select file to expand")
            .set_directory(exe_dir)
            .pick_files()
            .map(|files| files)
            .unwrap_or(vec![]),
    };

    if raw_paths.is_empty() {
        return;
    }

    for raw_path in &raw_paths {
        let filepath =
            std::fs::canonicalize(raw_path).unwrap_or_else(|_| Path::new(raw_path).to_path_buf());

        if filepath.is_dir() {
            if let Err(e) = create_schema(&filepath) {
                MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error While Creating Schema")
                    .set_description(&format!("{}", e))
                    .show();
            }
        } else {
            if let Err(e) = process_dir_file(&filepath) {
                MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Error While Processing File")
                    .set_description(&format!("{}", e))
                    .show();
            }
        }
    }
}
pub fn create_schema(filepath: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create_new(filepath.with_extension("dir"))?;
    let mut writer = LineWriter::new(file);
    let lines = sub_schema(filepath, 0)?;

    for line in &lines {
        writer.write(line.as_bytes())?;
        writer.write_all(b"\n")?;
    }

    println!("{} lines written to {}", lines.len(), filepath.display());
    Ok(())
}
pub fn sub_schema(
    filepath: &PathBuf,
    depth: usize,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(filepath)?
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort(); // alphabetical sort among siblings

    let mut lines = Vec::new();
    for path in entries {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            lines.push(format!("{}{}/", "    ".repeat(depth), name));
            lines.extend(sub_schema(&path, depth + 1)?);
        } else {
            lines.push(format!("{}{}", "    ".repeat(depth), name));
        }
    }
    Ok(lines)
}
pub fn process_dir_file(filepath: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let path = if filepath.extension().map(|e| e == "dir").unwrap_or(false) {
        filepath.clone()
    } else {
        filepath.with_extension("dir")
    };

    let file = fs::File::open(&path).expect("Failed to open input file");
    let reader = std::io::BufReader::new(file);
    let mut lines = reader
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.unwrap();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let indent = line.chars().take_while(|c| c.is_whitespace()).count();
            let name = line.trim().to_string();
            Some(IndentMeasuredString {
                indent,
                name,
                source_line: i + 1,
            })
        })
        .collect::<Vec<_>>();

    normalize_indentation(&mut lines);

    let mut root = Node::Directory(Directory {
        name: filepath
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string(),
        children: Vec::new(),
    });

    let mut stack = vec![&mut root as *mut Node];
    for line in &mut lines {
        let level = line.indent;
        stack.truncate(level + 1);

        let new_node = if line.name.ends_with('/') {
            let dir_name = line.name.trim_end_matches('/');
            Node::Directory(Directory {
                name: dir_name.to_string(),
                children: Vec::new(),
            })
        } else {
            Node::File(line.name.clone())
        };

        unsafe {
            match &mut (*stack[level]) {
                Node::Directory(dir) => {
                    dir.children.push(new_node);
                    let child_ref = dir.children.last_mut().unwrap() as *mut Node;
                    stack.push(child_ref);
                }
                Node::File(_) => {
                    return Err(format!(
                        "Unexpected child node at line {}. Files cannot have children",
                        line.source_line
                    )
                    .into());
                }
            }
        }
    }

    let path = filepath.parent().ok_or("Failed to get parent directory")?;

    create_node(&root, &path)?;
    Ok(())
}

fn create_node(node: &Node, path: &Path) -> Result<(), std::io::Error> {
    match &node {
        Node::Directory(dir) => {
            let dir_path = path.join(&dir.name);
            if dir_path.exists() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!(
                        "Path '{}' already exists",
                        dir_path.file_name().unwrap_or(OsStr::new("")).display()
                    ),
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
    source_line: usize,
}

struct Directory {
    name: String,
    children: Vec<Node>,
}

enum Node {
    Directory(Directory),
    File(String),
}
