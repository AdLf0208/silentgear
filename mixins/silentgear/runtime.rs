// Silent Gear slice 1: the pickaxe's material stat block.
//
// This file is both the standalone library (`cargo test`) and the module the
// planner injects into the game as `crate::game::silentgear`; it uses no game
// types, so the rules are testable without staging Horizon.
//
// Slice 1 is one gear: `silentgear:pickaxe`, assembled from a
// `silentgear:pickaxe_head` (main part) and a `silentgear:rod` (rod part).
// The material property fold follows Silent Gear's `NumberProperty.compute`
// (see `FINDINGS.md`), and the result is written into the stack's
// `silentgear:gear` component.

use std::fmt;

/// The component a gear stack carries (`silentgear:gear`).
pub const COMPONENT: &str = "silentgear:gear";
/// The assembled tool this slice builds.
pub const PICKAXE: &str = "silentgear:pickaxe";
/// The reusable template the assembly consumes.
pub const BLUEPRINT: &str = "silentgear:pickaxe_blueprint";
/// The `main` part a pickaxe needs.
pub const HEAD: &str = "silentgear:pickaxe_head";
/// The `rod` part a pickaxe needs.
pub const ROD: &str = "silentgear:rod";
/// An axe head: the wrong part for a pickaxe (the e2e's control).
pub const AXE_HEAD: &str = "silentgear:axe_head";

/// A part of a gear item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PartType {
    /// The head of a harvest tool (`silentgear:main`).
    Main,
    /// The handle (`silentgear:rod`).
    Rod,
}

impl PartType {
    /// Every part type this slice models.
    pub const ALL: [PartType; 2] = [PartType::Main, PartType::Rod];

    /// The part type's data id.
    pub fn id(self) -> &'static str {
        match self {
            PartType::Main => "silentgear:main",
            PartType::Rod => "silentgear:rod",
        }
    }

    /// Parses a part type's data id.
    pub fn from_id(id: &str) -> Option<PartType> {
        PartType::ALL.into_iter().find(|part| part.id() == id)
    }
}

/// How one number combines into the gear's value (`NumberProperty.Operation`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// The weighted average upstream uses for a part's own value.
    Average,
    /// The largest value wins.
    Max,
    /// Added after the multiplications.
    Add,
    /// `value * factor` added to the running total.
    MultiplyBase,
    /// The running total multiplied by `1 + factor`.
    MultiplyTotal,
}

/// One number a part contributes, with the operation that folds it in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Modifier {
    /// The number itself.
    pub value: f32,
    /// How it combines.
    pub operation: Operation,
}

impl Modifier {
    /// A value with an operation.
    pub const fn new(value: f32, operation: Operation) -> Modifier {
        Modifier { value, operation }
    }

    /// The operation a bare number in a material file means.
    pub const fn average(value: f32) -> Modifier {
        Modifier::new(value, Operation::Average)
    }
}

/// The properties one material gives as one part type.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PartStats {
    /// `GearProperties.DURABILITY` for this part.
    pub durability: Option<Modifier>,
    /// `GearProperties.HARVEST_SPEED` for this part.
    pub harvest_speed: Option<Modifier>,
    /// `GearProperties.ENCHANTMENT_VALUE` for this part.
    pub enchantment_value: Option<Modifier>,
    /// The harvest tier this part's material asks for.
    pub harvest_tier: Option<HarvestTier>,
}

impl PartStats {
    /// The numeric modifiers for `property`, in material-file order.
    pub fn modifiers(&self, property: Property) -> Vec<Modifier> {
        let one = match property {
            Property::Durability => self.durability,
            Property::HarvestSpeed => self.harvest_speed,
            Property::EnchantmentValue => self.enchantment_value,
        };
        one.into_iter().collect()
    }
}

/// A numeric gear property this slice reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Property {
    /// Durability (the item's max damage).
    Durability,
    /// Mining speed.
    HarvestSpeed,
    /// Enchanting table power.
    EnchantmentValue,
}

