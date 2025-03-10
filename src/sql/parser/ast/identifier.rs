use std::{fmt::Display, rc::Rc};

use super::leaf::{Location, WithLocation};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Identifier {
    Single(SingleIdentifier),
    Combined(Box<[SingleIdentifier]>),
    WithWildcard(Box<[SingleIdentifier]>),
    Wildcard(Location),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleIdentifier {
    pub value: Rc<str>,
    pub leaf: Location,
}

impl WithLocation for SingleIdentifier {
    fn location(&self) -> &Location {
        &self.leaf
    }
}

impl WithLocation for Identifier {
    fn location(&self) -> &Location {
        match self {
            Identifier::Single(ident) => ident.location(),
            Identifier::Combined(identifiers) => identifiers[0].location(),
            Identifier::WithWildcard(identifiers) => identifiers[0].location(),
            Identifier::Wildcard(location) => location,
        }
    }
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
