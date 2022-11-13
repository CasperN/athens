#![feature(async_closure)]
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;
use yew::context::ContextHandle;
use yew::prelude::*;
// use yew::virtual_dom::AttrValue;

use model::{AthensSpace, SimpleAthensSpace, TaskId, UserId};

// TODO: Should this be Box<dyn Athens> or Rc<dyn Athens>
// or something?
#[derive(Clone, Default)]
struct Athens {
    inner: model::ParallelSimpleAthensSpace,
}
impl PartialEq for Athens {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Athens {
    fn new() -> Self {
        Default::default()
    }
    fn get(&self) -> &dyn AthensSpace {
        &self.inner
    }
}

#[derive(PartialEq, Properties)]
struct EditableInputP {
    editable: bool,
    text: String,
    size: usize, // TODO: this should be in css?
    set_text: Callback<String>,
    set_editable: Callback<bool>,
    #[prop_or_default]
    id: Option<&'static str>,
}

#[function_component(EditableInput)]
fn editable_input(props: &EditableInputP) -> Html {
    let onkeyup = {
        let set_editable = props.set_editable.clone();
        Callback::from(move |e: KeyboardEvent| {
            const ENTER_KEY_CODE: u32 = 13;
            if e.key_code() == ENTER_KEY_CODE {
                set_editable.emit(false);
            }
        })
    };
    let oninput = {
        let set_text = props.set_text.clone();
        // TODO: Probably should not emit on every keypress,
        // but Keybaord event values are behind for some reason.
        Callback::from(move |e: InputEvent| {
            let t: HtmlTextAreaElement = e.target_unchecked_into();
            set_text.emit(t.value());
        })
    };
    html! {
        <input
            type="text"
            id={props.id}
            size={format!("{}", props.size)}
            disabled={!props.editable}
            onfocusout={props.set_editable.reform(|_| false)}
            onclick={props.set_editable.reform(|_| true)}
            value={props.text.clone()}
            onkeyup={onkeyup}
            oninput={oninput}
        />
    }
}

#[derive(PartialEq, Properties)]
struct TaskInputP {
    id: usize,
}
#[function_component(TaskInput)]
fn task_input(props: &TaskInputP) -> Html {
    let editing = use_state(|| false);
    let binding = use_context::<Athens>().unwrap();
    let athens = binding.get();
    let text = athens.get_task(TaskId(props.id)).unwrap().text;
    let set_text = {
        let a = binding.inner.clone(); // TODO bad abstraction hack.
        let id = TaskId(props.id);
        Callback::from(move |text| {
            a.set_task(model::Task { id, text });
        })
    };
    html! {
        <EditableInput
            editable={*editing}
            size=80
            text={text}
            set_editable={Callback::from(move |b| editing.set(b))}
            set_text={set_text}
        />
    }
}

#[derive(PartialEq, Properties)]
struct DraggableEntryP {
    callback: Callback<ListM>,
    draggable: bool,
    order: usize,
    children: Children,
}
#[function_component(DraggableEntry)]
fn draggable_entry(props: &DraggableEntryP) -> Html {
    // Communicate drag and drop to enclosing List
    let draggable = if props.draggable { "true" } else { "false" };
    let set_dragged = |d| props.callback.reform(move |_| ListM::SetDragged(d));
    let set_dragged_over = |d| props.callback.reform(move |_| ListM::SetDraggedOver(d));
    let drop = props.callback.reform(|_| ListM::Dropped);
    let ondragover = {
        let order = props.order;
        props.callback.reform(move |e: DragEvent| {
            e.prevent_default(); // Neccessary for ondrop to be called.
            ListM::SetDraggedOver(Some(order))
        })
    };
    html! {
        <li draggable={draggable}
            ondragstart={set_dragged(Some(props.order))}
            ondragend={set_dragged(None)}
            ondragenter={ondragover.clone()}
            ondragover={ondragover}
            ondragleave={set_dragged_over(None)}
            ondrop={drop}
        >
        { for props.children.iter() }
        </li>
    }
}

#[derive(PartialEq, Properties)]
struct UserSelectP {
    active: Option<UserId>,
    set_active: Callback<Option<UserId>>,
    add_user: Callback<()>,
}
#[function_component(UserSelect)]
fn user_select(props: &UserSelectP) -> Html {
    // User select is a main button showing the current selection and
    // a list of hidden buttons. The buttons are either "Everyone" or
    // the actual users. Also there's an "Add user" button at the end
    // of the hidden buttons.
    let mut hidden = Vec::new();
    let mut main_button = None;

    // "Everyone button"
    let select_everyone = props.set_active.reform(move |_| None);
    let select_everyone_button = html! {
        <button onclick={select_everyone}>
        {"Everyone"}
        </button>
    };
    // The main visible button is "Everyone" or it will be some user, set later.
    if props.active.is_none() {
        main_button = Some(select_everyone_button);
    } else {
        hidden.push(select_everyone_button);
    }

    // Select user buttons
    let binding = use_context::<Athens>().unwrap();
    let athens = binding.get();
    for user in athens.users().into_iter() {
        let select_user = props.set_active.reform(move |_| Some(user));
        let alias = athens.get_user(user).unwrap().alias;
        let value = if alias.is_empty() {
            format!("user/{}", user.0)
        } else {
            alias
        };
        if Some(user) == props.active {
            assert_eq!(main_button, None);
            let editing = use_state(|| false);
            let start_editing = {
                let editing = editing.clone();
                Callback::from(move |_| editing.set(true))
            };
            let set_user_alias = {
                let a = binding.inner.clone(); // TODO hack?
                Callback::from(move |alias| {
                    a.set_user(model::User {
                        id: user,
                        alias,
                        weight: 1,
                    });
                })
            };
            if *editing {
                // When the user clicks on the main user button, we set
                // `editing` to true and render a new input element to edit the
                // selected user's name. After element is rendered, focus on it,
                // so users are immediately editing the username and do not have
                // to click twice.
                use_effect(|| {
                    web_sys::window()
                        .expect("expected a window")
                        .document()
                        .expect("expected a document")
                        .query_selector("#selected-user-input")
                        .expect("invalid query")
                        .expect("could not find selected user input element")
                        .dyn_ref::<web_sys::HtmlElement>()
                        .expect("was not an html element")
                        .focus()
                        .expect("Failed to focus");
                    || {}
                });
            }

            main_button = Some(html! {
                <button onclick={start_editing} >
                    if *editing {
                        <EditableInput
                            editable=true
                            size=10
                            id="selected-user-input"
                            text={value}
                            set_editable={Callback::from(move |b| editing.set(b))}
                            set_text={set_user_alias}
                        />
                    } else {
                        {value}
                    }
                </button>
            });
        } else {
            hidden.push(html! {
                <button onclick={select_user}>
                    {value}
                </button>
            });
        }
    }
    let main_button = main_button.expect("Active user not found");

    let add_user = props.add_user.reform(|_| ());
    hidden.push(html! {
        <button onclick={add_user}>{"New user"}</button>
    });

    html! {
        <div class="dropdown">
            {main_button}
            <div class="dropdown-content">
                {for hidden.into_iter()}
            </div>
        </div>
    }
}

struct List {
    dragged: Option<usize>,
    dragged_over: Option<usize>,
    ordering: Ordering,
    athens: Athens,
    _handle: ContextHandle<Athens>,
    selected_user: Option<UserId>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ordering {
    Easiness,
    Importance,
    ImportantAndEasy,
}
impl Default for Ordering {
    fn default() -> Self {
        Ordering::Importance
    }
}

#[derive(Clone, Debug)]
enum ListM {
    // Entry CRUD
    AddEntry,
    // Drag and drop.
    SetDragged(Option<usize>),
    SetDraggedOver(Option<usize>),
    Dropped,
    // Saving.
    StartSaving,
    LoadData(SimpleAthensSpace),
    // Sorting
    SetOrdering(Ordering),
    // Null
    Ignore,
    //
    SetActiveUser(Option<UserId>),
    AddUser,
}

impl List {
    fn save_request(&self) -> Request {
        Request::post("/tasks")
            .json(&self.athens.inner.lock().unwrap().clone())
            .expect("Failed to make request")
    }
    fn athens(&self) -> &dyn AthensSpace {
        self.athens.get()
    }
}

async fn load_tasks() -> Option<SimpleAthensSpace> {
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
        let cb = ctx.link().callback(|_| ListM::Ignore);
        let (athens, _handle) = ctx.link().context::<Athens>(cb).unwrap();
        Self {
            dragged: None,
            dragged_over: None,
            ordering: Ordering::Importance,
            athens,
            selected_user: None,
            _handle,
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("List received Message: {:?}", &msg);
        match msg {
            ListM::Ignore => false,
            ListM::AddEntry => {
                self.athens().create_task();
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
                    match self.ordering {
                        Ordering::Importance => {
                            let user = self.selected_user.unwrap();
                            self.athens().swap_user_importance(user, from, to);
                        }
                        Ordering::Easiness => {
                            let user = self.selected_user.unwrap();
                            self.athens().swap_user_easiness(user, from, to);
                        }
                        _ => log::error!(
                            "Tried to drag and drop when ordering is {:?}",
                            self.ordering
                        ),
                    };
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
            ListM::StartSaving => {
                let rq = self.save_request();
                ctx.link().send_future(async {
                    let rp = rq.send().await;
                    log::info!("Response: {:?}", rp);
                    ListM::Ignore
                });
                false
            }
            ListM::SetOrdering(o) => {
                self.ordering = o;
                true
            }
            ListM::LoadData(model) => {
                *self.athens.inner.lock().unwrap() = model;
                true
            }
            ListM::SetActiveUser(u) => {
                self.selected_user = u;
                true
            }
            ListM::AddUser => {
                self.athens().create_user();
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        use Ordering::*;
        let task_ids = match (self.selected_user, self.ordering) {
            (Some(user), Importance) => self.athens().user_importance(user),
            (Some(user), Easiness) => self.athens().user_easiness(user),
            (Some(user), ImportantAndEasy) => self.athens().user_important_and_easy(user),
            (None, Importance) => self.athens().important_tasks(),
            (None, Easiness) => self.athens().easy_tasks(),
            (None, ImportantAndEasy) => self.athens().important_and_easy_tasks(),
        };
        let draggable = self.selected_user.is_some() && self.ordering != Ordering::ImportantAndEasy;
        let entries_html: Vec<Html> = task_ids
            .into_iter()
            .enumerate()
            .map(|(order, TaskId(id))| {
                // Pass down a callback to modify the entry text.
                html! {
                    <DraggableEntry
                        callback={ctx.link().callback(|x| x)}
                        draggable={draggable} order={order}
                    >
                        <TaskInput id={id}/>
                    </DraggableEntry>
                }
            })
            .collect();

        let addentry = ctx.link().callback(|_| ListM::AddEntry);
        let save = ctx.link().callback(|_| ListM::StartSaving);

        let sort_msg = match self.ordering {
            Importance => "Sorted by importance",
            Easiness => "Sorted by easiness",
            ImportantAndEasy => "Sorted by important and easy",
        };
        let toggle_sort = {
            let next = match self.ordering {
                Importance => Easiness,
                Easiness => ImportantAndEasy,
                ImportantAndEasy => Importance,
            };
            ctx.link().callback(move |_| ListM::SetOrdering(next))
        };

        html! {
            <div>
                <button onclick={toggle_sort}>{sort_msg}</button>
                <p style="display:inline-block; padding: 0 4 0 5">{" according to "}</p>
                <UserSelect
                    active={self.selected_user}
                    set_active={ctx.link().callback(|i| ListM::SetActiveUser(i))}
                    add_user={ctx.link().callback(|_| ListM::AddUser)}
                />
                <ul>{ for entries_html }</ul>
                <button onclick={addentry}>{"Add"}</button>
                <button onclick={save}>{"Save"}</button>
            </div>
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let space = Athens::new();
    html! {
        <ContextProvider<Athens> context={space.clone()}>
        <link href="public/style.css" rel="stylesheet"/>
        <h1>{"Lists!"}</h1>
        <List/>
        </ContextProvider<Athens>>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Starting app");
    yew::start_app::<App>();
    Ok(())
}
