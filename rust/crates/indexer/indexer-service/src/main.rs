use indexer_core::{sum_scores, Document};

fn main() {
    let seed_docs = vec![
        Document { id: "alpha", score: 5 },
        Document { id: "beta", score: 2 },
        Document { id: "gamma", score: -1 },
    ];

    let total = sum_scores(&seed_docs);
    println!(
        "Indexed {} docs -> aggregate score {}",
        seed_docs.len(),
        total
    );
}
