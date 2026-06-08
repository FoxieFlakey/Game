use std::fmt::Display;

pub const GAME_ID: &str = "foxiefox";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl Identifier {
    pub(crate) fn new<S: Into<String>>(path: S) -> Self {
        Self {
            namespace: GAME_ID.to_string(),
            path: path.into(),
        }
    }
}
