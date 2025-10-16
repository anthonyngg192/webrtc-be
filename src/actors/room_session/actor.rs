use crate::models::room_session::RoomSession;
use actix::{Actor, AsyncContext, Context};
use tokio::time::Duration;

use super::messages::PrometheusPulling;

impl Actor for RoomSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(3), |_, context| {
            context.address().do_send(PrometheusPulling {});
        });
    }
}
