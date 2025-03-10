use std::{fmt::Display, rc::Rc};

use super::leaf::Leaf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Identifier {
    Single(SingleIdentifier),
    Combined(Box<[SingleIdentifier]>),
    WithWildcard(Box<[SingleIdentifier]>),
    Wildcard(Leaf),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleIdentifier {
    pub value: Rc<str>,
    pub leaf: Leaf,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Single(ident) => write!(f, "{}", ident.value),
            Identifier::Combined(identifiers) => {
                for (index, ident) in identifiers.iter().enumerate() {
                    if index > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{}", ident.value)?;
                }
                Ok(())
            }
            Identifier::WithWildcard(identifiers) => {
                for (index, ident) in identifiers.iter().enumerate() {
                    if index > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{}", ident.value)?;
                }
                write!(f, ".*")
            }
            Identifier::Wildcard(_) => write!(f, "*"),
        }
    }
}
