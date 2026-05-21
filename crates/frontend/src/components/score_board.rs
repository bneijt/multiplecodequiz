use leptos::prelude::*;

#[component]
pub fn ScoreBoard(
    score: Signal<usize>,
    total: usize,
    current: Signal<usize>,
) -> impl IntoView {
    view! {
        <div style="display:flex;gap:2rem;align-items:center;border-bottom:1px solid #eee;padding-bottom:0.5rem;margin-bottom:1rem">
            <span>"Question: " {move || current.get() + 1} " / " {total}</span>
            <span>"Score: " {move || score.get()} " / " {move || current.get()}</span>
        </div>
    }
}
