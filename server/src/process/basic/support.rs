use crate::server::BasicServiceProvider;
use pb::service::basic::support::v1::{Contact, ContactRole, SupportRequest, SupportResponse};
use tonic::{Request, Response, Status};

pub async fn support(
    server: &BasicServiceProvider,
    _request: Request<SupportRequest>,
) -> Result<Response<SupportResponse>, Status> {
    // Convert contacts from user settings to proto format
    let contacts = server
        .shared_data
        .cfg()
        .user_setting
        .contacts
        .iter()
        .map(|contact| {
            let role = match contact.role {
                base::setting::ContactRole::Admin => ContactRole::Admin,
                base::setting::ContactRole::Security => ContactRole::Security,
            };

            Contact {
                email_address: contact.email_address.as_ref().map(|e| e.to_string()),
                ocid: contact.ocid.clone(),
                role: role.into(),
                phone_number: contact.phone_number.clone(),
            }
        })
        .collect();

    let ret = SupportResponse {
        contacts,
        support_page: server
            .shared_data
            .cfg()
            .user_setting
            .support_page
            .clone()
            .map(|x| x.to_string()),
    };

    Ok(Response::new(ret))
}
