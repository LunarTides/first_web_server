use lazy_static::lazy_static;
use rocket::{
    http::Status,
    serde::{json::Json, Serialize},
};
use std::sync::Mutex;

#[macro_use]
extern crate rocket;

#[deny(clippy::all)]
#[derive(Clone, Serialize)]
#[serde(crate = "rocket::serde")]
struct Todo {
    id: usize,
    description: String,
    completed: bool,
}

impl Todo {
    fn new(description: String) -> Todo {
        let mut todos_lock = match TODOS_COUNT.lock() {
            Ok(count) => count,
            Err(error) => {
                panic!("{}", error);
            }
        };

        *todos_lock += 1;

        Todo {
            id: *todos_lock,
            description,
            completed: false,
        }
    }

    fn complete(&mut self) {
        self.completed = true;
    }

    fn uncomplete(&mut self) {
        self.completed = false;
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Response {
    success: bool,
    todo: Option<Todo>,
    error: Option<String>,
}

lazy_static! {
    static ref TODOS: Mutex<Vec<Todo>> = Mutex::new(vec![]);
    static ref TODOS_COUNT: Mutex<usize> = Mutex::new(0);
}

async fn find_todo<T>(todo_id: usize, callback: impl Fn(&mut Todo) -> T) -> (Status, Result<T, String>) {
    let mut todos_lock = match TODOS.lock() {
        Ok(todos) => todos,
        Err(error) => {
            return (Status::InternalServerError, Err(error.to_string()));
        }
    };

    let todo = match todos_lock.iter_mut().find(|todo| todo.id == todo_id) {
        Some(todo) => todo,
        None => {
            return (Status::BadRequest, Err(String::from("not found")));
        }
    };

    (Status::Ok, Ok(callback(todo)))
}

async fn todo_response(
    todo_id: usize,
    callback: impl Fn(&mut Todo),
) -> (Status, Result<Json<Response>, Json<Response>>) {
    let response = find_todo(todo_id, |todo| {
        callback(todo);

        Response {
            success: true,
            todo: Some(todo.clone()),
            error: None,
        }
    });

    let (status, response) = response.await;

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let response = Response {
                success: false,
                todo: None,
                error: Some(error),
            };

            return (status, Err(Json(response)));
        }
    };

    (status, Ok(Json(response)))
}

#[post("/todo/create", data = "<todo_description>")]
async fn add_todo(todo_description: String) -> (Status, Result<Json<Response>, Json<Response>>) {
    let mut todos_lock = match TODOS.lock() {
        Ok(todos) => todos,
        Err(error) => {
            let response = Response {
                success: false,
                todo: None,
                error: Some(error.to_string()),
            };

            return (Status::InternalServerError, Err(Json(response)));
        }
    };

    if todos_lock
        .iter()
        .any(|todo| todo.description == todo_description)
    {
        let response = Response {
            success: false,
            todo: None,
            error: Some(String::from("already exists")),
        };

        return (Status::BadRequest, Err(Json(response)));
    }

    let todo = Todo::new(todo_description);
    todos_lock.push(todo.clone());

    let response = Response {
        success: true,
        todo: Some(todo),
        error: None,
    };

    (Status::Created, Ok(Json(response)))
}

#[get("/todo/<todo_id>/get")]
async fn get_todo(todo_id: usize) -> (Status, Result<Json<Response>, Json<Response>>) {
    todo_response(todo_id, |_| {}).await
}

#[get("/todo/<todo_id>/complete")]
async fn complete_todo(todo_id: usize) -> (Status, Result<Json<Response>, Json<Response>>) {
    todo_response(todo_id, |todo| todo.complete()).await
}

#[get("/todo/<todo_id>/uncomplete")]
async fn uncomplete_todo(todo_id: usize) -> (Status, Result<Json<Response>, Json<Response>>) {
    todo_response(todo_id, |todo| todo.uncomplete()).await
}

#[launch]
fn launch() -> _ {
    rocket::build().mount(
        "/api/v1",
        routes![add_todo, get_todo, complete_todo, uncomplete_todo],
    )
}
