#![feature(async_closure)]
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

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

struct List {
    entries: Vec<EntryData>,
    dragged: Option<usize>,
    dragged_over: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
struct EntryData(String);

#[derive(Clone)]
enum ListM {
    AddEntry,
    LoadData(Vec<EntryData>),
    SetDragged(Option<usize>),
    SetDraggedOver(Option<usize>),
    Dropped,
    SetEntryText(usize, String),
    StartSaving,
    SaveResult,
    Ignore, // Do thing, basically means "None"
}

impl List {
    fn move_entries(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }

        let entry = self.entries.remove(from);
        self.entries.insert(to, entry);
    }

    fn save_request(&self) -> Request {
        Request::post("/tasks")
            .json(&self.entries)
            .expect("Failed to make request")
    }
}

async fn load_tasks() -> Option<Vec<EntryData>> {
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

        Self {
            entries: vec![],
            dragged: None,
            dragged_over: None,
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ListM::Ignore => false,
            ListM::AddEntry => {
                let e = format!("entry: {:?}", self.entries.len());
                self.entries.push(EntryData(e));
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
                    self.move_entries(from, to);
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
                self.entries[i].0 = text;
                true
            }
            ListM::StartSaving => {
                let rq = self.save_request();
                ctx.link().send_future(async {
                    let rp = rq.send().await;
                    log::info!("Response: {:?}", rp);
                    ListM::SaveResult
                });
                false
            }
            ListM::SaveResult => false,
            ListM::LoadData(entries) => {
                self.entries = entries;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let entries: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let set_dragged = |d| ctx.link().callback(move |_| ListM::SetDragged(d));
                let set_dragged_over = |d| ctx.link().callback(move |_| ListM::SetDraggedOver(d));
                let drop = ctx.link().callback(|_e: DragEvent| ListM::Dropped);
                let set_entry_cb = ctx.link().callback(move |s| ListM::SetEntryText(i, s));

                html! {
                    <li draggable="true"
                        ondragstart={set_dragged(Some(i))}
                        ondragend={set_dragged(None)}
                        ondragover={ctx.link().callback(move |e: DragEvent| {
                            e.prevent_default();  // Neccessary for ondrop to be called.
                            ListM::SetDraggedOver(Some(i))
                        })}
                        ondragleave={set_dragged_over(None)}
                        ondrop={drop}
                    >
                        <Entry
                            id={i}
                            text={entry.0.clone()}
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
    log::info!("app running");

    let color = use_state(|| "red");
    let cb = {
        let color = color.clone();
        Callback::from(move |_| {
            log::info!("changing color!");
            let next_color = match *color {
                "red" => "green",
                "green" => "blue",
                "blue" => "cyan",
                "cyan" => "magenta",
                _ => "red",
            };
            color.set(next_color)
        })
    };

    let style = format!("background-color:{};", *color);
    html! {
        <>
        <h1 style={style}>
            { "Hello world!" }
        <button onclick={ cb.clone() }>{ "Color me!" }</button>
        </h1>
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
