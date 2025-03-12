use super::column::Column;

/// schema 代表有序的列集合
pub struct Schema {
    pub columns: Box<[Column]>,
}

impl Schema {
    pub fn new(columns: Vec<Column>) -> Self {
        Schema {
            columns: columns.into_boxed_slice(),
        }
    }

    pub fn column(&self, index: usize) -> &Column {
        &self.columns[index]
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|column| column.name() == name)
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn column_iter(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }
}
