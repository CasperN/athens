use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
struct ListEntryP {
    id: usize,
}

#[function_component(ListEntry)]
fn list_entry(p: &ListEntryP) -> Html {
    let user_value = use_state(|| String::new());
    let oninput = {
        let uv = user_value.clone();
        Callback::from(move |event: InputEvent| {
            let i: HtmlTextAreaElement = event.target_unchecked_into();
            uv.set(i.value())
        })
    };

    html! {
        <li draggable="true">
        <b> {p.id}{":  "} </b>
        <input oninput={oninput.clone()} type="text"/>
        {&*user_value}
        </li>

    }
}

//#[derive(Properties, PartialEq)]

#[function_component(List)]
fn list() -> Html {
    let elements = use_state(|| Vec::new());
    let add_li = {
        let elements = elements.clone();
        Callback::from(move |_| {
            let mut e = (*elements).clone();
            let n = e.len();
            e.push(html! {
                <ListEntry id={n}/>
            });
            elements.set(e);
        })
    };

    html! {
        <>
        { for (*elements).clone() }
        <button onclick={add_li}> {"Another!"} </button>
        </>
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
