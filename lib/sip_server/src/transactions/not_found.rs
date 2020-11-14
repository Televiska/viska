use crate::presets;
use rsip::{Request, Response};

impl super::TransactionFSM for models::transactions::NotFound {
    fn next(&self, request: Request) -> Result<Response, crate::Error> {
        Ok(presets::create_404_from(request)?)
    }
}
