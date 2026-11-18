use unity::prelude::*;

// Layouts and offsets from Engage 2.0.0 (main_200), AArch64.
// The hook leaves all other weapons and all non-immunity checks untouched.
#[unity::class("App", "BattleInfoSide")]
struct BattleInfoSide {}

#[unity::class("App", "UnitItem")]
struct UnitItem {}

const BREAK_IMMUNITY_MASK: i32 = 0x0c00;
const ALREADY_BROKEN: i32 = 0x4000;

pub fn install() {
    skyline::install_hooks!(can_breakable);
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
        && status_bits & ALREADY_BROKEN == 0
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