/// Silent Gear's harvest tiers, in the order the game adds them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HarvestTier {
    /// Hand level: no block is gated.
    Zero,
    /// Wood.
    Wood,
    /// Stone (flint uses this tier in the port).
    Stone,
    /// Iron.
    Iron,
    /// Diamond.
    Diamond,
    /// Gold.
    Gold,
    /// Netherite.
    Netherite,
}

impl HarvestTier {
    /// The material file's tier name.
    pub fn name(self) -> &'static str {
        match self {
            HarvestTier::Zero => "zero",
            HarvestTier::Wood => "wood",
            HarvestTier::Stone => "stone",
            HarvestTier::Iron => "iron",
            HarvestTier::Diamond => "diamond",
            HarvestTier::Gold => "gold",
            HarvestTier::Netherite => "netherite",
        }
    }

    /// Parses a material file's tier name.
    pub fn from_name(name: &str) -> Option<HarvestTier> {
        [
            HarvestTier::Zero,
            HarvestTier::Wood,
            HarvestTier::Stone,
            HarvestTier::Iron,
            HarvestTier::Diamond,
            HarvestTier::Gold,
            HarvestTier::Netherite,
        ]
        .into_iter()
        .find(|tier| tier.name().eq_ignore_ascii_case(name))
    }
}

/// A material: the properties it gives as a main part and as a rod.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// The material id without its namespace (`iron`).
    pub id: &'static str,
    /// What the material gives a `main` part.
    pub main: PartStats,
    /// What the material gives a `rod` part.
    pub rod: PartStats,
}

/// The material table, taken from the jar's
/// `data/silentgear/silentgear_materials/<id>.json` (see `FINDINGS.md`).
const MATERIALS: [Material; 7] = [
    Material {
        id: "iron",
        main: PartStats {
            durability: Some(Modifier::average(250.0)),
            harvest_speed: Some(Modifier::average(6.0)),
            enchantment_value: Some(Modifier::average(14.0)),
            harvest_tier: Some(HarvestTier::Iron),
        },
        rod: PartStats::DEFAULT,
    },
    Material {
        id: "wood",
        main: PartStats {
            durability: Some(Modifier::average(59.0)),
            harvest_speed: Some(Modifier::average(2.0)),
            enchantment_value: Some(Modifier::average(15.0)),
            harvest_tier: Some(HarvestTier::Wood),
        },
        rod: PartStats::DEFAULT,
    },
    Material {
        id: "stone",
        main: PartStats {
            durability: Some(Modifier::average(131.0)),
            harvest_speed: Some(Modifier::average(4.0)),
            enchantment_value: Some(Modifier::average(5.0)),
            harvest_tier: Some(HarvestTier::Stone),
        },
        rod: PartStats {
            harvest_speed: Some(Modifier::new(0.1, Operation::MultiplyTotal)),
            ..PartStats::DEFAULT
        },
    },
    Material {
        id: "diamond",
        main: PartStats {
            durability: Some(Modifier::average(1561.0)),
            harvest_speed: Some(Modifier::average(8.0)),
            enchantment_value: Some(Modifier::average(10.0)),
            harvest_tier: Some(HarvestTier::Diamond),
        },
        rod: PartStats {
            harvest_speed: Some(Modifier::new(0.2, Operation::MultiplyTotal)),
            ..PartStats::DEFAULT
        },
    },
    Material {
        id: "gold",
        main: PartStats {
            durability: Some(Modifier::average(32.0)),
            harvest_speed: Some(Modifier::average(12.0)),
            enchantment_value: Some(Modifier::average(22.0)),
            harvest_tier: Some(HarvestTier::Gold),
        },
        rod: PartStats {
            harvest_speed: Some(Modifier::new(3.0, Operation::Add)),
            ..PartStats::DEFAULT
        },
    },
    Material {
        id: "flint",
        main: PartStats {
            durability: Some(Modifier::average(124.0)),
            harvest_speed: Some(Modifier::average(5.0)),
            enchantment_value: Some(Modifier::average(3.0)),
            harvest_tier: Some(HarvestTier::Stone),
        },
        rod: PartStats::DEFAULT,
    },
    Material {
        id: "netherite",
        main: PartStats::DEFAULT,
        rod: PartStats::DEFAULT,
    },
];

