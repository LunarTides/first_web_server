use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq, Deserialize)]
struct Todo {
    id: usize,
    description: String,
    completed: bool,
}

#[derive(Deserialize)]
struct APIResponse {
    success: bool,
    todo: Option<Todo>,
    error: Option<String>,
}

#[function_component(TodoComponent)]
fn TodoComp(props: &Todo) -> Html {
    html! {
        <div>
            <input type="checkbox" checked={props.completed} />
            <span>{ &props.description }</span>
        </div>
    }
}

#[function_component]
fn App() -> Html {
    let todos: UseStateHandle<Vec<Todo>> = use_state(Vec::new);
    let description = use_state(|| String::new());

    let add_todo = |action: MouseEvent| {
        wasm_bindgen_futures::spawn_local(async move {
            Request::post("127.0.0.1:8000/api/v1/todo/create")
                .body(serde_json::from_str(&description.as_str()).unwrap_or(""))
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap()
        });
    };

    let update_description = |value: InputEvent| {
        let description = description.clone();
        description.set(value.data().unwrap_or(String::new()));
    };

    html! {
        <>
            <div>
                <h1>{ "Todo List" }</h1>
            </div>
            <div>
                <input type="text" oninput={ update_description } />
                <button onclick={ add_todo }>{ "Add Todo" }</button>
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
