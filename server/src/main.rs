#[macro_use]
extern crate rocket;

use rocket::response::content::RawHtml;
use rocket::fs::FileServer;

#[get("/")]
fn index() -> RawHtml<&'static str> {
    RawHtml(r#"
        <h1> Hello world! </h1>
        <script type="module">
        import init from './public/frontend.js';
        await init('./public/frontend_bg.wasm');
        </script>
    "#)
}

#[post("/tasks", format = "application/json", data = "<tasks>")]
fn save_tasks(tasks: &str) {
    dbg!(tasks);

}



#[launch]
fn build() -> rocket::Rocket<rocket::Build> {
    rocket::build()
        .mount("/", routes![index, save_tasks])
        .mount("/public", FileServer::from("./static"))
}
