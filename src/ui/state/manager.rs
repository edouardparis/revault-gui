use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, get_spend_tx, list_spend_txs, list_vaults, update_spend_tx},
    vault::{Vault, VaultListItem},
    State,
};

use crate::revaultd::{
    model::{self, VaultStatus},
    RevaultD,
};

use crate::revault::TransactionKind;

use crate::ui::{
    error::Error,
    message::{InputMessage, Message, RecipientMessage, SignMessage, SpendTxMessage, VaultMessage},
    state::{sign::SignState, SpendTransactionListItem, SpendTransactionState},
    view::manager::{
        manager_send_input_view, ManagerImportTransactionView, ManagerSelectFeeView,
        ManagerSelectInputsView, ManagerSelectOutputsView, ManagerSendOutputView,
        ManagerSendWelcomeView, ManagerSignView, ManagerSpendTransactionCreatedView,
    },
    view::{vault::VaultListItemView, Context, ManagerHomeView, ManagerNetworkView},
};

#[derive(Debug)]
pub struct ManagerHomeState {
    revaultd: Arc<RevaultD>,
    view: ManagerHomeView,

    /// balance as active and inactive tuple.
    balance: (u64, u64),
    blockheight: u64,
    warning: Option<Error>,

    unvaulting_vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,

    spend_txs: Vec<SpendTransactionListItem>,
    selected_spend_tx: Option<SpendTransactionState>,
}

impl ManagerHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerHomeState {
            revaultd,
            balance: (0, 0),
            view: ManagerHomeView::new(),
            blockheight: 0,
            unvaulting_vaults: Vec::new(),
            warning: None,
            selected_vault: None,
            spend_txs: Vec::new(),
            selected_spend_tx: None,
        }
    }

    pub fn update_spend_txs(&mut self, txs: Vec<model::SpendTx>) {
        self.spend_txs = txs.into_iter().map(SpendTransactionListItem::new).collect();
    }

    pub fn on_spend_tx_select(&mut self, psbt: Psbt) -> Command<Message> {
        if let Some(selected) = &self.selected_spend_tx {
            if selected.psbt.global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid() {
                self.selected_spend_tx = None;
                return Command::none();
            }
        }

        if self
            .spend_txs
            .iter()
            .find(|item| item.tx.psbt.global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid())
            .is_some()
        {
            let selected_spend_tx = SpendTransactionState::new(self.revaultd.clone(), psbt);
            let cmd = selected_spend_tx.load();
            self.selected_spend_tx = Some(selected_spend_tx);
            return cmd;
        };
        Command::none()
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.calculate_balance(&vaults);
        self.unvaulting_vaults = vaults
            .into_iter()
            .filter_map(|vlt| {
                if vlt.status == VaultStatus::Unvaulting || vlt.status == VaultStatus::Unvaulted {
                    Some(VaultListItem::new(vlt))
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return Command::none();
            }
        }

        if let Some(selected) = self
            .unvaulting_vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.load(self.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd;
        };
        Command::none()
    }

    pub fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        let mut active_amount: u64 = 0;
        let mut inactive_amount: u64 = 0;
        for vault in vaults {
            match vault.status {
                VaultStatus::Active | VaultStatus::Unvaulting | VaultStatus::Unvaulted => {
                    active_amount += vault.amount
                }
                VaultStatus::Secured | VaultStatus::Funded | VaultStatus::Unconfirmed => {
                    inactive_amount += vault.amount
                }
                _ => {}
            }
        }

        self.balance = (active_amount, inactive_amount);
    }
}

impl State for ManagerHomeState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                return self.on_spend_tx_select(psbt);
            }
            Message::SpendTx(msg) => {
                if let Some(tx) = &mut self.selected_spend_tx {
                    return tx.update(Message::SpendTx(msg));
                }
            }
            Message::SpendTransactions(res) => match res {
                Ok(txs) => self.update_spend_txs(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vault(VaultMessage::Select(outpoint)) => {
                return self.on_vault_select(outpoint)
            }
            Message::Vault(msg) => {
                if let Some(vault) = &mut self.selected_vault {
                    return vault.update(self.revaultd.clone(), msg);
                }
            }
            Message::BlockHeight(b) => match b {
                Ok(height) => {
                    self.blockheight = height.into();
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                }
            },
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }

        if let Some(tx) = &mut self.selected_spend_tx {
            return tx.view(ctx);
        }

        self.view.view(
            ctx,
            self.warning.as_ref().into(),
            self.spend_txs
                .iter_mut()
                .map(|tx| tx.view(ctx).map(Message::SpendTx))
                .collect(),
            self.unvaulting_vaults
                .iter_mut()
                .map(|v| v.view(ctx))
                .collect(),
            &self.balance,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(list_vaults(self.revaultd.clone(), None), Message::Vaults),
            Command::perform(
                list_spend_txs(self.revaultd.clone()),
                Message::SpendTransactions,
            ),
        ])
    }
}

