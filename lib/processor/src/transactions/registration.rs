use crate::presets;
use rsip::{Request, Response};

impl super::TransactionFSM for models::transactions::Registration {
    fn next(&self, request: Request) -> Result<Response, crate::Error> {
        match self {
            Self::Trying(data) => {
                common::log::debug!("inside trying");
                let next_state =
                    Self::Completed(models::transactions::registration::TransactionData {
                        id: data.id,
                        dialog_id: data.dialog_id,
                        branch_id: data.branch_id.clone(),
                    });
                store::Transaction::update(next_state, data.id)?;
                update_registration_for(request.clone())?;
                Ok(presets::create_registration_ok_from(request)?)
            }
            _ => Err("wrong transaction state".into()),
        }
    }
}

fn update_registration_for(request: Request) -> Result<models::Registration, crate::Error> {
    use std::convert::TryInto;

    Ok(
        store::Registration::upsert(TryInto::<models::UpdateRegistration>::try_into(request)?)?
            .into(),
    )
}
