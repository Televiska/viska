use crate::presets;

impl super::TransactionFSM for models::transactions::NotFound {
    fn next(&self, request: models::Request) -> Result<models::Response, crate::Error> {
        match self {
            _ => Ok(presets::create_404_from(request)?),
        }
    }
}
