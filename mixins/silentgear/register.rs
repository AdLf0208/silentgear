// Declares the `/silentgear` commands (permission level 2).
fn register_commands(commands: &mut Vec<NativeCommand>) {
    crate::game::silentgear_state::on_register_commands(commands);
}
