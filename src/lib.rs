extern crate movecell;
extern crate typed_arena;

use std::cell::Cell;
use std::ops::Deref;

use movecell::MoveCell;
use typed_arena::Arena;

pub struct Graph<'a, T: 'a> {
    arena: Arena<Node<'a, T>>,
    root: Cell<Option<&'a Node<'a, T>>>
}

impl <'a, T: 'a> Graph<'a, T> {
    pub fn new() -> Self {
        Graph {
            arena: Arena::new(),
            root: Cell::new(None),
        }
    }

    #[inline]
    pub fn new_node(&'a self, node: Node<'a, T>) -> &'a Node<'a, T> {
        self.arena.alloc(node)
    }

    /// # Panics
    /// If root is None
    #[inline]
    pub fn root(&self) -> &Node<'a, T> {
        self.root.get().unwrap()
    }

    #[inline]
    pub fn set_root(&'a self, root: Node<'a, T>) -> &'a Node<'a, T> {
        let root = self.new_node(root);
        self.root.set(Some(root));
        root
    }
}

impl<'a, T> Deref for Graph<'a, T> {
    type Target = Node<'a, T>;

    /// # Panics
    /// If root is not set
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.root()
    }
}

pub struct Node<'a, T: 'a> {
    pub datum: T,
    edges: MoveCell<Option<Vec<&'a Node<'a, T>>>>
}

impl<'a, T: 'a> Node<'a, T> {
    pub fn new(datum: T) -> Node<'a, T> {
        Node {
            datum: datum,
            edges: MoveCell::new(None),
        }
    }

    pub fn add_edge(&self, edge: &'a Node<'a, T>) -> &'a Node<'a, T> {
        let mut edges = self.edges.take().unwrap_or(Vec::new());
        edges.push(edge);
        self.edges.replace(Some(edges));
        edge
    }

    pub fn dfs(&'a self) -> DfsIter<T> {
        DfsIter {
            branch_points: vec![(None, self)]
        }
    }
}

pub struct DfsIter<'a, T: 'a> {
    branch_points: Vec<(Option<usize>, &'a Node<'a, T>)>,
}

impl<'a, T> Iterator for DfsIter<'a, T> {
    type Item = &'a Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(branch_point) = self.branch_points.pop() {
            let (index, node) = branch_point;

            // TODO: None panicing version that returns none if no edges
            if let Some(edges) = node.edges.take() {
                let found_node = match index {
                    None => {
                        self.branch_points.push((Some(0), node));
                        Some(node)
                    },
                    Some(index) => {
                        if index < edges.len() {
                            self.branch_points.push((Some(index + 1), node));
                            self.branch_points.push((None, edges[index]));
                        }
                        None
                    }
                };

                node.edges.replace(Some(edges));
                if found_node.is_some() {
                    return found_node;
                }
            } else {
                return Some(node);
            }
        }
        None
    }
}

#[test]
fn it_works() {
    let graph = Graph::new();
    let node1 = graph.set_root(Node::new(1));
    let node2 = node1.add_edge(graph.new_node(Node::new(2)));
    let _node3 = node1.add_edge(graph.new_node(Node::new(3)));
    let _node4 = node2.add_edge(graph.new_node(Node::new(4)));
    let _node5 = node2.add_edge(graph.new_node(Node::new(5)));

    for node in graph.dfs() {
        println!("{}", node.datum);
    }
}
