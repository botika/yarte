use wasm_bindgen::{prelude::*, JsCast, UnwrapThrowExt};
use web_sys::{window, Element, Event, Node};

use yarte_wasm_app::*;

use crate::{app::NonKeyed, handler::*};

#[derive(Debug, Deserialize)]
pub struct Row {
    pub id: u32,
    pub label: String,
}

pub struct RowDOM {
    pub root: Node,
    // depend item.id
    pub id_node: Node,
    // depend item.label item.id
    pub label_node: Node,
    // depend item.id
    pub delete_node: Node,
    // depend item.id
    pub closure_select: Option<Closure<dyn Fn(Event)>>,
    // depend item.id
    pub closure_delete: Option<Closure<dyn Fn(Event)>>,
}

impl RowDOM {
    pub fn new(id: u32, label: &str, root: &Element, mb: &Mailbox<NonKeyed>) -> Self {
        let root = root.clone_node_with_deep(true).unwrap_throw();
        let id_node = root.first_child().unwrap_throw();
        let label_parent = id_node.next_sibling().unwrap_throw();
        let label_node = label_parent.first_child().unwrap_throw();
        let delete_parent = label_parent.next_sibling().unwrap_throw();
        let delete_node = delete_parent.first_child().unwrap_throw();

        // depend id
        id_node.set_text_content(Some(&id.to_string()));

        // depend label
        label_node.set_text_content(Some(label));

        // depend id
        let cloned = mb.clone();
        let closure_select = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Select(id));
        }) as Box<dyn Fn(Event)>);
        label_node
            .add_event_listener_with_callback("click", closure_select.as_ref().unchecked_ref())
            .unwrap_throw();

        let cloned = mb.clone();
        let closure_delete = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Delete(id));
        }) as Box<dyn Fn(Event)>);
        delete_node
            .add_event_listener_with_callback("click", closure_delete.as_ref().unchecked_ref())
            .unwrap_throw();

        RowDOM {
            root,
            id_node,
            label_node,
            delete_node,
            closure_select: Some(closure_select),
            closure_delete: Some(closure_delete),
        }
    }

    pub fn update(&mut self, Row { id, label }: &Row, mb: &Mailbox<NonKeyed>) {
        // depend id
        self.id_node.set_text_content(Some(&id.to_string()));

        // depend label
        self.label_node.set_text_content(Some(label));

        // depend id
        if let Some(cl) = &self.closure_select {
            self.label_node
                .remove_event_listener_with_callback("click", cl.as_ref().unchecked_ref())
                .unwrap_throw();
        }
        let id = *id;
        let cloned = mb.clone();
        self.closure_select = Some(Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Select(id))
        }) as Box<dyn Fn(Event)>));
        self.label_node
            .add_event_listener_with_callback(
                "click",
                &self
                    .closure_select
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap_throw();

        let cloned = mb.clone();
        if let Some(cl) = &self.closure_delete {
            self.delete_node
                .remove_event_listener_with_callback("click", cl.as_ref().unchecked_ref())
                .unwrap_throw();
        }
        self.closure_delete = Some(Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Delete(id));
        }) as Box<dyn Fn(Event)>));
        self.delete_node
            .add_event_listener_with_callback(
                "click",
                &self
                    .closure_delete
                    .as_ref()
                    .unwrap_throw()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap_throw();
    }

    pub fn hydrate(&mut self, row: &Row, mb: &Mailbox<NonKeyed>) {
        let cloned = mb.clone();
        let id = row.id;
        let closure_select = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Select(id));
        }) as Box<dyn Fn(Event)>);
        self.label_node
            .add_event_listener_with_callback("click", closure_select.as_ref().unchecked_ref())
            .unwrap_throw();
        self.closure_select = Some(closure_select);

        let cloned = mb.clone();
        let closure_delete = Closure::wrap(Box::new(move |event: Event| {
            event.prevent_default();
            cloned.send(Delete(id));
        }) as Box<dyn Fn(Event)>);
        self.delete_node
            .add_event_listener_with_callback("click", closure_delete.as_ref().unchecked_ref())
            .unwrap_throw();
        self.closure_delete = Some(closure_delete);
    }
}

pub fn row_element() -> Element {
    let document = window().unwrap_throw().document().unwrap_throw();
    let tr = document.create_element("tr").unwrap_throw();
    let td1 = td("col-md-1");
    tr.append_child(&td1).unwrap_throw();

    let td2 = td("col-md-4");
    tr.append_child(&td2).unwrap_throw();
    let a2 = document.create_element("a").unwrap_throw();
    a2.set_attribute("class", "lbl").unwrap_throw();
    td2.append_child(&a2).unwrap_throw();

    let td3 = td("col-md-1");
    tr.append_child(&td3).unwrap_throw();
    let a3 = document.create_element("a").unwrap_throw();
    a3.set_attribute("class", "remove").unwrap_throw();
    td3.append_child(&a3).unwrap_throw();
    let span = document.create_element("span").unwrap_throw();
    span.set_attribute("class", "glyphicon glyphicon-remove remove")
        .unwrap_throw();
    span.set_attribute("aria-hidden", "true").unwrap_throw();
    a3.append_child(&span).unwrap_throw();

    let td4 = td("col-md-6");
    tr.append_child(&td4).unwrap_throw();

    tr
}

fn td(class_name: &str) -> Element {
    let document = window().unwrap_throw().document().unwrap_throw();
    let td = document.create_element("td").unwrap_throw();
    td.set_attribute("class", class_name).unwrap_throw();
    td
}
