use crate::{actors::session::actor::Session, core::state::SharedState, utils::decrypt_token};
use actix_web::{http::header, web, Error, HttpRequest, Responder};
use actix_web_actors::ws;

pub async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<SharedState>,
) -> Result<impl Responder, Error> {
    let auth_header = req.headers().get(header::SEC_WEBSOCKET_PROTOCOL);
    match auth_header {
        Some(value) => match value.to_str() {
            Ok(token) => {
                let credential = decrypt_token(token);
                match credential {
                    Ok(credential) => {
                        let mut resp = ws::start(
                            Session::new(
                                credential.user.code,
                                credential.user.name,
                                app_state.chat_addr.clone(),
                            ),
                            &req,
                            stream,
                        )
                        .unwrap();
                        resp.headers_mut()
                            .insert(header::SEC_WEBSOCKET_PROTOCOL, value.clone());
                        Ok(resp)
                    }
                    Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid Token")),
                }
            }
            Err(_) => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
        },
        None => Err(actix_web::error::ErrorUnauthorized("Unauthorized")),
    }
}
