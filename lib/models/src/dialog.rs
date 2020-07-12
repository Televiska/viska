use crate::Transaction;
use crate::NotFound;

#[derive(Debug, Clone)]
pub struct Dialog {
    pub computed_id: String,
    pub call_id: String,
    pub from_tag: String,
    pub to_tag: String,
    pub flow: DialogFlow,
}

#[derive(Debug, Clone)]
pub enum DialogFlow {
    Registration(Transaction),
    Invite(Transaction),
    Publish(NotFound)
}

pub enum TransactionType {
    Transaction(Transaction),
    NotFound(NotFound)
}

impl Dialog {
    pub fn transaction(&self) -> TransactionType {
        match &self.flow {
            DialogFlow::Registration(transaction) => TransactionType::Transaction(transaction.clone()),
            DialogFlow::Invite(transaction) => TransactionType::Transaction(transaction.clone()),
            DialogFlow::Publish(transaction) => TransactionType::NotFound(transaction.clone()),
        }
    }
}

impl From<store::DialogWithTransaction> for Dialog {
    fn from(record: store::DialogWithTransaction) -> Self {
        Self {
            computed_id: record.dialog.computed_id,
            call_id: record.dialog.call_id,
            from_tag: record.dialog.from_tag,
            to_tag: record.dialog.to_tag,
            flow: DialogFlow::from((record.dialog.flow, record.transaction)),
        }
    }
}

impl From<(store::DialogFlow, store::Transaction)> for DialogFlow {
    fn from(record: (store::DialogFlow, store::Transaction)) -> Self {
        match record.0 {
            store::DialogFlow::Registration => Self::Registration(record.1.into()),
            store::DialogFlow::Invite => Self::Invite(record.1.into()),
            store::DialogFlow::Publish => Self::Publish(record.1.into()),
        }
    }
}

impl Into<store::DialogWithTransaction> for Dialog {
    fn into(self) -> store::DialogWithTransaction {
        let (flow, transaction): (store::DialogFlow, store::Transaction) = self.flow.into();
        store::DialogWithTransaction {
            dialog: store::Dialog {
                computed_id: self.computed_id,
                call_id: self.call_id,
                from_tag: self.from_tag,
                to_tag: self.to_tag,
                flow: flow,
            },
            transaction,
        }
    }
}

impl Into<(store::DialogFlow, store::Transaction)> for DialogFlow {
    fn into(self) -> (store::DialogFlow, store::Transaction) {
        match self {
            Self::Registration(transaction) => {
                (store::DialogFlow::Registration, transaction.into())
            }
            Self::Invite(transaction) => (store::DialogFlow::Invite, transaction.into()),
            Self::Publish(transaction) => (store::DialogFlow::Publish, transaction.into()),
        }
    }
}
