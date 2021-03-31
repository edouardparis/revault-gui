use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use chrono::NaiveDateTime;
use iced::{scrollable, Align, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    component::{badge, button, card, separation, text, ContainerBackgroundStyle},
    error::Error,
    icon,
    message::{Message, SignMessage, VaultMessage},
    view::Context,
};

use crate::{
    revault::Role,
    revaultd::model::{BroadcastedTransaction, Vault, VaultStatus, VaultTransactions},
};

#[derive(Debug)]
pub struct VaultModal {
    cancel_button: iced::button::State,
    copy_button: iced::button::State,
    scroll: scrollable::State,
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            copy_button: iced::button::State::default(),
            cancel_button: iced::button::State::default(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vlt: &Vault,
        warning: Option<&Error>,
        panel: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let mut col = Column::new();
        if let Some(error) = warning {
            col = col.push(
                Container::new(card::alert_warning(Container::new(text::small(
                    &error.to_string(),
                ))))
                .padding(20),
            )
        }
        let header = Row::new().push(col.width(Length::Fill)).push(
            Container::new(
                button::cancel(
                    &mut self.cancel_button,
                    Container::new(text::simple("X Close")).padding(10),
                )
                .on_press(Message::Vault(VaultMessage::Select(vlt.outpoint()))),
            )
            .width(Length::Shrink),
        );
        Container::new(
            Scrollable::new(&mut self.scroll).push(
                Container::new(
                    Column::new()
                        .push(header)
                        .push(
                            Container::new(
                                Column::new()
                                    .push(
                                        Container::new(text::simple("Vault Detail"))
                                            .width(Length::Fill)
                                            .align_x(Align::Center),
                                    )
                                    .push(Container::new(vault(ctx, &mut self.copy_button, vlt)))
                                    .push(Container::new(panel))
                                    .spacing(20),
                            )
                            .max_width(1000)
                            .align_x(Align::Center)
                            .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .width(Length::Fill)
                .align_x(Align::Center),
            ),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .into()
    }
}

fn vault<'a>(
    ctx: &Context,
    copy_button: &'a mut iced::button::State,
    vlt: &Vault,
) -> Container<'a, Message> {
    card::simple(Container::new(
        Column::new()
            .push(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(vault_badge(&vlt))
                                .push(
                                    Column::new()
                                        .push(
                                            Row::new()
                                                .push(text::small(&vlt.txid))
                                                .push(button::clipboard(
                                                    copy_button,
                                                    Message::Clipboard(vlt.txid.to_string()),
                                                ))
                                                .align_items(Align::Center),
                                        )
                                        .push(text::small(&format!(
                                            "{} ( {} )",
                                            &vlt.status,
                                            NaiveDateTime::from_timestamp(vlt.updated_at, 0)
                                        ))),
                                )
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(vlt.amount),
                                ))))
                                .push(text::simple(&ctx.converter.unit.to_string())),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            )
            .spacing(20),
    ))
}

#[derive(Debug)]
pub struct VaultOnChainTransactionsPanel {
    /// button used for ack fund panel or delegate vault panel or cancel spending panel
    /// depending of vault status.
    action_button: iced::button::State,
}