impl PartStats {
    /// An empty set of part properties.
    pub const DEFAULT: PartStats = PartStats {
        durability: None,
        harvest_speed: None,
        enchantment_value: None,
        harvest_tier: None,
    };
}

/// Looks a material up by its short id or its namespaced id.
pub fn material(id: &str) -> Option<Material> {
    let short = id.strip_prefix("silentgear:").unwrap_or(id);
    MATERIALS.into_iter().find(|material| material.id == short)
}

/// The stat block the carried `silentgear:pickaxe` item is defined with
/// (vanilla's iron pickaxe components).
pub const ITEM_STATS: GearStats = GearStats {
    durability: 250,
    harvest_speed: 6.0,
    enchantment_value: 14,
    harvest_tier: HarvestTier::Iron,
};

/// The combined stats of an assembled gear.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GearStats {
    /// The item's max damage.
    pub durability: u32,
    /// The tool's mining speed.
    pub harvest_speed: f32,
    /// The enchanting table power.
    pub enchantment_value: u32,
    /// The harvest tier (what the tool can harvest for drops).
    pub harvest_tier: HarvestTier,
}

/// Why an assembly was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssembleError {
    /// A required part type is absent.
    MissingPart(PartType),
    /// A part type is present twice.
    DuplicatePart(PartType),
    /// The material gives nothing to that part type.
    NotGearMaterial(String),
    /// The folded durability is zero, so the item would break at once.
    NoDurability,
    /// The stat block is not the one the slice's item is defined with.
    Unsupported(Vec<String>),
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssembleError::MissingPart(part) => write!(f, "missing {} part", part.id()),
            AssembleError::DuplicatePart(part) => write!(f, "more than one {} part", part.id()),
            AssembleError::NotGearMaterial(material) => {
                write!(f, "{material} gives this part no properties")
            }
            AssembleError::NoDurability => f.write_str("the gear would have no durability"),
            AssembleError::Unsupported(stat) => {
                write!(
                    f,
                    "this slice builds only the iron pickaxe (got {})",
                    stat.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for AssembleError {}

/// The fold `NumberProperty.compute` performs.
pub fn fold(base: f32, modifiers: &[Modifier]) -> f32 {
    // The properties this slice folds have vanilla's `(0, i32::MAX)` range.
    let min = 0.0f32;
    let max = i32::MAX as f32;
    let mut value = base;

    // AVERAGE: the weighted average upstream uses, added to the base. The
    // first AVERAGE value sets the weights; the same `1 + v/(1+|first|)`
    // formula as `NumberProperty.getWeightedAverage`.
    let averages: Vec<Modifier> = modifiers
        .iter()
        .copied()
        .filter(|modifier| modifier.operation == Operation::Average)
        .collect();
    if let Some(first) = averages.first() {
        let primary = if first.value > 0.0 { first.value } else { 1.0 };
        let mut total_weight = 0.0f32;
        let mut weighted = 0.0f32;
        for modifier in &averages {
            let weight = 1.0 + modifier.value / (1.0 + primary.abs());
            total_weight += weight;
            weighted += modifier.value * weight;
        }
        if total_weight > 0.0 {
            value += weighted / total_weight;
        }
    }
    for modifier in modifiers {
        if modifier.operation == Operation::Max {
            value = value.max(modifier.value);
        }
    }
    let before_multiplies = value;
    let mut result = value;
    for modifier in modifiers {
        if modifier.operation == Operation::MultiplyBase {
            result += before_multiplies * modifier.value;
        }
    }
    for modifier in modifiers {
        if modifier.operation == Operation::MultiplyTotal {
            result *= 1.0 + modifier.value;
        }
    }
    for modifier in modifiers {
        if modifier.operation == Operation::Add {
            result += modifier.value;
        }
    }
    result.clamp(min, max)
}

/// The stat values' precision: the fold's f32 rounding is trimmed to four
/// decimals so an exact material reads back as its file value (`6`, not
/// `5.9999995`) and the item's components and the component agree.
fn round_stat(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

/// Assembles the pickaxe's stat block from one part per part type.
pub fn assemble(parts: &[(PartType, Material)]) -> Result<GearStats, AssembleError> {
    // The head is the gear's body: a material that gives a main part nothing
    // is not a gear material (a rod may add only traits, as wood does).
    if let Some((_, main)) = parts.iter().find(|(kind, _)| *kind == PartType::Main)
        && main.main == PartStats::DEFAULT
    {
        return Err(AssembleError::NotGearMaterial(main.id.to_owned()));
    }

    for part in PartType::ALL {
        let count = parts.iter().filter(|(kind, _)| *kind == part).count();
        if count > 1 {
            return Err(AssembleError::DuplicatePart(part));
        }
        if count == 0 {
            return Err(AssembleError::MissingPart(part));
        }
    }

    let main = parts
        .iter()
        .find(|(kind, _)| *kind == PartType::Main)
        .map(|(_, material)| material.main)
        .expect("the part count check ensured a main part");
    let rod = parts
        .iter()
        .find(|(kind, _)| *kind == PartType::Rod)
        .map(|(_, material)| material.rod)
        .expect("the part count check ensured a rod part");

    let modifiers = |property: Property| {
        let mut all = main.modifiers(property);
        all.extend(rod.modifiers(property));
        all
    };
    let durability = round_stat(fold(0.0, &modifiers(Property::Durability))).round();
    if durability <= 0.0 {
        return Err(AssembleError::NoDurability);
    }
    Ok(GearStats {
        durability: durability as u32,
        harvest_speed: round_stat(fold(0.0, &modifiers(Property::HarvestSpeed))),
        enchantment_value: round_stat(fold(0.0, &modifiers(Property::EnchantmentValue))).round()
            as u32,
        harvest_tier: main.harvest_tier.unwrap_or(HarvestTier::Zero),
    })
}

/// The gear record stored in the `silentgear:gear` component.
#[derive(Clone, Debug, PartialEq)]
pub struct GearRecord {
    /// The gear type id (`silentgear:pickaxe`).
    pub gear_type: String,
    /// One entry per part, in assembly order.
    pub parts: Vec<(PartType, String)>,
    /// The combined stats.
    pub stats: GearStats,
}

/// A value in the component tree.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// An integer (`Tag::Int`).
    Int(i64),
    /// A double (`Tag::Double`).
    Double(f64),
    /// A string (`Tag::String`).
    Str(String),
    /// A list of values.
    List(Vec<Value>),
    /// Key/value pairs.
    Compound(Vec<(String, Value)>),
}

impl Value {
    /// The value as an integer, if it is one.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// The value as a double, accepting an integer too.
    pub fn as_double(&self) -> Option<f64> {
        match self {
            Value::Double(value) => Some(*value),
            Value::Int(value) => Some(*value as f64),
            _ => None,
        }
    }

    /// The value as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(value) => Some(value),
            _ => None,
        }
    }
}

