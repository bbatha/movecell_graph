extern crate movecell;
extern crate typed_arena;

use std::cell::Cell;
use std::ops::Deref;
use std::iter::FromIterator;

use movecell::MoveCell;
use typed_arena::Arena;

/// An immovable graph type
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

    /// Moves a constructed node into the graph. This allows for construction
    /// of nodes in a more convient fashion.
    #[inline]
    pub fn own_node(&'a self, node: Node<'a, T>) -> &'a Node<'a, T> {
        self.arena.alloc(node)
    }

    /// Get the root node of the graph
    /// # Panics
    /// If root is None
    #[inline]
    pub fn root(&self) -> &Node<'a, T> {
        self.root.get().unwrap()
    }

    /// Set the root node to some `&Node` owned by the graph.
    #[inline]
    pub fn set_root(&'a self, root: &'a Node<'a, T>) -> &'a Node<'a, T> {
        self.root.set(Some(root));
        root
    }
}

/// Most actions applied to the graph are really just applied to the root node
/// recursively (or through an iterator). The graph type is just a facade to
/// own the data for each node.
impl<'a, T> Deref for Graph<'a, T> {
    type Target = Node<'a, T>;

    /// # Panics
    /// If root is not set
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.root()
    }
}

/// Graph node type
pub struct Node<'a, T: 'a> {
    pub datum: T,
    // Allows interior mutability so that the graph can be constructed top down rather than bottom
    // up
    edges: MoveCell<Option<Vec<&'a Node<'a, T>>>>
}

impl<'a, T: 'a> Node<'a, T> {
    pub fn new(datum: T) -> Node<'a, T> {
        Node {
            datum: datum,
            edges: MoveCell::new(None),
        }
    }

    /// Adds a `&Node<T>` to the list of edges
    pub fn add_edge(&self, edge: &'a Node<'a, T>) -> &'a Node<'a, T> {
        let mut edges = self.edges.take().unwrap_or(Vec::new());
        edges.push(edge);
        self.edges.replace(Some(edges));
        edge
    }

    /// Iterator adapter for Depth-first traversals of the graph
    pub fn dfs(&'a self) -> DfsIter<T> {
        DfsIter {
            branch_points: vec![(None, self)]
        }
    }
}

impl<'a, T> FromIterator<&'a Node<'a, T>> for &'a Node<'a, T> {
    /// Makes the assumption that the first node to come off an IntoIterator of `&Node` is the root
    /// node. This happens to be correct for BFS and DFS iteration. Any users of `.map()` must be
    /// careful to ensure that the root node is the first produced node.
    ///
    /// NOTE: When running the evaluations this will stop at the first `None` seen. If the
    /// iteration performs operations on any nodes after a single iteration returns a `None`,
    /// those subsequent executions will be skipped.
    ///
    /// # Panics
    /// If the iteration doesn't produce any `&Node`s
    fn from_iter<I: IntoIterator<Item=&'a Node<'a, T>>>(iterable: I) -> Self {
        let mut iter = iterable.into_iter();
        let root = iter.next().unwrap();
        while iter.next().is_some() {  };
        root
    }
}

/// Depth-first iterator adapter for Nodes
pub struct DfsIter<'a, T: 'a> {
    branch_points: Vec<(Option<usize>, &'a Node<'a, T>)>,
}

impl<'a, T> Iterator for DfsIter<'a, T> {
    type Item = &'a Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(branch_point) = self.branch_points.pop() {
            let (index, node) = branch_point;

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
    let node1 = graph.set_root(graph.own_node(Node::new(1)));
    let node2 = node1.add_edge(graph.own_node(Node::new(2)));
    let _node3 = node1.add_edge(graph.own_node(Node::new(3)));
    let _node4 = node2.add_edge(graph.own_node(Node::new(4)));
    let _node5 = node2.add_edge(graph.own_node(Node::new(5)));

    println!("Graph 1 (original):");
    for node in graph.dfs() {
        println!("{}", node.datum);
    }

    println!("Graph 1 (mapped):");
    let graph2 = Graph::new();
    let root = graph.dfs().map(|e| graph2.own_node(Node::new(e.datum as f32 * 2.1))).collect();
    graph2.set_root(root);
    for node in graph2.dfs() {
        println!("{}", node.datum);
    }
}
