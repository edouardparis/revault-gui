use iced::{pick_list, Container, Length, Row};

use crate::revault::Role;
use crate::ui::{
    component::{
        button,
        icon::{
            deposit_icon, dot_icon, home_icon, network_icon, person_check_icon, send_icon,
            settings_icon, vaults_icon,
        },
        separation, text, TransparentPickListStyle,
    },
    menu::Menu,
    message::Message,
    view::{layout, Context},
};

#[derive(Debug, Clone)]
pub struct Sidebar {
    pick_role: pick_list::State<Role>,
    deposit_menu_button: iced::button::State,
    delegate_menu_button: iced::button::State,
    home_menu_button: iced::button::State,
    vaults_menu_button: iced::button::State,
    network_menu_button: iced::button::State,
    spend_menu_button: iced::button::State,
    settings_menu_button: iced::button::State,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            deposit_menu_button: iced::button::State::new(),
            delegate_menu_button: iced::button::State::new(),
            home_menu_button: iced::button::State::new(),
            vaults_menu_button: iced::button::State::new(),
            network_menu_button: iced::button::State::new(),
            spend_menu_button: iced::button::State::new(),
            settings_menu_button: iced::button::State::new(),
            pick_role: pick_list::State::default(),
        }
    }

    pub fn view(&mut self, context: &Context) -> Container<Message> {
        let role = if context.role_edit {
            Container::new(
                pick_list::PickList::new(
                    &mut self.pick_role,
                    &Role::ALL[..],
                    Some(context.role),
                    Message::ChangeRole,
                )
                .padding(10)
                .width(Length::Units(200))
                .style(TransparentPickListStyle),
            )
        } else {
            Container::new(text::simple(&context.role.to_string())).padding(10)
        };
        let home_button = if context.menu == Menu::Home {
            button::primary(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
            )
            .on_press(Message::Menu(Menu::Home))
        } else {
            button::transparent(
                &mut self.home_menu_button,
                button::button_content(Some(home_icon()), "Home"),
            )
            .on_press(Message::Menu(Menu::Home))
        };
        let vaults_button = if context.menu == Menu::Vaults {
            button::primary(
                &mut self.vaults_menu_button,
                button::button_content(Some(vaults_icon()), "Vaults"),
            )
            .on_press(Message::Menu(Menu::Vaults))
        } else {
            button::transparent(
                &mut self.vaults_menu_button,
                button::button_content(Some(vaults_icon()), "Vaults"),
            )
            .on_press(Message::Menu(Menu::Vaults))
        };
        let network_button = if context.menu == Menu::Network {
            button::primary(
                &mut self.network_menu_button,
                button::button_content(Some(network_icon()), "Network"),
            )
            .on_press(Message::Menu(Menu::Network))
        } else {
            let mut row = Row::new()
                .push(network_icon())
                .push(text::simple("Network"))
                .spacing(10)
                .align_items(iced::Align::Center);

            if context.network_up {
                row = row.push(text::success(dot_icon().size(7)))
            } else {
                row = row.push(text::danger(dot_icon().size(7)))
            }

            button::transparent(
                &mut self.network_menu_button,
                Container::new(row).padding(5),
            )
            .on_press(Message::Menu(Menu::Network))
        };

        let deposit_button = if context.menu == Menu::Deposit {
            button::primary(
                &mut self.deposit_menu_button,
                button::button_content(Some(deposit_icon()), "Deposit"),
            )
            .on_press(Message::Menu(Menu::Deposit))
        } else {
            button::transparent(
                &mut self.deposit_menu_button,
                button::button_content(Some(deposit_icon()), "Deposit"),
            )
            .on_press(Message::Menu(Menu::Deposit))
        };

        let settings_button = if context.menu == Menu::Settings {
            button::primary(
                &mut self.settings_menu_button,
                button::button_content(Some(settings_icon()), "Settings"),
            )
            .on_press(Message::Menu(Menu::Settings))
            .width(iced::Length::Units(200))
        } else {
            button::transparent(
                &mut self.settings_menu_button,
                button::button_content(Some(settings_icon()), "Settings"),
            )
            .on_press(Message::Menu(Menu::Settings))
            .width(iced::Length::Units(200))
        };

        let actions = if context.role == Role::Manager {
            Container::new(
                button::transparent(
                    &mut self.spend_menu_button,
                    button::button_content(Some(send_icon()), "Send"),
                )
                .on_press(Message::Menu(Menu::Send))
                .width(iced::Length::Units(200)),
            )
        } else if context.menu == Menu::DelegateFunds {
            Container::new(
                button::primary(
                    &mut self.delegate_menu_button,
                    button::button_content(Some(person_check_icon()), "Delegate funds"),
                )
                .on_press(Message::Menu(Menu::DelegateFunds))
                .width(iced::Length::Units(200)),
            )
        } else {
            Container::new(
                button::transparent(
                    &mut self.delegate_menu_button,
                    button::button_content(Some(person_check_icon()), "Delegate funds"),
                )
                .on_press(Message::Menu(Menu::DelegateFunds))
                .width(iced::Length::Units(200)),
            )
        };
        layout::sidebar(
            layout::sidebar_menu(vec![
                role.width(Length::Units(200)),
                separation().width(iced::Length::Units(200)),
                Container::new(home_button.width(Length::Units(200))),
                Container::new(vaults_button.width(Length::Units(200))),
                Container::new(network_button.width(Length::Units(200))),
                separation().width(Length::Units(200)),
                Container::new(deposit_button.width(Length::Units(200))),
                actions,
            ]),
            Container::new(settings_button),
        )
    }
}
