use std::{collections::HashMap};

use hyper::Method;

use crate::app::App;

mod automaton;

// pub trait Handler<'a, F: Future<Output = HTTPResult<'a>>> =
//     Fn(App, Request<Body>, HashMap<String, String>) -> F;

#[derive(Hash, PartialEq, Eq)]
enum InputToken {
    Char(char),
    Method(Method),
}

#[derive(Default)]
struct AutomatonState {
    params: HashMap<String, String>,
}

pub struct Router<H> {
    app: App,
    automaton: automaton::Automaton<AutomatonState, Option<H>, InputToken>,
}

impl<H> Router<H> {
    pub fn route(&self, method: &Method, path: &str) {
        let automaton_iter = self.automaton.iter();
    }
}

pub struct Builder<H> {
    app: App,
    routes: Vec<(Route, H)>,
}

impl<H> Builder<H> {
    /// Create a new router.
    pub fn new(app: App) -> Self {
        Self {
            app,
            routes: vec![],
        }
    }

    /// Register a route by its HTTP method and path.
    pub fn register(mut self, route: Route, handler: H) -> Self {
        self.routes.push((route, handler));
        self
    }

    pub fn build(self) -> Result<Router<H>, RouterBuilderError> {
        let result = Router {
            app: self.app,
            automaton: automaton::Automaton::new(),
        };
        for (route, handler) in self.routes {
            enum State {
                Init,
                ReadingParameter,
            };
            let state = State::Init;
            let param_name = String::new();

            let mut automaton_iter = result.automaton.iter();
            for c in route.path.chars() {
                let input_token = InputToken::Char(c);
                match state {
                    State::Init => match c {
                        '<' => state = State::ReadingParameter,
                        _ => automaton_iter.next_or_create_new_node(input_token),
                    },
                    State::ReadingParameter => match c {
                        '>' => {
                            state = State::Init;
                            automaton_iter.set_loop(|state, c| {
                                let param = state.params.get_mut(&param_name);
                                let param = if param.is_none() {
                                    state.params.insert(param_name, String::new());
                                    state.params.get_mut(&param_name).unwrap()
                                } else {
                                    param.unwrap()
                                };
                                if let InputToken::Char(c) = c {
                                    param.push(c)
                                }
                            });
                            automaton_iter.next_or_create_new_node(InputToken::Char('/'));
                            param_name.clear();
                        }
                        _ => param_name.push(c),
                    },
                }
            }
            automaton_iter.next_or_create_new_node(InputToken::Method(route.method));
            automaton_iter.change_node(|node| *node = Some(handler));
        }
        Ok(result)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RouterBuilderError {
    #[error("invalid path")]
    InvalidPath,
}

pub struct Route {
    method: Method,
    path: String,
}

impl Route {
    pub fn get(path: &str) -> Self {
        Self::from(Method::GET, path)
    }

    pub fn post(path: &str) -> Self {
        Self::from(Method::POST, path)
    }

    pub fn put(path: &str) -> Self {
        Self::from(Method::PUT, path)
    }

    pub fn delete(path: &str) -> Self {
        Self::from(Method::DELETE, path)
    }

    pub fn from(method: Method, path: &str) -> Self {
        Self {
            method,
            path: path.to_string(),
        }
    }
}
