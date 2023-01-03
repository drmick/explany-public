use paperclip::actix::web::{delete, get, post, put};
use paperclip::actix::web::{resource, Scope};

use crate::handler;
use crate::web::scope;

pub fn spec() -> Scope {
    scope("/spec")
        .service(resource("/search").route(get().to(handler::spec::search::search)))
        .service(
            scope("/{user_id}")
                .service(
                    resource("/main_info")
                        .route(get().to(handler::spec::profile::show_main_info))
                        .route(put().to(handler::spec::profile::update_main_info)),
                )
                .service(
                    resource("/avatar")
                        .route(post().to(handler::spec::profile::set_avatar))
                        .route(delete().to(handler::spec::profile::remove_avatar)),
                )
                .service(
                    scope("/calendar")
                        .route("", put().to(handler::spec::calendar::update))
                        .route("", get().to(handler::spec::calendar::index))
                        .route("/available_time", get().to(handler::spec::calendar::available_time))
                        .route(
                            "/available_time_by_days",
                            get().to(handler::spec::calendar::available_time_by_days),
                        ),
                )
                .service(
                    scope("/services").service(
                        scope("/{service_id}")
                            .service(
                                scope("/education")
                                    .route("", get().to(handler::spec::education::index))
                                    .route("/{id}", get().to(handler::spec::education::get))
                                    .route("", post().to(handler::spec::education::create))
                                    .route("/{id}", put().to(handler::spec::education::update))
                                    .route("/{id}", delete().to(handler::spec::education::destroy))
                                    .route("/{id}", get().to(handler::spec::education::get)),
                            )
                            .service(
                                scope("/specializations")
                                    .route("", get().to(handler::spec::service::specialization::index))
                                    .route("", post().to(handler::spec::service::specialization::update)),
                            ),
                    ),
                ),
        )
}
