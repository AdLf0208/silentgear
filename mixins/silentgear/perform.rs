// Runs `/silentgear assemble|stats|help` when a player types one.
fn perform_command(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    crate::game::silentgear_state::on_perform_command(game, player, command, ran);
}
