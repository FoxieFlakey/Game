use std::{borrow::Cow, fmt::Display};

pub const GAME_ID: &str = "foxiefox";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Identifier {
    pub namespace: Cow<'static, str>,
    pub path: Cow<'static, str>,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl Identifier {
    pub(crate) fn new<S: Into<String>>(path: S) -> Self {
        Self {
            namespace: Cow::Borrowed(GAME_ID),
            path: Cow::Owned(path.into()),
        }
    }

    pub(crate) const fn new_const(path: &'static str) -> Self {
        Self {
            namespace: Cow::Borrowed(GAME_ID),
            path: Cow::Borrowed(path),
        }
    }
}
