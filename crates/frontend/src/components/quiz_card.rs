use leptos::prelude::*;

#[component]
pub fn QuizCard(
    code: String,
    answers: Vec<String>,
    correct_idx: usize,
    answered: Signal<bool>,
    selected: Signal<Option<usize>>,
    on_answer: impl Fn(usize) + Copy + Send + 'static,
) -> impl IntoView {
    let answers = StoredValue::new_local(answers);

    let buttons: Vec<_> = (0..4)
        .map(|i| {
            let answer = answers.with_value(|a| a.get(i).cloned().unwrap_or_default());
            let style = move || {
                if !answered.get() {
                    "margin:0.25rem 0;width:100%".to_string()
                } else if i == correct_idx {
                    "margin:0.25rem 0;width:100%;background:#28a745;color:white".to_string()
                } else if selected.get() == Some(i) {
                    "margin:0.25rem 0;width:100%;background:#dc3545;color:white".to_string()
                } else {
                    "margin:0.25rem 0;width:100%;opacity:0.6".to_string()
                }
            };
            view! {
                <div style="padding:0.1rem 0">
                    <button
                        style=style
                        on:click=move |_| on_answer(i)
                    >
                        {answer}
                    </button>
                </div>
            }
        })
        .collect();

    view! {
        <div class="grid">
            <div><article>
            <pre><code>{code}</code></pre>
            </article>
            </div>
            <div>
                <p style="margin-top:0"><strong>"What does this code do?"</strong></p>
                {buttons}
            </div>
        </div>
    }
}
