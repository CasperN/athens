#![feature(async_closure)]
use gloo_net::http::Request;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::context::ContextHandle;
use yew::prelude::*;
// use yew::virtual_dom::AttrValue;

use model::{AthensSpace, TaskId, UserId, SimpleAthensSpace};

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

struct Entry {
    editing: bool,
    athens: Athens,
    _handle: ContextHandle<Athens>,
}

#[derive(Clone, Copy)]
enum EntryM {
    StartEditing,
    StopEditing,
    Ignore,
}

#[derive(PartialEq, Properties)]
struct EntryP {
    id: usize,
}
impl Component for Entry {
    type Properties = EntryP;
    type Message = EntryM;

    fn create(ctx: &Context<Self>) -> Self {
        let cb = ctx.link().callback(|_| EntryM::Ignore);
        let (athens, _handle) = ctx.link().context::<Athens>(cb).unwrap();
        Entry {
            athens,
            _handle,
            editing: false,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: EntryM) -> bool {
        match msg {
            EntryM::StartEditing => self.editing = true,
            EntryM::StopEditing => self.editing = false,
            EntryM::Ignore => (),
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
            let id = TaskId(ctx.props().id);
            let a = self.athens.inner.clone();
            Callback::from(move |e: InputEvent| {
                let t: HtmlTextAreaElement = e.target_unchecked_into();
                a.set_task(model::Task { id, text: t.value() });
            })
        };
        let value = self
            .athens
            .get()
            .get_task(TaskId(ctx.props().id))
            .unwrap()
            .text;

        html! {
            <input
                type="text"
                size="80"
                disabled={!self.editing}
                onclick={start_editing}
                onfocusout={stop_editing}
                value={value}
                onkeypress={stop_editing_on_enter}
                oninput={emit_text}
            />
        }
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
    let set_dragged_over = |d| props.callback.reform(move |_| {
        log::info!("set dragover");
        ListM::SetDraggedOver(d)
    });
    let drop = props.callback.reform(|_| ListM::Dropped);
    let ondragover = {
        let order = props.order;
        props.callback.reform(move |e: DragEvent| {
            e.prevent_default(); // Neccessary for ondrop to be called.
            log::info!("on dragover!");
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
    active: UserId,
    set_active: Callback<UserId>,
    add_user: Callback<()>,
}
#[function_component(UserSelect)]
fn user_select(props: &UserSelectP) -> Html {
    let binding = use_context::<Athens>().unwrap();
    let athens = binding.get();
    let users = athens.users().into_iter().map(|i|{
        let onclick = props.set_active.reform(move |_| i);
        html! {
            <button onclick={onclick}>
                {format!("user/{}:`{}`", i.0, athens.get_user(i).unwrap().alias)}
            </button>
        }
    });

    let onclick = props.add_user.reform(|_|());
    html! {
        <div class="dropdown">
            <button onclick={onclick}>{"AddUser"}</button>
            <div class="dropdown-content">
                { for users }
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
    selected_user: UserId,
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
    SetActiveUser(UserId),
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
        let user_id = {
            let users = athens.get().users();
            if users.is_empty() {
                athens.get().create_user().id
            } else {
                users[0]
            }
        };
        Self {
            dragged: None,
            dragged_over: None,
            ordering: Ordering::Importance,
            athens,
            selected_user: user_id,
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
                            self.athens()
                                .swap_user_importance(self.selected_user, from, to);
                        }
                        Ordering::Easiness => {
                            self.athens()
                                .swap_user_easiness(self.selected_user, from, to);
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
        // TODO: selected user should be optional and
        // None should mean "aggregated", which are not draggable.
        let task_ids = match self.ordering {
            Importance => self.athens().user_importance(self.selected_user),
            Easiness => self.athens().user_easiness(self.selected_user),
            ImportantAndEasy => self.athens().user_important_and_easy(self.selected_user),
        };
        let draggable = self.ordering != Ordering::ImportantAndEasy;
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
                        <Entry id={id}/>
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
                <UserSelect
                    active={self.selected_user}
                    set_active={ctx.link().callback(|i| ListM::SetActiveUser(i))}
                    add_user={ctx.link().callback(|_| ListM::AddUser)}
                />
                <button onclick={toggle_sort}>{sort_msg}</button>
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
