use gloo_net::http::Request;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{QuizCard, ScoreBoard};
use crate::quiz::shuffle_answers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizItem {
    pub code: String,
    pub correct: String,
    pub distractors: [String; 3],
}

async fn load_quiz_data() -> Vec<QuizItem> {
    let resp = Request::get("/public/quiz_data.json")
        .send()
        .await
        .expect("Failed to fetch quiz_data.json");
    resp.json::<Vec<QuizItem>>()
        .await
        .expect("Failed to parse quiz_data.json")
}

#[component]
pub fn App() -> impl IntoView {
    let data = LocalResource::new(|| load_quiz_data());

    view! {
        <div class="pure-g" style="max-width:900px;margin:0 auto;padding:1rem">
            <div class="pure-u-1">
                <h1>"Code Quiz"</h1>
                <Suspense fallback=|| view! { <p>"Loading quiz..."</p> }>
                    {move || {
                        data.get().map(|items| {
                            let items: Vec<QuizItem> = (*items).clone();
                            view! { <Quiz items=items /> }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn Quiz(items: Vec<QuizItem>) -> impl IntoView {
    let total = items.len();
    let current = RwSignal::new(0usize);
    let score = RwSignal::new(0usize);
    let answered = RwSignal::new(false);
    let selected = RwSignal::new(Option::<usize>::None);

    // Pre-shuffle answers for each question at component creation time.
    // Each entry is (answers: Vec<String>, correct_index: usize)
    let shuffled: Vec<(Vec<String>, usize)> = items
        .iter()
        .map(|item| {
            let all = std::iter::once(item.correct.clone())
                .chain(item.distractors.iter().cloned())
                .collect::<Vec<_>>();
            shuffle_answers(all)
        })
        .collect();

    let shuffled = StoredValue::new_local(shuffled);
    let items = StoredValue::new_local(items);

    let on_answer = move |idx: usize| {
        if answered.get() {
            return;
        }
        answered.set(true);
        selected.set(Some(idx));
        let q = current.get();
        let (_, correct_idx) = shuffled.with_value(|s| (s[q].0.clone(), s[q].1));
        if idx == correct_idx {
            score.update(|s| *s += 1);
        }
    };

    let on_next = move |_| {
        current.update(|c| *c += 1);
        answered.set(false);
        selected.set(None);
    };

    view! {
        <ScoreBoard score=score.into() total=total current=current.into() />
        {move || {
            let q = current.get();
            if q >= total {
                view! {
                    <div>
                        <h2>"Quiz complete!"</h2>
                        <p>"Final score: " {score.get()} " / " {total}</p>
                    </div>
                }.into_any()
            } else {
                let code = items.with_value(|it| it[q].code.clone());
                let (answers, correct_idx) = shuffled.with_value(|s| s[q].clone());
                view! {
                    <QuizCard
                        code=code
                        answers=answers
                        correct_idx=correct_idx
                        answered=answered.into()
                        selected=selected.into()
                        on_answer=on_answer
                    />
                    <div style="margin-top:1rem">
                        {move || {
                            if answered.get() && current.get() + 1 < total {
                                view! {
                                    <button class="pure-button pure-button-primary" on:click=on_next>
                                        "Next"
                                    </button>
                                }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }
                        }}
                    </div>
                }.into_any()
            }
        }}
    }
}
