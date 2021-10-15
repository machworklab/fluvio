//!
//! # Create Smart Module Request
//!
//! Converts Smart Module API request into KV request and sends to KV store for processing.
//!

use std::io::{Error, ErrorKind};

use tracing::{debug, trace, instrument};

use dataplane::ErrorCode;
use fluvio_sc_schema::Status;
use fluvio_controlplane_metadata::smartmodule::SmartModuleSpec;
use fluvio_controlplane_metadata::extended::SpecExt;
use fluvio_auth::{AuthContext, TypeAction};

use crate::core::Context;
use crate::services::auth::AuthServiceContext;

/// Handler for smart module request
#[instrument(skip(name, spec, _dry_run, auth_ctx))]
pub async fn handle_create_smart_module_request<AC: AuthContext>(
    name: String,
    spec: SmartModuleSpec,
    _dry_run: bool,
    auth_ctx: &AuthServiceContext<AC>,
) -> Result<Status, Error> {
    debug!("creating smart module: {}", name);

    if let Ok(authorized) = auth_ctx
        .auth
        .allow_type_action(SmartModuleSpec::OBJECT_TYPE, TypeAction::Create)
        .await
    {
        if !authorized {
            trace!("authorization failed");
            return Ok(Status::new(
                name.clone(),
                ErrorCode::PermissionDenied,
                Some(String::from("permission denied")),
            ));
        }
    } else {
        return Err(Error::new(ErrorKind::Interrupted, "authorization io error"));
    }

    let status = process_smart_module_request(&auth_ctx.global_ctx, name, spec).await;
    trace!("create smart module response {:#?}", status);

    Ok(status)
}

/// Process custom smart module, converts smart module spec to K8 and sends to KV store
#[instrument(skip(ctx, name, smart_module_spec))]
async fn process_smart_module_request(
    ctx: &Context,
    name: String,
    smart_module_spec: SmartModuleSpec,
) -> Status {
    if let Err(err) = ctx
        .smart_modules()
        .create_spec(name.clone(), smart_module_spec)
        .await
    {
        let error = Some(err.to_string());
        Status::new(name, ErrorCode::SmartModuleError, error) // TODO: create error type
    } else {
        Status::new_ok(name.clone())
    }
}
