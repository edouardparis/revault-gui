use iced::{scrollable, Column, Container, Element};

use crate::ui::{
    component::{navbar, scroll},
    error::Error,
    message::Message,
    view::{layout, sidebar::Sidebar, Context},
};

#[derive(Debug)]
pub struct VaultsView {
    scroll: scrollable::State,
    sidebar: Sidebar,
}

impl VaultsView {
    pub fn new() -> Self {
        VaultsView {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(
                    Column::new()
                        .push(Column::with_children(vaults).spacing(5))
                        .spacing(20),
                ),
            ))),
        )
        .into()
    }
}
