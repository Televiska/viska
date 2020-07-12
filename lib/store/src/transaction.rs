use once_cell::sync::Lazy;
use std::sync::Mutex;

static TRANSACTIONS: Lazy<Mutex<Transactions>> = Lazy::new(|| Mutex::new(Transactions(vec![])));

#[derive(Debug, Clone)]
pub struct Transaction {
    pub state: TransactionState,
    pub branch_id: String,
    pub dialog_id: String,
}

pub struct NewTransaction {
    pub state: TransactionState,
    pub branch_id: String,
}

impl NewTransaction {
    pub fn with(branch_id: String) -> Self {
        Self {
            state: TransactionState::Trying,
            branch_id,
        }
    }

    pub fn into_transaction(self, dialog_id: String) -> Transaction {
        Transaction {
            state: self.state,
            branch_id: self.branch_id,
            dialog_id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransactionState {
    Trying,
    Proceeding,
    Completed,
    Terminated,
}

impl Transaction {
    fn id(&self) -> String {
        self.branch_id.clone()
    }
}

pub struct Transactions(Vec<Transaction>);

impl Transactions {
    pub fn find(id: String) -> Option<Transaction> {
        Transactions::instance()
            .0
            .iter()
            .find(|dialog| dialog.id() == id)
            .cloned()
    }

    pub fn find_by_dialog_id(dialog_id: String) -> Option<Transaction> {
        Transactions::instance()
            .0
            .iter()
            .find(|dialog| dialog.dialog_id == dialog_id)
            .cloned()
    }

    pub fn create(transaction: Transaction) -> Transaction {
        Transactions::instance().0.push(transaction.clone());

        transaction
    }

    fn instance() -> std::sync::MutexGuard<'static, Transactions> {
        TRANSACTIONS
            .lock()
            .expect("Transactions is not initialized")
    }
}
