use super::leaf::Leaf;

pub enum Identifier {
    SingleIdentifier(SingleIdentifier),
    CombinedIdentifier(CombinedIdentifier),
}

pub struct SingleIdentifier {
    pub value: String,
    pub leaf: Leaf,
}

/// CombinedIdentifier t0.a
pub struct CombinedIdentifier {
    pub values: Vec<SingleIdentifier>,
}
