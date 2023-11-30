use actix_web::{web, App, HttpServer, get, HttpResponse, Responder};
mod idtolu;

#[get("/")]
async fn welcome() -> impl Responder {
    HttpResponse::Ok().body("Welcome to Images server")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    HttpServer::new(|| {
        /*let keycloak_auth = KeycloakAuth {
            detailed_responses: true,
            passthrough_policy: AlwaysReturnPolicy,
            keycloak_oid_public_key: DecodingKey::from_rsa_pem(KEYCLOAK_PK.as_bytes()).unwrap(),
            required_roles: vec![
                Role::Realm { role: "super-admin".to_owned() },
                Role::Client { client: "IDT-IDT-DTC-CLD-011".to_owned(), role: "cloud-advanced".to_owned() },
            ],
        };*/
        //let keycloak_auth = KeycloakAuth::default_with_pk(DecodingKey::from_rsa_pem(KEYCLOAK_PK.as_bytes()).unwrap());
        App::new()
            //.wrap(middleware::Logger::default())
            .service(
                web::scope("/api")
                    .configure(idtolu::images::config),
            )
            .service(welcome)
            .route("/", web::get())
    })
    .bind(("0.0.0.0", 1995))?
    .run()
    .await
}
