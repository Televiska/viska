use crate::{
    transactions::{NotFound, Registration},
    DialogExt, TransactionFSM,
};

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
    Registration(Registration),
    Invite(NotFound),
    Publish(NotFound),
}

impl DialogExt for Dialog {
    fn transaction(&self) -> Box<dyn TransactionFSM> {
        match &self.flow {
            DialogFlow::Registration(transaction) => Box::new(transaction.clone()),
            DialogFlow::Invite(transaction) => Box::new(transaction.clone()),
            DialogFlow::Publish(transaction) => Box::new(transaction.clone()),
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

impl Into<store::DirtyDialogWithTransaction> for Dialog {
    fn into(self) -> store::DirtyDialogWithTransaction {
        let (flow, transaction): (store::DialogFlow, store::DirtyTransaction) = self.flow.into();
        store::DirtyDialogWithTransaction {
            dialog: store::DirtyDialog {
                computed_id: Some(self.computed_id),
                call_id: Some(self.call_id),
                from_tag: Some(self.from_tag),
                to_tag: Some(self.to_tag),
                flow: Some(flow),
                ..Default::default()
            },
            transaction,
        }
    }
}

impl Into<(store::DialogFlow, store::DirtyTransaction)> for DialogFlow {
    fn into(self) -> (store::DialogFlow, store::DirtyTransaction) {
        match self {
            Self::Registration(transaction) => {
                (store::DialogFlow::Registration, transaction.into())
            }
            Self::Invite(transaction) => (store::DialogFlow::Invite, transaction.into()),
            Self::Publish(transaction) => (store::DialogFlow::Publish, transaction.into()),
        }
    }
}
