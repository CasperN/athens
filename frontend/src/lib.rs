use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    log::info!("app running");

    let color = use_state(|| "red");
    let cb = {
        let color = color.clone();
        Callback::from(move |_| {
            let next_color = match *color {
                "red" => "green",
                "green" => "blue",
                _ => "red",
            };
            color.set(next_color)
        })
    };

    html! {
        <>
        <h1 style={format!("background-color:{};", {*color})}>
            { "Hello world from frontend lol ol " }
        <button onclick={ cb.clone() }>{ "Color me!" }</button>
        </h1>
        </>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
    Ok(())
}
