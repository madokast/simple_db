use super::leaf::Leaf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Identifier {
    SingleIdentifier(SingleIdentifier),
    CombinedIdentifier(CombinedIdentifier),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SingleIdentifier {
    pub value: String,
    pub leaf: Leaf,
}

/// CombinedIdentifier t0.a
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CombinedIdentifier {
    pub values: Vec<SingleIdentifier>,
}
