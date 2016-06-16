#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowsCount(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnsCount(pub usize);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowLength(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnLength(pub usize);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RowIndex(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ColumnIndex(pub usize);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Width(pub usize);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Height(pub usize);
