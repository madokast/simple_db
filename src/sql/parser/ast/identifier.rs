use std::{fmt::Display, rc::Rc};

use super::leaf::Leaf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Identifier {
    SingleIdentifier(SingleIdentifier),
    CombinedIdentifier(Box<[SingleIdentifier]>),
    IdentifierWithWildcard(Box<[SingleIdentifier]>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleIdentifier {
    pub value: Rc<str>,
    pub leaf: Leaf,
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::SingleIdentifier(ident) => write!(f, "{}", ident.value),
            Identifier::CombinedIdentifier(identifiers) => {
                for (index, ident) in identifiers.iter().enumerate() {
                    if index > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{}", ident.value)?;
                }
                Ok(())
            }
            Identifier::IdentifierWithWildcard(identifiers) => {
                for (index, ident) in identifiers.iter().enumerate() {
                    if index > 0 {
                        write!(f, ".")?;
                    }
                    write!(f, "{}", ident.value)?;
                }
                write!(f, ".*")
            }
        }
    }
}
