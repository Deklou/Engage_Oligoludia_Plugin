use unity::prelude::*;

// Layouts and offsets from Engage 2.0.0 (main_200), AArch64.
// The hooks preserve all vanilla behavior except Oligoludia's Break immunity bypass.
#[unity::class("App", "BattleInfoSide")]
struct BattleInfoSide {}

#[unity::class("App", "UnitItem")]
struct UnitItem {}

const BREAK_IMMUNITY_MASK: i32 = 0x0c00;
const BREAK_PROTECTION_STATUS: i32 = 0x4000;
const BREAK_RESULT: i32 = 0x20;
const BREAK_ALREADY_APPLIED: i32 = 0x20000000;

pub fn install() {
    skyline::install_hooks!(can_breakable, get_breaked);
}

#[skyline::hook(offset = 0x01E89B00)]
fn can_breakable(side: &BattleInfoSide, method: OptionalMethod) -> bool {
    // Keep every vanilla positive result as-is.
    if call_original!(side, method) {
        return true;
    }

    // A vanilla negative result may be overridden only when its blocker was
    // the target's Break immunity and the equipped weapon is Oligoludia.
    unsafe { oligoludia_can_break_immune_target(side) }
}

unsafe fn oligoludia_can_break_immune_target(side: &BattleInfoSide) -> bool {
    let current = side as *const BattleInfoSide as *const u8;
    let reverse = field_ptr::<BattleInfoSide>(current, 0x78);
    if reverse.is_null() {
        return false;
    }
    let target = reverse as *const u8;

    // Match the two valid-unit checks in the original CanBreakable.
    if field_ptr::<u8>(current, 0x20).is_null()
        || field_ptr::<u8>(target, 0x20).is_null()
    {
        return false;
    }

    let current_item = battle_side_item(side, None);
    let reverse_item = battle_side_item(&*reverse, None);
    if current_item.is_null() || reverse_item.is_null() {
        return false;
    }

    // No index check here: vanilla checks that UnitItem itself is non-null,
    // not that the selected item slot has a nonzero index.
    if !is_oligoludia(current_item) {
        return false;
    }

    let mask_skills = field_ptr::<u8>(target, 0x88);
    let status = field_ptr::<u8>(target, 0x50);
    if mask_skills.is_null() || status.is_null() {
        return false;
    }

    let bad_ignore = field_i32(mask_skills, 0x3c);
    let bad_states = field_i32(mask_skills, 0x38);
    let status_bits = field_i32(status, 0x10);
    let interact = get_interact(current_item, reverse_item, None);
    let can_break = bad_ignore & BREAK_IMMUNITY_MASK != 0
        && status_bits & BREAK_ALREADY_APPLIED == 0
        && interact == 1;

    // Diagnostics for this weapon only, capped to avoid flooding Ryujinx.
    use core::sync::atomic::{AtomicUsize, Ordering};
    static LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
    if LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 64 {
        println!(
            "[oligoludia_plugin] Break check: item_index={} target_index={} bad_ignore={:#x} bad_states={:#x} status={:#x} interact={} override={}",
            field_i32(current_item as *const u8, 0x10),
            field_i32(reverse_item as *const u8, 0x10),
            bad_ignore,
            bad_states,
            status_bits,
            interact,
            can_break
        );
    }
    can_break
}

// A target with BattleInfoSide.Status 0x4000 passes through a second
// native gate. GetBreaked returns 0x4000 instead of the actual Break result
// (0x20). CanBreakable alone cannot bypass this protection.
#[unity::class("App", "BattleCalculator")]
struct BattleCalculator {}

#[skyline::hook(offset = 0x0246DB90)]
fn get_breaked(
    calculator: &BattleCalculator,
    target: &BattleInfoSide,
    method: OptionalMethod,
) -> i32 {
    let vanilla = call_original!(calculator, target, method);
    if vanilla != BREAK_PROTECTION_STATUS {
        return vanilla;
    }

    unsafe {
        let target_data = target as *const BattleInfoSide as *const u8;
        let attacker = field_ptr::<BattleInfoSide>(target_data, 0x78);
        if attacker.is_null() {
            return vanilla;
        }

        let target_skills = field_ptr::<u8>(target_data, 0x88);
        if target_skills.is_null()
            || (field_i32(target_skills, 0x3c) & BREAK_IMMUNITY_MASK) == 0
        {
            return vanilla;
        }

        let attacker_data = attacker as *const u8;
        if field_ptr::<u8>(attacker_data, 0x20).is_null()
            || field_ptr::<u8>(target_data, 0x20).is_null()
        {
            return vanilla;
        }
        let attacker_item = battle_side_item(&*attacker, None);
        let defender_item = battle_side_item(target, None);
        if attacker_item.is_null()
            || defender_item.is_null()
            || !is_oligoludia(attacker_item)
            || get_interact(attacker_item, defender_item, None) != 1
        {
            return vanilla;
        }

        // Preserve the native stun/already-broken/battle-flag checks:
        // they were checked by call_original before it returned 0x4000.
        use core::sync::atomic::{AtomicUsize, Ordering};
        static LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
        if LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 12 {
            println!(
                "[oligoludia_plugin] GetBreaked: blocked={:#x} -> break={:#x}",
                vanilla, BREAK_RESULT
            );
        }
        BREAK_RESULT
    }
}

unsafe fn is_oligoludia(unit_item: *const UnitItem) -> bool {
    // UnitItem.m_Item @ 0x18; ItemData.Iid @ 0x20.
    let item_data = field_ptr::<u8>(unit_item as *const u8, 0x18);
    if item_data.is_null() {
        return false;
    }
    let iid = field_ptr::<u8>(item_data, 0x20);
    if iid.is_null() {
        return false;
    }
    il2cpp_string_eq(iid, "IID_リュール_オリゴルディア")
        || il2cpp_string_eq(iid, "IID_リュール_オリゴルディア_通常")
}

unsafe fn il2cpp_string_eq(il2cpp_string: *const u8, expected: &str) -> bool {
    // System.String: length @ 0x10, UTF-16 characters @ 0x14.
    let length = field_i32(il2cpp_string, 0x10);
    if !(0..=96).contains(&length) {
        return false;
    }
    let utf16 = core::slice::from_raw_parts(
        il2cpp_string.add(0x14) as *const u16,
        length as usize,
    );
    utf16.iter().copied().eq(expected.encode_utf16())
}

unsafe fn field_ptr<T>(object: *const u8, offset: usize) -> *const T {
    (object.add(offset) as *const *const T).read()
}

unsafe fn field_i32(object: *const u8, offset: usize) -> i32 {
    (object.add(offset) as *const i32).read()
}

// BattleInfoSide.get_UnitItem respects m_SpecifiedItem over m_UnitItem.
#[skyline::from_offset(0x01E74CD0)]
fn battle_side_item(
    side: &BattleInfoSide,
    method: OptionalMethod,
) -> *const UnitItem;

// InteractData.GetInteract: 1 = attacker advantage, 2 = defender advantage,
// 0 = neutral. Its favorable branch is identical to CanBreakable's.
#[skyline::from_offset(0x02917ED0)]
fn get_interact(
    current: *const UnitItem,
    reverse: *const UnitItem,
    method: OptionalMethod,
) -> i32;
