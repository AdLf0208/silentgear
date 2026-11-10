// Adds the assembled gear's stat line to its tooltip.
fn add_gear_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    crate::game::silentgear_state::on_tooltip(game, stack, lines);
}
