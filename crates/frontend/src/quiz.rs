/// Shuffle a vec of answers, returning (shuffled_vec, new_correct_index).
/// The correct answer is always originally at index 0 from app.rs.
pub fn shuffle_answers(answers: Vec<String>) -> (Vec<String>, usize) {
    // Simple deterministic-ish Fisher-Yates using a hash of the first answer
    // as a seed, so order is fixed per question but not obviously sorted.
    let seed = answers[0]
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    let n = answers.len();
    let mut indices: Vec<usize> = (0..n).collect();

    // Shuffle indices
    let mut rng = seed;
    for i in (1..n).rev() {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }

    // Find where the original correct answer (index 0) ended up
    let correct_idx = indices.iter().position(|&x| x == 0).unwrap();

    let shuffled = indices.into_iter().map(|i| answers[i].clone()).collect();
    (shuffled, correct_idx)
}
