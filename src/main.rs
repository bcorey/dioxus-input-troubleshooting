use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus::events::{MouseEvent, FormEvent, PointerData};
use dioxus::core::UiEvent;

use wasm_logger;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::web::launch(app);
}

fn dims(id: &str) -> (f64, f64) {
    let parent_rect = web_sys::window()
    .unwrap().document()
    .unwrap().get_element_by_id(id)
    .unwrap().get_bounding_client_rect();
    (parent_rect.width(), parent_rect.height())
}

static VALUE: Atom<i32> = |_| 0;

#[derive(Debug, Clone, Copy)]
struct DraggableState {
    id: u32,
    start_pos: (f64, f64),
    pos: (f64, f64),
}

fn app(cx: Scope) -> Element {
    let value = use_read(&cx, VALUE);
    let set_value = use_set(&cx, VALUE);
    
    log::info!("updating ui");

    //DRAGGING CODE
    
    let active_draggable: &UseState<Option<DraggableState>> = use_state(&cx, || None);
    let positions: &UseRef<HashMap<u32, (f64, f64)>> = use_ref(&cx, || HashMap::new());  

    // When user starts dragging, track where on the square they started
    let mouse_down_handler =
        move |event: UiEvent<PointerData>, el: u32| {
            let pos = *positions.read().get(&el).expect("missing position");
            let start_pos = (event.data.page_x as f64 - pos.0, 
                event.data.page_y as f64 - pos.1);
            
            log::info!("dragging {el}");
            active_draggable.set(Some(DraggableState{
                id: el,
                start_pos: start_pos,
                pos: pos
            }));

        };
    
    // When the mouse moves on the container
    let mouse_move_handler = move |event: UiEvent<PointerData>| {
        // If we are currently dragging the square
        active_draggable.current().and_then(move |active| {
            let (s_x, s_y) = active.start_pos;
            let x = (event.data.page_x as f64 - s_x);
            let y = (event.data.page_y as f64 - s_y);
            positions.write().insert(active.id, (x, y))
        });

    };
    
    // When mouse is released, stop dragging
    let mouse_up_handler = move |_: UiEvent<PointerData>| {
        log::info!("mouseup");
        active_draggable.set(None);
    };

    let mut panel_content: HashMap<u32, Option<LazyNodes>> = HashMap::new(); // list of children for draggables

    panel_content.insert(1u32, Some(rsx!{
        div {
            class: "button-row",
            div {
                class: "button-column",
                VoteButton {
                    name: "+",
                    onclick: move |_| (),
                }
            }
            div {
                class: "button-spacer"
            }
            div {
                class: "button-column",
                VoteButton {
                    name: "-",
                    onclick: move |_| (),
                }
            }
        }
    })); 

    panel_content.insert(3u32, Some(rsx!{
            div {
                class: "button-row",
                VoteButton {
                    name: "Fill",
                    onclick: move |_| (),
                }
            }
    }));

    let text_box_content = use_state(&cx, || "From Rust with <3".to_string());    

    panel_content.insert(75u32, Some(rsx!{
        //
        // This text box won't work (drops focus)
        //
        TextBox {
            value: "{text_box_content}",
            oninput: move |evt: FormEvent| text_box_content.set(evt.value.clone()),
        }
    }));

    let mut draggables: Vec<Element> = Vec::new();

    let (view_width, view_height) = dims("main");

    let mut i = 0;

    for (id, body) in panel_content {
        if !positions.read().contains_key(&id) { // if key does not yet exist
            positions.write().insert(id, 
            ((view_width / 2.0) + (100.0 * i as f64) - (300.0 as f64), view_height / 2.0 )); // set position to initial value
            i += 1;
        }

        if let Some(content) = body {
            draggables.push(cx.render(rsx!{
                Draggable {
                    onpointerdown: move |evt| mouse_down_handler(evt, id),
                    pos: *positions.read().get(&id).unwrap(),
                    key: "{id}",
                    content // <- textbox element
                }
            }))
        }
    }

    cx.render(rsx!{         
        div {
            width: "100vw",
            height: "100vh",
            style: "overflow: hidden; height: 100vh; width: 100vw; position: absolute; top: 0; left: 0",
            onpointermove: move |evt| {
                mouse_move_handler(evt);
            },
            onpointerup: move |evt| {
                mouse_up_handler(evt);
            },
            draggables.iter(),
            //
            // This text box works
            //
            div {
                style: "position: relative; left: 80px; top: 80px",
                TextBox { 
                    value: "{text_box_content}",
                    oninput: move |evt: FormEvent| text_box_content.set(evt.value.clone()),
                }
            }
            
        }
    })
}

#[derive(Props)] 
pub struct VoteButtonProps<'a> {
    name: &'a str,
    onclick: EventHandler<'a, MouseEvent>
}

#[allow(non_snake_case)]
pub fn VoteButton<'a>(cx: Scope<'a,VoteButtonProps<'a>>) -> Element {
    cx.render(rsx!{
        div {
            button {
                class: "button button-outline",
                width: "100%",
                onclick: move |evt| cx.props.onclick.call(evt),
                "{cx.props.name}"
            }
        }
    })
}


#[allow(non_snake_case)]
pub fn Canvas(cx: Scope) -> Element {
    cx.render(rsx!{
        div {
            class: "",
            id: "parent",
            canvas {
                id: "canvas",
                prevent_default: "onclick",
                onclick: move |_|{
                    // handle the event without navigating the page.
                }
            }
        }
    })
}


#[derive(Copy, Clone)]
pub enum DraggableType {
    UI,
    Text
}

#[derive(Props)] 
pub struct DraggableProps<'a> {
    onpointerdown: EventHandler<'a, UiEvent<PointerData>>,
    pos: (f64, f64),
    children: Element<'a>
}

#[allow(non_snake_case)]
pub fn Draggable<'a>(cx: Scope<'a, DraggableProps<'a>>) -> Element {

    cx.render(rsx!{
        div {
            class: "draggable",
            left: "{cx.props.pos.0}px",
            top: "{cx.props.pos.1}px",
            DragIcon {
                onpointerdown:  move |evt| cx.props.onpointerdown.call(evt),
                draggable_type: DraggableType::UI,
            }
            &cx.props.children,
        }
    })
}

#[derive(Props)]
struct DragIconProps<'a> {
    onpointerdown: EventHandler<'a, UiEvent<PointerData>>,
    draggable_type: DraggableType,
}

#[allow(non_snake_case)]
fn DragIcon<'a>(cx: Scope<'a, DragIconProps<'a>>) -> Element {

    let classes = match &cx.props.draggable_type {
        DraggableType::UI => "arrows-box",
        DraggableType::Text => "arrows-box arrows-box-text-box"
    };
    cx.render(rsx!{
        div {
            class: "{classes}",
            div {
                onpointerdown: move |evt| cx.props.onpointerdown.call(evt),
                class: "arrows",
                div {
                    class: "arrow arrow-left",
                }
                div {
                    class: "arrow arrow-up",
                }
                div {
                    class: "arrow arrow-right",
                }
                div {
                    class: "arrow arrow-down",
                }
            }
        }
    })
}


#[derive(Props)]
pub struct TextProps<'a> {
    value: &'a str,
    oninput: EventHandler<'a, FormEvent>,
}

#[allow(non_snake_case)]
pub fn TextBox<'a>(cx: Scope<'a, TextProps<'a>>) -> Element {

    cx.render(rsx!{
        div {
            class: "text-box",
            input {
                r#type: "text",
                style: "border: none",
                value: "{cx.props.value}",
                oninput: move |evt| cx.props.oninput.call(evt),
            }
        }
    })
}