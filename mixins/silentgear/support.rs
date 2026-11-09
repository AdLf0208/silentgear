// Game-side glue for Silent Gear slice 1: the assembly command, the
// `silentgear:gear` component and the stats readout.
//
// Injected into `crate::game` as `silentgear_state`; it needs the game, so it
// is not part of the standalone library (only `runtime.rs` is).
//
// The assembly consumes the blueprint and the two part items from the acting
// player's inventory (main slots first, then the off hand), folds the parts'
// material stats with the pure runtime, refuses a stat block the carried
// item cannot honour (see `FINDINGS.md`), and gives back one
// `silentgear:pickaxe` carrying the `silentgear:gear` component.

use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{JavaString, Tag};

use crate::game::native_server::{NativeCommand, NativeServerPlayer};
use crate::game::silentgear::{
    AXE_HEAD, Action, BLUEPRINT, COMPONENT, GearRecord, HEAD, PICKAXE, PartType, ROD, assemble,
    help_lines, material, parse, stats_line_with_damage, supported,
};

/// The tooltip colour of the stat lines, in the HUD's `0xAABBGGRR` packing
/// (`hud::rgba(170, 170, 170, 255)`, vanilla's secondary tooltip grey).
const STAT_COLOR: u32 = 0xFFAA_AA_AA;

/// Registers the component the assembled gear carries, before content loads.
pub(crate) fn register_component_types(types: &mut Vec<String>) {
    types.push(COMPONENT.to_owned());
}

/// Declares `/silentgear ...` at operator level 2.
#[allow(private_interfaces)]
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    for usage in [
        "silentgear assemble <gear:word> <head:word> <rod:word>".to_owned(),
        "silentgear stats".to_owned(),
        "silentgear help".to_owned(),
    ] {
        commands.push(NativeCommand::op(&usage, 2));
    }
}

/// Runs `/silentgear ...` with the acting player's state swapped in.
#[allow(private_interfaces)]
pub fn on_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    match parse(command) {
        Action::Unknown => {}
        Action::Help => {
            *ran = true;
            for line in help_lines() {
                game.native_feedback(Text::literal(line));
            }
        }
        Action::Stats => {
            *ran = true;
            game.native_feedback(Text::literal(stats_text(game)));
        }
        Action::Assemble { gear, head, rod } => {
            *ran = true;
            let line = match assemble_command(game, player, &gear, &head, &rod) {
                Ok(line) => line,
                Err(line) => line,
            };
            game.native_feedback(Text::literal(line));
        }
    }
}

/// Adds the gear's stat lines to its tooltip.
#[allow(private_interfaces)]
pub fn on_tooltip(game: &crate::game::Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    let Some(record) = gear_record(game, stack) else {
        return;
    };
    lines.push((
        Text::literal(stats_line_with_damage(&record, stack.damage)),
        STAT_COLOR,
    ));
}

/// The held stack's stat block, as `/silentgear stats` answers it.
fn stats_text(game: &crate::game::Game<'_>) -> String {
    let Some(held) = game.held() else {
        return "Hold a Silent Gear tool first.".to_owned();
    };
    match gear_record(game, held) {
        Some(record) => stats_line_with_damage(&record, held.damage),
        None => format!("{} is not assembled Silent Gear.", held.item),
    }
}

/// Reads the `silentgear:gear` component of a stack.
fn gear_record(game: &crate::game::Game<'_>, stack: &ItemStack) -> Option<GearRecord> {
    let tag = game.native_component(stack, COMPONENT).ok().flatten()?;
    let fields = tag_to_fields(tag)?;
    GearRecord::decode(&fields).ok()
}

/// Builds the pickaxe from the blueprints and parts in the player's inventory.
fn assemble_command(
    game: &mut crate::game::Game<'_>,
    player: &NativeServerPlayer,
    gear: &str,
    head: &str,
    rod: &str,
) -> Result<String, String> {
    if gear != "pickaxe" {
        return Err(format!("this slice builds only the pickaxe, not {gear}."));
    }
    let Some(head_material) = material(head) else {
        return Err(format!("unknown material {head}."));
    };
    let Some(rod_material) = material(rod) else {
        return Err(format!("unknown material {rod}."));
    };

    // The parts and the blueprint must be carried; the wrong part is named so
    // the e2e's control reads as a refusal, not a silent no-op.
    if !carries(game, BLUEPRINT) {
        return Err(format!("carry a {BLUEPRINT} first."));
    }
    if !carries(game, HEAD) {
        if carries(game, AXE_HEAD) {
            return Err(format!("a {AXE_HEAD} is not a {HEAD}."));
        }
        return Err(format!("carry a {HEAD} first."));
    }
    if !carries(game, ROD) {
        return Err(format!("carry a {ROD} first."));
    }

    let parts = [
        (PartType::Main, head_material),
        (PartType::Rod, rod_material),
    ];
    let stats = assemble(&parts).map_err(|error| error.to_string())?;
    let record = GearRecord {
        gear_type: PICKAXE.to_owned(),
        parts: vec![
            (PartType::Main, material_id(head_material.id)),
            (PartType::Rod, material_id(rod_material.id)),
        ],
        stats,
    };
    supported(&record).map_err(|error| error.to_string())?;

    let blueprint =
        take_one(game, BLUEPRINT).ok_or_else(|| format!("carry a {BLUEPRINT} first."))?;
    let head_stack = take_one(game, HEAD).ok_or_else(|| format!("carry a {HEAD} first."))?;
    let rod_stack = take_one(game, ROD).ok_or_else(|| format!("carry a {ROD} first."))?;

    let mut stack = ItemStack::new(PICKAXE, 1);
    let tag = fields_to_tag(&record.encode());
    if let Err(error) = game.native_set_component(&mut stack, COMPONENT, tag) {
        refund(game, player, [blueprint, head_stack, rod_stack]);
        return Err(format!("the gear component could not be written: {error}"));
    }
    if let Some(left) = game.native_give_stack(player.uuid, stack) {
        refund(game, player, [blueprint, head_stack, rod_stack]);
        return Err(format!("no room for the pickaxe ({} left).", left.count));
    }
    Ok(format!(
        "Assembled {PICKAXE} from {head} and {rod}: {}",
        stats_line_with_damage(&record, 0)
    ))
}

