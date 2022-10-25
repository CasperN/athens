#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;

// TODO: use a persistent database or something.
type Data = Arc<Mutex<String>>;

const STORAGE: &'static str = &"data.txt";

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
