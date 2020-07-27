pub trait TransactionFSM {
    fn next(&self, request: models::Request) -> Result<models::Response, String>;
}

pub trait DialogExt {
    fn transaction(&self) -> Box<dyn TransactionFSM>;
}

mod not_found;
mod registration;

impl DialogExt for models::Dialog {
    fn transaction(&self) -> Box<dyn TransactionFSM> {
        use models::DialogFlow;

        match &self.flow {
            DialogFlow::Registration(transaction) => Box::new(transaction.clone()),
            DialogFlow::Invite(transaction) => Box::new(transaction.clone()),
            DialogFlow::Publish(transaction) => Box::new(transaction.clone()),
        }
    }
}