impl From<ManagerHomeState> for Box<dyn State> {
    fn from(s: ManagerHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

pub enum ManagerSendState {
    SendTransactionDetail(SpendTransactionState),
    ImportSendTransaction(ManagerImportSendTransactionState),
    CreateSendTransaction(ManagerCreateSendTransactionState),
}

impl ManagerSendState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        Self::CreateSendTransaction(ManagerCreateSendTransactionState::new(revaultd))
    }
}

impl State for ManagerSendState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Self::CreateSendTransaction(state) => match message {
                Message::SpendTx(SpendTxMessage::Import) => {
                    *self = ManagerSendState::ImportSendTransaction(
                        ManagerImportSendTransactionState::new(state.revaultd.clone()),
                    );
                    self.load()
                }
                _ => state.update(message),
            },
            Self::ImportSendTransaction(state) => match message {
                Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                    *self = ManagerSendState::SendTransactionDetail(SpendTransactionState::new(
                        state.revaultd.clone(),
                        psbt,
                    ));
                    self.load()
                }
                _ => state.update(message),
            },
            Self::SendTransactionDetail(state) => state.update(message),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::CreateSendTransaction(state) => state.view(ctx),
            Self::ImportSendTransaction(state) => state.view(ctx),
            Self::SendTransactionDetail(state) => state.view(ctx),
        }
    }

    fn load(&self) -> Command<Message> {
        match self {
            Self::CreateSendTransaction(state) => state.load(),
            Self::ImportSendTransaction(state) => state.load(),
            Self::SendTransactionDetail(state) => state.load(),
        }
    }
}

#[derive(Debug)]
pub struct ManagerImportSendTransactionState {
    revaultd: Arc<RevaultD>,
    psbt_imported: Option<Psbt>,
    psbt_input: String,
    warning: Option<String>,

    view: ManagerImportTransactionView,
}

impl ManagerImportSendTransactionState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        Self {
            revaultd,
            psbt_imported: None,
            psbt_input: "".to_string(),
            warning: None,
            view: ManagerImportTransactionView::new(),
        }
    }

    pub fn parse_pbst(&self) -> Option<Psbt> {
        bitcoin::base64::decode(&self.psbt_input)
            .ok()
            .and_then(|bytes| bitcoin::consensus::encode::deserialize(&bytes).ok())
    }
}

