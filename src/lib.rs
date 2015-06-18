extern crate movecell;
extern crate typed_arena;

use std::collections::HashSet;
use std::hash::Hash;

use movecell::MoveCell;
use typed_arena::Arena;

pub struct Graph<'a, T: 'a + Hash + Eq> {
    arena: Arena<Node<'a, T>>,
    root: MoveCell<Option<&'a Node<'a, T>>>
}

impl <'a, T: 'a + Hash + Eq> Graph<'a, T> {
    pub fn new() -> Self {
        Graph {
            arena: Arena::new(),
            root: MoveCell::new(None),
        }
    }

    #[inline]
    pub fn new_node(&'a self, node: Node<'a, T>) -> &'a Node<'a, T> {
        self.arena.alloc(node)
    }

    #[inline]
    pub fn set_root(&'a self, root: Node<'a, T>) -> &'a Node<'a, T> {
        let root = self.new_node(root);
        self.root.replace(Some(root));
        root
    }

    pub fn traverse<F>(&'a self, f: &F)
        where F: Fn(&T)
    {
        self.root.map(|e| e.traverse(f, &mut HashSet::new()));
    }
}

pub struct Node<'a, T: 'a + Hash + Eq> {
    pub datum: T,
    edges: MoveCell<Option<Vec<&'a Node<'a, T>>>>
}

impl<'a, T: 'a + Hash + Eq> Node<'a, T> {
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

    pub fn traverse<F>(&'a self, f: &F, seen: &mut HashSet<&'a T>)
        where F: Fn(&T)
    {
        if seen.contains(&self.datum) {
            return;
        }

        f(&self.datum);
        seen.insert(&self.datum);
        self.edges.map(|edges| {
            for e in edges {
                e.traverse(f, seen);
            }
        });
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

    graph.traverse(&|n| println!("{}", n));
}
