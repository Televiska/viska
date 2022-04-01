use crate::Error;
use common::rsip::{self, headers::ToTypedHeader, message::HeadersExt};

pub fn run(request: &rsip::Request) -> Result<(), Error> {
    if !contact_header_is_unique(request)? {
        return Err(Error::from("more than one or zero Contact headers found"));
    }

    if !contact_header_is_secure(request)? {
        return Err(Error::from("Contact header not secure"));
    }

    Ok(())
}

pub fn contact_header_is_secure(request: &rsip::Request) -> Result<bool, Error> {
    if !contact_must_be_secure(request)? {
        return Ok(true);
    }

    Ok(request.contact_header()?.typed()?.uri.is_sips()?)
}

pub fn contact_header_is_unique(request: &rsip::Request) -> Result<bool, Error> {
    Ok(request.contact_headers().len() == 1)
}

pub fn contact_must_be_secure(request: &rsip::Request) -> Result<bool, Error> {
    let first_record_route_is_sips = request
        .record_route_header()
        .map(|h| {
            h.typed().map(|h| {
                h.uris()
                    .first()
                    .map(|uri_with_params| uri_with_params.is_sips())
            })
        })
        .transpose()?
        .flatten()
        .transpose()?
        .unwrap_or(false);

    Ok(request.uri.is_sips()? || first_record_route_is_sips)
}