/// Why a gear record could not be read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// A required key is absent.
    Missing(&'static str),
    /// A key has the wrong kind of value.
    WrongKind(&'static str),
    /// A part type is not one this slice knows.
    UnknownPart(String),
    /// A material id is not in the table.
    UnknownMaterial(String),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Missing(key) => write!(f, "the gear record has no {key}"),
            DecodeError::WrongKind(key) => write!(f, "the gear record's {key} has the wrong kind"),
            DecodeError::UnknownPart(part) => write!(f, "unknown part type {part}"),
            DecodeError::UnknownMaterial(material) => write!(f, "unknown material {material}"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl GearRecord {
    /// The record's flat component pairs.
    pub fn encode(&self) -> Vec<(String, Value)> {
        let parts = self
            .parts
            .iter()
            .map(|(kind, material)| {
                Value::Compound(vec![
                    ("part".to_owned(), Value::Str(kind.id().to_owned())),
                    ("material".to_owned(), Value::Str(material.clone())),
                ])
            })
            .collect();
        vec![
            ("gear_type".to_owned(), Value::Str(self.gear_type.clone())),
            ("parts".to_owned(), Value::List(parts)),
            (
                "durability".to_owned(),
                Value::Int(i64::from(self.stats.durability)),
            ),
            (
                "harvest_speed".to_owned(),
                Value::Double(f64::from(self.stats.harvest_speed)),
            ),
            (
                "enchantment_value".to_owned(),
                Value::Int(i64::from(self.stats.enchantment_value)),
            ),
            (
                "harvest_tier".to_owned(),
                Value::Str(self.stats.harvest_tier.name().to_owned()),
            ),
        ]
    }

    /// Reads a record back from its flat component pairs.
    pub fn decode(fields: &[(String, Value)]) -> Result<GearRecord, DecodeError> {
        let field = |key: &str| {
            fields
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value)
        };
        let gear_type = field("gear_type")
            .ok_or(DecodeError::Missing("gear_type"))?
            .as_str()
            .ok_or(DecodeError::WrongKind("gear_type"))?
            .to_owned();
        let Value::List(entries) = field("parts").ok_or(DecodeError::Missing("parts"))? else {
            return Err(DecodeError::WrongKind("parts"));
        };
        if entries.is_empty() {
            return Err(DecodeError::Missing("parts"));
        }
        let mut parts = Vec::with_capacity(entries.len());
        for entry in entries {
            let Value::Compound(pairs) = entry else {
                return Err(DecodeError::WrongKind("parts"));
            };
            let part = pairs
                .iter()
                .find(|(name, _)| name == "part")
                .and_then(|(_, value)| value.as_str())
                .ok_or(DecodeError::Missing("parts"))?;
            let kind =
                PartType::from_id(part).ok_or_else(|| DecodeError::UnknownPart(part.to_owned()))?;
            let material = pairs
                .iter()
                .find(|(name, _)| name == "material")
                .and_then(|(_, value)| value.as_str())
                .ok_or(DecodeError::Missing("parts"))?;
            let short = material.strip_prefix("silentgear:").unwrap_or(material);
            if self::material(short).is_none() {
                return Err(DecodeError::UnknownMaterial(material.to_owned()));
            }
            parts.push((kind, material.to_owned()));
        }
        let durability = field("durability")
            .ok_or(DecodeError::Missing("durability"))?
            .as_int()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(DecodeError::WrongKind("durability"))?;
        let harvest_speed = field("harvest_speed")
            .ok_or(DecodeError::Missing("harvest_speed"))?
            .as_double()
            .ok_or(DecodeError::WrongKind("harvest_speed"))? as f32;
        let enchantment_value = field("enchantment_value")
            .ok_or(DecodeError::Missing("enchantment_value"))?
            .as_int()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(DecodeError::WrongKind("enchantment_value"))?;
        let harvest_tier = field("harvest_tier")
            .ok_or(DecodeError::Missing("harvest_tier"))?
            .as_str()
            .and_then(HarvestTier::from_name)
            .ok_or(DecodeError::WrongKind("harvest_tier"))?;
        Ok(GearRecord {
            gear_type,
            parts,
            stats: GearStats {
                durability,
                harvest_speed,
                enchantment_value,
                harvest_tier,
            },
        })
    }
}

/// A gear's durability, and what wear does to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Durability {
    max: u32,
}

impl Durability {
    /// The durability of a gear with `max` points.
    pub fn new(max: u32) -> Durability {
        Durability { max }
    }

    /// The maximum.
    pub fn max(self) -> u32 {
        self.max
    }

    /// The points left at `damage`.
    pub fn remaining(self, damage: u32) -> u32 {
        self.max.saturating_sub(damage)
    }

    /// Whether `damage` has destroyed the item.
    pub fn is_broken(self, damage: u32) -> bool {
        damage >= self.max
    }
}

/// What `/silentgear …` asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Assemble a gear from a head material and a rod material.
    Assemble {
        /// The gear type's short name.
        gear: String,
        /// The head material.
        head: String,
        /// The rod material.
        rod: String,
    },
    /// Print the held gear's stat block.
    Stats,
    /// Print the help text.
    Help,
    /// Nothing this mod knows.
    Unknown,
}