impl VaultOnChainTransactionsPanel {
    pub fn new() -> Self {
        VaultOnChainTransactionsPanel {
            action_button: iced::button::State::new(),
        }
    }
    pub fn view(
        &mut self,
        ctx: &Context,
        vault: &Vault,
        txs: &VaultTransactions,
    ) -> Element<Message> {
        let mut col = Column::new().spacing(20);
        if ctx.role == Role::Stakeholder {
            match vault.status {
                VaultStatus::Funded => {
                    col = col.push(card::white(Container::new(
                        Row::new()
                            .push(
                                Container::new(text::simple(
                                    "This vault needs to be acknowledged ",
                                ))
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::important(
                                        &mut self.action_button,
                                        button::button_content(None, "ACK vault"),
                                    )
                                    .on_press(Message::Vault(
                                        VaultMessage::Acknowledge(vault.outpoint()),
                                    )),
                                )
                                .width(Length::Shrink),
                            )
                            .align_items(Align::Center),
                    )))
                }
                VaultStatus::Secured => {
                    col = col.push(card::white(Container::new(
                        Row::new()
                            .push(
                                Container::new(text::simple(
                                    "Do you want to delegate vault to manager ? ",
                                ))
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::important(
                                        &mut self.action_button,
                                        button::button_content(None, "Delegate vault"),
                                    )
                                    .on_press(Message::Vault(
                                        VaultMessage::Delegate(vault.outpoint()),
                                    )),
                                )
                                .width(Length::Shrink),
                            )
                            .align_items(Align::Center),
                    )))
                }
                VaultStatus::Unvaulted | VaultStatus::Unvaulting => {
                    col = col.push(card::white(Container::new(
                        Row::new()
                            .push(
                                Container::new(text::simple(
                                    "Funds are moving, do you want to revault it ?",
                                ))
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::primary(
                                        &mut self.action_button,
                                        button::button_content(None, "Revault"),
                                    )
                                    .on_press(Message::Vault(
                                        VaultMessage::Delegate(vault.outpoint()),
                                    )),
                                )
                                .width(Length::Shrink),
                            )
                            .align_items(Align::Center),
                    )))
                }
                _ => {}
            };
        } else {
            if vault.status == VaultStatus::Unvaulted || vault.status == VaultStatus::Unvaulting {
                col = col.push(card::white(Container::new(
                    Row::new()
                        .push(
                            Container::new(text::simple(
                                "Funds are moving, do you want to revault it ?",
                            ))
                            .width(Length::Fill),
                        )
                        .push(
                            Container::new(
                                button::primary(
                                    &mut self.action_button,
                                    button::button_content(None, "Revault"),
                                )
                                .on_press(Message::Vault(VaultMessage::Delegate(vault.outpoint()))),
                            )
                            .width(Length::Shrink),
                        )
                        .align_items(Align::Center),
                )))
            }
        }

        col = col.push(Container::new(text::bold(text::simple(
            "Onchain transactions:",
        ))));
        if let Some(tx) = &txs.spend {
            col = col.push(transaction(ctx, "Spend transaction", &tx));
        }
        if let Some(tx) = &txs.cancel {
            col = col.push(transaction(ctx, "Cancel transaction", &tx));
        }
        if let Some(tx) = &txs.unvault_emergency {
            col = col.push(transaction(ctx, "Unvault Emergency transaction", &tx));
        }
        if let Some(tx) = &txs.emergency {
            col = col.push(transaction(ctx, "Emergency transaction", &tx));
        }
        if let Some(tx) = &txs.unvault {
            col = col.push(transaction(ctx, "Unvault transaction", &tx));
        }
        col = col.push(transaction(ctx, "Deposit transaction", &txs.deposit));
        Container::new(Column::new().push(col)).into()
    }
}

fn transaction<'a, T: 'a>(
    ctx: &Context,
    title: &str,
    transaction: &BroadcastedTransaction,
) -> Container<'a, T> {
    Container::new(
        Column::new()
            .push(separation().width(Length::Fill))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Container::new(text::bold(text::simple(title))).width(Length::Fill),
                            )
                            .push(
                                Container::new(text::bold(text::small(
                                    &transaction.tx.txid().to_string(),
                                )))
                                .width(Length::Shrink),
                            ),
                    )
                    .push(text::small(&format!(
                        "Received at {}",
                        NaiveDateTime::from_timestamp(transaction.received_at, 0)
                    )))
                    .push(text::small(
                        &if let Some(blockheight) = &transaction.blockheight {
                            format!("Blockheight: {}", blockheight)
                        } else {
                            "Not in a block".to_string()
                        },
                    )),
            )
            .push(
                Container::new(input_and_outputs(ctx, &transaction))
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(20),
    )
}

fn input_and_outputs<'a, T: 'a>(
    ctx: &Context,
    broadcasted: &BroadcastedTransaction,
) -> Container<'a, T> {
    let mut col_input = Column::new()
        .push(text::bold(text::simple("Inputs")))
        .spacing(10);
    for input in &broadcasted.tx.input {
        col_input = col_input.push(card::simple(Container::new(text::small(&format!(
            "{}",
            input.previous_output
        )))));
    }
    let mut col_output = Column::new()
        .push(text::bold(text::simple("Outputs")))
        .spacing(10);
    for output in &broadcasted.tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network);
        let mut col = Column::new();
        if let Some(a) = addr {
            col = col.push(text::small(&a.to_string()))
        } else {
            col = col.push(text::small(&output.script_pubkey.to_string()))
        }
        col_output = col_output.push(card::simple(Container::new(col.push(text::bold(
            text::small(&ctx.converter.converts(output.value).to_string()),
        )))));
    }
    Container::new(Row::new().push(col_input).push(col_output).spacing(20))
}

