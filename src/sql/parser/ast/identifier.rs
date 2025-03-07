use std::rc::Rc;

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
