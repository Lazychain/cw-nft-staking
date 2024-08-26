use crate::{error::ContractError, responses::MetadataResponse};

use super::ReadonlyContext;

pub fn query_metadata(ctx: ReadonlyContext) -> Result<MetadataResponse, ContractError> {
    let ReadonlyContext { .. } = ctx;
    Ok(MetadataResponse {})
}
