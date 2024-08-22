mod api;
mod repository;

use actix_web::{middleware::Logger, web::scope, web::Data, App, HttpServer};
use actix_web_lab::web::spa;
use api::user::{create_user, delete_user, get_user, get_users, upgrade_user};
use repository::kub::KubeRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let kube_repo: KubeRepository = KubeRepository::init().await?;
    HttpServer::new(move || {
        let kube_data = Data::new(kube_repo.clone());
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .app_data(kube_data)
            .service(
                scope("/api")
                    .service(get_users)
                    .service(get_user)
                    .service(create_user)
                    .service(upgrade_user)
                    .service(delete_user),
            )
            .service(
                spa()
                    .index_file("./dist/index.html")
                    .static_resources_mount("/")
                    .static_resources_location("./dist")
                    .finish(),
            )
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
