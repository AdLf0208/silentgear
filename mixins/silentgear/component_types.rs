// Runs before content loads: registers the gear's custom component id.
fn register_component_types(types: &mut Vec<String>) {
    crate::game::silentgear_state::register_component_types(types);
}
