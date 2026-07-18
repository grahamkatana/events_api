use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::handlers::register,
        crate::auth::handlers::login,
        crate::auth::handlers::resend_verification,
        crate::events::handlers::list_events,
        crate::events::handlers::get_event,
        crate::events::handlers::create_event,
        crate::events::handlers::update_event,
        crate::events::handlers::delete_event,
    ),
    components(schemas(
        crate::auth::models::User,
        crate::auth::models::RegisterUser,
        crate::auth::models::LoginUser,
        crate::auth::models::TokenResponse,
        crate::auth::models::ResendVerification,
        crate::auth::models::MessageResponse,
        crate::events::models::Event,
        crate::events::models::EventType,
        crate::events::models::CreateEvent,
        crate::events::models::UpdateEvent,
        crate::events::models::PaginatedEvents,
    )),
    tags(
        (name = "auth", description = "Registration, login, and email verification"),
        (name = "events", description = "Event CRUD (cover image upload not yet documented here)")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

// Registers the "bearer_auth" security scheme referenced by handlers'
// `security(("bearer_auth" = []))` attribute — this is what makes
// Swagger UI show an "Authorize" button where you can paste a JWT,
// which it then attaches to every request you try from the UI.
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}