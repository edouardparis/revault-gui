use iced::{scrollable, Align, Column, Container, Element, Length, Row};

use crate::revaultd::model::VaultStatus;

use crate::ui::{
    component::{button, card, navbar, scroll, text, ContainerBackgroundStyle},
    error::Error,
    menu::Menu,
    message::{Message, VaultFilterMessage},
    view::{layout, sidebar::Sidebar, Context},
};

#[derive(Debug)]
pub struct StakeholderACKFundsView {
    scroll: scrollable::State,
    close_button: iced::button::State,
}

impl StakeholderACKFundsView {
    pub fn new() -> Self {
        StakeholderACKFundsView {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        _ctx: &Context,
        deposits: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut col_deposits = Column::new();
        for element in deposits.into_iter() {
            col_deposits = col_deposits.push(element);
        }
        let element: Element<_> = col_deposits.spacing(20).max_width(1000).into();
        let col = Column::new()
            .push(
                Row::new().push(Column::new().width(Length::Fill)).push(
                    Container::new(
                        button::cancel(
                            &mut self.close_button,
                            Container::new(text::simple("X Close")).padding(10),
                        )
                        .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Shrink),
                ),
            )
            .push(
                Container::new(element)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(50);
        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}

#[derive(Debug)]
pub struct StakeholderDelegateFundsView {
    sidebar: Sidebar,
    scroll: scrollable::State,
    active_vaults_button: iced::button::State,
    secured_vaults_button: iced::button::State,
}

impl StakeholderDelegateFundsView {
    pub fn new() -> Self {
        StakeholderDelegateFundsView {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
            active_vaults_button: iced::button::State::new(),
            secured_vaults_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        active_balance: &u64,
        available_vaults: Vec<Element<'a, Message>>,
        unsecured_vaults: Vec<Element<'a, Message>>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().push(
            card::white(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(text::bold(text::simple(&format!(
                                "{}",
                                ctx.converter.converts(*active_balance)
                            ))))
                            .push(text::simple(&ctx.converter.unit.to_string()))
                            .spacing(10)
                            .align_items(Align::Center),
                    )
                    .push(text::simple("are delegated to the managers")),
            ))
            .width(Length::Fill),
        );

        if available_vaults.len() > 0 {
            col = col
                .push(
                    Container::new(text::simple(&format!(
                        "{} vaults are available",
                        available_vaults.len()
                    )))
                    .width(Length::Fill),
                )
                .push(
                    Column::new()
                        .push(Column::with_children(available_vaults).spacing(5))
                        .spacing(20),
                )
                .spacing(15);
        }

        if unsecured_vaults.len() > 0 {
            col = col
                .push(
                    Container::new(text::simple(&format!(
                        "{} vaults needs to be acknowledge first",
                        unsecured_vaults.len()
                    )))
                    .width(Length::Fill),
                )
                .push(
                    Column::new()
                        .push(Column::with_children(unsecured_vaults).spacing(5))
                        .spacing(20),
                )
                .spacing(15);
        }

        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(col),
            ))),
        )
        .into()
    }
}