/// Parses a command line, with or without its leading slash and root.
pub fn parse(command: &str) -> Action {
    let command = command.trim();
    let command = command.strip_prefix('/').unwrap_or(command);
    let mut words = command.split_whitespace();
    if !matches!(words.next(), Some("silentgear" | "sg")) {
        return Action::Unknown;
    }
    match words.next() {
        None | Some("help") => Action::Help,
        Some("stats") => Action::Stats,
        Some("assemble") => match (words.next(), words.next(), words.next()) {
            (Some(gear), Some(head), Some(rod)) => Action::Assemble {
                gear: gear.to_owned(),
                head: head.to_owned(),
                rod: rod.to_owned(),
            },
            _ => Action::Unknown,
        },
        Some(_) => Action::Unknown,
    }
}

/// The help text.
pub fn help_lines() -> Vec<&'static str> {
    vec![
        "/silentgear assemble pickaxe <head> <rod>   build the pickaxe",
        "/silentgear stats                           the held gear's stat block",
        "/silentgear help                            this text",
    ]
}

/// One line describing a gear's stat block, with the points it has lost.
pub fn stats_line_with_damage(record: &GearRecord, damage: u32) -> String {
    let durability = Durability::new(record.stats.durability);
    let broken = if durability.is_broken(damage) {
        " broken"
    } else {
        ""
    };
    format!(
        "durability={}/{} harvest_speed={} enchantment_value={} tier={}{broken}",
        durability.remaining(damage),
        durability.max(),
        record.stats.harvest_speed,
        record.stats.enchantment_value,
        record.stats.harvest_tier.name()
    )
}