/// `silentgear:<material>` for a material's short id.
fn material_id(short: &str) -> String {
    format!("silentgear:{short}")
}

/// Whether any inventory slot or the off hand holds `item`.
fn carries(game: &crate::game::Game<'_>, item: &str) -> bool {
    game.inventory
        .iter()
        .flatten()
        .any(|stack| &*stack.item == item)
        || game
            .offhand
            .as_ref()
            .is_some_and(|stack| &*stack.item == item)
}

/// Takes one `item` from the main slots, then the off hand.
fn take_one(game: &mut crate::game::Game<'_>, item: &str) -> Option<ItemStack> {
    if let Some(slot) = game
        .inventory
        .iter()
        .position(|stack| stack.as_ref().is_some_and(|stack| &*stack.item == item))
    {
        return take_from(game.inventory.get_mut(slot)?.as_mut()?);
    }
    let stack = game.offhand.as_mut().filter(|stack| &*stack.item == item)?;
    let one = take_from(stack)?;
    if game.offhand.as_ref().is_some_and(|stack| stack.count == 0) {
        game.offhand = None;
    }
    Some(one)
}

/// One item off a stack, clearing it when that was the last.
fn take_from(stack: &mut ItemStack) -> Option<ItemStack> {
    if stack.count == 0 {
        return None;
    }
    let mut one = stack.clone();
    one.count = 1;
    stack.count -= 1;
    Some(one)
}

/// Gives consumed items back where the assembly failed after taking them.
fn refund(game: &mut crate::game::Game<'_>, player: &NativeServerPlayer, stacks: [ItemStack; 3]) {
    for stack in stacks {
        let _ = game.native_give_stack(player.uuid, stack);
    }
}

/// Converts the runtime's flat component pairs into NBT.
fn fields_to_tag(fields: &[(String, crate::game::silentgear::Value)]) -> Tag {
    Tag::compound(
        fields
            .iter()
            .map(|(key, value)| (key.as_str(), value_to_tag(value))),
    )
}

fn value_to_tag(value: &crate::game::silentgear::Value) -> Tag {
    use crate::game::silentgear::Value;
    match value {
        Value::Int(number) => Tag::Int(*number as i32),
        Value::Double(number) => Tag::Double(number.to_bits()),
        Value::Str(text) => Tag::String(JavaString::from(text.as_str())),
        Value::List(items) => Tag::List(items.iter().map(value_to_tag).collect()),
        Value::Compound(pairs) => Tag::compound(
            pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value_to_tag(value))),
        ),
    }
}

/// Converts an NBT compound back into the runtime's flat pairs.
fn tag_to_fields(tag: &Tag) -> Option<Vec<(String, crate::game::silentgear::Value)>> {
    let Tag::Compound(pairs) = tag else {
        return None;
    };
    Some(
        pairs
            .iter()
            .map(|(key, value)| (key.to_string_lossy(), tag_to_value(value)))
            .collect(),
    )
}

fn tag_to_value(tag: &Tag) -> crate::game::silentgear::Value {
    use crate::game::silentgear::Value;
    match tag {
        Tag::Byte(number) => Value::Int(i64::from(*number)),
        Tag::Short(number) => Value::Int(i64::from(*number)),
        Tag::Int(number) => Value::Int(i64::from(*number)),
        Tag::Long(number) => Value::Int(*number),
        Tag::Float(number) => Value::Double(f64::from(f32::from_bits(*number))),
        Tag::Double(number) => Value::Double(f64::from_bits(*number)),
        Tag::String(text) => Value::Str(text.to_string_lossy()),
        Tag::List(items) => Value::List(items.iter().map(tag_to_value).collect()),
        Tag::Compound(pairs) => Value::Compound(
            pairs
                .iter()
                .map(|(key, value)| (key.to_string_lossy(), tag_to_value(value)))
                .collect(),
        ),
        _ => Value::Str(String::new()),
    }
}
