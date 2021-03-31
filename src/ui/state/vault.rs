use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use iced::{Command, Element};
use std::sync::Arc;

use crate::{
    revault::TransactionKind,
    revaultd::{
        model::{self, RevocationTransactions, VaultTransactions},
        RevaultD,
    },
    ui::{
        error::Error,
        message::{Message, SignMessage, VaultMessage},
        state::{
            cmd::{
                get_onchain_txs, get_revocation_txs, get_unvault_tx, set_revocation_txs,
                set_unvault_tx,
            },
            sign::SignState,
        },
        view::{
            vault::{
                AcknowledgeVaultView, DelegateVaultView, VaultModal, VaultOnChainTransactionsPanel,
                VaultView,
            },
            Context,
        },
    },
};

#[derive(Debug)]
pub struct VaultListItem<T> {
    pub vault: model::Vault,
    view: T,
}

impl<T: VaultView> VaultListItem<T> {
    pub fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            view: T::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault)
    }
}

/// SelectedVault is a widget displaying information of a vault
/// and handling user action on it.
#[derive(Debug)]
pub struct Vault {
    pub vault: model::Vault,
    warning: Option<Error>,
    section: VaultSection,
    view: VaultModal,
}

impl Vault {
    pub fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            section: VaultSection::Unloaded,
            view: VaultModal::new(),
            warning: None,
        }
    }

    pub fn update(&mut self, revaultd: Arc<RevaultD>, message: VaultMessage) -> Command<Message> {
        match message {
            VaultMessage::ListOnchainTransaction => {
                return Command::perform(
                    get_onchain_txs(revaultd.clone(), self.vault.outpoint()),
                    |res| Message::Vault(VaultMessage::OnChainTransactions(res)),
                );
            }
            VaultMessage::OnChainTransactions(res) => match res {
                Ok(txs) => self.section = VaultSection::new_onchain_txs_section(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::UnvaultTransaction(res) => match res {
                Ok(tx) => self.section = VaultSection::new_delegate_section(tx.unvault_tx),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::RevocationTransactions(res) => match res {
                Ok(tx) => self.section = VaultSection::new_ack_section(tx),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::Delegate(outpoint) => {
                if outpoint == self.vault.outpoint() {
                    return Command::perform(
                        get_unvault_tx(revaultd.clone(), self.vault.outpoint()),
                        |res| Message::Vault(VaultMessage::UnvaultTransaction(res)),
                    );
                }
            }
            VaultMessage::Acknowledge(outpoint) => {
                if outpoint == self.vault.outpoint() {
                    return Command::perform(
                        get_revocation_txs(revaultd.clone(), self.vault.outpoint()),
                        |res| Message::Vault(VaultMessage::RevocationTransactions(res)),
                    );
                }
            }
            _ => {
                return self
                    .section
                    .update(revaultd, &self.vault, message)
                    .map(Message::Vault);
            }
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            &self.vault,
            self.warning.as_ref(),
            self.section.view(ctx, &self.vault),
        )
    }

    pub fn load(&self, revaultd: Arc<RevaultD>) -> Command<Message> {
        Command::perform(
            get_onchain_txs(revaultd.clone(), self.vault.outpoint()),
            |res| Message::Vault(VaultMessage::OnChainTransactions(res)),
        )
    }
}

#[derive(Debug)]
pub enum VaultSection {
    Unloaded,
    OnchainTransactions {
        txs: VaultTransactions,
        view: VaultOnChainTransactionsPanel,
    },
    Delegate {
        signer: SignState,
        view: DelegateVaultView,
        warning: Option<Error>,
    },
    Acknowledge {
        emergency_tx: (Psbt, bool),
        emergency_unvault_tx: (Psbt, bool),
        cancel_tx: (Psbt, bool),
        warning: Option<Error>,
        view: AcknowledgeVaultView,
        signer: SignState,
    },
}

impl VaultSection {
    pub fn new_onchain_txs_section(txs: VaultTransactions) -> Self {
        Self::OnchainTransactions {
            txs,
            view: VaultOnChainTransactionsPanel::new(),
        }
    }

    pub fn new_delegate_section(unvault_tx: Psbt) -> Self {
        Self::Delegate {
            signer: SignState::new(unvault_tx, TransactionKind::Unvault),
            view: DelegateVaultView::new(),
            warning: None,
        }
    }

    pub fn new_ack_section(txs: RevocationTransactions) -> Self {
        Self::Acknowledge {
            emergency_tx: (txs.emergency_tx.clone(), false),
            emergency_unvault_tx: (txs.emergency_unvault_tx.clone(), false),
            cancel_tx: (txs.cancel_tx.clone(), false),
            signer: SignState::new(txs.emergency_tx, TransactionKind::Emergency),
            view: AcknowledgeVaultView::new(),
            warning: None,
        }
    }

    fn update(
        &mut self,
        revaultd: Arc<RevaultD>,
        vault: &model::Vault,
        message: VaultMessage,
    ) -> Command<VaultMessage> {
        match message {
            VaultMessage::Signed(res) => match self {
                VaultSection::Delegate {
                    warning, signer, ..
                } => match res {
                    Ok(()) => {
                        signer.update(SignMessage::Success);
                    }
                    Err(e) => {
                        *warning = Some(Error::RevaultDError(e));
                    }
                },
                VaultSection::Acknowledge {
                    warning, signer, ..
                } => match res {
                    Ok(()) => {
                        signer.update(SignMessage::Success);
                    }
                    Err(e) => {
                        *warning = Some(Error::RevaultDError(e));
                    }
                },
                _ => {}
            },
            VaultMessage::Sign(msg) => match self {
                VaultSection::Delegate { signer, .. } => {
                    signer.update(msg);
                    if let Some(psbt) = &signer.signed_psbt {
                        return Command::perform(
                            set_unvault_tx(revaultd.clone(), vault.outpoint(), psbt.clone()),
                            VaultMessage::Signed,
                        );
                    }
                }
                VaultSection::Acknowledge {
                    signer,
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                    ..
                } => {
                    signer.update(msg);
                    if let Some(psbt) = &signer.signed_psbt {
                        match signer.transaction_kind {
                            TransactionKind::Emergency => {
                                *emergency_tx = (psbt.clone(), true);
                                *signer = SignState::new(
                                    emergency_unvault_tx.0.clone(),
                                    TransactionKind::EmergencyUnvault,
                                );
                            }
                            TransactionKind::EmergencyUnvault => {
                                *emergency_unvault_tx = (psbt.clone(), true);
                                *signer =
                                    SignState::new(cancel_tx.0.clone(), TransactionKind::Cancel);
                            }
                            TransactionKind::Cancel => {
                                *cancel_tx = (psbt.clone(), true);
                                return Command::perform(
                                    set_revocation_txs(
                                        revaultd,
                                        vault.outpoint(),
                                        emergency_tx.0.clone(),
                                        emergency_unvault_tx.0.clone(),
                                        cancel_tx.0.clone(),
                                    ),
                                    VaultMessage::Signed,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context, vault: &model::Vault) -> Element<Message> {
        match self {
            Self::Unloaded => iced::Container::new(iced::Column::new()).into(),
            Self::OnchainTransactions { txs, view } => view.view(ctx, &vault, &txs),
            Self::Delegate {
                signer,
                view,
                warning,
                ..
            } => view.view(ctx, &vault, warning.as_ref(), signer.view(ctx)),
            Self::Acknowledge {
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                warning,
                view,
                signer,
            } => view
                .view(
                    ctx,
                    warning.as_ref(),
                    vault,
                    &emergency_tx,
                    &emergency_unvault_tx,
                    &cancel_tx,
                    signer.view(ctx).map(VaultMessage::Sign),
                )
                .map(Message::Vault),
        }
    }
}
