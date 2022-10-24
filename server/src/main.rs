#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket::response::content::RawHtml;
use rocket::State;
use std::sync::Arc;
use std::sync::Mutex;

// TODO: use a persistent database or something.
type Data = Arc<Mutex<String>>;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(
        r#"
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
}

#[get("/tasks", format = "application/json")]
fn get_tasks(db: &State<Data>) -> String {
    db.lock().unwrap().clone()
}

#[launch]
fn build() -> rocket::Rocket<rocket::Build> {
    let data = Data::default();
    rocket::build()
        .mount("/", routes![index, save_tasks, get_tasks])
        .mount("/public", FileServer::from("./static"))
        .manage(data)
}
