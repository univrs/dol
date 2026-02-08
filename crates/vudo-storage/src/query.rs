//! Query filter types for indexed lookups.

use serde::{Deserialize, Serialize};

/// Query filter for document lookups.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QueryFilter {
    /// Match all documents.
    All,

    /// Match documents updated after a timestamp.
    UpdatedAfter(u64),

    /// Match documents updated before a timestamp.
    UpdatedBefore(u64),

    /// Match documents within a time range.
    UpdatedBetween { start: u64, end: u64 },

    /// Match documents by custom field (if supported by adapter).
    ///
    /// This requires the adapter to maintain indexes on custom fields.
    /// Not all adapters may support this.
    Field {
        /// Field name.
        field: String,
        /// Expected value (JSON string).
        value: String,
    },

    /// Combine multiple filters with AND logic.
    And(Vec<QueryFilter>),

    /// Combine multiple filters with OR logic.
    Or(Vec<QueryFilter>),

    /// Negate a filter.
    Not(Box<QueryFilter>),
}

impl QueryFilter {
    /// Create a filter for documents updated after a timestamp.
    pub fn updated_after(timestamp: u64) -> Self {
        Self::UpdatedAfter(timestamp)
    }

    /// Create a filter for documents updated before a timestamp.
    pub fn updated_before(timestamp: u64) -> Self {
        Self::UpdatedBefore(timestamp)
    }

    /// Create a filter for documents updated within a time range.
    pub fn updated_between(start: u64, end: u64) -> Self {
        Self::UpdatedBetween { start, end }
    }

    /// Create a filter for a custom field.
    pub fn field(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Field {
            field: field.into(),
            value: value.into(),
        }
    }

    /// Combine this filter with another using AND logic.
    pub fn and(self, other: QueryFilter) -> Self {
        match self {
            Self::And(mut filters) => {
                filters.push(other);
                Self::And(filters)
            }
            _ => Self::And(vec![self, other]),
        }
    }

    /// Combine this filter with another using OR logic.
    pub fn or(self, other: QueryFilter) -> Self {
        match self {
            Self::Or(mut filters) => {
                filters.push(other);
                Self::Or(filters)
            }
            _ => Self::Or(vec![self, other]),
        }
    }

    /// Negate this filter.
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        Self::Not(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_filter_all() {
        let filter = QueryFilter::All;
        assert!(matches!(filter, QueryFilter::All));
    }

    #[test]
    fn test_query_filter_updated_after() {
        let filter = QueryFilter::updated_after(1000);
        assert_eq!(filter, QueryFilter::UpdatedAfter(1000));
    }

    #[test]
    fn test_query_filter_updated_before() {
        let filter = QueryFilter::updated_before(2000);
        assert_eq!(filter, QueryFilter::UpdatedBefore(2000));
    }

    #[test]
    fn test_query_filter_updated_between() {
        let filter = QueryFilter::updated_between(1000, 2000);
        assert_eq!(
            filter,
            QueryFilter::UpdatedBetween {
                start: 1000,
                end: 2000
            }
        );
    }

    #[test]
    fn test_query_filter_field() {
        let filter = QueryFilter::field("status", "active");
        assert_eq!(
            filter,
            QueryFilter::Field {
                field: "status".to_string(),
                value: "active".to_string()
            }
        );
    }

    #[test]
    fn test_query_filter_and() {
        let filter1 = QueryFilter::updated_after(1000);
        let filter2 = QueryFilter::field("status", "active");
        let combined = filter1.and(filter2.clone());

        if let QueryFilter::And(filters) = combined {
            assert_eq!(filters.len(), 2);
        } else {
            panic!("Expected And filter");
        }
    }

    #[test]
    fn test_query_filter_or() {
        let filter1 = QueryFilter::updated_after(1000);
        let filter2 = QueryFilter::field("status", "active");
        let combined = filter1.or(filter2.clone());

        if let QueryFilter::Or(filters) = combined {
            assert_eq!(filters.len(), 2);
        } else {
            panic!("Expected Or filter");
        }
    }

    #[test]
    fn test_query_filter_not() {
        let filter = QueryFilter::field("status", "inactive").not();

        if let QueryFilter::Not(inner) = filter {
            assert_eq!(
                *inner,
                QueryFilter::Field {
                    field: "status".to_string(),
                    value: "inactive".to_string()
                }
            );
        } else {
            panic!("Expected Not filter");
        }
    }

    #[test]
    fn test_query_filter_serialization() {
        let filter = QueryFilter::updated_after(1000);
        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: QueryFilter = serde_json::from_str(&json).unwrap();
        assert_eq!(filter, deserialized);
    }

    #[test]
    fn test_query_filter_complex() {
        let filter = QueryFilter::updated_after(1000)
            .and(QueryFilter::field("status", "active"))
            .or(QueryFilter::field("priority", "high"));

        // Should create nested structure
        if let QueryFilter::Or(_) = filter {
            // OK
        } else {
            panic!("Expected Or at top level");
        }
    }
}