/// vault_badge returns a badge headlining the vault status.
fn vault_badge<'a, T: 'a>(vault: &Vault) -> Container<'a, T> {
    match &vault.status {
        VaultStatus::Unconfirmed => badge::vault_unconfirmed(),
        VaultStatus::Funded
        | VaultStatus::Securing
        | VaultStatus::Secured
        | VaultStatus::Activating
        | VaultStatus::Active => badge::tx_deposit(),
        VaultStatus::Unvaulting | VaultStatus::Unvaulted => badge::vault_unvaulting(),
        VaultStatus::Canceling | VaultStatus::EmergencyVaulting => badge::vault_canceling(),
        VaultStatus::Canceled | VaultStatus::EmergencyVaulted => badge::vault_canceled(),
        VaultStatus::Spendable | VaultStatus::Spending => badge::vault_spending(),
        VaultStatus::Spent => badge::vault_spent(),
    }
}

pub trait VaultView {
    fn new() -> Self;
    fn view(&mut self, ctx: &Context, vault: &Vault) -> Element<Message>;
}

#[derive(Debug, Clone)]
pub struct VaultListItemView {
    state: iced::button::State,
}

impl VaultView for VaultListItemView {
    fn new() -> Self {
        VaultListItemView {
            state: iced::button::State::new(),
        }
    }

    fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        let updated_at = NaiveDateTime::from_timestamp(vault.updated_at, 0);
        button::white_card_button(
            &mut self.state,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(vault_badge(&vault))
                                .push(
                                    Column::new()
                                        .push(text::bold(text::small(&vault.address)))
                                        .push(text::small(&format!(
                                            "{} ( {} )",
                                            &vault.status, updated_at
                                        ))),
                                )
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(vault.amount),
                                ))))
                                .push(text::small(&format!(" {}", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(Message::Vault(VaultMessage::Select(vault.outpoint())))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct AcknowledgeVaultListItemView {
    select_button: iced::button::State,
}

impl VaultView for AcknowledgeVaultListItemView {
    fn new() -> Self {
        Self {
            select_button: iced::button::State::new(),
        }
    }

    fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        if vault.status == VaultStatus::Funded || vault.status == VaultStatus::Unconfirmed {
            vault_ack_pending(&mut self.select_button, ctx, vault).map(Message::Vault)
        } else {
            vault_ack_signed(ctx, vault).map(Message::Vault)
        }
    }
}

fn vault_ack_signed<'a, T: 'a>(ctx: &Context, deposit: &Vault) -> Element<'a, T> {
    card::white(Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(badge::shield_success())
                        .push(
                            Container::new(text::success(text::bold(text::small(
                                &deposit.address,
                            ))))
                            .align_y(Align::Center),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(text::success(text::bold(text::simple(&format!(
                            "{}",
                            ctx.converter.converts(deposit.amount),
                        )))))
                        .push(text::small(&format!(" {}", ctx.converter.unit)))
                        .align_items(Align::Center),
                )
                .width(Length::Shrink),
            )
            .spacing(20)
            .align_items(Align::Center),
    ))
    .into()
}

fn vault_ack_pending<'a>(
    state: &'a mut iced::button::State,
    ctx: &Context,
    deposit: &Vault,
) -> Element<'a, VaultMessage> {
    Container::new(
        button::white_card_button(
            state,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::shield_notif())
                                .push(
                                    Container::new(text::bold(text::small(&deposit.address)))
                                        .align_y(Align::Center),
                                )
                                .spacing(20)
                                .align_items(Align::Center),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(deposit.amount),
                                ))))
                                .push(text::small(&format!(" {}", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(VaultMessage::Select(deposit.outpoint())),
    )
    .into()
}

#[derive(Debug, Clone)]
pub struct DelegateVaultListItemView {
    select_button: iced::button::State,
    delegate_button: iced::button::State,
}

impl VaultView for DelegateVaultListItemView {
    fn new() -> Self {
        DelegateVaultListItemView {
            select_button: iced::button::State::new(),
            delegate_button: iced::button::State::new(),
        }
    }

    fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        let updated_at = NaiveDateTime::from_timestamp(vault.updated_at, 0);
        let mut row = Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(text::bold(text::simple(&format!(
                            "{}",
                            ctx.converter.converts(vault.amount),
                        ))))
                        .push(text::small(&format!(" {}", ctx.converter.unit)))
                        .align_items(Align::Center),
                )
                .width(Length::Shrink),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(Container::new(text::small(&format!("{}", updated_at))))
                        .push(Container::new(text::bold(text::small(&format!(
                            "{}...",
                            &vault.address[0..10]
                        )))))
                        .spacing(10),
                )
                .width(Length::Fill),
            )
            .spacing(20)
            .align_items(Align::Center);
        if vault.status == VaultStatus::Secured {
            row = row.push(
                Container::new(
                    button::important(
                        &mut self.delegate_button,
                        button::button_content(None, "delegate"),
                    )
                    .on_press(Message::Vault(VaultMessage::Delegate(vault.outpoint()))),
                )
                .width(Length::Shrink),
            )
        } else if vault.status == VaultStatus::Funded {
            row = row.push(
                Container::new(
                    button::important(
                        &mut self.delegate_button,
                        button::button_content(None, "Ack then delegate"),
                    )
                    .on_press(Message::Vault(VaultMessage::Acknowledge(vault.outpoint()))),
                )
                .width(Length::Shrink),
            )
        } else if vault.status == VaultStatus::Securing {
            row = row.push(Container::new(text::small("waiting ack...")).width(Length::Shrink))
        } else if vault.status == VaultStatus::Activating {
            row = row.push(Container::new(text::small("activating...")).width(Length::Shrink))
        }
        row.push(
            button::transparent(
                &mut self.select_button,
                button::button_content(None, "view"),
            )
            .on_press(Message::Vault(VaultMessage::Select(vault.outpoint()))),
        )
        .into()
    }
}

