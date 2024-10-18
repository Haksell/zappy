#![allow(non_snake_case)]
use dioxus::prelude::*;

fn main() {
    launch(App);
}

pub fn App() -> Element {
    rsx! { "story" }
}
