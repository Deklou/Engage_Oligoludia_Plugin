use unity::prelude::*;

// Engage 2.0.0 (main_200).
// This patch changes only the efficacy multiplier computed for Oligoludia;
// it never modifies a target's immunity mask or any other weapon.
#[unity::class("App", "BattleDetail")]
struct BattleDetail {}

#[unity::class("App", "BattleInfoSide")]
struct BattleInfoSide {}

#[unity::class("App", "UnitItem")]
struct UnitItem {}

const FELL_DRAGON: u32 = 0x20; // SID_邪竜特効 (EfficacyValue=2)
const ABERRATION: u32 = 0x40; // SID_異形特効 (EfficacyValue=3)
const ELIGIBLE: u32 = FELL_DRAGON | ABERRATION;

pub fn install() {
    skyline::install_hooks!(calc_attack);
}

#[skyline::hook(offset = 0x01E74400)]
fn calc_attack(
    detail: &BattleDetail,
    attacker: &BattleInfoSide,
    defender: &BattleInfoSide,
    method: OptionalMethod,
) {
    call_original!(detail, attacker, defender, method);
    unsafe { restore_oligoludia_efficacy(detail, attacker, defender) }
}

unsafe fn restore_oligoludia_efficacy(
    detail: &BattleDetail,
    attacker: &BattleInfoSide,
    defender: &BattleInfoSide,
) {
    let item = battle_side_item(attacker, None);
    if item.is_null() || !is_oligoludia(item) {
        return;
    }

    let attacker_side = attacker as *const BattleInfoSide as *const u8;
    let defender_side = defender as *const BattleInfoSide as *const u8;
    let attacker_unit = field_ptr::<u8>(attacker_side, 0x20);
    let defender_unit = field_ptr::<u8>(defender_side, 0x20);
    if attacker_unit.is_null() || defender_unit.is_null() {
        return;
    }

    // Mirror BattleDetail.CalcAttack: use Unit.m_MaskSkill, not
    // BattleInfoSide.m_MaskSkill, for the efficacy calculation.
    let attacker_skills = field_ptr::<u8>(attacker_unit, 0xf0);
    let defender_skills = field_ptr::<u8>(defender_unit, 0xf0);
    let person = field_ptr::<u8>(defender_unit, 0x40);
    let job = field_ptr::<u8>(defender_unit, 0x48);
    if attacker_skills.is_null()
        || defender_skills.is_null()
        || person.is_null()
        || job.is_null()
    {
        return;
    }

    let target_attrs = (field_i32(person, 0x138) | field_i32(job, 0x120)) as u32;
    let attacker_efficacies = field_i32(attacker_skills, 0x30) as u32;
    let target_immunities = field_i32(defender_skills, 0x34) as u32;

    // Only restore efficacy that is (a) actually possessed by the attacker,
    // (b) relevant to the target's attributes, and (c) blocked by immunity.
    // Unrelated efficacy types continue to obey all native immunities.
    let suppressed = attacker_efficacies & target_attrs & target_immunities & ELIGIBLE;
    if suppressed == 0 {
        return;
    }

    let detail_ptr = detail as *const BattleDetail as *const u8;
    let params = field_ptr::<u8>(detail_ptr, 0x18);
    if params.is_null() || field_usize(params, 0x18) <= 10 {
        return;
    }

    // CalcAttack writes the weapon's efficacy multiplier at BaseParams[10].
    // Preserve the native result if it already has a higher multiplier.
    let multiplier = params.add(0x20 + 10 * 4) as *mut i32;
    let original = multiplier.read();
    let restored = if (suppressed & ABERRATION) != 0 { 3 } else { 2 };
    let result = original.max(restored);
    if result == original {
        return;
    }
    multiplier.write(result);

    use core::sync::atomic::{AtomicUsize, Ordering};
    static LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
    if LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 24 {
        println!(
            "[oligoludia_plugin] Efficacy: attrs={:#x} skills={:#x} ignore={:#x} suppressed={:#x} multiplier={} -> {}",
            target_attrs, attacker_efficacies, target_immunities,
            suppressed, original, result
        );
    }
}

unsafe fn is_oligoludia(unit_item: *const UnitItem) -> bool {
    let item_data = field_ptr::<u8>(unit_item as *const u8, 0x18);
    if item_data.is_null() {
        return false;
    }
    let iid = field_ptr::<u8>(item_data, 0x20);
    !iid.is_null()
        && (il2cpp_string_eq(iid, "IID_リュール_オリゴルディア")
            || il2cpp_string_eq(iid, "IID_リュール_オリゴルディア_通常"))
}

unsafe fn il2cpp_string_eq(s: *const u8, expected: &str) -> bool {
    let length = field_i32(s, 0x10);
    if !(0..=96).contains(&length) {
        return false;
    }
    let utf16 = core::slice::from_raw_parts(s.add(0x14) as *const u16, length as usize);
    utf16.iter().copied().eq(expected.encode_utf16())
}

unsafe fn field_ptr<T>(object: *const u8, offset: usize) -> *const T {
    (object.add(offset) as *const *const T).read()
}

unsafe fn field_i32(object: *const u8, offset: usize) -> i32 {
    (object.add(offset) as *const i32).read()
}

unsafe fn field_usize(object: *const u8, offset: usize) -> usize {
    (object.add(offset) as *const usize).read()
}

#[skyline::from_offset(0x01E74CD0)]
fn battle_side_item(
    side: &BattleInfoSide,
    method: OptionalMethod,
) -> *const UnitItem;
