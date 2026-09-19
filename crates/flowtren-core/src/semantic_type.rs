#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Integer,
    Decimal,
    Currency,
    Percentage,
    Category,
    Boolean,
    Date,
    DateTime,
    Email,
    Phone,
    Identifier,
    Text,
    Unknown,
}