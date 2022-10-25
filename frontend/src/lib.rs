#![feature(async_closure)]
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;


mod model;
use model::Model;

struct Entry {
    editing: bool,
}

#[derive(Clone, Copy)]
enum EntryM {
    StartEditing,
    StopEditing,
}

#[derive(PartialEq, Properties)]
struct EntryP {
    id: usize,
    text: String, // Should this just be shared?
    set_text: Callback<String>,
}
impl Component for Entry {
    type Properties = EntryP;
    type Message = EntryM;

    fn create(_ctx: &Context<Self>) -> Self {
        Entry { editing: false }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: EntryM) -> bool {
        match msg {
            EntryM::StartEditing => self.editing = true,
            EntryM::StopEditing => self.editing = false,
        };
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let start_editing = ctx.link().callback(|_| EntryM::StartEditing);
        let stop_editing = ctx.link().callback(|_| EntryM::StopEditing);
        let stop_editing_on_enter = {
            let link = ctx.link().clone();
            Callback::from(move |e: KeyboardEvent| {
                const ENTER_KEY_CODE: u32 = 13;
                if e.key_code() == ENTER_KEY_CODE {
                    link.send_message(EntryM::StopEditing);
                }
            })
        };
        let emit_text = {
            let set_text = ctx.props().set_text.clone();
            Callback::from(move |e: InputEvent| {
                let t: HtmlTextAreaElement = e.target_unchecked_into();
                set_text.emit(t.value())
            })
        };

        html! {
            <input
                type="text"
                size="80"
                disabled={!self.editing}
                onclick={start_editing}
                onfocusout={stop_editing}
                value={ctx.props().text.clone()}
                onkeypress={stop_editing_on_enter}
                oninput={emit_text}
            />
        }
    }
}

#[derive(Default)]
struct List {
    model: Model,
    dragged: Option<usize>,
    dragged_over: Option<usize>,
}

#[derive(Clone)]
enum ListM {
    // Entry CRUD
    AddEntry,
    SetEntryText(usize, String),
    // Drag and drop.
    SetDragged(Option<usize>),
    SetDraggedOver(Option<usize>),
    Dropped,
    // Saving.
    StartSaving,
    LoadData(Model),
    Ignore, // Do thing, basically means "None"
}

impl List {
    fn save_request(&self) -> Request {
        Request::post("/tasks")
            .json(&self.model)
            .expect("Failed to make request")
    }
}

async fn load_tasks() -> Option<Model> {
    Request::get("/tasks")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?
}

impl Component for List {
    type Message = ListM;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            if let Some(tasks) = load_tasks().await {
                ListM::LoadData(tasks)
            } else {
                ListM::Ignore
            }
        });
        Self::default()
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ListM::Ignore => false,
            ListM::AddEntry => {
                self.model.add_entry();
                true
            }
            ListM::SetDragged(i) => {
                self.dragged = i;
                false
            }
            ListM::SetDraggedOver(x) => {
                self.dragged_over = x;
                false
            }
            ListM::Dropped => {
                if let (Some(from), Some(to)) = (self.dragged, self.dragged_over) {
                    self.model.move_entries(from, to);
                    self.dragged = Some(from);
                    true
                } else {
                    // dragged and dragover should both be set by web events before
                    // dropped is called and before dragexit.
                    log::error!(
                        "Bad drag and drop: from:{:?} to:{:?}",
                        self.dragged,
                        self.dragged_over
                    );
                    false
                }
            }
            ListM::SetEntryText(i, text) => {
                self.model.set_text(i, text);
                true
            }
            ListM::StartSaving => {
                let rq = self.save_request();
                ctx.link().send_future(async {
                    let rp = rq.send().await;
                    log::info!("Response: {:?}", rp);
                    ListM::Ignore
                });
                false
            }
            ListM::LoadData(model) => {
                self.model = model;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let entries: Vec<Html> = self
            .model
            .iter_entries()
            .into_iter()
            .map(|(order, id, entry)| {
                let set_dragged = |d| ctx.link().callback(move |_| ListM::SetDragged(d));
                let set_dragged_over = |d| ctx.link().callback(move |_| ListM::SetDraggedOver(d));
                let drop = ctx.link().callback(|_| ListM::Dropped);
                let set_entry_cb = ctx.link().callback(move |s| ListM::SetEntryText(id, s));

                html! {
                    <li draggable="true"
                        ondragstart={set_dragged(Some(order))}
                        ondragend={set_dragged(None)}
                        ondragover={ctx.link().callback(move |e: DragEvent| {
                            e.prevent_default();  // Neccessary for ondrop to be called.
                            ListM::SetDraggedOver(Some(order))
                        })}
                        ondragleave={set_dragged_over(None)}
                        ondrop={drop}
                    >
                        <Entry
                            id={id}
                            text={entry.clone()}
                            set_text={set_entry_cb}
                        />
                    </li>
                }
            })
            .collect();

        let addentry = ctx.link().callback(|_| ListM::AddEntry);
        let save = ctx.link().callback(|_| ListM::StartSaving);

        html! {
            <div>
                <ul>{ for entries }</ul>
                <button onclick={addentry}>{"Add"}</button>
                <button onclick={save}>{"Save"}</button>
            </div>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
        <link href="public/style.css" rel="stylesheet"/>
        <h1>{"Lists!"}</h1>
        <List/>
        </>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Starting app");
    yew::start_app::<App>();
    Ok(())
}
