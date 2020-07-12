use crate::{NewTransaction, Transaction, Transactions};
use once_cell::sync::Lazy;
use std::sync::Mutex;

static DIALOGS: Lazy<Mutex<Dialogs>> = Lazy::new(|| Mutex::new(Dialogs(vec![])));

#[derive(Debug, Clone)]
pub struct Dialog {
    pub computed_id: String,
    pub call_id: String,
    pub from_tag: String,
    pub to_tag: String,
    pub flow: DialogFlow,
}

#[derive(Debug, Clone)]
pub struct NewDialog {
    pub call_id: String,
    pub from_tag: String,
    pub flow: DialogFlow,
}

pub struct NewDialogWithTransaction {
    pub dialog: NewDialog,
    pub transaction: NewTransaction,
}

pub struct DialogWithTransaction {
    pub dialog: Dialog,
    pub transaction: Transaction,
}

impl Dialog {
    pub fn id(&self) -> String {
        self.computed_id.clone()
    }
}

impl Into<Dialog> for NewDialog {
    fn into(self) -> Dialog {
        let to_tag: String = "hello".into();
        Dialog {
            computed_id: compute_id_for(&self.call_id, &self.from_tag, &to_tag),
            call_id: self.call_id,
            from_tag: self.from_tag,
            to_tag,
            flow: self.flow,
        }
    }
}

pub struct Dialogs(Vec<Dialog>);

impl Dialogs {
    pub fn find(id: String) -> Option<Dialog> {
        Dialogs::instance()
            .0
            .iter()
            .find(|dialog| dialog.id() == id)
            .cloned()
    }

    pub fn find_with_transaction(id: String) -> Option<DialogWithTransaction> {
        let dialog = Dialogs::find(id);
        match dialog {
            Some(dialog) => Some(DialogWithTransaction {
                dialog: dialog.clone(),
                transaction: Transactions::find(dialog.id()).expect("Transaction can't be found"),
            }),
            None => None,
        }
    }

    pub fn create(new_record: NewDialogWithTransaction) -> DialogWithTransaction {
        let dialog: Dialog = new_record.dialog.into();
        Dialogs::instance().0.push(dialog.clone());
        let transaction = Transactions::create(
            new_record
                .transaction
                .into_transaction(dialog.computed_id.clone()),
        );

        DialogWithTransaction {
            dialog: dialog,
            transaction,
        }
    }

    fn instance() -> std::sync::MutexGuard<'static, Dialogs> {
        DIALOGS.lock().expect("Dialogs is not initialized")
    }
}

#[derive(Debug, Clone)]
pub enum DialogFlow {
    Registration,
    Invite,
    Publish
}

fn compute_id_for(call_id: &String, from_tag: &String, to_tag: &String) -> String {
    format!("{}-{}-{}", call_id, from_tag, to_tag)
}