#[derive(Debug)]
pub struct AcknowledgeVaultView {
    retry_button: iced::button::State,
}

impl AcknowledgeVaultView {
    pub fn new() -> Self {
        AcknowledgeVaultView {
            retry_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        deposit: &Vault,
        emergency_tx: &(Psbt, bool),
        emergency_unvault_tx: &(Psbt, bool),
        cancel_tx: &(Psbt, bool),
        signer: Element<'a, VaultMessage>,
    ) -> Element<'a, VaultMessage> {
        let mut row_transactions = Row::new();
        let (_, emergency_signed) = emergency_tx;
        if *emergency_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple("   Emergency TX")))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let (_, emergency_unvault_signed) = emergency_unvault_tx;
        if *emergency_unvault_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple(
                            "   Emergency unvault TX",
                        )))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else if *emergency_signed {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency Unvault TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::grey(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency Unvault TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let (_, cancel_signed) = cancel_tx;
        if *cancel_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple("   Cancel TX")))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else if *emergency_unvault_signed {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Cancel TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::grey(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Cancel TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let mut col = Column::new()
            .push(Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::shield())
                                .push(
                                    Container::new(text::bold(text::small(&deposit.address)))
                                        .align_y(Align::Center),
                                )
                                .spacing(20)
                                .align_items(Align::Center),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(deposit.amount)
                                ))))
                                .push(text::small(&format!(" {}", ctx.converter.unit)))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ))
            .push(separation().width(Length::Fill))
            .push(row_transactions.spacing(10))
            .push(signer)
            .spacing(20)
            .push(Column::new());

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(
                Column::new()
                    .push(Container::new(text::simple(&format!(
                        "Failed to connect to revaultd: {}",
                        error
                    ))))
                    .push(
                        button::primary(
                            &mut self.retry_button,
                            button::button_content(None, "Retry"),
                        )
                        .on_press(VaultMessage::Retry),
                    )
                    .spacing(20),
            )))
        }

        card::white(Container::new(col)).into()
    }
}

#[derive(Debug, Clone)]
pub struct DelegateVaultView {
    back_button: iced::button::State,
}

impl DelegateVaultView {
    pub fn new() -> Self {
        Self {
            back_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        _ctx: &Context,
        _vault: &Vault,
        warning: Option<&Error>,
        signer: Element<'a, SignMessage>,
    ) -> Element<'a, Message> {
        let mut col = Column::new();
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }
        col.push(button::transparent(
                &mut self.back_button,
                Container::new(text::small("< vault transactions")),
            ).on_press(Message::Vault(VaultMessage::ListOnchainTransaction)))
            .push(card::white(Container::new(
                Column::new()
                    .push(
                        Column::new()
                            .push(text::bold(text::simple("Delegate vault to manager")))
                            .push(text::simple("the unvault transaction must be signed in order to delegate the fund to the managers.")),
                    )
                    .push(signer.map(move |msg| match msg {
                        SignMessage::Clipboard(s) => Message::Clipboard(s),
                        _ => Message::Vault(VaultMessage::Sign(msg)),
                    }))
                    .spacing(20),
            )))
            .into()
    }
}
