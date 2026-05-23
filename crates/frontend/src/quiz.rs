use crate::app::QuizItem;

/// Fisher-Yates shuffle in place using a deterministic LCG seeded by `seed`.
fn shuffle_in_place<T>(items: &mut [T], seed: u64) {
    let n = items.len();
    let mut rng = seed;
    for i in (1..n).rev() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        items.swap(i, j);
    }
}

/// Shuffle a list of quiz items using Fisher-Yates with the given seed.
pub fn shuffle_items(mut items: Vec<QuizItem>, seed: u64) -> Vec<QuizItem> {
    shuffle_in_place(&mut items, seed);
    items
}

/// Shuffle a vec of answers, returning (shuffled_vec, new_correct_index).
/// The correct answer is always originally at index 0 from app.rs.
pub fn shuffle_answers(answers: Vec<String>) -> (Vec<String>, usize) {
    // Deterministic seed from the first answer so order is fixed per question.
    let seed = answers[0]
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    let mut indices: Vec<usize> = (0..answers.len()).collect();
    shuffle_in_place(&mut indices, seed);

    let correct_idx = indices.iter().position(|&x| x == 0).unwrap();
    let shuffled = indices.into_iter().map(|i| answers[i].clone()).collect();
    (shuffled, correct_idx)
}
