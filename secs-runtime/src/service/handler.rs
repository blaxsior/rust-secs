use crate::{error::HandlerError, service::ServiceContext};

pub trait SecsService {
    fn serve(&mut self, ctx: &mut ServiceContext) -> Result<(), HandlerError>;
}
