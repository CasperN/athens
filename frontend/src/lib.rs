use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

struct Entry;
#[derive(PartialEq, Properties)]
struct EntryP {
    id: usize,
    text: String,  // Should this just be shared?
    set_text: Callback<String>,
}
impl Component for Entry {
    type Properties = EntryP;
    type Message = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Entry
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <input
                type="text"
                oninput={
                    let set_text = ctx.props().set_text.clone();
                    Callback::from(move |e: InputEvent| {
                        let t: HtmlTextAreaElement = e.target_unchecked_into();
                        set_text.emit(t.value())
                    })
                }
            />
        }
    }
}

struct List2 {
    entries: Vec<EntryData>,
    dragged: Option<usize>,
    dragged_over: Option<usize>,
}

struct EntryData(String);

#[derive(Clone)]
enum List2M {
    AddEntry,
    SetDragged(Option<usize>),
    SetDraggedOver(Option<usize>),
    Dropped,
    SetEntryText(usize, String)
}

impl List2 {
    fn move_entries(&mut self, from: usize, to: usize) {
        let entry = self.entries.remove(from);
        self.entries.insert(to, entry);
    }
}

impl Component for List2 {
    type Message = List2M;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            entries: vec![],
            dragged: None,
            dragged_over: None,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            List2M::AddEntry => {
                let e = format!("entry: {:?}", self.entries.len());
                self.entries.push(EntryData(e));
                true
            }
            List2M::SetDragged(i) => {
                self.dragged = i;
                false
            }
            List2M::SetDraggedOver(x) => {
                self.dragged_over = x;
                false
            }
            List2M::Dropped => {
                if let (Some(from), Some(to)) = (self.dragged, self.dragged_over) {
                    self.move_entries(from, to);
                    true
                } else {
                    // dragged and dragover should both be set by web events before
                    // dropped is called and before dragexit.
                    log::error!("Bad drag and drop: from:{:?} to:{:?}", self.dragged, self.dragged_over);
                    false
                }
            }
            List2M::SetEntryText(i, text) => {
                self.entries[i].0 = text;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let addentry = ctx.link().callback(|_| List2M::AddEntry);
        let entries: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                // let set_being_dragged = ctx.link().callback(move |_| List2M::SetDragged(Some(i)));
                let set_dragged = |d| ctx.link().callback(move |_event: DragEvent| {
                    List2M::SetDragged(d)
                });
                let set_dragged_over = |d| ctx.link().callback(move |_event: DragEvent| {
                    List2M::SetDraggedOver(d)
                });

                html! {
                    <li draggable="true"
                        ondragstart={set_dragged(Some(i))}
                        ondragend={set_dragged(None)}
                        ondragover={ctx.link().callback(move |e: DragEvent| {
                            e.prevent_default();
                            List2M::SetDraggedOver(Some(i))
                        })}
                        ondragleave={set_dragged_over(None)}
                        ondrop={ctx.link().callback(|_e: DragEvent| {
                            List2M::Dropped
                        })}
                    > <Entry id={i} text={entry.0.clone()} set_text={
                        ctx.link().callback(move |s| List2M::SetEntryText(i, s)) }/>
                        {entry.0.clone()}

                    </li>
                }
            })
            .collect();
        html! {
            <div>
            <ul>
                { for entries }
            </ul>
            <button onclick={addentry}>{"Add"}</button>
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
        <List2/>
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