impl State for ManagerImportSendTransactionState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Updated(res)) => match res {
                Ok(()) => self.psbt_imported = self.parse_pbst(),
                Err(e) => self.warning = Some(e.to_string()),
            },
            Message::SpendTx(SpendTxMessage::PsbtEdited(psbt)) => {
                self.warning = None;
                self.psbt_input = psbt;
            }
            Message::SpendTx(SpendTxMessage::Import) => {
                if !self.psbt_input.is_empty() {
                    if let Some(psbt) = self.parse_pbst() {
                        return Command::perform(
                            update_spend_tx(self.revaultd.clone(), psbt),
                            |res| Message::SpendTx(SpendTxMessage::Updated(res)),
                        );
                    } else {
                        self.warning = Some("Please enter valid PSBT".to_string());
                    }
                } else {
                    self.warning = Some("Please enter valid PSBT".to_string());
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, _ctx: &Context) -> Element<Message> {
        self.view.view(
            &self.psbt_input,
            self.psbt_imported.as_ref(),
            self.warning.as_ref(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::none()
    }
}

#[derive(Debug)]
enum ManagerSendStep {
    WelcomeUser(ManagerSendWelcomeView),
    SelectOutputs(ManagerSelectOutputsView),
    SelectInputs(ManagerSelectInputsView),
    SelectFee(ManagerSelectFeeView),
    Sign {
        signer: SignState,
        view: ManagerSignView,
    },
    Success(ManagerSpendTransactionCreatedView),
}

#[derive(Debug)]
pub struct ManagerCreateSendTransactionState {
    revaultd: Arc<RevaultD>,

    warning: Option<Error>,

    vaults: Vec<ManagerSendInput>,
    outputs: Vec<ManagerSendOutput>,
    feerate: u32,
    psbt: Option<(Psbt, u32)>,
    processing: bool,

    step: ManagerSendStep,
}

impl ManagerCreateSendTransactionState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        Self {
            revaultd,
            step: ManagerSendStep::WelcomeUser(ManagerSendWelcomeView::new()),
            warning: None,
            vaults: Vec::new(),
            outputs: vec![ManagerSendOutput::new()],
            feerate: 20,
            psbt: None,
            processing: false,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.vaults = vaults
            .into_iter()
            .map(|vlt| ManagerSendInput::new(vlt))
            .collect();
    }

    pub fn input_amount(&self) -> u64 {
        let mut input_amount = 0;
        for input in &self.vaults {
            if input.selected {
                input_amount += input.vault.amount;
            }
        }
        input_amount
    }

    pub fn output_amount(&self) -> u64 {
        let mut output_amount = 0;
        for output in &self.outputs {
            if let Ok(amount) = output.amount() {
                output_amount += amount;
            }
        }
        output_amount
    }

    pub fn selected_inputs(&self) -> Vec<model::Vault> {
        self.vaults
            .iter()
            .cloned()
            .filter_map(|input| {
                if input.selected {
                    Some(input.vault)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl State for ManagerCreateSendTransactionState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SpendTransaction(res) => {
                self.processing = false;
                match res {
                    Ok(tx) => {
                        self.psbt = Some((tx.spend_tx, tx.feerate));
                    }
                    Err(e) => self.warning = Some(Error::RevaultDError(e)),
                }
            }
            Message::SpendTx(SpendTxMessage::Generate) => {
                self.processing = true;
                self.warning = None;
                let inputs = self
                    .selected_inputs()
                    .into_iter()
                    .map(|input| input.outpoint())
                    .collect();

                let outputs: HashMap<String, u64> = self
                    .outputs
                    .iter()
                    .map(|output| (output.address.clone(), output.amount().unwrap()))
                    .collect();

                return Command::perform(
                    get_spend_tx(self.revaultd.clone(), inputs, outputs, self.feerate),
                    Message::SpendTransaction,
                );
            }
            Message::SpendTx(SpendTxMessage::FeerateEdited(feerate)) => {
                if !self.processing {
                    self.feerate = feerate;
                    self.psbt = None;
                }
            }
            Message::Vaults(res) => match res {
                Ok(vlts) => self.update_vaults(vlts),
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            Message::SpendTx(SpendTxMessage::Signed(res)) => match res {
                Ok(_) => {
                    if let ManagerSendStep::Sign { signer, .. } = &mut self.step {
                        // During this step state has a generated psbt
                        // and signer has a signed psbt.
                        self.psbt = Some((
                            signer.signed_psbt.clone().expect("As the received message is a sign success, the psbt should not be None"),
                            self.psbt.clone().expect("As the received message is a sign success, the psbt should not be None").1,
                        ));
                        signer.update(SignMessage::Success);
                        self.step =
                            ManagerSendStep::Success(ManagerSpendTransactionCreatedView::new());
                    };
                }
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            Message::SpendTx(SpendTxMessage::Sign(msg)) => match &mut self.step {
                ManagerSendStep::Sign { signer, .. } => {
                    signer
                        .update(msg)
                        .map(|m| Message::SpendTx(SpendTxMessage::Sign(m)));
                    if let Some(psbt) = &signer.signed_psbt {
                        return Command::perform(
                            update_spend_tx(self.revaultd.clone(), psbt.clone()),
                            |res| Message::SpendTx(SpendTxMessage::Signed(res)),
                        );
                    }
                }
                _ => (),
            },
            Message::Next => match self.step {
                ManagerSendStep::WelcomeUser(_) => {
                    self.step = ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new());
                }
                ManagerSendStep::SelectOutputs(_) => {
                    self.step = ManagerSendStep::SelectInputs(ManagerSelectInputsView::new());
                }
                ManagerSendStep::SelectInputs(_) => {
                    self.step = ManagerSendStep::SelectFee(ManagerSelectFeeView::new());
                }
                ManagerSendStep::SelectFee(_) => {
                    if let Some((psbt, _)) = &self.psbt {
                        self.step = ManagerSendStep::Sign {
                            signer: SignState::new(psbt.clone(), TransactionKind::Spend),
                            view: ManagerSignView::new(),
                        };
                    }
                }
                _ => (),
            },
            Message::Previous => {
                self.step = match self.step {
                    ManagerSendStep::SelectInputs(_) => {
                        ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new())
                    }
                    ManagerSendStep::SelectFee(_) => {
                        ManagerSendStep::SelectInputs(ManagerSelectInputsView::new())
                    }
                    ManagerSendStep::Sign { .. } => {
                        ManagerSendStep::SelectFee(ManagerSelectFeeView::new())
                    }
                    _ => ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new()),
                }
            }
            Message::AddRecipient => self.outputs.push(ManagerSendOutput::new()),
            Message::Recipient(i, RecipientMessage::Delete) => {
                self.outputs.remove(i);
            }
            Message::Input(i, msg) => {
                self.psbt = None;
                if let Some(input) = self.vaults.get_mut(i) {
                    input.update(msg);
                }
            }
            Message::Recipient(i, msg) => {
                self.psbt = None;
                if let Some(output) = self.outputs.get_mut(i) {
                    output.update(msg);
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        let selected_inputs = self.selected_inputs();
        let input_amount = self.input_amount();
        let output_amount = self.output_amount();
        match &mut self.step {
            ManagerSendStep::WelcomeUser(v) => v.view(),
            ManagerSendStep::SelectOutputs(v) => {
                let valid = !self.outputs.iter().any(|o| !o.valid());
                v.view(
                    self.outputs
                        .iter_mut()
                        .enumerate()
                        .map(|(i, v)| v.view().map(move |msg| Message::Recipient(i, msg)))
                        .collect(),
                    valid,
                )
            }
            ManagerSendStep::SelectInputs(v) => v.view(
                self.vaults
                    .iter_mut()
                    .enumerate()
                    .map(|(i, v)| v.view(ctx).map(move |msg| Message::Input(i, msg)))
                    .collect(),
                input_amount > output_amount,
            ),
            ManagerSendStep::SelectFee(v) => v.view(
                ctx,
                &selected_inputs,
                &self.feerate,
                self.psbt.as_ref(),
                &self.processing,
                self.warning.as_ref(),
            ),
            ManagerSendStep::Sign { signer, view } => {
                let (psbt, feerate) = self.psbt.as_ref().unwrap();
                view.view(
                    ctx,
                    &selected_inputs,
                    &psbt,
                    &feerate,
                    self.warning.as_ref(),
                    signer
                        .view(ctx)
                        .map(|m| Message::SpendTx(SpendTxMessage::Sign(m))),
                )
            }
            ManagerSendStep::Success(v) => {
                let (psbt, _) = self.psbt.as_ref().unwrap();
                v.view(ctx, &selected_inputs, &psbt, &self.feerate)
            }
        }
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(self.revaultd.clone(), Some(&[VaultStatus::Active])),
            Message::Vaults,
        )])
    }
}

impl From<ManagerSendState> for Box<dyn State> {
    fn from(s: ManagerSendState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
struct ManagerSendOutput {
    address: String,
    amount: String,

    warning_address: bool,
    warning_amount: bool,

    view: ManagerSendOutputView,
}

impl ManagerSendOutput {
    fn new() -> Self {
        Self {
            address: "".to_string(),
            amount: "".to_string(),
            warning_address: false,
            warning_amount: false,
            view: ManagerSendOutputView::new(),
        }
    }

    fn amount(&self) -> Result<u64, Error> {
        if self.amount.is_empty() {
            return Ok(0);
        }

        let amount = bitcoin::Amount::from_str_in(&self.amount, bitcoin::Denomination::Bitcoin)
            .map_err(|_| Error::UnexpectedError("cannot parse output amount".to_string()))?;
        Ok(amount.as_sat())
    }

    fn valid(&self) -> bool {
        !self.address.is_empty()
            && !self.warning_address
            && !self.amount.is_empty()
            && !self.warning_amount
    }

    fn update(&mut self, message: RecipientMessage) {
        match message {
            RecipientMessage::AddressEdited(address) => {
                self.address = address;
                if !self.address.is_empty() {
                    self.warning_address = bitcoin::Address::from_str(&self.address).is_err();
                }
            }
            RecipientMessage::AmountEdited(amount) => {
                self.amount = amount;
                if !self.amount.is_empty() {
                    self.warning_amount = self.amount().is_err();
                }
            }
            _ => {}
        };
    }

    fn view(&mut self) -> Element<RecipientMessage> {
        self.view.view(
            &self.address,
            &self.amount,
            &self.warning_address,
            &self.warning_amount,
        )
    }
}

#[derive(Debug, Clone)]
struct ManagerSendInput {
    vault: model::Vault,
    selected: bool,
}

impl ManagerSendInput {
    fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            selected: false,
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<InputMessage> {
        manager_send_input_view(
            ctx,
            &self.vault.outpoint(),
            &self.vault.amount,
            self.selected,
        )
    }

    pub fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::Selected(selected) => self.selected = selected,
        }
    }
}

#[derive(Debug)]
pub struct ManagerNetworkState {
    revaultd: Arc<RevaultD>,

    blockheight: Option<u64>,
    warning: Option<Error>,

    view: ManagerNetworkView,
}

impl ManagerNetworkState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerNetworkState {
            revaultd,
            blockheight: None,
            warning: None,
            view: ManagerNetworkView::new(),
        }
    }
}

impl State for ManagerNetworkState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BlockHeight(b) => {
                match b {
                    Ok(height) => {
                        self.blockheight = height.into();
                    }
                    Err(e) => {
                        self.warning = Error::from(e).into();
                    }
                };
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view
            .view(ctx, self.warning.as_ref().into(), self.blockheight.as_ref())
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_blockheight(self.revaultd.clone()),
            Message::BlockHeight,
        )])
    }
}

impl From<ManagerNetworkState> for Box<dyn State> {
    fn from(s: ManagerNetworkState) -> Box<dyn State> {
        Box::new(s)
    }
}
