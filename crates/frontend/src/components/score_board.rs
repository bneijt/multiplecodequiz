use leptos::prelude::*;

#[component]
pub fn ScoreBoard(score: Signal<usize>, total: usize, current: Signal<usize>) -> impl IntoView {
    view! {
        <div class="grid">
            <div>"Question: " {move || current.get() + 1} " / " {total}</div>
            <div>"Score: " {move || score.get()} " / " {move || current.get()}</div>
        </div>
    }
}
