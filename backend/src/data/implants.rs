use crate::{
    app::Application,
    core::esi::{ESIError, ESIScope},
};
use eve_data_core::TypeID;

pub async fn get_implants(app: &Application, character_id: i64) -> Result<Vec<TypeID>, ESIError> {
    // HARDCODED IMPLANTS FOR TESTING - uncomment to use hardcoded implants
    /*
    return Ok(vec![
        type_id!("High-grade Amulet Alpha"),
        type_id!("High-grade Amulet Beta"),
        type_id!("High-grade Amulet Delta"),
        type_id!("High-grade Amulet Epsilon"),
        type_id!("High-grade Amulet Gamma"),
        // type_id!("% WS-618"), // Uncomment to test full HYBRID set
    ]);
    */
    
    let path = format!("/v2/characters/{}/implants/", character_id);
    Ok(app
        .esi_client
        .get(&path, character_id, ESIScope::Clones_ReadImplants_v1)
        .await?)
}
