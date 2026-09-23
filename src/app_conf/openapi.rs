use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_axum::router::OpenApiRouter;
use crate::AppState;
use super::{api_routes, open_routes};

pub const COOKIE_AUTH: &str = "cookie_auth";

#[derive(OpenApi)]
#[openapi(
    info(title = "Rigidity API"),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "Authentication and account recovery"),
        (name = "user", description = "User accounts"),
        (name = "custom-room", description = "Custom room matchmaking"),
    )
)]
pub struct ApiDoc;

// Declares the private identity cookie set by `handlers::identity::login`
struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            COOKIE_AUTH,
            SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("id"))));
    }
}

/// Documented routes. Serve the returned spec with `split_for_parts`.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(open_routes::get_all())
        .merge(api_routes::get_all())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_spec_contains_all_routes() {
        let (_, api) = router().split_for_parts();
        let spec = serde_json::to_value(&api).unwrap();

        assert!(spec["paths"]["/api-open/login-steam"]["post"].is_object());
        assert!(spec["paths"]["/api/matchmaking/custom-room"]["delete"].is_object());
        assert!(spec["paths"]["/api/matchmaking/custom-room/{id}/join"]["put"].is_object());
        assert!(spec["components"]["securitySchemes"][COOKIE_AUTH].is_object());
        assert_eq!(api.paths.paths.len(), 12);
    }
}
