use std::{collections::HashMap, future::Future};

use hyper::{Body, Method, Request};
use regex::{Regex, RegexSet};

use crate::{app::App, HTTPResult};

pub trait Handler<'a, F: Future<Output = HTTPResult<'a>>> =
    Fn(App, Request<Body>, HashMap<String, String>) -> F;

pub struct Router {
    app: App,
    path_regex_set: RegexSet,
    routers: Vec<(Method, usize)>,
}

impl Router {
    pub fn route(&self, method: &Method, path: &str) {}
}

pub struct RouterBuilder<'a, F> {
    app: App,
    routers: Vec<(Method, String, Box<dyn Handler<'a, F>>)>,
}

impl<'a, F> RouterBuilder<'a, F> {
    /// Create a new router.
    pub fn new(app: App) -> Self {
        Self { app, routers: vec![] }
    }

    /// Register a route by its HTTP method and path.
    pub fn register<T>(mut self, method: &str, path: &str, handler: T) -> Self
    where
        T: Handler<'a, F>,
    {
        let token = Regex::new("<(?P<token>[a-zA-Z]*)>").unwrap();
        let path = format!("^{}$", token.replace_all(path, "(?P<$token>.*)"));

        self.routers.push((method.parse().unwrap(), path));
        self
    }

    pub fn build(self) -> Router {
        let pathes: Vec<_> = self.routers.iter().map(|(_, path)| path).collect();
        Router {
            app: self.app,
            path_regex_set: RegexSet::new(&pathes).unwrap(),
            routers: self
                .routers
                .iter()
                .enumerate()
                .map(|(offset, (method, _))| (method.to_owned(), offset))
                .collect(),
        }
    }
}
