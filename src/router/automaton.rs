use std::{collections::HashMap, hash::Hash};

struct Edge<S, T: Hash + Eq> {
    target: Node<S, T>,
    func: Box<dyn Fn(&mut S, T)>
}

struct Node<S, T: Hash + Eq> {
    matched_edges: HashMap<T, Edge<S, T>>,
    otherwises_edge: Option<Edge<S, T>>,
}

struct Builder<S: Default, T: Hash + Eq>(Automaton<S, T>);

/// The core part of automaton.
/// 
/// # Type parameters
/// 
/// - `S`: The type of state of the automaton.
/// - `T`: The type of input token of the automaton.
pub struct Automaton<S: Default, T: Hash + Eq> {
    start_node: Node<S, T>,
}

pub struct Iter<S: Default, T: Hash + Eq> {
    state: S,
    automaton: Automaton<S, T>,
    index: usize,
}

impl<S: Default, T: Hash + Eq> Builder<S, T> {
    pub fn new() -> Self {
        Builder(Automaton{
            start_node: Node {
                matched_edges: HashMap::new(),
                otherwises_edge: None
            },
        })
    }

    pub fn start_node(self) -> Node<S, T> {
        self.0.start_node
    }
}

impl<S: Default, T: Hash + Eq> BuilderWalker<S, T> {
    pub fn next(mut self, token: T) {
        let node = self.0;
        if let Some(edge) = node.matched_edges.get(&token) {

        }
    }
}

impl<S: Default, T: Hash + Eq> Automaton<S, T> {
    pub fn iter(self) -> Iter<S, T> {
        Iter {
            state: S::default(),
            automaton: &mut self,
            index: 0,
        }
    }
}

impl<S: Default, T: Hash + Eq> Iter<S, T> {
    /// Go the the next node by given token.
    /// 
    /// If there is not a edge for the token, or there is a loop function,
    /// it will create a new node (and a new edge with the token).
    pub fn next_or_create_new_node(self, token: T) {
        let next_node_index = self.automaton.edges.get(&(self.index, token));
        let next_node_index = match next_node_index {
            Some(n) => *n,
            None => {
                let loop_fn = self.automaton.nodes[self.index].loop_fn;
                if loop_fn.is_none() {
                    let last_node_index = self.automaton.nodes.len();
                    self.automaton.nodes.push(Node::default());
                    self.automaton
                        .edges
                        .insert((self.index, token), last_node_index);
                    last_node_index
                } else {
                    loop_fn.unwrap()(&mut self.state, token);
                    self.index
                }
            }
        };
        self.index = next_node_index;
    }

    pub fn next(self, token: T) {
        let next_node_index = self.automaton.edges.get(&(self.index, token));
        let next_node_index = match next_node_index {
            Some(n) => *n,
            None => {
                let loop_fn = self.automaton.nodes[self.index].loop_fn;
                if loop_fn.is_none() {
                    let last_node_index = self.automaton.nodes.len();
                    self.automaton.nodes.push(Node::default());
                    self.automaton
                        .edges
                        .insert((self.index, token), last_node_index);
                    last_node_index
                } else {
                    loop_fn.unwrap()(&mut self.state, token);
                    self.index
                }
            }
        };
        self.index = next_node_index;
    }

    /// Change the current node.
    pub fn change_node<F>(self, f: F)
    where
        F: Fn(&mut N),
    {
        f(&mut self.automaton.nodes[self.index].inner)
    }

    /// Set the loop function.
    pub fn set_loop<F>(self, f: F)
    where
        F: Fn(&mut S, T),
    {
        self.automaton.nodes[self.index].loop_fn = Some(Box::new(f))
    }
}