#![feature(linked_list_cursors)]

#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;

// TODO: use a persistent database or something.

// GET  /space/{spaceid} -> Exists or Nah
// POST /space/{spaceid} -> Create space
//
// GET  /space/{spaceid}/user/{userid} -> User { alias, weight }
// POST /space/{spaceid}/user/{userid} -> update username
//
// GET  /space/{spaceid}/task/{taskid} -> Task { text, lifecycle }
// POST /space/{spaceid}/task/{taskid} -> update task
//
// -- If the taskid is not in the list, it is prepended in descending order
// GET  /space/{spaceid}/importance/{userid} -> Vec<TaskId>;
// POST /space/{spaceid}/importance/{userid} -> Vec<TaskId>;
//
// GET  /space/{spaceid}/blocking/{userid} -> Vec<(TaskId, TaskId)>
// POST /space/{spaceid}/blocking/{userid} -> Vec<(TaskId, TaskId)>
//
// GET  /space -> list of spaces (probably not the best idea for security)
// GET  /space/{spaceid}/task -> Vec<TaskId>
// GET  /space/{spaceid}/importance -> Aggregated importance ordering
// GET  /space/{spaceid}/easiness -> Aggregated easiness ordering
// GET  /space/{spaceid}/final -> Aggregated ordering considering everything.

mod model;

//
//
type Data = Arc<Mutex<String>>;

const STORAGE: &str = "data.txt";

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(
        r#"
        <style> * { background-color: black; } </style>
        <h1> Hello world! </h1>
        <script type="module">
        import init from './public/frontend.js';
        await init('./public/frontend_bg.wasm');
        </script>
    "#,
    )
}

#[post("/tasks", format = "application/json", data = "<tasks>")]
fn save_tasks(db: &State<Data>, tasks: &str) {
    *db.lock().unwrap() = tasks.to_string();
    std::fs::write(STORAGE, tasks).unwrap_or_else(|e| {
        log::error!("Failed to save tasks: {:?}", e);
    });
}

#[get("/tasks", format = "application/json")]
fn get_tasks(db: &State<Data>) -> String {
    let data = db.lock().unwrap().clone();
    if data.is_empty() {
        std::fs::read_to_string(STORAGE).unwrap_or_else(|e| {
            log::error!("Failed to save tasks: {:?}", e);
            String::new()
        })
    } else {
        data
    }
}

#[launch]
fn build() -> rocket::Rocket<rocket::Build> {
    let data = Data::default();
    rocket::build()
        .mount("/", routes![index, save_tasks, get_tasks])
        .mount("/public", FileServer::from("./static"))
        .manage(data)
}
