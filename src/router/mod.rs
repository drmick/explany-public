use paperclip::actix::web::Scope;
use paperclip::actix::web::{get, post, put, ServiceConfig};

use crate::handler;
use crate::router::spec::spec;
use crate::web::scope;

mod spec;

pub(crate) fn routes(app: &mut ServiceConfig) {
    app.service(common()).service(spec());
}

fn common() -> Scope {
    scope("/meets")
        .route("", get().to(handler::meet::index))
        .route("", post().to(handler::meet::create))
        .route("/{id}", get().to(handler::meet::get))
        .route("/{id}", put().to(handler::meet::status))
}
