/// Basic domain model for a piece of indexed content.
#[derive(Debug, Clone)]
pub struct Document<'a> {
    pub id: &'a str,
    pub score: i32,
}

/// Aggregates document scores so higher-level services can act on summary data.
pub fn sum_scores<'a>(docs: impl IntoIterator<Item = &'a Document<'a>>) -> i32 {
    docs.into_iter().map(|doc| doc.score).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sums_scores() {
        let docs = vec![
            Document { id: "a", score: 3 },
            Document { id: "b", score: -1 },
            Document { id: "c", score: 10 },
        ];
        assert_eq!(sum_scores(&docs), 12);
    }
}
