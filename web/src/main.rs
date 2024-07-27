use yew::prelude::*;

pub struct ChatBox;

pub enum ChatBoxMsg {}

impl Component for ChatBox {
    type Message = ChatBoxMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {}
    }
}

#[function_component(App)]
fn app() -> Html {
    derive::file_html!("html/main.html")
}

fn main() {
    yew::Renderer::<App>::new().render();
}
