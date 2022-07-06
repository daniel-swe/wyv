use std::{collections::HashSet, fs, io, path::Path};

use serde::{Deserialize, Serialize};
use tui::widgets::StatefulWidget;

const NAME_SEP: &str = "/";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileTree {
    file_root: Box<Path>,
    root_node: FileNode,
    state: FileTreeState,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FileTreeState {
    expanded_nodes: HashSet<String>,
}

impl FileTree {
    pub fn new(open: &Path) -> anyhow::Result<Self> {
        let root_node = FileNode::new_from_path(open);
        match root_node {
            Ok(root_node) => Ok(FileTree {
                file_root: Box::from(open.to_owned()),
                root_node,
                state: Default::default(),
            }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn state(&mut self) -> &mut FileTreeState {
        &mut self.state
    }

    fn to_list_with_limit<'a>(self: &'a Self, limit: u16) -> Vec<&'a FileNode> {
        let mut i = 0;
        let mut nodes: Vec<&'a FileNode> = Vec::new();
        nodes.push(&self.root_node);
        while i < limit && (i as usize) < nodes.len() {
            let next = nodes[i as usize];
            if next.has_children() && self.state.expanded_nodes.contains(next.path()) {
                match next {
                    FileNode::Directory(_, c) => {
                        for n in c {
                            nodes.push(&n);
                        }
                    }
                    _ => (),
                };
            }

            i += 1;
        }

        nodes
    }
}

impl StatefulWidget for FileTree {
    type State = FileTreeState;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        if area.width < 1 || area.height < 1 {
            return;
        }

        let h = area.height;
        let w = area.width;
        let list = self.to_list_with_limit(h);

        for i in 0..h {
            if usize::from(i) >= list.len() { break; }
            let start_idx = (i as u64) * (w as u64);
            let node = list[i as usize];
            let indent = node.depth();
            let name = node.name();

            for iw in 0..w {
                if iw < indent { continue; }
                if iw + indent > name.len() as u16 { break; }
                let fx = area.x + iw;
                let fy = area.y + i;
                let cell = &mut buf.content[((fx * fy) as u64 + start_idx) as usize];
                cell.symbol.push_str("X");
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FileNode {
    Directory(String, Vec<FileNode>),
    File(String),
    Link(String, Box<Path>),
}

impl FileNode {
    pub fn new_from_path(path: &Path) -> Result<FileNode, io::Error> {
        let path = fs::canonicalize(path)?;
        FileNode::new_recursive(&path, None)
    }

    fn new_recursive(path: &Path, parent: Option<&str>) -> Result<FileNode, io::Error> {
        let path_name = path.file_name();
        if path_name.is_none() || path_name.unwrap().to_str().is_none() {
            return Err(io::Error::new(io::ErrorKind::Other, ""));
        }

        let mut path_name = path_name.unwrap().to_str().unwrap().to_string();
        if parent.is_some() {
            path_name = parent.unwrap().to_string() + NAME_SEP + &path_name;
        }

        if path.is_file() {
            Ok(FileNode::File(path_name))
        } else if path.is_dir() {
            let mut nodes = fs::read_dir(path)?
                .map(|d| FileNode::new_recursive(&d?.path(), Some(&path_name)))
                .filter(|pr| pr.is_ok())
                .map(|pr| pr.unwrap())
                .collect::<Vec<FileNode>>();
            nodes.sort();

            Ok(FileNode::Directory(path_name, nodes))
        } else if path.is_symlink() {
            Ok(FileNode::Link(
                path_name,
                Box::from(fs::read_link(path)?.as_path()),
            ))
        } else {
            Err(io::Error::new(io::ErrorKind::Other, ""))
        }
    }

    pub fn path(&self) -> &str {
        match self {
            FileNode::Directory(full_name, _)
            | FileNode::File(full_name)
            | FileNode::Link(full_name, _) => full_name,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            FileNode::Directory(full_name, _)
            | FileNode::File(full_name)
            | FileNode::Link(full_name, _) => {
                full_name.rsplit(NAME_SEP).next().unwrap_or(full_name)
            }
        }
    }

    pub fn has_children(&self) -> bool {
        match self {
            &FileNode::Directory(_, _) => true,
            _ => false,
        }
    }

    pub fn depth(&self) -> u16 {
        match self {
            FileNode::Directory(path, _) | FileNode::File(path) | FileNode::Link(path, _) => {
                path.split(NAME_SEP).count().try_into().unwrap()
            }
        }
    }
}

#[test]
fn test_node_build() {
    let n = FileNode::new_from_path(Path::new("./src/widgets")).unwrap();
    match n {
        FileNode::Directory(_, contents) => {
            assert!(contents.contains(&FileNode::File("widgets/file_tree.rs".to_string())))
        }
        _ => panic!(),
    }
}

#[test]
fn test_tree_save_and_load() {
    let n = FileTree::new(Path::new("./src/widgets")).unwrap();
    let s = serde_json::to_string(&n).unwrap();
    let sn = serde_json::from_str::<FileTree>(&s).unwrap();
    assert_eq!(n, sn)
}