/// Whether the slice's fixed item components can honour `record`.
pub fn supported(record: &GearRecord) -> Result<(), AssembleError> {
    let stats = record.stats;
    let same = record.gear_type == PICKAXE
        && stats.durability == ITEM_STATS.durability
        && (stats.harvest_speed - ITEM_STATS.harvest_speed).abs() < 0.01
        && stats.enchantment_value == ITEM_STATS.enchantment_value
        && stats.harvest_tier == ITEM_STATS.harvest_tier;
    if same {
        return Ok(());
    }
    Err(AssembleError::Unsupported(vec![
        format!("durability {}", stats.durability),
        format!("harvest_speed {}", stats.harvest_speed),
        format!("enchantment_value {}", stats.enchantment_value),
        format!("tier {}", stats.harvest_tier.name()),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(head: &str, rod: &str) -> Vec<(PartType, Material)> {
        vec![
            (PartType::Main, material(head).expect("head material")),
            (PartType::Rod, material(rod).expect("rod material")),
        ]
    }

    #[test]
    fn weighted_average_uses_upstream_weights() {
        let folded = fold(0.0, &[Modifier::average(2.0), Modifier::average(8.0)]);
        assert!(
            (folded - 6.125).abs() < 1e-4,
            "weighted average was {folded}"
        );
    }

    #[test]
    fn operations_fold_in_upstream_order() {
        let folded = fold(
            0.0,
            &[
                Modifier::average(10.0),
                Modifier::new(0.5, Operation::MultiplyBase),
                Modifier::new(0.2, Operation::MultiplyTotal),
                Modifier::new(2.0, Operation::Add),
                Modifier::new(30.0, Operation::Max),
            ],
        );
        // base 0 -> avg 10 -> max 30 -> +0.5*30 = 45 -> *1.2 = 54 -> +2 = 56
        assert!((folded - 56.0).abs() < 1e-4, "fold was {folded}");
    }

    #[test]
    fn a_fold_never_goes_negative() {
        assert_eq!(fold(0.0, &[Modifier::new(-50.0, Operation::Add)]), 0.0);
    }

    #[test]
    fn an_iron_head_and_a_wood_rod_make_the_iron_pickaxe() {
        let stats = assemble(&parts("iron", "wood")).expect("iron is buildable");
        assert_eq!(stats.durability, 250);
        assert!((stats.harvest_speed - 6.0).abs() < 1e-4);
        assert_eq!(stats.enchantment_value, 14);
        assert_eq!(stats.harvest_tier, HarvestTier::Iron);
        assert_eq!(stats, ITEM_STATS);
    }

    #[test]
    fn a_diamond_head_follows_the_jar_numbers() {
        let stats = assemble(&parts("diamond", "wood")).expect("diamond is a material");
        assert_eq!(stats.durability, 1561);
        assert!((stats.harvest_speed - 8.0).abs() < 1e-4);
        assert_eq!(stats.enchantment_value, 10);
        assert_eq!(stats.harvest_tier, HarvestTier::Diamond);
    }

    #[test]
    fn a_diamond_rod_multiplies_the_harvest_speed() {
        let stats = assemble(&parts("iron", "diamond")).expect("diamond rod");
        assert!((stats.harvest_speed - 7.2).abs() < 1e-4, "{stats:?}");
    }

    #[test]
    fn a_gold_rod_adds_three_speed() {
        let stats = assemble(&parts("iron", "gold")).expect("gold rod");
        assert!((stats.harvest_speed - 9.0).abs() < 1e-4, "{stats:?}");
    }

    #[test]
    fn a_missing_rod_is_refused() {
        let only_head = [(PartType::Main, material("iron").expect("iron"))];
        assert_eq!(
            assemble(&only_head),
            Err(AssembleError::MissingPart(PartType::Rod))
        );
    }

    #[test]
    fn a_missing_head_is_refused() {
        let only_rod = [(PartType::Rod, material("wood").expect("wood"))];
        assert_eq!(
            assemble(&only_rod),
            Err(AssembleError::MissingPart(PartType::Main))
        );
    }

    #[test]
    fn a_repeated_part_is_refused() {
        let both_heads = [
            (PartType::Main, material("iron").expect("iron")),
            (PartType::Main, material("stone").expect("stone")),
            (PartType::Rod, material("wood").expect("wood")),
        ];
        assert_eq!(
            assemble(&both_heads),
            Err(AssembleError::DuplicatePart(PartType::Main))
        );
    }

    #[test]
    fn a_material_with_no_main_properties_is_refused() {
        let netherite = [(PartType::Main, material("netherite").expect("netherite"))];
        assert_eq!(
            assemble(&netherite),
            Err(AssembleError::NotGearMaterial("netherite".to_owned()))
        );
    }

    #[test]
    fn the_supported_guard_accepts_iron_and_refuses_diamond() {
        let iron = assemble(&parts("iron", "wood")).expect("iron");
        let record = GearRecord {
            gear_type: PICKAXE.to_owned(),
            parts: vec![
                (PartType::Main, "silentgear:iron".to_owned()),
                (PartType::Rod, "silentgear:wood".to_owned()),
            ],
            stats: iron,
        };
        assert_eq!(supported(&record), Ok(()));

        let diamond = assemble(&parts("diamond", "wood")).expect("diamond");
        let record = GearRecord {
            stats: diamond,
            ..record
        };
        let refused = supported(&record).expect_err("diamond needs per-stack tool stats");
        assert!(
            matches!(refused, AssembleError::Unsupported(_)),
            "{refused:?}"
        );
    }

    #[test]
    fn durability_counts_down_and_breaks_at_the_maximum() {
        let durability = Durability::new(250);
        assert_eq!(durability.remaining(0), 250);
        assert_eq!(durability.remaining(1), 249);
        assert_eq!(durability.remaining(250), 0);
        assert!(!durability.is_broken(249));
        assert!(durability.is_broken(250));
        assert!(durability.is_broken(u32::MAX));
    }

    #[test]
    fn a_gear_record_round_trips_through_its_component() {
        let record = GearRecord {
            gear_type: PICKAXE.to_owned(),
            parts: vec![
                (PartType::Main, "silentgear:iron".to_owned()),
                (PartType::Rod, "silentgear:wood".to_owned()),
            ],
            stats: ITEM_STATS,
        };
        let fields = record.encode();
        assert_eq!(GearRecord::decode(&fields), Ok(record));
    }

    #[test]
    fn a_record_without_parts_is_refused() {
        let fields = vec![
            ("gear_type".to_owned(), Value::Str(PICKAXE.to_owned())),
            ("parts".to_owned(), Value::List(Vec::new())),
            ("durability".to_owned(), Value::Int(250)),
            ("harvest_speed".to_owned(), Value::Double(6.0)),
            ("enchantment_value".to_owned(), Value::Int(14)),
            ("harvest_tier".to_owned(), Value::Str("iron".to_owned())),
        ];
        assert_eq!(
            GearRecord::decode(&fields),
            Err(DecodeError::Missing("parts"))
        );
    }

    #[test]
    fn a_record_with_an_unknown_part_type_is_refused() {
        let fields = vec![
            ("gear_type".to_owned(), Value::Str(PICKAXE.to_owned())),
            (
                "parts".to_owned(),
                Value::List(vec![Value::Compound(vec![
                    ("part".to_owned(), Value::Str("silentgear:tip".to_owned())),
                    (
                        "material".to_owned(),
                        Value::Str("silentgear:iron".to_owned()),
                    ),
                ])]),
            ),
            ("durability".to_owned(), Value::Int(250)),
            ("harvest_speed".to_owned(), Value::Double(6.0)),
            ("enchantment_value".to_owned(), Value::Int(14)),
            ("harvest_tier".to_owned(), Value::Str("iron".to_owned())),
        ];
        assert_eq!(
            GearRecord::decode(&fields),
            Err(DecodeError::UnknownPart("silentgear:tip".to_owned()))
        );
    }

    #[test]
    fn assemble_parses_short_and_namespaced_materials() {
        assert_eq!(
            parse("/silentgear assemble pickaxe iron wood"),
            Action::Assemble {
                gear: "pickaxe".to_owned(),
                head: "iron".to_owned(),
                rod: "wood".to_owned(),
            }
        );
        assert_eq!(
            parse("silentgear assemble pickaxe silentgear:iron silentgear:wood"),
            Action::Assemble {
                gear: "pickaxe".to_owned(),
                head: "silentgear:iron".to_owned(),
                rod: "silentgear:wood".to_owned(),
            }
        );
    }

    #[test]
    fn stats_and_help_parse() {
        assert_eq!(parse("/silentgear stats"), Action::Stats);
        assert_eq!(parse("/silentgear help"), Action::Help);
        assert_eq!(parse("/silentgear"), Action::Help);
        assert_eq!(parse("/sg stats"), Action::Stats);
    }

    #[test]
    fn an_unknown_command_is_unknown() {
        assert_eq!(parse("/silentgear dance"), Action::Unknown);
        assert_eq!(parse("/othermod stats"), Action::Unknown);
    }

    #[test]
    fn the_stats_line_reports_every_stat() {
        let record = GearRecord {
            gear_type: PICKAXE.to_owned(),
            parts: vec![
                (PartType::Main, "silentgear:iron".to_owned()),
                (PartType::Rod, "silentgear:wood".to_owned()),
            ],
            stats: ITEM_STATS,
        };
        assert_eq!(
            stats_line_with_damage(&record, 0),
            "durability=250/250 harvest_speed=6 enchantment_value=14 tier=iron"
        );
    }

    #[test]
    fn the_stats_line_shows_damage() {
        let record = GearRecord {
            gear_type: PICKAXE.to_owned(),
            parts: vec![(PartType::Main, "silentgear:iron".to_owned())],
            stats: ITEM_STATS,
        };
        assert_eq!(
            stats_line_with_damage(&record, 1),
            "durability=249/250 harvest_speed=6 enchantment_value=14 tier=iron"
        );
    }
}
